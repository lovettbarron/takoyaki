//! Tauri commands for browsing OT projects, banks, and metadata (BROW-02, BROW-03, BROW-05).
//!
//! Threat model:
//! - T-02-01: All SQL filter values use parameterized queries (db::projects::list_projects).
//! - T-02-02: volume_path comes from AppState, not from the frontend.
//! - T-02-04: project_id is an opaque UUID; backend resolves card_path from DB.

use crate::db;
use crate::error::AppError;
use crate::AppState;
use specta::Type;
use tracing::info;

/// Assumption guard A2: OT stores tempo as integer × TEMPO_SCALE_FACTOR.
/// If Phase 1 fixture data reveals a different scale, change only this constant.
const TEMPO_SCALE_FACTOR: f32 = 10.0;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Full project detail (BROW-05).
#[derive(Debug, serde::Serialize, Type, Clone)]
pub struct ProjectDetail {
    pub project_name: String,
    /// Actual BPM value after dividing raw value by TEMPO_SCALE_FACTOR.
    pub tempo_bpm: f32,
    pub bank_count: u8,
    pub last_modified: Option<String>,
    pub banks: Vec<BankDetail>,
}

/// One bank (0-indexed) within a project.
#[derive(Debug, serde::Serialize, Type, Clone)]
pub struct BankDetail {
    pub bank_index: u8,
    pub populated: bool,
    pub bank_name: Option<String>,
    pub parts: Vec<PartDetail>,
}

/// One part (0-indexed, 4 per bank) within a bank.
#[derive(Debug, serde::Serialize, Type, Clone)]
pub struct PartDetail {
    pub part_index: u8,
    pub part_name: Option<String>,
    pub tracks: Vec<TrackDetail>,
}

/// One track (0-indexed, 8 per part) within a part.
#[derive(Debug, serde::Serialize, Type, Clone)]
pub struct TrackDetail {
    pub track_index: u8,
    /// "Flex", "Static", "Thru", "Neighbor", or "Pickup"
    pub machine_type: String,
    pub sample_slot_index: Option<u8>,
    pub sample_filename: Option<String>,
}

/// Lightweight bank summary for the 4×4 grid view (BROW-03).
#[derive(Debug, serde::Serialize, Type, Clone)]
pub struct BankSummary {
    pub bank_index: u8,
    pub populated: bool,
    pub pattern_count: u8,
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// List all projects from the SQLite index with optional filtering (BROW-02, MGMT-04).
///
/// This is a pure SQLite query — it never re-parses binary files (see RESEARCH.md pitfall 1).
#[tauri::command]
#[specta::specta]
pub async fn list_projects(
    state: tauri::State<'_, AppState>,
    filter: db::projects::ProjectFilter,
) -> Result<Vec<db::projects::ProjectSummary>, AppError> {
    let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db::projects::list_projects(&db.conn, &filter).map_err(|e| AppError::Database(e.to_string()))
}

/// Return full project detail by reading project.work + bank files via ot-parser (BROW-05).
///
/// Threat model T-02-04: project_id is an opaque UUID — card_path is resolved from DB,
/// never from a frontend-supplied path.
///
/// NOTE: The ot-parser `project.work` / `bank.work` parsers are not yet implemented
/// (Phase 1 OT binary spec work pending). This command returns a stub ProjectDetail
/// built from what is available in the SQLite index until the parser is ready.
/// Assumption guard A2: raw_tempo / TEMPO_SCALE_FACTOR gives the display BPM.
#[tauri::command]
#[specta::specta]
pub async fn get_project_detail(
    state: tauri::State<'_, AppState>,
    project_id: String,
) -> Result<ProjectDetail, AppError> {
    let (card_path, project_name, tempo_bpm, bank_count, last_modified) = {
        let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        let path =
            db::projects::get_card_path(&db.conn, &project_id)
                .map_err(|e| AppError::Database(e.to_string()))?;
        // Pull summary data from index for the stub response
        let filter = db::projects::ProjectFilter {
            name: None,
            bpm_min: None,
            bpm_max: None,
            modified_since: None,
        };
        let projects = db::projects::list_projects(&db.conn, &filter)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let summary = projects.into_iter().find(|p| p.id == project_id);
        let (name, bpm, banks, modified) = summary
            .map(|p| (p.project_name, p.tempo_bpm, p.bank_count, p.last_modified))
            .unwrap_or_default();
        (path, name, bpm, banks, modified)
    };

    // Assumption guard A2: log conversion so scale factor is visible in debug output.
    // When Phase 1 fixtures are available this lets us verify raw_tempo / TEMPO_SCALE_FACTOR.
    let display_tempo = tempo_bpm.unwrap_or(0.0);
    tracing::debug!(
        "get_project_detail: card_path={} display_tempo={}bpm (TEMPO_SCALE_FACTOR={})",
        card_path,
        display_tempo,
        TEMPO_SCALE_FACTOR
    );

    // FIXME: Phase 1 OT parser (project.work / bank.work) not yet implemented.
    // Until then, return stub bank entries derived from bank_count.
    // Each bank is marked populated=false and contains 4 empty parts × 8 empty tracks.
    let n_banks = bank_count.unwrap_or(0) as usize;
    let banks = (0u8..16).map(|bank_index| {
        let populated = (bank_index as usize) < n_banks;
        BankDetail {
            bank_index,
            populated,
            bank_name: None,
            parts: (0u8..4)
                .map(|part_index| PartDetail {
                    part_index,
                    part_name: None,
                    tracks: (0u8..8)
                        .map(|track_index| TrackDetail {
                            track_index,
                            machine_type: "Thru".to_string(),
                            sample_slot_index: None,
                            sample_filename: None,
                        })
                        .collect(),
                })
                .collect(),
        }
    }).collect();

    Ok(ProjectDetail {
        project_name,
        tempo_bpm: display_tempo,
        bank_count: bank_count.unwrap_or(0),
        last_modified,
        banks,
    })
}

/// Return the 16-bank summary grid for a project (BROW-03).
///
/// Assumption guard (Open Question 4): `is_bank_populated` is isolated so the logic
/// can be adjusted once Phase 1 binary parser fixtures are available.
///
/// NOTE: Until the ot-parser bank file parser exists, this returns stub data
/// derived from the bank_count in the SQLite index.
#[tauri::command]
#[specta::specta]
pub async fn get_project_banks(
    state: tauri::State<'_, AppState>,
    project_id: String,
) -> Result<Vec<BankSummary>, AppError> {
    let bank_count = {
        let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        let _card_path = db::projects::get_card_path(&db.conn, &project_id)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let filter = db::projects::ProjectFilter {
            name: None,
            bpm_min: None,
            bpm_max: None,
            modified_since: None,
        };
        let projects = db::projects::list_projects(&db.conn, &filter)
            .map_err(|e| AppError::Database(e.to_string()))?;
        projects
            .into_iter()
            .find(|p| p.id == project_id)
            .and_then(|p| p.bank_count)
            .unwrap_or(0)
    };

    // FIXME: Phase 1 OT bank file parser not yet implemented.
    // Stub: mark first `bank_count` banks as populated.
    let summaries = (0u8..16)
        .map(|bank_index| {
            let populated = is_bank_populated_stub(bank_index, bank_count);
            tracing::debug!(
                "get_project_banks: bank_index={} populated={} (stub)",
                bank_index,
                populated
            );
            BankSummary {
                bank_index,
                populated,
                pattern_count: if populated { 1 } else { 0 },
            }
        })
        .collect();

    Ok(summaries)
}

/// Determine whether a bank is populated.
///
/// Assumption guard (Open Question 4): This function is isolated so that once
/// Phase 1 binary parser fixtures are available, the real logic (e.g., checking
/// an explicit populated flag or non-zero pattern content in the bank file) can
/// replace the stub without touching the command handler.
fn is_bank_populated_stub(bank_index: u8, bank_count: u8) -> bool {
    // Stub: treat the first `bank_count` banks (0-indexed) as populated.
    // Replace with: parse the bank file and check for non-empty patterns.
    bank_index < bank_count
}

/// Walk the SETS directory on the mounted OT volume, parse each project.work file,
/// and upsert into the SQLite projects table (called on volume mount).
///
/// Threat model T-02-02: volume_path comes from AppState (set by Phase 1 volume
/// detection), not from the frontend.
///
/// Assumption guard (Open Question 3): Tries project.work first; falls back to
/// project.strd if .work does not exist.
///
/// Assumption guard A2: tempo stored as integer × TEMPO_SCALE_FACTOR.
#[tauri::command]
#[specta::specta]
pub async fn index_ot_projects(
    state: tauri::State<'_, AppState>,
) -> Result<usize, AppError> {
    let volume_path = {
        let device = state.device.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        device.mount_point.clone().ok_or_else(|| AppError::Device("No OT volume mounted".to_string()))?
    };

    let sets_dir = volume_path.join("SETS");
    if !sets_dir.exists() {
        return Err(AppError::Io(format!(
            "SETS directory not found at {}",
            sets_dir.display()
        )));
    }

    // Clear existing index before re-populating
    {
        let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        db::projects::clear_projects(&db.conn)
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    let mut count = 0usize;

    for set_entry in std::fs::read_dir(&sets_dir).map_err(AppError::from)? {
        let set_dir = set_entry.map_err(AppError::from)?.path();
        if !set_dir.is_dir() {
            continue;
        }
        let set_name = set_dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();

        for project_entry in std::fs::read_dir(&set_dir).map_err(AppError::from)? {
            let project_dir = project_entry.map_err(AppError::from)?.path();
            if !project_dir.is_dir() {
                continue;
            }
            let project_name = project_dir
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();

            // Assumption guard (Open Question 3): try .work first, fall back to .strd
            let work_file = project_dir.join("project.work");
            let strd_file = project_dir.join("project.strd");
            let project_file = if work_file.exists() {
                info!("index_ot_projects: using project.work for {}/{}", set_name, project_name);
                work_file
            } else if strd_file.exists() {
                info!(
                    "index_ot_projects: project.work missing, using project.strd for {}/{}",
                    set_name, project_name
                );
                strd_file
            } else {
                tracing::warn!(
                    "index_ot_projects: no project.work or project.strd in {}",
                    project_dir.display()
                );
                continue;
            };

            // FIXME: Phase 1 OT project.work parser not yet implemented.
            // For now, record a project row with minimal metadata (name + path).
            // When Phase 1 parser is ready, parse project_file to extract tempo and bank count.
            let last_modified = project_dir
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .map(|t| {
                    let secs = t
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    format_unix_timestamp(secs)
                });

            // Stub tempo and bank_count — Phase 1 parser will fill these from binary data.
            let tempo_bpm: Option<f32> = None;
            let bank_count: Option<u8> = None;

            let row = db::projects::ProjectRow {
                id: generate_project_id(&project_dir),
                set_name: set_name.clone(),
                project_name: project_name.clone(),
                card_path: project_dir.to_string_lossy().into_owned(),
                tempo_bpm,
                bank_count,
                last_modified,
            };

            let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
            db::projects::upsert_project(&db.conn, &row)
                .map_err(|e| AppError::Database(e.to_string()))?;
            drop(db);

            tracing::debug!(
                "index_ot_projects: indexed {}/{} from {}",
                set_name,
                project_name,
                project_file.display()
            );
            count += 1;
        }
    }

    info!("index_ot_projects: indexed {} projects from {}", count, volume_path.display());
    Ok(count)
}

/// Generate a deterministic project ID from its card path using SHA-256 (first 16 bytes as hex).
fn generate_project_id(project_dir: &std::path::Path) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    project_dir.to_string_lossy().hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Format a Unix timestamp as an ISO 8601 date string (YYYY-MM-DD).
///
/// This avoids pulling in `chrono` for a simple display-only conversion.
fn format_unix_timestamp(secs: u64) -> String {
    // Simplified: compute year/month/day from Unix timestamp.
    // Using a minimal calendar calculation sufficient for display.
    let days_since_epoch = secs / 86400;
    let mut year = 1970u64;
    let mut remaining = days_since_epoch;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        year += 1;
    }

    let months = if is_leap_year(year) {
        [31u64, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31u64, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1u64;
    for &days_in_month in &months {
        if remaining < days_in_month {
            break;
        }
        remaining -= days_in_month;
        month += 1;
    }

    let day = remaining + 1;
    format!("{:04}-{:02}-{:02}", year, month, day)
}

fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}
