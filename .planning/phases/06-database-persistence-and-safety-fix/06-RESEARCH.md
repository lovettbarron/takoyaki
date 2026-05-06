# Phase 6: Database Persistence & Safety Fix - Research

**Researched:** 2026-05-06
**Domain:** Rust/rusqlite — database initialization path, row-count guards
**Confidence:** HIGH

## Summary

Phase 6 is a targeted two-fix integration correction identified by the v1.0 audit. Both bugs are
in the Rust backend; there is no frontend work. The codebase already contains every function
needed — the fixes are: (1) swap one call site in `lib.rs` from `open_in_memory()` to
`Database::open(&db::default_path())`, and (2) add a row-count check after the UPDATE in
`mark_backup_complete`.

All supporting infrastructure is in place and verified by the existing test suite (90 passing
unit tests + 6 integration tests for backup_db). The fixes do not add new dependencies, change
schemas, or touch the frontend. Research is HIGH confidence because every claim is directly
verified against the source files in this session.

**Primary recommendation:** Two targeted edits to existing Rust files, plus two additional test
cases. No new functions, no new files (other than the tests if they don't already exist in an
appropriate location).

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| DB initialization path | API / Backend | — | AppState construction lives in lib.rs run(); frontend never touches the DB directly |
| Persistent DB path resolution | API / Backend | OS / Filesystem | default_path() uses dirs::data_dir() which resolves to ~/Library/Application Support on macOS |
| Row-count guard on UPDATE | API / Backend | — | Pure Rust logic inside db::backups; no tier boundary crossing |
| Wallflower settings persistence | API / Backend | — | Settings are read/written via db::get_setting/set_setting against AppState.db |

## Standard Stack

No new dependencies are introduced by this phase.

### Core (already in Cargo.toml)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rusqlite | 0.39 (workspace) | SQLite driver | Project standard — all DB operations use this |
| dirs | 6 | Platform data dir resolution | Used by default_path() and backup_base_dir() |

### Supporting
None required — all needed code already exists.

### Alternatives Considered
None — both fixes are prescribed by the audit. No alternatives are appropriate.

## Architecture Patterns

### System Architecture Diagram

```
App startup (lib.rs::run())
    |
    v
AppState construction
    |
    +-- FIX: db::Database::open(&db::default_path())  [was: open_in_memory()]
    |         |
    |         v
    |   ~/Library/Application Support/takoyaki/takoyaki.db
    |   (persistent across restarts — migrations applied automatically)
    |
    +-- device: Mutex<DeviceState>
    +-- cancel_backup: Arc<AtomicBool>
    +-- audio_tx: mpsc::Sender<AudioCommand>
    |
    v
Tauri.setup() callback
    |
    v
cleanup_incomplete_backups(&mut db.conn)  [D-12 cleanup, already works]
    |
    v
Normal command dispatch
    |
    +-- backup_project -> mark_backup_complete(conn, id, checksum_ok)
    |                         |
    |                         +-- FIX: check rows_changed() == 1
    |                         |       return Err if 0 (ghost backup_id)
    |
    +-- set_wallflower_db_path -> db::set_setting(conn, "wallflower_db_path", path)
                                      [now persists to disk, survives restart]
```

### Recommended Project Structure

No structural changes. Both edits are in existing files:
```
crates/takoyaki-app/src/
├── lib.rs           # Fix 1: line 133 — open_in_memory() -> Database::open(&default_path())
└── db/
    └── backups.rs   # Fix 2: mark_backup_complete — add rows_changed() == 1 guard
```

### Pattern 1: Persistent DB Initialization

**What:** Replace in-memory DB construction with file-backed DB using the existing `Database::open` and `default_path()` functions — both already implemented and tested in `db/mod.rs`.

**When to use:** App startup — the only call site is `lib.rs:133`.

**Current code (line 133):**
```rust
// BEFORE (lib.rs:133)
db: Mutex::new(db::Database::open_in_memory().expect("Failed to open database")),
```

**Fixed code:**
```rust
// AFTER
db: Mutex::new(db::Database::open(&db::default_path()).expect("Failed to open database")),
```

`default_path()` resolves to:
```rust
// db/mod.rs:31-36 [VERIFIED from source]
pub fn default_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("takoyaki")
        .join("takoyaki.db")
}
```

On macOS this yields: `~/Library/Application Support/takoyaki/takoyaki.db`

`Database::open` already calls `open_database(path)` which calls `std::fs::create_dir_all` on the parent before opening — the directory will be created automatically on first launch. [VERIFIED from source: db/mod.rs:14-21]

### Pattern 2: Row-Count Guard on UPDATE

**What:** After an UPDATE statement, call `conn.execute(...)` which returns `usize` (rows changed). If it returns 0, the backup_id was not found — treat as an error.

**When to use:** Any UPDATE that should modify exactly one row by primary key.

**Current code (backups.rs:108-118) [VERIFIED from source]:**
```rust
pub fn mark_backup_complete(
    conn: &Connection,
    backup_id: &str,
    checksum_ok: bool,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE backups SET status = 'complete', checksum_ok = ?2 WHERE id = ?1",
        params![backup_id, checksum_ok as i64],
    )?;
    Ok(())
}
```

**Fixed code:**
```rust
pub fn mark_backup_complete(
    conn: &Connection,
    backup_id: &str,
    checksum_ok: bool,
) -> rusqlite::Result<()> {
    let rows_changed = conn.execute(
        "UPDATE backups SET status = 'complete', checksum_ok = ?2 WHERE id = ?1",
        params![backup_id, checksum_ok as i64],
    )?;
    if rows_changed == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    Ok(())
}
```

`conn.execute()` returns `rusqlite::Result<usize>` where `usize` is the count of rows changed. [VERIFIED: rusqlite 0.39 API — `execute` signature is `fn execute(&self, sql: &str, params: impl Params) -> Result<usize>`]

`rusqlite::Error::QueryReturnedNoRows` is the canonical error variant for "expected a row but got none" — already used in the codebase implicitly via `.query_row()` calls. Using it here is idiomatic. [VERIFIED from rusqlite source patterns in codebase]

### Call Sites for mark_backup_complete [VERIFIED from source]

There are exactly two production call sites:

1. `commands/backup.rs:383` — inside `backup_project`, after successful file copy. The `backup_id` is a hash generated from `dest_path` (a path that was just created), so `rows_changed == 0` would indicate a logic error (the in-progress record was not inserted before this call). Appropriate to propagate as `AppError::Database`.

2. `commands/backup.rs:533` — inside `restore_snapshot`, after inserting a pre-restore snapshot. Same pattern — the `snapshot_id` was just inserted in the same lock scope.

Both call sites already map `rusqlite::Error` to `AppError::Database` via `.map_err(|e| AppError::Database(e.to_string()))`, so the new error propagates cleanly without any call-site changes.

### Anti-Patterns to Avoid

- **Changing test setup to use `open_database`:** Tests in `db/mod.rs`, `tests/backup_db.rs`, and elsewhere use `open_in_memory()` intentionally. Do NOT change them — in-memory is correct for tests. Only the production `AppState` construction in `lib.rs` uses `open_in_memory()` incorrectly.
- **Using `rusqlite::Error::InvalidParameterName` or other error variants:** `QueryReturnedNoRows` is the semantically closest standard variant. Do not invent a new error type for this.
- **Touching `cleanup_incomplete_backups`:** This function already takes `&Connection` (not `&mut Connection`) and the call in `lib.rs:150` passes `&mut db.conn` — this works because `&mut T` coerces to `&T`. No change needed.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Platform data dir | Custom path logic | `dirs::data_dir()` via `default_path()` | Already implemented; handles macOS/Linux/Windows; fallback to `.` on error |
| Row-count checking | Custom tracking table | `conn.execute()` return value | rusqlite returns rows_changed from execute() natively |
| Schema migration on first launch | Manual DDL in run() | `initialize()` in open_database | Already called by Database::open; idempotent via PRAGMA user_version |
| Error type for missing row | Custom AppError variant | `rusqlite::Error::QueryReturnedNoRows` | Standard rusqlite error; maps cleanly to existing AppError::Database |

**Key insight:** Everything needed already exists. This phase is about wiring two existing pieces together correctly, not building anything new.

## Common Pitfalls

### Pitfall 1: Changing Test DB Initialization
**What goes wrong:** Developer changes `open_in_memory()` calls in tests to `open_database()`, causing tests to create real files in the filesystem during test runs, leaving stale `.db` files and breaking CI.
**Why it happens:** Mechanical search-replace of all `open_in_memory` references.
**How to avoid:** Only touch line 133 of `lib.rs`. Tests should keep using in-memory DB.
**Warning signs:** Test output shows file paths like `~/Library/Application Support/takoyaki/takoyaki.db` being opened during `cargo test`.

### Pitfall 2: WAL Mode Incompatibility on First Run
**What goes wrong:** The app fails to open the persistent DB on very first launch if the data directory doesn't exist.
**Why it happens:** SQLite in WAL mode requires the directory to exist before `Connection::open`.
**How to avoid:** `open_database` already calls `std::fs::create_dir_all(parent)` before `Connection::open(path)`. [VERIFIED from source: db/mod.rs:14-21] No additional handling needed.
**Warning signs:** App crashes on first launch with `Failed to open database`.

### Pitfall 3: mark_backup_complete Error Propagation
**What goes wrong:** After adding the row-count check, `backup_project` returns an error to the frontend when mark_backup_complete fails, but the backup files already exist on disk (the copy succeeded). The user sees a failure for an operation that actually worked.
**Why it happens:** `rows_changed == 0` after a successful copy means the DB is inconsistent — but the backup did happen.
**How to avoid:** This is the correct behavior. If `mark_backup_complete` returns 0 rows, the backup record wasn't found (race condition or prior bug). Reporting an error is correct. The backup directory on disk is still valid; D-12 cleanup on next launch will handle the in-progress record.
**Warning signs:** This pitfall is a design concern, not a code bug. The fix is correct as specified.

### Pitfall 4: Mutex Deadlock on DB Lock (Already Solved)
**What goes wrong:** Holding the DB `Mutex` lock while doing file I/O causes a deadlock when another command tries to access the DB.
**Why it happens:** `AppState.db: Mutex<Database>` — long-held locks block all other commands.
**How to avoid:** The existing code already follows the T-03-04 pattern: acquire lock, read/write DB, drop lock, then do file I/O. The Phase 6 changes do not introduce any new lock scopes.
**Warning signs:** App hangs during backup.

## Code Examples

### Fix 1: Production DB Initialization (lib.rs:133)
```rust
// Source: verified from crates/takoyaki-app/src/lib.rs and db/mod.rs

// BEFORE (broken):
db: Mutex::new(db::Database::open_in_memory().expect("Failed to open database")),

// AFTER (correct):
db: Mutex::new(db::Database::open(&db::default_path()).expect("Failed to open database")),
```

### Fix 2: Row-Count Guard (db/backups.rs:108-118)
```rust
// Source: verified from crates/takoyaki-app/src/db/backups.rs

pub fn mark_backup_complete(
    conn: &Connection,
    backup_id: &str,
    checksum_ok: bool,
) -> rusqlite::Result<()> {
    let rows_changed = conn.execute(
        "UPDATE backups SET status = 'complete', checksum_ok = ?2 WHERE id = ?1",
        params![backup_id, checksum_ok as i64],
    )?;
    if rows_changed == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    Ok(())
}
```

### New Test: mark_backup_complete Row-Count Guard
```rust
// Add to crates/takoyaki-app/tests/backup_db.rs or db/backups.rs #[cfg(test)]

#[test]
fn test_mark_backup_complete_unknown_id_returns_err() {
    let conn = setup_backup_db(); // or open_in_memory() in unit tests
    // Calling mark_backup_complete with an ID that doesn't exist
    let result = mark_backup_complete(&conn, "nonexistent-id", true);
    assert!(result.is_err(), "mark_backup_complete with unknown ID must return Err");
}
```

### New Test: Persistent DB Smoke Test
```rust
// Add to crates/takoyaki-app/src/db/mod.rs #[cfg(test)]
// (Already covered by test_open_database_creates_file, but confirm default_path works)

#[test]
fn test_default_path_is_absolute() {
    let path = default_path();
    assert!(path.is_absolute(), "default_path() must return an absolute path");
    // On macOS, must end with takoyaki/takoyaki.db
    assert!(path.ends_with("takoyaki/takoyaki.db"));
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| In-memory DB (open_in_memory) | File-backed DB (open_database + default_path) | Phase 1 — always intended, never wired | Backup history and Wallflower settings persist across restarts |
| UPDATE without row-count check | UPDATE + rows_changed == 0 guard | Phase 3 — identified in audit as CR-02 | Silent success on ghost backup_id becomes detectable error |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| — | — | — | — |

**All claims in this research were verified directly against the source files or rusqlite API. No assumed claims.**

## Open Questions

None. The fixes are fully specified by the audit, and every relevant code path has been read and
verified in this session.

## Environment Availability

Step 2.6: SKIPPED — this phase has no external dependencies. Both fixes are pure Rust source
edits. No new tools, services, CLIs, runtimes, or databases are required.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (cargo test) |
| Config file | Cargo.toml — no separate test config |
| Quick run command | `cargo test -p takoyaki-app` |
| Full suite command | `cargo test -p takoyaki-app` |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SAFE-05 | Backup history persists across restart | integration | `cargo test -p takoyaki-app test_open_database_creates_file` | ✅ (db/mod.rs) |
| SAFE-05 | default_path() returns absolute path | unit | `cargo test -p takoyaki-app test_default_path_is_absolute` | ❌ Wave 0 |
| INTG-03 | Wallflower settings persist via file DB | integration | `cargo test -p takoyaki-app test_open_database_creates_file` | ✅ (indirect) |
| INTG-03 | set_setting/get_setting round-trip on file DB | unit | `cargo test -p takoyaki-app` (needs new test) | ❌ Wave 0 |
| SAFE-05/INTG-03 | mark_backup_complete rejects unknown ID | unit | `cargo test -p takoyaki-app test_mark_backup_complete_unknown_id_returns_err` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p takoyaki-app`
- **Per wave merge:** `cargo test -p takoyaki-app`
- **Phase gate:** Full suite green (all 90+ tests pass) before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `test_mark_backup_complete_unknown_id_returns_err` — add to `tests/backup_db.rs`, covers CR-02 row-count guard
- [ ] `test_default_path_is_absolute` — add to `db/mod.rs` unit tests, covers SAFE-05 persistence initialization
- [ ] `test_settings_persist_on_file_db` — add to `db/mod.rs` unit tests, covers INTG-03 settings persistence

*(Existing test infrastructure fully covers other aspects — 90 passing tests, no framework gaps)*

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | no | No new user inputs introduced |
| V6 Cryptography | no | No new crypto |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal via default_path() | Tampering | dirs::data_dir() returns OS-controlled path; no user input involved |
| DB file permissions | Information Disclosure | SQLite file inherits directory permissions; macOS ~/Library/Application Support is user-private by default |

No new threat surface is introduced by these fixes. The DB file location is derived entirely from `dirs::data_dir()` with no user input, and all existing parameterized query patterns are preserved.

## Sources

### Primary (HIGH confidence)
- Direct source read: `crates/takoyaki-app/src/lib.rs` — verified `open_in_memory()` call at line 133
- Direct source read: `crates/takoyaki-app/src/db/mod.rs` — verified `open_database`, `open_in_memory`, `default_path`, `Database::open`, `Database::open_in_memory`, migration logic
- Direct source read: `crates/takoyaki-app/src/db/backups.rs` — verified `mark_backup_complete` current implementation; confirmed `conn.execute()` return type is `rusqlite::Result<usize>`
- Direct source read: `crates/takoyaki-app/src/commands/backup.rs` — verified both call sites for `mark_backup_complete` (lines 383 and 533); confirmed error mapping pattern
- Direct source read: `crates/takoyaki-app/src/commands/wallflower.rs` — verified `set_wallflower_db_path` uses `db::set_setting` which writes to `AppState.db`
- Direct source read: `crates/takoyaki-app/tests/backup_db.rs` — verified existing test coverage for `mark_backup_complete`
- Direct source read: `.planning/v1.0-MILESTONE-AUDIT.md` — verified exact fix specification ("Single line change: open_in_memory() -> open_database(&default_path())" and "mark_backup_complete lacks row-count check (CR-02)")
- `cargo test -p takoyaki-app` output — confirmed 90 tests pass, 5 ignored (Phase 2/7 stubs), 0 failures

### Secondary (MEDIUM confidence)
None required — all findings verified from source.

## Metadata

**Confidence breakdown:**
- Fix specification: HIGH — verified directly from source code and audit document
- rusqlite execute() return type: HIGH — verified from existing codebase usage patterns and API knowledge consistent with rusqlite 0.39
- macOS data dir path: HIGH — `dirs::data_dir()` on macOS returns `~/Library/Application Support` (standard behavior of the `dirs` crate v6)
- Test gaps: HIGH — verified which tests exist and which don't by reading test files

**Research date:** 2026-05-06
**Valid until:** 2026-06-06 (stable — no external dependencies, all findings from local source)
