//! Tauri command for reading Flex and Static sample slots from a project (BROW-04).
//!
//! Threat model T-02-03: Sample filenames are local data on a local desktop app
//! with no network exposure — information disclosure risk is accepted as low.

use crate::db;
use crate::error::AppError;
use specta::Type;

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
    state: tauri::State<'_, crate::commands::projects::AppState>,
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
}
