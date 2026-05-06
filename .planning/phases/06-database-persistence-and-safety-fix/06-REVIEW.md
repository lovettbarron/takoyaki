---
phase: 06-database-persistence-and-safety-fix
reviewed: 2026-05-06T00:00:00Z
depth: standard
files_reviewed: 4
files_reviewed_list:
  - crates/takoyaki-app/src/lib.rs
  - crates/takoyaki-app/src/db/backups.rs
  - crates/takoyaki-app/tests/backup_db.rs
  - crates/takoyaki-app/src/db/mod.rs
findings:
  critical: 0
  warning: 4
  info: 1
  total: 5
status: issues_found
---

# Phase 06: Code Review Report

**Reviewed:** 2026-05-06T00:00:00Z
**Depth:** standard
**Files Reviewed:** 4
**Status:** issues_found

## Summary

Reviewed the database persistence layer: migration system (`db/mod.rs`), backup CRUD operations (`db/backups.rs`), application startup integration (`lib.rs`), and backup integration tests (`backup_db.rs`). The code is well-structured overall -- parameterized queries are used consistently (mitigating SQL injection), `insert_backup` correctly wraps its multi-row insert in a transaction, and the test suite covers the key CRUD paths including edge cases like unknown IDs and CASCADE deletes.

Four warnings were identified: a non-atomic cleanup function that could leave the database in an inconsistent state if the DELETE fails after the SELECT; migrations that are not wrapped in transactions (risking partial schema application on failure); a test helper that manually inlines DDL instead of using the migration files (schema drift risk); and a fallback in `default_path()` that silently uses the current working directory when the system data directory is unavailable.

## Warnings

### WR-01: cleanup_incomplete_backups is not atomic (SELECT then DELETE without transaction)

**File:** `crates/takoyaki-app/src/db/backups.rs:224-236`
**Issue:** `cleanup_incomplete_backups` performs a SELECT to collect dest_paths, then a separate DELETE. These two operations are not wrapped in a transaction. If the DELETE fails (e.g., disk full on WAL, I/O error), the caller in `lib.rs:150-155` will have already received paths and may delete filesystem directories for backups that still have DB rows. On next startup the function would attempt to clean up the same rows again (likely benign), but the gap between SELECT and DELETE means the function's postcondition ("rows are deleted and paths are returned") is not guaranteed atomically. For a safety-critical backup tool, this is a concern.
**Fix:**
```rust
pub fn cleanup_incomplete_backups(conn: &mut Connection) -> rusqlite::Result<Vec<String>> {
    let tx = conn.transaction()?;
    let mut stmt = tx.prepare(
        "SELECT dest_path FROM backups WHERE status = 'in-progress'",
    )?;
    let paths: rusqlite::Result<Vec<String>> = stmt
        .query_map([], |row| row.get(0))?
        .collect();
    let paths = paths?;
    drop(stmt);

    tx.execute("DELETE FROM backups WHERE status = 'in-progress'", [])?;
    tx.commit()?;

    Ok(paths)
}
```
Note: this changes the signature to `&mut Connection` (needed for `transaction()`), which matches how it is already called in `lib.rs:150` (`&mut db.conn`). The current function signature (`&Connection`) actually accepts the mutable reference via auto-deref, but taking `&mut Connection` is more honest about intent.

### WR-02: Migrations are not wrapped in transactions -- partial migration leaves inconsistent state

**File:** `crates/takoyaki-app/src/db/mod.rs:51-68`
**Issue:** Each migration block runs `execute_batch(MIGRATION_VN)` followed by `execute_batch("PRAGMA user_version = N;")` without a wrapping transaction. If the migration DDL partially applies and then fails (e.g., V2 creates the `backups` table but fails on `backup_files`), the `user_version` PRAGMA is never bumped. On next open, the migration re-runs and hits "table already exists" errors, leaving the database permanently broken. SQLite's `execute_batch` does not automatically wrap multiple statements in a transaction.
**Fix:** Wrap each migration in an explicit transaction:
```rust
if current_version < 2 {
    info!("Running V2 migration: backup schema");
    conn.execute_batch("BEGIN;")?;
    match conn.execute_batch(MIGRATION_V2) {
        Ok(()) => {
            conn.execute_batch("PRAGMA user_version = 2;")?;
            conn.execute_batch("COMMIT;")?;
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK;");
            return Err(e);
        }
    }
}
```
Alternatively, use `IF NOT EXISTS` in all DDL (V1 and V2 currently do not use it for all statements, though V3 does). The defensive approach is both: transactional migrations AND idempotent DDL.

### WR-03: Test schema manually inlines DDL instead of using migration files -- schema drift risk

**File:** `crates/takoyaki-app/tests/backup_db.rs:11-41`
**Issue:** The `setup_backup_db()` helper manually inlines the V2 DDL as a string literal rather than using the production `db::open_in_memory()` or `db::initialize()` function. If the production migration SQL is updated (e.g., adding a column, changing a default, adding an index), the test schema will silently diverge from production, and tests will pass against a stale schema. This is a correctness risk for a safety-critical backup system.
**Fix:** Replace the manual DDL with the production initialization path:
```rust
fn setup_backup_db() -> rusqlite::Connection {
    let db = takoyaki_app::db::Database::open_in_memory().expect("open_in_memory");
    db.conn
}
```
This ensures tests always run against the exact same schema as production, including all migrations, WAL mode, and foreign key enforcement.

### WR-04: default_path() silently falls back to current working directory

**File:** `crates/takoyaki-app/src/db/mod.rs:31-36`
**Issue:** If `dirs::data_dir()` returns `None` (which can happen on sandboxed or unusual macOS configurations), the fallback is `PathBuf::from(".")`, creating the database at `./takoyaki/takoyaki.db` relative to whatever the process's current working directory is. For a Tauri desktop app, CWD is unpredictable (could be `/`, the app bundle, or the user's home directory). This means the database could be silently created in an unexpected location, and on subsequent launches with a different CWD, a fresh empty database would be opened -- losing all backup history. Given the project's emphasis on data safety, this silent fallback is a risk.
**Fix:** Either panic with a clear error message or use a known fallback like `$HOME/.takoyaki/`:
```rust
pub fn default_path() -> PathBuf {
    dirs::data_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".local/share")))
        .expect("Unable to determine data directory for Takoyaki database. \
                 Set $XDG_DATA_HOME or ensure $HOME is set.")
        .join("takoyaki")
        .join("takoyaki.db")
}
```

## Info

### IN-01: Dead writes in test to suppress compiler warnings

**File:** `crates/takoyaki-app/tests/backup_db.rs:95-96`
**Issue:** Lines 95-96 mutate `b1.status` and `b2.status` with the comment "Keep compiler happy", but these writes are never read. The `mut` keyword on the bindings at lines 78-79 is the actual cause of the warning -- removing `mut` from the bindings is the correct fix rather than adding dead writes.
**Fix:** Change lines 78-79 from:
```rust
let mut b1 = make_backup_insert(...);
let mut b2 = make_backup_insert(...);
```
to:
```rust
let b1 = make_backup_insert(...);
let b2 = make_backup_insert(...);
```
And remove lines 95-96 entirely.

---

_Reviewed: 2026-05-06T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
