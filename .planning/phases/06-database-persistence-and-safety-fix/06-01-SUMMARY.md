---
phase: 06-database-persistence-and-safety-fix
plan: 01
subsystem: database
tags: [tdd, persistence, safety, sqlite, rusqlite]
dependency_graph:
  requires: []
  provides: [persistent-db-on-startup, mark-backup-complete-error-guard]
  affects: [crates/takoyaki-app/src/lib.rs, crates/takoyaki-app/src/db/backups.rs]
tech_stack:
  added: []
  patterns: [TDD RED/GREEN, row-count guard, file-backed SQLite]
key_files:
  created:
    - (none)
  modified:
    - crates/takoyaki-app/src/lib.rs
    - crates/takoyaki-app/src/db/backups.rs
    - crates/takoyaki-app/tests/backup_db.rs
    - crates/takoyaki-app/src/db/mod.rs
decisions:
  - "Database::open(&db::default_path()) replaces open_in_memory() in AppState construction"
  - "mark_backup_complete returns Err(QueryReturnedNoRows) when rows_changed == 0"
metrics:
  duration: "2 min"
  completed: "2026-05-06"
  tasks_completed: 2
  files_modified: 4
---

# Phase 06 Plan 01: Database Persistence and Safety Fix Summary

**One-liner:** File-backed SQLite persistence via `Database::open(&default_path())` and row-count guard in `mark_backup_complete` returning `QueryReturnedNoRows` for nonexistent IDs.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Write failing tests (RED phase) | 02fca34 | tests/backup_db.rs, src/db/mod.rs |
| 2 | Apply production fixes (GREEN phase) | b6fb795 | src/lib.rs, src/db/backups.rs |

## What Was Built

Two targeted production fixes plus three new tests, executed in strict TDD RED/GREEN order.

**Fix 1 — Persistent database (SAFE-05, INTG-03):**
`AppState` in `lib.rs` line 133 was calling `db::Database::open_in_memory()`, which meant all backup history and Wallflower settings were lost on every app restart. Changed to `db::Database::open(&db::default_path())`, which resolves to `~/Library/Application Support/takoyaki/takoyaki.db` via `dirs::data_dir()`. The `Database::open()` path already existed and ran all migrations; only the call site needed fixing.

**Fix 2 — Row-count guard (logic error detection):**
`mark_backup_complete` previously discarded the `usize` return value of `conn.execute()` and returned `Ok(())` unconditionally — silently succeeding even when the `WHERE id = ?1` clause matched zero rows. Now captures `rows_changed` and returns `Err(rusqlite::Error::QueryReturnedNoRows)` when `rows_changed == 0`. Both call sites in `commands/backup.rs` already map `rusqlite::Error` to `AppError::Database` via `.map_err()` — no call-site changes required.

**Tests added:**
- `test_mark_backup_complete_unknown_id_returns_err` (backup_db.rs) — integration test confirming the row-count guard (was RED, now GREEN)
- `test_default_path_is_absolute` (db/mod.rs) — regression guard: `default_path()` returns an absolute `PathBuf` ending with `takoyaki/takoyaki.db`
- `test_settings_persist_on_file_db` (db/mod.rs) — regression guard: `set_setting`/`get_setting` round-trips correctly on a file-backed DB

## TDD Gate Compliance

| Gate | Commit | Status |
|------|--------|--------|
| RED (test commit) | 02fca34 | PASS — `test_mark_backup_complete_unknown_id_returns_err` failed as expected |
| GREEN (feat commit) | b6fb795 | PASS — all 96 tests pass including the new test |

## Verification

```
cargo test -p takoyaki-app  →  96 passed; 0 failed
grep "Database::open(&db::default_path())" crates/takoyaki-app/src/lib.rs  →  line 133
grep "rows_changed == 0" crates/takoyaki-app/src/db/backups.rs  →  line 120
grep "QueryReturnedNoRows" crates/takoyaki-app/src/db/backups.rs  →  lines 109, 121
grep "open_in_memory" crates/takoyaki-app/tests/  →  still present (test files unchanged)
```

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Threat Flags

No new security-relevant surface introduced. Both changes are internal to the app startup path and an internal DB helper. The `default_path()` is derived from `dirs::data_dir()` (OS-controlled, no user input). The `QueryReturnedNoRows` error maps to `AppError::Database` via existing error handling — no new trust boundary crossed.

## Self-Check: PASSED

- [x] `crates/takoyaki-app/tests/backup_db.rs` — modified, contains `test_mark_backup_complete_unknown_id_returns_err`
- [x] `crates/takoyaki-app/src/db/mod.rs` — modified, contains `test_default_path_is_absolute` and `test_settings_persist_on_file_db`
- [x] `crates/takoyaki-app/src/lib.rs` — modified, contains `Database::open(&db::default_path())`
- [x] `crates/takoyaki-app/src/db/backups.rs` — modified, contains `rows_changed == 0` and `QueryReturnedNoRows`
- [x] Commit `02fca34` exists (RED phase)
- [x] Commit `b6fb795` exists (GREEN phase)
