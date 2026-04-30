# Phase 3: Write Path and Backup — Research

**Researched:** 2026-04-30
**Domain:** Rust file I/O, backup/restore pipeline, Tauri v2 Channel API, SQLite schema extension, React/Zustand UI state for long-running operations
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Backup Destination & Organization**
- D-01: Backups live in `~/takoyaki/backups/`
- D-02: Organized `~/takoyaki/backups/PROJECT_NAME/YYYY-MM-DD_HH-MM_label/`
- D-03: Full project directory copy — every backup is complete and self-contained, including AUDIO folder
- D-04: Manual backups only. User explicitly clicks "Back Up". No auto-backup.

**Snapshot History & Timeline UX**
- D-05: Snapshot timeline is a reverse-chronological list per project with timestamp, operation label, file count, total size
- D-06: Clicking a snapshot shows file listing with change indicators (added/modified/removed/unchanged) vs current project state; includes [Restore This Snapshot]
- D-07: Backups section in top-level sidebar, accessible when disconnected

**Dry-Run Preview**
- D-08: Dry-run modal shows operation summary + file change list + [Cancel] and [Apply]. Blocks other interaction.
- D-09: Dry-run preview is ALWAYS mandatory for ALL destructive operations. No skip. No "don't show again".
- D-10: Modal includes exact line: "A snapshot of the current state will be created before applying."

**Restore Workflow & Safety**
- D-11: Every restore automatically creates a "pre-restore" snapshot before applying
- D-12: If OT disconnects mid-restore — staging dir cleaned on next launch, project unchanged. If mid-backup — partial backup deleted, app shows error.
- D-13: Success feedback is an inline banner (not modal): "✓ Backed up LIVESET_01 — 42 files · 128 MB · ~/takoyaki". Auto-dismisses after a few seconds.

### Claude's Discretion
- Progress indicator style during long backup/restore operations
- Checksum verification UX — how/whether to surface SAFE-02 result to user
- Backup deletion/cleanup UX — how users manage or prune old backups
- Snapshot retention policy
- Exact banner styling, animation, and auto-dismiss timing
- SQLite schema for backup history records
- How the backup button is presented in the Projects view

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SAFE-01 | User can back up any OT project to a Mac-side location (full directory tree copy) | `fs_copy_project_tree()` function using `walkdir` + `std::fs::copy`, destination at `~/takoyaki/backups/` |
| SAFE-02 | User can verify backup integrity via checksum comparison between source and backup | SHA-256 per-file comparison after copy, `sha2 = "0.10"` already in Cargo; extend SnapshotEngine pattern |
| SAFE-05 | User can browse snapshot history chronologically with timestamps and operation labels | New `list_backups` Tauri command queries `snapshots` table; React `BackupsView` renders timeline |
| SAFE-06 | User can restore any previous snapshot to roll a project back to prior state | `restore_snapshot` command: pre-restore snapshot → `atomic_write_batch` each file back to project dir |
| SAFE-07 | User can preview exactly what files will change before any destructive operation | Dry-run command computes diff between source and destination, returns `FileChangeManifest`; frontend renders modal |
</phase_requirements>

---

## Summary

Phase 3 builds the write path and backup system on top of the atomic write engine and snapshot infrastructure already in place from Phase 1. The core Rust primitives — `atomic_write()`, `atomic_write_batch()`, `SnapshotEngine`, SHA-256 hashing via `sha2` — are all present and tested. Phase 3's job is to wire them into user-facing commands and build the UI that surfaces them.

The main new Rust work is: a backup command that walks a project directory, computes a file manifest, writes files to `~/takoyaki/backups/`, and records the backup in SQLite; a restore command that reverses the process through the existing atomic write engine; and a dry-run command that computes what would change without writing anything. Progress reporting uses Tauri's Channel API (tagged enum events, `tauri::ipc::Channel`), which is the preferred v2 pattern for streaming data from a long-running command to the frontend.

The main new frontend work is: the Backups sidebar view (works offline), the dry-run preview modal (mandatory gatekeeper), the inline progress view (replaces content area while running), and the success banner. All design decisions are locked in the UI-SPEC. The navigation model extends the existing `useNavigationStore` from two views to three (adding `"backups"`).

The key architectural constraint is that the backup destination (`~/takoyaki/backups/`) is on a different filesystem from the OT card. This means the backup copy is NOT an atomic rename — it is a file-by-file copy. The atomicity guarantee for backup is: if interrupted, the partial backup directory is cleaned up on next launch (per D-12). For restore, atomicity is preserved because restoring uses `atomic_write_batch` which writes to the same filesystem as the OT card (staging on card → rename on card).

**Primary recommendation:** Implement as three Rust commands (`backup_project`, `restore_snapshot`, `compute_dry_run`) with Channel-based progress streaming, backed by a `db::backups` module for the SQLite extension. Frontend drives from `useBackupStore` (new Zustand store) and renders via `BackupsView`, `DryRunModal`, and `BackupProgressView`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| File copy (backup) | Rust backend | — | File I/O, filesystem access, SHA-256 hashing must happen in Rust |
| File manifest / diff computation (dry-run) | Rust backend | — | Needs direct filesystem access to compare source and destination |
| Atomic restore (write) | Rust backend | — | Uses existing `atomic_write_batch`; must run in Rust near the CF card |
| Pre-write snapshot creation | Rust backend | — | SnapshotEngine is Rust-only |
| Progress streaming | Rust backend → Frontend | Tauri Channel API | Channel is the correct v2 pattern for streaming events from long-running commands |
| Backup history persistence | Rust backend (SQLite) | — | `snapshots`/`snapshot_files` tables already exist; add `backups` table in migration V2 |
| Backup timeline UI | Frontend (React) | — | Pure read-only display of data fetched from Rust |
| Dry-run preview modal | Frontend (React) | — | Display only — manifest computed by Rust, rendered by React |
| Inline progress view | Frontend (React) | — | Renders events streamed from Channel; no logic beyond display |
| Cancellation signaling | Frontend → Rust backend | Tauri event | Frontend emits cancel event; Rust backend checks `AtomicBool` flag per operation |
| Backup destination path | Rust backend | — | `dirs::home_dir().join("takoyaki/backups/")` — computed in Rust, not from frontend |
| Success banner state | Frontend (React/Zustand) | — | Local transient UI state, auto-dismisses |

---

## Standard Stack

### Core — Already in Project

| Library | Version | Purpose | Status |
|---------|---------|---------|--------|
| `atomic-write-file` | 0.3.0 | Atomic file writes for restore operations | [VERIFIED: cargo search] — already in `Cargo.toml` |
| `sha2` | 0.10.x | SHA-256 per-file checksums for backup verification | [VERIFIED: Cargo.toml] — `sha2 = "0.10"` present |
| `tempfile` | 3.x | Staging directories | [VERIFIED: Cargo.toml] — present |
| `rusqlite` | 0.39 (bundled) | SQLite for backup history | [VERIFIED: workspace Cargo.toml] — `rusqlite = { version = "0.39", features = ["bundled"] }` |
| `dirs` | 6.x | Resolve `~/takoyaki/backups/` home path | [VERIFIED: Cargo.toml] — `dirs = "6"` present |
| `serde` / `serde_json` | 1.x | Serialization for IPC types | [VERIFIED: Cargo.toml] — present |
| `tracing` | 0.1.x | Structured logging | [VERIFIED: Cargo.toml] — present |
| `tokio` | 1.x (time feature) | Async runtime for Tauri commands | [VERIFIED: Cargo.toml] — present |

### New Dependencies Needed

| Library | Version | Purpose | Why Needed |
|---------|---------|---------|------------|
| `walkdir` | 2.5.0 | Recursive directory walk for backup copy and diff | [VERIFIED: cargo search] — `zip` crate docs use it for directory traversal; no equivalent in std |
| `zip` | 8.6.0 | Optional: project export as .zip archive (MGMT-03 prep) | [VERIFIED: cargo search] — not strictly needed for Phase 3, but zip is in CLAUDE.md stack list; out of scope for Phase 3 unless needed for snapshot format |

**Note on zip:** Phase 3 does NOT require zip. Backups are plain directory copies, not archives (per D-03: "a backup can be restored and played immediately"). Zip is out of scope until Phase 4 (MGMT-03).

### Frontend — Already in Project

| Library | Version | Purpose | Status |
|---------|---------|---------|--------|
| `zustand` | 5.0.12 | Backup operation state store | [VERIFIED: package.json] |
| `@tanstack/react-query` | 5.100.6 | Fetching backup history list | [VERIFIED: package.json] |
| `@tauri-apps/api` | 2.11.0 | `invoke`, `listen`, `Channel` | [VERIFIED: package.json] |
| `tw-animate-css` | 1.4.0 | Banner slide-out animation | [VERIFIED: package.json] |
| `lucide-react` | 1.8.0 | Icons: Archive, CircleCheck, X, Loader | [VERIFIED: package.json] |

### New Frontend Components Needed

| Component | Source | Notes |
|-----------|--------|-------|
| `scroll-area` | shadcn official | Dry-run file list + snapshot timeline scroll [VERIFIED: UI-SPEC] |
| `BackupsView` | custom | Reverse-chronological backup timeline per project |
| `DryRunModal` | custom (extends shadcn `dialog`) | Mandatory confirmation modal |
| `BackupProgressView` | custom | Progress bar + file counter, replaces content area |
| `InlineSuccessBanner` | custom | Fixed strip, auto-dismiss at 4 seconds |
| `SnapshotDetailPanel` | custom | Inline below selected snapshot row |

**Installation (new Rust dependency):**
```bash
# Add to crates/takoyaki-app/Cargo.toml
walkdir = "2.5.0"
```

**Installation (new frontend component):**
```bash
npx shadcn@latest add scroll-area
```

---

## Architecture Patterns

### System Architecture Diagram

```
USER                  FRONTEND (React)                RUST BACKEND
 │                         │                               │
 │ click "Back Up"         │                               │
 ├────────────────────────►│                               │
 │                         │  invoke("compute_dry_run")    │
 │                         ├──────────────────────────────►│
 │                         │                               │ walk project dir
 │                         │                               │ walk backup dest (if exists)
 │                         │                               │ compute diff (added/modified/removed/unchanged)
 │                         │  FileChangeManifest           │
 │                         │◄──────────────────────────────┤
 │    DryRunModal renders  │                               │
 │◄────────────────────────┤                               │
 │                         │                               │
 │ click [Apply]           │                               │
 ├────────────────────────►│                               │
 │                         │  invoke("backup_project",     │
 │                         │         channel: onEvent)     │
 │                         ├──────────────────────────────►│
 │                         │                               │ SnapshotEngine.snapshot_files()  ← pre-backup snapshot
 │                         │                               │ walk project dir with walkdir
 │  progress view appears  │  Channel: BackupEvent::Started│
 │◄────────────────────────┼◄──────────────────────────────┤
 │                         │                               │ for each file: std::fs::copy()
 │  "12 of 42 files"       │  Channel: BackupEvent::Progress│
 │◄────────────────────────┼◄──────────────────────────────┤
 │                         │                               │ sha256_compare(source, dest) per file
 │                         │                               │ db::backups::insert_backup()
 │                         │  Channel: BackupEvent::Complete│
 │  success banner         │◄──────────────────────────────┤
 │◄────────────────────────┤                               │
```

For restore, the flow is:
```
RESTORE PATH:
invoke("compute_dry_run", { source: snapshot_dir, dest: project_dir })
  → returns FileChangeManifest (same structure as backup dry-run)
  → user sees DryRunModal with [Restore Snapshot] destructive button
invoke("restore_snapshot", { snapshot_id, channel: onEvent })
  → SnapshotEngine.snapshot_files(all project files, "pre-restore")  ← safety net
  → atomic_write_batch(snapshot files → project dir)                  ← atomic, on-card
  → db::backups::insert_snapshot("pre-restore", ...)
  → Channel: RestoreEvent::Complete
```

### Recommended Project Structure (additions only)

```
crates/takoyaki-app/src/
├── commands/
│   ├── backup.rs        # backup_project, restore_snapshot, compute_dry_run, list_backups commands
│   └── mod.rs           # add pub mod backup
├── db/
│   ├── backups.rs       # insert_backup, list_backups, get_backup_files, delete_backup DB functions
│   └── mod.rs           # add migration V2 for backups table
└── atomic/
    └── snapshot.rs      # existing — unchanged

migrations/
└── V2__backup_schema.sql   # new backups + backup_files tables

src/
├── components/
│   ├── backups/
│   │   ├── BackupsView.tsx          # top-level Backups section
│   │   ├── BackupTimeline.tsx       # reverse-chrono list with project group headers
│   │   ├── SnapshotRow.tsx          # one row: timestamp + label + file count + size + [Restore]
│   │   ├── SnapshotDetailPanel.tsx  # inline file diff list + [Restore This Snapshot]
│   │   └── DryRunModal.tsx          # mandatory dry-run confirmation modal
│   ├── backup-progress/
│   │   ├── BackupProgressView.tsx   # replaces content area during operation
│   │   └── InlineSuccessBanner.tsx  # fixed top strip, auto-dismisses
│   └── project-detail/
│       └── MetadataHeader.tsx       # add [Back Up] button (already exists, add prop/handler)
├── lib/
│   └── stores/
│       └── backup.ts                # useBackupStore — operation state, progress, banner
└── bindings.ts                      # auto-generated by tauri-specta (add new commands)
```

### Pattern 1: Tauri Channel for Progress Streaming

**What:** A typed enum sent from Rust → frontend via `tauri::ipc::Channel` as the long-running command executes. No extra round-trip; all progress arrives in order.

**When to use:** Any operation that takes > ~1 second and has per-file progress to report. Both backup and restore use this.

```rust
// Source: https://context7.com/tauri-apps/tauri-docs
// crates/takoyaki-app/src/commands/backup.rs

use tauri::ipc::Channel;
use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum BackupEvent {
    Started {
        total_files: usize,
        destination: String,
    },
    Progress {
        files_copied: usize,
        total_files: usize,
        current_file: String,
    },
    Complete {
        files_copied: usize,
        total_bytes: u64,
        destination: String,
        checksum_ok: bool,
    },
    Failed {
        reason: String,
    },
}

#[tauri::command]
#[specta::specta]
pub async fn backup_project(
    state: tauri::State<'_, AppState>,
    project_id: String,
    on_event: Channel<BackupEvent>,
) -> Result<(), AppError> {
    // ... implementation
}
```

```typescript
// Source: https://context7.com/tauri-apps/tauri-docs
// src/components/backup-progress/BackupProgressView.tsx
import { invoke, Channel } from '@tauri-apps/api/core';

const channel = new Channel<BackupEvent>();
channel.onmessage = (event) => {
  if (event.event === 'progress') {
    setProgress(event.data.filesCopied, event.data.totalFiles, event.data.currentFile);
  } else if (event.event === 'complete') {
    setComplete(event.data);
  }
};
await invoke('backup_project', { projectId, onEvent: channel });
```

### Pattern 2: Cancellation via AtomicBool in AppState

**What:** Frontend emits a cancel event; Rust backend checks an `AtomicBool` flag inside the file copy loop. On cancel, the incomplete backup directory is deleted.

**When to use:** Backup and restore commands that loop over files.

```rust
// Cancellation check inside backup copy loop
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// In AppState, add:
pub cancel_backup: Arc<AtomicBool>,

// Inside backup loop:
if state.cancel_backup.load(Ordering::Relaxed) {
    // Clean up partial destination directory
    let _ = std::fs::remove_dir_all(&dest_dir);
    return Err(AppError::Cancelled("Backup cancelled by user".to_string()));
}
```

**Note:** This is a polling-based cancellation. It checks at each file boundary — suitable for backup where files are copied one at a time. For large single-file copies (audio files > 100MB), the cancel check between files is the correct granularity; mid-file cancellation is not needed.

### Pattern 3: Dry-Run File Manifest Computation

**What:** Walk both source (project dir) and destination (backup dir if it exists). Compare by filename + size + mtime or hash. Return a `FileChangeManifest` to the frontend.

**For backup dry-run:** Destination is `~/takoyaki/backups/PROJECT/DATE_label/` — always fresh, so all files are "Added". The manifest purpose is to show the user what will be copied.

**For restore dry-run:** Destination is the live project dir on the OT card. Compare snapshot files vs current project files to show what would change.

```rust
// [ASSUMED] — pattern based on standard file diffing approach
#[derive(Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct FileChangeEntry {
    pub path: String,         // relative to project root
    pub change_type: ChangeType,
    pub size_bytes: u64,
}

#[derive(Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum ChangeType {
    Added,
    Modified,
    Removed,
    Unchanged,
}

#[derive(Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct FileChangeManifest {
    pub entries: Vec<FileChangeEntry>,
    pub total_added: usize,
    pub total_modified: usize,
    pub total_removed: usize,
    pub total_unchanged: usize,
    pub total_bytes: u64,
    pub destination_path: String,
}
```

### Pattern 4: SQLite Migration V2 for Backup History

**What:** The existing V1 schema has a `snapshots` table used for internal pre-write snapshots (SAFE-03). Phase 3 needs a separate `backups` table that records user-visible backup events (SAFE-01/SAFE-05). These are conceptually different: snapshots are internal safety infrastructure; backups are user-controlled archive operations.

**Why separate tables:** The CONTEXT.md D-05 timeline shows "manual backup" and "pre-restore" as distinct operation labels. The Backups view (D-07) shows only user-created backups. Internal pre-write snapshots are not shown in the backup timeline. Mixing them into one table would complicate queries and surface internal plumbing to the user.

```sql
-- migrations/V2__backup_schema.sql

-- User-created backup records (SAFE-01, SAFE-05)
CREATE TABLE backups (
    id           TEXT PRIMARY KEY NOT NULL,
    project_id   TEXT NOT NULL,          -- references projects.id (not FK — survives card removal)
    project_name TEXT NOT NULL,
    dest_path    TEXT NOT NULL,          -- ~/takoyaki/backups/PROJECT/DATE_label/
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    operation    TEXT NOT NULL,          -- 'manual-backup', 'pre-restore'
    file_count   INTEGER NOT NULL,
    total_bytes  INTEGER NOT NULL,
    checksum_ok  INTEGER NOT NULL DEFAULT 1  -- 0 = failed, 1 = verified
);

-- Files captured in each backup
CREATE TABLE backup_files (
    id           TEXT PRIMARY KEY NOT NULL,
    backup_id    TEXT NOT NULL REFERENCES backups(id) ON DELETE CASCADE,
    relative_path TEXT NOT NULL,
    stored_path  TEXT NOT NULL,
    file_hash    TEXT NOT NULL,
    size_bytes   INTEGER NOT NULL,
    change_type  TEXT NOT NULL  -- 'added','modified','removed','unchanged'
);

CREATE INDEX idx_backups_project_id ON backups(project_id);
CREATE INDEX idx_backups_created_at ON backups(created_at);
CREATE INDEX idx_backup_files_backup_id ON backup_files(backup_id);
```

**Note on project_id FK:** No `REFERENCES projects(id)` constraint. The projects table is cleared and rebuilt on each card mount (see `clear_projects()` in `db/projects.rs`). A FK would cause backup records to become orphaned when the card is re-indexed. Store project_id as a plain TEXT reference and project_name for offline display.

### Anti-Patterns to Avoid

- **Backup to same filesystem as target (for restore):** `atomic_write_batch` for restore MUST write to the OT card, not to `~/takoyaki/`. The staging file must be on the same volume as the destination. AtomicWriteFile places the temp file in the same directory as the target by default — this is correct and must not be changed.
- **Using `app.emit()` instead of `Channel` for progress:** `app.emit()` is fire-and-forget and does not guarantee ordering or delivery. For sequential per-file progress, use `Channel` which maintains FIFO ordering. [CITED: tauri-docs/calling-frontend.mdx]
- **Reading backup history from filesystem:** Don't scan `~/takoyaki/backups/` to build the backup timeline. Always read from SQLite. If a backup directory is manually deleted by the user, the SQLite record remains (dangling reference) — handle gracefully with an existence check on restore.
- **Blocking the Tauri command thread:** Backup and restore are long-running. Use `async fn` with `tauri::async_runtime::spawn_blocking` for the file copy loop (which is inherently synchronous I/O). Avoid blocking the async executor.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Recursive directory traversal | Manual `read_dir` recursion | `walkdir 2.5` | Stack-safe, handles symlinks, handles deep trees [VERIFIED: cargo search] |
| Per-file SHA-256 | Custom hash loop | `sha2 0.10` (already present) | Already in `atomic/snapshot.rs` as `sha256_hex()` — reuse directly |
| Atomic file writes for restore | Custom temp-file-rename | `atomic-write-file 0.3` + existing `atomic_write_batch()` | Already implemented and tested in Phase 1 |
| Home directory path | Hardcoded `~` expansion | `dirs::home_dir()` (already present) | Handles edge cases on macOS; `dirs = "6"` already in `Cargo.toml` |
| Progress streaming | Event polling with timers | `tauri::ipc::Channel<T>` | v2 canonical pattern; ordered delivery, no polling [CITED: tauri-docs] |
| Focus trapping in modal | Custom tab-cycle logic | shadcn `dialog` (already present) | Radix UI Dialog provides built-in focus trap |
| Scrollable file list | Custom overflow div | shadcn `scroll-area` (new) | Radix ScrollArea handles keyboard scroll, correct overflow on macOS |
| Auto-dismiss timer | `setTimeout` in component | React `useEffect` + `setTimeout` cleanup | Standard pattern — no library needed, but must clear timer on unmount |

**Key insight:** The file copy and checksum infrastructure is already ~80% built in `atomic/snapshot.rs`. `sha256_hex()` is there. The SnapshotEngine copies files. The backup command is essentially: SnapshotEngine + walkdir traversal + destination path logic + SQLite record + Channel progress.

---

## Runtime State Inventory

> Not a rename/refactor/migration phase. Included for completeness.

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | `snapshots` and `snapshot_files` tables in SQLite (V1 schema) — used internally for SAFE-03 | No migration of existing data needed; add new `backups` / `backup_files` tables in V2 migration |
| Live service config | None | — |
| OS-registered state | None | — |
| Secrets/env vars | None | — |
| Build artifacts | None | — |

---

## Common Pitfalls

### Pitfall 1: FAT32 Rename Atomicity (Research Flag from STATE.md)

**What goes wrong:** The Phase 1 STATE.md explicitly flags: "Research flag — verify FAT32 rename atomicity with integration test on real FAT32 volume before depending on it."

**Why it matters:** `atomic_write_batch` uses `AtomicWriteFile::commit()` which calls `rename()`. POSIX guarantees atomicity of rename within the same filesystem. macOS FAT32 volumes (HFS wrapper) also honor atomic rename within the same volume — but this is not guaranteed by the FAT32 spec itself.

**Current posture:** Phase 1 already made the correct architectural choice: staging file is in the same directory as the target (same volume). This is correct. The research flag is a reminder to add an integration test against a real FAT32 mount, not a signal that the implementation is wrong.

**How to avoid:** The restore command uses `atomic_write_batch` which is already correct. Add a comment in the restore command: `// SAFE-04: staging on same volume as target (FAT32 rename atomicity)`. The integration test can be deferred to Phase 5 when SMPL-01 also needs to write to the card.

**Warning signs:** `EXDEV` errors from rename = cross-filesystem rename attempted = staging file ended up on a different volume from the target.

### Pitfall 2: Backup Destination NOT on Same Filesystem — Do NOT Use Atomic Rename for Backup

**What goes wrong:** `~/takoyaki/backups/` is on the Mac's HFS+/APFS filesystem. The OT card is FAT32. You cannot atomically rename between them (EXDEV). Even within `~/takoyaki/`, you would not want to use `atomic_write_batch` because:
1. There is no "current state" to protect — a new backup destination directory does not exist yet.
2. The backup copy is inherently a sequence of file copies, not an atomic operation.

**The correct approach:** For backup: `std::fs::copy()` file by file into the destination dir. If interrupted, the destination dir is incomplete. On next launch, detect and delete incomplete backup dirs (those without a corresponding completed SQLite record). This satisfies D-12.

**For restore:** `atomic_write_batch` from snapshot files back to the OT card. This IS atomic (same filesystem — both staging and target are on the card).

**Warning signs:** Using `AtomicWriteFile` to write to `~/takoyaki/backups/` — this will work (it's not wrong per se) but wastes a temp file and adds overhead for no safety benefit when writing to a fresh destination directory.

### Pitfall 3: projects Table FK Constraint on backups

**What goes wrong:** Adding `REFERENCES projects(id)` to the `backups.project_id` column causes backup records to be deleted (via cascade) or blocked (FK violation) when the projects table is cleared during re-indexing.

**Why it happens:** `clear_projects()` runs `DELETE FROM projects` on every card mount. With a FK constraint and `ON DELETE CASCADE`, all backup records for the reconnected project would be deleted. Without cascade, `clear_projects()` would fail with FK constraint violations.

**How to avoid:** Store `project_id` as `TEXT NOT NULL` without a REFERENCES constraint. Store `project_name` as a denormalized column for display in the Backups view when the project row has been cleared. See Pattern 4 SQL above.

### Pitfall 4: Tauri State Mutex Deadlock During Long-Running Commands

**What goes wrong:** Holding a `Mutex<db::Database>` lock for the duration of a backup (which loops over potentially 200+ files) blocks every other Tauri command from accessing the database.

**How it happens:** `state.db.lock()` inside the backup loop holds the lock until the loop completes.

**How to avoid:** Lock the Mutex only for the SQLite reads/writes (path lookups, record inserts). Do the file copy loop outside the lock. Pattern:
```rust
// GOOD: lock only for DB access
let card_path = {
    let db = state.db.lock()?;
    db::projects::get_card_path(&db.conn, &project_id)?
};
// File copy loop runs without holding the lock
for entry in walkdir::WalkDir::new(&card_path) { ... }
// Lock again only for the final DB insert
{
    let db = state.db.lock()?;
    db::backups::insert_backup(&db.conn, &backup_record)?;
}
```

### Pitfall 5: Showing Internal Snapshots in the Backups Timeline

**What goes wrong:** `SELECT * FROM snapshots ORDER BY created_at DESC` returns both user-visible "manual backup" events AND internal "pre-write" snapshots (created by SAFE-03 infrastructure for every write operation). The Backups view becomes polluted with internal plumbing.

**How to avoid:** Use separate tables (`backups` for user-visible, `snapshots` for internal). The Backups view queries only `backups`. Internal snapshot infrastructure continues to use the existing `snapshots` / `snapshot_files` tables unchanged.

### Pitfall 6: Tauri Channel Not Registered with tauri-specta

**What goes wrong:** `Channel<BackupEvent>` in a command signature causes the specta TypeScript export to fail or produce incorrect types if `BackupEvent` is not decorated with `#[derive(specta::Type)]`.

**How to avoid:** All event payload types must derive both `serde::Serialize` and `specta::Type`. The `Channel<T>` parameter is handled by tauri-specta automatically when `T: specta::Type`. [ASSUMED — based on tauri-specta pattern from Phase 1 codebase]

---

## Code Examples

Verified patterns from official sources and existing codebase:

### Walkdir Recursive Copy (Backup Implementation)

```rust
// Source: https://context7.com/zip-rs/zip2 (walkdir pattern), adapted for backup
// crates/takoyaki-app/src/commands/backup.rs

use walkdir::WalkDir;
use std::path::Path;

pub fn copy_project_tree(
    src: &Path,
    dest: &Path,
    on_progress: impl Fn(usize, &Path),
) -> Result<Vec<CopiedFile>, AppError> {
    std::fs::create_dir_all(dest)?;
    let mut copied = Vec::new();
    let mut count = 0usize;

    for entry in WalkDir::new(src).min_depth(1) {
        let entry = entry.map_err(|e| AppError::Io(e.to_string()))?;
        let rel_path = entry.path().strip_prefix(src)
            .map_err(|_| AppError::InvalidPath)?;
        let dest_path = dest.join(rel_path);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&dest_path)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let bytes = std::fs::copy(entry.path(), &dest_path)?;
            count += 1;
            on_progress(count, entry.path());
            copied.push(CopiedFile {
                src: entry.path().to_path_buf(),
                dest: dest_path,
                size_bytes: bytes,
            });
        }
    }
    Ok(copied)
}
```

### Tauri Channel Event Streaming

```rust
// Source: https://github.com/tauri-apps/tauri-docs/blob/v2/src/content/docs/develop/calling-frontend.mdx
use tauri::ipc::Channel;

#[tauri::command]
#[specta::specta]
pub async fn backup_project(
    state: tauri::State<'_, AppState>,
    project_id: String,
    on_event: Channel<BackupEvent>,
) -> Result<(), AppError> {
    on_event.send(BackupEvent::Started {
        total_files: file_count,
        destination: dest_path.to_string_lossy().into(),
    }).map_err(|e| AppError::Io(e.to_string()))?;

    // ... copy loop emits BackupEvent::Progress per file ...

    on_event.send(BackupEvent::Complete { ... })
        .map_err(|e| AppError::Io(e.to_string()))?;
    Ok(())
}
```

### Rusqlite Transaction for Backup Record Insert

```rust
// Source: https://docs.rs/rusqlite/0.39.0/rusqlite/struct.Transaction.html
pub fn insert_backup(conn: &Connection, record: &BackupRecord) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;  // rolls back on drop if not committed

    tx.execute(
        "INSERT INTO backups (id, project_id, project_name, dest_path, operation, \
         file_count, total_bytes, checksum_ok) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        rusqlite::params![
            record.id, record.project_id, record.project_name, record.dest_path,
            record.operation, record.file_count as i64, record.total_bytes as i64,
            record.checksum_ok as i64,
        ],
    )?;

    for file in &record.files {
        tx.execute(
            "INSERT INTO backup_files (id, backup_id, relative_path, stored_path, \
             file_hash, size_bytes, change_type) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            rusqlite::params![
                file.id, record.id, file.relative_path, file.stored_path,
                file.file_hash, file.size_bytes as i64, file.change_type,
            ],
        )?;
    }

    tx.commit()
}
```

### Frontend Channel Listener (TypeScript)

```typescript
// Source: https://github.com/tauri-apps/tauri-docs/blob/v2/src/content/docs/develop/calling-frontend.mdx
import { invoke, Channel } from '@tauri-apps/api/core';

const channel = new Channel<BackupEvent>();
channel.onmessage = (message) => {
  switch (message.event) {
    case 'started':
      setTotal(message.data.totalFiles);
      break;
    case 'progress':
      setCopied(message.data.filesCopied);
      setCurrentFile(message.data.currentFile);
      break;
    case 'complete':
      onComplete(message.data);
      break;
    case 'failed':
      onError(message.data.reason);
      break;
  }
};
await invoke('backup_project', { projectId, onEvent: channel });
```

### Auto-Dismiss Banner (React)

```typescript
// [ASSUMED] — standard React useEffect timer pattern
// src/components/backup-progress/InlineSuccessBanner.tsx
useEffect(() => {
  if (!visible) return;
  const timer = setTimeout(() => setVisible(false), 4000);
  return () => clearTimeout(timer); // cleanup on unmount or re-render
}, [visible]);
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `app.emit()` for progress | `tauri::ipc::Channel<T>` | Tauri v2 | Channel is ordered, typed, and does not require unlisten cleanup — preferred for streaming [CITED: tauri-docs v2] |
| `app.listen()` in frontend | `listen()` from `@tauri-apps/api/event` + `Channel.onmessage` | Tauri v2 | Channel avoids event name strings; typed via generics |
| Global `Mutex<Vec<Snapshot>>` in memory | SQLite persistence via rusqlite | Project standard | Backup history survives app restarts |

**Deprecated/outdated:**
- `tauri::Window::emit()` (v1 pattern): replaced by `AppHandle::emit()` and `Channel<T>` in v2. Do not use the v1 window-based emit pattern.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `Channel<BackupEvent>` parameter requires `BackupEvent: specta::Type` for tauri-specta TypeScript export to succeed | Code Examples, Common Pitfalls | TypeScript bindings export fails; workaround is to manually type the channel payload |
| A2 | Auto-dismiss banner uses `useEffect` + `setTimeout` without a third-party library | Don't Hand-Roll | No risk — this is universally correct React pattern |
| A3 | Restore dry-run should compare snapshot files vs current project files using SHA-256 for "modified" detection (not just mtime) | Architecture Patterns | If mtime is sufficient, SHA-256 comparison is unnecessary overhead; but SHA-256 is safer and already in the codebase |
| A4 | `AtomicBool` cancellation check between file copies is sufficient granularity (no need for mid-file cancellation) | Architecture Patterns | For very large files (>500MB audio), cancel latency could be noticeable; accept this limitation for Phase 3 |

**Claims A1–A4 are LOW risk.** The code patterns they describe are standard and the project already follows them. No user confirmation needed before planning.

---

## Open Questions (RESOLVED)

1. **Incomplete backup cleanup on launch**
   - What we know: D-12 says partial backup is deleted on next launch
   - What's unclear: The mechanism — scan `~/takoyaki/backups/` for directories with no corresponding `completed` record in SQLite? Or write a `.in-progress` marker file that is deleted on completion?
   - Recommendation: Use a SQLite `status` column on the `backups` table (`'in-progress'` → `'complete'`). On launch, delete any backup directories whose SQLite record has status `'in-progress'`. Simpler than scanning the filesystem.

2. **Checksum verification UX (Claude's Discretion)**
   - What we know: SAFE-02 requires checksum verification. UI-SPEC defines "✓ Verified" badge inline in success banner.
   - What's unclear: Should every file be SHA-256 compared, or a spot-check sample?
   - Recommendation: Full verification (compare every file's hash between source and backup copy). OT projects are typically < 1GB total. Full verification adds < 2 seconds for typical project sizes and is the correct behavior for a backup tool where trust is paramount.

3. **Snapshot detail "compare vs current state" when disconnected (D-06)**
   - What we know: D-06 says change indicators compare backup against "current project state on the OT card (or last known state if disconnected)"
   - What's unclear: What is the "last known state" when disconnected? The projects table does not store file manifests.
   - Recommendation: When disconnected, the dry-run manifest shows the snapshot files vs the backup's own `backup_files` records (i.e., what files exist in that backup). The "current state" comparison is only meaningful during a live restore when the card is connected. Show a badge "Connect your Octatrack to see changes from current state" when disconnected.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` / Rust toolchain | Rust compilation | ✓ | (in project) | — |
| `walkdir` crate | Backup copy traversal | ✗ (not yet in Cargo.toml) | 2.5.0 | None — needed |
| `~/takoyaki/backups/` directory | Backup destination | ✗ (created on first backup) | — | `std::fs::create_dir_all()` |
| shadcn scroll-area | Dry-run file list | ✗ (not yet installed) | via shadcn CLI | `npx shadcn@latest add scroll-area` |
| OT card (FAT32) | Restore operation | Device-dependent | — | Graceful error: "Connect Octatrack to restore" |

**Missing dependencies with no fallback:**
- `walkdir` — must be added to `Cargo.toml` before backup command can be implemented

**Missing dependencies with fallback or deferred:**
- `scroll-area` — add via shadcn CLI in Wave 0 setup task
- OT card — graceful offline handling already designed in UI-SPEC

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test runner (`cargo test`) |
| Config file | `Cargo.toml` per crate |
| Quick run command | `cargo test -p takoyaki-app --lib` |
| Full suite command | `cargo test` (all workspace crates) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SAFE-01 | `backup_project` copies all files to `~/takoyaki/backups/PROJECT/DATE_label/` | unit (tempdir) | `cargo test -p takoyaki-app backup` | ❌ Wave 0 |
| SAFE-01 | Backup directory structure matches `PROJECT_NAME/YYYY-MM-DD_HH-MM_label/` format | unit | `cargo test -p takoyaki-app backup_dir_format` | ❌ Wave 0 |
| SAFE-02 | SHA-256 of each source file matches SHA-256 of copied file | unit | `cargo test -p takoyaki-app backup_checksum` | ❌ Wave 0 |
| SAFE-02 | Checksum mismatch detected and reported in `BackupEvent::Complete` | unit | `cargo test -p takoyaki-app backup_checksum_fail` | ❌ Wave 0 |
| SAFE-05 | `list_backups` returns records ordered by `created_at DESC` | unit (in-memory DB) | `cargo test -p takoyaki-app list_backups` | ❌ Wave 0 |
| SAFE-06 | `restore_snapshot` creates pre-restore snapshot before overwriting | unit (tempdir) | `cargo test -p takoyaki-app restore_creates_pre_snapshot` | ❌ Wave 0 |
| SAFE-06 | `restore_snapshot` uses `atomic_write_batch` — original unchanged on failure | unit | `cargo test -p takoyaki-app restore_atomic` | ❌ Wave 0 |
| SAFE-07 | `compute_dry_run` returns correct `Added`/`Modified`/`Removed`/`Unchanged` classification | unit | `cargo test -p takoyaki-app dry_run_manifest` | ❌ Wave 0 |
| SAFE-07 | `compute_dry_run` does not modify any files | unit | `cargo test -p takoyaki-app dry_run_no_write` | ❌ Wave 0 |
| D-12 | Interrupted backup (simulated) leaves no partial backup record in SQLite | unit | `cargo test -p takoyaki-app backup_interrupted_cleanup` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p takoyaki-app --lib 2>&1 | tail -5`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `crates/takoyaki-app/tests/backup.rs` — covers SAFE-01, SAFE-02 (backup copy + checksum)
- [ ] `crates/takoyaki-app/tests/restore.rs` — covers SAFE-06 (restore + pre-restore snapshot)
- [ ] `crates/takoyaki-app/tests/dry_run.rs` — covers SAFE-07 (manifest computation, no writes)
- [ ] `crates/takoyaki-app/tests/backup_db.rs` — covers SAFE-05 (list_backups query, V2 schema)
- [ ] `migrations/V2__backup_schema.sql` — needed before any backup DB tests can run

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | yes (partial) | Backup destination path must be within `~/takoyaki/backups/` — reject paths outside |
| V5 Input Validation | yes | `project_id` is an opaque UUID looked up from DB (T-02-04 pattern from Phase 2 — reuse) |
| V6 Cryptography | yes | SHA-256 via `sha2 0.10` — no hand-rolled crypto |

### Known Threat Patterns for This Stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal in backup destination | Tampering | Destination always computed from `dirs::home_dir().join("takoyaki/backups/")` in Rust — never from frontend-supplied path |
| Path traversal in restore source | Tampering | `snapshot_id` resolved to path via DB lookup (same T-02-04 pattern as `project_id`) — frontend never supplies raw paths |
| Symlink following during backup copy | Tampering/Elevation | `walkdir` follows symlinks by default — use `WalkDir::new().follow_links(false)` for backup traversal of user content |
| Large file exhaustion (restore from untrusted snapshot) | Denial of Service | Snapshots are always created by the app from real project files — no external snapshot import in Phase 3; low risk |

---

## Sources

### Primary (HIGH confidence)

- `crates/takoyaki-app/src/atomic/mod.rs` — Existing `atomic_write_batch` implementation confirmed
- `crates/takoyaki-app/src/atomic/snapshot.rs` — Existing `SnapshotEngine` and `sha256_hex` confirmed
- `migrations/V1__initial_schema.sql` — V1 schema confirmed (`snapshots`, `snapshot_files`, `projects` tables)
- `crates/takoyaki-app/Cargo.toml` — Confirmed: `sha2 = "0.10"`, `dirs = "6"`, `atomic-write-file = "0.3"`, `tempfile = "3"`, `tokio`
- `.planning/phases/03-write-path-and-backup/03-UI-SPEC.md` — Full UI design contract confirmed and locked
- Context7 `/tauri-apps/tauri-docs` — `Channel<T>` pattern for progress streaming confirmed with code examples

### Secondary (MEDIUM confidence)

- Context7 `/zip-rs/zip2` — `walkdir` pattern for recursive directory traversal
- Context7 `/websites/rs_rusqlite_0_39_0_rusqlite` — Transaction pattern for batch insert confirmed
- `cargo search walkdir` — version 2.5.0 verified current
- `npm view @radix-ui/react-scroll-area version` — 1.2.10 current
- `npm view tw-animate-css version` — 1.4.0 current (already in project)

### Tertiary (LOW confidence)

- None

---

## Project Constraints (from CLAUDE.md)

| Directive | Impact on Phase 3 |
|-----------|-------------------|
| Atomic writes + snapshot-before-write for ALL write operations. No exceptions. | Restore uses `atomic_write_batch`. Pre-backup and pre-restore snapshots are mandatory. |
| No GPL dependencies | `walkdir` is MIT licensed. `zip` (if used) is MIT. No GPL risk. |
| Clean-room OT parser — no GPL dependencies | Backup copies OT files as opaque bytes — no parsing needed for backup. Restore also copies bytes. No parser involvement. |
| MIT license for all project code | All dependencies in this phase (walkdir, sha2, atomic-write-file, dirs) are MIT or Apache-2.0. |
| SQLite for Takoyaki's own metadata | Backup history stored in SQLite `backups` table. |
| Full test coverage. OT binary parser must have extensive test fixtures. | Phase 3 backup/restore commands need test coverage (Wave 0 gaps listed above). |
| Tauri v2 + React/Next.js frontend | All IPC via `invoke` + `Channel`. Frontend follows existing patterns. |
| GSD Workflow Enforcement | All implementation goes through GSD execute-phase workflow. |

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all dependencies verified via Cargo.toml grep and cargo search
- Architecture: HIGH — patterns derived from existing codebase (SnapshotEngine, atomic_write_batch) and Tauri v2 official docs
- Pitfalls: HIGH — three pitfalls from STATE.md research flags + two from direct code inspection (FK constraint, mutex deadlock)
- SQLite schema: HIGH — derived from existing V1 schema structure and CONTEXT.md requirements
- Frontend patterns: HIGH — UI-SPEC is locked and detailed; patterns follow existing Phase 2 code

**Research date:** 2026-04-30
**Valid until:** 2026-05-30 (stable stack — Rust/Tauri versions unlikely to change materially)
