//! Snapshot engine — copies files before any destructive write operation.
//!
//! Per SAFE-03: a snapshot of all affected files must exist before any write
//! commits. The SnapshotEngine handles creating timestamped snapshot directories
//! and copying files into them with SHA-256 integrity hashes.

use std::path::{Path, PathBuf};
use crate::error::AppError;
use sha2::{Digest, Sha256};
use tracing::info;

/// SnapshotEngine creates file-level snapshots before destructive operations.
/// Snapshots are stored under a root directory, organized by timestamp + operation label.
pub struct SnapshotEngine {
    snapshot_root: PathBuf,
}

impl SnapshotEngine {
    pub fn new(snapshot_root: PathBuf) -> Self {
        SnapshotEngine { snapshot_root }
    }

    /// Create a snapshot of the given files.
    ///
    /// Snapshot directory structure:
    ///   {snapshot_root}/{unix_timestamp}_{operation}/
    ///     {filename_1}
    ///     {filename_2}
    ///     ...
    ///
    /// Files that do not exist on disk are silently skipped (not an error —
    /// new files being created for the first time have nothing to snapshot).
    ///
    /// Returns a `SnapshotResult` describing what was captured.
    pub fn snapshot_files(
        &self,
        files: &[&Path],
        operation: &str,
    ) -> Result<SnapshotResult, AppError> {
        let timestamp = unix_timestamp_secs();
        let snapshot_dir = self
            .snapshot_root
            .join(format!("{}_{}", timestamp, operation));
        std::fs::create_dir_all(&snapshot_dir)?;

        let mut total_bytes: u64 = 0;
        let mut file_records: Vec<SnapshotFileRecord> = Vec::new();

        for src in files {
            if !src.exists() {
                info!("Snapshot: skipping non-existent file {}", src.display());
                continue;
            }
            let filename = src.file_name().ok_or(AppError::InvalidPath)?;
            let dest = snapshot_dir.join(filename);
            let bytes_copied = std::fs::copy(src, &dest)?;
            total_bytes += bytes_copied;

            // Compute SHA-256 hash for integrity verification
            let hash = sha256_hex(&dest)?;

            file_records.push(SnapshotFileRecord {
                original_path: src.to_path_buf(),
                stored_path: dest,
                file_hash: hash,
            });
        }

        info!(
            "Snapshot created: {} files, {} bytes at {}",
            file_records.len(),
            total_bytes,
            snapshot_dir.display()
        );

        Ok(SnapshotResult {
            snapshot_dir,
            operation: operation.to_string(),
            file_count: file_records.len(),
            total_bytes,
            files: file_records,
        })
    }
}

/// Result of a snapshot operation.
#[derive(Debug)]
pub struct SnapshotResult {
    pub snapshot_dir: PathBuf,
    pub operation: String,
    pub file_count: usize,
    pub total_bytes: u64,
    pub files: Vec<SnapshotFileRecord>,
}

/// Record for a single file captured in a snapshot.
#[derive(Debug)]
pub struct SnapshotFileRecord {
    pub original_path: PathBuf,
    pub stored_path: PathBuf,
    pub file_hash: String,
}

/// Return the current Unix timestamp in seconds (used for snapshot directory naming).
fn unix_timestamp_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Compute the SHA-256 hex digest of a file.
fn sha256_hex(path: &Path) -> Result<String, AppError> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_snapshot_files_copies_files() {
        let tmp = TempDir::new().unwrap();
        let snapshot_root = tmp.path().join("snapshots");
        let engine = SnapshotEngine::new(snapshot_root);

        let src_dir = tmp.path().join("source");
        std::fs::create_dir_all(&src_dir).unwrap();
        let f1 = src_dir.join("file1.bin");
        let f2 = src_dir.join("file2.bin");
        std::fs::write(&f1, b"content one").unwrap();
        std::fs::write(&f2, b"content two").unwrap();

        let result = engine
            .snapshot_files(&[f1.as_path(), f2.as_path()], "pre-write")
            .unwrap();

        assert_eq!(result.file_count, 2);
        assert!(result.snapshot_dir.exists());
        assert_eq!(
            std::fs::read(result.snapshot_dir.join("file1.bin")).unwrap(),
            b"content one"
        );
        assert_eq!(
            std::fs::read(result.snapshot_dir.join("file2.bin")).unwrap(),
            b"content two"
        );
    }

    #[test]
    fn test_snapshot_preserves_exact_bytes() {
        let tmp = TempDir::new().unwrap();
        let engine = SnapshotEngine::new(tmp.path().join("snapshots"));

        let src = tmp.path().join("original.bin");
        let content: Vec<u8> = (0u8..=255).collect(); // All 256 byte values
        std::fs::write(&src, &content).unwrap();

        let result = engine
            .snapshot_files(&[src.as_path()], "test")
            .unwrap();
        let snapped = std::fs::read(result.snapshot_dir.join("original.bin")).unwrap();
        assert_eq!(content, snapped, "Snapshot must preserve exact bytes");
    }

    #[test]
    fn test_snapshot_creates_timestamped_dir() {
        let tmp = TempDir::new().unwrap();
        let snapshot_root = tmp.path().join("snapshots");
        let engine = SnapshotEngine::new(snapshot_root);

        let src = tmp.path().join("f.bin");
        std::fs::write(&src, b"data").unwrap();

        let result = engine
            .snapshot_files(&[src.as_path()], "manual")
            .unwrap();
        let dir_name = result.snapshot_dir.file_name().unwrap().to_str().unwrap();
        assert!(
            dir_name.contains("manual"),
            "Directory name '{}' should contain operation label 'manual'",
            dir_name
        );
    }

    #[test]
    fn test_snapshot_skips_nonexistent_files() {
        let tmp = TempDir::new().unwrap();
        let engine = SnapshotEngine::new(tmp.path().join("snapshots"));

        let missing = tmp.path().join("does_not_exist.bin");
        let result = engine
            .snapshot_files(&[missing.as_path()], "test")
            .unwrap();
        assert_eq!(result.file_count, 0, "Non-existent files must be skipped");
    }

    #[test]
    fn test_snapshot_file_hash_present() {
        let tmp = TempDir::new().unwrap();
        let engine = SnapshotEngine::new(tmp.path().join("snapshots"));

        let src = tmp.path().join("hashed.bin");
        std::fs::write(&src, b"hash me").unwrap();

        let result = engine.snapshot_files(&[src.as_path()], "test").unwrap();
        assert_eq!(result.files.len(), 1);
        let hash = &result.files[0].file_hash;
        // SHA-256 hex is 64 characters
        assert_eq!(hash.len(), 64, "SHA-256 hex digest must be 64 characters");
        // Must be all hex chars
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash must be a valid hex string"
        );
    }

    #[test]
    fn test_snapshot_db_record_fields() {
        // Verifies that snapshot results contain all fields needed for DB storage
        let tmp = TempDir::new().unwrap();
        let engine = SnapshotEngine::new(tmp.path().join("snapshots"));

        let src = tmp.path().join("record.bin");
        std::fs::write(&src, b"db record test").unwrap();

        let result = engine.snapshot_files(&[src.as_path()], "backup").unwrap();

        assert_eq!(result.operation, "backup");
        assert_eq!(result.file_count, 1);
        assert!(result.total_bytes > 0);

        let record = &result.files[0];
        assert_eq!(record.original_path, src);
        assert!(record.stored_path.exists());
        assert!(!record.file_hash.is_empty());
    }
}
