# Phase 3: Write Path and Backup - Pattern Map

**Mapped:** 2026-04-30
**Files analyzed:** 17 new/modified files
**Analogs found:** 17 / 17

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `crates/takoyaki-app/src/commands/backup.rs` | command | streaming + file-I/O | `crates/takoyaki-app/src/commands/health.rs` | role-match (async spawn + emit) |
| `crates/takoyaki-app/src/commands/mod.rs` | config | — | `crates/takoyaki-app/src/commands/mod.rs` | exact (add one line) |
| `crates/takoyaki-app/src/db/backups.rs` | service | CRUD | `crates/takoyaki-app/src/db/projects.rs` | exact |
| `crates/takoyaki-app/src/db/mod.rs` | config | — | `crates/takoyaki-app/src/db/mod.rs` | exact (add migration V2) |
| `crates/takoyaki-app/src/lib.rs` | config | — | `crates/takoyaki-app/src/lib.rs` | exact (add commands + AppState field) |
| `crates/takoyaki-app/src/error.rs` | utility | — | `crates/takoyaki-app/src/error.rs` | exact (add Cancelled variant) |
| `migrations/V2__backup_schema.sql` | migration | CRUD | `migrations/V1__initial_schema.sql` | exact |
| `crates/takoyaki-app/tests/backup.rs` | test | file-I/O | `crates/takoyaki-app/tests/projects.rs` | role-match |
| `crates/takoyaki-app/tests/restore.rs` | test | file-I/O | `crates/takoyaki-app/src/atomic/snapshot.rs` (tests) | role-match |
| `crates/takoyaki-app/tests/dry_run.rs` | test | file-I/O | `crates/takoyaki-app/tests/projects.rs` | role-match |
| `crates/takoyaki-app/tests/backup_db.rs` | test | CRUD | `crates/takoyaki-app/tests/projects.rs` | exact |
| `src/lib/stores/backup.ts` | store | event-driven | `src/lib/stores/device.ts` | role-match |
| `src/components/backups/BackupsView.tsx` | component | request-response | `src/components/project-detail/ProjectDetailView.tsx` | role-match |
| `src/components/backups/BackupTimeline.tsx` | component | request-response | `src/components/projects/ProjectTable.tsx` | role-match |
| `src/components/backups/SnapshotRow.tsx` | component | request-response | `src/components/projects/ProjectRow.tsx` | role-match |
| `src/components/backups/SnapshotDetailPanel.tsx` | component | request-response | `src/components/project-detail/MetadataHeader.tsx` | role-match |
| `src/components/backups/DryRunModal.tsx` | component | request-response | `src/components/volume-confirm-dialog.tsx` | exact |
| `src/components/backup-progress/BackupProgressView.tsx` | component | streaming | `src/components/health/HealthEventListener.tsx` | role-match |
| `src/components/backup-progress/InlineSuccessBanner.tsx` | component | event-driven | `src/components/tauri-event-listener.tsx` | role-match |
| `src/components/project-detail/MetadataHeader.tsx` | component | — | `src/components/project-detail/MetadataHeader.tsx` | exact (add Back Up button) |
| `src/lib/tauri.ts` | utility | request-response | `src/lib/tauri.ts` | exact (add new invoke wrappers) |
| `src/components/sidebar-nav.tsx` | component | — | `src/components/sidebar-nav.tsx` | exact (enable backups section) |

---

## Pattern Assignments

### `crates/takoyaki-app/src/commands/backup.rs` (command, streaming + file-I/O)

**Analog:** `crates/takoyaki-app/src/commands/health.rs` (lock-release pattern, spawn, tracing) and `crates/takoyaki-app/src/commands/projects.rs` (T-02-04 pattern, DB lock bracketing)

**Module header comment pattern** (`commands/health.rs` lines 1-9):
```rust
//! Tauri commands for backup, restore, and dry-run operations (SAFE-01, SAFE-02, SAFE-05, SAFE-06, SAFE-07).
//!
//! Threat model:
//! - T-02-04: project_id is an opaque UUID; card_path resolved from DB, never from frontend.
//! - T-03-01: backup destination always computed in Rust via dirs::home_dir() — frontend never supplies a raw path.
//! - T-03-02: snapshot_id resolved to stored_path via DB lookup (same T-02-04 pattern).
//! - T-03-03: WalkDir used with follow_links(false) to prevent symlink traversal.
```

**Imports pattern** (`commands/projects.rs` lines 8-12, `commands/health.rs` lines 11-12):
```rust
use crate::db;
use crate::error::AppError;
use crate::AppState;
use specta::Type;
use tracing::info;
use tauri::ipc::Channel;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use walkdir::WalkDir;
```

**DB lock-release-before-IO pattern** (`commands/health.rs` lines 33-51, `commands/projects.rs` lines 247-265):
```rust
// 1. Grab path from state, then DROP the lock.
//    All file I/O happens outside the lock (Pitfall 4: Mutex Deadlock).
let card_path = {
    let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db::projects::get_card_path(&db.conn, &project_id)
        .map_err(|e| AppError::Database(e.to_string()))?
};
// DB lock is dropped here — file copy loop runs without holding it.
```

**Tauri Channel command signature** (RESEARCH.md Pattern 1):
```rust
#[tauri::command]
#[specta::specta]
pub async fn backup_project(
    state: tauri::State<'_, AppState>,
    project_id: String,
    on_event: Channel<BackupEvent>,
) -> Result<(), AppError> {
```

**Tagged enum event type** — must derive both `Serialize` and `specta::Type` (Pitfall 6):
```rust
#[derive(Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum BackupEvent {
    Started { total_files: usize, destination: String },
    Progress { files_copied: usize, total_files: usize, current_file: String },
    Complete { files_copied: usize, total_bytes: u64, destination: String, checksum_ok: bool },
    Failed { reason: String },
}
```

**Cancellation check in file loop** (RESEARCH.md Pattern 2):
```rust
if state.cancel_backup.load(Ordering::Relaxed) {
    let _ = std::fs::remove_dir_all(&dest_dir);
    return Err(AppError::Cancelled("Backup cancelled by user".to_string()));
}
```

**Error handling pattern** (`commands/projects.rs` lines 81-82):
```rust
.map_err(|e| AppError::Lock(e.to_string()))?;
.map_err(|e| AppError::Database(e.to_string()))?;
```

**Tracing pattern** (`commands/projects.rs` lines 343-348, `commands/health.rs` line 88):
```rust
tracing::info!("backup_project: starting backup of {} to {}", project_id, dest_path.display());
tracing::debug!("backup_project: copied {} ({} bytes)", rel_path.display(), bytes);
tracing::error!("backup_project: channel send failed: {e}");
```

---

### `crates/takoyaki-app/src/db/backups.rs` (service, CRUD)

**Analog:** `crates/takoyaki-app/src/db/projects.rs`

**Module header + imports** (`db/projects.rs` lines 1-6):
```rust
//! SQLite query functions for the backup history (SAFE-01, SAFE-05).
//!
//! Threat model T-02-01: All filter values use parameterized queries — never string interpolation.

use rusqlite::{params, Connection};
use specta::Type;
```

**Struct pattern** (`db/projects.rs` lines 22-34):
```rust
/// A backup record as returned by list queries (SAFE-05).
#[derive(Debug, serde::Serialize, Type, Clone)]
pub struct BackupSummary {
    pub id: String,
    pub project_id: String,
    pub project_name: String,
    pub dest_path: String,
    pub created_at: String,
    pub operation: String,     // 'manual-backup', 'pre-restore'
    pub file_count: i64,
    pub total_bytes: i64,
    pub checksum_ok: bool,
    pub status: String,        // 'in-progress', 'complete'
}
```

**Transaction-based insert** (RESEARCH.md Pattern 4 — `rusqlite::Transaction`):
```rust
pub fn insert_backup(conn: &Connection, record: &BackupRecord) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;  // rolls back on drop if not committed
    tx.execute(
        "INSERT INTO backups (id, project_id, project_name, dest_path, operation, \
         file_count, total_bytes, checksum_ok, status) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        params![
            record.id, record.project_id, record.project_name, record.dest_path,
            record.operation, record.file_count as i64, record.total_bytes as i64,
            record.checksum_ok as i64, "in-progress",
        ],
    )?;
    for file in &record.files { /* backup_files insert */ }
    tx.commit()
}
```

**Parameterized list query** (`db/projects.rs` lines 70-119):
```rust
pub fn list_backups(conn: &Connection, project_id: &str) -> rusqlite::Result<Vec<BackupSummary>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, project_name, dest_path, created_at, operation, \
         file_count, total_bytes, checksum_ok, status \
         FROM backups WHERE project_id = ?1 AND status = 'complete' \
         ORDER BY created_at DESC"
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
        Ok(BackupSummary { id: row.get(0)?, ... })
    })?;
    rows.collect()
}
```

**get_card_path equivalent for snapshots** (`db/projects.rs` lines 122-129):
```rust
pub fn get_backup_dest_path(conn: &Connection, backup_id: &str) -> rusqlite::Result<String> {
    conn.query_row(
        "SELECT dest_path FROM backups WHERE id = ?1",
        params![backup_id],
        |row| row.get(0),
    )
}
```

---

### `crates/takoyaki-app/src/db/mod.rs` (config — add V2 migration)

**Analog:** `crates/takoyaki-app/src/db/mod.rs` (exact — extend existing pattern)

**V2 migration constant + initialization block** (`db/mod.rs` lines 8, 42-53):
```rust
const MIGRATION_V1: &str = include_str!("../../../../migrations/V1__initial_schema.sql");
const MIGRATION_V2: &str = include_str!("../../../../migrations/V2__backup_schema.sql");

// In initialize():
if current_version < 2 {
    info!("Running V2 migration: backup schema");
    conn.execute_batch(MIGRATION_V2)?;
    conn.execute_batch("PRAGMA user_version = 2;")?;
}
```

**Module declaration addition** (`db/mod.rs` line 1):
```rust
pub mod backups;   // add after existing pub mod projects;
pub mod projects;
pub mod wallflower;
```

---

### `crates/takoyaki-app/src/lib.rs` (config — extend AppState)

**Analog:** `crates/takoyaki-app/src/lib.rs` (exact — add cancel flag + register commands)

**AppState extension** (`lib.rs` lines 21-24):
```rust
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub struct AppState {
    pub db: Mutex<db::Database>,
    pub device: Mutex<DeviceState>,
    pub cancel_backup: Arc<AtomicBool>,   // add this field
}
```

**collect_commands registration** (`lib.rs` lines 27-37):
```rust
let builder = tauri_specta::Builder::<tauri::Wry>::new().commands(collect_commands![
    // existing commands ...
    commands::backup::backup_project,
    commands::backup::restore_snapshot,
    commands::backup::compute_dry_run,
    commands::backup::list_backups,
    commands::backup::cancel_backup,
]);
```

---

### `crates/takoyaki-app/src/error.rs` (utility — add Cancelled variant)

**Analog:** `crates/takoyaki-app/src/error.rs` (exact — add one variant)

**Error enum pattern** (`error.rs` lines 5-24):
```rust
#[derive(Debug, Error, Serialize, Type)]
pub enum AppError {
    // ... existing variants ...
    #[error("Operation cancelled: {0}")]
    Cancelled(String),
}
```

---

### `migrations/V2__backup_schema.sql` (migration, CRUD)

**Analog:** `migrations/V1__initial_schema.sql`

**SQL style** (`V1__initial_schema.sql` lines 1-36):
```sql
-- V2__backup_schema.sql
-- Backup history and file manifests for user-visible backup operations (SAFE-01, SAFE-05)

-- Note: No REFERENCES projects(id) on project_id — projects table is cleared on re-index.
-- Store project_name as denormalized column for offline display (Pitfall 3).
CREATE TABLE backups (
    id           TEXT PRIMARY KEY NOT NULL,
    project_id   TEXT NOT NULL,
    project_name TEXT NOT NULL,
    dest_path    TEXT NOT NULL,
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    operation    TEXT NOT NULL,   -- 'manual-backup', 'pre-restore'
    file_count   INTEGER NOT NULL,
    total_bytes  INTEGER NOT NULL,
    checksum_ok  INTEGER NOT NULL DEFAULT 1,
    status       TEXT NOT NULL DEFAULT 'in-progress'  -- for D-12 incomplete cleanup
);

CREATE TABLE backup_files (
    id            TEXT PRIMARY KEY NOT NULL,
    backup_id     TEXT NOT NULL REFERENCES backups(id) ON DELETE CASCADE,
    relative_path TEXT NOT NULL,
    stored_path   TEXT NOT NULL,
    file_hash     TEXT NOT NULL,
    size_bytes    INTEGER NOT NULL,
    change_type   TEXT NOT NULL   -- 'added','modified','removed','unchanged'
);

CREATE INDEX idx_backups_project_id ON backups(project_id);
CREATE INDEX idx_backups_created_at ON backups(created_at);
CREATE INDEX idx_backup_files_backup_id ON backup_files(backup_id);
```

---

### `crates/takoyaki-app/tests/backup.rs` (test, file-I/O)

**Analog:** `crates/takoyaki-app/src/atomic/snapshot.rs` tests (lines 133-254) and `crates/takoyaki-app/tests/projects.rs`

**Test file structure** (`atomic/snapshot.rs` lines 133-140, `tests/projects.rs` lines 3-37):
```rust
//! Unit tests for backup_project command (SAFE-01, SAFE-02)

use std::path::Path;
use tempfile::TempDir;

fn setup_project_dir(tmp: &TempDir) -> std::path::PathBuf {
    let project_dir = tmp.path().join("SETS/LIVESET_01");
    std::fs::create_dir_all(&project_dir).unwrap();
    std::fs::write(project_dir.join("project.work"), b"fake project data").unwrap();
    std::fs::create_dir_all(project_dir.join("AUDIO")).unwrap();
    std::fs::write(project_dir.join("AUDIO/kick.wav"), b"fake audio data").unwrap();
    project_dir
}

#[test]
fn test_backup_copies_all_files() {
    let tmp = TempDir::new().unwrap();
    // ...
}
```

**TempDir pattern** (`atomic/snapshot.rs` lines 138-142):
```rust
let tmp = TempDir::new().unwrap();
let snapshot_root = tmp.path().join("snapshots");
let engine = SnapshotEngine::new(snapshot_root);
```

---

### `crates/takoyaki-app/tests/backup_db.rs` (test, CRUD)

**Analog:** `crates/takoyaki-app/tests/projects.rs` (exact setup_db pattern)

**In-memory DB setup** (`tests/projects.rs` lines 3-22):
```rust
fn setup_db() -> rusqlite::Connection {
    let conn = rusqlite::Connection::open_in_memory().expect("open_in_memory");
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS backups ( ... ); \
         CREATE TABLE IF NOT EXISTS backup_files ( ... );"
    ).expect("create schema");
    conn
}
```

---

### `src/lib/stores/backup.ts` (store, event-driven)

**Analog:** `src/lib/stores/device.ts` (Zustand pattern) and `src/lib/stores/navigation.ts` (multi-state pattern)

**Zustand create pattern** (`stores/device.ts` lines 1-19):
```typescript
import { create } from "zustand";

type BackupStatus = "idle" | "dry-running" | "in-progress" | "complete" | "failed" | "cancelled";

interface BackupProgress {
  filesCopied: number;
  totalFiles: number;
  currentFile: string;
}

interface BackupState {
  status: BackupStatus;
  progress: BackupProgress | null;
  successBanner: SuccessBanner | null;
  dryRunManifest: FileChangeManifest | null;

  setStatus: (status: BackupStatus) => void;
  setProgress: (progress: BackupProgress) => void;
  setSuccessBanner: (banner: SuccessBanner | null) => void;
  setDryRunManifest: (manifest: FileChangeManifest | null) => void;
  reset: () => void;
}

export const useBackupStore = create<BackupState>((set) => ({
  status: "idle",
  progress: null,
  successBanner: null,
  dryRunManifest: null,
  setStatus: (status) => set({ status }),
  setProgress: (progress) => set({ progress }),
  setSuccessBanner: (banner) => set({ successBanner: banner }),
  setDryRunManifest: (manifest) => set({ dryRunManifest: manifest }),
  reset: () => set({ status: "idle", progress: null, successBanner: null, dryRunManifest: null }),
}));
```

---

### `src/components/backups/DryRunModal.tsx` (component, request-response)

**Analog:** `src/components/volume-confirm-dialog.tsx` (exact Dialog pattern with shadcn)

**Dialog import + structure** (`volume-confirm-dialog.tsx` lines 1-57):
```typescript
"use client";

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";

interface DryRunModalProps {
  open: boolean;
  manifest: FileChangeManifest;
  operationLabel: string;
  onApply: () => void;
  onCancel: () => void;
}

export function DryRunModal({ open, manifest, operationLabel, onApply, onCancel }: DryRunModalProps) {
  return (
    <Dialog open={open} onOpenChange={(isOpen) => { if (!isOpen) onCancel(); }}>
      <DialogContent className="sm:max-w-lg" showCloseButton={false}>
        <DialogHeader>
          <DialogTitle className="font-mono text-base font-semibold">
            {operationLabel}
          </DialogTitle>
          <DialogDescription className="text-sm leading-relaxed">
            A snapshot of the current state will be created before applying.
          </DialogDescription>
        </DialogHeader>
        {/* File change manifest list — use shadcn scroll-area */}
        <DialogFooter>
          <Button variant="ghost" onClick={onCancel} className="font-mono text-xs">
            Cancel
          </Button>
          <Button
            onClick={onApply}
            className="font-mono text-xs bg-accent text-accent-foreground hover:bg-accent/90"
          >
            Apply
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
```

---

### `src/components/backup-progress/BackupProgressView.tsx` (component, streaming)

**Analog:** `src/components/health/HealthEventListener.tsx` (useEffect + async listen pattern) and `src/components/tauri-event-listener.tsx`

**Channel listener pattern** (`tauri-event-listener.tsx` lines 11-44):
```typescript
"use client";

import { useEffect } from "react";
import { Channel } from "@tauri-apps/api/core";
import { useBackupStore } from "@/lib/stores/backup";
import type { BackupEvent } from "@/lib/types";

export function BackupProgressView() {
  const { progress, setProgress, setStatus, setSuccessBanner } = useBackupStore();

  // Channel is created once per backup operation and passed to invoke()
  // This component renders the progress state driven by the store.
  // See useBackupStore for channel setup logic in BackupsView.

  return (
    <div className="flex flex-col gap-4 p-6">
      <p className="font-mono text-sm text-muted-foreground">
        {progress ? `${progress.filesCopied} of ${progress.totalFiles} files` : "Starting…"}
      </p>
      {/* shadcn Progress component */}
    </div>
  );
}
```

---

### `src/components/backup-progress/InlineSuccessBanner.tsx` (component, event-driven)

**Analog:** `src/components/tauri-event-listener.tsx` (useEffect cleanup pattern)

**Auto-dismiss useEffect** (RESEARCH.md Code Examples, `tauri-event-listener.tsx` lines 11-43):
```typescript
"use client";

import { useEffect, useState } from "react";

interface InlineSuccessBannerProps {
  message: string;
  onDismiss: () => void;
}

export function InlineSuccessBanner({ message, onDismiss }: InlineSuccessBannerProps) {
  useEffect(() => {
    const timer = setTimeout(() => onDismiss(), 4000);
    return () => clearTimeout(timer); // cleanup on unmount
  }, [onDismiss]);

  return (
    <div className="fixed top-0 inset-x-0 z-50 flex items-center gap-2 px-4 py-2 bg-accent/10 border-b border-accent/20 font-mono text-xs text-foreground">
      {/* lucide-react CircleCheck icon + message */}
      {message}
    </div>
  );
}
```

---

### `src/components/backups/BackupsView.tsx` (component, request-response)

**Analog:** `src/components/project-detail/ProjectDetailView.tsx` (useQuery + useEffect + store pattern)

**useQuery pattern** (`ProjectDetailView.tsx` lines 33-37):
```typescript
"use client";

import { useQuery } from "@tanstack/react-query";
import { useBackupStore } from "@/lib/stores/backup";
import { listBackups } from "@/lib/tauri";

export function BackupsView() {
  const { data, isPending } = useQuery({
    queryKey: ["backups", selectedProjectId],
    queryFn: () => listBackups(selectedProjectId!),
    enabled: selectedProjectId !== null,
  });
  // ...
}
```

---

### `src/components/backups/BackupTimeline.tsx` (component, request-response)

**Analog:** `src/components/projects/ProjectTable.tsx` (reverse-chrono list with loading state)

**isPending skeleton pattern** (`ProjectTable.tsx` lines 77-84, 125-138):
```typescript
const { data, isPending } = useQuery({
  queryKey: ["backups", projectId],
  queryFn: () => listBackups(projectId),
});

// Render: skeleton while loading, empty state, or data rows
{isPending ? (
  Array.from({ length: 3 }).map((_, i) => (
    <Skeleton key={i} className="h-9 w-full" />
  ))
) : data?.length === 0 ? (
  <p className="font-mono text-sm text-muted-foreground py-12 text-center">
    No backups yet
  </p>
) : (
  data?.map((backup) => <SnapshotRow key={backup.id} backup={backup} />)
)}
```

---

### `src/components/backups/SnapshotRow.tsx` (component, request-response)

**Analog:** `src/components/projects/ProjectRow.tsx`
<br>Read this file for exact row click + typography pattern:

**File:** `/Users/albair/src/takoyaki/src/components/projects/ProjectRow.tsx`

```typescript
"use client";

import { useNavigationStore } from "@/lib/stores/navigation";
import type { BackupSummary } from "@/lib/types";

interface SnapshotRowProps {
  backup: BackupSummary;
  selected: boolean;
  onSelect: (id: string) => void;
}

export function SnapshotRow({ backup, selected, onSelect }: SnapshotRowProps) {
  return (
    <div
      onClick={() => onSelect(backup.id)}
      className={`flex items-center justify-between px-4 py-2 cursor-pointer font-mono text-xs
        ${selected ? "bg-accent/10" : "hover:bg-muted"}`}
    >
      <span className="text-foreground">{backup.created_at}</span>
      <span className="text-muted-foreground">{backup.operation}</span>
      <span className="text-muted-foreground">{backup.file_count} files</span>
    </div>
  );
}
```

---

### `src/components/project-detail/MetadataHeader.tsx` (component — modify existing)

**Analog:** `src/components/project-detail/MetadataHeader.tsx` (exact — add Back Up button)

**Existing structure to preserve** (`MetadataHeader.tsx` lines 9-45):
```typescript
// Add a onBackUp?: () => void prop and a Button in the right metadata section.
// Import Button from "@/components/ui/button" (already in codebase).
// Place it after the Modified span, matching the warm dark palette style.
```

---

### `src/lib/tauri.ts` (utility — extend)

**Analog:** `src/lib/tauri.ts` (exact — add new invoke wrappers following same pattern)

**Invoke wrapper pattern** (`tauri.ts` lines 16-57):
```typescript
import { invoke, Channel } from "@tauri-apps/api/core";
import type { BackupSummary, FileChangeManifest, BackupEvent } from "./types";

export async function listBackups(projectId: string): Promise<BackupSummary[]> {
  return invoke("list_backups", { projectId });
}

export async function computeDryRun(
  projectId: string,
  operation: "backup" | "restore",
  snapshotId?: string
): Promise<FileChangeManifest> {
  return invoke("compute_dry_run", { projectId, operation, snapshotId });
}

export async function backupProject(
  projectId: string,
  onEvent: Channel<BackupEvent>
): Promise<void> {
  return invoke("backup_project", { projectId, onEvent });
}

export async function restoreSnapshot(
  snapshotId: string,
  onEvent: Channel<BackupEvent>
): Promise<void> {
  return invoke("restore_snapshot", { snapshotId, onEvent });
}

export async function cancelBackup(): Promise<void> {
  return invoke("cancel_backup");
}
```

---

### `src/components/sidebar-nav.tsx` (component — modify existing)

**Analog:** `src/components/sidebar-nav.tsx` (exact — change `available: false` to `available: true` for the `"backups"` entry)

**Line to change** (`sidebar-nav.tsx` line 16):
```typescript
// Change:
{ key: "backups", label: "Backups", icon: Archive, available: false },
// To:
{ key: "backups", label: "Backups", icon: Archive, available: true },
```

---

## Shared Patterns

### DB Lock Release Before File I/O
**Source:** `crates/takoyaki-app/src/commands/health.rs` lines 33-51
**Apply to:** All commands in `commands/backup.rs` that do file I/O after a DB lookup

Pattern: Lock the Mutex only for DB lookups (path resolution), drop it before the file loop, lock again only for the final DB insert. Never hold `state.db.lock()` across a file copy or WalkDir iteration.

```rust
// GOOD — lock only for DB access (Pitfall 4 avoidance)
let card_path = {
    let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db::projects::get_card_path(&db.conn, &project_id)
        .map_err(|e| AppError::Database(e.to_string()))?
};
// DB lock dropped here. File copy loop runs freely.
for entry in WalkDir::new(&card_path).follow_links(false) { ... }
// Lock again only for final insert.
{
    let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db::backups::insert_backup(&db.conn, &record)?;
}
```

### Error Mapping
**Source:** `crates/takoyaki-app/src/commands/projects.rs` lines 81-82, `error.rs` lines 5-44
**Apply to:** All Rust commands in `commands/backup.rs`

```rust
.map_err(|e| AppError::Lock(e.to_string()))?
.map_err(|e| AppError::Database(e.to_string()))?
.map_err(|e| AppError::Io(e.to_string()))?
```

### Specta Type Derivation
**Source:** `crates/takoyaki-app/src/commands/projects.rs` lines 24-31, `error.rs` lines 5-6
**Apply to:** All public types in `commands/backup.rs` and `db/backups.rs` that cross the IPC boundary

```rust
// Response types going to frontend:
#[derive(Debug, serde::Serialize, specta::Type, Clone)]

// Input types from frontend:
#[derive(Debug, serde::Deserialize, specta::Type)]

// Error types:
#[derive(Debug, Error, serde::Serialize, specta::Type)]

// Event enum types for Channel:
#[derive(Clone, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
```

### Zustand Store Structure
**Source:** `src/lib/stores/device.ts` lines 1-19, `src/lib/stores/navigation.ts` lines 1-67
**Apply to:** `src/lib/stores/backup.ts`

```typescript
import { create } from "zustand";
// Interface with state fields + setter functions
// create<Interface>((set) => ({ initial state..., setters using set({}) }))
```

### shadcn Dialog Structure
**Source:** `src/components/volume-confirm-dialog.tsx` lines 1-57
**Apply to:** `src/components/backups/DryRunModal.tsx`

```typescript
"use client";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
// Props interface, functional component, Dialog with onOpenChange dismiss handler
// showCloseButton={false} for modal that requires explicit user decision (D-09)
```

### Tailwind Typography Classes
**Source:** Throughout frontend components
**Apply to:** All Phase 3 frontend components

```typescript
// Monospace data labels:
className="font-mono text-xs text-muted-foreground"
// Section headers:
className="font-mono text-sm font-semibold text-foreground"
// Project/backup names:
className="font-mono text-2xl font-semibold text-foreground"
// Accent color (warm amber):
className="text-[hsl(38,85%,55%)]" // or className="text-accent"
// Borders:
className="border-[hsl(30,8%,26%)]"
```

### useQuery Data Fetching
**Source:** `src/components/projects/ProjectTable.tsx` lines 77-84
**Apply to:** `src/components/backups/BackupsView.tsx`, `src/components/backups/BackupTimeline.tsx`

```typescript
const { data, isPending } = useQuery({
  queryKey: ["backups", projectId],
  queryFn: () => listBackups(projectId),
  enabled: projectId !== null,
});
```

### useEffect Tauri Listener Cleanup
**Source:** `src/components/tauri-event-listener.tsx` lines 11-43
**Apply to:** `src/components/backup-progress/InlineSuccessBanner.tsx` (auto-dismiss timer)

```typescript
useEffect(() => {
  const cleanupFns: (() => void)[] = [];
  async function setup() { /* ... */ }
  setup();
  return () => { for (const fn of cleanupFns) fn(); };
}, [dependencies]);
```

### SHA-256 Hashing
**Source:** `crates/takoyaki-app/src/atomic/snapshot.rs` lines 113-126 (`sha256_hex` function)
**Apply to:** `commands/backup.rs` — reuse `sha256_hex` by making it `pub` in `atomic/snapshot.rs` or copying into a shared utility

```rust
// Already implemented — make pub or re-export:
pub fn sha256_hex(path: &Path) -> Result<String, AppError> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
```

---

## No Analog Found

All files have analogs. The Channel-based progress streaming pattern has no existing example in this codebase (health check uses `app.emit()` instead), so the RESEARCH.md code examples and Tauri v2 docs are the authoritative reference for that specific sub-pattern.

| Sub-Pattern | Closest Existing | Gap |
|---|---|---|
| `tauri::ipc::Channel<T>` progress streaming | `app.emit("health-complete", ...)` in `commands/health.rs` | Health check uses fire-and-forget emit; backup needs ordered per-file Channel streaming — follow RESEARCH.md Pattern 1 exactly |
| `AtomicBool` cancellation | None | New pattern — follow RESEARCH.md Pattern 2 |
| WalkDir recursive traversal | `std::fs::read_dir` loop in `commands/projects.rs` lines 269-284 | Existing code uses manual single-level `read_dir`; WalkDir is the correct replacement for deep recursive backup traversal |

---

## Metadata

**Analog search scope:** `crates/takoyaki-app/src/`, `src/components/`, `src/lib/`, `migrations/`
**Files scanned:** 29 Rust files, 21 TypeScript/TSX files, 1 SQL migration
**Pattern extraction date:** 2026-04-30
