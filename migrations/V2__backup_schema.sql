-- V2: Backup history and file manifests (SAFE-01, SAFE-05)
-- No REFERENCES projects(id) — projects table cleared on re-index (Pitfall 3).

CREATE TABLE backups (
    id           TEXT PRIMARY KEY NOT NULL,
    project_id   TEXT NOT NULL,
    project_name TEXT NOT NULL,
    dest_path    TEXT NOT NULL,
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    operation    TEXT NOT NULL,
    file_count   INTEGER NOT NULL,
    total_bytes  INTEGER NOT NULL,
    checksum_ok  INTEGER NOT NULL DEFAULT 1,
    status       TEXT NOT NULL DEFAULT 'in-progress'
);

CREATE TABLE backup_files (
    id            TEXT PRIMARY KEY NOT NULL,
    backup_id     TEXT NOT NULL REFERENCES backups(id) ON DELETE CASCADE,
    relative_path TEXT NOT NULL,
    stored_path   TEXT NOT NULL,
    file_hash     TEXT NOT NULL,
    size_bytes    INTEGER NOT NULL,
    change_type   TEXT NOT NULL
);

CREATE INDEX idx_backups_project_id ON backups(project_id);
CREATE INDEX idx_backups_created_at ON backups(created_at);
CREATE INDEX idx_backup_files_backup_id ON backup_files(backup_id);
