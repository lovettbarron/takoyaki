-- V1__initial_schema.sql
-- Takoyaki own metadata database

-- Snapshot records: one row per snapshot event
CREATE TABLE snapshots (
    id         TEXT PRIMARY KEY NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    operation  TEXT NOT NULL,  -- 'manual', 'pre-write', 'backup'
    project_path TEXT,
    file_count INTEGER NOT NULL,
    total_bytes INTEGER NOT NULL
);

-- Individual files captured in a snapshot
CREATE TABLE snapshot_files (
    id           TEXT PRIMARY KEY NOT NULL,
    snapshot_id  TEXT NOT NULL REFERENCES snapshots(id) ON DELETE CASCADE,
    original_path TEXT NOT NULL,
    stored_path  TEXT NOT NULL,
    file_hash    TEXT NOT NULL
);

-- Project index — updated on connect / rescan
CREATE TABLE projects (
    id           TEXT PRIMARY KEY NOT NULL,
    set_name     TEXT NOT NULL,
    project_name TEXT NOT NULL,
    card_path    TEXT NOT NULL UNIQUE,
    tempo_bpm    INTEGER,
    bank_count   INTEGER,
    last_modified TEXT,
    indexed_at   TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_projects_card_path ON projects(card_path);
CREATE INDEX idx_snapshots_project_path ON snapshots(project_path);
CREATE INDEX idx_snapshots_created_at ON snapshots(created_at);
