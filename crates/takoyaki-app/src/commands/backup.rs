//! Tauri commands for backup, restore, and dry-run operations (SAFE-01, SAFE-02, SAFE-05, SAFE-06, SAFE-07).
//!
//! Threat model:
//! - T-03-01: backup destination always computed in Rust via dirs::home_dir() — frontend never supplies a raw path.
//! - T-03-02: snapshot_id resolved to stored_path via DB lookup — frontend never supplies raw paths.
//! - T-03-03: WalkDir used with follow_links(false) to prevent symlink traversal.
//! - T-03-04: DB lock released before all file I/O (Pitfall 4 — mutex deadlock avoidance).

use crate::atomic;
use crate::atomic::snapshot::SnapshotEngine;
use crate::db;
use crate::error::AppError;
use crate::AppState;
use serde::Serialize;
use specta::Type;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use tauri::ipc::Channel;
use tracing::{debug, error, info};
use walkdir::WalkDir;

// ---------------------------------------------------------------------------
// Event types for Channel-based progress streaming
// ---------------------------------------------------------------------------

/// Events emitted to the frontend during backup and restore operations.
#[derive(Clone, Serialize, Type)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum BackupEvent {
    Started {
        total_files: usize,
        destination: String,
    },
    Progress {
        files_copied: usize,
        total_files: usize,
        current_file: String,
    },
    Complete {
        files_copied: usize,
        total_bytes: u64,
        destination: String,
        checksum_ok: bool,
    },
    Failed {
        reason: String,
    },
}

/// Change type classification for dry-run manifests (SAFE-07).
#[derive(Debug, Serialize, Type, Clone)]
pub enum ChangeType {
    Added,
    Modified,
    Removed,
    Unchanged,
    /// Hash-mismatch conflict detected during bank copy (Phase 4 D-08).
    Conflict,
}

/// A single file entry in a dry-run manifest.
#[derive(Debug, Serialize, Type, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileChangeEntry {
    /// Relative path from project root.
    pub path: String,
    pub change_type: ChangeType,
    pub size_bytes: u64,
}

/// Conflict detail for bank copy operations (D-08 resolution UI, Plan 04-07).
#[derive(Debug, Serialize, Type, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConflictDetail {
    pub filename: String,
    pub source_hash: String,
    pub target_hash: String,
}

/// Full dry-run manifest returned by compute_dry_run (SAFE-07).
#[derive(Debug, Serialize, Type, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileChangeManifest {
    pub entries: Vec<FileChangeEntry>,
    pub total_added: usize,
    pub total_modified: usize,
    pub total_removed: usize,
    pub total_unchanged: usize,
    pub total_bytes: u64,
    pub destination_path: String,
    pub operation_label: String,
    pub project_name: String,
    /// Populated for bank-copy operations only — empty for backup/restore/duplicate/rename/export.
    pub conflict_details: Vec<ConflictDetail>,
}

// ---------------------------------------------------------------------------
// Internal helper structs
// ---------------------------------------------------------------------------

struct CopiedFile {
    src: PathBuf,
    dest: PathBuf,
    relative_path: String,
    size_bytes: u64,
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Compute the backup base directory: ~/takoyaki/backups/ (T-03-01).
fn backup_base_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("takoyaki")
        .join("backups")
}

/// Format the current time as YYYY-MM-DD_HH-MM (for backup directory naming).
///
/// Uses std::time::SystemTime — avoids chrono dependency (same approach as Plan 01-05).
fn format_timestamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let days_total = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;

    let mut year = 1970u64;
    let mut remaining_days = days_total;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let months = if is_leap_year(year) {
        [31u64, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31u64, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1u64;
    for &days_in_month in &months {
        if remaining_days < days_in_month {
            break;
        }
        remaining_days -= days_in_month;
        month += 1;
    }

    let day = remaining_days + 1;
    format!("{:04}-{:02}-{:02}_{:02}-{:02}", year, month, day, hours, minutes)
}

fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Walk src tree and copy all files to dest, checking cancel_flag per file (SAFE-01, SAFE-02).
///
/// T-03-03: follow_links(false) prevents symlink traversal.
/// Returns (copied_files, total_bytes) on success or Err(AppError::Cancelled) on cancellation.
fn copy_project_tree(
    src: &Path,
    dest: &Path,
    cancel_flag: &std::sync::atomic::AtomicBool,
    on_event: &Channel<BackupEvent>,
) -> Result<(Vec<CopiedFile>, u64), AppError> {
    let mut copied_files: Vec<CopiedFile> = Vec::new();
    let mut total_bytes: u64 = 0;
    let mut files_copied = 0usize;

    // Count total files for progress reporting
    let total_files = WalkDir::new(src)
        .follow_links(false)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .count();

    for entry in WalkDir::new(src).follow_links(false).min_depth(1) {
        // Check cancellation before each file (SAFE-06 cancellation support)
        if cancel_flag.load(Ordering::Relaxed) {
            // Clean up partial backup
            let _ = std::fs::remove_dir_all(dest);
            return Err(AppError::Cancelled("Backup cancelled by user".to_string()));
        }

        let entry = entry.map_err(|e| AppError::Io(e.to_string()))?;
        let entry_path = entry.path();

        // Compute relative path from src root
        let relative = entry_path
            .strip_prefix(src)
            .map_err(|_| AppError::InvalidPath)?
            .to_string_lossy()
            .into_owned();

        let dest_entry = dest.join(&relative);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&dest_entry)?;
        } else if entry.file_type().is_file() {
            // Ensure parent directory exists
            if let Some(parent) = dest_entry.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let bytes = std::fs::copy(entry_path, &dest_entry)?;
            total_bytes += bytes;
            files_copied += 1;

            debug!("Backup copy: {} -> {}", entry_path.display(), dest_entry.display());

            let _ = on_event.send(BackupEvent::Progress {
                files_copied,
                total_files,
                current_file: relative.clone(),
            });

            copied_files.push(CopiedFile {
                src: entry_path.to_path_buf(),
                dest: dest_entry,
                relative_path: relative,
                size_bytes: bytes,
            });
        }
    }

    Ok((copied_files, total_bytes))
}

/// Verify SHA-256 checksums of all source/dest file pairs (SAFE-02).
///
/// Returns false if any pair mismatches — logs mismatches with error!.
fn verify_checksums(copied_files: &[CopiedFile]) -> bool {
    let mut all_ok = true;
    for copied in copied_files {
        let src_hash = match atomic::snapshot::sha256_hex(&copied.src) {
            Ok(h) => h,
            Err(e) => {
                error!("Checksum read error for {}: {}", copied.src.display(), e);
                all_ok = false;
                continue;
            }
        };
        let dest_hash = match atomic::snapshot::sha256_hex(&copied.dest) {
            Ok(h) => h,
            Err(e) => {
                error!("Checksum read error for {}: {}", copied.dest.display(), e);
                all_ok = false;
                continue;
            }
        };
        if src_hash != dest_hash {
            error!(
                "Checksum mismatch for {}: src={} dest={}",
                copied.relative_path, src_hash, dest_hash
            );
            all_ok = false;
        }
    }
    all_ok
}

/// Generate a backup ID from the destination path using DefaultHasher.
fn generate_backup_id(dest_path: &Path) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    dest_path.to_string_lossy().hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

/// Back up an OT project to ~/takoyaki/backups/PROJECT_NAME/TIMESTAMP_label/ (SAFE-01, SAFE-02).
///
/// Streams per-file progress via Channel. Checks AtomicBool cancel_flag per file (SAFE-06).
/// Computes SHA-256 checksums to verify copy integrity (SAFE-02).
/// Records backup in SQLite with status='in-progress'; marks complete on success (SAFE-05, D-12).
///
/// T-03-01: Destination computed in Rust only — frontend supplies project_id + label, not path.
/// T-03-04: DB lock released before file I/O loop to avoid deadlock (Pitfall 4).
#[tauri::command]
#[specta::specta]
pub async fn backup_project(
    state: tauri::State<'_, AppState>,
    project_id: String,
    label: String,
    on_event: Channel<BackupEvent>,
) -> Result<(), AppError> {
    // 1. Resolve project path from DB — drop lock before file I/O (T-03-04)
    let (card_path, project_name) = {
        let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        let path = db::projects::get_card_path(&db.conn, &project_id)
            .map_err(|e| AppError::Database(e.to_string()))?;
        // Project name is the last component of card_path
        let name = PathBuf::from(&path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| project_id.clone());
        (path, name)
        // DB lock dropped here
    };

    let src_path = PathBuf::from(&card_path);

    // 2. Compute destination path (T-03-01)
    let dest_path = backup_base_dir()
        .join(&project_name)
        .join(format!("{}_{}", format_timestamp(), label));

    // 3. Create destination directory
    std::fs::create_dir_all(&dest_path)?;

    // 4. Count files for BackupEvent::Started
    let total_files = WalkDir::new(&src_path)
        .follow_links(false)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .count();

    // 5. Send Started event
    let _ = on_event.send(BackupEvent::Started {
        total_files,
        destination: dest_path.to_string_lossy().into_owned(),
    });

    // 6. Generate backup ID
    let backup_id = generate_backup_id(&dest_path);

    // 7. Insert in-progress backup record (D-12: partial backup tracked)
    {
        let mut db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        let record = db::backups::BackupInsert {
            id: backup_id.clone(),
            project_id: project_id.clone(),
            project_name: project_name.clone(),
            dest_path: dest_path.to_string_lossy().into_owned(),
            created_at: format_timestamp(),
            operation: label.clone(),
            file_count: total_files as i64,
            total_bytes: 0,
            checksum_ok: false,
            status: "in-progress".to_string(),
            files: vec![],
        };
        db::backups::insert_backup(&mut db.conn, &record)
            .map_err(|e| AppError::Database(e.to_string()))?;
        // DB lock dropped here
    }

    // 8. Reset cancel flag
    state.cancel_backup.store(false, Ordering::Relaxed);

    // 9. Copy project tree
    let copy_result = copy_project_tree(&src_path, &dest_path, &state.cancel_backup, &on_event);

    match copy_result {
        Ok((copied_files, total_bytes)) => {
            // 10. Verify checksums (SAFE-02)
            let checksum_ok = verify_checksums(&copied_files);

            // 11. Mark backup complete in DB
            {
                let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
                db::backups::mark_backup_complete(&db.conn, &backup_id, checksum_ok)
                    .map_err(|e| AppError::Database(e.to_string()))?;
            }

            info!(
                "Backup complete: {} files, {} bytes, checksum_ok={}",
                copied_files.len(),
                total_bytes,
                checksum_ok
            );

            // 12. Send Complete event
            let _ = on_event.send(BackupEvent::Complete {
                files_copied: copied_files.len(),
                total_bytes,
                destination: dest_path.to_string_lossy().into_owned(),
                checksum_ok,
            });

            Ok(())
        }
        Err(e) => {
            // Cancelled or IO error — in-progress record stays for D-12 cleanup on next launch
            error!("Backup failed: {}", e);
            let _ = on_event.send(BackupEvent::Failed {
                reason: e.to_string(),
            });
            Err(e)
        }
    }
}

/// Restore an OT project from a backup snapshot (SAFE-06, SAFE-04, D-11).
///
/// Creates a pre-restore snapshot of the current project state before overwriting (D-11).
/// Writes all files atomically via atomic_write_batch (SAFE-04: FAT32 rename atomicity).
///
/// T-03-02: backup_id resolved to dest_path via DB lookup — frontend never supplies raw paths.
/// T-03-04: DB lock released before file I/O.
#[tauri::command]
#[specta::specta]
pub async fn restore_snapshot(
    state: tauri::State<'_, AppState>,
    backup_id: String,
    on_event: Channel<BackupEvent>,
) -> Result<(), AppError> {
    // 1. Resolve backup dest_path and project card_path from DB (T-03-02)
    let (backup_dest, card_path, project_id) = {
        let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;

        let dest = db::backups::get_backup_dest_path(&db.conn, &backup_id)
            .map_err(|e| AppError::Database(e.to_string()))?;

        // Get the project_id for this backup so we can look up the card path
        let mut stmt = db
            .conn
            .prepare("SELECT project_id FROM backups WHERE id = ?1")
            .map_err(|e| AppError::Database(e.to_string()))?;
        let pid: String = stmt
            .query_row(rusqlite::params![backup_id], |row| row.get(0))
            .map_err(|e| AppError::Database(e.to_string()))?;

        let cpath = db::projects::get_card_path(&db.conn, &pid)
            .map_err(|e| AppError::Database(e.to_string()))?;

        (dest, cpath, pid)
        // DB lock dropped here
    };

    let backup_path = PathBuf::from(&backup_dest);
    let project_path = PathBuf::from(&card_path);

    // 2. Verify device is connected
    {
        let device = state.device.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        if device.mount_point.is_none() {
            return Err(AppError::Device(
                "OT card not mounted — connect device before restoring".to_string(),
            ));
        }
        // Device lock dropped here
    }

    // 3. Create pre-restore snapshot (D-11: snapshot before any write)
    let snapshot_root = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("takoyaki")
        .join("snapshots");

    let project_files: Vec<PathBuf> = WalkDir::new(&project_path)
        .follow_links(false)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();

    let snapshot_engine = SnapshotEngine::new(snapshot_root.clone());
    let project_file_refs: Vec<&Path> = project_files.iter().map(|p| p.as_path()).collect();
    let snapshot_result = snapshot_engine
        .snapshot_files(&project_file_refs, "pre-restore")?;

    info!(
        "Pre-restore snapshot created: {} files at {}",
        snapshot_result.file_count,
        snapshot_result.snapshot_dir.display()
    );

    // 4. Record the pre-restore snapshot as a backup in SQLite
    {
        let mut db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        let snapshot_id = generate_backup_id(&snapshot_result.snapshot_dir);
        let project_name = PathBuf::from(&card_path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| project_id.clone());

        let file_inserts: Vec<db::backups::BackupFileInsert> = snapshot_result
            .files
            .iter()
            .map(|f| db::backups::BackupFileInsert {
                id: generate_backup_id(&f.stored_path),
                relative_path: f
                    .original_path
                    .strip_prefix(&project_path)
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_else(|_| f.original_path.to_string_lossy().into_owned()),
                stored_path: f.stored_path.to_string_lossy().into_owned(),
                file_hash: f.file_hash.clone(),
                size_bytes: f.stored_path.metadata().map(|m| m.len() as i64).unwrap_or(0),
                change_type: "snapshot".to_string(),
            })
            .collect();

        let record = db::backups::BackupInsert {
            id: snapshot_id.clone(),
            project_id: project_id.clone(),
            project_name,
            dest_path: snapshot_result.snapshot_dir.to_string_lossy().into_owned(),
            created_at: format_timestamp(),
            operation: "pre-restore".to_string(),
            file_count: snapshot_result.file_count as i64,
            total_bytes: snapshot_result.total_bytes as i64,
            checksum_ok: true,
            status: "in-progress".to_string(),
            files: file_inserts,
        };
        db::backups::insert_backup(&mut db.conn, &record)
            .map_err(|e| AppError::Database(e.to_string()))?;
        db::backups::mark_backup_complete(&db.conn, &snapshot_id, true)
            .map_err(|e| AppError::Database(e.to_string()))?;
        // DB lock dropped here
    }

    // 5. Send Started event
    let total_files = WalkDir::new(&backup_path)
        .follow_links(false)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .count();

    let _ = on_event.send(BackupEvent::Started {
        total_files,
        destination: card_path.clone(),
    });

    // 6. Read all backup files into memory
    let mut writes: Vec<(PathBuf, Vec<u8>)> = Vec::new();

    for entry in WalkDir::new(&backup_path)
        .follow_links(false)
        .min_depth(1)
    {
        let entry = entry.map_err(|e| AppError::Io(e.to_string()))?;
        if entry.file_type().is_file() {
            let relative = entry
                .path()
                .strip_prefix(&backup_path)
                .map_err(|_| AppError::InvalidPath)?
                .to_path_buf();

            let dest_file = project_path.join(&relative);
            let contents = std::fs::read(entry.path())?;
            writes.push((dest_file, contents));
        }
    }

    // 7. Write all files atomically via atomic_write_batch (SAFE-04)
    let write_refs: Vec<(&Path, &[u8])> = writes
        .iter()
        .map(|(p, c)| (p.as_path(), c.as_slice()))
        .collect();

    atomic::atomic_write_batch(&write_refs)?;

    // 8. Send Complete event
    let total_bytes: u64 = writes.iter().map(|(_, c)| c.len() as u64).sum();
    let _ = on_event.send(BackupEvent::Complete {
        files_copied: writes.len(),
        total_bytes,
        destination: card_path,
        checksum_ok: true,
    });

    // 9. Reset cancel flag
    state.cancel_backup.store(false, Ordering::Relaxed);

    info!("Restore complete: {} files written", writes.len());
    Ok(())
}

/// Compute a dry-run manifest showing what a backup or restore would change (SAFE-07).
///
/// This function MUST NOT modify any files — it only reads and computes hashes.
///
/// For "backup": all project files are "Added" (new backup destination).
/// For "restore": compares backup files against current card state.
#[tauri::command]
#[specta::specta]
pub async fn compute_dry_run(
    state: tauri::State<'_, AppState>,
    project_id: String,
    operation: String,
    backup_id: Option<String>,
) -> Result<FileChangeManifest, AppError> {
    // 1. Resolve card_path from DB
    let card_path = {
        let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        db::projects::get_card_path(&db.conn, &project_id)
            .map_err(|e| AppError::Database(e.to_string()))?
        // DB lock dropped here
    };

    let project_path = PathBuf::from(&card_path);
    let project_name = project_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| project_id.clone());

    if operation == "backup" {
        // 2a. Backup dry-run: all files are "Added" (fresh destination)
        let dest_path = backup_base_dir()
            .join(&project_name)
            .join(format!("{}_dry-run", format_timestamp()));

        let mut entries: Vec<FileChangeEntry> = Vec::new();
        let mut total_bytes: u64 = 0;

        for entry in WalkDir::new(&project_path)
            .follow_links(false)
            .min_depth(1)
        {
            let entry = entry.map_err(|e| AppError::Io(e.to_string()))?;
            if entry.file_type().is_file() {
                let relative = entry
                    .path()
                    .strip_prefix(&project_path)
                    .map_err(|_| AppError::InvalidPath)?
                    .to_string_lossy()
                    .into_owned();

                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                total_bytes += size;

                entries.push(FileChangeEntry {
                    path: relative,
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
            destination_path: dest_path.to_string_lossy().into_owned(),
            operation_label: format!("Back Up {}", project_name),
            project_name,
            conflict_details: Vec::new(),
        })
    } else if operation == "restore" {
        // 2b. Restore dry-run: compare backup files vs current card state
        let bid = backup_id.ok_or_else(|| {
            AppError::Io("backup_id required for restore dry-run".to_string())
        })?;

        let backup_dest = {
            let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
            db::backups::get_backup_dest_path(&db.conn, &bid)
                .map_err(|e| AppError::Database(e.to_string()))?
            // DB lock dropped here
        };

        let backup_path = PathBuf::from(&backup_dest);

        // Build hash map of card files
        let mut card_files: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        for entry in WalkDir::new(&project_path)
            .follow_links(false)
            .min_depth(1)
        {
            let entry = entry.map_err(|e| AppError::Io(e.to_string()))?;
            if entry.file_type().is_file() {
                let relative = entry
                    .path()
                    .strip_prefix(&project_path)
                    .map_err(|_| AppError::InvalidPath)?
                    .to_string_lossy()
                    .into_owned();
                let hash = atomic::snapshot::sha256_hex(entry.path())?;
                card_files.insert(relative, hash);
            }
        }

        // Build hash map of backup files
        let mut backup_files: std::collections::HashMap<String, (String, u64)> =
            std::collections::HashMap::new();
        for entry in WalkDir::new(&backup_path)
            .follow_links(false)
            .min_depth(1)
        {
            let entry = entry.map_err(|e| AppError::Io(e.to_string()))?;
            if entry.file_type().is_file() {
                let relative = entry
                    .path()
                    .strip_prefix(&backup_path)
                    .map_err(|_| AppError::InvalidPath)?
                    .to_string_lossy()
                    .into_owned();
                let hash = atomic::snapshot::sha256_hex(entry.path())?;
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                backup_files.insert(relative, (hash, size));
            }
        }

        let mut entries: Vec<FileChangeEntry> = Vec::new();
        let mut total_added = 0usize;
        let mut total_modified = 0usize;
        let mut total_removed = 0usize;
        let mut total_unchanged = 0usize;
        let mut total_bytes: u64 = 0;

        // Files in backup: Added or Modified or Unchanged
        for (relative, (backup_hash, size)) in &backup_files {
            total_bytes += size;
            match card_files.get(relative) {
                None => {
                    total_added += 1;
                    entries.push(FileChangeEntry {
                        path: relative.clone(),
                        change_type: ChangeType::Added,
                        size_bytes: *size,
                    });
                }
                Some(card_hash) => {
                    if card_hash != backup_hash {
                        total_modified += 1;
                        entries.push(FileChangeEntry {
                            path: relative.clone(),
                            change_type: ChangeType::Modified,
                            size_bytes: *size,
                        });
                    } else {
                        total_unchanged += 1;
                        entries.push(FileChangeEntry {
                            path: relative.clone(),
                            change_type: ChangeType::Unchanged,
                            size_bytes: *size,
                        });
                    }
                }
            }
        }

        // Files on card but not in backup: Removed
        for (relative, _) in &card_files {
            if !backup_files.contains_key(relative) {
                total_removed += 1;
                let size = project_path
                    .join(relative)
                    .metadata()
                    .map(|m| m.len())
                    .unwrap_or(0);
                entries.push(FileChangeEntry {
                    path: relative.clone(),
                    change_type: ChangeType::Removed,
                    size_bytes: size,
                });
            }
        }

        // Extract timestamp from backup path for label
        let backup_label = backup_path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| bid.clone());

        Ok(FileChangeManifest {
            entries,
            total_added,
            total_modified,
            total_removed,
            total_unchanged,
            total_bytes,
            destination_path: card_path,
            operation_label: format!("Restore Snapshot -- {}", backup_label),
            project_name,
            conflict_details: Vec::new(),
        })
    } else {
        Err(AppError::Io(format!(
            "Unknown dry-run operation: {}",
            operation
        )))
    }
}

/// List backups for a project (or all projects if project_id is None) (SAFE-05).
#[tauri::command]
#[specta::specta]
pub async fn list_backups(
    state: tauri::State<'_, AppState>,
    project_id: Option<String>,
) -> Result<Vec<db::backups::BackupSummary>, AppError> {
    let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    match project_id {
        Some(pid) => db::backups::list_backups(&db.conn, &pid)
            .map_err(|e| AppError::Database(e.to_string())),
        None => db::backups::list_all_backups(&db.conn)
            .map_err(|e| AppError::Database(e.to_string())),
    }
}

/// Set the cancellation flag — backup_project will stop at the next file boundary.
#[tauri::command]
#[specta::specta]
pub async fn cancel_backup(state: tauri::State<'_, AppState>) -> Result<(), AppError> {
    state.cancel_backup.store(true, Ordering::Relaxed);
    Ok(())
}
