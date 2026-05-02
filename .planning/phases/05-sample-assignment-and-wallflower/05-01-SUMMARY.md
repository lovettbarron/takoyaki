---
phase: 05-sample-assignment-and-wallflower
plan: 01
subsystem: sample-assignment-backend
tags: [rust, tauri, sqlite, tdd, sample-assignment, atomic-write, snapshot]
dependency_graph:
  requires: []
  provides:
    - compute_sample_dry_run Tauri command
    - assign_sample Tauri command
    - V3 settings migration with wallflower_db_path
    - get_setting / set_setting DB helpers
    - tauri-plugin-dialog registration
  affects:
    - crates/takoyaki-app/src/commands/samples.rs
    - crates/takoyaki-app/src/db/mod.rs
    - crates/takoyaki-app/src/lib.rs
tech_stack:
  added:
    - tauri-plugin-dialog = "2" (Cargo.toml)
    - "@tauri-apps/plugin-dialog" ^2.7.0 (package.json)
  patterns:
    - TDD RED/GREEN cycle for Tauri command unit tests
    - snapshot-before-write (SAFE-03) via SnapshotEngine
    - atomic_write_batch for all-or-nothing project file writes (SAFE-04)
    - DB lock release before file I/O (T-03-04 pattern)
    - FormatIssue enum dispatch for hard_block vs soft_warnings
key_files:
  created:
    - migrations/V3__wallflower_settings.sql
  modified:
    - crates/takoyaki-app/src/commands/samples.rs
    - crates/takoyaki-app/src/db/mod.rs
    - crates/takoyaki-app/src/lib.rs
    - crates/takoyaki-app/Cargo.toml
    - package.json
decisions:
  - "FormatIssue variants are WrongSampleRate/WrongBitDepth/UnsupportedFormat — plan sample used different names; matched actual enum"
  - "FileChangeManifest has extended fields (total_added/modified/etc.) — used all fields with zero values for dry-run"
  - "get_card_path is the DB function (not get_project_dir) — plan referenced non-existent function; corrected"
  - "snapshot_root is a local helper function (same pattern as management.rs) — not a DB function as plan implied"
  - "OptionalExtension trait must be imported in db/mod.rs for .optional() on query_row result"
metrics:
  duration: "~15 min"
  completed: "2026-05-02T10:57:26Z"
  tasks_completed: 2
  files_modified: 7
---

# Phase 5 Plan 01: Sample Assignment Backend Summary

**One-liner:** Two Tauri commands for format-validated, snapshot-protected sample slot assignment, plus V3 migration for Wallflower settings and tauri-plugin-dialog registration.

## What Was Built

### Task 1: Plugin installation, V3 migration, settings helpers

- `tauri-plugin-dialog = "2"` added to `Cargo.toml`; `@tauri-apps/plugin-dialog` installed via npm
- `migrations/V3__wallflower_settings.sql` creates a `settings` table with `wallflower_db_path` key (empty default = use auto-discovery)
- `db/mod.rs` runs V3 migration at `user_version < 3` and exposes `get_setting()` / `set_setting()` for the settings table
- `tauri_plugin_dialog::init()` registered in `tauri::Builder` chain in `lib.rs`

### Task 2: compute_sample_dry_run and assign_sample (TDD)

**compute_sample_dry_run:**
- Validates `file_path` via `canonicalize()` + `is_file()` (T-05-01 path traversal prevention)
- Validates `slot_type` enum: only "flex" / "static" accepted (T-05-02)
- Validates `slot_index` 0..=127 (T-05-03)
- Hard block for non-WAV/AIFF: maps `FormatIssue::UnsupportedFormat` → `hard_block` (D-14)
- Soft warnings for non-44.1kHz (`WrongSampleRate`) and non-16/24-bit (`WrongBitDepth`)
- Hard block for Flex slot with file > 200MB (D-13); Static slot: no size restriction
- Returns `FileChangeManifest` listing `project.work` + `project.strd` as Modified

**assign_sample:**
- Copies Wallflower-origin files to OT `/AUDIO/` before any rewrite when `from_wallflower=true` (RESEARCH Pitfall 5; T-05-05 dest hardcoded)
- Snapshots `project.work` + `project.strd` via `SnapshotEngine` before any modification (SAFE-03)
- Rewrites both files via `project_work::rewrite_slot_path` using 1-indexed slot numbers
- Warning logged if `rewrite_slot_path` returns unchanged bytes (A2 assertion guard)
- Writes both files atomically via `atomic_write_batch` (SAFE-04)

Both commands registered in `lib.rs` `collect_commands!` macro.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] FormatIssue variant names mismatch**
- **Found during:** Task 2 implementation
- **Issue:** Plan's code sample referenced `FormatIssue::IncompatibleFormat`, `SampleRateWarning`, `BitDepthWarning` — actual enum variants are `UnsupportedFormat`, `WrongSampleRate`, `WrongBitDepth`
- **Fix:** Used actual variant names from `health/mod.rs`
- **Files modified:** `commands/samples.rs`

**2. [Rule 1 - Bug] FileChangeManifest missing required fields**
- **Found during:** Task 2 implementation
- **Issue:** Plan's sample only showed `operation_label`, `entries`, `total_files` — actual struct has `total_added`, `total_modified`, `total_removed`, `total_unchanged`, `total_bytes`, `destination_path`, `project_name`, `conflict_details`, and `FileChangeEntry` requires `size_bytes`
- **Fix:** Constructed `FileChangeManifest` with all required fields; used zero values for sizes in dry-run context
- **Files modified:** `commands/samples.rs`

**3. [Rule 1 - Bug] get_project_dir DB function does not exist**
- **Found during:** Task 2 implementation
- **Issue:** Plan referenced `db::projects::get_project_dir` — actual function is `db::projects::get_card_path`
- **Fix:** Used `db::projects::get_card_path` (which returns the project directory path)
- **Files modified:** `commands/samples.rs`

**4. [Rule 1 - Bug] db::backups::snapshot_root does not exist**
- **Found during:** Task 2 implementation
- **Issue:** Plan referenced `db::backups::snapshot_root(&db.conn)` — this function does not exist; snapshot_root is a local helper in management.rs
- **Fix:** Added local `snapshot_root()` helper (same pattern as `management.rs`)
- **Files modified:** `commands/samples.rs`

**5. [Rule 2 - Missing critical] OptionalExtension import missing in db/mod.rs**
- **Found during:** Task 2 RED phase compilation
- **Issue:** `get_setting()` uses `.optional()` on query_row result, which requires `rusqlite::OptionalExtension` trait in scope
- **Fix:** Added `OptionalExtension` to the `use rusqlite::` import
- **Files modified:** `crates/takoyaki-app/src/db/mod.rs`

## TDD Gate Compliance

- RED commit: `ed679b2` — `test(05-01): add failing tests for compute_sample_dry_run and assign_sample`
- GREEN commit: `88aafd9` — `feat(05-01): implement compute_sample_dry_run and assign_sample Tauri commands`
- REFACTOR: not needed — implementation was clean on first pass

## Known Stubs

None — all data flows are wired. The `project.work` reader in `get_project_samples` is a pre-existing stub from Phase 2 (separate concern from this plan's sample assignment commands).

## Self-Check: PASSED

| Item | Status |
|------|--------|
| migrations/V3__wallflower_settings.sql | FOUND |
| crates/takoyaki-app/src/commands/samples.rs | FOUND |
| .planning/phases/05-sample-assignment-and-wallflower/05-01-SUMMARY.md | FOUND |
| Commit 14a477d (Task 1: plugin + migration) | FOUND |
| Commit ed679b2 (RED: failing tests) | FOUND |
| Commit 88aafd9 (GREEN: implementation) | FOUND |
| cargo check -p takoyaki-app | PASSED (0 errors, 2 pre-existing warnings) |
| cargo test commands::samples | 13/13 PASSED |
