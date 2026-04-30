---
phase: 03-write-path-and-backup
plan: "01"
subsystem: backup-backend
tags: [backup, sqlite, tauri-commands, safety, atomics]
dependency_graph:
  requires: [01-05, 01-07, 02-01]
  provides: [backup-commands, db-backups-module, v2-migration]
  affects: [03-02, 03-03, 03-04]
tech_stack:
  added: [walkdir 2.5]
  patterns: [channel-progress-streaming, atomicbool-cancellation, transaction-insert, sha256-checksum-verification]
key_files:
  created:
    - migrations/V2__backup_schema.sql
    - crates/takoyaki-app/src/db/backups.rs
    - crates/takoyaki-app/src/commands/backup.rs
    - crates/takoyaki-app/tests/backup.rs
    - crates/takoyaki-app/tests/restore.rs
    - crates/takoyaki-app/tests/dry_run.rs
    - crates/takoyaki-app/tests/backup_db.rs
  modified:
    - crates/takoyaki-app/src/db/mod.rs
    - crates/takoyaki-app/src/error.rs
    - crates/takoyaki-app/Cargo.toml
    - crates/takoyaki-app/src/commands/mod.rs
    - crates/takoyaki-app/src/lib.rs
    - crates/takoyaki-app/src/atomic/snapshot.rs
decisions:
  - insert_backup takes &mut Connection (rusqlite transaction() requires mutable borrow)
  - atomic module made pub in lib.rs to allow integration tests to access sha256_hex and SnapshotEngine
  - cleanup_incomplete_backups uses &Connection (no transaction needed — plain DELETE)
  - restore_snapshot records pre-restore snapshot as a backup record with operation="pre-restore" for full audit trail
metrics:
  duration_seconds: 395
  completed_date: "2026-04-30"
  tasks_completed: 3
  files_changed: 13
---

# Phase 03 Plan 01: Write Path and Backup Backend Summary

Complete Rust backend for backup, restore, and dry-run operations — V2 SQLite migration, db::backups CRUD module, five Tauri IPC commands (backup_project, restore_snapshot, compute_dry_run, list_backups, cancel_backup) with Channel-based progress streaming, AtomicBool cancellation, SHA-256 checksum verification, and full integration test coverage for SAFE-01 through SAFE-07.

## What Was Built

### migrations/V2__backup_schema.sql
- `backups` table (id, project_id, project_name, dest_path, created_at, operation, file_count, total_bytes, checksum_ok, status)
- `backup_files` table with CASCADE foreign key to backups
- Three indexes: idx_backups_project_id, idx_backups_created_at, idx_backup_files_backup_id
- No REFERENCES projects(id) on backups.project_id — projects table cleared on re-index (Pitfall 3)

### crates/takoyaki-app/src/db/backups.rs
- `BackupSummary`, `BackupFileRecord` (public, specta::Type)
- `BackupInsert`, `BackupFileInsert` (internal)
- `insert_backup(&mut Connection, ...)` — transaction-based batch insert
- `mark_backup_complete`, `list_backups`, `list_all_backups`, `get_backup_files`, `get_backup_dest_path`, `cleanup_incomplete_backups`, `delete_backup`
- All queries use parameterized `params![]` — no string interpolation (T-02-01)

### crates/takoyaki-app/src/commands/backup.rs
- `BackupEvent` enum with `#[serde(tag = "event", content = "data")]` for Channel streaming
- `FileChangeManifest` + `FileChangeEntry` + `ChangeType` for dry-run output
- `backup_project`: WalkDir copy loop, AtomicBool cancellation per file, SHA-256 verify, Channel progress
- `restore_snapshot`: pre-restore snapshot via SnapshotEngine (D-11), atomic_write_batch (SAFE-04)
- `compute_dry_run`: hash-compare without writing (SAFE-07)
- `list_backups`: delegates to db::backups with optional project_id filter
- `cancel_backup`: sets AtomicBool flag

### lib.rs / AppState
- `cancel_backup: Arc<AtomicBool>` added to AppState
- All five commands registered in collect_commands![]
- D-12 startup cleanup: cleanup_incomplete_backups + fs::remove_dir_all on app launch
- atomic module made pub for integration test access

### Integration Tests
- `tests/backup.rs` (4 tests): SAFE-01 copy fidelity, SAFE-02 checksum match + mismatch detection
- `tests/restore.rs` (2 tests): SAFE-06 pre-restore snapshot, SAFE-04 atomic write
- `tests/dry_run.rs` (2 tests): SAFE-07 correct classification, no-write verification
- `tests/backup_db.rs` (6 tests): SAFE-05 ordering, in-progress exclusion, D-12 cleanup, cascade delete, file records

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] insert_backup requires &mut Connection**
- **Found during:** Task 1 compile
- **Issue:** `conn.transaction()` requires `&mut Connection`; plan showed `&Connection`
- **Fix:** Changed `insert_backup` signature to `pub fn insert_backup(conn: &mut Connection, ...)`
- **Files modified:** crates/takoyaki-app/src/db/backups.rs
- **Commit:** c76cecd

**2. [Rule 1 - Bug] test_user_version_is_1 fails after V2 migration**
- **Found during:** Task 1 test run
- **Issue:** Existing test asserted version == 1; V2 migration sets version to 2
- **Fix:** Renamed test to `test_user_version_is_2` and updated assertion
- **Files modified:** crates/takoyaki-app/src/db/mod.rs
- **Commit:** c76cecd

**3. [Rule 3 - Blocker] tauri::Manager trait not in scope**
- **Found during:** Task 2 compile
- **Issue:** `app.state::<AppState>()` in setup closure requires `use tauri::Manager`
- **Fix:** Added `use tauri::Manager;` to lib.rs
- **Files modified:** crates/takoyaki-app/src/lib.rs
- **Commit:** 40015b0

**4. [Rule 3 - Blocker] atomic module private — integration tests cannot access sha256_hex**
- **Found during:** Task 3 compile
- **Issue:** `mod atomic;` was private; tests/backup.rs, tests/restore.rs, tests/dry_run.rs all need pub access to sha256_hex and SnapshotEngine
- **Fix:** Changed `mod atomic;` to `pub mod atomic;` in lib.rs
- **Files modified:** crates/takoyaki-app/src/lib.rs
- **Commit:** 7bfa008

## Known Stubs

None. All backup/restore/dry-run commands are fully implemented with real file I/O.

## Threat Flags

No new threat surface beyond what is documented in the plan's threat model (T-03-01 through T-03-06). All mitigations are implemented:
- T-03-01: dirs::home_dir() for destination path
- T-03-02: DB lookup to resolve backup_id to dest_path
- T-03-03: WalkDir::new().follow_links(false) in all traversals
- T-03-04: DB lock released before file I/O in all commands

## Self-Check: PASSED
