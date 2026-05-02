pub mod backups;
pub mod projects;
pub mod wallflower;

use rusqlite::{Connection, OptionalExtension};
use std::path::{Path, PathBuf};
use tracing::info;

const MIGRATION_V1: &str = include_str!("../../../../migrations/V1__initial_schema.sql");
const MIGRATION_V2: &str = include_str!("../../../../migrations/V2__backup_schema.sql");
const MIGRATION_V3: &str = include_str!("../../../../migrations/V3__wallflower_settings.sql");

/// Open (or create) a Takoyaki database at the given path, running migrations as needed.
pub fn open_database(path: &Path) -> rusqlite::Result<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let conn = Connection::open(path)?;
    initialize(&conn)?;
    Ok(conn)
}

/// Open an in-memory database (for tests and command-line use).
pub fn open_in_memory() -> rusqlite::Result<Connection> {
    let conn = Connection::open_in_memory()?;
    initialize(&conn)?;
    Ok(conn)
}

/// Return the default path for the Takoyaki database on macOS/Linux/Windows.
pub fn default_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("takoyaki")
        .join("takoyaki.db")
}

/// Run schema migrations on a fresh or existing connection.
///
/// Uses `PRAGMA user_version` to track the current migration level:
/// - 0 → run V1 migration (create snapshots, snapshot_files, projects tables + indexes)
/// - 1 → already migrated; no-op
///
/// WAL mode and foreign-key enforcement are applied unconditionally on every open.
fn initialize(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    let current_version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    if current_version < 1 {
        info!("Running V1 migration: initial schema");
        conn.execute_batch(MIGRATION_V1)?;
        conn.execute_batch("PRAGMA user_version = 1;")?;
    }

    if current_version < 2 {
        info!("Running V2 migration: backup schema");
        conn.execute_batch(MIGRATION_V2)?;
        conn.execute_batch("PRAGMA user_version = 2;")?;
    }

    if current_version < 3 {
        info!("Running V3 migration: settings table");
        conn.execute_batch(MIGRATION_V3)?;
        conn.execute_batch("PRAGMA user_version = 3;")?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Settings helpers (V3 migration)
// ---------------------------------------------------------------------------

/// Read a settings value by key. Returns None if the key is not found or value is empty.
pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>, rusqlite::Error> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        rusqlite::params![key],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .map(|opt| opt.filter(|s| !s.is_empty()))
}

/// Write or overwrite a settings value.
pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        rusqlite::params![key, value],
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Legacy Database struct — retained for backward compatibility with existing
// commands that reference db::Database::open_in_memory() and db.conn.
// ---------------------------------------------------------------------------

/// Application database handle.
pub struct Database {
    pub conn: Connection,
}

impl Database {
    /// Open a database at the given path, creating it and running schema migrations if needed.
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        let conn = open_database(path)?;
        Ok(Self { conn })
    }

    /// Open an in-memory database (for tests).
    pub fn open_in_memory() -> rusqlite::Result<Self> {
        let conn = open_in_memory()?;
        Ok(Self { conn })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_memory_ok() {
        assert!(open_in_memory().is_ok());
    }

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

    #[test]
    fn test_tables_snapshots_exists() {
        let conn = open_in_memory().unwrap();
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='snapshots'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "Table 'snapshots' must exist");
    }

    #[test]
    fn test_tables_snapshot_files_exists() {
        let conn = open_in_memory().unwrap();
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='snapshot_files'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "Table 'snapshot_files' must exist");
    }

    #[test]
    fn test_tables_projects_exists() {
        let conn = open_in_memory().unwrap();
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='projects'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "Table 'projects' must exist");
    }

    #[test]
    fn test_user_version_is_3() {
        let conn = open_in_memory().unwrap();
        let version: i32 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 3, "PRAGMA user_version must be 3 after V3 migration");
    }

    #[test]
    fn test_journal_mode_wal() {
        let conn = open_in_memory().unwrap();
        let mode: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        // In-memory DBs report "memory" — WAL cannot be applied to in-memory databases.
        assert!(
            mode == "wal" || mode == "memory",
            "Expected 'wal' or 'memory', got '{}'",
            mode
        );
    }

    #[test]
    fn test_foreign_keys_enabled() {
        let conn = open_in_memory().unwrap();
        let fk: i32 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(fk, 1, "PRAGMA foreign_keys must be 1");
    }

    #[test]
    fn test_open_database_idempotent() {
        // Opening the same in-memory db twice via initialize should not error
        let conn = open_in_memory().unwrap();
        let result = initialize(&conn);
        assert!(result.is_ok(), "Re-initializing an already-migrated DB must be a no-op");
    }

    #[test]
    fn test_all_three_tables() {
        let conn = open_in_memory().unwrap();
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table'
                 AND name IN ('snapshots', 'snapshot_files', 'projects')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 3, "Expected 3 tables: snapshots, snapshot_files, projects");
    }
}
