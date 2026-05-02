# Phase 5: Sample Assignment and Wallflower - Pattern Map

**Mapped:** 2026-05-02
**Files analyzed:** 10 new/modified files
**Analogs found:** 10 / 10

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/takoyaki-app/src/commands/samples.rs` | command/service | request-response + file-I/O | `crates/takoyaki-app/src/commands/management.rs` | exact |
| `crates/takoyaki-app/src/commands/wallflower.rs` | command/service | request-response + CRUD | `crates/takoyaki-app/src/commands/management.rs` + `src/db/wallflower.rs` | role-match |
| `migrations/V3__wallflower_settings.sql` | migration | batch | `migrations/V1__initial_schema.sql` | exact |
| `crates/takoyaki-app/src/db/mod.rs` | config | batch | `crates/takoyaki-app/src/db/mod.rs` (extend) | exact |
| `src/components/project-detail/SlotRow.tsx` | component | request-response | `src/components/project-detail/SlotRow.tsx` (extend) | exact |
| `src/components/project-detail/SamplesTab.tsx` | component | request-response | `src/components/project-detail/SamplesTab.tsx` (extend) | exact |
| `src/components/project-detail/WallflowerPanel.tsx` | component | CRUD + event-driven | `src/components/management/BankCopyPickerDialog.tsx` | role-match |
| `src/components/project-detail/WallflowerSampleRow.tsx` | component | request-response | `src/components/project-detail/SlotRow.tsx` | role-match |
| `src/components/project-detail/SlotPickerDialog.tsx` | component | request-response | `src/components/management/BankCopyPickerDialog.tsx` | exact |
| `src/lib/stores/samples.ts` | store | event-driven | `src/lib/stores/management.ts` | exact |

---

## Pattern Assignments

### `crates/takoyaki-app/src/commands/samples.rs` (extend — add assign_sample + compute_sample_dry_run)

**Analog:** `crates/takoyaki-app/src/commands/management.rs`

**Imports pattern** (management.rs lines 1-23):
```rust
use crate::atomic::snapshot::SnapshotEngine;
use crate::commands::backup::{ChangeType, ConflictDetail, FileChangeEntry, FileChangeManifest};
use crate::db;
use crate::error::AppError;
use crate::management;
use crate::AppState;
use serde::Serialize;
use specta::Type;
use std::path::{Path, PathBuf};
use tracing::{error, info};
```

**Response type pattern** (samples.rs lines 14-33):
```rust
#[derive(Debug, serde::Serialize, Type, Clone)]
pub struct SampleDryRunResult {
    pub manifest: FileChangeManifest,
    pub hard_block: Option<String>,     // e.g. "Non-WAV/AIFF format — OT cannot load MP3"
    pub soft_warnings: Vec<String>,     // e.g. "48kHz sample rate — OT prefers 44.1kHz"
}

#[derive(Debug, serde::Serialize, Type, Clone)]
pub struct AssignSampleResult {
    pub files_written: u8,
    pub slot_type: String,
    pub slot_index: u8,
    pub filename: String,
}
```

**Command skeleton pattern** (management.rs lines 119-127, 391-401):
```rust
#[tauri::command]
#[specta::specta]
pub async fn compute_sample_dry_run(
    state: tauri::State<'_, crate::AppState>,
    project_id: String,
    slot_type: String,      // "flex" | "static"
    slot_index: u8,
    file_path: String,      // absolute path from native file picker
) -> Result<SampleDryRunResult, AppError> {
    // 1. DB lookup with lock release before I/O (T-03-04)
    let card_path = {
        let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        db::projects::get_card_path(&db.conn, &project_id)
            .map_err(|e| AppError::Database(e.to_string()))?
        // DB lock dropped
    };
    // 2. validate format via health::read_audio_spec + check_format_compatibility
    // 3. validate slot_type enum, slot_index 0..=127
    // 4. canonicalize file_path (T-02-05 pattern from health::resolve_ot_path)
    // 5. build FileChangeManifest (project.work + project.strd entries)
    // 6. return SampleDryRunResult { manifest, hard_block, soft_warnings }
}

#[tauri::command]
#[specta::specta]
pub async fn assign_sample(
    state: tauri::State<'_, crate::AppState>,
    project_id: String,
    slot_type: String,
    slot_index: u8,
    file_path: String,
    from_wallflower: bool,  // triggers /AUDIO/ copy step
) -> Result<AssignSampleResult, AppError> {
    // 1. DB lookup (T-03-04)
    // 2. SnapshotEngine::new(snapshot_root()).snapshot_files(...) (SAFE-03)
    // 3. If from_wallflower: std::fs::copy(src, ot_audio_dir/filename) — BEFORE project.work
    // 4. management::project_work::rewrite_slot_path(raw, slot_type, slot_number, new_path)
    // 5. atomic::atomic_write_batch(&[(project_work_path, &new_bytes), ...])
    // 6. return AssignSampleResult
}
```

**Snapshot pattern** (management.rs lines 94-106):
```rust
fn snapshot_project(project_dir: &Path, label: &str) -> Result<(), AppError> {
    let files = collect_files(project_dir);
    let file_refs: Vec<&Path> = files.iter().map(|p| p.as_path()).collect();
    let engine = SnapshotEngine::new(snapshot_root());
    let result = engine.snapshot_files(&file_refs, label)?;
    info!(
        "Pre-{} snapshot created: {} files at {}",
        label, result.file_count, result.snapshot_dir.display()
    );
    Ok(())
}
```

**project.work rewrite pattern** (`crates/takoyaki-app/src/management/project_work.rs` lines 77-142):
```rust
// Rewrite a single slot's PATH= value without touching any other content
pub fn rewrite_slot_path(
    raw: &[u8],
    slot_type: SlotType,     // SlotType::Flex | SlotType::Static
    slot_number: u8,         // 1-indexed per OT convention (SLOT=001)
    new_path: &str,          // e.g. "../AUDIO/new_kick.wav"
) -> Vec<u8>
// Uses TYPE=FLEX/STATIC discriminator lines + SLOT=NNN + PATH=... structure
// Preserves CRLF vs LF line ending style from original file
// If slot not found, returns raw unchanged (safe no-op)
```

**Error handling pattern** (management.rs lines 381-388):
```rust
Err(e) => {
    error!("Assign sample failed: {}", e);
    Err(e)
}
```

**Test pattern** (samples.rs lines 177-215):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assign_sample_updates_project_work() {
        // Use fixture bytes from management/project_work.rs test pattern
        // fixture format: TYPE=FLEX\nSLOT=001\nPATH=../AUDIO/old.wav\n...
        let raw = b"TYPE=FLEX\nSLOT=001\nPATH=../AUDIO/old.wav\n";
        let result = management::project_work::rewrite_slot_path(
            raw, SlotType::Flex, 1, "../AUDIO/new.wav"
        );
        assert!(String::from_utf8(result).unwrap().contains("PATH=../AUDIO/new.wav"));
    }
}
```

---

### `crates/takoyaki-app/src/commands/wallflower.rs` (new)

**Analog:** `crates/takoyaki-app/src/commands/management.rs` (command structure) + `crates/takoyaki-app/src/db/wallflower.rs` (DB open pattern)

**Imports pattern:**
```rust
use crate::db::wallflower::open_wallflower_db;
use crate::error::AppError;
use crate::AppState;
use serde::Serialize;
use specta::Type;
use std::path::PathBuf;
use tracing::info;
```

**Response types:**
```rust
#[derive(Debug, serde::Serialize, Type, Clone)]
pub struct WallflowerStatus {
    pub connected: bool,
    pub db_path: Option<String>,
    pub sample_count: Option<u32>,
}

#[derive(Debug, serde::Serialize, Type, Clone)]
pub struct WallflowerSample {
    pub id: i64,
    pub filename: String,
    pub file_path: String,
    pub sample_rate: Option<u32>,
    pub bit_depth: Option<u16>,
    pub bpm: Option<f64>,
    pub key_name: Option<String>,
    pub scale: Option<String>,
    pub tags: Vec<String>,
}
```

**DB open pattern** (db/wallflower.rs lines 17-26):
```rust
pub fn open_wallflower_db(path: &Path) -> Result<Connection, AppError> {
    if !path.exists() {
        return Err(AppError::Io(format!(
            "Wallflower database not found at: {}",
            path.display()
        )));
    }
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    Ok(conn)
}
```

**Auto-discovery pattern** (new — no direct analog, based on db/mod.rs `default_path()` pattern lines 30-35):
```rust
fn discover_wallflower_db() -> Option<PathBuf> {
    // Priority 1: user-configured path from Takoyaki settings table
    // Priority 2: ~/Library/Application Support/wallflower/wallflower.db (dirs::data_dir())
    // Priority 3: ~/wallflower/wallflower.db (from Wallflower watch_folder default)
    let candidates = vec![
        dirs::data_dir()
            .map(|d| d.join("wallflower").join("wallflower.db")),
        dirs::home_dir()
            .map(|h| h.join("wallflower").join("wallflower.db")),
    ];
    candidates.into_iter()
        .flatten()
        .find(|p| p.exists())
}
```

**Command pattern** (management.rs lines 119-128):
```rust
#[tauri::command]
#[specta::specta]
pub async fn get_wallflower_status(
    state: tauri::State<'_, AppState>,
) -> Result<WallflowerStatus, AppError>

#[tauri::command]
#[specta::specta]
pub async fn search_wallflower_samples(
    state: tauri::State<'_, AppState>,
    query: String,
) -> Result<Vec<WallflowerSample>, AppError>
```

**SQL query pattern** (from RESEARCH.md Pattern 4, verified against Wallflower schema):
```rust
// rusqlite params![] — never string interpolation (T-02-01 pattern)
conn.prepare(sql)?
    .query_map(rusqlite::params![query, 200i64], |row| {
        Ok(WallflowerSample {
            id: row.get(0)?,
            filename: row.get(1)?,
            // ...
            tags: row.get::<_, Option<String>>(8)?
                .map(|s| s.split(',').map(String::from).collect())
                .unwrap_or_default(),
        })
    })?
    .collect()
```

**Test pattern** (db/wallflower.rs lines 33-61):
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_wallflower_status_not_found() {
        // open_wallflower_db with nonexistent path returns Err
        let result = open_wallflower_db(Path::new("/nonexistent/wallflower.db"));
        assert!(result.is_err());
    }
}
```

---

### `migrations/V3__wallflower_settings.sql` (new)

**Analog:** `migrations/V1__initial_schema.sql`

**Migration file pattern** (V1 lines 1-36):
```sql
-- V3__wallflower_settings.sql
-- Add wallflower_db_path setting to Takoyaki DB

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

-- Default: no configured path (auto-discovery used)
INSERT OR IGNORE INTO settings (key, value) VALUES ('wallflower_db_path', '');
```

**Migration registration pattern** (db/mod.rs lines 9-60 — extend `initialize()`):
```rust
// In db/mod.rs: add new constant and increment migration check
const MIGRATION_V3: &str = include_str!("../../../../migrations/V3__wallflower_settings.sql");

// In initialize():
if current_version < 3 {
    info!("Running V3 migration: wallflower settings");
    conn.execute_batch(MIGRATION_V3)?;
    conn.execute_batch("PRAGMA user_version = 3;")?;
}
```

---

### `src/components/project-detail/SlotRow.tsx` (extend — add assign button)

**Analog:** `src/components/project-detail/SlotRow.tsx` (self — extend existing)

**Key constraint — stopPropagation pattern** (SlotRow.tsx lines 127-188):

The entire row is wrapped in `<CollapsibleTrigger>`. The assign button must use `e.stopPropagation()` to prevent triggering expand. The button must be rendered INSIDE the trigger but intercept the click:

```tsx
// Inside CollapsibleTrigger, after Status icon column:
<span className="w-10 shrink-0 flex items-center justify-center">
  {/* Assign button — stopPropagation isolates from CollapsibleTrigger */}
  <button
    type="button"
    onClick={(e) => {
      e.stopPropagation();
      onAssign(slot.slot_index, slotType);
    }}
    className="h-6 w-6 flex items-center justify-center rounded hover:bg-[hsl(30,8%,26%)] text-muted-foreground hover:text-foreground"
    aria-label={`Assign sample to ${slotType} slot ${slot.slot_index + 1}`}
  >
    <Upload size={12} />
  </button>
</span>
```

**Inline error pattern** (new — no direct analog — below CollapsibleContent):
```tsx
{/* Inline slot type error — shown when hard_block present for this slot */}
{assignError && (
  <div className="bg-[hsl(0,20%,12%)] border-b border-[hsl(0,50%,30%)] px-4 py-2">
    <span className="font-mono text-xs text-[hsl(0,68%,48%)]">{assignError}</span>
    {assignErrorRedirect && (
      <button
        type="button"
        className="ml-2 font-mono text-xs text-[hsl(38,85%,55%)] underline"
        onClick={assignErrorRedirect.onRedirect}
      >
        {assignErrorRedirect.label}
      </button>
    )}
  </div>
)}
```

**Updated props interface:**
```tsx
interface SlotRowProps {
  slot: SampleSlot;
  slotType: "flex" | "static";
  crossRefs?: string[];
  healthIssues?: HealthIssue[];
  onAssign?: (slotIndex: number, slotType: "flex" | "static") => void; // NEW
  assignError?: string | null;                                          // NEW
  assignErrorRedirect?: { label: string; onRedirect: () => void } | null; // NEW
}
```

---

### `src/components/project-detail/SamplesTab.tsx` (extend — add WallflowerPanel, wire assign flow)

**Analog:** `src/components/project-detail/SamplesTab.tsx` (self — extend existing)

**Zustand store usage pattern** (SamplesTab does not yet use a store — management.ts pattern applies):
```tsx
// Add to SamplesTab:
import { useSamplesStore } from "@/lib/stores/samples";

export function SamplesTab({ projectId }: SamplesTabProps) {
  const { wallflowerConnected, assignInProgress, setAssignInProgress } = useSamplesStore();
  // ...
  // Existing useQuery for samples — invalidate after assign success:
  const queryClient = useQueryClient();
  // After successful assign:
  queryClient.invalidateQueries({ queryKey: ["samples", projectId] });
}
```

**WallflowerPanel conditional render** (pattern from D-07 — hidden when unavailable):
```tsx
{/* Wallflower panel — rendered only when DB connected (D-07) */}
{wallflowerConnected && (
  <WallflowerPanel
    projectId={projectId}
    onPushToSlot={handlePushToSlot}
  />
)}
```

**DryRunModal wiring pattern** (backup.ts store pattern lines 46-70, adapted):
```tsx
const [dryRunManifest, setDryRunManifest] = useState<FileChangeManifest | null>(null);
const [dryRunOpen, setDryRunOpen] = useState(false);
const [pendingAssign, setPendingAssign] = useState<AssignParams | null>(null);

async function handleAssign(slotIndex: number, slotType: "flex" | "static") {
  const filePath = await open({ multiple: false, filters: [{ name: 'Audio', extensions: ['wav', 'aif', 'aiff'] }] });
  if (!filePath) return;
  const result = await computeSampleDryRun(projectId, slotType, slotIndex, filePath);
  if (result.hardBlock) { /* show inline error on slot row */ return; }
  setDryRunManifest(result.manifest);
  setPendingAssign({ slotIndex, slotType, filePath });
  setDryRunOpen(true);
}

async function handleApply() {
  if (!pendingAssign) return;
  setDryRunOpen(false);
  const result = await assignSample(projectId, pendingAssign.slotType, pendingAssign.slotIndex, pendingAssign.filePath, false);
  // show InlineSuccessBanner via store
  queryClient.invalidateQueries({ queryKey: ["samples", projectId] });
}
```

---

### `src/components/project-detail/WallflowerPanel.tsx` (new)

**Analog:** `src/components/management/BankCopyPickerDialog.tsx` (collapsible panel with search and list)

**Imports pattern** (BankCopyPickerDialog.tsx lines 1-15):
```tsx
"use client";

import { useState, useEffect } from "react";
import { useQuery } from "@tanstack/react-query";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Input } from "@/components/ui/input";
import { searchWallflowerSamples } from "@/lib/tauri";
import type { WallflowerSample } from "@/lib/types";
```

**Collapsible panel pattern** (adapted from BankCopyPickerDialog Step 1 structure, lines 91-148):
```tsx
// Collapsible below Flex/Static slot lists — default expanded (D-09)
const [isExpanded, setIsExpanded] = useState(true);
const [query, setQuery] = useState("");
const [debouncedQuery, setDebouncedQuery] = useState("");

// Debounce 300ms (RESEARCH.md Claude's Discretion)
useEffect(() => {
  const t = setTimeout(() => setDebouncedQuery(query), 300);
  return () => clearTimeout(t);
}, [query]);

const { data: samples } = useQuery({
  queryKey: ["wallflower-search", debouncedQuery],
  queryFn: () => searchWallflowerSamples(debouncedQuery),
  enabled: true,
});
```

**Result count indicator pattern** (RESEARCH.md Pitfall 6):
```tsx
{/* Show count when truncated at 200 */}
{samples && samples.length === 200 && (
  <span className="font-mono text-xs text-muted-foreground px-4 py-1">
    Showing 200 results — refine your search
  </span>
)}
```

**Collapse toggle styling** (matching SlotRow hover pattern):
```tsx
<button
  type="button"
  onClick={() => setIsExpanded((p) => !p)}
  className="flex h-9 w-full items-center gap-2 px-4 border-b border-[hsl(30,8%,26%)] hover:bg-[hsl(30,8%,20%)] font-mono text-sm font-semibold text-foreground"
>
  <ChevronDown
    size={14}
    className={`transition-transform duration-200 ${isExpanded ? "" : "-rotate-90"}`}
  />
  WALLFLOWER LIBRARY
</button>
```

---

### `src/components/project-detail/WallflowerSampleRow.tsx` (new)

**Analog:** `src/components/project-detail/SlotRow.tsx` (same table-row structure, different content)

**Row layout pattern** (SlotRow.tsx lines 138-187 — adapt column layout):
```tsx
// Compact row: filename | key | BPM | tags | push button
// Uses same h-9 row height, border-b, hover pattern as SlotRow
<div className="flex h-9 w-full items-center gap-0 border-b border-[hsl(30,8%,26%)] hover:bg-[hsl(30,8%,20%)]">
  {/* Filename — flex-1 */}
  <span className="min-w-0 flex-1 px-3 font-mono text-xs truncate">
    {sample.filename}
  </span>
  {/* Key — w-16 */}
  <span className="w-16 shrink-0 px-2 font-mono text-xs text-muted-foreground tabular-nums">
    {sample.keyName ?? "--"}
  </span>
  {/* BPM — w-16 */}
  <span className="w-16 shrink-0 px-2 font-mono text-xs text-muted-foreground tabular-nums">
    {sample.bpm ? Math.round(sample.bpm) : "--"}
  </span>
  {/* Tags — flex-1 max-w-[120px] */}
  <div className="flex gap-1 px-2 max-w-[120px] overflow-hidden">
    {sample.tags.slice(0, 3).map((tag) => (
      <Badge key={tag} variant="secondary" className="font-mono text-[10px] px-1 py-0">
        {tag}
      </Badge>
    ))}
  </div>
  {/* Push button */}
  <button
    type="button"
    onClick={() => onPush(sample)}
    className="w-10 shrink-0 flex items-center justify-center h-full hover:bg-[hsl(30,8%,26%)] text-muted-foreground hover:text-foreground"
    aria-label={`Push ${sample.filename} to slot`}
  >
    <ArrowUp size={12} />
  </button>
</div>
```

---

### `src/components/project-detail/SlotPickerDialog.tsx` (new)

**Analog:** `src/components/management/BankCopyPickerDialog.tsx` (exact structural match — picker dialog with slot list)

**Full dialog structure pattern** (BankCopyPickerDialog.tsx lines 1-219):

```tsx
"use client";

import { useState } from "react";
import {
  Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { SampleSlotResponse } from "@/lib/types";

interface SlotPickerDialogProps {
  open: boolean;
  sampleFilename: string;
  slots: SampleSlotResponse;         // from react-query cache ["samples", projectId]
  onConfirm: (slotType: "flex" | "static", slotIndex: number) => void;
  onCancel: () => void;
}
```

**Step pattern** (BankCopyPickerDialog.tsx lines 35-61 — adapt for Flex/Static toggle + slot dropdown):
```tsx
// Single-step dialog (no Next/Back needed — simpler than bank copy)
const [slotTypeTab, setSlotTypeTab] = useState<"flex" | "static">("flex");
const [selectedSlot, setSelectedSlot] = useState<number | null>(null);

// Reset on open
useEffect(() => {
  if (open) {
    setSlotTypeTab("flex");
    setSelectedSlot(null);
  }
}, [open]);
```

**Occupied slot indicator pattern** (BankCopyPickerDialog.tsx lines 165-198):
```tsx
// Show occupied chip on occupied slots — amber chip per UI-SPEC
{slot.occupied && (
  <span className="ml-auto font-mono text-[10px] text-[hsl(38,85%,55%)] border border-[hsl(38,85%,35%)] rounded px-1">
    occupied
  </span>
)}
```

**Dialog footer pattern** (BankCopyPickerDialog.tsx lines 200-215):
```tsx
<DialogFooter className="flex justify-end gap-2 pt-4 border-t border-border">
  <Button variant="ghost" className="font-mono text-xs" onClick={onCancel}>
    Cancel
  </Button>
  <Button
    variant="default"
    className="font-mono text-xs"
    onClick={handleConfirm}
    disabled={selectedSlot === null}
  >
    Assign to Slot
  </Button>
</DialogFooter>
```

---

### `src/lib/stores/samples.ts` (new)

**Analog:** `src/lib/stores/management.ts` (exact pattern — zustand store for operation state)

**Full store pattern** (management.ts lines 1-67):
```typescript
import { create } from "zustand";
import type { FileChangeManifest } from "@/lib/types";

export type AssignStatus =
  | "idle"
  | "picking-file"       // native file picker open
  | "dry-running"        // awaiting compute_sample_dry_run
  | "confirming"         // DryRunModal showing
  | "assigning"          // awaiting assign_sample
  | "complete"
  | "failed";

interface SamplesState {
  assignStatus: AssignStatus;
  wallflowerConnected: boolean;
  wallflowerDbPath: string | null;
  dryRunManifest: FileChangeManifest | null;
  hardBlock: string | null;
  softWarnings: string[];
  successMessage: string | null;
  pendingSlotType: "flex" | "static" | null;
  pendingSlotIndex: number | null;

  setAssignStatus: (status: AssignStatus) => void;
  setWallflowerConnected: (connected: boolean, dbPath?: string) => void;
  setDryRunManifest: (manifest: FileChangeManifest | null) => void;
  setHardBlock: (message: string | null) => void;
  setSuccessMessage: (message: string) => void;
  reset: () => void;
}

export const useSamplesStore = create<SamplesState>((set) => ({
  assignStatus: "idle",
  wallflowerConnected: false,
  wallflowerDbPath: null,
  dryRunManifest: null,
  hardBlock: null,
  softWarnings: [],
  successMessage: null,
  pendingSlotType: null,
  pendingSlotIndex: null,

  setAssignStatus: (status) => set({ assignStatus: status }),
  setWallflowerConnected: (connected, dbPath) =>
    set({ wallflowerConnected: connected, wallflowerDbPath: dbPath ?? null }),
  setDryRunManifest: (manifest) => set({ dryRunManifest: manifest }),
  setHardBlock: (message) => set({ hardBlock: message }),
  setSuccessMessage: (message) =>
    set({ successMessage: message, assignStatus: "complete" }),
  reset: () => set({
    assignStatus: "idle",
    dryRunManifest: null,
    hardBlock: null,
    softWarnings: [],
    successMessage: null,
    pendingSlotType: null,
    pendingSlotIndex: null,
  }),
}));
```

---

## Shared Patterns

### Tauri Command Registration
**Source:** `crates/takoyaki-app/src/lib.rs` lines 32-52
**Apply to:** `commands/samples.rs` new commands + `commands/wallflower.rs` all commands
```rust
// In lib.rs collect_commands![] — append new commands:
commands::samples::compute_sample_dry_run,
commands::samples::assign_sample,
commands::wallflower::get_wallflower_status,
commands::wallflower::search_wallflower_samples,
// Also add: pub mod wallflower; to commands/mod.rs
```

### IPC Wrapper Pattern
**Source:** `src/lib/tauri.ts` lines 103-157
**Apply to:** `src/lib/tauri.ts` (extend with Phase 5 wrappers)
```typescript
// Phase 5: Sample assignment IPC wrappers
export async function computeSampleDryRun(
  projectId: string,
  slotType: "flex" | "static",
  slotIndex: number,
  filePath: string,
): Promise<SampleDryRunResult> {
  return invoke("compute_sample_dry_run", { projectId, slotType, slotIndex, filePath });
}

export async function assignSample(
  projectId: string,
  slotType: "flex" | "static",
  slotIndex: number,
  filePath: string,
  fromWallflower: boolean,
): Promise<AssignSampleResult> {
  return invoke("assign_sample", { projectId, slotType, slotIndex, filePath, fromWallflower });
}

export async function getWallflowerStatus(): Promise<WallflowerStatus> {
  return invoke("get_wallflower_status");
}

export async function searchWallflowerSamples(query: string): Promise<WallflowerSample[]> {
  return invoke("search_wallflower_samples", { query });
}
```

### DB Lock Release Before I/O (T-03-04)
**Source:** `crates/takoyaki-app/src/commands/management.rs` lines 130-139
**Apply to:** All new Rust commands that touch DB then filesystem
```rust
let card_path = {
    let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db::projects::get_card_path(&db.conn, &project_id)
        .map_err(|e| AppError::Database(e.to_string()))?
    // DB lock dropped here — all file I/O happens outside lock
};
```

### Error Type
**Source:** `crates/takoyaki-app/src/error.rs` lines 1-47
**Apply to:** All new Rust command return types — use `Result<T, AppError>` uniformly
```rust
// AppError variants in use for Phase 5:
// AppError::Io(String)       — file copy failures, audio read failures
// AppError::Parse(String)    — format validation failures, slot type parse
// AppError::Database(String) — rusqlite errors, settings read
// AppError::Lock(String)     — Mutex::lock() failures
// AppError::InvalidPath      — path traversal prevention, canonicalize failure
```

### atomic_write_batch Usage
**Source:** `crates/takoyaki-app/src/atomic/mod.rs` lines 64-80
**Apply to:** `assign_sample` command — stage all writes then commit
```rust
// Always: snapshot BEFORE batch write
let engine = SnapshotEngine::new(snapshot_root());
engine.snapshot_files(&affected_file_refs, "pre-sample-assign")?;
// Then batch write
atomic_write_batch(&[
    (&project_work_path, &new_project_work_bytes),
    (&project_strd_path, &new_project_strd_bytes),
])?;
```

### Font / Visual Identity
**Source:** `src/components/project-detail/SlotRow.tsx` lines 138-188, `src/components/management/BankCopyPickerDialog.tsx` lines 91-148
**Apply to:** All new frontend components
- Row height: `h-9`
- Monospace text: `font-mono text-xs` (small text) or `font-mono text-sm` (body)
- Border: `border-b border-[hsl(30,8%,26%)]`
- Hover: `hover:bg-[hsl(30,8%,20%)]`
- Selected/active: `bg-[hsl(30,8%,20%)] border-l-2 border-[hsl(38,85%,55%)]`
- Success green: `text-[hsl(140,60%,42%)]`
- Warning amber: `text-[hsl(38,85%,55%)]`
- Error red: `text-[hsl(0,68%,48%)]`
- Muted: `text-muted-foreground`

---

## No Analog Found

All files have sufficient analogs. No files require falling back to RESEARCH.md patterns exclusively.

| File | Notes |
|------|-------|
| `migrations/V3__wallflower_settings.sql` | Structurally identical to V1/V2 migrations — no surprises |
| `crates/takoyaki-app/src/commands/wallflower.rs` | Split analog: command skeleton from management.rs, DB open from db/wallflower.rs, SQL from RESEARCH.md Pattern 4 |

---

## Metadata

**Analog search scope:** `crates/takoyaki-app/src/` (Rust), `src/components/` (TSX), `src/lib/` (TS), `migrations/` (SQL)
**Files scanned:** 14 source files read
**Pattern extraction date:** 2026-05-02
