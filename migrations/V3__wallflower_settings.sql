-- V3: Settings table for user-configurable values (Phase 5: Wallflower DB path)
CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

-- Default: empty string means auto-discovery is used
INSERT OR IGNORE INTO settings (key, value) VALUES ('wallflower_db_path', '');
