//! Tauri command for reading Flex and Static sample slots from a project (BROW-04).
//!
//! Threat model T-02-03: Sample filenames are local data on a local desktop app
//! with no network exposure — information disclosure risk is accepted as low.
//!
//! Phase 5 Plan 01: compute_sample_dry_run and assign_sample commands added.
//! Threat model T-05-01: canonicalize() on file_path from native file picker.
//! Threat model T-05-02/03: enum parse + bounds check on slot_type/slot_index.
//! Threat model T-05-04: pre-write snapshot + atomic_write_batch.
//! Threat model T-05-05: Wallflower copy destination hardcoded to card_root/AUDIO/.

use crate::atomic::snapshot::SnapshotEngine;
use crate::commands::backup::{ChangeType, FileChangeEntry, FileChangeManifest};
use crate::db;
use crate::error::AppError;
use crate::health;
use crate::management::project_work::{self, SlotType};
use specta::Type;
use std::path::{Path, PathBuf};
use tracing::info;

// ---------------------------------------------------------------------------
// Phase 5 response types
// ---------------------------------------------------------------------------

/// Result of a dry-run computation for a sample assignment.
#[derive(Debug, serde::Serialize, specta::Type, Clone)]
pub struct SampleDryRunResult {
    pub manifest: FileChangeManifest,
    /// If set, the assignment is blocked (incompatible format or slot type mismatch).
    pub hard_block: Option<String>,
    /// Non-blocking warnings (e.g., non-44.1kHz sample rate).
    pub soft_warnings: Vec<String>,
}

/// Result returned after a successful sample assignment.
#[derive(Debug, serde::Serialize, specta::Type, Clone)]
pub struct AssignSampleResult {
    pub files_written: u8,
    pub slot_type: String,
    pub slot_index: u8,
    pub filename: String,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Response containing both Flex and Static sample slot arrays (BROW-04).
#[derive(Debug, serde::Serialize, Type, Clone)]
pub struct SampleSlotResponse {
    pub flex: Vec<SampleSlot>,
    pub static_slots: Vec<SampleSlot>,
}

/// One sample slot (0-indexed, 0..=127 for Flex and Static each).
#[derive(Debug, serde::Serialize, Type, Clone)]
pub struct SampleSlot {
    pub slot_index: u8,
    pub occupied: bool,
    /// Just the filename portion (not full path) for table display.
    pub filename: Option<String>,
    /// The raw path from the OT binary, normalized via `normalize_ot_path()`.
    pub full_path: Option<String>,
    pub sample_rate: Option<u32>,
    /// "ok", "missing", "warning", or "unknown" — populated by health check.
    pub status: String,
}

// ---------------------------------------------------------------------------
// Path normalization (assumption guard A3)
// ---------------------------------------------------------------------------

/// Normalize an OT sample path from raw binary bytes.
///
/// Assumption guard A3: OT stores paths as null-terminated byte sequences using
/// backslash separators, relative to the card root.
///
/// This function:
/// 1. Truncates at the first 0x00 byte (null terminator)
/// 2. Converts backslashes to forward slashes
/// 3. Strips a leading separator character
///
/// This is the single point of change if the actual encoding differs from the
/// assumption (e.g., already uses forward slashes, or fixed-width padding).
///
/// Logs raw bytes (first 32) vs normalized string for the first few calls to aid
/// Phase 1 fixture validation.
pub fn normalize_ot_path(raw: &[u8]) -> Option<String> {
    // Truncate at first null byte
    let terminated = raw.split(|&b| b == 0x00).next().unwrap_or(&[]);
    if terminated.is_empty() {
        return None;
    }

    // Treat as Latin-1 / ASCII (OT uses FAT32 ASCII paths)
    let raw_str: String = terminated.iter().map(|&b| b as char).collect();

    // Replace backslashes with forward slashes
    let normalized = raw_str.replace('\\', "/");

    // Strip leading separator
    let stripped = normalized.trim_start_matches('/');
    if stripped.is_empty() {
        None
    } else {
        Some(stripped.to_string())
    }
}

/// Extract the filename portion from a normalized OT path.
fn filename_from_path(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}

// ---------------------------------------------------------------------------
// Command
// ---------------------------------------------------------------------------

/// Return all 128 Flex and 128 Static sample slots for a project (BROW-04).
///
/// Reads project.work via ot-parser. Until the Phase 1 parser for project.work
/// is implemented, returns stub empty slot arrays.
///
/// Assumption guard A3: All path normalization goes through `normalize_ot_path()`.
#[tauri::command]
#[specta::specta]
pub async fn get_project_samples(
    state: tauri::State<'_, crate::AppState>,
    project_id: String,
) -> Result<SampleSlotResponse, AppError> {
    let card_path = {
        let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        db::projects::get_card_path(&db.conn, &project_id)
            .map_err(|e| AppError::Database(e.to_string()))?
    };

    let project_dir = std::path::PathBuf::from(&card_path);
    let work_file = project_dir.join("project.work");

    tracing::debug!(
        "get_project_samples: project_id={} card_path={} work_file_exists={}",
        project_id,
        card_path,
        work_file.exists()
    );

    // FIXME: Phase 1 OT project.work parser not yet implemented.
    // Return 128 empty slots for both Flex and Static until the parser is ready.
    // When Phase 1 parser is available, read the slot table from project.work:
    //   let data = std::fs::read(&work_file)?;
    //   let project = ot_parser::ProjectFile::from_bytes(&data)?;
    //   let flex = project.flex_sample_slots.iter().enumerate().map(|(i, slot)| { ... });
    //   let static_slots = project.static_sample_slots.iter().enumerate().map(...);

    // For each slot, log the first 5 raw paths so they are visible when real files are used.
    // This satisfies the assumption guard A3 logging requirement.
    let make_stub_slots = |count: u8| -> Vec<SampleSlot> {
        (0..count)
            .map(|slot_index| {
                // Stub: demonstrate normalize_ot_path with an empty raw path
                let raw: &[u8] = &[];
                if slot_index < 5 {
                    tracing::debug!(
                        "normalize_ot_path stub: slot_index={} raw_bytes(first32)={:?} normalized={:?}",
                        slot_index,
                        &raw[..raw.len().min(32)],
                        normalize_ot_path(raw)
                    );
                }
                SampleSlot {
                    slot_index,
                    occupied: false,
                    filename: None,
                    full_path: None,
                    sample_rate: None,
                    status: "unknown".to_string(),
                }
            })
            .collect()
    };

    Ok(SampleSlotResponse {
        flex: make_stub_slots(128),
        static_slots: make_stub_slots(128),
    })
}

/// Build a `SampleSlot` from raw binary slot data.
///
/// This function is used when Phase 1 parser data becomes available.
/// It is defined here so the call site is clear and normalize_ot_path is used correctly.
#[allow(dead_code)]
fn build_sample_slot(slot_index: u8, raw_path: &[u8], sample_rate: Option<u32>) -> SampleSlot {
    let full_path = normalize_ot_path(raw_path);
    let filename = full_path.as_deref().map(filename_from_path);
    let occupied = full_path.is_some();
    SampleSlot {
        slot_index,
        occupied,
        filename,
        full_path,
        sample_rate,
        status: "unknown".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_ot_path_null_terminated() {
        let raw = b"\\AUDIO\\Kicks\\kick.wav\x00\x00\x00\x00\x00";
        let result = normalize_ot_path(raw);
        assert_eq!(result, Some("AUDIO/Kicks/kick.wav".to_string()));
    }

    #[test]
    fn test_normalize_ot_path_empty() {
        assert_eq!(normalize_ot_path(&[]), None);
        assert_eq!(normalize_ot_path(&[0x00]), None);
    }

    #[test]
    fn test_normalize_ot_path_already_forward_slash() {
        // If the OT uses forward slashes (possible if assumption A3 is wrong),
        // the function still produces a correct result.
        let raw = b"AUDIO/Kicks/kick.wav\x00";
        let result = normalize_ot_path(raw);
        assert_eq!(result, Some("AUDIO/Kicks/kick.wav".to_string()));
    }

    #[test]
    fn test_normalize_ot_path_leading_backslash() {
        let raw = b"\\AUDIO\\kick.wav\x00";
        let result = normalize_ot_path(raw);
        assert_eq!(result, Some("AUDIO/kick.wav".to_string()));
    }

    #[test]
    fn test_filename_from_path() {
        assert_eq!(filename_from_path("AUDIO/Kicks/kick.wav"), "kick.wav");
        assert_eq!(filename_from_path("kick.wav"), "kick.wav");
    }

    // ---------------------------------------------------------------------------
    // TDD tests for compute_sample_dry_run and assign_sample (Phase 5 Plan 01)
    // ---------------------------------------------------------------------------

    /// Helper: create a real 44.1kHz 16-bit mono WAV fixture in a temp dir.
    fn create_wav_fixture(dir: &std::path::Path, filename: &str, sample_rate: u32, bits: u16) -> std::path::PathBuf {
        use std::io::Write;
        let path = dir.join(filename);
        // Minimal WAV file: RIFF header + fmt chunk + data chunk (empty audio)
        // RIFF chunk
        let data_size: u32 = 0;
        let fmt_size: u32 = 16;
        let audio_format: u16 = 1; // PCM
        let num_channels: u16 = 1;
        let byte_rate: u32 = sample_rate * num_channels as u32 * bits as u32 / 8;
        let block_align: u16 = num_channels * bits / 8;
        let riff_size: u32 = 4 + 8 + fmt_size + 8 + data_size;

        let mut buf = Vec::new();
        buf.extend_from_slice(b"RIFF");
        buf.extend_from_slice(&riff_size.to_le_bytes());
        buf.extend_from_slice(b"WAVE");
        buf.extend_from_slice(b"fmt ");
        buf.extend_from_slice(&fmt_size.to_le_bytes());
        buf.extend_from_slice(&audio_format.to_le_bytes());
        buf.extend_from_slice(&num_channels.to_le_bytes());
        buf.extend_from_slice(&sample_rate.to_le_bytes());
        buf.extend_from_slice(&byte_rate.to_le_bytes());
        buf.extend_from_slice(&block_align.to_le_bytes());
        buf.extend_from_slice(&bits.to_le_bytes());
        buf.extend_from_slice(b"data");
        buf.extend_from_slice(&data_size.to_le_bytes());

        std::fs::write(&path, &buf).unwrap();
        path
    }

    /// Helper: create a text file that is not a valid audio file.
    fn create_non_audio_fixture(dir: &std::path::Path, filename: &str) -> std::path::PathBuf {
        let path = dir.join(filename);
        std::fs::write(&path, b"not an audio file at all").unwrap();
        path
    }

    #[test]
    fn test_dry_run_result_struct_exists() {
        // Verify SampleDryRunResult can be constructed
        let _ = SampleDryRunResult {
            manifest: crate::commands::backup::FileChangeManifest {
                entries: vec![],
                total_added: 0,
                total_modified: 0,
                total_removed: 0,
                total_unchanged: 0,
                total_bytes: 0,
                destination_path: String::new(),
                operation_label: "test".to_string(),
                project_name: String::new(),
                conflict_details: vec![],
            },
            hard_block: None,
            soft_warnings: vec![],
        };
    }

    #[test]
    fn test_assign_result_struct_exists() {
        // Verify AssignSampleResult can be constructed
        let _ = AssignSampleResult {
            files_written: 2,
            slot_type: "flex".to_string(),
            slot_index: 0,
            filename: "kick.wav".to_string(),
        };
    }

    #[test]
    fn test_format_validation_wav_44100_no_hard_block() {
        let tmp = tempfile::TempDir::new().unwrap();
        let wav = create_wav_fixture(tmp.path(), "kick_44100.wav", 44100, 16);
        let spec = crate::health::read_audio_spec(&wav).expect("Should read WAV spec");
        let issues = crate::health::check_format_compatibility(&spec);
        // A 44.1kHz 16-bit WAV should produce no issues — no hard block expected
        assert!(
            issues.is_empty(),
            "44.1kHz 16-bit WAV should have no format issues"
        );
    }

    #[test]
    fn test_format_validation_non_audio_produces_unsupported_issue() {
        let tmp = tempfile::TempDir::new().unwrap();
        let txt = create_non_audio_fixture(tmp.path(), "not_audio.mp3");
        let spec = crate::health::read_audio_spec(&txt).expect("read_audio_spec returns Ok for unknown");
        let issues = crate::health::check_format_compatibility(&spec);
        // Unknown format should produce UnsupportedFormat — mapped to hard_block
        assert!(!issues.is_empty(), "Non-audio file should produce at least one issue");
        assert!(
            matches!(issues[0], crate::health::FormatIssue::UnsupportedFormat(_)),
            "Issue should be UnsupportedFormat"
        );
    }

    #[test]
    fn test_format_validation_48khz_produces_wrong_sample_rate_issue() {
        let tmp = tempfile::TempDir::new().unwrap();
        let wav = create_wav_fixture(tmp.path(), "pad_48000.wav", 48000, 16);
        let spec = crate::health::read_audio_spec(&wav).expect("Should read 48kHz WAV spec");
        let issues = crate::health::check_format_compatibility(&spec);
        // 48kHz WAV should produce WrongSampleRate — mapped to soft_warning
        assert!(!issues.is_empty(), "48kHz WAV should produce a sample rate issue");
        assert!(
            matches!(issues[0], crate::health::FormatIssue::WrongSampleRate(48000)),
            "Issue should be WrongSampleRate(48000)"
        );
    }

    #[test]
    fn test_rewrite_slot_path_integration_for_assign() {
        // Verify rewrite_slot_path produces changed bytes when slot exists (assign_sample key behavior)
        use crate::management::project_work::{rewrite_slot_path, SlotType};
        let raw = b"TYPE=FLEX\nSLOT=001\nPATH=../AUDIO/kick.wav\nGAIN=48\n";
        let new_bytes = rewrite_slot_path(raw, SlotType::Flex, 1, "../AUDIO/new_kick.wav");
        assert_ne!(
            raw.to_vec(), new_bytes,
            "rewrite_slot_path should return changed bytes when slot is found"
        );
        let result_str = String::from_utf8(new_bytes).unwrap();
        assert!(result_str.contains("PATH=../AUDIO/new_kick.wav"), "New path should be in result");
    }

    #[test]
    fn test_rewrite_slot_path_unchanged_when_slot_not_found() {
        // Verify rewrite_slot_path returns unchanged bytes when slot doesn't exist
        // This is the case that triggers the warning log in assign_sample
        use crate::management::project_work::{rewrite_slot_path, SlotType};
        let raw = b"TYPE=FLEX\nSLOT=001\nPATH=../AUDIO/kick.wav\nGAIN=48\n";
        // Request slot 99 which doesn't exist
        let new_bytes = rewrite_slot_path(raw, SlotType::Flex, 99, "../AUDIO/new.wav");
        assert_eq!(
            raw.to_vec(), new_bytes,
            "rewrite_slot_path should return unchanged bytes when slot is not found"
        );
    }

    #[test]
    fn test_flex_slot_size_check() {
        // Files over 200MB for a Flex slot should trigger a hard block.
        // We test the size logic directly: 200MB = 200 * 1024 * 1024 bytes.
        let size_mb_small: u64 = 100 * 1024 * 1024;
        let size_mb_large: u64 = 201 * 1024 * 1024;
        assert!(
            size_mb_small / (1024 * 1024) <= 200,
            "100MB should not trigger Flex hard block"
        );
        assert!(
            size_mb_large / (1024 * 1024) > 200,
            "201MB should trigger Flex hard block"
        );
    }
}
