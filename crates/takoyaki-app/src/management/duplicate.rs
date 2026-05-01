//! Project duplication with sample path remapping.
//!
//! Copies the entire project directory tree to a new name under `/SETS/`.
//! Audio files are shared in the OT's `/AUDIO/` pool — the copied project.work
//! retains the same `../AUDIO/` relative paths, which remain valid from any
//! project directory under `/SETS/`.
//!
//! Per D-01: the duplicate is self-contained — it has its own copies of all
//! project files (project.work, project.strd, bank*.work, bank*.strd). Audio
//! files in /AUDIO/ are shared at the filesystem level; that is how OT works.
//!
//! Threat model T-04-02: All PATH= values in project.work are resolved through
//! `health::resolve_ot_path()` which uses `canonicalize()` for traversal
//! prevention when audio files need to be located.

use crate::error::AppError;
use crate::management::project_work;
use serde::Serialize;
use specta::Type;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Result of a completed project duplication.
#[derive(Debug, Serialize, Type)]
pub struct DuplicateResult {
    pub new_project_dir: PathBuf,
    pub files_copied: usize,
}

// ---------------------------------------------------------------------------
// Public functions
// ---------------------------------------------------------------------------

/// Compute the default duplicate name: `{original_name}_copy` (D-02).
///
/// Note: if the result exceeds 16 characters, the caller must prompt the user
/// for a shorter name — D-03 forbids auto-truncation.
pub fn compute_default_name(original_name: &str) -> String {
    format!("{}_copy", original_name)
}

/// Duplicate an OT project directory to `new_name` under `/SETS/`.
///
/// Steps:
/// 1. Validate `new_name` via `validate_ot_name`
/// 2. Check the destination does not already exist (Pitfall 5)
/// 3. Copy the entire project directory tree with WalkDir
/// 4. The copied project.work and project.strd retain original PATH= entries —
///    relative paths `../AUDIO/` are valid from any project dir under /SETS/
/// 5. Return `DuplicateResult` with destination path and file count
///
/// T-04-02: PATH= values from project.work are not blindly used as filesystem
/// paths here; the duplicate operation does not need to resolve audio paths
/// since it preserves the original relative references unchanged.
pub fn duplicate_project(
    project_dir: &Path,
    new_name: &str,
    card_volume_path: &Path,
) -> Result<DuplicateResult, AppError> {
    // T-04-01: Validate name before any filesystem operation.
    project_work::validate_ot_name(new_name)?;

    let sets_dir = card_volume_path.join("SETS");
    let new_project_dir = sets_dir.join(new_name);

    // Pitfall 5: Guard against clobbering an existing project.
    if new_project_dir.exists() {
        return Err(AppError::Io(format!(
            "Duplicate target {} already exists -- choose a different name",
            new_name
        )));
    }

    // Step 1: Copy the entire project directory tree.
    let files_copied = copy_project_tree(project_dir, &new_project_dir)?;

    Ok(DuplicateResult {
        new_project_dir,
        files_copied,
    })
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Copy all files from `src` to `dest`, creating directories as needed.
///
/// Uses WalkDir with follow_links(false) to prevent symlink traversal
/// (T-03-03 pattern from backup.rs).
///
/// Returns the number of files copied.
fn copy_project_tree(src: &Path, dest: &Path) -> Result<usize, AppError> {
    let mut files_copied = 0usize;

    for entry in WalkDir::new(src).follow_links(false).min_depth(1) {
        let entry = entry.map_err(|e| AppError::Io(e.to_string()))?;
        let entry_path = entry.path();

        let relative = entry_path
            .strip_prefix(src)
            .map_err(|_| AppError::InvalidPath)?
            .to_string_lossy()
            .into_owned();

        let dest_entry = dest.join(&relative);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&dest_entry)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = dest_entry.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(entry_path, &dest_entry)?;
            files_copied += 1;
        }
    }

    Ok(files_copied)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_ot_card(tmp: &TempDir, project_name: &str) -> (PathBuf, PathBuf) {
        let card = tmp.path().to_path_buf();
        let sets = card.join("SETS");
        let audio = card.join("AUDIO");
        std::fs::create_dir_all(&sets).unwrap();
        std::fs::create_dir_all(&audio).unwrap();

        let project_dir = sets.join(project_name);
        std::fs::create_dir_all(&project_dir).unwrap();

        std::fs::write(
            project_dir.join("project.work"),
            b"TYPE=FLEX\nSLOT=001\nPATH=../AUDIO/kick.wav\nGAIN=48\n",
        )
        .unwrap();
        std::fs::write(
            project_dir.join("project.strd"),
            b"TYPE=FLEX\nSLOT=001\nPATH=../AUDIO/kick.wav\nGAIN=48\n",
        )
        .unwrap();
        std::fs::write(audio.join("kick.wav"), b"RIFF....fake wav").unwrap();

        (card, project_dir)
    }

    #[test]
    fn test_compute_default_name() {
        assert_eq!(compute_default_name("LIVESET_01"), "LIVESET_01_copy");
        assert_eq!(compute_default_name("MY_PROJ"), "MY_PROJ_copy");
    }

    #[test]
    fn test_default_name_exceeds_limit() {
        // "ABCDEFGHIJKLMNOP" = 16 chars; + "_copy" = 21 chars
        let original = "ABCDEFGHIJKLMNOP";
        assert_eq!(original.len(), 16);
        let default_name = compute_default_name(original);
        assert_eq!(default_name.len(), 21);

        // validate_ot_name should reject this — D-03 forbids auto-truncation
        let result = project_work::validate_ot_name(&default_name);
        assert!(
            result.is_err(),
            "21-char auto-generated name should fail validation (D-03)"
        );
    }

    #[test]
    fn test_duplicate_project_copies_files() {
        let tmp = TempDir::new().unwrap();
        let (card, project_dir) = make_ot_card(&tmp, "SRC_PROJECT");

        let result = duplicate_project(&project_dir, "DST_PROJECT", &card).unwrap();

        assert!(result.new_project_dir.exists());
        assert_eq!(result.files_copied, 2); // project.work + project.strd
        assert!(result.new_project_dir.join("project.work").exists());
        assert!(result.new_project_dir.join("project.strd").exists());
    }

    #[test]
    fn test_duplicate_project_preserves_content() {
        let tmp = TempDir::new().unwrap();
        let (card, project_dir) = make_ot_card(&tmp, "SRC_PROJECT");

        let result = duplicate_project(&project_dir, "DST_PROJECT", &card).unwrap();

        let orig = std::fs::read(project_dir.join("project.work")).unwrap();
        let copy = std::fs::read(result.new_project_dir.join("project.work")).unwrap();
        assert_eq!(orig, copy, "Copied project.work should be byte-identical");
    }

    #[test]
    fn test_duplicate_project_rejects_collision() {
        let tmp = TempDir::new().unwrap();
        let (card, project_dir) = make_ot_card(&tmp, "SRC_PROJECT");
        // Pre-create the destination
        std::fs::create_dir_all(card.join("SETS").join("EXISTING")).unwrap();

        let result = duplicate_project(&project_dir, "EXISTING", &card);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("already exists"),
            "Error should mention already exists: {msg}"
        );
    }

    #[test]
    fn test_duplicate_project_rejects_invalid_name() {
        let tmp = TempDir::new().unwrap();
        let (card, project_dir) = make_ot_card(&tmp, "SRC_PROJECT");

        let result = duplicate_project(&project_dir, "BAD NAME", &card);
        assert!(result.is_err());
    }
}
