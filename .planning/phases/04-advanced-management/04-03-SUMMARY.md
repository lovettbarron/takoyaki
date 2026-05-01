---
phase: 04-advanced-management
plan: 03
subsystem: management-backend
tags: [rust, export, bank-copy, tauri-commands, zip, snapshot, ipc]
dependency_graph:
  requires: [04-01]
  provides: [export-to-zip, bank-copy-with-conflicts, management-ipc-commands]
  affects: [frontend-management-store, 04-04, 04-05]
tech_stack:
  added: []
  patterns:
    - zip-stored-for-audio-deflated-for-text
    - resolve-slot-path-handles-ot-absolute-and-relative
    - db-lock-release-before-file-io
    - pre-operation-snapshot-before-destructive-write
key_files:
  created:
    - crates/takoyaki-app/src/management/export.rs
    - crates/takoyaki-app/src/management/bank_copy.rs
    - crates/takoyaki-app/src/commands/management.rs
  modified:
    - crates/takoyaki-app/src/commands/backup.rs
    - crates/takoyaki-app/src/commands/mod.rs
    - crates/takoyaki-app/src/lib.rs
decisions:
  - resolve_slot_path() handles both OT absolute (backslash-prefixed) and relative (../AUDIO/) path formats — real OT cards use backslash absolute paths, tests use OT absolute format
  - Conflict variant added to ChangeType enum in backup.rs for bank-copy D-08 conflict reporting
  - compute_export_dest() returns Result<PathBuf> (not PathBuf) to surface directory creation errors
  - Pre-duplicate snapshot is non-fatal (log and continue) — duplicate creates new files, original is preserved
  - bank_index parameter in compute_management_dry_run prefixed with _ (accepted but unused — future feature)
metrics:
  duration: "25 min"
  completed_date: "2026-05-01"
  tasks_completed: 2
  files_modified: 6
---

# Phase 4 Plan 03: Export, Bank Copy, and Tauri Commands Summary

Export-to-zip with OT-correct SETS/AUDIO structure and Stored compression for audio, bank copy with SHA-256 conflict detection and three resolution strategies, plus 5 Tauri IPC commands exposing all management operations to the frontend.

## Tasks Completed

### Task 1: Export and bank copy business logic

**Commits:** `8972d80`

**Files:**
- `crates/takoyaki-app/src/management/export.rs` — 416 lines (was 3-line stub)
- `crates/takoyaki-app/src/management/bank_copy.rs` — 669 lines (was 3-line stub)

**export.rs:**
- `export_project()`: creates zip with `SETS/{project_name}/` tree + `AUDIO/` audio files + `.ot` sidecars
- `compute_export_dest()`: `~/takoyaki/exports/{project_name}_{unix_timestamp}.zip` (D-06)
- WAV/AIFF files use `CompressionMethod::Stored`; project files use `Deflated` (per RESEARCH.md A3)
- `zip.finish()` called before return — prevents corrupt central directory (Pitfall 2)
- `resolve_slot_path()` helper handles both OT absolute (`\AUDIO\kick.wav`) and relative (`../AUDIO/`) paths
- 5 unit tests covering: dest path format, timestamp validity, zip structure, zip validity, audio inclusion

**bank_copy.rs:**
- `compute_bank_copy_conflicts()`: reads both project.work files, SHA-256 compares same-named files, returns `BankCopyAnalysis { auto_copy, skip, conflicts }`
- `copy_bank()`: atomically writes bank.work/strd, copies auto-copy audio files with sidecar support, applies conflict resolutions
- T-04-09: conflict resolution values validated against `{"keep-target", "use-source", "rename-incoming"}` before any I/O
- 4 unit tests covering: identical files → skip, empty target slot → auto-copy, conflict validation rejection

### Task 2: Tauri management commands and registration

**Commits:** `719d82c`

**Files:**
- `crates/takoyaki-app/src/commands/management.rs` — 685 lines (new)
- `crates/takoyaki-app/src/commands/mod.rs` — added `pub mod management`
- `crates/takoyaki-app/src/commands/backup.rs` — added `ChangeType::Conflict` variant
- `crates/takoyaki-app/src/lib.rs` — registered 5 commands in `collect_commands![]`

**management.rs commands:**
- `compute_management_dry_run`: returns `FileChangeManifest` for duplicate/rename/export/bank-copy preview
- `duplicate_project`: pre-snapshot + duplicate + Channel progress events
- `rename_project`: pre-snapshot + rename + DB upsert with new card_path/project_name
- `export_project`: no snapshot (read-only) + export + Channel progress events
- `copy_bank`: pre-snapshot ALL target files (Pitfall 3) + bank copy + Channel progress events

All commands follow `backup.rs` patterns: DB lock release before I/O (T-03-04), `#[tauri::command] #[specta::specta]` decoration.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] OT slot path format mismatch in resolve_ot_path**
- **Found during:** Task 1 test execution
- **Issue:** Plan specified `resolve_ot_path(card_volume_path, &slot.path)` for resolving slot paths like `../AUDIO/kick.wav`. This is a relative path from the project directory — but `resolve_ot_path` joins with `card_volume_path` (card root), producing `card_root/../AUDIO/kick.wav` which resolves to the wrong directory.
- **Fix:** Added `resolve_slot_path(project_dir, card_volume_path, raw_path)` helper in export.rs and bank_copy.rs. Handles OT absolute paths (`\AUDIO\kick.wav`) via `resolve_ot_path` and relative paths (`../AUDIO/`) by joining with `project_dir`. Updated test fixtures to use OT absolute path format (`\AUDIO\kick.wav`) consistent with real OT card data.
- **Files modified:** `management/export.rs`, `management/bank_copy.rs`
- **Commit:** `8972d80`

**2. [Rule 1 - Bug] compute_export_dest returned PathBuf but could fail on create_dir_all**
- **Found during:** Task 1 implementation
- **Issue:** Plan signature shows `pub fn compute_export_dest(project_name: &str) -> PathBuf` but `std::fs::create_dir_all` can fail with an I/O error.
- **Fix:** Changed return type to `Result<PathBuf, AppError>` so directory creation failures are surfaced to callers. All call sites updated.
- **Files modified:** `management/export.rs`, `commands/management.rs`
- **Commit:** `8972d80`

## Known Stubs

None — all functions are fully implemented and wired.

## Threat Flags

No new security surface beyond what was planned in the threat model (T-04-05 through T-04-09).

## Self-Check: PASSED

- FOUND: crates/takoyaki-app/src/management/export.rs
- FOUND: crates/takoyaki-app/src/management/bank_copy.rs
- FOUND: crates/takoyaki-app/src/commands/management.rs
- FOUND commit: 8972d80 (Task 1)
- FOUND commit: 719d82c (Task 2)
- cargo check exits 0 — zero compile errors
- All 67 lib unit tests pass + all integration tests pass
