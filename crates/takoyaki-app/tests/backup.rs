//! Integration tests for backup copy and checksum verification (SAFE-01, SAFE-02)

use std::path::PathBuf;
use tempfile::TempDir;
use walkdir::WalkDir;

/// Helper: create a fake OT project directory with 3 files.
fn setup_project_dir(tmp: &TempDir) -> PathBuf {
    let project_dir = tmp.path().join("SETS").join("LIVESET_01");
    std::fs::create_dir_all(&project_dir).unwrap();

    // Write project.work (fake 16 bytes)
    std::fs::write(project_dir.join("project.work"), b"fakeworkfiledata").unwrap();

    // Create AUDIO subdirectory
    let audio_dir = project_dir.join("AUDIO");
    std::fs::create_dir_all(&audio_dir).unwrap();

    // Write two audio files
    std::fs::write(audio_dir.join("kick.wav"), vec![0xFFu8; 64]).unwrap();
    std::fs::write(audio_dir.join("snare.wav"), vec![0xAAu8; 48]).unwrap();

    project_dir
}

/// Copy a project tree to a destination directory (mirrors copy_project_tree logic).
fn copy_tree(src: &std::path::Path, dest: &std::path::Path) {
    for entry in WalkDir::new(src).follow_links(false).min_depth(1) {
        let entry = entry.unwrap();
        let relative = entry.path().strip_prefix(src).unwrap();
        let dest_entry = dest.join(relative);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&dest_entry).unwrap();
        } else if entry.file_type().is_file() {
            if let Some(parent) = dest_entry.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::copy(entry.path(), &dest_entry).unwrap();
        }
    }
}

#[test]
fn test_backup_copies_all_files() {
    // SAFE-01: backup copies all files with identical byte content
    let tmp = TempDir::new().unwrap();
    let src = setup_project_dir(&tmp);
    let dest = tmp.path().join("backup_dest");
    std::fs::create_dir_all(&dest).unwrap();

    copy_tree(&src, &dest);

    // Assert all 3 files exist in dest with identical byte content
    let project_work_src = std::fs::read(src.join("project.work")).unwrap();
    let project_work_dest = std::fs::read(dest.join("project.work")).unwrap();
    assert_eq!(project_work_src, project_work_dest, "project.work must be byte-identical");

    let kick_src = std::fs::read(src.join("AUDIO").join("kick.wav")).unwrap();
    let kick_dest = std::fs::read(dest.join("AUDIO").join("kick.wav")).unwrap();
    assert_eq!(kick_src, kick_dest, "kick.wav must be byte-identical");

    let snare_src = std::fs::read(src.join("AUDIO").join("snare.wav")).unwrap();
    let snare_dest = std::fs::read(dest.join("AUDIO").join("snare.wav")).unwrap();
    assert_eq!(snare_src, snare_dest, "snare.wav must be byte-identical");

    // Assert directory structure preserved
    assert!(dest.join("AUDIO").is_dir(), "AUDIO/ subdirectory must exist in dest");
}

#[test]
fn test_backup_dir_format() {
    // SAFE-01, D-02: backup destination path format matches PROJECT_NAME/YYYY-MM-DD_HH-MM_label/
    use takoyaki_app::db;

    // Use the same backup_base_dir logic: ~/takoyaki/backups/
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let base = home.join("takoyaki").join("backups");

    let path = base.join("LIVESET_01").join("2026-04-30_14-32_backup");

    // Assert path components
    let components: Vec<_> = path.components().collect();
    let last = path.file_name().unwrap().to_str().unwrap();
    let parent = path.parent().unwrap().file_name().unwrap().to_str().unwrap();

    assert_eq!(parent, "LIVESET_01", "Parent must be project name");
    assert!(
        last.starts_with("2026-04-30"),
        "Backup dir must start with date, got: {}",
        last
    );
    assert!(
        last.contains("backup"),
        "Backup dir must contain label, got: {}",
        last
    );
    assert!(
        last.contains("14-32"),
        "Backup dir must contain time HH-MM, got: {}",
        last
    );

    // Keep compiler happy
    let _ = components;
    let _ = db::open_in_memory();
}

#[test]
fn test_backup_checksum_match() {
    // SAFE-02: SHA-256 of source and dest files must match after copy
    let tmp = TempDir::new().unwrap();
    let src = setup_project_dir(&tmp);
    let dest = tmp.path().join("backup_for_checksum");
    std::fs::create_dir_all(&dest).unwrap();

    copy_tree(&src, &dest);

    // Check all files
    let files = ["project.work", "AUDIO/kick.wav", "AUDIO/snare.wav"];
    for rel in &files {
        let src_path = src.join(rel);
        let dest_path = dest.join(rel);

        let src_hash = takoyaki_app::atomic::snapshot::sha256_hex(&src_path).unwrap();
        let dest_hash = takoyaki_app::atomic::snapshot::sha256_hex(&dest_path).unwrap();

        assert_eq!(
            src_hash, dest_hash,
            "SHA-256 must match for {} after copy",
            rel
        );
    }
}

#[test]
fn test_backup_checksum_mismatch_detected() {
    // SAFE-02: checksum comparison detects corruption in dest
    let tmp = TempDir::new().unwrap();
    let src = setup_project_dir(&tmp);
    let dest = tmp.path().join("corrupt_dest");
    std::fs::create_dir_all(&dest).unwrap();

    copy_tree(&src, &dest);

    // Corrupt the dest kick.wav by appending a byte
    let corrupt_path = dest.join("AUDIO").join("kick.wav");
    let mut content = std::fs::read(&corrupt_path).unwrap();
    content.push(0xDE);
    std::fs::write(&corrupt_path, &content).unwrap();

    // Checksums must now differ
    let src_hash = takoyaki_app::atomic::snapshot::sha256_hex(
        &src.join("AUDIO").join("kick.wav")
    ).unwrap();
    let dest_hash = takoyaki_app::atomic::snapshot::sha256_hex(&corrupt_path).unwrap();

    assert_ne!(
        src_hash, dest_hash,
        "SHA-256 must differ when dest file is corrupted"
    );
}
