---
phase: 01-foundation
plan: "05"
subsystem: database
tags: [sqlite, rusqlite, atomic-write, snapshot, sha256, migrations, wallflower]

requires:
  - phase: 01-01
    provides: "Cargo workspace with takoyaki-app crate, rusqlite + atomic-write-file + sha2 dependencies"

provides:
  - "migrations/V1__initial_schema.sql — canonical schema for snapshots, snapshot_files, projects tables + 3 indexes"
  - "db::open_database() and db::open_in_memory() free functions with embedded V1 migration"
  - "db::wallflower::open_wallflower_db() — driver-level SQLITE_OPEN_READ_ONLY connection"
  - "atomic::atomic_write() — AtomicWriteFile + sync_all (F_FULLFSYNC on macOS) + parent dir sync"
  - "atomic::atomic_write_batch() — stages all files before committing any"
  - "atomic::snapshot::SnapshotEngine — pre-write file copies with SHA-256 integrity hashes"

affects:
  - "All future plans that write OT project files — must call SnapshotEngine before any atomic_write"
  - "Plans that read Wallflower metadata — use open_wallflower_db"
  - "Phase 2+ commands that need DB access — db::open_database / db::Database"

tech-stack:
  added:
    - "atomic-write-file 0.3 — already in Cargo.toml, now wired into atomic_write()"
    - "sha2 0.10 — already in Cargo.toml, now used in snapshot SHA-256 hashing"
    - "dirs 6 — already in Cargo.toml, used in db::default_path()"
  patterns:
    - "Embedded SQL migration via include_str! + PRAGMA user_version gate"
    - "AtomicWriteFile::options().open() → write_all → flush → sync_all → commit → dir.sync_all()"
    - "SnapshotEngine: timestamp_operation/ directory naming, skip non-existent files"
    - "SQLITE_OPEN_READ_ONLY as driver-level write protection (not convention)"

key-files:
  created:
    - "migrations/V1__initial_schema.sql"
    - "crates/takoyaki-app/src/db/wallflower.rs"
    - "crates/takoyaki-app/src/atomic/mod.rs"
    - "crates/takoyaki-app/src/atomic/snapshot.rs"
  modified:
    - "crates/takoyaki-app/src/db/mod.rs"
    - "crates/takoyaki-app/src/lib.rs"

key-decisions:
  - "Kept Database struct for backward compatibility — free functions open_database/open_in_memory delegate to same initialize() fn"
  - "include_str! path is ../../../../migrations/ (4 levels up from src/db/mod.rs to workspace root)"
  - "unix_timestamp_secs() used instead of chrono for snapshot dir naming (avoids new dependency)"
  - "Wallflower connection: existence check before open_with_flags (SQLite in READ_ONLY mode cannot create new files)"

patterns-established:
  - "Pattern: atomic_write + SnapshotEngine always paired — snapshot before write, never after"
  - "Pattern: open_in_memory() for tests, open_database(&path) for production"
  - "Pattern: PRAGMA journal_mode + foreign_keys + user_version applied on every connection open"

requirements-completed: [FNDN-04, FNDN-05, FNDN-07, FNDN-08, SAFE-03, SAFE-04]

duration: 4min
completed: "2026-04-30"
---

# Phase 01 Plan 05: SQLite Database Layer and Atomic Write Engine Summary

**SQLite V1 schema with embedded migration, SQLITE_OPEN_READ_ONLY Wallflower connection, AtomicWriteFile + F_FULLFSYNC write engine, and SnapshotEngine with SHA-256 integrity hashes**

## Performance

- **Duration:** 4 min
- **Started:** 2026-04-30T04:58:22Z
- **Completed:** 2026-04-30T05:02:03Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- SQLite database initializes with V1 schema (snapshots, snapshot_files, projects tables + 3 indexes) via embedded migration tracked by PRAGMA user_version
- Wallflower DB connection enforces read-only access at the driver level (SQLITE_OPEN_READ_ONLY flag) — verified by test that write attempt returns error
- Atomic write engine stages temp file on same volume as target, calls sync_all (F_FULLFSYNC on macOS), then atomic rename + parent dir sync — satisfies SAFE-04
- Snapshot engine copies all affected files to timestamped directories before any write commits — satisfies SAFE-03
- 28 unit tests pass across db, wallflower, atomic write, and snapshot modules

## Task Commits

Each task was committed atomically:

1. **Task 1: SQLite schema, database module, and Wallflower read-only connection** - `0ac12ec` (feat)
2. **Task 2: Atomic write engine and snapshot engine** - `c0f27bf` (feat)

**Plan metadata:** (docs commit follows)

_Note: TDD tasks — implementation written alongside tests (schema/data files have no meaningful RED state)_

## Files Created/Modified

- `migrations/V1__initial_schema.sql` — V1 schema: snapshots, snapshot_files, projects + 3 indexes
- `crates/takoyaki-app/src/db/mod.rs` — Added open_database(), open_in_memory() free functions, embedded migration via include_str!, retained Database struct for backward compat
- `crates/takoyaki-app/src/db/wallflower.rs` — open_wallflower_db() with SQLITE_OPEN_READ_ONLY (T-01-09 mitigation)
- `crates/takoyaki-app/src/atomic/mod.rs` — atomic_write() and atomic_write_batch() with AtomicWriteFile
- `crates/takoyaki-app/src/atomic/snapshot.rs` — SnapshotEngine with snapshot_files(), SnapshotResult, SHA-256 hashing
- `crates/takoyaki-app/src/lib.rs` — Added mod atomic declaration

## Decisions Made

- Kept the existing `Database` struct for backward compatibility with `commands/projects.rs` which uses `db::Database::open_in_memory()` and `db.conn`. Added free functions that delegate to the same `initialize()` function — no duplicate logic.
- Used `unix_timestamp_secs()` (std::time::SystemTime) instead of `chrono` for snapshot directory naming to avoid adding a new dependency. The timestamp only needs to be monotonically increasing, not human-readable.
- Existence check before `open_with_flags(..., SQLITE_OPEN_READ_ONLY)` — SQLite in read-only mode cannot create a new file and would return a confusing error, so we give a clear `AppError::Io` instead.
- `include_str!` path is `../../../../migrations/` — 4 directory levels up from `crates/takoyaki-app/src/db/mod.rs` to the workspace root.

## Deviations from Plan

None — plan executed exactly as written. The `Database` struct was preserved as a backward-compatibility wrapper rather than removed, which is consistent with the plan's intent (the plan adds new free functions, does not require removing the struct).

## Issues Encountered

- `include_str!` path was initially `../../../migrations/` (3 levels) — compile error caught immediately, corrected to `../../../../migrations/` (4 levels). No impact on committed code.

## User Setup Required

None — no external service configuration required.

## Self-Check: PASSED

- migrations/V1__initial_schema.sql: FOUND
- db/mod.rs: FOUND
- db/wallflower.rs: FOUND
- atomic/mod.rs: FOUND
- atomic/snapshot.rs: FOUND
- 01-05-SUMMARY.md: FOUND
- Commit 0ac12ec (Task 1): FOUND
- Commit c0f27bf (Task 2): FOUND

## Next Phase Readiness

- All safety primitives are in place: every write path in subsequent plans must use `SnapshotEngine::snapshot_files()` before `atomic_write()` or `atomic_write_batch()`
- `db::open_database()` and `db::Database` are both available for commands that need DB access
- Wallflower integration can use `db::wallflower::open_wallflower_db()` whenever the optional feature is implemented
- No blockers for Phase 2 work

---
*Phase: 01-foundation*
*Completed: 2026-04-30*
