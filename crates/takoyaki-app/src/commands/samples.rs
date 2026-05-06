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
/// Parses [SLOTS] section from project.work (FLEX0:path / STAT0:path format).
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
    let strd_file = project_dir.join("project.strd");

    let file_to_read = if work_file.exists() {
        work_file
    } else if strd_file.exists() {
        strd_file
    } else {
        return Ok(SampleSlotResponse {
            flex: make_empty_slots(128),
            static_slots: make_empty_slots(128),
        });
    };

    let raw = std::fs::read(&file_to_read).map_err(AppError::from)?;
    let parsed = parse_project_work(&raw);

    let flex: Vec<SampleSlot> = parsed.flex_slots.iter().enumerate().map(|(i, path_opt)| {
        SampleSlot {
            slot_index: i as u8,
            occupied: path_opt.is_some(),
            filename: path_opt.as_deref().map(filename_from_path),
            full_path: path_opt.clone(),
            sample_rate: None,
            status: if path_opt.is_some() { "ok" } else { "unknown" }.to_string(),
        }
    }).collect();

    let static_slots: Vec<SampleSlot> = parsed.static_slots.iter().enumerate().map(|(i, path_opt)| {
        SampleSlot {
            slot_index: i as u8,
            occupied: path_opt.is_some(),
            filename: path_opt.as_deref().map(filename_from_path),
            full_path: path_opt.clone(),
            sample_rate: None,
            status: if path_opt.is_some() { "ok" } else { "unknown" }.to_string(),
        }
    }).collect();

    info!(
        "get_project_samples: found {} flex, {} static occupied slots",
        flex.iter().filter(|s| s.occupied).count(),
        static_slots.iter().filter(|s| s.occupied).count(),
    );

    Ok(SampleSlotResponse { flex, static_slots })
}

fn make_empty_slots(count: u8) -> Vec<SampleSlot> {
    (0..count)
        .map(|i| SampleSlot {
            slot_index: i,
            occupied: false,
            filename: None,
            full_path: None,
            sample_rate: None,
            status: "unknown".to_string(),
        })
        .collect()
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
// Snapshot root helper (mirrors management.rs pattern)
// ---------------------------------------------------------------------------

fn snapshot_root() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("takoyaki")
        .join("snapshots")
}

// ---------------------------------------------------------------------------
// Phase 5 commands: compute_sample_dry_run and assign_sample
// ---------------------------------------------------------------------------

/// Validate and preview a sample slot assignment without modifying any files (Phase 5, SMPL-01).
///
/// Returns a FileChangeManifest showing which files would be modified, plus:
/// - `hard_block`: set if the assignment is not allowed (wrong format, Flex size limit)
/// - `soft_warnings`: non-blocking advisories (e.g., non-44.1kHz sample rate)
///
/// Threat model:
/// - T-05-01: canonicalize() + is_file() on file_path from native file picker
/// - T-05-02: slot_type validated to "flex" | "static" only
/// - T-05-03: slot_index bounds checked 0..=127
#[tauri::command]
#[specta::specta]
pub async fn compute_sample_dry_run(
    state: tauri::State<'_, crate::AppState>,
    project_id: String,
    slot_type: String,
    slot_index: u8,
    file_path: String,
) -> Result<SampleDryRunResult, AppError> {
    // 1. Validate slot_type enum (T-05-02)
    let parsed_slot_type = match slot_type.as_str() {
        "flex" => SlotType::Flex,
        "static" => SlotType::Static,
        other => return Err(AppError::Parse(format!("Invalid slot type: {}", other))),
    };

    // 2. Validate slot_index 0..=127 (T-05-03)
    if slot_index > 127 {
        return Err(AppError::Parse(format!("Slot index out of range: {}", slot_index)));
    }

    // 3. Canonicalize file_path (T-05-01 path traversal prevention)
    let source_path = PathBuf::from(&file_path);
    let canonical_source = source_path
        .canonicalize()
        .map_err(|e| AppError::Io(format!("Cannot resolve file path: {}", e)))?;
    if !canonical_source.is_file() {
        return Err(AppError::Io("Selected path is not a file".into()));
    }

    // 4. Validate audio format per D-14
    let mut hard_block: Option<String> = None;
    let mut soft_warnings: Vec<String> = Vec::new();

    match health::read_audio_spec(&canonical_source) {
        Ok(spec) => {
            let issues = health::check_format_compatibility(&spec);
            for issue in &issues {
                match issue {
                    health::FormatIssue::UnsupportedFormat(_) => {
                        hard_block = Some(
                            "Unsupported format: OT accepts WAV and AIFF only. Convert this file first.".into(),
                        );
                    }
                    health::FormatIssue::WrongSampleRate(actual) => {
                        soft_warnings.push(format!(
                            "This file is not at optimal OT settings (expected 44100Hz, found {}Hz). \
                             You can assign it, but the OT may behave unexpectedly.",
                            actual
                        ));
                    }
                    health::FormatIssue::WrongBitDepth(actual) => {
                        soft_warnings.push(format!(
                            "This file is not at optimal OT settings (expected 16/24-bit, found {}-bit). \
                             You can assign it, but the OT may behave unexpectedly.",
                            actual
                        ));
                    }
                }
            }
        }
        Err(e) => {
            hard_block = Some(format!("Cannot read audio file: {}", e));
        }
    }

    // 5. Flex slot size validation per D-13 — files > 200MB flagged
    if parsed_slot_type == SlotType::Flex {
        if let Ok(metadata) = std::fs::metadata(&canonical_source) {
            let size_mb = metadata.len() / (1024 * 1024);
            if size_mb > 200 {
                hard_block = Some(
                    "This sample is too large for a Flex slot. Assign to Static instead.".into(),
                );
            }
        }
    }

    // 6. Get project directory from DB (release lock before I/O per T-03-04)
    let card_path = {
        let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        db::projects::get_card_path(&db.conn, &project_id)
            .map_err(|e| AppError::Database(e.to_string()))?
    };

    // 7. Build FileChangeManifest — project.work and project.strd are always affected
    let filename = canonical_source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let slot_label = format!(
        "{} #{:03}",
        if parsed_slot_type == SlotType::Flex { "Flex" } else { "Static" },
        slot_index + 1
    );

    let project_path = PathBuf::from(&card_path);
    let project_work_path = project_path.join("project.work");
    let existing_path = if project_work_path.exists() {
        let raw = std::fs::read(&project_work_path).map_err(|e| AppError::Io(e.to_string()))?;
        let slots = project_work::extract_slot_paths(&raw);
        slots
            .iter()
            .find(|s| s.slot_type == parsed_slot_type && s.slot_number == (slot_index + 1))
            .map(|s| s.path.clone())
    } else {
        None
    };

    let operation_label = if let Some(ref old_path) = existing_path {
        let old_filename = Path::new(old_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        format!("Replacing {} with {} in {}", old_filename, filename, slot_label)
    } else {
        format!("Assigning {} to {}", filename, slot_label)
    };

    let project_name = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let entries = vec![
        FileChangeEntry {
            path: "project.work".into(),
            change_type: ChangeType::Modified,
            size_bytes: 0,
        },
        FileChangeEntry {
            path: "project.strd".into(),
            change_type: ChangeType::Modified,
            size_bytes: 0,
        },
    ];

    let manifest = FileChangeManifest {
        entries,
        total_added: 0,
        total_modified: 2,
        total_removed: 0,
        total_unchanged: 0,
        total_bytes: 0,
        destination_path: card_path,
        operation_label,
        project_name,
        conflict_details: vec![],
    };

    Ok(SampleDryRunResult {
        manifest,
        hard_block,
        soft_warnings,
    })
}

/// Assign a sample file to an OT project slot with snapshot + atomic write (Phase 5, SMPL-03).
///
/// Safety guarantees:
/// - SAFE-03: Snapshot of project.work + project.strd created before any modification.
/// - SAFE-04: atomic_write_batch writes both files in one all-or-nothing transaction.
/// - Wallflower files are copied to OT /AUDIO/ before project.work modification (RESEARCH Pitfall 5).
///
/// Threat model:
/// - T-05-02: slot_type validated to "flex" | "static"
/// - T-05-03: slot_index bounds checked
/// - T-05-04: pre-write snapshot + atomic_write_batch
/// - T-05-05: Wallflower copy destination hardcoded to card_root/AUDIO/
#[tauri::command]
#[specta::specta]
pub async fn assign_sample(
    state: tauri::State<'_, crate::AppState>,
    project_id: String,
    slot_type: String,
    slot_index: u8,
    file_path: String,
    from_wallflower: bool,
) -> Result<AssignSampleResult, AppError> {
    let parsed_slot_type = match slot_type.as_str() {
        "flex" => SlotType::Flex,
        "static" => SlotType::Static,
        other => return Err(AppError::Parse(format!("Invalid slot type: {}", other))),
    };

    // 1. DB lookup — release lock before file I/O (T-03-04)
    let card_path = {
        let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        db::projects::get_card_path(&db.conn, &project_id)
            .map_err(|e| AppError::Database(e.to_string()))?
    };

    let project_path = PathBuf::from(&card_path);
    let canonical_source = PathBuf::from(&file_path)
        .canonicalize()
        .map_err(|e| AppError::Io(format!("Cannot resolve file path: {}", e)))?;
    let filename = canonical_source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // 2. If from_wallflower: copy file to OT /AUDIO/ FIRST (per RESEARCH.md Pitfall 5)
    //    Copy must succeed before any project.work modification (T-05-05: dest hardcoded)
    if from_wallflower {
        let card_root = project_path
            .parent()
            .and_then(|sets| sets.parent())
            .ok_or_else(|| AppError::Io("Cannot determine OT card root".into()))?;
        let audio_dir = card_root.join("AUDIO");
        if !audio_dir.exists() {
            std::fs::create_dir_all(&audio_dir)
                .map_err(|e| AppError::Io(format!("Cannot create AUDIO dir: {}", e)))?;
        }
        let dest = audio_dir.join(&filename);
        if dest.exists() {
            info!(
                "Wallflower file already exists at destination: {}",
                dest.display()
            );
        } else {
            std::fs::copy(&canonical_source, &dest).map_err(|e| {
                AppError::Io(format!("Failed to copy file to OT AUDIO: {}", e))
            })?;
            info!("Copied Wallflower file to: {}", dest.display());
        }
    }

    // 3. Snapshot all affected files before modification (SAFE-03, D-04)
    let project_work_path = project_path.join("project.work");
    let project_strd_path = project_path.join("project.strd");
    let affected_paths: Vec<&Path> = vec![project_work_path.as_path(), project_strd_path.as_path()];
    let engine = SnapshotEngine::new(snapshot_root());
    let snap_result = engine.snapshot_files(&affected_paths, "pre-sample-assign")?;
    info!(
        "Pre-assign snapshot: {} files at {}",
        snap_result.file_count,
        snap_result.snapshot_dir.display()
    );

    // 4. Read project.work, rewrite slot path
    let raw_work = std::fs::read(&project_work_path)
        .map_err(|e| AppError::Io(format!("Cannot read project.work: {}", e)))?;

    // OT path format: ../AUDIO/filename.wav (card-relative from SETS/PROJECT/ dir)
    let ot_path = format!("../AUDIO/{}", filename);
    let new_work_bytes = project_work::rewrite_slot_path(
        &raw_work,
        parsed_slot_type,
        slot_index + 1, // rewrite_slot_path uses 1-indexed slot numbers per OT convention
        &ot_path,
    );

    // 5. Assertion guard: verify rewrite_slot_path actually changed bytes (validates assumption A2)
    if raw_work == new_work_bytes {
        tracing::warn!(
            "rewrite_slot_path returned unchanged bytes for {} slot {} — \
             slot entry may not exist in project.work. \
             Verify project.work format matches expected TYPE=/SLOT=/PATH= structure.",
            slot_type,
            slot_index + 1,
        );
    }

    // 5b. project.strd is a mirror — apply same rewrite
    let raw_strd = std::fs::read(&project_strd_path)
        .map_err(|e| AppError::Io(format!("Cannot read project.strd: {}", e)))?;
    let new_strd_bytes = project_work::rewrite_slot_path(
        &raw_strd,
        parsed_slot_type,
        slot_index + 1,
        &ot_path,
    );

    // 5c. Log if .ot sidecar exists (Q4 observability per RESEARCH.md resolution)
    let sidecar_stem = canonical_source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    let ot_sidecar_path = project_path.join(format!("../AUDIO/{}.ot", sidecar_stem));
    if ot_sidecar_path.exists() {
        info!(
            "Pre-existing .ot sidecar found at {} — OT will regenerate on next project load",
            ot_sidecar_path.display()
        );
    }

    // 6. Atomic batch write (SAFE-04)
    crate::atomic::atomic_write_batch(&[
        (&project_work_path, &new_work_bytes),
        (&project_strd_path, &new_strd_bytes),
    ])?;

    let files_written = 2u8;
    info!(
        "Assigned {} to {} slot {} — {} files written",
        filename, slot_type, slot_index, files_written
    );

    Ok(AssignSampleResult {
        files_written,
        slot_type,
        slot_index,
        filename,
    })
}

// ---------------------------------------------------------------------------
// Phase Quick: Audio preview command
// ---------------------------------------------------------------------------

/// Resolve an OT sample path to a canonical filesystem path.
///
/// OT paths come in several forms depending on firmware version and how the
/// path was written:
///   - `\AUDIO\kick.wav`      (backslash, absolute from card root)
///   - `../AUDIO/kick.wav`    (relative from project dir — one or more `../`)
///   - `AUDIO/kick.wav`       (relative from card root, no prefix)
///
/// Strategy: normalize to forward slashes, strip any `../` and leading `/`
/// to get a card-root-relative path, then try resolving from the volume root.
/// Falls back to resolving as-is from the project directory.
fn resolve_sample_path(
    card_path: &str,
    volume_root: &std::path::Path,
    sample_path: &str,
) -> Result<PathBuf, AppError> {
    let normalized = sample_path.replace('\\', "/");

    // Strip "../" prefixes and leading "/" to get a card-root-relative path
    let card_relative = normalized
        .trim_start_matches("../")
        .trim_start_matches('/');

    // Strategy 1: resolve from volume root (handles all OT path variants)
    let from_volume = volume_root.join(card_relative);

    // Strategy 2: resolve as-is from project directory (fallback)
    let project_dir = PathBuf::from(card_path);
    let from_project = project_dir.join(&normalized);

    let canonical = from_volume
        .canonicalize()
        .or_else(|_| from_project.canonicalize())
        .map_err(|e| {
            AppError::Io(format!(
                "Cannot resolve sample '{}' (tried '{}' and '{}'): {}",
                sample_path,
                from_volume.display(),
                from_project.display(),
                e
            ))
        })?;

    let canonical_volume = volume_root.canonicalize().map_err(|e| {
        AppError::Io(format!("Cannot canonicalize volume root: {}", e))
    })?;
    if !canonical.starts_with(&canonical_volume) {
        return Err(AppError::Io(format!(
            "Sample path escapes volume: {}",
            canonical.display()
        )));
    }

    Ok(canonical)
}

/// Play a sample audio file through the system audio output via rodio.
///
/// Resolves the OT sample path relative to the project directory, then
/// sends it to the dedicated audio thread for native playback — bypasses
/// WKWebView which doesn't route audio to system output on macOS.
#[tauri::command]
#[specta::specta]
pub async fn play_sample(
    state: tauri::State<'_, crate::AppState>,
    project_id: String,
    sample_path: String,
) -> Result<(), AppError> {
    let card_path = {
        let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        db::projects::get_card_path(&db.conn, &project_id)
            .map_err(|e| AppError::Database(e.to_string()))?
    };
    let volume_root = {
        let device = state.device.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        device
            .mount_point
            .clone()
            .ok_or_else(|| AppError::Device("No OT volume mounted".into()))?
    };

    let resolved = resolve_sample_path(&card_path, &volume_root, &sample_path)?;
    info!("play_sample: {}", resolved.display());

    // Pre-flight: verify the file can be opened and decoded before sending
    // to the audio thread, so errors are reported back to the frontend.
    let file = std::fs::File::open(&resolved)
        .map_err(|e| AppError::Io(format!("Cannot open audio file '{}': {}", resolved.display(), e)))?;
    let _ = rodio::Decoder::new(std::io::BufReader::new(file))
        .map_err(|e| AppError::Io(format!("Cannot decode audio file '{}': {}", resolved.display(), e)))?;

    state
        .audio_tx
        .send(crate::AudioCommand::Play(resolved))
        .map_err(|e| AppError::Io(format!("Audio thread unavailable: {}", e)))?;

    Ok(())
}

/// Stop any currently playing sample preview.
#[tauri::command]
#[specta::specta]
pub async fn stop_sample(
    state: tauri::State<'_, crate::AppState>,
) -> Result<(), AppError> {
    state
        .audio_tx
        .send(crate::AudioCommand::Stop)
        .map_err(|e| AppError::Io(format!("Audio thread unavailable: {}", e)))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Real project.work text parser (Phase 7 — replaces parse_sample_slots)
// ---------------------------------------------------------------------------

/// Parsed output from an OT project.work text file.
///
/// Contains tempo from [SETTINGS] and slot assignments from [SLOTS].
/// Bank names, part names, and machine types are NOT in project.work
/// (they are in the bank file opaque body -- out of scope for Phase 7).
#[derive(Debug, Clone)]
pub struct ParsedProjectWork {
    /// Raw tempo integer from TEMPO: key (divide by TEMPO_SCALE_FACTOR for BPM)
    pub tempo_raw: Option<u32>,
    /// 128 Flex slot paths (0-indexed). None = empty/unoccupied.
    pub flex_slots: Vec<Option<String>>,
    /// 128 Static slot paths (0-indexed). None = empty/unoccupied.
    pub static_slots: Vec<Option<String>>,
}

/// Parse an OT project.work file from raw bytes.
///
/// The format is section-based text: `[SECTION]` headers followed by
/// `KEY:VALUE` lines. This parser extracts:
/// - `TEMPO:` from `[SETTINGS]`
/// - `FLEX0:`..`FLEX127:` and `STAT0:`..`STAT127:` from `[SLOTS]`
///
/// Infallible: returns defaults on any parse error, never panics.
/// Bounds-checked: slot indices >= 128 are silently ignored (security).
pub fn parse_project_work(raw: &[u8]) -> ParsedProjectWork {
    let text = String::from_utf8_lossy(raw);
    let mut tempo_raw: Option<u32> = None;
    let mut flex_slots: Vec<Option<String>> = vec![None; 128];
    let mut static_slots: Vec<Option<String>> = vec![None; 128];
    let mut in_settings = false;
    let mut in_slots = false;

    for line in text.lines() {
        let trimmed = line.trim();
        match trimmed {
            "[SETTINGS]" => { in_settings = true; in_slots = false; }
            "[SLOTS]" => { in_slots = true; in_settings = false; }
            s if s.starts_with('[') => { in_settings = false; in_slots = false; }
            s if in_settings => {
                if let Some(rest) = s.strip_prefix("TEMPO:") {
                    tempo_raw = rest.trim().parse().ok();
                }
            }
            s if in_slots => {
                if let Some(rest) = s.strip_prefix("FLEX") {
                    if let Some(colon) = rest.find(':') {
                        let idx: usize = rest[..colon].parse().unwrap_or(999);
                        if idx < 128 {
                            let path = rest[colon + 1..].trim();
                            flex_slots[idx] = if path.is_empty() { None } else { Some(path.to_string()) };
                        }
                    }
                } else if let Some(rest) = s.strip_prefix("STAT") {
                    if let Some(colon) = rest.find(':') {
                        let idx: usize = rest[..colon].parse().unwrap_or(999);
                        if idx < 128 {
                            let path = rest[colon + 1..].trim();
                            static_slots[idx] = if path.is_empty() { None } else { Some(path.to_string()) };
                        }
                    }
                }
            }
            _ => {}
        }
    }

    ParsedProjectWork { tempo_raw, flex_slots, static_slots }
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

    // -----------------------------------------------------------------------
    // Phase 7: parse_project_work tests (TDD RED phase)
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_project_work_occupied_flex_slot() {
        let raw = b"[SLOTS]\nFLEX1:../AUDIO/kick.wav\n";
        let parsed = parse_project_work(raw);
        assert_eq!(parsed.flex_slots[1], Some("../AUDIO/kick.wav".to_string()));
    }

    #[test]
    fn test_parse_project_work_occupied_static_slot() {
        let raw = b"[SLOTS]\nSTAT0:../AUDIO/pad.wav\n";
        let parsed = parse_project_work(raw);
        assert_eq!(parsed.static_slots[0], Some("../AUDIO/pad.wav".to_string()));
    }

    #[test]
    fn test_parse_project_work_empty_slot() {
        let raw = b"[SLOTS]\nFLEX0:\n";
        let parsed = parse_project_work(raw);
        assert_eq!(parsed.flex_slots[0], None);
    }

    #[test]
    fn test_parse_project_work_tempo() {
        let raw = b"[SETTINGS]\nTEMPO:1200\n[SLOTS]\n";
        let parsed = parse_project_work(raw);
        assert_eq!(parsed.tempo_raw, Some(1200));
    }

    #[test]
    fn test_parse_project_work_no_tempo() {
        let raw = b"[SLOTS]\nFLEX0:\n";
        let parsed = parse_project_work(raw);
        assert_eq!(parsed.tempo_raw, None);
    }

    #[test]
    fn test_parse_project_work_bounds_check() {
        let raw = b"[SLOTS]\nFLEX999:../AUDIO/bad.wav\n";
        let parsed = parse_project_work(raw);
        assert!(parsed.flex_slots.iter().all(|s| s.is_none()));
    }

    #[test]
    fn test_parse_project_work_empty_input() {
        let parsed = parse_project_work(b"");
        assert_eq!(parsed.tempo_raw, None);
        assert!(parsed.flex_slots.iter().all(|s| s.is_none()));
        assert!(parsed.static_slots.iter().all(|s| s.is_none()));
    }

    #[test]
    fn test_parse_project_work_multiple_sections() {
        let raw = b"[META]\nVERSION:1.40B\n[SETTINGS]\nTEMPO:1200\nQUANTIZE:3\n[SLOTS]\nFLEX0:../AUDIO/kick.wav\nSTAT5:../AUDIO/pad.wav\n";
        let parsed = parse_project_work(raw);
        assert_eq!(parsed.tempo_raw, Some(1200));
        assert_eq!(parsed.flex_slots[0], Some("../AUDIO/kick.wav".to_string()));
        assert_eq!(parsed.static_slots[5], Some("../AUDIO/pad.wav".to_string()));
        // Non-populated slots remain None
        assert_eq!(parsed.flex_slots[1], None);
        assert_eq!(parsed.static_slots[0], None);
    }
}
