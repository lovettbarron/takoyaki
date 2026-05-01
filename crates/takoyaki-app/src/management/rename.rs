//! Project rename business logic.
//!
//! Renames an OT project directory under `/SETS/` on the card volume.
//! The directory name IS the authoritative project name — `project.work` has no
//! internal NAME= field (verified: ot-tools-io OsMetadata/Settings structs, A3).
//!
//! Threat model T-04-01: `validate_ot_name` is called first, restricting the
//! new name to A-Z, a-z, 0-9, underscore, max 16 chars. No path separators
//! are possible, preventing directory traversal.

use crate::error::AppError;
use crate::management::project_work;
use std::path::{Path, PathBuf};

/// Rename an OT project directory to `new_name`.
///
/// Steps:
/// 1. Validate `new_name` via `validate_ot_name`
/// 2. Compute the new directory path (sibling of `project_dir`)
/// 3. Check the target directory does not already exist
/// 4. Perform `std::fs::rename` (both src and dest are under `/SETS/` on the
///    same FAT32 volume — same-filesystem rename is safe; see Pitfall 6)
///
/// Returns the new directory path on success.
///
/// Note: `project.work` and `project.strd` do NOT need modification.
/// OT paths inside those files use `../AUDIO/` relative references that
/// remain valid regardless of the project directory name.
pub fn rename_project(project_dir: &Path, new_name: &str) -> Result<PathBuf, AppError> {
    // T-04-01: Validate name before any filesystem operation.
    project_work::validate_ot_name(new_name)?;

    let parent = project_dir
        .parent()
        .ok_or(AppError::InvalidPath)?;

    let new_dir = parent.join(new_name);

    // Guard against clobbering an existing project.
    if new_dir.exists() {
        return Err(AppError::Io(format!(
            "Directory {} already exists",
            new_name
        )));
    }

    // Same-volume rename — atomic on FAT32 (Pitfall 6).
    std::fs::rename(project_dir, &new_dir)?;

    Ok(new_dir)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_project_dir(tmp: &TempDir, name: &str) -> PathBuf {
        let dir = tmp.path().join("SETS").join(name);
        std::fs::create_dir_all(&dir).unwrap();
        // Create a minimal project.work file
        std::fs::write(
            dir.join("project.work"),
            b"TYPE=FLEX\nSLOT=001\nPATH=../AUDIO/kick.wav\n",
        )
        .unwrap();
        dir
    }

    #[test]
    fn test_rename_project_succeeds() {
        let tmp = TempDir::new().unwrap();
        let project_dir = make_project_dir(&tmp, "OLD_NAME");

        let new_dir = rename_project(&project_dir, "NEW_NAME").unwrap();

        assert!(!project_dir.exists(), "Old directory should be gone");
        assert!(new_dir.exists(), "New directory should exist");
        assert_eq!(new_dir.file_name().unwrap(), "NEW_NAME");
    }

    #[test]
    fn test_rename_project_rejects_invalid_name() {
        let tmp = TempDir::new().unwrap();
        let project_dir = make_project_dir(&tmp, "MY_PROJECT");

        // Space is invalid
        let result = rename_project(&project_dir, "MY PROJECT");
        assert!(result.is_err());
        // The original directory should still exist
        assert!(project_dir.exists());
    }

    #[test]
    fn test_rename_project_rejects_name_too_long() {
        let tmp = TempDir::new().unwrap();
        let project_dir = make_project_dir(&tmp, "MY_PROJECT");

        let result = rename_project(&project_dir, "ABCDEFGHIJKLMNOPQ"); // 17 chars
        assert!(result.is_err());
        assert!(project_dir.exists());
    }

    #[test]
    fn test_rename_project_rejects_collision() {
        let tmp = TempDir::new().unwrap();
        let project_dir = make_project_dir(&tmp, "OLD_NAME");
        let _existing = make_project_dir(&tmp, "EXISTING");

        let result = rename_project(&project_dir, "EXISTING");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("EXISTING") || err_msg.contains("already exists"),
            "Error should mention existing directory: {err_msg}"
        );
        // Original should still be there
        assert!(project_dir.exists());
    }
}
