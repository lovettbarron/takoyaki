//! Integration tests for restore with pre-restore snapshot (SAFE-06)

use std::path::Path;
use takoyaki_app::atomic;
use takoyaki_app::atomic::snapshot::SnapshotEngine;
use tempfile::TempDir;

#[test]
fn test_restore_creates_pre_snapshot() {
    // SAFE-06, D-11: pre-restore snapshot captures all project files with correct hashes
    let tmp = TempDir::new().unwrap();

    // Create a "project dir" with 2 files (original content)
    let project_dir = tmp.path().join("project");
    std::fs::create_dir_all(&project_dir).unwrap();
    std::fs::write(project_dir.join("project.work"), b"original project data").unwrap();
    std::fs::write(project_dir.join("bank01.work"), b"original bank data").unwrap();

    // Create a "backup dir" with the same 2 files (modified content)
    let backup_dir = tmp.path().join("backup");
    std::fs::create_dir_all(&backup_dir).unwrap();
    std::fs::write(backup_dir.join("project.work"), b"restored project data").unwrap();
    std::fs::write(backup_dir.join("bank01.work"), b"restored bank data").unwrap();

    // Create SnapshotEngine
    let snapshot_root = tmp.path().join("snapshots");
    let engine = SnapshotEngine::new(snapshot_root.clone());

    let project_files: Vec<_> = vec![
        project_dir.join("project.work"),
        project_dir.join("bank01.work"),
    ];
    let file_refs: Vec<&Path> = project_files.iter().map(|p| p.as_path()).collect();

    // Take pre-restore snapshot
    let result = engine.snapshot_files(&file_refs, "pre-restore").unwrap();

    // Assert: SnapshotResult contains records for both original project files
    assert_eq!(result.file_count, 2, "Snapshot must capture both files");

    // Assert: snapshot copies exist on disk
    assert!(
        result.snapshot_dir.exists(),
        "Snapshot directory must exist on disk"
    );
    assert_eq!(
        result.files.len(),
        2,
        "Snapshot must have 2 file records"
    );

    for record in &result.files {
        assert!(
            record.stored_path.exists(),
            "Snapshot copy must exist: {}",
            record.stored_path.display()
        );
    }

    // Assert: SHA-256 of snapshot copies matches SHA-256 of original project files
    for record in &result.files {
        let original_hash =
            takoyaki_app::atomic::snapshot::sha256_hex(&record.original_path).unwrap();
        let snapshot_hash =
            takoyaki_app::atomic::snapshot::sha256_hex(&record.stored_path).unwrap();
        assert_eq!(
            original_hash, snapshot_hash,
            "Snapshot file must have same hash as original: {}",
            record.original_path.display()
        );
    }
}

#[test]
fn test_restore_atomic_write() {
    // SAFE-06, SAFE-04: atomic_write_batch writes files atomically
    let tmp = TempDir::new().unwrap();

    // Create a project file with "original content"
    let project_dir = tmp.path().join("project");
    std::fs::create_dir_all(&project_dir).unwrap();
    let project_file = project_dir.join("project.work");
    std::fs::write(&project_file, b"original content").unwrap();

    // Restore via atomic_write_batch
    let restored_content = b"restored content";
    let writes: Vec<(&Path, &[u8])> = vec![(&project_file, restored_content)];
    atomic::atomic_write_batch(&writes).unwrap();

    // Assert: file now contains restored content
    let actual = std::fs::read(&project_file).unwrap();
    assert_eq!(
        actual, restored_content,
        "File must contain restored content after atomic write"
    );
}
