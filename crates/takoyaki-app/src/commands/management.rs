//! Tauri commands for project management operations (Phase 4 — Plan 04-03).
//!
//! Exposes 5 IPC commands: duplicate, rename, export, bank copy, and dry-run preview.
//!
//! Threat model:
//! - T-04-07: validate_ot_name() called on new_name before any filesystem operation.
//! - T-04-09: conflict_resolutions values validated against enum set before copy_bank.
//! - T-03-04 pattern: DB lock released before all file I/O (mutex deadlock avoidance).
//! - SAFE-03: Pre-operation snapshot created before any destructive write.

use crate::atomic::snapshot::SnapshotEngine;
use crate::commands::backup::{ChangeType, ConflictDetail, FileChangeEntry, FileChangeManifest};
use crate::db;
use crate::error::AppError;
use crate::management;
use crate::AppState;
use serde::Serialize;
use specta::Type;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tauri::ipc::Channel;
use tracing::{error, info};
use walkdir::WalkDir;

// ---------------------------------------------------------------------------
// Event types
// ---------------------------------------------------------------------------

/// Events emitted to the frontend during management operations.
///
/// Mirrors `BackupEvent` shape (Plan 03-02 pattern) for consistent frontend handling.
#[derive(Clone, Serialize, Type)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum ManagementEvent {
    Started {
        total_files: usize,
        destination: String,
    },
    Progress {
        files_processed: usize,
        total_files: usize,
        current_file: String,
    },
    Complete {
        files_processed: usize,
        total_bytes: u64,
        destination: String,
    },
    Failed {
        reason: String,
    },
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Snapshot root directory: ~/takoyaki/snapshots/
fn snapshot_root() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("takoyaki")
        .join("snapshots")
}

/// Collect all file paths under a directory tree (for snapshot input).
fn collect_files(dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .follow_links(false)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect()
}

/// Resolve a project directory from card_path stored in DB.
/// card_path is the absolute path to the project dir (e.g. /Volumes/OT/SETS/MY_PROJECT).
fn project_dir_from_card_path(card_path: &str) -> PathBuf {
    PathBuf::from(card_path)
}

/// Extract card volume root from project dir path.
/// Project lives at {volume}/SETS/{project_name} so volume = project_dir/../..
fn card_volume_from_project_dir(project_dir: &Path) -> Option<PathBuf> {
    project_dir
        .parent() // SETS/
        .and_then(|p| p.parent()) // volume root
        .map(|p| p.to_path_buf())
}

/// Create pre-operation snapshot of all files in a project directory.
fn snapshot_project(project_dir: &Path, label: &str) -> Result<(), AppError> {
    let files = collect_files(project_dir);
    let file_refs: Vec<&Path> = files.iter().map(|p| p.as_path()).collect();
    let engine = SnapshotEngine::new(snapshot_root());
    let result = engine.snapshot_files(&file_refs, label)?;
    info!(
        "Pre-{} snapshot created: {} files at {}",
        label,
        result.file_count,
        result.snapshot_dir.display()
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

/// Compute a dry-run manifest showing what a management operation would change (SAFE-07).
///
/// Returns a `FileChangeManifest` without modifying any files.
/// For bank-copy, uses SHA-256 conflict detection (T-04-06).
///
/// T-03-04: DB lock released before all file I/O.
/// T-04-07: new_name validated via validate_ot_name before use.
#[tauri::command]
#[specta::specta]
pub async fn compute_management_dry_run(
    state: tauri::State<'_, AppState>,
    project_id: String,
    operation: String,
    target_project_id: Option<String>,
    _bank_index: Option<u32>,
    new_name: Option<String>,
) -> Result<FileChangeManifest, AppError> {
    // 1. Resolve project info from DB (T-03-04: release before I/O)
    let (card_path, project_name) = {
        let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        let path = db::projects::get_card_path(&db.conn, &project_id)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let name = PathBuf::from(&path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| project_id.clone());
        (path, name)
        // DB lock dropped
    };

    let project_dir = project_dir_from_card_path(&card_path);
    let card_volume = card_volume_from_project_dir(&project_dir)
        .ok_or(AppError::InvalidPath)?;

    let snapshot_prefix =
        "A snapshot of the current state will be created before applying. ";

    match operation.as_str() {
        "duplicate" => {
            let dest_name = new_name.as_deref().unwrap_or(&project_name);
            // T-04-07: validate name
            management::project_work::validate_ot_name(dest_name)?;

            let dest_dir = card_volume.join("SETS").join(dest_name);
            let mut entries: Vec<FileChangeEntry> = Vec::new();
            let mut total_bytes: u64 = 0;

            for entry in WalkDir::new(&project_dir).follow_links(false).min_depth(1) {
                let entry = entry.map_err(|e| AppError::Io(e.to_string()))?;
                if entry.file_type().is_file() {
                    let relative = entry
                        .path()
                        .strip_prefix(&project_dir)
                        .map_err(|_| AppError::InvalidPath)?
                        .to_string_lossy()
                        .into_owned();
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    total_bytes += size;
                    entries.push(FileChangeEntry {
                        path: format!("{}/{}", dest_name, relative),
                        change_type: ChangeType::Added,
                        size_bytes: size,
                    });
                }
            }

            let total_added = entries.len();
            Ok(FileChangeManifest {
                entries,
                total_added,
                total_modified: 0,
                total_removed: 0,
                total_unchanged: 0,
                total_bytes,
                destination_path: dest_dir.to_string_lossy().into_owned(),
                operation_label: format!(
                    "{}Duplicate {} → {}",
                    snapshot_prefix, project_name, dest_name
                ),
                project_name,
                conflict_details: Vec::new(),
            })
        }

        "rename" => {
            let dest_name = new_name.as_deref().ok_or_else(|| {
                AppError::Io("new_name required for rename dry-run".to_string())
            })?;
            // T-04-07: validate name
            management::project_work::validate_ot_name(dest_name)?;

            Ok(FileChangeManifest {
                entries: vec![FileChangeEntry {
                    path: project_name.clone(),
                    change_type: ChangeType::Modified,
                    size_bytes: 0,
                }],
                total_added: 0,
                total_modified: 1,
                total_removed: 0,
                total_unchanged: 0,
                total_bytes: 0,
                destination_path: card_volume
                    .join("SETS")
                    .join(dest_name)
                    .to_string_lossy()
                    .into_owned(),
                operation_label: format!(
                    "{}Rename {} → {}",
                    snapshot_prefix, project_name, dest_name
                ),
                project_name,
                conflict_details: Vec::new(),
            })
        }

        "export" => {
            let export_dest =
                management::export::compute_export_dest(&project_name)?;

            let mut entries: Vec<FileChangeEntry> = Vec::new();
            let mut total_bytes: u64 = 0;

            // List all project files
            for entry in WalkDir::new(&project_dir).follow_links(false).min_depth(1) {
                let entry = entry.map_err(|e| AppError::Io(e.to_string()))?;
                if entry.file_type().is_file() {
                    let relative = entry
                        .path()
                        .strip_prefix(&project_dir)
                        .map_err(|_| AppError::InvalidPath)?
                        .to_string_lossy()
                        .into_owned();
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    total_bytes += size;
                    entries.push(FileChangeEntry {
                        path: format!("SETS/{}/{}", project_name, relative),
                        change_type: ChangeType::Added,
                        size_bytes: size,
                    });
                }
            }

            // List referenced audio files
            let pw_path = project_dir.join("project.work");
            if pw_path.exists() {
                let pw_bytes = std::fs::read(&pw_path)?;
                let slots = management::project_work::extract_slot_paths(&pw_bytes);
                let mut seen = std::collections::HashSet::new();
                for slot in &slots {
                    let audio_path = resolve_audio_path(&project_dir, &card_volume, &slot.path);
                    if let Some(path) = audio_path {
                        if seen.insert(path.clone()) && path.exists() {
                            let size = path.metadata().map(|m| m.len()).unwrap_or(0);
                            total_bytes += size;
                            let fname = path.file_name().map(|n| n.to_string_lossy().into_owned())
                                .unwrap_or_default();
                            entries.push(FileChangeEntry {
                                path: format!("AUDIO/{}", fname),
                                change_type: ChangeType::Added,
                                size_bytes: size,
                            });
                        }
                    }
                }
            }

            let total_added = entries.len();
            Ok(FileChangeManifest {
                entries,
                total_added,
                total_modified: 0,
                total_removed: 0,
                total_unchanged: 0,
                total_bytes,
                destination_path: export_dest.to_string_lossy().into_owned(),
                operation_label: format!("{}Export {}", snapshot_prefix, project_name),
                project_name,
                conflict_details: Vec::new(),
            })
        }

        "bank-copy" => {
            let target_id = target_project_id.ok_or_else(|| {
                AppError::Io("target_project_id required for bank-copy dry-run".to_string())
            })?;

            // Resolve target project path from DB
            let target_card_path = {
                let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
                db::projects::get_card_path(&db.conn, &target_id)
                    .map_err(|e| AppError::Database(e.to_string()))?
                // DB lock dropped
            };

            let target_dir = project_dir_from_card_path(&target_card_path);

            let analysis = management::bank_copy::compute_bank_copy_conflicts(
                &project_dir,
                &target_dir,
                &card_volume,
            )?;

            let mut entries: Vec<FileChangeEntry> = Vec::new();
            let mut total_bytes: u64 = 0;

            for slot in &analysis.auto_copy {
                let audio = resolve_audio_path(&project_dir, &card_volume, &slot.path);
                let size = audio.as_ref().and_then(|p| p.metadata().ok()).map(|m| m.len()).unwrap_or(0);
                total_bytes += size;
                let fname = slot.path.replace('\\', "/");
                let fname = fname.trim_start_matches('/');
                let fname = std::path::Path::new(fname).file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| slot.path.clone());
                entries.push(FileChangeEntry {
                    path: format!("AUDIO/{}", fname),
                    change_type: ChangeType::Added,
                    size_bytes: size,
                });
            }

            for slot in &analysis.skip {
                let fname = slot.path.replace('\\', "/");
                let fname = fname.trim_start_matches('/');
                let fname = std::path::Path::new(fname).file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| slot.path.clone());
                entries.push(FileChangeEntry {
                    path: format!("AUDIO/{}", fname),
                    change_type: ChangeType::Unchanged,
                    size_bytes: 0,
                });
            }

            for conflict in &analysis.conflicts {
                entries.push(FileChangeEntry {
                    path: format!("AUDIO/{}", conflict.filename),
                    change_type: ChangeType::Conflict,
                    size_bytes: 0,
                });
            }

            let total_added = analysis.auto_copy.len();
            let total_unchanged = analysis.skip.len();

            let conflict_details: Vec<ConflictDetail> = analysis.conflicts.iter().map(|c| {
                ConflictDetail {
                    filename: c.filename.clone(),
                    source_hash: c.source_hash.clone(),
                    target_hash: c.target_hash.clone(),
                }
            }).collect();

            Ok(FileChangeManifest {
                entries,
                total_added,
                total_modified: 0,
                total_removed: 0,
                total_unchanged,
                total_bytes,
                destination_path: target_card_path,
                operation_label: format!(
                    "{}Bank Copy → {}",
                    snapshot_prefix, project_name
                ),
                project_name,
                conflict_details,
            })
        }

        _ => Err(AppError::Io(format!(
            "Unknown management operation: {}",
            operation
        ))),
    }
}

/// Duplicate an OT project with pre-operation snapshot (SAFE-03).
///
/// T-04-07: new_name validated via validate_ot_name.
/// T-03-04: DB lock released before file I/O.
#[tauri::command]
#[specta::specta]
pub async fn duplicate_project(
    state: tauri::State<'_, AppState>,
    project_id: String,
    new_name: String,
    on_event: Channel<ManagementEvent>,
) -> Result<(), AppError> {
    // T-04-07: Validate name before DB lookup
    management::project_work::validate_ot_name(&new_name)?;

    // Resolve project path from DB (T-03-04: release before I/O)
    let card_path = {
        let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        db::projects::get_card_path(&db.conn, &project_id)
            .map_err(|e| AppError::Database(e.to_string()))?
        // DB lock dropped
    };

    let project_dir = project_dir_from_card_path(&card_path);
    let card_volume = card_volume_from_project_dir(&project_dir)
        .ok_or(AppError::InvalidPath)?;

    // Count files for progress reporting
    let total_files = collect_files(&project_dir).len();

    // Send Started event
    let _ = on_event.send(ManagementEvent::Started {
        total_files,
        destination: card_volume
            .join("SETS")
            .join(&new_name)
            .to_string_lossy()
            .into_owned(),
    });

    // Pre-operation snapshot (SAFE-03)
    if let Err(e) = snapshot_project(&project_dir, "pre-duplicate") {
        error!("Pre-duplicate snapshot failed: {}", e);
        // Non-fatal: log and continue (snapshot is best-effort for duplicate)
    }

    // Perform duplication
    match management::duplicate::duplicate_project(&project_dir, &new_name, &card_volume) {
        Ok(result) => {
            info!(
                "Duplicate complete: {} -> {} ({} files)",
                project_dir.display(),
                result.new_project_dir.display(),
                result.files_copied
            );
            let _ = on_event.send(ManagementEvent::Complete {
                files_processed: result.files_copied,
                total_bytes: 0,
                destination: result.new_project_dir.to_string_lossy().into_owned(),
            });
            Ok(())
        }
        Err(e) => {
            error!("Duplicate failed: {}", e);
            let _ = on_event.send(ManagementEvent::Failed {
                reason: e.to_string(),
            });
            Err(e)
        }
    }
}

/// Rename an OT project with pre-operation snapshot (SAFE-03).
///
/// Updates the DB `card_path` after successful rename.
/// No channel needed — rename is fast and synchronous.
///
/// T-04-07: new_name validated via validate_ot_name.
/// T-03-04: DB lock released before file I/O, re-acquired for DB update.
#[tauri::command]
#[specta::specta]
pub async fn rename_project(
    state: tauri::State<'_, AppState>,
    project_id: String,
    new_name: String,
) -> Result<(), AppError> {
    // T-04-07: Validate name before DB lookup
    management::project_work::validate_ot_name(&new_name)?;

    // Resolve project path from DB (T-03-04: release before I/O)
    let (card_path, set_name) = {
        let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        let path = db::projects::get_card_path(&db.conn, &project_id)
            .map_err(|e| AppError::Database(e.to_string()))?;
        // Resolve set_name from DB for upsert
        let set_name = {
            let mut stmt = db.conn
                .prepare("SELECT set_name FROM projects WHERE id = ?1")
                .map_err(|e| AppError::Database(e.to_string()))?;
            stmt.query_row(rusqlite::params![project_id], |row| row.get::<_, String>(0))
                .unwrap_or_else(|_| "DEFAULT".to_string())
        };
        (path, set_name)
        // DB lock dropped
    };

    let project_dir = project_dir_from_card_path(&card_path);

    // Pre-operation snapshot (SAFE-03)
    snapshot_project(&project_dir, "pre-rename")?;

    // Perform rename
    let new_dir = management::rename::rename_project(&project_dir, &new_name)?;

    info!(
        "Rename complete: {} -> {}",
        project_dir.display(),
        new_dir.display()
    );

    // Update DB: card_path and project_name reflect new name (T-03-04: re-acquire lock)
    {
        let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        let row = db::projects::ProjectRow {
            id: project_id,
            set_name,
            project_name: new_name,
            card_path: new_dir.to_string_lossy().into_owned(),
            tempo_bpm: None,
            bank_count: None,
            last_modified: None,
        };
        db::projects::upsert_project(&db.conn, &row)
            .map_err(|e| AppError::Database(e.to_string()))?;
        // DB lock dropped
    }

    Ok(())
}

/// Export an OT project to a self-contained zip archive.
///
/// Export is read-only — no snapshot needed (nothing on the card is modified).
/// Zip is written to ~/takoyaki/exports/{project_name}_{timestamp}.zip (D-06).
///
/// T-03-04: DB lock released before file I/O.
#[tauri::command]
#[specta::specta]
pub async fn export_project(
    state: tauri::State<'_, AppState>,
    project_id: String,
    on_event: Channel<ManagementEvent>,
) -> Result<(), AppError> {
    // Resolve project path from DB (T-03-04: release before I/O)
    let (card_path, project_name) = {
        let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        let path = db::projects::get_card_path(&db.conn, &project_id)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let name = PathBuf::from(&path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| project_id.clone());
        (path, name)
        // DB lock dropped
    };

    let project_dir = project_dir_from_card_path(&card_path);
    let card_volume = card_volume_from_project_dir(&project_dir)
        .ok_or(AppError::InvalidPath)?;

    // Compute export destination
    let export_dest = management::export::compute_export_dest(&project_name)?;

    let total_files = collect_files(&project_dir).len();

    // Send Started event
    let _ = on_event.send(ManagementEvent::Started {
        total_files,
        destination: export_dest.to_string_lossy().into_owned(),
    });

    // Perform export (read-only — no snapshot needed)
    match management::export::export_project(&project_dir, &card_volume, &export_dest) {
        Ok(result) => {
            info!(
                "Export complete: {} files, {} bytes -> {}",
                result.files_exported,
                result.total_bytes,
                result.zip_path.display()
            );
            let _ = on_event.send(ManagementEvent::Complete {
                files_processed: result.files_exported,
                total_bytes: result.total_bytes,
                destination: result.zip_path.to_string_lossy().into_owned(),
            });
            Ok(())
        }
        Err(e) => {
            error!("Export failed: {}", e);
            let _ = on_event.send(ManagementEvent::Failed {
                reason: e.to_string(),
            });
            Err(e)
        }
    }
}

/// Copy a bank from source project to target project, with pre-operation snapshot (SAFE-03).
///
/// Streams progress via Channel. Pre-snapshots ALL files in target project (Pitfall 3).
///
/// T-04-09: conflict_resolutions values validated in copy_bank() business logic.
/// T-03-04: DB lock released before file I/O.
#[tauri::command]
#[specta::specta]
pub async fn copy_bank(
    state: tauri::State<'_, AppState>,
    source_project_id: String,
    source_bank_index: u32,
    target_project_id: String,
    target_bank_index: u32,
    conflict_resolutions: HashMap<String, String>,
    on_event: Channel<ManagementEvent>,
) -> Result<(), AppError> {
    // Resolve both project paths from DB (T-03-04: release before I/O)
    let (source_card_path, target_card_path) = {
        let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        let src = db::projects::get_card_path(&db.conn, &source_project_id)
            .map_err(|e| AppError::Database(e.to_string()))?;
        let tgt = db::projects::get_card_path(&db.conn, &target_project_id)
            .map_err(|e| AppError::Database(e.to_string()))?;
        (src, tgt)
        // DB lock dropped
    };

    let source_dir = project_dir_from_card_path(&source_card_path);
    let target_dir = project_dir_from_card_path(&target_card_path);
    let card_volume = card_volume_from_project_dir(&source_dir)
        .ok_or(AppError::InvalidPath)?;

    let total_files = collect_files(&target_dir).len();

    // Send Started event
    let _ = on_event.send(ManagementEvent::Started {
        total_files,
        destination: target_card_path.clone(),
    });

    // Pre-operation snapshot of ALL target project files (Pitfall 3 — SAFE-03)
    snapshot_project(&target_dir, "pre-bank-copy")?;

    // Perform bank copy
    match management::bank_copy::copy_bank(
        &source_dir,
        source_bank_index as u8,
        &target_dir,
        target_bank_index as u8,
        &card_volume,
        &conflict_resolutions,
    ) {
        Ok(result) => {
            info!(
                "Bank copy complete: {} files copied, {} conflicts resolved",
                result.files_copied, result.conflicts_resolved
            );
            let _ = on_event.send(ManagementEvent::Complete {
                files_processed: result.files_copied,
                total_bytes: 0,
                destination: target_card_path,
            });
            Ok(())
        }
        Err(e) => {
            error!("Bank copy failed: {}", e);
            let _ = on_event.send(ManagementEvent::Failed {
                reason: e.to_string(),
            });
            Err(e)
        }
    }
}

// ---------------------------------------------------------------------------
// Internal path resolution helper
// ---------------------------------------------------------------------------

/// Resolve a slot path from project.work to an absolute filesystem path.
/// Handles both relative (../AUDIO/) and OT absolute (\AUDIO\) formats.
fn resolve_audio_path(
    project_dir: &Path,
    card_volume_path: &Path,
    raw_path: &str,
) -> Option<PathBuf> {
    let normalized = raw_path.replace('\\', "/");

    if normalized.starts_with('/') {
        // OT absolute path
        crate::health::resolve_ot_path(card_volume_path, raw_path)
    } else {
        // Relative path
        let resolved = project_dir.join(&normalized);
        if resolved.exists() {
            std::fs::canonicalize(&resolved).ok()
        } else {
            Some(resolved)
        }
    }
}
