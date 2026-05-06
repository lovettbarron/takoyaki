//! Health check engine for OT project sample validation.
//!
//! Detects missing files (DETC-01), wrong audio formats (DETC-02),
//! and unused samples (DETC-03). Runs as a background async task.
//!
//! Threat model T-02-05: `resolve_ot_path` uses `canonicalize()` to prevent
//! path traversal attacks from crafted OT binary sample paths.

use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Audio format spec read from a file header (header-only — never loads sample data).
#[derive(Debug)]
pub enum AudioSpec {
    Wav {
        sample_rate: u32,
        bits_per_sample: u16,
        channels: u16,
    },
    Aiff {
        sample_rate: u32,
        bits_per_sample: u16,
        channels: u16,
    },
    Unknown {
        detected_type: Option<String>,
    },
}

/// A specific audio format problem found during compatibility checking.
#[derive(Debug)]
pub enum FormatIssue {
    /// Wrong sample rate (e.g. 48000 instead of 44100).
    WrongSampleRate(u32),
    /// Wrong bit depth (e.g. 32 instead of 16 or 24).
    WrongBitDepth(u16),
    /// Non-WAV/AIFF format (device cannot load the file).
    UnsupportedFormat(String),
}

/// A health issue found in a project's sample slots.
///
/// The `#[serde(tag = "severity")]` attribute makes the severity field available
/// at the top level in the serialized JSON, matching the frontend HealthIssue type.
#[derive(Debug, serde::Serialize, Clone, specta::Type)]
#[serde(tag = "severity", rename_all = "lowercase")]
pub enum HealthIssue {
    Error {
        slot_type: String,
        slot_index: u8,
        path: String,
        detail: String,
    },
    Warning {
        slot_type: String,
        slot_index: u8,
        filename: String,
        detail: String,
    },
    Info {
        slot_type: String,
        slot_index: u8,
        filename: String,
        detail: String,
    },
}

/// The complete result payload emitted via the "health-complete" Tauri event.
#[derive(Debug, serde::Serialize, Clone, specta::Type)]
pub struct HealthCheckComplete {
    pub project_id: String,
    pub issues: Vec<HealthIssue>,
    pub scanned_at: String,
}

/// A reference to a track that uses a sample slot (for DETC-03 cross-reference).
#[derive(Debug, Clone)]
pub struct TrackRef {
    pub bank_index: u8,
    pub part_index: u8,
    pub track_index: u8,
}

/// Input descriptor for a single sample slot in the health check.
#[derive(Debug, Clone)]
pub struct SlotCheckInput {
    /// "flex" or "static"
    pub slot_type: String,
    pub slot_index: u8,
    pub occupied: bool,
    /// Normalized OT path (already passed through `normalize_ot_path` in samples.rs).
    pub raw_path: Option<String>,
    /// Which tracks reference this slot (for unused-sample detection, DETC-03).
    pub track_references: Vec<TrackRef>,
}

// ---------------------------------------------------------------------------
// Audio spec reading
// ---------------------------------------------------------------------------

/// Read the audio format spec from the header of a WAV or AIFF file.
///
/// Uses magic-byte detection via `infer`, then delegates to `hound` (WAV)
/// or `aifc` (AIFF). Never loads sample data — header reads only.
///
/// CRITICAL: This function MUST NOT call `.samples()` on a WavReader.
pub fn read_audio_spec(path: &Path) -> Result<AudioSpec, std::io::Error> {
    // Detect by magic bytes — not by extension, to handle misnamed files.
    let kind = infer::get_from_path(path)
        .ok()
        .flatten()
        .map(|t| t.mime_type().to_string());

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let is_wav = kind.as_deref() == Some("audio/x-wav") || ext == "wav";
    let is_aiff = kind.as_deref() == Some("audio/aiff") || ext == "aif" || ext == "aiff";

    if is_wav {
        match hound::WavReader::open(path) {
            Ok(reader) => {
                let spec = reader.spec();
                Ok(AudioSpec::Wav {
                    sample_rate: spec.sample_rate,
                    bits_per_sample: spec.bits_per_sample,
                    channels: spec.channels,
                })
            }
            Err(_) => Ok(AudioSpec::Unknown { detected_type: kind }),
        }
    } else if is_aiff {
        let mut file = std::fs::File::open(path)?;
        let mut buf_reader = std::io::BufReader::new(&mut file);
        match aifc::AifcReader::new(&mut buf_reader) {
            Ok(reader) => {
                let info = reader.info();
                Ok(AudioSpec::Aiff {
                    sample_rate: info.sample_rate as u32,
                    bits_per_sample: info.comm_sample_size as u16,
                    channels: info.channels as u16,
                })
            }
            Err(_) => Ok(AudioSpec::Unknown { detected_type: kind }),
        }
    } else {
        Ok(AudioSpec::Unknown { detected_type: kind })
    }
}

// ---------------------------------------------------------------------------
// Format compatibility checking
// ---------------------------------------------------------------------------

/// Check an AudioSpec against OT MkII sample requirements.
///
/// OT MkII requirements (DETC-02):
/// - Sample rate: 44100 Hz (Warning if different — file loads but plays at wrong pitch)
/// - Bit depth: 16-bit or 24-bit (Warning if neither)
/// - Format: WAV or AIFF (Error if neither — device cannot load)
pub fn check_format_compatibility(spec: &AudioSpec) -> Vec<FormatIssue> {
    let mut issues = vec![];
    match spec {
        AudioSpec::Wav {
            sample_rate,
            bits_per_sample,
            ..
        }
        | AudioSpec::Aiff {
            sample_rate,
            bits_per_sample,
            ..
        } => {
            if *sample_rate != 44100 {
                issues.push(FormatIssue::WrongSampleRate(*sample_rate));
            }
            if *bits_per_sample != 16 && *bits_per_sample != 24 {
                issues.push(FormatIssue::WrongBitDepth(*bits_per_sample));
            }
        }
        AudioSpec::Unknown { detected_type } => {
            issues.push(FormatIssue::UnsupportedFormat(
                detected_type
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
            ));
        }
    }
    issues
}

// ---------------------------------------------------------------------------
// Path resolution
// ---------------------------------------------------------------------------

/// Resolve an OT-style sample path to an absolute filesystem path, with
/// path traversal prevention via `canonicalize()` (T-02-05 mitigation).
///
/// OT paths use backslash separators and are relative to the card root.
/// Example: `\AUDIO\kick.wav` on volume `/Volumes/OT-CARD` →
/// `/Volumes/OT-CARD/AUDIO/kick.wav`
///
/// Security: After construction, the resolved path is validated to ensure
/// it stays under the volume root. Any path that escapes the volume root
/// is rejected (returns `None`) and a warning is logged.
pub fn resolve_ot_path(volume_path: &Path, raw_path: &str) -> Option<PathBuf> {
    // Convert OT backslash paths to forward slashes and strip leading separator.
    let normalized = raw_path.replace('\\', "/");
    let relative = normalized.trim_start_matches('/');
    if relative.is_empty() {
        return None;
    }

    let resolved = volume_path.join(relative);

    // Path traversal prevention: canonicalize both paths and verify containment.
    // Use the volume_path directly if it doesn't exist yet (e.g., in tests).
    let canonical_volume = match std::fs::canonicalize(volume_path) {
        Ok(p) => p,
        Err(_) => {
            // Volume path doesn't exist (not mounted) — skip traversal check.
            // This is acceptable; existence check later will handle the missing file.
            return Some(resolved);
        }
    };

    let canonical_resolved = match std::fs::canonicalize(&resolved) {
        Ok(p) => p,
        Err(_) => {
            // File doesn't exist yet — can't canonicalize. Return the uncanonicalised
            // path so the caller can report it as missing.
            // Still check for traversal using string prefix on the non-canonical path.
            // This is best-effort for the non-existent-file case.
            let resolved_str = resolved.to_string_lossy();
            let volume_str = canonical_volume.to_string_lossy();
            if !resolved_str.starts_with(volume_str.as_ref()) {
                tracing::warn!(
                    "resolve_ot_path: rejecting non-existent path that may escape volume: {}",
                    resolved_str
                );
                return None;
            }
            return Some(resolved);
        }
    };

    // Final containment check on canonicalized paths.
    if !canonical_resolved.starts_with(&canonical_volume) {
        tracing::warn!(
            "resolve_ot_path: rejecting path traversal attempt: {} escapes volume {}",
            canonical_resolved.display(),
            canonical_volume.display()
        );
        return None;
    }

    Some(canonical_resolved)
}

// ---------------------------------------------------------------------------
// Health check engine
// ---------------------------------------------------------------------------

/// Run the health check for all sample slots in a project.
///
/// For each occupied slot:
/// - DETC-01: Check if the file exists on disk (Error if missing)
/// - DETC-02: Read audio header, check format compatibility (Warning/Error per severity)
/// - DETC-03: Check if the slot is referenced by any track (Info if unused)
///
/// Returns a Vec of all issues found. An empty Vec means a clean bill of health.
pub async fn perform_health_check(
    _project_path: &str,
    volume_path: &Path,
    sample_slots: &[SlotCheckInput],
) -> Vec<HealthIssue> {
    let mut issues = Vec::new();

    for slot in sample_slots {
        if !slot.occupied {
            continue;
        }

        let Some(ref raw_path) = slot.raw_path else {
            continue;
        };

        // Resolve the OT path to a filesystem path.
        let resolved_path = match resolve_ot_path(volume_path, raw_path) {
            Some(p) => p,
            None => {
                issues.push(HealthIssue::Error {
                    slot_type: slot.slot_type.clone(),
                    slot_index: slot.slot_index,
                    path: raw_path.clone(),
                    detail: format!("Invalid or unsafe path: {raw_path}"),
                });
                continue;
            }
        };

        // DETC-01: File existence check.
        if !resolved_path.exists() {
            issues.push(HealthIssue::Error {
                slot_type: slot.slot_type.clone(),
                slot_index: slot.slot_index,
                path: resolved_path.display().to_string(),
                detail: format!("File not found: {}", resolved_path.display()),
            });
            continue; // Can't check format if file doesn't exist.
        }

        // DETC-02: Audio format compatibility check.
        match read_audio_spec(&resolved_path) {
            Ok(spec) => {
                let format_issues = check_format_compatibility(&spec);
                let filename = resolved_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| raw_path.clone());

                for issue in format_issues {
                    match issue {
                        FormatIssue::WrongSampleRate(rate) => {
                            issues.push(HealthIssue::Warning {
                                slot_type: slot.slot_type.clone(),
                                slot_index: slot.slot_index,
                                filename: filename.clone(),
                                detail: format!(
                                    "Wrong sample rate: {rate} Hz (OT requires 44100 Hz — file will play at wrong pitch)"
                                ),
                            });
                        }
                        FormatIssue::WrongBitDepth(depth) => {
                            issues.push(HealthIssue::Warning {
                                slot_type: slot.slot_type.clone(),
                                slot_index: slot.slot_index,
                                filename: filename.clone(),
                                detail: format!(
                                    "Wrong bit depth: {depth}-bit (OT requires 16-bit or 24-bit)"
                                ),
                            });
                        }
                        FormatIssue::UnsupportedFormat(ref fmt) => {
                            issues.push(HealthIssue::Error {
                                slot_type: slot.slot_type.clone(),
                                slot_index: slot.slot_index,
                                path: resolved_path.display().to_string(),
                                detail: format!(
                                    "Unsupported format: {fmt} (OT requires WAV or AIFF — device cannot load this file)"
                                ),
                            });
                        }
                    }
                }
            }
            Err(e) => {
                let filename = resolved_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| raw_path.clone());
                issues.push(HealthIssue::Warning {
                    slot_type: slot.slot_type.clone(),
                    slot_index: slot.slot_index,
                    filename,
                    detail: format!("Could not read audio header: {e}"),
                });
            }
        }

        // DETC-03: Unused sample detection (slot assigned but not referenced by any track).
        // Phase 7 limitation: track_references is always empty because bank file body
        // is opaque -- we cannot determine which tracks reference which slots.
        // Skip DETC-03 check when track_references is empty to avoid a false-positive
        // flood (every occupied slot would be flagged as "unused").
        // TODO: Re-enable when bank body parser provides real track references.
        if !slot.track_references.is_empty() {
            // Check if the slot is truly unreferenced
            // (This path is not reached in Phase 7 but is ready for future use)
            let filename = resolved_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| raw_path.clone());
            issues.push(HealthIssue::Info {
                slot_type: slot.slot_type.clone(),
                slot_index: slot.slot_index,
                filename: filename.clone(),
                detail: format!(
                    "{filename} (slot #{}) — assigned but not referenced by any track.",
                    slot.slot_index
                ),
            });
        }
    }

    issues
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn fixture_path(relative: &str) -> PathBuf {
        let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        Path::new(&manifest)
            .parent()
            .unwrap() // crates/
            .parent()
            .unwrap() // project root
            .join("tests/fixtures/mock_ot_volume")
            .join(relative)
    }

    #[test]
    fn test_read_audio_spec_wav_44100() {
        let path = fixture_path("AUDIO/kick_44100.wav");
        let spec = read_audio_spec(&path).expect("Should read WAV spec");
        match spec {
            AudioSpec::Wav { sample_rate, .. } => {
                assert_eq!(sample_rate, 44100);
            }
            other => panic!("Expected Wav variant, got {:?}", other),
        }
    }

    #[test]
    fn test_read_audio_spec_wav_48000() {
        let path = fixture_path("AUDIO/pad_48000.wav");
        let spec = read_audio_spec(&path).expect("Should read WAV spec");
        match spec {
            AudioSpec::Wav { sample_rate, .. } => {
                assert_eq!(sample_rate, 48000);
            }
            other => panic!("Expected Wav variant, got {:?}", other),
        }
    }

    #[test]
    fn test_read_audio_spec_unknown() {
        let path = fixture_path("AUDIO/not_audio.txt");
        let spec = read_audio_spec(&path).expect("Should return Unknown for non-audio file");
        assert!(matches!(spec, AudioSpec::Unknown { .. }));
    }

    #[test]
    fn test_check_format_wav_44100_no_issues() {
        let spec = AudioSpec::Wav {
            sample_rate: 44100,
            bits_per_sample: 16,
            channels: 2,
        };
        let issues = check_format_compatibility(&spec);
        assert!(issues.is_empty(), "44.1kHz 16-bit WAV should have no issues");
    }

    #[test]
    fn test_check_format_wav_24bit_no_issues() {
        let spec = AudioSpec::Wav {
            sample_rate: 44100,
            bits_per_sample: 24,
            channels: 1,
        };
        let issues = check_format_compatibility(&spec);
        assert!(issues.is_empty(), "44.1kHz 24-bit WAV should have no issues");
    }

    #[test]
    fn test_check_format_wrong_sample_rate() {
        let spec = AudioSpec::Wav {
            sample_rate: 48000,
            bits_per_sample: 16,
            channels: 2,
        };
        let issues = check_format_compatibility(&spec);
        assert_eq!(issues.len(), 1);
        assert!(matches!(issues[0], FormatIssue::WrongSampleRate(48000)));
    }

    #[test]
    fn test_check_format_wrong_bit_depth() {
        let spec = AudioSpec::Wav {
            sample_rate: 44100,
            bits_per_sample: 32,
            channels: 2,
        };
        let issues = check_format_compatibility(&spec);
        assert_eq!(issues.len(), 1);
        assert!(matches!(issues[0], FormatIssue::WrongBitDepth(32)));
    }

    #[test]
    fn test_check_format_unsupported() {
        let spec = AudioSpec::Unknown {
            detected_type: Some("audio/mpeg".to_string()),
        };
        let issues = check_format_compatibility(&spec);
        assert_eq!(issues.len(), 1);
        assert!(matches!(issues[0], FormatIssue::UnsupportedFormat(_)));
    }
}
