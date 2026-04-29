# Phase 2: Read-Only Browser - Pattern Map

**Mapped:** 2026-04-29
**Files analyzed:** 20 new/modified files
**Analogs found:** 18 / 20

---

## Greenfield Note

Takoyaki itself has zero source code — this is a greenfield project. All analogs are drawn from **Wallflower** (`/Users/albair/src/wallflower`), the sister app on the identical Tauri v2 + React/Next.js + tauri-specta + zustand + react-query stack. Wallflower patterns are authoritative — the RESEARCH.md explicitly directs Phase 2 to follow them "precisely."

Phase 1 establishes the scaffold that Phase 2 builds on. Pattern assignments below reference the Wallflower analog to copy from, then note any Takoyaki-specific adaptation.

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `crates/takoyaki-app/src/commands/projects.rs` | command | request-response (CRUD) | `wallflower/crates/wallflower-app/src/commands/samples.rs` | exact |
| `crates/takoyaki-app/src/commands/samples.rs` | command | request-response (CRUD) | `wallflower/crates/wallflower-app/src/commands/samples.rs` | exact |
| `crates/takoyaki-app/src/commands/health.rs` | command | event-driven (background) | `wallflower/crates/wallflower-app/src/commands/analysis.rs` | exact |
| `crates/takoyaki-app/src/commands/mod.rs` | config | N/A | `wallflower/crates/wallflower-app/src/commands/mod.rs` | exact |
| `crates/takoyaki-app/src/db/projects.rs` | service | CRUD (SQLite) | `wallflower/crates/wallflower-core/src/db/mod.rs` | role-match |
| `crates/takoyaki-app/src/health/mod.rs` | service | batch (file I/O) | `wallflower/crates/wallflower-core/src/error.rs` (error types) + RESEARCH.md Pattern 3 | partial |
| `crates/takoyaki-app/src/lib.rs` | config | N/A | `wallflower/crates/wallflower-app/src/lib.rs` | exact |
| `src/lib/tauri.ts` | utility | request-response | `wallflower/src/lib/tauri.ts` | exact |
| `src/lib/types.ts` | model | N/A | `wallflower/src/lib/types.ts` | exact |
| `src/lib/stores/navigation.ts` | store | event-driven | `wallflower/src/lib/stores/library.ts` + `wallflower/src/lib/stores/sample-browser.ts` | role-match |
| `src/components/projects/ProjectTable.tsx` | component | CRUD | `wallflower/src/components/explore/SampleTable.tsx` | exact |
| `src/components/projects/ProjectSearchBar.tsx` | component | request-response | `wallflower/src/components/library/FilterBar.tsx` | exact |
| `src/components/projects/ProjectRow.tsx` | component | CRUD | `wallflower/src/components/explore/SampleTableRow.tsx` | exact |
| `src/components/project-detail/ProjectDetailView.tsx` | component | CRUD | `wallflower/src/components/library/JamDetail.tsx` | role-match |
| `src/components/project-detail/MetadataHeader.tsx` | component | CRUD | `wallflower/src/components/library/JamDetail.tsx` | role-match |
| `src/components/project-detail/BanksTab.tsx` | component | CRUD | `wallflower/src/components/explore/SampleBrowser.tsx` | partial |
| `src/components/project-detail/BankGridCell.tsx` | component | CRUD | no close analog | no-analog |
| `src/components/project-detail/SamplesTab.tsx` | component | CRUD | `wallflower/src/components/explore/SampleTable.tsx` | role-match |
| `src/components/project-detail/SlotRow.tsx` | component | CRUD | `wallflower/src/components/explore/SampleTableRow.tsx` | role-match |
| `src/components/project-detail/HealthTab.tsx` | component | event-driven | `wallflower/src/components/analysis/AnalysisStatus.tsx` | role-match |
| `src/components/health/HealthEventListener.tsx` | component | event-driven | `wallflower/src/components/tauri-event-listener.tsx` | exact |
| `src/app/page.tsx` | component | request-response | `wallflower/src/app/page.tsx` | exact |
| `crates/takoyaki-app/tests/projects.rs` | test | CRUD | no close analog | no-analog |
| `crates/takoyaki-app/tests/health_check.rs` | test | batch | no close analog | no-analog |

---

## Pattern Assignments

### `crates/takoyaki-app/src/commands/projects.rs` (command, request-response)

**Analog:** `/Users/albair/src/wallflower/crates/wallflower-app/src/commands/samples.rs`

**Imports pattern** (lines 1-4):
```rust
use crate::AppState;
use wallflower_core::db;
use wallflower_core::db::schema::{SampleFilter, SampleFilterOptions, SampleRecord};
```

Adapt to Takoyaki:
```rust
use crate::AppState;
use crate::db;
use crate::error::AppError;
```

**Core command pattern** (lines 5-20, `samples.rs`):
```rust
#[tauri::command]
pub async fn get_all_samples(
    state: tauri::State<'_, AppState>,
    filter: SampleFilter,
) -> Result<Vec<SampleRecord>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db::get_all_samples(&db.conn, &filter).map_err(|e| e.to_string())
}
```

Adaptation — for Takoyaki, use typed `AppError` (from RESEARCH.md) instead of `String` errors, and add `#[specta::specta]` on each command:
```rust
#[tauri::command]
#[specta::specta]
pub async fn list_projects(
    state: tauri::State<'_, AppState>,
    filter: ProjectFilter,
) -> Result<Vec<ProjectSummary>, AppError> {
    let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db::projects::list_projects(&db.conn, &filter).map_err(AppError::Database)
}
```

**Key difference from Wallflower:** Takoyaki uses `#[specta::specta]` on every command (tauri-specta v2 Builder pattern); Wallflower predates this and returns `String` errors. For Takoyaki, all commands must return `Result<T, AppError>` where `AppError` derives `specta::Type`.

---

### `crates/takoyaki-app/src/commands/health.rs` (command, event-driven)

**Analog:** `/Users/albair/src/wallflower/crates/wallflower-app/src/commands/analysis.rs`

**Spawn + emit pattern** (lines 9-11, 283-298 of `analysis.rs`):
```rust
#[command]
pub async fn analyze_jam(app: AppHandle, jam_id: String) -> Result<(), String> {
    // ... setup ...
    tokio::spawn(async move {
        for jam_id in pending {
            if let Err(e) = analyze_jam(app_clone.clone(), jam_id.clone()).await {
                tracing::warn!("Analysis failed for {}: {}", jam_id, e);
            }
        }
    });
    Ok(count)
}
```

Adapt to health check — return `Ok(())` immediately, spawn background work, emit event on completion:
```rust
#[tauri::command]
#[specta::specta]
pub async fn run_health_check(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    project_id: String,
) -> Result<(), AppError> {
    let project_path = {
        let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        db::projects::get_card_path(&db.conn, &project_id)?
    };

    tauri::async_runtime::spawn(async move {
        use tauri::Emitter;
        let issues = crate::health::perform_health_check(&project_path).await;
        let result = HealthCheckComplete { project_id, issues, scanned_at: chrono::Utc::now().to_rfc3339() };
        app.emit("health-complete", result).unwrap_or_else(|e| {
            tracing::error!("health-complete emit failed: {e}");
        });
    });

    Ok(())
}
```

**Critical anti-pattern from RESEARCH.md:** Never block the command handler on file I/O. `run_health_check` returns `Ok(())` before any file reads happen. This mirrors the `queue_pending_analysis` pattern in `analysis.rs` lines 275-298.

---

### `crates/takoyaki-app/src/db/projects.rs` (service, CRUD)

**Analog:** `/Users/albair/src/wallflower/crates/wallflower-core/src/db/mod.rs`

**Database struct pattern** (lines 57-102 of `db/mod.rs`):
```rust
pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    pub fn open_default() -> Result<Self> {
        let data_dir = dirs::data_dir()
            .ok_or_else(|| WallflowerError::Config("Could not determine app data directory".into()))?
            .join("wallflower");
        let db_path = data_dir.join("wallflower.db");
        Self::open(&db_path)
    }

    pub fn open_in_memory() -> Result<Self> { ... }  // for testing
}
```

**SearchFilter pattern** (lines 17-40 of `db/mod.rs`) — copy this pattern for `ProjectFilter`:
```rust
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchFilter {
    pub query: Option<String>,
    pub keys: Option<Vec<String>>,
    pub tempo_min: Option<f64>,
    pub tempo_max: Option<f64>,
    // ...
}
```

For Takoyaki `ProjectFilter`, add `specta::Type` derive (not present in Wallflower):
```rust
#[derive(Debug, serde::Deserialize, specta::Type)]
pub struct ProjectFilter {
    pub name: Option<String>,
    pub bpm_min: Option<u16>,
    pub bpm_max: Option<u16>,
    pub modified_since: Option<String>,
}
```

**Parameterized query pattern** (RESEARCH.md Pattern 5) — use `params![]` macro, never string interpolation for user-supplied values:
```rust
pub fn list_projects(
    conn: &rusqlite::Connection,
    filter: &ProjectFilter,
) -> Result<Vec<ProjectSummary>, rusqlite::Error> {
    let mut conditions = vec!["1=1"];
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];

    if let Some(ref name) = filter.name {
        conditions.push("project_name LIKE ?");
        params.push(Box::new(format!("%{}%", name)));
    }
    // ... other conditions ...
    let sql = format!("SELECT ... FROM projects WHERE {} ORDER BY ...", conditions.join(" AND "));
    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    // conn.prepare(&sql)?.query_map(params_refs.as_slice(), |row| { ... })
}
```

---

### `crates/takoyaki-app/src/health/mod.rs` (service, file I/O batch)

**No direct analog in Wallflower** — use RESEARCH.md Pattern 3 directly.

The closest pattern is the recording engine's event emission in `lib.rs` lines 200-276, which shows the channel-based event bridge. For health check, use the simpler `tauri::async_runtime::spawn` + `app.emit()` approach from RESEARCH.md Pattern 2.

**Audio spec reading pattern** (RESEARCH.md Pattern 3 — use verbatim):
```rust
use hound::WavReader;
use std::path::Path;

pub fn read_audio_spec(path: &Path) -> Result<AudioSpec, std::io::Error> {
    let kind = infer::get_from_path(path)
        .ok()
        .flatten()
        .map(|t| t.mime_type().to_string());

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

    if kind.as_deref() == Some("audio/x-wav") || ext == "wav" {
        match WavReader::open(path) {
            Ok(reader) => {
                let spec = reader.spec();
                Ok(AudioSpec::Wav {
                    sample_rate: spec.sample_rate,
                    bits_per_sample: spec.bits_per_sample,
                    channels: spec.channels,
                })
            }
            Err(_) => Ok(AudioSpec::Unknown { detected_type: kind }),
        }
    } else if kind.as_deref() == Some("audio/aiff") || ext == "aif" || ext == "aiff" {
        // Use aifc::AifcReader
        // ...
    } else {
        Ok(AudioSpec::Unknown { detected_type: kind })
    }
}
```

**Critical:** Call `WavReader::open().spec()` only — NEVER call `.samples()`. Header read only.

---

### `crates/takoyaki-app/src/lib.rs` (config, app wiring)

**Analog:** `/Users/albair/src/wallflower/crates/wallflower-app/src/lib.rs`

**AppState pattern** (lines 33-43 of `lib.rs`):
```rust
pub struct AppState {
    pub db: Mutex<Database>,
    pub config: Mutex<AppConfig>,
    pub watcher: Mutex<Option<WatcherHandle>>,
    // ... additional fields per phase ...
}
```

Takoyaki's AppState at Phase 2 — simpler than Wallflower (no recording engine, no sidecar):
```rust
pub struct AppState {
    pub db: Mutex<Database>,
    pub volume_path: Mutex<Option<std::path::PathBuf>>,  // mounted OT volume
}
```

**invoke_handler pattern** (lines 438-507 of `lib.rs`) — register all `#[specta::specta]` commands here using `tauri::generate_handler![]`. For tauri-specta v2, also add a Builder export step in the `run()` function before the Tauri builder (see tauri-specta v2 docs).

**Emitter import** (line 1 of `analysis.rs`):
```rust
use tauri::{command, AppHandle, Emitter, Manager};
```
The `Emitter` trait must be in scope for `app.emit()` to work.

---

### `src/lib/tauri.ts` (utility, request-response)

**Analog:** `/Users/albair/src/wallflower/src/lib/tauri.ts`

**Full file structure** (lines 1-29 of `tauri.ts`) — copy this exact pattern:
```typescript
import { invoke } from "@tauri-apps/api/core";
import type {
  ProjectSummary,
  ProjectFilter,
  ProjectDetail,
  SampleSlot,
  HealthCheckResult,
} from "./types";
```

**Invoke function pattern** (lines 34-36 of `tauri.ts`):
```typescript
export async function listJams(): Promise<JamRecord[]> {
  return invoke("list_jams");
}
```

Takoyaki Phase 2 additions (from RESEARCH.md Code Examples):
```typescript
export async function listProjects(filter: ProjectFilter): Promise<ProjectSummary[]> {
  return invoke("list_projects", { filter });
}

export async function getProjectDetail(projectId: string): Promise<ProjectDetail> {
  return invoke("get_project_detail", { projectId });
}

export async function getProjectSamples(projectId: string): Promise<{
  flex: SampleSlot[];
  static: SampleSlot[];
}> {
  return invoke("get_project_samples", { projectId });
}

export async function runHealthCheck(projectId: string): Promise<void> {
  return invoke("run_health_check", { projectId });
  // Results arrive via "health-complete" event, not return value
}
```

**Key convention from Wallflower:** Every function is `async`, returns a typed Promise, passes args as an object literal `{ argName }` matching Rust snake_case command parameter names.

---

### `src/lib/stores/navigation.ts` (store, event-driven)

**Analog:** `/Users/albair/src/wallflower/src/lib/stores/library.ts` + `sample-browser.ts`

**Zustand store pattern** (lines 1-40 of `library.ts`):
```typescript
import { create } from "zustand";
import type { SearchFilter } from "@/lib/types";

export interface LibraryState {
  selectedJamId: string | null;
  setSelectedJam: (id: string | null) => void;
  filter: SearchFilter;
  hasActiveFilters: boolean;
  setFilter: (partial: Partial<SearchFilter>) => void;
  clearFilter: () => void;
  clearFilterField: (field: keyof SearchFilter) => void;
}

export const useLibraryStore = create<LibraryState>((set) => ({
  selectedJamId: null,
  setSelectedJam: (id) => set({ selectedJamId: id }),
  filter: {},
  hasActiveFilters: false,
  // ...
}));
```

**Sort toggle pattern** (lines 67-80 of `sample-browser.ts`) — use for any sortable column:
```typescript
setSort: (column) =>
  set((state) => {
    if (state.sortColumn === column) {
      return { sortDirection: state.sortDirection === "asc" ? "desc" : "asc" };
    }
    return { sortColumn: column, sortDirection: "asc" };
  }),
```

Takoyaki navigation store (from RESEARCH.md Pattern 6):
```typescript
import { create } from "zustand";

type View = "project-list" | "project-detail";

interface NavigationState {
  view: View;
  selectedProjectId: string | null;
  selectedBankIndex: number | null;
  activeTab: "banks" | "samples" | "health";
  navigateToProject: (projectId: string) => void;
  navigateToList: () => void;
  selectBank: (bankIndex: number | null) => void;
  setActiveTab: (tab: "banks" | "samples" | "health") => void;
}

export const useNavigationStore = create<NavigationState>((set) => ({
  view: "project-list",
  selectedProjectId: null,
  selectedBankIndex: null,
  activeTab: "banks",
  navigateToProject: (projectId) =>
    set({ view: "project-detail", selectedProjectId: projectId, selectedBankIndex: null, activeTab: "banks" }),
  navigateToList: () =>
    set({ view: "project-list", selectedProjectId: null, selectedBankIndex: null }),
  selectBank: (bankIndex) => set({ selectedBankIndex: bankIndex }),
  setActiveTab: (tab) => set({ activeTab: tab }),
}));
```

---

### `src/components/projects/ProjectTable.tsx` (component, CRUD)

**Analog:** `/Users/albair/src/wallflower/src/components/explore/SampleTable.tsx`

**File header pattern** (lines 1-16 of `SampleTable.tsx`):
```typescript
"use client";

import { useMemo, useCallback } from "react";
import {
  Table,
  TableHeader,
  TableBody,
  TableHead,
  TableRow,
} from "@/components/ui/table";
import { ScrollArea } from "@/components/ui/scroll-area";
```

**Column definition pattern** (lines 18-31 of `SampleTable.tsx`):
```typescript
const COLUMNS: {
  key: SortColumn | null;
  label: string;
  sortable: boolean;
  className: string;
}[] = [
  { key: "name", label: "Name", sortable: true, className: "min-w-[120px]" },
  { key: "bpm", label: "BPM", sortable: true, className: "w-14" },
  // ...
];
```

**React-query hook pattern** (RESEARCH.md Code Examples):
```typescript
import { useQuery } from "@tanstack/react-query";
import { listProjects } from "@/lib/tauri";
import { useNavigationStore } from "@/lib/stores/navigation";

export function useProjectList(filter: ProjectFilter) {
  return useQuery({
    queryKey: ["projects", filter],
    queryFn: () => listProjects(filter),
  });
}
```

**Empty state pattern** (lines 146-169 of `SampleTable.tsx`):
```typescript
if (isLoaded && samples.length === 0) {
  if (hasActiveFilters) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-4 px-12 py-24">
        <h2 className="text-xl font-semibold text-foreground">No matching samples</h2>
        <p className="max-w-md text-center text-sm text-muted-foreground">
          Try adjusting your filters or clearing the search.
        </p>
        <button type="button" onClick={clearFilter} ...>Clear Filters</button>
      </div>
    );
  }
  return null;  // Global empty state handled by parent
}
```

**TanStack Query v5 note:** Use `isPending` (not `isLoading`) for "no data yet" skeleton states. `isLoading` in v5 means "fetching AND no cached data."

---

### `src/components/projects/ProjectSearchBar.tsx` (component, request-response)

**Analog:** `/Users/albair/src/wallflower/src/components/library/FilterBar.tsx`

**Filter bar structure** (lines 180-304 of `FilterBar.tsx`):
```typescript
export function FilterBar({ resultCount }: FilterBarProps) {
  const { filter, setFilter, hasActiveFilters, clearFilter } = useLibraryStore();

  return (
    <div role="search" aria-label="Filter jams" className="sticky top-0 z-20 mb-4 rounded-xl p-4">
      {/* Row 1: Filter controls */}
      <div className="flex flex-wrap items-center gap-2">
        <SearchInput />
        {/* BPM/date dropdowns */}
      </div>

      {/* Row 2: Active filter chips */}
      {hasActiveFilters && (
        <div className="mt-2 flex flex-wrap items-center gap-2">
          {chips.map(chip => <FilterChip key={...} ... />)}
          <button type="button" onClick={clearFilter}>Clear all</button>
          {resultCount !== undefined && (
            <span className="ml-auto text-xs text-muted-foreground">
              {resultCount} {resultCount === 1 ? "result" : "results"}
            </span>
          )}
        </div>
      )}

      {/* Screen reader live region */}
      <div aria-live="polite" className="sr-only">
        {resultCount !== undefined ? `${resultCount} projects matching` : ""}
      </div>
    </div>
  );
}
```

**Input + Select shadcn pattern** — Wallflower uses shadcn `Input`, `Popover`, `Command` for filters. For Takoyaki's simpler filter (text + BPM range + date), use shadcn `Input` + shadcn `Select` directly (no Command needed).

---

### `src/components/projects/ProjectRow.tsx` (component, CRUD)

**Analog:** `/Users/albair/src/wallflower/src/components/explore/SampleTableRow.tsx`

**Row interaction pattern** (lines 36-53 of `SampleTableRow.tsx`):
```typescript
<TableRow
  className={`cursor-pointer h-10 transition-colors ${
    isSelected ? "bg-[hsl(220,14%,18%)]" : "hover:bg-muted/50"
  }`}
  onClick={onSelect}
  tabIndex={0}
  onKeyDown={(e) => {
    if (e.key === "Enter") { e.preventDefault(); onSelect(); }
  }}
  aria-selected={isSelected}
>
```

**Cell truncation pattern** (lines 74-89 of `SampleTableRow.tsx`):
```typescript
<TableCell className="min-w-[120px]">
  <span className="truncate text-sm text-foreground">
    {sample.name}
  </span>
</TableCell>
```

**Tabular nums for numeric columns** (lines 128-131 of `SampleTableRow.tsx`):
```typescript
<TableCell className="w-14 text-xs tabular-nums text-muted-foreground">
  {sample.tempoBpm ? Math.round(sample.tempoBpm) : "--"}
</TableCell>
```

For Takoyaki's BPM display: `{project.tempoBpm !== null ? project.tempoBpm.toFixed(1) : "--"}` (OT stores tempo × 10; parser divides by 10 — display with one decimal per UI-SPEC).

---

### `src/app/page.tsx` (component, request-response)

**Analog:** `/Users/albair/src/wallflower/src/app/page.tsx`

**View-switch pattern** (lines 51-142 of `page.tsx`) — Wallflower uses `activeTab` + `selectedJamId` to determine what to render. Takoyaki uses `useNavigationStore` view state:
```typescript
"use client";

import { useNavigationStore } from "@/lib/stores/navigation";
import { ProjectTable } from "@/components/projects/ProjectTable";
import { ProjectDetailView } from "@/components/project-detail/ProjectDetailView";
import { HealthEventListener } from "@/components/health/HealthEventListener";

export default function Home() {
  const { view } = useNavigationStore();

  return (
    <main id="main-content" role="main" className="flex min-h-screen flex-col">
      <HealthEventListener />
      {/* Sidebar nav (Phase 1 scaffold) */}
      {view === "project-list" && <ProjectTable />}
      {view === "project-detail" && <ProjectDetailView />}
    </main>
  );
}
```

**useEffect on mount** (lines 33-47 of `page.tsx`) — Wallflower calls `queuePendingAnalysis()` on mount. Takoyaki should similarly trigger `indexProjects()` or check volume state on mount (from Phase 1 volume detection store).

---

### `src/components/project-detail/ProjectDetailView.tsx` (component, CRUD)

**Analog:** `/Users/albair/src/wallflower/src/components/library/JamDetail.tsx`

**useQuery for detail data** (lines 28-33 of `JamDetail.tsx`):
```typescript
export function JamDetail({ jamId, onBack }: JamDetailProps) {
  const queryClient = useQueryClient();
  // ...
}
```

Takoyaki adaptation:
```typescript
export function ProjectDetailView() {
  const { selectedProjectId } = useNavigationStore();
  const queryClient = useQueryClient();

  const { data: project, isPending } = useQuery({
    queryKey: ["project", selectedProjectId],
    queryFn: () => getProjectDetail(selectedProjectId!),
    enabled: selectedProjectId !== null,
  });

  // Trigger health check on project open
  useEffect(() => {
    if (selectedProjectId) {
      runHealthCheck(selectedProjectId).catch(() => {});
    }
  }, [selectedProjectId]);
  // ...
}
```

**invalidateQueries on event receipt** — copy from `JamDetail.tsx` lines 258-262:
```typescript
queryClient.invalidateQueries({ queryKey: ["jams"] });
if (jamId) {
  queryClient.invalidateQueries({ queryKey: ["jam", jamId] });
}
```

---

### `src/components/health/HealthEventListener.tsx` (component, event-driven)

**Analog:** `/Users/albair/src/wallflower/src/components/tauri-event-listener.tsx`

**Full event listener pattern** (lines 51-339 of `tauri-event-listener.tsx`) — copy the entire structure:

```typescript
"use client";

import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";

export function HealthEventListener() {
  const queryClient = useQueryClient();

  useEffect(() => {
    const cleanupFns: (() => void)[] = [];

    async function setupListeners() {
      try {
        const { listen } = await import("@tauri-apps/api/event");

        const unlisten = await listen<HealthCheckComplete>(
          "health-complete",
          (event) => {
            const { project_id } = event.payload;
            queryClient.setQueryData(["health", project_id], event.payload);
          }
        );
        cleanupFns.push(unlisten);
      } catch {
        // Not in Tauri context (SSR, browser dev)
      }
    }

    setupListeners();
    return () => cleanupFns.forEach((fn) => fn());
  }, [queryClient]);

  return null;
}
```

**Critical pattern from `tauri-event-listener.tsx` lines 84-99:**
- `const { listen } = await import("@tauri-apps/api/event")` — dynamic import inside try/catch to handle non-Tauri context (SSR)
- All `unlisten` functions pushed to `cleanupFns[]` array
- `return () => cleanupFns.forEach(fn => fn())` — cleanup in useEffect return

---

### `src/components/project-detail/HealthTab.tsx` (component, event-driven)

**Analog:** `/Users/albair/src/wallflower/src/components/analysis/AnalysisStatus.tsx`

**Step/status display pattern** (lines 14-62 of `AnalysisStatus.tsx`):
```typescript
export function AnalysisStatus({ currentStep, completedSteps, variant }: AnalysisStatusProps) {
  if (variant === "card") {
    return (
      <div className="flex items-center gap-1.5">
        <span className="relative flex h-1 w-1">
          <span className="absolute inline-flex h-full w-full animate-pulse rounded-full bg-[#E8863A]" />
        </span>
        <span className="text-xs text-muted-foreground">Analyzing...</span>
      </div>
    );
  }
  // Detail variant: step-by-step progress
  return (
    <div className="mt-4 flex items-center gap-2">
      {STEPS.map((step) => {
        const isCompleted = completedSteps.includes(step.toLowerCase());
        const isCurrent = currentStep === step.toLowerCase();
        return (
          <div key={step} className="flex items-center gap-1">
            {isCompleted ? <Check size={12} /> : isCurrent ? <Loader2 size={12} className="animate-spin" /> : <Minus size={12} />}
            <span className={cn("text-xs", ...)}>{step}</span>
          </div>
        );
      })}
    </div>
  );
}
```

Takoyaki HealthTab groups by severity (Error / Warning / Info) with counts — same "grouped items with icons" pattern but applied to health issues. Use `Check` icon for all-clear, `X` for errors, `AlertTriangle` for warnings, `Info` for info items.

**Data source** — health results come from react-query cache keyed by `["health", projectId]`, populated by `HealthEventListener`:
```typescript
const { data: healthData, isLoading } = useQuery({
  queryKey: ["health", selectedProjectId],
  enabled: false,  // never fetches — only reads what HealthEventListener sets via setQueryData
});
```

---

### `src/components/project-detail/SamplesTab.tsx` (component, CRUD)

**Analog:** `/Users/albair/src/wallflower/src/components/explore/SampleTable.tsx`

**Two-section table pattern** — Wallflower's SampleTable renders a single homogeneous list. Takoyaki's SamplesTab renders two sections (Flex / Static) as separate `<Table>` instances or `<TableBody>` groups with a section header row in between. Use the same `Table` / `TableBody` / `TableRow` shadcn components.

**Collapsible cross-reference rows (D-10)** — shadcn `Collapsible` wraps the cross-reference panel inside the slot row. Each `SlotRow` contains a `Collapsible` that expands to show which banks/parts/tracks reference that sample:
```typescript
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
// Inside SlotRow:
<Collapsible>
  <CollapsibleTrigger>...</CollapsibleTrigger>
  <CollapsibleContent>
    {/* Banks/parts/tracks cross-ref list */}
  </CollapsibleContent>
</Collapsible>
```

**Hide empty slots toggle (D-08)** — use shadcn `Toggle` component:
```typescript
import { Toggle } from "@/components/ui/toggle";
<Toggle pressed={showEmpty} onPressedChange={setShowEmpty}>
  Show empty slots
</Toggle>
```

---

### `src/components/project-detail/BanksTab.tsx` (component, CRUD)

**No close Wallflower analog** — Wallflower has no grid-of-cells component. Use CSS Grid directly.

**Bank grid pattern** (from RESEARCH.md D-06 + D-05):
```typescript
// 4×4 grid matching OT layout (banks 1-16)
<div className="grid grid-cols-4 gap-2">
  {banks.map((bank, i) => (
    <BankGridCell
      key={i}
      bankIndex={i}
      populated={bank.populated}
      onClick={bank.populated ? () => selectBank(i) : undefined}
    />
  ))}
</div>
```

**Drill-down panel** — when `selectedBankIndex !== null`, render a panel below the grid showing parts (4) and tracks (8 per part) for the selected bank. This is a conditional render pattern identical to `page.tsx` lines 118-130 where `selectedJamId` controls whether to show Timeline or JamDetail.

---

### `src/components/project-detail/BankGridCell.tsx` (component, CRUD)

**No analog.** Design from scratch:
```typescript
interface BankGridCellProps {
  bankIndex: number;
  populated: boolean;
  selected: boolean;
  onClick?: () => void;
}

export function BankGridCell({ bankIndex, populated, selected, onClick }: BankGridCellProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={!populated}
      className={`
        flex h-12 w-12 flex-col items-center justify-center rounded border font-mono text-xs
        ${populated ? "cursor-pointer hover:bg-muted/50" : "cursor-default opacity-40"}
        ${selected ? "border-accent bg-muted/30" : "border-border"}
      `}
      aria-label={`Bank ${bankIndex + 1}${populated ? "" : " (empty)"}`}
      aria-pressed={selected}
    >
      <span className={`h-2 w-2 rounded-full ${populated ? "bg-foreground" : "border border-muted-foreground"}`} />
      <span className="mt-1 tabular-nums">{String(bankIndex + 1).padStart(2, "0")}</span>
    </button>
  );
}
```

---

## Shared Patterns

### Tauri IPC Layer
**Source:** `/Users/albair/src/wallflower/src/lib/tauri.ts` (entire file)
**Apply to:** All `src/lib/tauri.ts` functions

```typescript
import { invoke } from "@tauri-apps/api/core";
// Each exported function: async, typed return, args as object literal
export async function fnName(arg: Type): Promise<ReturnType> {
  return invoke("command_name", { arg });
}
```

**Do not use** `window.__TAURI__.invoke()` global — use the module import.

---

### Rust Command Signature
**Source:** `/Users/albair/src/wallflower/crates/wallflower-app/src/commands/samples.rs` lines 5-20
**Apply to:** All `crates/takoyaki-app/src/commands/*.rs` files

```rust
#[tauri::command]
#[specta::specta]  // TAKOYAKI ADDITION: required for tauri-specta v2 type generation
pub async fn command_name(
    state: tauri::State<'_, AppState>,
    param: InputType,
) -> Result<OutputType, AppError> {
    let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db::module::function(&db.conn, &param).map_err(AppError::Database)
}
```

Note: Wallflower returns `Result<T, String>` — Takoyaki uses typed `AppError` with `specta::Type` for auto-generated TypeScript error types.

---

### Event Listener Lifecycle
**Source:** `/Users/albair/src/wallflower/src/components/tauri-event-listener.tsx` lines 81-336
**Apply to:** `src/components/health/HealthEventListener.tsx`, any future event listener components

```typescript
useEffect(() => {
  const cleanupFns: (() => void)[] = [];
  async function setupListeners() {
    try {
      const { listen } = await import("@tauri-apps/api/event");
      const unlisten = await listen<PayloadType>("event-name", (event) => {
        // handle event.payload
      });
      cleanupFns.push(unlisten);
    } catch {
      // Not in Tauri context
    }
  }
  setupListeners();
  return () => cleanupFns.forEach((fn) => fn());
}, [/* stable refs only */]);
```

---

### SQLite Lock Pattern
**Source:** `/Users/albair/src/wallflower/crates/wallflower-app/src/commands/samples.rs` line 10
**Apply to:** All Rust command handlers that access the DB

```rust
let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
```

Hold the lock for the minimum necessary scope. For background tasks (health check), grab path from DB, drop the lock, then do all file I/O outside the lock.

---

### Zustand Store Structure
**Source:** `/Users/albair/src/wallflower/src/lib/stores/library.ts` lines 1-40
**Apply to:** `src/lib/stores/navigation.ts`, any additional stores

```typescript
import { create } from "zustand";

interface StoreState {
  // State fields
  field: Type;
  // Actions
  setField: (value: Type) => void;
}

export const useStoreName = create<StoreState>((set) => ({
  field: defaultValue,
  setField: (value) => set({ field: value }),
}));
```

---

### React-Query Cache Management
**Source:** `/Users/albair/src/wallflower/src/components/library/JamDetail.tsx` lines 56-65 and `tauri-event-listener.tsx` lines 256-264
**Apply to:** All components that read or write react-query cache

```typescript
// Invalidate after mutation:
queryClient.invalidateQueries({ queryKey: ["projects"] });

// Write to cache directly (for event-sourced data like health results):
queryClient.setQueryData(["health", projectId], eventPayload);

// Read from cache set by event listener (enabled: false prevents re-fetch):
const { data } = useQuery({
  queryKey: ["health", projectId],
  queryFn: () => Promise.resolve(null),  // never called when enabled: false
  enabled: false,
});
```

---

### Error Type Pattern
**Source:** `/Users/albair/src/wallflower/crates/wallflower-core/src/error.rs`
**Apply to:** `crates/takoyaki-app/src/error.rs` (new file, Phase 1 or Phase 2)

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Lock error: {0}")]
    Lock(String),

    #[error("Parse error: {0}")]
    Parse(String),
}

// For tauri-specta v2 IPC, AppError must also implement:
// serde::Serialize + specta::Type
```

Wallflower uses `String` for IPC errors; Takoyaki uses typed `AppError` — this is the primary structural difference.

---

### Monospace Typography Pattern
**Source:** Phase 1 CONTEXT.md D-08
**Apply to:** All table cells, numeric displays, slot numbers, BPM values

```typescript
// Slot numbers: zero-padded, monospace
<span className="font-mono tabular-nums text-xs">#001</span>

// BPM: one decimal, monospace
<span className="font-mono tabular-nums">{(bpm / 10).toFixed(1)}</span>

// Filename: truncated with tooltip
<span className="truncate font-mono text-sm">{filename}</span>
```

Warm dark palette colors from Phase 1 (match Wallflower's `#1D2129` background, `#E8863A` accent).

---

## No Analog Found

Files with no close match in the codebase (planner should use RESEARCH.md patterns instead):

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `src/components/project-detail/BankGridCell.tsx` | component | CRUD | No grid-of-cells component exists in Wallflower — OT's 4×4 bank layout is unique |
| `crates/takoyaki-app/tests/projects.rs` | test | CRUD | Wallflower tests not available in search; use Rust standard `#[cfg(test)]` module + `rusqlite::Connection::open_in_memory()` |
| `crates/takoyaki-app/tests/health_check.rs` | test | batch | Same as above; create mock OT volume directory in `tests/fixtures/mock_ot_volume/` |
| `crates/takoyaki-app/src/health/mod.rs` | service | file I/O | Wallflower has no audio header read module — use RESEARCH.md Pattern 3 verbatim (hound + aifc + infer) |

---

## Metadata

**Analog search scope:** `/Users/albair/src/wallflower/crates/` (Rust), `/Users/albair/src/wallflower/src/` (TypeScript/TSX)
**Files scanned:** 22 source files read
**Pattern extraction date:** 2026-04-29
**Wallflower commit:** current HEAD (main branch)

**Key structural difference — Wallflower vs Takoyaki:**
- Wallflower commands return `Result<T, String>` for IPC errors
- Takoyaki must use `Result<T, AppError>` where `AppError: serde::Serialize + specta::Type` for tauri-specta v2 typed error generation
- This means `AppError` must be defined before any command module and must derive the specta type trait
