//! SQLite query functions for backup history (SAFE-01, SAFE-05).
//! Threat model T-02-01: All values use parameterized queries — never string interpolation.

use rusqlite::{params, Connection};
use specta::Type;

/// A backup summary row as returned by list queries (SAFE-05).
#[derive(Debug, serde::Serialize, Type, Clone)]
pub struct BackupSummary {
    pub id: String,
    pub project_id: String,
    pub project_name: String,
    pub dest_path: String,
    pub created_at: String,
    pub operation: String,
    pub file_count: i64,
    pub total_bytes: i64,
    pub checksum_ok: bool,
    pub status: String,
}

/// A backup file record as returned by get_backup_files.
#[derive(Debug, serde::Serialize, Type, Clone)]
pub struct BackupFileRecord {
    pub id: String,
    pub backup_id: String,
    pub relative_path: String,
    pub stored_path: String,
    pub file_hash: String,
    pub size_bytes: i64,
    pub change_type: String,
}

/// Internal: one file to insert when creating a backup record.
#[derive(Debug)]
pub struct BackupFileInsert {
    pub id: String,
    pub relative_path: String,
    pub stored_path: String,
    pub file_hash: String,
    pub size_bytes: i64,
    pub change_type: String,
}

/// Internal: a full backup record to insert (backup row + file manifest).
#[derive(Debug)]
pub struct BackupInsert {
    pub id: String,
    pub project_id: String,
    pub project_name: String,
    pub dest_path: String,
    pub created_at: String,
    pub operation: String,
    pub file_count: i64,
    pub total_bytes: i64,
    pub checksum_ok: bool,
    pub status: String,
    pub files: Vec<BackupFileInsert>,
}

/// Insert a backup record and all its file entries in a single transaction (SAFE-05).
///
/// Uses `conn.transaction()` to batch-insert the backup row and all backup_files rows.
/// All values use parameterized queries — never string interpolation (T-02-01).
pub fn insert_backup(conn: &mut Connection, record: &BackupInsert) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;

    tx.execute(
        "INSERT INTO backups
            (id, project_id, project_name, dest_path, created_at, operation, file_count, total_bytes, checksum_ok, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            record.id,
            record.project_id,
            record.project_name,
            record.dest_path,
            record.created_at,
            record.operation,
            record.file_count,
            record.total_bytes,
            record.checksum_ok as i64,
            record.status,
        ],
    )?;

    for file in &record.files {
        tx.execute(
            "INSERT INTO backup_files
                (id, backup_id, relative_path, stored_path, file_hash, size_bytes, change_type)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                file.id,
                record.id,
                file.relative_path,
                file.stored_path,
                file.file_hash,
                file.size_bytes,
                file.change_type,
            ],
        )?;
    }

    tx.commit()?;
    Ok(())
}

/// Mark a backup as complete and set checksum_ok status.
pub fn mark_backup_complete(
    conn: &Connection,
    backup_id: &str,
    checksum_ok: bool,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE backups SET status = 'complete', checksum_ok = ?2 WHERE id = ?1",
        params![backup_id, checksum_ok as i64],
    )?;
    Ok(())
}

/// List completed backups for a project, ordered by created_at DESC (SAFE-05).
pub fn list_backups(
    conn: &Connection,
    project_id: &str,
) -> rusqlite::Result<Vec<BackupSummary>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, project_name, dest_path, created_at, operation,
                file_count, total_bytes, checksum_ok, status
         FROM backups
         WHERE project_id = ?1 AND status = 'complete'
         ORDER BY created_at DESC",
    )?;

    let rows = stmt.query_map(params![project_id], |row| {
        Ok(BackupSummary {
            id: row.get(0)?,
            project_id: row.get(1)?,
            project_name: row.get(2)?,
            dest_path: row.get(3)?,
            created_at: row.get(4)?,
            operation: row.get(5)?,
            file_count: row.get(6)?,
            total_bytes: row.get(7)?,
            checksum_ok: row.get::<_, i64>(8)? != 0,
            status: row.get(9)?,
        })
    })?;

    rows.collect()
}

/// List all completed backups across all projects, ordered by created_at DESC.
pub fn list_all_backups(conn: &Connection) -> rusqlite::Result<Vec<BackupSummary>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, project_name, dest_path, created_at, operation,
                file_count, total_bytes, checksum_ok, status
         FROM backups
         WHERE status = 'complete'
         ORDER BY created_at DESC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(BackupSummary {
            id: row.get(0)?,
            project_id: row.get(1)?,
            project_name: row.get(2)?,
            dest_path: row.get(3)?,
            created_at: row.get(4)?,
            operation: row.get(5)?,
            file_count: row.get(6)?,
            total_bytes: row.get(7)?,
            checksum_ok: row.get::<_, i64>(8)? != 0,
            status: row.get(9)?,
        })
    })?;

    rows.collect()
}

/// Get all file records for a backup.
pub fn get_backup_files(
    conn: &Connection,
    backup_id: &str,
) -> rusqlite::Result<Vec<BackupFileRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, backup_id, relative_path, stored_path, file_hash, size_bytes, change_type
         FROM backup_files
         WHERE backup_id = ?1",
    )?;

    let rows = stmt.query_map(params![backup_id], |row| {
        Ok(BackupFileRecord {
            id: row.get(0)?,
            backup_id: row.get(1)?,
            relative_path: row.get(2)?,
            stored_path: row.get(3)?,
            file_hash: row.get(4)?,
            size_bytes: row.get(5)?,
            change_type: row.get(6)?,
        })
    })?;

    rows.collect()
}

/// Get the destination path for a backup by ID.
pub fn get_backup_dest_path(conn: &Connection, backup_id: &str) -> rusqlite::Result<String> {
    conn.query_row(
        "SELECT dest_path FROM backups WHERE id = ?1",
        params![backup_id],
        |row| row.get(0),
    )
}

/// Delete in-progress backups and return their dest_paths for filesystem cleanup (D-12).
///
/// Returns the list of dest_paths of incomplete backups so the caller can remove them
/// from the filesystem. Then deletes all in-progress backup rows (CASCADE removes files).
pub fn cleanup_incomplete_backups(conn: &Connection) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT dest_path FROM backups WHERE status = 'in-progress'",
    )?;
    let paths: rusqlite::Result<Vec<String>> = stmt
        .query_map([], |row| row.get(0))?
        .collect();
    let paths = paths?;

    conn.execute("DELETE FROM backups WHERE status = 'in-progress'", [])?;

    Ok(paths)
}

/// Delete a backup and its file records (CASCADE handles backup_files).
pub fn delete_backup(conn: &Connection, backup_id: &str) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM backups WHERE id = ?1",
        params![backup_id],
    )?;
    Ok(())
}
