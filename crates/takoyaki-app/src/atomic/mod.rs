pub mod snapshot;

use atomic_write_file::AtomicWriteFile;
use std::io::Write;
use std::path::Path;
use crate::error::AppError;
use tracing::info;

/// Write content to target_path atomically.
///
/// Process:
/// 1. Create temp file on same volume as target (same directory)
/// 2. Write content to temp file
/// 3. Flush and sync_all (F_FULLFSYNC on macOS — guarantees physical media write)
/// 4. Atomic rename from temp to target
/// 5. Sync parent directory inode
///
/// If any step fails, the original file at target_path remains untouched.
///
/// CRITICAL: The temp file MUST be on the same filesystem as target_path.
/// AtomicWriteFile creates the temp in the same directory by default.
/// If target is on a FAT32 CF card, the temp is also on that card.
/// Cross-filesystem renames are NOT atomic (EXDEV error).
pub fn atomic_write(target_path: &Path, content: &[u8]) -> Result<(), AppError> {
    info!("Atomic write: {} ({} bytes)", target_path.display(), content.len());

    // Ensure parent directory exists
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Stage on same volume as target
    let mut staging = AtomicWriteFile::options()
        .open(target_path)
        .map_err(|e| AppError::Io(format!("Failed to create staging file: {}", e)))?;

    staging.write_all(content)?;
    staging.flush()?;

    // sync_all() calls F_FULLFSYNC on macOS — protects against hot-unplug
    staging.sync_all()?;

    // Atomic rename
    staging
        .commit()
        .map_err(|e| AppError::Io(format!("Failed to commit atomic write: {}", e)))?;

    // Also sync the parent directory inode to ensure the directory entry is flushed
    if let Some(parent) = target_path.parent() {
        if let Ok(dir) = std::fs::File::open(parent) {
            let _ = dir.sync_all(); // Best effort on directory sync
        }
    }

    info!("Atomic write complete: {}", target_path.display());
    Ok(())
}

/// Write multiple files atomically as a batch.
/// All files are staged first, then committed in sequence.
/// If any staging fails, no files are committed.
///
/// snapshot_engine should be called BEFORE this function to preserve originals.
pub fn atomic_write_batch(writes: &[(&Path, &[u8])]) -> Result<(), AppError> {
    info!("Atomic batch write: {} files", writes.len());

    // Stage all files first
    let mut staged: Vec<AtomicWriteFile> = Vec::with_capacity(writes.len());

    for (target_path, content) in writes {
        if let Some(parent) = target_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut staging = AtomicWriteFile::options()
            .open(target_path)
            .map_err(|e| AppError::Io(format!("Failed to stage {}: {}", target_path.display(), e)))?;
        staging.write_all(content)?;
        staging.flush()?;
        staging.sync_all()?;
        staged.push(staging);
    }

    // Commit all staged files
    for staging in staged {
        staging
            .commit()
            .map_err(|e| AppError::Io(format!("Failed to commit: {}", e)))?;
    }

    // Sync parent directories
    let mut synced_dirs = std::collections::HashSet::new();
    for (target_path, _) in writes {
        if let Some(parent) = target_path.parent() {
            if synced_dirs.insert(parent.to_path_buf()) {
                if let Ok(dir) = std::fs::File::open(parent) {
                    let _ = dir.sync_all();
                }
            }
        }
    }

    info!("Atomic batch write complete: {} files", writes.len());
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_atomic_write_creates_file() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("test.bin");
        let content = b"hello takoyaki";
        atomic_write(&target, content).unwrap();
        let read_back = std::fs::read(&target).unwrap();
        assert_eq!(read_back, content);
    }

    #[test]
    fn test_atomic_write_correct_content() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("content.bin");
        let content = b"exact content verification";
        atomic_write(&target, content).unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), content);
    }

    #[test]
    fn test_atomic_write_overwrites_existing() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("test.bin");
        std::fs::write(&target, b"original").unwrap();
        atomic_write(&target, b"updated").unwrap();
        let read_back = std::fs::read(&target).unwrap();
        assert_eq!(read_back, b"updated");
    }

    #[test]
    fn test_atomic_write_batch() {
        let tmp = TempDir::new().unwrap();
        let f1 = tmp.path().join("a.bin");
        let f2 = tmp.path().join("b.bin");
        let writes: Vec<(&Path, &[u8])> = vec![(&f1, b"file a"), (&f2, b"file b")];
        atomic_write_batch(&writes).unwrap();
        assert_eq!(std::fs::read(&f1).unwrap(), b"file a");
        assert_eq!(std::fs::read(&f2).unwrap(), b"file b");
    }

    #[test]
    fn test_atomic_write_creates_parent_dir() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("subdir/nested/test.bin");
        atomic_write(&target, b"nested content").unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), b"nested content");
    }
}
