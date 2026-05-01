---
phase: 04-advanced-management
reviewed: 2026-05-01T18:36:55Z
depth: standard
files_reviewed: 23
files_reviewed_list:
  - crates/takoyaki-app/Cargo.toml
  - crates/takoyaki-app/src/commands/backup.rs
  - crates/takoyaki-app/src/commands/management.rs
  - crates/takoyaki-app/src/commands/mod.rs
  - crates/takoyaki-app/src/lib.rs
  - crates/takoyaki-app/src/management/bank_copy.rs
  - crates/takoyaki-app/src/management/duplicate.rs
  - crates/takoyaki-app/src/management/export.rs
  - crates/takoyaki-app/src/management/mod.rs
  - crates/takoyaki-app/src/management/project_work.rs
  - crates/takoyaki-app/src/management/rename.rs
  - src/app/page.tsx
  - src/components/backups/DryRunModal.tsx
  - src/components/backups/SnapshotDetailPanel.tsx
  - src/components/management/BankCopyPickerDialog.tsx
  - src/components/project-detail/BankGridCell.tsx
  - src/components/project-detail/BanksTab.tsx
  - src/components/project-detail/MetadataHeader.tsx
  - src/components/project-detail/ProjectDetailView.tsx
  - src/components/ui/context-menu.tsx
  - src/lib/stores/management.ts
  - src/lib/tauri.ts
  - src/lib/types.ts
findings:
  critical: 1
  warning: 4
  info: 3
  total: 8
status: issues_found
---

# Phase 04: Code Review Report

**Reviewed:** 2026-05-01T18:36:55Z
**Depth:** standard
**Files Reviewed:** 23
**Status:** issues_found

## Summary

Phase 04 adds project management operations (duplicate, rename, export, bank copy) across the Rust backend and React frontend. The implementation follows the established patterns from Phase 3 (snapshot-before-write, dry-run preview, atomic writes) with good structural consistency. The safety model is generally well-applied: all destructive operations create pre-operation snapshots, conflict resolution is validated server-side, and name validation prevents path traversal via project names.

However, there is one critical type mismatch between the Rust backend and TypeScript frontend for `ManagementEvent` payloads that will cause all management progress tracking to silently read `undefined` values at runtime. There are also missing path traversal checks on relative paths in the bank copy module, an unused parameter hiding a logic gap in dry-run scoping, and non-atomic audio file writes during bank copy operations.

## Critical Issues

### CR-01: ManagementEvent field names mismatch between Rust and TypeScript (snake_case vs camelCase)

**File:** `src/lib/types.ts:136-140`
**Issue:** The Rust `ManagementEvent` enum uses `#[serde(rename_all = "camelCase")]` (line 33 of `commands/management.rs`), which serializes field names like `files_processed` to `filesProcessed`, `total_files` to `totalFiles`, `current_file` to `currentFile`, and `total_bytes` to `totalBytes`. However, the TypeScript `ManagementEvent` type declares these fields using snake_case (`files_processed`, `total_files`, `current_file`, `total_bytes`). This means every access to `event.data.files_processed` etc. in `page.tsx` lines 304-350 will read `undefined` at runtime, breaking all management operation progress tracking. The `BackupEvent` type is correctly defined with camelCase, so this is an inconsistency specific to the new Phase 4 types.
**Fix:**
```typescript
// In src/lib/types.ts, change ManagementEvent to use camelCase field names:
export type ManagementEvent =
  | { event: "started"; data: { totalFiles: number; destination: string } }
  | { event: "progress"; data: { filesProcessed: number; totalFiles: number; currentFile: string } }
  | { event: "complete"; data: { filesProcessed: number; totalBytes: number; destination: string } }
  | { event: "failed"; data: { reason: string } };
```

Then update all access sites in `src/app/page.tsx` to use camelCase:
```typescript
// Lines 304-309 (and similar blocks at ~325, ~347):
mgmtSetProgress({
    filesProcessed: event.data.filesProcessed,
    totalFiles: event.data.totalFiles,
    currentFile: event.data.currentFile,
});
```

## Warnings

### WR-01: Relative path traversal not checked in bank_copy::resolve_slot_path and management::resolve_audio_path

**File:** `crates/takoyaki-app/src/management/bank_copy.rs:450-458`
**Issue:** For relative paths (e.g., `../AUDIO/kick.wav`), `resolve_slot_path` resolves from `project_dir` without any containment check. A crafted `project.work` could contain a path like `../../../../etc/passwd` which resolves outside the card volume. When the file exists, `canonicalize` is called and the path is returned, which is then used in `std::fs::copy` (line 299) and `sha256_hex` (line 163). The same pattern exists in `commands/management.rs:666-685` (`resolve_audio_path`) and `management/export.rs:239-260`. While OT-absolute paths (backslash-prefixed) go through `resolve_ot_path()` with traversal prevention, relative paths bypass this check entirely.
**Fix:** After resolving relative paths, verify the canonicalized result stays within the card volume:
```rust
fn resolve_slot_path(
    project_dir: &Path,
    card_volume_path: &Path,
    raw_path: &str,
) -> Option<PathBuf> {
    let normalized = raw_path.replace('\\', "/");
    if normalized.starts_with('/') {
        resolve_ot_path(card_volume_path, raw_path)
    } else {
        let resolved = project_dir.join(&normalized);
        if resolved.exists() {
            let canonical = std::fs::canonicalize(&resolved).ok()?;
            let canonical_volume = std::fs::canonicalize(card_volume_path).ok()?;
            if !canonical.starts_with(&canonical_volume) {
                tracing::warn!(
                    "resolve_slot_path: rejecting relative path traversal: {}",
                    canonical.display()
                );
                return None;
            }
            Some(canonical)
        } else {
            Some(resolved)
        }
    }
}
```
Apply the same fix to all three `resolve_slot_path` / `resolve_audio_path` implementations.

### WR-02: _bank_index parameter unused in compute_management_dry_run -- bank-copy dry-run analyzes all slots, not scoped to source bank

**File:** `crates/takoyaki-app/src/commands/management.rs:125`
**Issue:** The `_bank_index: Option<u32>` parameter is accepted but never used. The bank-copy dry-run calls `compute_bank_copy_conflicts()` which analyzes ALL slot assignments in the source project, not just the slots belonging to the specified bank. This means the dry-run manifest shows conflicts for all banks, not just the one being copied. The frontend sends `bankCopySourceIndex` (line 276 of page.tsx) but it has no effect on the result. Similarly, in `bank_copy::copy_bank` (line 266), the conflict analysis covers the entire source project rather than just the specified bank index. The bank files themselves (bank01.work etc.) are correctly scoped by index, but the audio slot analysis is not.
**Fix:** Filter `source_slots` by bank index in `compute_bank_copy_conflicts()`, or pass the bank index through and use it to scope the slot path extraction. This requires understanding the OT binary format's bank-to-slot mapping, which may need to be addressed in a follow-up plan.

### WR-03: Non-atomic audio file copy during bank copy auto_copy step

**File:** `crates/takoyaki-app/src/management/bank_copy.rs:299`
**Issue:** Audio files in the auto_copy step are written using `std::fs::copy()` (a non-atomic operation), while bank files and project.work use `atomic_write_batch()`. The project constraint states "Atomic writes, snapshot-before-write, dry-run preview for ALL operations that modify OT project files. No exceptions." While the audio files are new (not overwrites), a crash during copy could leave a partially-written audio file on the card. The .ot sidecar copy at line 305 has the same issue. The "use-source" conflict resolution at line 354-355 does use `atomic::atomic_write`, so the inconsistency is only in auto_copy.
**Fix:** Use `atomic::atomic_write` for the auto_copy audio files:
```rust
if !dest_audio.exists() {
    let content = std::fs::read(&src_audio)?;
    atomic::atomic_write(&dest_audio, &content)?;
    files_copied += 1;

    // Copy .ot sidecar if present
    let sidecar_src = src_audio.with_file_name(format!("{}.ot", filename));
    if sidecar_src.exists() {
        let sidecar_content = std::fs::read(&sidecar_src)?;
        let sidecar_dest = audio_dir.join(format!("{}.ot", filename));
        atomic::atomic_write(&sidecar_dest, &sidecar_content)?;
        files_copied += 1;
    }
}
```

### WR-04: Duplicate project snapshot failure is silently swallowed

**File:** `crates/takoyaki-app/src/commands/management.rs:419-421`
**Issue:** The pre-duplicate snapshot failure is logged but treated as non-fatal (`if let Err(e) = ...`), allowing the destructive operation to proceed without a safety snapshot. The comment says "Non-fatal: log and continue (snapshot is best-effort for duplicate)" but the project constraint states "snapshot-before-write" for ALL destructive operations with "No exceptions." While duplicate is additive (creates a new directory, does not modify existing files), the inconsistency with rename (line 487, where snapshot failure IS fatal) is confusing. If the intent is that duplicate is truly non-destructive, that should be documented as a deliberate exception.
**Fix:** Either make the snapshot failure fatal (consistent with the safety model):
```rust
snapshot_project(&project_dir, "pre-duplicate")?;
```
Or add a clear comment documenting why this is a deliberate exception to the safety model (duplicate creates new files, never modifies existing ones).

## Info

### IN-01: Fallback to current directory when home_dir() returns None

**File:** `crates/takoyaki-app/src/commands/backup.rs:101-102`
**Issue:** `dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))` falls back to the process working directory if the home directory cannot be determined. This could write backups/snapshots to unexpected locations. The same pattern appears in `commands/management.rs:61`, `commands/backup.rs:454`, and `management/export.rs:46`.
**Fix:** Consider returning an error instead of silently falling back:
```rust
fn backup_base_dir() -> Result<PathBuf, AppError> {
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::Io("Cannot determine home directory".to_string()))?;
    Ok(home.join("takoyaki").join("backups"))
}
```

### IN-02: Code duplication -- resolve_slot_path implemented three times with identical logic

**File:** `crates/takoyaki-app/src/management/bank_copy.rs:440-459`, `crates/takoyaki-app/src/management/export.rs:239-260`, `crates/takoyaki-app/src/commands/management.rs:666-685`
**Issue:** The `resolve_slot_path` / `resolve_audio_path` function is implemented three times with nearly identical logic. This increases the risk that a fix (such as the traversal check in WR-01) is applied to one copy but missed in another.
**Fix:** Extract a single shared `resolve_slot_path` function into `management/project_work.rs` (which already holds the shared slot-related logic) and import it from all three call sites.

### IN-03: export.rs test_compute_export_dest uses fragile string splitting

**File:** `crates/takoyaki-app/src/management/export.rs:299-303`
**Issue:** The test splits the filename `MY_PROJECT_1234.zip` on `_` with `splitn(2, '_')` and asserts `parts[0] == "MY"`, but the project name is `MY_PROJECT` (contains an underscore). The assertion checks the wrong thing -- it verifies that "MY" is the first segment, not that the full project name is present. This makes the test misleading. The test at line 316 (`strip_prefix("TEST_PROJ_")`) handles underscore-containing names correctly by using `strip_prefix` instead.
**Fix:** Use `strip_prefix` or `starts_with` for the assertion:
```rust
assert!(filename.starts_with("MY_PROJECT_"), "Filename should start with project name");
```

---

_Reviewed: 2026-05-01T18:36:55Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
