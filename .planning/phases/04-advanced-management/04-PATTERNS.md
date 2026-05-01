# Phase 4: Advanced Management - Pattern Map

**Mapped:** 2026-05-01
**Files analyzed:** 11 new/modified files
**Analogs found:** 11 / 11

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/takoyaki-app/src/commands/management.rs` | controller | request-response | `crates/takoyaki-app/src/commands/backup.rs` | exact |
| `crates/takoyaki-app/src/commands/mod.rs` | config | - | `crates/takoyaki-app/src/commands/mod.rs` | exact |
| `crates/takoyaki-app/src/lib.rs` | config | - | `crates/takoyaki-app/src/lib.rs` | exact |
| `crates/takoyaki-app/src/management/mod.rs` | service | CRUD | `crates/takoyaki-app/src/atomic/mod.rs` | role-match |
| `crates/takoyaki-app/src/management/project_work.rs` | utility | transform | `crates/takoyaki-app/src/commands/samples.rs` | role-match |
| `crates/takoyaki-app/src/management/duplicate.rs` | service | file-I/O | `crates/takoyaki-app/src/commands/backup.rs` | role-match |
| `crates/takoyaki-app/src/management/rename.rs` | service | CRUD | `crates/takoyaki-app/src/db/projects.rs` | role-match |
| `crates/takoyaki-app/src/management/export.rs` | service | file-I/O | `crates/takoyaki-app/src/commands/backup.rs` | role-match |
| `crates/takoyaki-app/src/management/bank_copy.rs` | service | file-I/O | `crates/takoyaki-app/src/commands/backup.rs` | role-match |
| `src/lib/stores/management.ts` | store | event-driven | `src/lib/stores/backup.ts` | exact |
| `src/lib/tauri.ts` | utility | request-response | `src/lib/tauri.ts` | exact |
| `src/components/project-detail/MetadataHeader.tsx` | component | request-response | `src/components/project-detail/MetadataHeader.tsx` | exact |
| `src/components/project-detail/BankGridCell.tsx` | component | event-driven | `src/components/project-detail/BankGridCell.tsx` | exact |
| `src/components/management/BankCopyPickerDialog.tsx` | component | request-response | `src/components/volume-confirm-dialog.tsx` | role-match |

---

## Pattern Assignments

### `crates/takoyaki-app/src/commands/management.rs` (controller, request-response)

**Analog:** `crates/takoyaki-app/src/commands/backup.rs`

**Imports pattern** (lines 1-20):
```rust
use crate::atomic;
use crate::atomic::snapshot::SnapshotEngine;
use crate::db;
use crate::error::AppError;
use crate::AppState;
use serde::Serialize;
use specta::Type;
use std::path::{Path, PathBuf};
use tauri::ipc::Channel;
use tracing::{info, error};
use walkdir::WalkDir;
```

**Event enum pattern** (lines 26-48, `BackupEvent` → copy as `ManagementEvent`):
```rust
#[derive(Clone, Serialize, Type)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum ManagementEvent {
    Started { total_files: usize, destination: String },
    Progress { files_processed: usize, total_files: usize, current_file: String },
    Complete { files_processed: usize, total_bytes: u64, destination: String },
    Failed { reason: String },
}
```

**Dry-run manifest types pattern** (lines 51-80, copy `ChangeType` + `FileChangeEntry` + `FileChangeManifest`):
Extend `ChangeType` with a `Conflict` variant for bank copy hash-mismatch entries (D-08).

**Tauri command declaration pattern** (lines 283-291):
```rust
#[tauri::command]
#[specta::specta]
pub async fn duplicate_project(
    state: tauri::State<'_, AppState>,
    project_id: String,
    on_event: Channel<ManagementEvent>,
) -> Result<(), AppError> {
```

**DB lock / release before file I/O pattern** (lines 292-303):
```rust
// Pattern: acquire lock, extract values, drop lock BEFORE file I/O (T-03-04)
let (card_path, project_name) = {
    let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    let path = db::projects::get_card_path(&db.conn, &project_id)
        .map_err(|e| AppError::Database(e.to_string()))?;
    let name = PathBuf::from(&path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| project_id.clone());
    (path, name)
    // DB lock dropped here
};
```

**Pre-operation snapshot pattern** (lines 452-468 of `backup.rs`):
```rust
let snapshot_root = dirs::home_dir()
    .unwrap_or_else(|| PathBuf::from("."))
    .join("takoyaki")
    .join("snapshots");

let project_files: Vec<PathBuf> = WalkDir::new(&project_path)
    .follow_links(false)
    .min_depth(1)
    .into_iter()
    .filter_map(|e| e.ok())
    .filter(|e| e.file_type().is_file())
    .map(|e| e.path().to_path_buf())
    .collect();

let snapshot_engine = SnapshotEngine::new(snapshot_root.clone());
let project_file_refs: Vec<&Path> = project_files.iter().map(|p| p.as_path()).collect();
let _snapshot_result = snapshot_engine.snapshot_files(&project_file_refs, "pre-management-op")?;
```

**Error + Channel event pattern** (lines 388-398):
```rust
Err(e) => {
    error!("Operation failed: {}", e);
    let _ = on_event.send(ManagementEvent::Failed {
        reason: e.to_string(),
    });
    Err(e)
}
```

**Destination base dir helper** (lines 98-103, copy as `exports_base_dir()`):
```rust
fn exports_base_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("takoyaki")
        .join("exports")
}
```

---

### `crates/takoyaki-app/src/commands/mod.rs` (config)

**Analog:** `crates/takoyaki-app/src/commands/mod.rs` (lines 1-5)

**Pattern** — add one line:
```rust
pub mod backup;
pub mod device;
pub mod health;
pub mod management;   // ADD THIS
pub mod projects;
pub mod samples;
```

---

### `crates/takoyaki-app/src/lib.rs` (config)

**Analog:** `crates/takoyaki-app/src/lib.rs` (lines 31-46)

**collect_commands! registration pattern** (lines 31-46):
```rust
let builder = tauri_specta::Builder::<tauri::Wry>::new().commands(collect_commands![
    // existing commands ...
    commands::management::duplicate_project,
    commands::management::rename_project,
    commands::management::export_project,
    commands::management::copy_bank,
    commands::management::compute_management_dry_run,
]);
```

---

### `crates/takoyaki-app/src/management/mod.rs` (service, CRUD)

**Analog:** `crates/takoyaki-app/src/atomic/mod.rs` (lines 1-6)

**Module declaration pattern**:
```rust
pub mod bank_copy;
pub mod duplicate;
pub mod export;
pub mod project_work;
pub mod rename;
```

---

### `crates/takoyaki-app/src/management/project_work.rs` (utility, transform)

**Analog:** `crates/takoyaki-app/src/commands/samples.rs` (text parsing and path normalization pattern)

**Core text-parsing pattern** — extract slot paths from ASCII key=value:
```rust
// project.work is ASCII key=value with SLOT/PATH/TYPE entries.
// Use String::from_utf8_lossy + .lines() — same approach as any text parsing in this codebase.
// See RESEARCH.md Pattern 1 and Code Examples section for full slot path extractor.

pub fn extract_slot_paths(project_work_bytes: &[u8]) -> Vec<(SlotType, u8, String)> {
    let text = String::from_utf8_lossy(project_work_bytes);
    let mut results = Vec::new();
    let mut current_type: Option<SlotType> = None;
    let mut current_slot: Option<u8> = None;

    for line in text.lines() {
        let line = line.trim();
        if line == "TYPE=FLEX" { current_type = Some(SlotType::Flex); }
        else if line == "TYPE=STATIC" { current_type = Some(SlotType::Static); }
        else if let Some(rest) = line.strip_prefix("SLOT=") {
            current_slot = rest.parse().ok();
        } else if let Some(rest) = line.strip_prefix("PATH=") {
            if let (Some(t), Some(s)) = (current_type, current_slot) {
                results.push((t, s, rest.to_string()));
            }
        }
    }
    results
}
```

**Error handling pattern** — use existing `AppError` from `crates/takoyaki-app/src/error.rs`:
```rust
use crate::error::AppError;
// Return Result<_, AppError> throughout; map parse errors to AppError::Parse(...)
```

**Test structure pattern** (copy from `atomic/snapshot.rs` lines 133-254):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_slot_paths_flex_and_static() {
        let raw = b"TYPE=FLEX\nSLOT=001\nPATH=../AUDIO/kick.wav\n\
                    TYPE=STATIC\nSLOT=001\nPATH=../AUDIO/snare.wav\n";
        let slots = extract_slot_paths(raw);
        assert_eq!(slots.len(), 2);
    }
}
```

---

### `crates/takoyaki-app/src/management/duplicate.rs` (service, file-I/O)

**Analog:** `crates/takoyaki-app/src/commands/backup.rs` — `copy_project_tree()` function (lines 154-227)

**WalkDir copy tree pattern** (lines 177-226):
```rust
for entry in WalkDir::new(src).follow_links(false).min_depth(1) {
    let entry = entry.map_err(|e| AppError::Io(e.to_string()))?;
    let entry_path = entry.path();
    let relative = entry_path
        .strip_prefix(src)
        .map_err(|_| AppError::InvalidPath)?
        .to_string_lossy()
        .into_owned();
    let dest_entry = dest.join(&relative);

    if entry.file_type().is_dir() {
        std::fs::create_dir_all(&dest_entry)?;
    } else if entry.file_type().is_file() {
        if let Some(parent) = dest_entry.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(entry_path, &dest_entry)?;
    }
}
```

**Checksum (SHA-256) pattern** (lines 232-260, `verify_checksums` + `atomic::snapshot::sha256_hex`):
```rust
// Reuse existing sha256_hex() from atomic::snapshot — do NOT implement a new one
use crate::atomic::snapshot::sha256_hex;
let src_hash = sha256_hex(&src_audio_path)?;
let tgt_hash = sha256_hex(&tgt_audio_path)?;
```

**Name collision check** — check before creating dest dir:
```rust
// D-03/Pitfall 5: verify new_name does not already exist
let candidate = sets_dir.join(&new_name);
if candidate.exists() {
    return Err(AppError::Io(format!(
        "Duplicate target {} already exists — choose a different name", new_name
    )));
}
```

**atomic_write_batch for project.work + project.strd** (Pitfall 4):
```rust
// Atomic write BOTH .work and .strd (Pitfall 4 — OT reloads from .strd on reconnect)
use crate::atomic::atomic_write_batch;
let writes: Vec<(&Path, &[u8])> = vec![
    (project_work_dest.as_path(), rewritten_work.as_slice()),
    (project_strd_dest.as_path(), rewritten_strd.as_slice()),
];
atomic_write_batch(&writes)?;
```

---

### `crates/takoyaki-app/src/management/rename.rs` (service, CRUD)

**Analog:** `crates/takoyaki-app/src/db/projects.rs` (SQLite update pattern)

**DB UPDATE pattern** (lines 48-64, `upsert_project`):
```rust
use rusqlite::{params, Connection};
use crate::error::AppError;

pub fn update_project_name(conn: &Connection, project_id: &str, new_name: &str, new_card_path: &str) -> Result<(), AppError> {
    conn.execute(
        "UPDATE projects SET project_name = ?1, card_path = ?2 WHERE id = ?3",
        params![new_name, new_card_path, project_id],
    )?;
    Ok(())
}
```

**Filesystem rename pattern** (from RESEARCH.md Pattern 5 + same-volume constraint Pitfall 6):
```rust
// Both src and dest are under /SETS/ on the same FAT32 volume — std::fs::rename is safe.
// NEVER use std::fs::rename across filesystems (EXDEV).
std::fs::rename(&old_dir_path, &new_dir_path)?;
```

**OT name validation** — validate before snapshot/rename:
```rust
// OT directory name: A-Z, 0-9, underscore only, max 16 chars
pub fn validate_ot_name(name: &str) -> Result<(), AppError> {
    if name.len() > 16 {
        return Err(AppError::Parse("OT name exceeds 16 character limit".to_string()));
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(AppError::Parse("OT name must contain only A-Z, 0-9, underscore".to_string()));
    }
    Ok(())
}
```

---

### `crates/takoyaki-app/src/management/export.rs` (service, file-I/O)

**Analog:** `crates/takoyaki-app/src/commands/backup.rs` — overall async + Channel<Event> streaming structure

**Channel progress streaming pattern** (lines 325-329, 379-386):
```rust
let _ = on_event.send(ManagementEvent::Started {
    total_files,
    destination: zip_dest.to_string_lossy().into_owned(),
});
// ... per-file loop ...
let _ = on_event.send(ManagementEvent::Progress {
    files_processed,
    total_files,
    current_file: relative.clone(),
});
```

**zip crate usage pattern** (from RESEARCH.md Pattern 4 + Code Examples):
```rust
use zip::{ZipWriter, CompressionMethod, write::SimpleFileOptions};

let opts_audio = SimpleFileOptions::default()
    .compression_method(CompressionMethod::Stored);  // WAV/AIFF already compressed
let opts_text = SimpleFileOptions::default()
    .compression_method(CompressionMethod::Deflated);

let file = std::fs::File::create(&zip_dest)?;
let mut zip = ZipWriter::new(file);
// ... add files ...
zip.finish().map_err(|e| AppError::Io(e.to_string()))?;  // MUST call finish() (Pitfall 2)
```

**WalkDir traversal** (lines 177-226 of `backup.rs`):
Use `WalkDir::new(project_dir).follow_links(false)` — same pattern as backup, never follow_links.

---

### `crates/takoyaki-app/src/management/bank_copy.rs` (service, file-I/O)

**Analog:** `crates/takoyaki-app/src/commands/backup.rs` (file hashing + atomic_write_batch)

**Hash comparison pattern** (lines 232-260):
```rust
use crate::atomic::snapshot::sha256_hex;

let src_hash = sha256_hex(&resolved_src_audio)?;
let tgt_hash = sha256_hex(&resolved_tgt_audio)?;
if src_hash == tgt_hash {
    // D-07: same content — skip
} else {
    // D-08: hash mismatch — add Conflict entry to dry-run manifest
}
```

**resolve_ot_path usage** (from `health/mod.rs` line 212):
```rust
use crate::health::resolve_ot_path;

// All path resolution goes through resolve_ot_path — never raw string manipulation
let abs_path = resolve_ot_path(card_volume_path, &slot.path)
    .ok_or(AppError::InvalidPath)?;
```

**Snapshot ALL target project files before write** (Pitfall 3):
```rust
// Snapshot the entire target project directory (all .work + .strd files)
let target_files: Vec<PathBuf> = WalkDir::new(&target_project_dir)
    .follow_links(false)
    .min_depth(1)
    .into_iter()
    .filter_map(|e| e.ok())
    .filter(|e| e.file_type().is_file())
    .map(|e| e.path().to_path_buf())
    .collect();
let refs: Vec<&Path> = target_files.iter().map(|p| p.as_path()).collect();
snapshot_engine.snapshot_files(&refs, "pre-bank-copy")?;
```

---

### `src/lib/stores/management.ts` (store, event-driven)

**Analog:** `src/lib/stores/backup.ts` (lines 1-81) — exact structural copy with management-specific operation types

**Store shape pattern** (lines 1-81 of `backup.ts`):
```typescript
import { create } from "zustand";
import type { FileChangeManifest } from "@/lib/types";

export type ManagementStatus =
  | "idle"
  | "dry-running"
  | "in-progress"
  | "complete"
  | "failed";

export type ManagementOperation =
  | "duplicate"
  | "rename"
  | "export"
  | "bank-copy"
  | null;

interface ManagementState {
  status: ManagementStatus;
  operation: ManagementOperation;
  activeProjectId: string | null;
  activeProjectName: string | null;
  dryRunManifest: FileChangeManifest | null;
  // ... setters + reset
}

export const useManagementStore = create<ManagementState>((set) => ({
  // initial state mirrors backup store shape
}));
```

**Setter pattern** (lines 56-80 of `backup.ts`):
```typescript
setStatus: (status) => set({ status }),
startOperation: (projectId, projectName, operation) =>
  set({ activeProjectId: projectId, activeProjectName: projectName,
        operation, status: "dry-running", dryRunManifest: null }),
reset: () => set({ status: "idle", operation: null, activeProjectId: null,
                   activeProjectName: null, dryRunManifest: null }),
```

---

### `src/lib/tauri.ts` (utility, request-response)

**Analog:** `src/lib/tauri.ts` (lines 1-99) — append new management IPC wrappers in the same style

**Existing IPC wrapper pattern** (lines 63-98):
```typescript
// Phase 4: Management IPC wrappers — add after Phase 3 section
import type { ManagementEvent } from "./types";  // add to existing import block

export async function computeManagementDryRun(
  projectId: string,
  operation: string,
  bankIndex?: number,
  targetProjectId?: string,
): Promise<FileChangeManifest> {
  return invoke("compute_management_dry_run", {
    projectId,
    operation,
    bankIndex: bankIndex ?? null,
    targetProjectId: targetProjectId ?? null,
  });
}

export async function duplicateProject(
  projectId: string,
  onEvent: Channel<ManagementEvent>
): Promise<void> {
  return invoke("duplicate_project", { projectId, onEvent });
}
```

**Channel import** — already present in `src/lib/tauri.ts` line 1:
```typescript
import { invoke, Channel } from "@tauri-apps/api/core";
```

---

### `src/components/project-detail/MetadataHeader.tsx` (component, modified)

**Analog:** `src/components/project-detail/MetadataHeader.tsx` (lines 1-61) — add buttons to existing right cluster

**Existing Button pattern** (lines 47-57):
```tsx
{onBackUp && (
  <Button
    variant="default"
    size="sm"
    className="font-mono text-xs h-8 gap-1"
    onClick={onBackUp}
  >
    <Archive className="h-3.5 w-3.5" />
    Back Up
  </Button>
)}
```

**New buttons follow same pattern** (from RESEARCH.md Code Examples "MetadataHeader Toolbar Additions"):
```tsx
// Add to props interface:
interface MetadataHeaderProps {
  project: ProjectDetail;
  onBackUp?: () => void;
  onRename?: () => void;       // ADD
  onDuplicate?: () => void;    // ADD
  onExport?: () => void;       // ADD
}

// In the right cluster div, after Back Up button:
import { Archive, Pencil, Copy, PackageOpen } from "lucide-react";

{onRename && (
  <Button variant="ghost" size="sm" className="font-mono text-xs h-8 gap-1"
    onClick={onRename} aria-label="Rename project">
    <Pencil className="h-3.5 w-3.5" />
    Rename
  </Button>
)}
```

**Inline rename edit** — when rename mode active, replace the `<span>` project name (line 34) with an `<input>`:
```tsx
{isRenaming ? (
  <input
    className="font-mono text-2xl font-semibold text-foreground bg-transparent border-b border-[hsl(38,85%,55%)] outline-none w-48"
    value={renameValue}
    onChange={(e) => setRenameValue(e.target.value.toUpperCase().replace(/[^A-Z0-9_]/g, ""))}
    onKeyDown={(e) => { if (e.key === "Enter") onRenameConfirm?.(renameValue); if (e.key === "Escape") setIsRenaming(false); }}
    maxLength={16}
    autoFocus
  />
) : (
  <span className="font-mono text-2xl font-semibold text-foreground">
    {project.project_name}
  </span>
)}
```

---

### `src/components/project-detail/BankGridCell.tsx` (component, modified)

**Analog:** `src/components/project-detail/BankGridCell.tsx` (lines 1-46) — wrap existing button with ContextMenu

**Existing button pattern** (lines 16-46) is wrapped inside `<ContextMenu>` + `<ContextMenuTrigger asChild>`:
```tsx
// Add to imports:
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { ArrowRightFromLine } from "lucide-react";

// Add to props interface:
interface BankGridCellProps {
  bankIndex: number;
  populated: boolean;
  selected: boolean;
  onClick?: () => void;
  onCopyToProject?: () => void;  // ADD
}

// Wrap existing return:
return (
  <ContextMenu>
    <ContextMenuTrigger asChild>
      <button ...existing button props...>
        {/* existing content unchanged */}
      </button>
    </ContextMenuTrigger>
    {populated && (
      <ContextMenuContent className="font-mono text-xs">
        <ContextMenuItem
          onSelect={() => onCopyToProject?.()}
          className="flex items-center gap-2"
        >
          <ArrowRightFromLine className="h-3.5 w-3.5" />
          Copy to project...
        </ContextMenuItem>
      </ContextMenuContent>
    )}
  </ContextMenu>
);
```

---

### `src/components/management/BankCopyPickerDialog.tsx` (component, request-response)

**Analog:** `src/components/volume-confirm-dialog.tsx` (Dialog + two-step picker) and `src/components/backups/DryRunModal.tsx` (Dialog + ScrollArea pattern)

**Dialog shell pattern** (from `volume-confirm-dialog.tsx` lines 1-57):
```tsx
"use client";

import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";

interface BankCopyPickerDialogProps {
  open: boolean;
  step: 1 | 2;
  projects: ProjectSummary[];
  selectedProjectId: string | null;
  bankSlots: { index: number; populated: boolean }[];
  selectedBankSlot: number | null;
  onSelectProject: (id: string) => void;
  onSelectSlot: (index: number) => void;
  onConfirm: () => void;
  onCancel: () => void;
}
```

**Step 1 project list** — use `ScrollArea` + project rows (same `ScrollArea` usage as `DryRunModal.tsx` line 110):
```tsx
<ScrollArea className="max-h-72">
  {projects.map((p) => (
    <button key={p.id}
      className={["w-full text-left px-4 py-2 font-mono text-xs hover:bg-[hsl(30,8%,20%)]",
        selectedProjectId === p.id ? "bg-[hsl(30,8%,20%)] text-[hsl(38,85%,55%)]" : ""].join(" ")}
      onClick={() => onSelectProject(p.id)}
    >
      {p.project_name}
    </button>
  ))}
</ScrollArea>
```

**Step 2 bank slot 4×4 grid** — mirrors `BankGridCell` layout from `BanksTab.tsx`:
```tsx
// 4x4 grid showing populated vs empty, same visual language as BankGridCell
<div className="grid grid-cols-4 gap-2">
  {bankSlots.map(({ index, populated }) => (
    <button key={index}
      className={["flex w-12 h-12 flex-col items-center justify-center rounded border font-mono text-xs",
        populated ? "border-[hsl(38,85%,55%)]" : "border-border opacity-40",
        selectedBankSlot === index ? "bg-[hsl(30,8%,20%)]/30" : ""].join(" ")}
      onClick={() => onSelectSlot(index)}
    >
      <span className={["h-2 w-2 rounded-full", populated ? "bg-foreground" : "border border-muted-foreground"].join(" ")} />
      <span className="mt-1 tabular-nums">{String(index + 1).padStart(2, "0")}</span>
    </button>
  ))}
</div>
```

**Footer confirm/cancel pattern** (from `DryRunModal.tsx` lines 118-147):
```tsx
<DialogFooter className="flex justify-end gap-2 pt-4 border-t border-border bg-transparent border-none -mx-0 -mb-0 rounded-none p-0">
  <Button variant="ghost" className="font-mono text-xs" onClick={onCancel}>
    Cancel
  </Button>
  <Button variant="default" className="font-mono text-xs" onClick={onConfirm}
    disabled={step === 1 ? !selectedProjectId : selectedBankSlot === null}>
    {step === 1 ? "Next" : "Copy Bank"}
  </Button>
</DialogFooter>
```

---

## Shared Patterns

### Atomic Write (all Rust service files)
**Source:** `crates/takoyaki-app/src/atomic/mod.rs` lines 64-103
**Apply to:** `duplicate.rs`, `rename.rs` (project.work/strd updates), `bank_copy.rs`
```rust
use crate::atomic::atomic_write_batch;
// Stage all writes first, then commit as a batch
let writes: Vec<(&Path, &[u8])> = vec![...];
atomic::atomic_write_batch(&writes)?;
```

### Pre-Operation Snapshot (all Rust service files that write)
**Source:** `crates/takoyaki-app/src/commands/backup.rs` lines 452-468
**Apply to:** `duplicate.rs`, `rename.rs`, `bank_copy.rs` — every write operation
```rust
let snapshot_engine = SnapshotEngine::new(snapshot_root);
let project_file_refs: Vec<&Path> = project_files.iter().map(|p| p.as_path()).collect();
snapshot_engine.snapshot_files(&project_file_refs, "pre-<operation-name>")?;
```

### AppError Error Handling (all Rust files)
**Source:** `crates/takoyaki-app/src/error.rs` lines 1-47
**Apply to:** All new Rust files
```rust
use crate::error::AppError;
// Map IO errors: std::fs::* operations auto-convert via From<std::io::Error>
// Map parse failures: AppError::Parse(msg.to_string())
// Map DB errors: .map_err(|e| AppError::Database(e.to_string()))
// Map lock failures: .map_err(|e| AppError::Lock(e.to_string()))
```

### DB Lock Release Before File I/O (all Tauri commands)
**Source:** `crates/takoyaki-app/src/commands/backup.rs` lines 292-303
**Apply to:** `commands/management.rs` — every command that does both DB lookup and file I/O
```rust
// Pattern: inner block drops the MutexGuard before file I/O begins (T-03-04)
let card_path = {
    let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db::projects::get_card_path(&db.conn, &project_id)
        .map_err(|e| AppError::Database(e.to_string()))?
    // MutexGuard dropped here
};
```

### resolve_ot_path for All Path Resolution (all Rust files that touch OT paths)
**Source:** `crates/takoyaki-app/src/health/mod.rs` line 212
**Apply to:** `project_work.rs`, `duplicate.rs`, `export.rs`, `bank_copy.rs`
```rust
use crate::health::resolve_ot_path;
// NEVER resolve OT paths manually — always use this function
let abs_path = resolve_ot_path(card_volume_path, &raw_ot_path)
    .ok_or(AppError::InvalidPath)?;
```

### Font and Styling Convention (all React components)
**Source:** `src/components/project-detail/MetadataHeader.tsx` + `src/components/backups/DryRunModal.tsx`
**Apply to:** `BankCopyPickerDialog.tsx`, inline rename input in `MetadataHeader.tsx`
- All text: `font-mono text-xs` or `font-mono text-base font-semibold` for headings
- Accent color: `hsl(38,85%,55%)` (amber/orange)
- Borders: `border-[hsl(30,8%,26%)]`
- Interactive hover: `hover:bg-[hsl(30,8%,20%)]`

### Zustand Store Shape (frontend store)
**Source:** `src/lib/stores/backup.ts` lines 1-81
**Apply to:** `src/lib/stores/management.ts`
- State fields + typed setters + `reset()` function
- `create<StateInterface>((set) => ({ ... }))` pattern
- No async in store — async work stays in component handlers that call tauri.ts wrappers

---

## No Analog Found

All files have close analogs. No items in this section.

---

## Metadata

**Analog search scope:** `crates/takoyaki-app/src/`, `src/components/`, `src/lib/`
**Files scanned:** 14 source files read directly
**Pattern extraction date:** 2026-05-01
