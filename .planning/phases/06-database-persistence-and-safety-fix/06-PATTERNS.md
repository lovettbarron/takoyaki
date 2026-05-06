# Phase 6: Database Persistence & Safety Fix - Pattern Map

**Mapped:** 2026-05-06
**Files analyzed:** 4 (2 modified, 2 test additions)
**Analogs found:** 4 / 4

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/takoyaki-app/src/lib.rs` (line 133) | config/init | request-response | `crates/takoyaki-app/src/db/mod.rs` (Database::open, open_database) | exact — same module, same types |
| `crates/takoyaki-app/src/db/backups.rs` (mark_backup_complete) | service | CRUD | `crates/takoyaki-app/src/db/backups.rs` (cleanup_incomplete_backups, delete_backup) | exact — same file, same execute() return-value pattern |
| `crates/takoyaki-app/tests/backup_db.rs` (new test) | test | CRUD | `crates/takoyaki-app/tests/backup_db.rs` (existing tests) | exact — same file, same setup_backup_db() harness |
| `crates/takoyaki-app/src/db/mod.rs` (new unit tests) | test | CRUD | `crates/takoyaki-app/src/db/mod.rs` (existing #[cfg(test)] block) | exact — same file, same inline test pattern |

## Pattern Assignments

### `crates/takoyaki-app/src/lib.rs` line 133 (config/init)

**Analog:** `crates/takoyaki-app/src/db/mod.rs` — `Database::open` and `default_path`

**Existing broken line** (lib.rs:133):
```rust
db: Mutex::new(db::Database::open_in_memory().expect("Failed to open database")),
```

**Target pattern — Database::open signature** (db/mod.rs:108-110):
```rust
pub fn open(path: &Path) -> rusqlite::Result<Self> {
    let conn = open_database(path)?;
    Ok(Self { conn })
}
```

**Target pattern — default_path** (db/mod.rs:31-36):
```rust
pub fn default_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("takoyaki")
        .join("takoyaki.db")
}
```

**Fixed AppState construction** — replace lib.rs:133 with:
```rust
db: Mutex::new(db::Database::open(&db::default_path()).expect("Failed to open database")),
```

**Context: surrounding AppState struct** (lib.rs:132-140):
```rust
let app_state = AppState {
    db: Mutex::new(db::Database::open_in_memory().expect("Failed to open database")),
    device: Mutex::new(DeviceState {
        mount_point: None,
        confirmed: false,
    }),
    cancel_backup: Arc::new(AtomicBool::new(false)),
    audio_tx,
};
```

**Directory-creation guarantee** — `open_database` already calls `create_dir_all` before opening (db/mod.rs:14-21):
```rust
pub fn open_database(path: &Path) -> rusqlite::Result<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let conn = Connection::open(path)?;
    initialize(&conn)?;
    Ok(conn)
}
```

**Constraint:** Only line 133 changes. Do NOT touch any `open_in_memory()` calls in test files — in-memory is correct for tests.

---

### `crates/takoyaki-app/src/db/backups.rs` — `mark_backup_complete` (service, CRUD)

**Analog:** `crates/takoyaki-app/src/db/backups.rs` — `cleanup_incomplete_backups` and `delete_backup`

**Current implementation** (backups.rs:108-118):
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

**Row-count guard pattern** — `conn.execute()` returns `rusqlite::Result<usize>` (rows changed). Capture the return value; `QueryReturnedNoRows` is the canonical error variant for "expected a row but found none". Fixed implementation:
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

**Analog: execute() return-value usage** — `cleanup_incomplete_backups` (backups.rs:218-230) and `delete_backup` (backups.rs:233-239) both call `conn.execute()` but do not guard on `rows_changed`. The new pattern captures that return value explicitly.

**Call-site error mapping** (commands/backup.rs:383 and :533) — both already map `rusqlite::Error` to `AppError::Database` via `.map_err(|e| AppError::Database(e.to_string()))`. No call-site changes are needed; the new error propagates cleanly.

---

### `crates/takoyaki-app/tests/backup_db.rs` — new test: `test_mark_backup_complete_unknown_id_returns_err` (test, CRUD)

**Analog:** `crates/takoyaki-app/tests/backup_db.rs` — existing tests using `setup_backup_db()` and direct function calls

**Test harness pattern** (backup_db.rs:11-41):
```rust
fn setup_backup_db() -> rusqlite::Connection {
    let conn = rusqlite::Connection::open_in_memory().expect("open_in_memory");
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS backups ( ... );
         CREATE TABLE IF NOT EXISTS backup_files ( ... );
         PRAGMA foreign_keys = ON;",
    )
    .expect("create V2 schema");
    conn
}
```

**Test structure to copy** (backup_db.rs:73-97 — `test_insert_and_list_backups`):
```rust
#[test]
fn test_insert_and_list_backups() {
    let mut conn = setup_backup_db();
    // ... insert records ...
    mark_backup_complete(&conn, "b1", true).unwrap();
    // ... assertions ...
}
```

**New test to add** (append to backup_db.rs after the last test):
```rust
#[test]
fn test_mark_backup_complete_unknown_id_returns_err() {
    let conn = setup_backup_db();
    // No records inserted — "nonexistent-id" does not exist
    let result = mark_backup_complete(&conn, "nonexistent-id", true);
    assert!(result.is_err(), "mark_backup_complete with unknown ID must return Err");
}
```

**Import line** (backup_db.rs:1-6) — `mark_backup_complete` is already imported:
```rust
use takoyaki_app::db::backups::{
    BackupFileInsert, BackupInsert, cleanup_incomplete_backups, delete_backup,
    get_backup_files, insert_backup, list_all_backups, list_backups, mark_backup_complete,
};
```

---

### `crates/takoyaki-app/src/db/mod.rs` — new unit tests (test, CRUD)

**Analog:** `crates/takoyaki-app/src/db/mod.rs` — existing `#[cfg(test)] mod tests` block (mod.rs:124-241)

**Test module wrapper pattern** (mod.rs:124-127):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    // tests go here
}
```

**Existing inline test pattern to copy** (mod.rs:134-147 — `test_open_database_creates_file`):
```rust
#[test]
fn test_open_database_creates_file() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().to_path_buf();
    drop(tmp); // Remove the temp file so open_database creates it fresh
    let conn = open_database(&path).unwrap();
    let count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(count >= 3, "Expected at least 3 tables, got {}", count);
}
```

**New test 1 — default_path absolute** (add inside existing `mod tests`):
```rust
#[test]
fn test_default_path_is_absolute() {
    let path = default_path();
    assert!(path.is_absolute(), "default_path() must return an absolute path");
    assert!(
        path.ends_with("takoyaki/takoyaki.db"),
        "default_path() must end with takoyaki/takoyaki.db, got: {:?}",
        path
    );
}
```

**New test 2 — settings round-trip on file DB** (add inside existing `mod tests`):
```rust
#[test]
fn test_settings_persist_on_file_db() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().to_path_buf();
    drop(tmp);
    let conn = open_database(&path).unwrap();
    set_setting(&conn, "wallflower_db_path", "/some/path.db").unwrap();
    let value = get_setting(&conn, "wallflower_db_path").unwrap();
    assert_eq!(
        value,
        Some("/some/path.db".to_string()),
        "set_setting/get_setting must round-trip on a file-backed DB"
    );
}
```

**Note:** `tempfile` is already a dev-dependency in Cargo.toml (verified in use at mod.rs:135).

---

## Shared Patterns

### rusqlite parameterized queries
**Source:** `crates/takoyaki-app/src/db/backups.rs` (all functions)
**Apply to:** All DB edits in this phase
```rust
// All SQL uses params![] macro — never string interpolation (T-02-01)
conn.execute(
    "UPDATE backups SET status = 'complete', checksum_ok = ?2 WHERE id = ?1",
    params![backup_id, checksum_ok as i64],
)?;
```

### Error propagation via `?`
**Source:** `crates/takoyaki-app/src/db/backups.rs` (all functions)
**Apply to:** All DB functions in this phase
```rust
// rusqlite::Result<()> with ? propagation is the standard return type
pub fn mark_backup_complete(conn: &Connection, ...) -> rusqlite::Result<()> {
    let rows_changed = conn.execute(...)?;
    // explicit guard, then Ok(())
    Ok(())
}
```

### In-memory DB for tests (never file-backed)
**Source:** `crates/takoyaki-app/tests/backup_db.rs` (setup_backup_db), `crates/takoyaki-app/src/db/mod.rs` (test_open_in_memory_ok)
**Apply to:** All new test cases
```rust
// Tests always use open_in_memory() or rusqlite::Connection::open_in_memory()
// Never open a file-backed DB in tests unless specifically testing file-DB behavior
// (and if so, use tempfile::NamedTempFile to avoid leaving stale .db files)
fn setup_backup_db() -> rusqlite::Connection {
    rusqlite::Connection::open_in_memory().expect("open_in_memory")
}
```

## No Analog Found

None. All four files have exact analogs — every pattern needed already exists in the codebase.

## Metadata

**Analog search scope:** `crates/takoyaki-app/src/` and `crates/takoyaki-app/tests/`
**Files scanned:** 4 (lib.rs, db/mod.rs, db/backups.rs, tests/backup_db.rs)
**Pattern extraction date:** 2026-05-06
