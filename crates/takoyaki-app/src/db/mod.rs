pub mod projects;

use rusqlite::Connection;

/// Application database handle.
pub struct Database {
    pub conn: Connection,
}

impl Database {
    /// Open a database at the given path, creating it and running schema migrations if needed.
    pub fn open(path: &std::path::Path) -> rusqlite::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    /// Open an in-memory database (for tests).
    pub fn open_in_memory() -> rusqlite::Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    /// Create schema tables if they do not already exist.
    fn initialize(&self) -> rusqlite::Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS projects (
                id            TEXT PRIMARY KEY,
                set_name      TEXT NOT NULL DEFAULT '',
                project_name  TEXT NOT NULL DEFAULT '',
                card_path     TEXT NOT NULL DEFAULT '',
                tempo_bpm     REAL,
                bank_count    INTEGER,
                last_modified TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_projects_card_path
                ON projects(card_path);
            CREATE INDEX IF NOT EXISTS idx_projects_name
                ON projects(project_name COLLATE NOCASE);",
        )
    }
}
