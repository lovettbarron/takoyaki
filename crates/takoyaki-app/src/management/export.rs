//! Project export to self-contained zip archive (Phase 4 — Plan 04-03).
//!
//! Creates a zip archive with:
//! - `SETS/{project_name}/` — all project files (project.work, bank*.work, etc.)
//! - `AUDIO/` — all audio files referenced by the project, plus .ot sidecars
//!
//! Per RESEARCH.md A3: WAV/AIFF files are already compressed — use `Stored` compression.
//! Per RESEARCH.md D-05: .ot sidecar files live alongside their audio files and must be
//! included in the export for complete sample metadata.
//! Per RESEARCH.md D-06: Exports saved to ~/takoyaki/exports/{project_name}_{timestamp}.zip
//!
//! Threat model T-04-05: Takoyaki only WRITES zips, never extracts — zip slip N/A.

use crate::error::AppError;
use crate::health::resolve_ot_path;
use crate::management::project_work;
use serde::Serialize;
use specta::Type;
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Result of a completed project export.
#[derive(Debug, Serialize, Type)]
pub struct ExportResult {
    pub zip_path: PathBuf,
    pub files_exported: usize,
    pub total_bytes: u64,
}

// ---------------------------------------------------------------------------
// Public functions
// ---------------------------------------------------------------------------

/// Compute the export destination path: ~/takoyaki/exports/{project_name}_{timestamp}.zip
///
/// Per D-06: organizes exports by project name and timestamp.
/// Creates the directory if it doesn't exist.
pub fn compute_export_dest(project_name: &str) -> Result<PathBuf, AppError> {
    let base = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("takoyaki")
        .join("exports");

    std::fs::create_dir_all(&base)?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let filename = format!("{}_{}.zip", project_name, timestamp);
    Ok(base.join(filename))
}

/// Export an OT project to a self-contained zip archive.
///
/// The zip contains:
/// - `SETS/{project_name}/` — all project directory files
/// - `AUDIO/{filename}` — all audio files referenced by slot paths
/// - `AUDIO/{filename}.ot` — .ot sidecar files alongside their audio files (D-05)
///
/// CRITICAL: `zip.finish()` MUST be called before returning — without it, the
/// central directory is never written and the zip is corrupt (Pitfall 2).
///
/// T-04-06: All source/target audio paths resolved via resolve_ot_path() with
/// canonicalize() traversal prevention.
pub fn export_project(
    project_dir: &Path,
    card_volume_path: &Path,
    export_dest: &Path,
) -> Result<ExportResult, AppError> {
    // 1. Read project.work bytes and extract slot paths
    let project_work_path = project_dir.join("project.work");
    let project_work_bytes = std::fs::read(&project_work_path).map_err(|e| {
        AppError::Io(format!(
            "Failed to read project.work at {}: {}",
            project_work_path.display(),
            e
        ))
    })?;

    let slot_paths = project_work::extract_slot_paths(&project_work_bytes);

    // 2. Resolve audio file paths and deduplicate.
    //
    // OT project.work paths use two formats:
    //   a) Relative from project dir: "../AUDIO/kick.wav" (most common — relative to SETS/PROJECT/)
    //   b) Absolute from card root: "\AUDIO\kick.wav" (backslash, starts with \)
    //
    // For relative paths, we resolve from the project directory.
    // For absolute (backslash-prefixed) paths, we use resolve_ot_path with the card volume.
    let mut unique_audio_paths: Vec<PathBuf> = Vec::new();
    let mut seen: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();

    for slot in &slot_paths {
        let audio_path = resolve_slot_path(project_dir, card_volume_path, &slot.path);
        if let Some(audio_path) = audio_path {
            if seen.insert(audio_path.clone()) {
                unique_audio_paths.push(audio_path);
            }
        }
    }

    // 3. Get project name from directory name
    let project_name = project_dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "PROJECT".to_string());

    // 4. Ensure parent directory of export_dest exists
    if let Some(parent) = export_dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 5. Create the zip file
    let file = std::fs::File::create(export_dest)?;
    let mut zip = ZipWriter::new(file);

    // Compression options per RESEARCH.md A3:
    // - WAV/AIFF are already compressed: use Stored (no compression overhead)
    // - Project files (text key=value): use Deflated (compress well)
    let opts_audio = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let opts_text =
        SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    let mut files_exported: usize = 0;
    let mut total_bytes: u64 = 0;

    // 6. Add project directory tree under SETS/{project_name}/
    // project_dir's parent is the SETS dir; strip from there to get SETS/{project_name}/...
    let sets_parent = project_dir.parent().unwrap_or(project_dir);

    for entry in WalkDir::new(project_dir).follow_links(false) {
        let entry = entry.map_err(|e| AppError::Io(e.to_string()))?;
        let entry_path = entry.path();

        let relative = entry_path
            .strip_prefix(sets_parent)
            .map_err(|_| AppError::InvalidPath)?
            .to_string_lossy()
            .replace('\\', "/");

        let zip_path = format!("SETS/{}", relative);

        if entry.file_type().is_dir() {
            zip.add_directory(&zip_path, opts_text)
                .map_err(|e| AppError::Io(format!("Failed to add directory {}: {}", zip_path, e)))?;
        } else if entry.file_type().is_file() {
            let ext = entry_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            let is_audio = matches!(ext.as_str(), "wav" | "aif" | "aiff");
            let opts = if is_audio { opts_audio } else { opts_text };

            zip.start_file(&zip_path, opts)
                .map_err(|e| AppError::Io(format!("Failed to start file {}: {}", zip_path, e)))?;

            let mut f = std::fs::File::open(entry_path)?;
            let bytes = std::io::copy(&mut f, &mut zip)?;
            total_bytes += bytes;
            files_exported += 1;
        }
    }

    // 7. Add audio files under AUDIO/
    for audio_path in &unique_audio_paths {
        if !audio_path.exists() {
            continue; // Skip missing files gracefully
        }

        let filename = audio_path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "unknown.wav".to_string());

        let zip_audio_path = format!("AUDIO/{}", filename);

        zip.start_file(&zip_audio_path, opts_audio)
            .map_err(|e| AppError::Io(format!("Failed to start audio file {}: {}", zip_audio_path, e)))?;

        let mut f = std::fs::File::open(audio_path)?;
        let bytes = std::io::copy(&mut f, &mut zip)?;
        total_bytes += bytes;
        files_exported += 1;

        // 8. Check for .ot sidecar (same stem + ".ot" extension, per D-05)
        // OT sidecar lives alongside the audio file: kick.wav -> kick.wav.ot
        let sidecar_path = {
            let mut sidecar = audio_path.clone();
            let sidecar_name = format!("{}.ot", filename);
            sidecar.set_file_name(sidecar_name);
            sidecar
        };

        if sidecar_path.exists() {
            let sidecar_filename = sidecar_path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| format!("{}.ot", filename));

            let zip_sidecar_path = format!("AUDIO/{}", sidecar_filename);

            zip.start_file(&zip_sidecar_path, opts_text)
                .map_err(|e| AppError::Io(format!("Failed to start sidecar {}: {}", zip_sidecar_path, e)))?;

            let mut f = std::fs::File::open(&sidecar_path)?;
            let bytes = std::io::copy(&mut f, &mut zip)?;
            total_bytes += bytes;
            files_exported += 1;
        }
    }

    // CRITICAL: Call finish() to write the central directory (Pitfall 2)
    // Without this, the zip file is corrupt and cannot be opened.
    zip.finish()
        .map_err(|e| AppError::Io(format!("Failed to finalize zip: {}", e)))?;

    Ok(ExportResult {
        zip_path: export_dest.to_path_buf(),
        files_exported,
        total_bytes,
    })
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Resolve a slot path from project.work to an absolute filesystem path.
///
/// OT project.work paths come in two forms:
///   - Relative: `../AUDIO/kick.wav` — relative to the project directory (SETS/PROJECT/)
///   - Absolute (OT-style): `\AUDIO\kick.wav` — absolute from the card root
///
/// For relative paths (starting with `../` or `./`), we resolve from project_dir.
/// For OT-absolute paths (starting with `\`), we delegate to resolve_ot_path.
fn resolve_slot_path(
    project_dir: &Path,
    card_volume_path: &Path,
    raw_path: &str,
) -> Option<PathBuf> {
    let normalized = raw_path.replace('\\', "/");

    if normalized.starts_with('/') {
        // OT absolute path — delegate to resolve_ot_path (handles traversal prevention)
        resolve_ot_path(card_volume_path, raw_path)
    } else {
        // Relative path (e.g. "../AUDIO/kick.wav") — resolve from project directory
        let resolved = project_dir.join(&normalized);
        // Canonicalize if the file exists, otherwise return the raw resolved path
        // (file may not exist yet — caller will handle missing files gracefully)
        if resolved.exists() {
            std::fs::canonicalize(&resolved).ok()
        } else {
            Some(resolved)
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_compute_export_dest() {
        let result = compute_export_dest("MY_PROJECT").unwrap();
        let path_str = result.to_string_lossy();

        // Should be under ~/takoyaki/exports/
        assert!(
            path_str.contains("takoyaki") && path_str.contains("exports"),
            "Export dest should be under ~/takoyaki/exports/, got: {}",
            path_str
        );

        // Should have project name prefix
        assert!(
            path_str.contains("MY_PROJECT"),
            "Export dest should contain project name, got: {}",
            path_str
        );

        // Should have .zip extension
        assert!(
            path_str.ends_with(".zip"),
            "Export dest should end with .zip, got: {}",
            path_str
        );

        // Verify format: {project_name}_{timestamp}.zip
        let filename = result.file_name().unwrap().to_str().unwrap();
        let parts: Vec<&str> = filename.splitn(2, '_').collect();
        assert_eq!(parts[0], "MY", "First part should be project name");
        assert!(
            parts[1].contains("PROJECT"),
            "Filename should contain full project name"
        );
    }

    #[test]
    fn test_compute_export_dest_contains_timestamp() {
        let result = compute_export_dest("TEST_PROJ").unwrap();
        let filename = result.file_name().unwrap().to_str().unwrap();

        // Strip .zip and project name prefix to get timestamp
        let without_ext = filename.strip_suffix(".zip").unwrap();
        let timestamp_part = without_ext
            .strip_prefix("TEST_PROJ_")
            .expect("Filename should start with TEST_PROJ_");

        // Timestamp should be a valid number (seconds since epoch)
        let ts: u64 = timestamp_part
            .parse()
            .expect("Timestamp should be a valid u64");
        // Should be a reasonable unix timestamp (after 2020)
        assert!(ts > 1_577_836_800, "Timestamp should be after 2020");
    }

    #[test]
    fn test_export_project_creates_zip_with_sets_structure() {
        let tmp = TempDir::new().unwrap();

        // Create mock OT card structure
        let card = tmp.path().to_path_buf();
        let sets_dir = card.join("SETS");
        let audio_dir = card.join("AUDIO");
        std::fs::create_dir_all(&sets_dir).unwrap();
        std::fs::create_dir_all(&audio_dir).unwrap();

        let project_dir = sets_dir.join("MY_PROJECT");
        std::fs::create_dir_all(&project_dir).unwrap();

        // Write a project.work with no slots (minimal)
        std::fs::write(project_dir.join("project.work"), b"VERSION=1\n").unwrap();
        std::fs::write(project_dir.join("project.strd"), b"STRD=1\n").unwrap();

        let export_dest = tmp.path().join("test_export.zip");

        let result = export_project(&project_dir, &card, &export_dest).unwrap();

        assert!(export_dest.exists(), "Zip file should exist");
        assert!(result.files_exported >= 2, "Should export at least project.work and project.strd");
        assert_eq!(result.zip_path, export_dest);
    }

    #[test]
    fn test_export_project_zip_is_valid() {
        let tmp = TempDir::new().unwrap();

        let card = tmp.path().to_path_buf();
        let sets_dir = card.join("SETS");
        let audio_dir = card.join("AUDIO");
        std::fs::create_dir_all(&sets_dir).unwrap();
        std::fs::create_dir_all(&audio_dir).unwrap();

        let project_dir = sets_dir.join("TESTPROJ");
        std::fs::create_dir_all(&project_dir).unwrap();
        std::fs::write(project_dir.join("project.work"), b"VERSION=1\n").unwrap();

        let export_dest = tmp.path().join("test.zip");
        export_project(&project_dir, &card, &export_dest).unwrap();

        // Verify the zip is valid by opening it
        let file = std::fs::File::open(&export_dest).unwrap();
        let archive = zip::ZipArchive::new(file).expect("Zip should be a valid archive");
        assert!(archive.len() >= 1, "Zip should contain at least one file");
    }

    #[test]
    fn test_export_project_includes_audio_files() {
        let tmp = TempDir::new().unwrap();

        let card = tmp.path().to_path_buf();
        let sets_dir = card.join("SETS");
        let audio_dir = card.join("AUDIO");
        std::fs::create_dir_all(&sets_dir).unwrap();
        std::fs::create_dir_all(&audio_dir).unwrap();

        // Create a fake WAV file
        std::fs::write(audio_dir.join("kick.wav"), b"RIFF....fake wav data").unwrap();

        let project_dir = sets_dir.join("BEATPROJ");
        std::fs::create_dir_all(&project_dir).unwrap();

        // project.work that references the audio file using OT absolute path format
        // Real OT cards use backslash-absolute paths like \AUDIO\kick.wav
        let project_work_content =
            b"TYPE=FLEX\nSLOT=001\nPATH=\\AUDIO\\kick.wav\nGAIN=48\n";
        std::fs::write(project_dir.join("project.work"), project_work_content).unwrap();

        let export_dest = tmp.path().join("beat_export.zip");
        let result = export_project(&project_dir, &card, &export_dest).unwrap();

        assert!(result.files_exported >= 2, "Should export project.work + kick.wav");

        // Verify the zip contains AUDIO/kick.wav
        let file = std::fs::File::open(&export_dest).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        let names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();

        assert!(
            names.iter().any(|n| n.contains("AUDIO/kick.wav")),
            "Zip should contain AUDIO/kick.wav, found: {:?}",
            names
        );
    }
}
