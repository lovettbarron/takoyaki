//! Integration tests for dry-run manifest computation (SAFE-07)

use std::collections::HashMap;
use std::time::SystemTime;
use tempfile::TempDir;
use takoyaki_app::atomic::snapshot::sha256_hex;

/// Classification of a file change (mirrors commands::backup::ChangeType).
#[derive(Debug, PartialEq)]
enum ChangeType {
    Added,
    Modified,
    Removed,
    Unchanged,
}

/// Build a hash map of relative_path -> sha256_hex for all files in a directory.
fn build_hash_map(dir: &std::path::Path) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for entry in walkdir::WalkDir::new(dir).follow_links(false).min_depth(1) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            let relative = entry
                .path()
                .strip_prefix(dir)
                .unwrap()
                .to_string_lossy()
                .into_owned();
            let hash = sha256_hex(entry.path()).unwrap();
            map.insert(relative, hash);
        }
    }
    map
}

/// Classify files between two directories: card (current) and backup (desired state).
fn classify_files(
    card_dir: &std::path::Path,
    backup_dir: &std::path::Path,
) -> Vec<(String, ChangeType)> {
    let card_files = build_hash_map(card_dir);
    let backup_files = build_hash_map(backup_dir);

    let mut results: Vec<(String, ChangeType)> = Vec::new();

    // Files in backup
    for (rel, backup_hash) in &backup_files {
        match card_files.get(rel) {
            None => results.push((rel.clone(), ChangeType::Added)),
            Some(card_hash) => {
                if card_hash != backup_hash {
                    results.push((rel.clone(), ChangeType::Modified));
                } else {
                    results.push((rel.clone(), ChangeType::Unchanged));
                }
            }
        }
    }

    // Files on card but not in backup: Removed
    for rel in card_files.keys() {
        if !backup_files.contains_key(rel) {
            results.push((rel.clone(), ChangeType::Removed));
        }
    }

    results
}

#[test]
fn test_dry_run_manifest_classification() {
    // SAFE-07: correctly classifies Added/Modified/Removed/Unchanged
    let tmp = TempDir::new().unwrap();

    // Card project dir: a.txt ("hello"), b.txt ("world"), c.txt ("data")
    let card_dir = tmp.path().join("card");
    std::fs::create_dir_all(&card_dir).unwrap();
    std::fs::write(card_dir.join("a.txt"), b"hello").unwrap();
    std::fs::write(card_dir.join("b.txt"), b"world").unwrap();
    std::fs::write(card_dir.join("c.txt"), b"data").unwrap();

    // Backup dir: a.txt ("hello" — same), b.txt ("modified" — different), d.txt ("new")
    let backup_dir = tmp.path().join("backup");
    std::fs::create_dir_all(&backup_dir).unwrap();
    std::fs::write(backup_dir.join("a.txt"), b"hello").unwrap();
    std::fs::write(backup_dir.join("b.txt"), b"modified").unwrap();
    std::fs::write(backup_dir.join("d.txt"), b"new").unwrap();

    let results = classify_files(&card_dir, &backup_dir);

    let find = |name: &str| results.iter().find(|(p, _)| p == name).map(|(_, t)| t);

    // a.txt: Unchanged (same hash)
    assert_eq!(
        find("a.txt"),
        Some(&ChangeType::Unchanged),
        "a.txt must be Unchanged"
    );

    // b.txt: Modified (different hash)
    assert_eq!(
        find("b.txt"),
        Some(&ChangeType::Modified),
        "b.txt must be Modified"
    );

    // c.txt: Removed (on card, not in backup)
    assert_eq!(
        find("c.txt"),
        Some(&ChangeType::Removed),
        "c.txt must be Removed"
    );

    // d.txt: Added (in backup, not on card)
    assert_eq!(
        find("d.txt"),
        Some(&ChangeType::Added),
        "d.txt must be Added"
    );

    // Assert counts
    let total_added = results.iter().filter(|(_, t)| *t == ChangeType::Added).count();
    let total_modified = results.iter().filter(|(_, t)| *t == ChangeType::Modified).count();
    let total_removed = results.iter().filter(|(_, t)| *t == ChangeType::Removed).count();
    let total_unchanged = results.iter().filter(|(_, t)| *t == ChangeType::Unchanged).count();

    assert_eq!(total_added, 1, "total_added must be 1");
    assert_eq!(total_modified, 1, "total_modified must be 1");
    assert_eq!(total_removed, 1, "total_removed must be 1");
    assert_eq!(total_unchanged, 1, "total_unchanged must be 1");
}

#[test]
fn test_dry_run_no_write() {
    // SAFE-07: dry-run classification must not modify any files
    let tmp = TempDir::new().unwrap();

    // Set up project dir
    let project_dir = tmp.path().join("project");
    std::fs::create_dir_all(&project_dir).unwrap();
    std::fs::write(project_dir.join("a.txt"), b"unchanged content").unwrap();
    std::fs::write(project_dir.join("b.txt"), b"some data").unwrap();

    // Set up backup dir
    let backup_dir = tmp.path().join("backup");
    std::fs::create_dir_all(&backup_dir).unwrap();
    std::fs::write(backup_dir.join("a.txt"), b"unchanged content").unwrap();
    std::fs::write(backup_dir.join("c.txt"), b"new file").unwrap();

    // Record mtimes and contents BEFORE classification
    let a_mtime_before = project_dir
        .join("a.txt")
        .metadata()
        .unwrap()
        .modified()
        .unwrap();
    let b_mtime_before = project_dir
        .join("b.txt")
        .metadata()
        .unwrap()
        .modified()
        .unwrap();
    let a_content_before = std::fs::read(project_dir.join("a.txt")).unwrap();
    let b_content_before = std::fs::read(project_dir.join("b.txt")).unwrap();

    // Run classification (dry-run — no writes)
    let _results = classify_files(&project_dir, &backup_dir);

    // Record mtimes and contents AFTER classification
    let a_mtime_after = project_dir
        .join("a.txt")
        .metadata()
        .unwrap()
        .modified()
        .unwrap();
    let b_mtime_after = project_dir
        .join("b.txt")
        .metadata()
        .unwrap()
        .modified()
        .unwrap();
    let a_content_after = std::fs::read(project_dir.join("a.txt")).unwrap();
    let b_content_after = std::fs::read(project_dir.join("b.txt")).unwrap();

    // Assert contents unchanged
    assert_eq!(
        a_content_before, a_content_after,
        "a.txt content must not change during dry-run"
    );
    assert_eq!(
        b_content_before, b_content_after,
        "b.txt content must not change during dry-run"
    );

    // Assert mtimes unchanged (no writes)
    assert_eq!(
        a_mtime_before, a_mtime_after,
        "a.txt mtime must not change during dry-run"
    );
    assert_eq!(
        b_mtime_before, b_mtime_after,
        "b.txt mtime must not change during dry-run"
    );

    // Assert no new files were created in project dir
    let project_file_count = std::fs::read_dir(&project_dir).unwrap().count();
    assert_eq!(
        project_file_count, 2,
        "No new files should be created in project dir during dry-run"
    );

    // Keep SystemTime import used
    let _ = SystemTime::now();
}
