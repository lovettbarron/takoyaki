//! Integration tests for backup DB CRUD (SAFE-05, D-12)

use takoyaki_app::db::backups::{
    BackupFileInsert, BackupInsert, cleanup_incomplete_backups, delete_backup,
    get_backup_files, insert_backup, list_all_backups, list_backups, mark_backup_complete,
};

/// Helper: open an in-memory database with the V2 backup schema applied.
///
/// Inlines the V2 DDL to avoid touching the filesystem (same pattern as tests/projects.rs).
fn setup_backup_db() -> rusqlite::Connection {
    let conn = rusqlite::Connection::open_in_memory().expect("open_in_memory");
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS backups (
            id           TEXT PRIMARY KEY NOT NULL,
            project_id   TEXT NOT NULL,
            project_name TEXT NOT NULL,
            dest_path    TEXT NOT NULL,
            created_at   TEXT NOT NULL DEFAULT (datetime('now')),
            operation    TEXT NOT NULL,
            file_count   INTEGER NOT NULL,
            total_bytes  INTEGER NOT NULL,
            checksum_ok  INTEGER NOT NULL DEFAULT 1,
            status       TEXT NOT NULL DEFAULT 'in-progress'
        );

        CREATE TABLE IF NOT EXISTS backup_files (
            id            TEXT PRIMARY KEY NOT NULL,
            backup_id     TEXT NOT NULL REFERENCES backups(id) ON DELETE CASCADE,
            relative_path TEXT NOT NULL,
            stored_path   TEXT NOT NULL,
            file_hash     TEXT NOT NULL,
            size_bytes    INTEGER NOT NULL,
            change_type   TEXT NOT NULL
        );

        PRAGMA foreign_keys = ON;",
    )
    .expect("create V2 schema");
    conn
}

/// Helper: construct a BackupInsert with one BackupFileInsert for testing.
fn make_backup_insert(
    id: &str,
    project_id: &str,
    project_name: &str,
    operation: &str,
    created_at: &str,
) -> BackupInsert {
    BackupInsert {
        id: id.to_string(),
        project_id: project_id.to_string(),
        project_name: project_name.to_string(),
        dest_path: format!("/backups/{}/{}", project_name, id),
        created_at: created_at.to_string(),
        operation: operation.to_string(),
        file_count: 1,
        total_bytes: 128,
        checksum_ok: true,
        status: "in-progress".to_string(),
        files: vec![BackupFileInsert {
            id: format!("file-{}", id),
            relative_path: "project.work".to_string(),
            stored_path: format!("/backups/{}/{}/project.work", project_name, id),
            file_hash: format!("hash-{}", id),
            size_bytes: 128,
            change_type: "backup".to_string(),
        }],
    }
}

#[test]
fn test_insert_and_list_backups() {
    // SAFE-05: insert and list returns correct records ordered DESC
    let mut conn = setup_backup_db();

    let mut b1 = make_backup_insert("b1", "proj-1", "LIVESET_01", "backup", "2026-01-01T10:00:00");
    let mut b2 = make_backup_insert("b2", "proj-1", "LIVESET_01", "backup", "2026-02-01T10:00:00");

    insert_backup(&mut conn, &b1).unwrap();
    insert_backup(&mut conn, &b2).unwrap();

    mark_backup_complete(&conn, "b1", true).unwrap();
    mark_backup_complete(&conn, "b2", true).unwrap();

    let results = list_backups(&conn, "proj-1").unwrap();
    assert_eq!(results.len(), 2, "Must return 2 records");

    // Most recent first (2026-02-01 > 2026-01-01)
    assert_eq!(results[0].id, "b2", "Most recent backup must be first");
    assert_eq!(results[1].id, "b1", "Older backup must be second");

    // Keep compiler happy
    b1.status = "complete".to_string();
    b2.status = "complete".to_string();
}

#[test]
fn test_list_backups_ordered() {
    // SAFE-05: list_backups returns results ordered by created_at DESC
    let mut conn = setup_backup_db();

    let dates = [
        ("id-a", "2026-01-01T00:00:00"),
        ("id-b", "2026-03-01T00:00:00"),
        ("id-c", "2026-02-01T00:00:00"),
    ];

    for (id, date) in &dates {
        let record = make_backup_insert(id, "proj-x", "TECHNO_01", "backup", date);
        insert_backup(&mut conn, &record).unwrap();
        mark_backup_complete(&conn, id, true).unwrap();
    }

    let results = list_backups(&conn, "proj-x").unwrap();
    assert_eq!(results.len(), 3);

    assert_eq!(results[0].created_at, "2026-03-01T00:00:00", "First must be 2026-03-01");
    assert_eq!(results[1].created_at, "2026-02-01T00:00:00", "Second must be 2026-02-01");
    assert_eq!(results[2].created_at, "2026-01-01T00:00:00", "Third must be 2026-01-01");
}

#[test]
fn test_list_backups_excludes_in_progress() {
    // Only completed backups appear in list_backups
    let mut conn = setup_backup_db();

    let r1 = make_backup_insert("complete-1", "proj-y", "AMBIENT", "backup", "2026-01-01T00:00:00");
    let r2 = make_backup_insert("incomplete-1", "proj-y", "AMBIENT", "backup", "2026-01-02T00:00:00");

    insert_backup(&mut conn, &r1).unwrap();
    insert_backup(&mut conn, &r2).unwrap();

    // Only mark r1 complete
    mark_backup_complete(&conn, "complete-1", true).unwrap();

    let results = list_backups(&conn, "proj-y").unwrap();
    assert_eq!(results.len(), 1, "Only completed backup must appear");
    assert_eq!(results[0].id, "complete-1");
}

#[test]
fn test_cleanup_incomplete_backups() {
    // D-12: cleanup_incomplete_backups removes in-progress records and returns their paths
    let mut conn = setup_backup_db();

    let complete = make_backup_insert("done-1", "proj-z", "SET_01", "backup", "2026-01-01T00:00:00");
    let incomplete = make_backup_insert("partial-1", "proj-z", "SET_01", "backup", "2026-01-02T00:00:00");

    insert_backup(&mut conn, &complete).unwrap();
    insert_backup(&mut conn, &incomplete).unwrap();
    mark_backup_complete(&conn, "done-1", true).unwrap();

    // Cleanup returns the dest_path of the in-progress backup
    let cleaned = cleanup_incomplete_backups(&conn).unwrap();
    assert_eq!(cleaned.len(), 1, "Must return 1 incomplete backup path");
    assert!(
        cleaned[0].contains("partial-1"),
        "Returned path must contain the incomplete backup ID"
    );

    // Only the complete backup remains
    let remaining = list_all_backups(&conn).unwrap();
    assert_eq!(remaining.len(), 1, "Only the completed backup must remain");
    assert_eq!(remaining[0].id, "done-1");
}

#[test]
fn test_delete_backup_cascades() {
    // CASCADE delete: deleting a backup also removes its backup_files
    let mut conn = setup_backup_db();

    // Insert backup with 2 backup_files
    let mut record = make_backup_insert("del-1", "proj-d", "DRUM_SET", "backup", "2026-01-01T00:00:00");
    record.files = vec![
        BackupFileInsert {
            id: "file-del-1a".to_string(),
            relative_path: "project.work".to_string(),
            stored_path: "/backup/project.work".to_string(),
            file_hash: "hash-a".to_string(),
            size_bytes: 64,
            change_type: "backup".to_string(),
        },
        BackupFileInsert {
            id: "file-del-1b".to_string(),
            relative_path: "bank01.work".to_string(),
            stored_path: "/backup/bank01.work".to_string(),
            file_hash: "hash-b".to_string(),
            size_bytes: 128,
            change_type: "backup".to_string(),
        },
    ];

    insert_backup(&mut conn, &record).unwrap();
    mark_backup_complete(&conn, "del-1", true).unwrap();

    // Verify files exist before delete
    let files_before = get_backup_files(&conn, "del-1").unwrap();
    assert_eq!(files_before.len(), 2, "Must have 2 backup files before delete");

    // Delete the backup
    delete_backup(&conn, "del-1").unwrap();

    // Files must be CASCADE deleted
    let files_after = get_backup_files(&conn, "del-1").unwrap();
    assert_eq!(files_after.len(), 0, "Backup files must be CASCADE deleted");

    // Backup no longer in list
    let remaining = list_all_backups(&conn).unwrap();
    assert_eq!(remaining.len(), 0, "Backup must be gone from list");
}

#[test]
fn test_get_backup_files() {
    // Verify get_backup_files returns all file records with correct fields
    let mut conn = setup_backup_db();

    let mut record = make_backup_insert("files-1", "proj-f", "FILE_TEST", "backup", "2026-01-01T00:00:00");
    record.files = vec![
        BackupFileInsert {
            id: "f1".to_string(),
            relative_path: "project.work".to_string(),
            stored_path: "/backup/project.work".to_string(),
            file_hash: "aabbcc".to_string(),
            size_bytes: 64,
            change_type: "backup".to_string(),
        },
        BackupFileInsert {
            id: "f2".to_string(),
            relative_path: "bank01.work".to_string(),
            stored_path: "/backup/bank01.work".to_string(),
            file_hash: "ddeeff".to_string(),
            size_bytes: 128,
            change_type: "backup".to_string(),
        },
        BackupFileInsert {
            id: "f3".to_string(),
            relative_path: "AUDIO/kick.wav".to_string(),
            stored_path: "/backup/AUDIO/kick.wav".to_string(),
            file_hash: "112233".to_string(),
            size_bytes: 4096,
            change_type: "backup".to_string(),
        },
    ];

    insert_backup(&mut conn, &record).unwrap();
    mark_backup_complete(&conn, "files-1", true).unwrap();

    let files = get_backup_files(&conn, "files-1").unwrap();
    assert_eq!(files.len(), 3, "Must return 3 file records");

    // Verify fields
    let project_work = files.iter().find(|f| f.relative_path == "project.work");
    assert!(project_work.is_some(), "project.work must be in file records");
    assert_eq!(project_work.unwrap().file_hash, "aabbcc");

    let kick = files.iter().find(|f| f.relative_path == "AUDIO/kick.wav");
    assert!(kick.is_some(), "AUDIO/kick.wav must be in file records");
    assert_eq!(kick.unwrap().size_bytes, 4096);
}
