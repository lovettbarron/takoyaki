# Phase 2: Read-Only Browser - Research

**Researched:** 2026-04-29
**Domain:** Tauri v2 IPC layer for read-heavy data, Rust audio format detection, SQLite search/filter indexing, React/Next.js component patterns for dense data tables and tabbed detail views
**Confidence:** HIGH (stack verified against Wallflower blueprint and cargo/npm registries), MEDIUM (OT format field structure for bank/machine queries)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Project List & Navigation**
- D-01: Projects displayed as a compact monospace table with columns: name, BPM, banks (used/total), last modified. Dense and scannable.
- D-02: Always-visible search/filter bar above the project table. Text search filters by name; dropdown filters for BPM range and date. Instant filtering, no separate search page. Backed by SQLite index (MGMT-04).
- D-03: Clicking a project replaces the list with a project detail view. Breadcrumb trail at top (Projects › LIVESET_01 › Bank 03) for navigation back.
- D-04: Project detail view uses tabbed sections: Banks, Samples, Health. Each tab is a distinct concern with its own presentation.

**Project Detail Structure**
- D-05: Bank drill-down goes to banks → parts → tracks depth. Patterns shown as populated/empty indicator grid (dots) — not individually expandable.
- D-06: Banks displayed as a 4×4 grid matching the OT's own 16-bank layout. Filled dot for populated banks, empty dot for unused. Click a bank to drill into its 4 parts and 8 tracks per part.
- D-07: Compact metadata header always visible below breadcrumb: project name, tempo, bank count, last modified.

**Sample Slot Display**
- D-08: Flex and Static samples displayed as two separate table sections on the Samples tab. Empty slots hidden by default with a "show all" toggle.
- D-09: Each sample slot row shows: slot number (#001–#128), filename (truncated if long), sample rate, and a status icon (✓ OK, ✘ missing, ⚠ format issue).
- D-10: Click/expand a slot row to see which banks, parts, and tracks reference that sample.

**Health Check UX**
- D-11: Health check runs automatically in the background when a project is opened. Results populate the Health tab badge count and inline status icons.
- D-12: Three severity tiers: Error (missing files), Warning (wrong format), Info (unused samples). Grouped by severity with counts.
- D-13: Health issues appear inline on Samples tab (status icons) AND in full detail on Health tab.
- D-14: Healthy project state shows a calm "All clear" message with timestamp.

### Claude's Discretion
- Exact table column widths and truncation behavior for long filenames
- How the "show all" toggle for empty slots works (button, switch, or link)
- Breadcrumb styling and back-navigation interaction
- Tab styling (underline, pill, segmented control)
- Health check loading state while background scan is in progress
- Sort order defaults for project list and sample slot tables
- Whether bank grid cells show additional info on hover
- Pattern grid dot layout within a bank detail view
- Animation/transition behavior when navigating between views

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| BROW-02 | User can list all OT projects on a mounted card with metadata (name, bank count, tempo, last modified) | SQLite projects table (Phase 1 schema); `list_projects` Tauri command queries indexed metadata; no re-parsing files on every list view |
| BROW-03 | User can view which banks and patterns are populated within a project | `get_project_banks` command calls ot-parser to read bank files; bank populated flag derived from pattern count > 0 in bank file |
| BROW-04 | User can view all Flex and Static sample slots (128 each) with assigned file paths | `get_project_samples` command reads project.work via ot-parser; returns `Vec<SampleSlot>` for Flex and Static arrays |
| BROW-05 | User can view project-level metadata including tempo, bank names, part names, and active machine types per track | `get_project_detail` command reads project.work + all 16 bank files; combines into `ProjectDetail` response type |
| DETC-01 | User can detect missing or broken sample references across all slots in a project | `run_health_check` command: for each occupied slot, `std::fs::exists()` against OT volume mount point; background async task |
| DETC-02 | User can validate audio file format compatibility (flag non-44.1kHz, wrong bit depth, non-WAV/AIFF) | `hound` crate (WAV header read), `aifc` crate (AIFF header read); read spec only — no audio data loaded |
| DETC-03 | User can detect unused samples (assigned to slots but never triggered in any pattern) | Cross-reference sample slot list against track machine assignments across all banks/parts; emitted as Info severity |
| MGMT-04 | User can search and filter projects by name, tempo, or date via indexed metadata | SQLite WHERE/LIKE queries on `projects` table; no FTS5 needed — simple LIKE for name, range for BPM, date comparison |
</phase_requirements>

---

## Summary

Phase 2 is a read-heavy presentation layer. It does not write to the CF card — every operation is read-only. The technical work divides cleanly into three concerns: (1) populating the SQLite project index from the OT card on connect, (2) reading and projecting parsed OT binary data as Tauri commands that the React frontend consumes via react-query, and (3) the background health check that fires async on project open and streams results back via Tauri events.

The Wallflower sister project provides a direct blueprint for every IPC pattern needed: the `tauri.ts` wrapper layer (`invoke`-based typed functions), the `TauriEventListener` component for `listen`-based background events, and the react-query `useQuery` + `queryClient.invalidateQueries` pattern for cache-coherent data. Phase 2 should follow Wallflower patterns precisely — no novel patterns needed.

Audio format detection for health check (DETC-02) is handled by reading only the header of each audio file. The `hound` crate (Apache-2.0, MIT-compatible) reads WAV headers without loading sample data — `WavReader::open()` + `.spec()` gives `sample_rate`, `bits_per_sample`, `channels`, and `sample_format`. AIFF files are detected by the 'FORM'...'AIFF' magic bytes; the `aifc` crate (version 0.7.0) reads AIFF/AIFF-C headers. For the format-type detection (is it WAV or AIFF at all?), the `infer` crate (0.19.0) identifies files by magic number rather than extension.

The OT's sample requirements (DETC-02) are: 44.1 kHz sample rate; 16-bit or 24-bit depth (both accepted by MkII); WAV or AIFF format. 48 kHz files play at wrong pitch. Non-WAV/AIFF files are unsupported. The health check must flag 48 kHz as a Warning (wrong speed, device accepts the file but it plays wrong) and non-WAV/AIFF as an Error (device cannot load the file). [CITED: Elektronauts community + OT MkII user manual via ManualsLib]

**Primary recommendation:** Model Phase 2's IPC layer directly on Wallflower — `invoke`-based typed tauri functions for project/bank/sample data, `listen`-based events for background health check progress, react-query for all data fetching and cache invalidation. The health check is the only async background operation; everything else is synchronous read-from-parser-then-return.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Project index population (on card mount) | Rust backend (takoyaki-app) | — | Reads OT file system, writes SQLite; no frontend involvement |
| Project list query + search/filter | Rust backend (takoyaki-app) | Frontend (react-query consumer) | SQLite WHERE query in Rust; frontend sends filter params as invoke args |
| Project detail parsing (banks, parts, tracks) | Rust backend / ot-parser | — | Reads binary bank files; returns structured data over IPC |
| Sample slot reading (Flex/Static) | Rust backend / ot-parser | — | Reads project.work; parser extracts slot table |
| Sample cross-reference (which banks reference each slot) | Rust backend (takoyaki-app) | — | Must read all bank files and correlate — too data-heavy for frontend |
| Health check: file existence | Rust backend (takoyaki-app) | — | fs::exists() with OT volume path; runs async |
| Health check: audio format validation | Rust backend (takoyaki-app) | — | hound + aifc header reads; Rust's ownership makes header-only reads clean |
| Health check progress/results streaming | Rust backend → Frontend via Tauri event | Frontend (listen consumer) | AppHandle.emit() for progress; react-query invalidation on complete |
| Project list view (table, search bar, filters) | Frontend (React/Next.js) | — | Pure presentation; data from react-query |
| Project detail view (tabs, bank grid, sample table) | Frontend (React/Next.js) | — | shadcn Tabs, custom bank grid cell, shadcn Table |
| Navigation state (current view, breadcrumb path) | Frontend zustand store | — | UI state only; no Rust involvement |
| Health check results display (Health tab, inline icons) | Frontend (React/Next.js) | — | Receives results via event listener + react-query cache |

---

## Standard Stack

### Core (inherited from Phase 1 — verified)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tauri | 2.10.3 | Desktop app shell, IPC | Established in Phase 1 |
| tauri-specta | =2.0.0-rc.24 | Auto-generated TypeScript types | Required for type-safe IPC |
| rusqlite | 0.39.0 | SQLite queries for project index | Established in Phase 1 |
| ot-parser | local crate | Parse OT binary files | Core parser from Phase 1 |
| serde / serde_json | 1.0.228 | IPC serialization | Required by Tauri IPC |
| thiserror | 2.0.18 | IPC error types | Established in Phase 1 |
| tracing | 0.1.44 | Structured logging | Established in Phase 1 |
| Next.js | 15.x (16.2.4) | React framework (static export) | Established in Phase 1 |
| React | 19.x | UI framework | Established in Phase 1 |
| Tailwind CSS | 4.x | Styling | Established in Phase 1 |
| zustand | 5.0.12 | UI state (navigation, view selection) | Established in Phase 1 |
| @tanstack/react-query | 5.100.6 | Data fetching + caching for all Tauri commands | Established in Phase 1 |

[VERIFIED: cargo search, npm view, Phase 1 RESEARCH.md, 2026-04-29]

### New in Phase 2

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| hound | 3.5.1 | WAV header reading (sample_rate, bits_per_sample) | Header-only read with no sample data allocation; Apache-2.0 (MIT-compatible) |
| aifc | 0.7.0 | AIFF/AIFF-C header reading | Dedicated AIFF reader; supports compressed variants |
| infer | 0.19.0 | File type detection by magic number | Detects WAV/AIFF by bytes, not extension — handles misnamed files |
| shadcn tabs | via shadcn CLI | Banks/Samples/Health tab switcher | Already specified in UI-SPEC.md; Radix UI backed |
| shadcn breadcrumb | via shadcn CLI | Navigation trail | Already specified in UI-SPEC.md |
| shadcn table | via shadcn CLI | Project list, sample slot tables | Already specified in UI-SPEC.md |
| shadcn collapsible | via shadcn CLI | Expandable slot cross-reference rows | Already specified in UI-SPEC.md |
| shadcn input | via shadcn CLI | Search bar | Already specified in UI-SPEC.md |
| shadcn select | via shadcn CLI | BPM/date filter dropdowns | Already specified in UI-SPEC.md |
| shadcn toggle | via shadcn CLI | Show/hide empty sample slots | Already specified in UI-SPEC.md |
| shadcn progress | via shadcn CLI | Health check scan progress bar | Already specified in UI-SPEC.md |
| shadcn tooltip | via shadcn CLI | Status icon hover, long filename hover | Already specified in UI-SPEC.md |

[VERIFIED: cargo search 2026-04-29, UI-SPEC.md]

### New shadcn components to install

```bash
npx shadcn@latest add tabs breadcrumb table collapsible input select toggle progress tooltip
```

### New Rust dependencies to add to takoyaki-app/Cargo.toml

```toml
hound = "3.5.1"
aifc = "0.7.0"
infer = "0.19.0"
```

---

## Architecture Patterns

### System Architecture Diagram

```
User opens project                OT card mounted (via Phase 1 detection)
         │                                    │
         ▼                                    ▼
┌─────────────────────┐          ┌────────────────────────┐
│  React Frontend     │          │  Rust: on_volume_mount  │
│                     │          │  background task        │
│  useQuery(          │          │  - Walk SETS/**         │
│   "projects",       │◄─invoke──│  - Parse project.work   │
│   list_projects     │          │  - INSERT INTO projects │
│  )                  │          │    (project index)      │
│                     │          └────────────────────────┘
│  filter/search      │
│  params in state    │──invoke──►┌─────────────────────────┐
│                     │           │  list_projects(filter)  │
│  Project rows       │           │  SELECT * FROM projects │
│  rendered           │◄──result──│  WHERE name LIKE ?      │
└────────┬────────────┘           │  AND tempo BETWEEN ? ?  │
         │ click row              └─────────────────────────┘
         ▼
┌─────────────────────┐
│  ProjectDetailView  │
│                     │──invoke──►┌─────────────────────────┐
│  useQuery(          │           │  get_project_detail()   │
│   "project", id     │           │  ot-parser reads:       │
│  )                  │           │  - project.work         │
│                     │◄──result──│  - bank01-16.work       │
│  Tabs: Banks /      │           │  Returns: ProjectDetail │
│  Samples / Health   │           └─────────────────────────┘
│                     │
│  on mount:          │──invoke──►┌─────────────────────────┐
│  trigger health     │           │  run_health_check()     │
│  check command      │           │  Spawns async task:     │
│                     │           │  - fs::exists per slot  │
│  listen(            │           │  - hound WAV headers    │
│  "health-progress") │◄──emit────│  - aifc AIFF headers    │
│                     │           │  - cross-ref tracking   │
│  listen(            │◄──emit────│  app_handle.emit(       │
│  "health-complete") │           │    "health-complete",   │
│                     │           │    results)             │
│  invalidateQuery(   │           └─────────────────────────┘
│  "health", id)      │
└─────────────────────┘
```

### Recommended Project Structure (Phase 2 additions)

```
crates/takoyaki-app/src/
├── commands/
│   ├── mod.rs               # existing
│   ├── device.rs            # existing (Phase 1: volume detection)
│   ├── projects.rs          # NEW: list_projects, get_project_detail
│   ├── samples.rs           # NEW: get_project_samples (Flex + Static slots)
│   └── health.rs            # NEW: run_health_check (async, event-emitting)
├── db/
│   ├── mod.rs               # existing
│   └── projects.rs          # NEW: index_projects(), list_projects(), search queries
├── health/
│   └── mod.rs               # NEW: HealthCheck engine (existence + format validation)
└── ...

src/                         # Next.js frontend
├── app/
│   └── page.tsx             # route switch: list view vs detail view (zustand state)
├── components/
│   ├── projects/
│   │   ├── ProjectTable.tsx        # NEW: compact monospace table (shadcn Table)
│   │   ├── ProjectSearchBar.tsx    # NEW: search input + BPM/date selects
│   │   └── ProjectRow.tsx          # NEW: single row with click handler
│   ├── project-detail/
│   │   ├── ProjectDetailView.tsx   # NEW: detail shell with breadcrumb + header + tabs
│   │   ├── MetadataHeader.tsx      # NEW: always-visible name/BPM/banks/modified strip
│   │   ├── BanksTab.tsx            # NEW: 4×4 bank grid + drill-down panel
│   │   ├── BankGridCell.tsx        # NEW: 48×48 custom cell (filled/empty dot)
│   │   ├── SamplesTab.tsx          # NEW: Flex/Static sections with slot table
│   │   ├── SlotRow.tsx             # NEW: slot row with Collapsible cross-ref detail
│   │   └── HealthTab.tsx           # NEW: grouped severity list + all-clear state
│   └── health/
│       └── HealthSeverityGroup.tsx # NEW: Error/Warning/Info group header + items
├── lib/
│   ├── tauri.ts             # existing pattern — add Phase 2 invoke functions
│   ├── stores/
│   │   └── navigation.ts    # NEW: zustand store for view state (list vs detail, breadcrumb)
│   └── types.ts             # extended with Phase 2 types from tauri-specta bindings
└── ...
```

### Pattern 1: Tauri Command for Project List with Filter

```rust
// Source: Wallflower commands/samples.rs pattern + Phase 1 Pattern 5
// In crates/takoyaki-app/src/commands/projects.rs

#[derive(Debug, serde::Deserialize, specta::Type)]
pub struct ProjectFilter {
    pub name: Option<String>,
    pub bpm_min: Option<u16>,
    pub bpm_max: Option<u16>,
    pub modified_since: Option<String>, // ISO date string
}

#[derive(Debug, serde::Serialize, specta::Type, Clone)]
pub struct ProjectSummary {
    pub id: String,
    pub set_name: String,
    pub project_name: String,
    pub card_path: String,
    pub tempo_bpm: Option<f32>,  // stored as f32 for "120.0 BPM" display
    pub bank_count: Option<u8>,  // banks used out of 16
    pub last_modified: Option<String>,
}

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

[VERIFIED: Wallflower blueprint at /Users/albair/src/wallflower/crates/wallflower-app/src/commands/samples.rs]

### Pattern 2: Background Health Check with Event Emission

```rust
// Source: v2.tauri.app/develop/calling-frontend/ + Wallflower TauriEventListener pattern
// In crates/takoyaki-app/src/commands/health.rs

#[derive(Debug, serde::Serialize, Clone, specta::Type)]
#[serde(tag = "severity", rename_all = "lowercase")]
pub enum HealthIssue {
    Error { slot_type: String, slot_index: u8, path: String, detail: String },
    Warning { slot_type: String, slot_index: u8, filename: String, detail: String },
    Info { slot_type: String, slot_index: u8, filename: String, detail: String },
}

#[derive(Debug, serde::Serialize, Clone, specta::Type)]
pub struct HealthCheckComplete {
    pub project_id: String,
    pub issues: Vec<HealthIssue>,
    pub scanned_at: String,
}

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

    // Spawn background; return immediately so frontend is not blocked
    tauri::async_runtime::spawn(async move {
        use tauri::Emitter;
        let issues = perform_health_check(&project_path).await;
        let result = HealthCheckComplete {
            project_id,
            issues,
            scanned_at: chrono::Utc::now().to_rfc3339(),
        };
        app.emit("health-complete", result).unwrap_or_else(|e| {
            tracing::error!("health-complete emit failed: {e}");
        });
    });

    Ok(())
}
```

### Pattern 3: Audio Format Validation (hound + aifc + infer)

```rust
// Source: docs.rs/hound WavSpec, docs.rs/infer, docs.rs/aifc
// In crates/takoyaki-app/src/health/mod.rs

use hound::WavReader;
use std::path::Path;

#[derive(Debug)]
pub enum AudioSpec {
    Wav { sample_rate: u32, bits_per_sample: u16, channels: u16 },
    Aiff { sample_rate: u32, bits_per_sample: u16, channels: u16 },
    Unknown { detected_type: Option<String> },
}

pub fn read_audio_spec(path: &Path) -> Result<AudioSpec, std::io::Error> {
    // Detect by magic bytes first — not by extension
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
        match aifc::AifcReader::new(&mut std::fs::File::open(path)?) {
            Ok(reader) => {
                let comm = reader.comm().map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "AIFF COMM read error")
                })?;
                Ok(AudioSpec::Aiff {
                    sample_rate: comm.sample_rate as u32,
                    bits_per_sample: comm.sample_size as u16,
                    channels: comm.num_channels as u16,
                })
            }
            Err(_) => Ok(AudioSpec::Unknown { detected_type: kind }),
        }
    } else {
        Ok(AudioSpec::Unknown { detected_type: kind })
    }
}

pub fn check_format_compatibility(spec: &AudioSpec) -> Vec<FormatIssue> {
    let mut issues = vec![];
    match spec {
        AudioSpec::Wav { sample_rate, bits_per_sample, .. } |
        AudioSpec::Aiff { sample_rate, bits_per_sample, .. } => {
            if *sample_rate != 44100 {
                issues.push(FormatIssue::WrongSampleRate(*sample_rate));
            }
            if *bits_per_sample != 16 && *bits_per_sample != 24 {
                issues.push(FormatIssue::WrongBitDepth(*bits_per_sample));
            }
        }
        AudioSpec::Unknown { detected_type } => {
            issues.push(FormatIssue::UnsupportedFormat(
                detected_type.clone().unwrap_or_else(|| "unknown".into())
            ));
        }
    }
    issues
}
```

### Pattern 4: Frontend Event Listener for Health Check

```typescript
// Source: Wallflower TauriEventListener pattern
// In src/components/health/HealthEventListener.tsx

"use client";

import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";

interface HealthIssue {
  severity: "error" | "warning" | "info";
  slot_type: string;
  slot_index: number;
  path?: string;
  filename?: string;
  detail: string;
}

interface HealthCheckComplete {
  project_id: string;
  issues: HealthIssue[];
  scanned_at: string;
}

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
            // Store results in react-query cache keyed by project_id
            queryClient.setQueryData(["health", project_id], event.payload);
          }
        );
        cleanupFns.push(unlisten);
      } catch {
        // Not in Tauri context
      }
    }

    setupListeners();
    return () => cleanupFns.forEach((fn) => fn());
  }, [queryClient]);

  return null;
}
```

### Pattern 5: SQLite Project Search Query

```rust
// Source: rusqlite docs, Wallflower db/mod.rs pattern
// In crates/takoyaki-app/src/db/projects.rs

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
    if let Some(bpm_min) = filter.bpm_min {
        conditions.push("tempo_bpm >= ?");
        params.push(Box::new(bpm_min as i64));
    }
    if let Some(bpm_max) = filter.bpm_max {
        conditions.push("tempo_bpm <= ?");
        params.push(Box::new(bpm_max as i64));
    }
    if let Some(ref since) = filter.modified_since {
        conditions.push("last_modified >= ?");
        params.push(Box::new(since.clone()));
    }

    let sql = format!(
        "SELECT id, set_name, project_name, card_path, tempo_bpm, bank_count, last_modified
         FROM projects WHERE {} ORDER BY last_modified DESC NULLS LAST",
        conditions.join(" AND ")
    );

    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    // ... prepare + query_map
}
```

### Pattern 6: React Navigation State (zustand)

```typescript
// In src/lib/stores/navigation.ts
// Follows Wallflower zustand store pattern

import { create } from "zustand";

type View = "project-list" | "project-detail";

interface NavigationState {
  view: View;
  selectedProjectId: string | null;
  selectedBankIndex: number | null;  // 0-15 when a bank is selected; null otherwise
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
  selectBank: (bankIndex) =>
    set({ selectedBankIndex: bankIndex }),
  setActiveTab: (tab) =>
    set({ activeTab: tab }),
}));
```

### Pattern 7: Project Index Population on Card Mount

```rust
// Triggered when Phase 1 volume detection fires
// In crates/takoyaki-app/src/commands/device.rs (extend existing)

pub async fn index_ot_projects(
    conn: &rusqlite::Connection,
    volume_path: &std::path::Path,
    parser: &ot_parser::OtParser,
) -> Result<usize, AppError> {
    let sets_dir = volume_path.join("SETS");
    let mut count = 0;

    // Walk SETS/{set_name}/{project_name}/project.work
    for set_entry in std::fs::read_dir(&sets_dir)? {
        let set_dir = set_entry?.path();
        if !set_dir.is_dir() { continue; }
        for project_entry in std::fs::read_dir(&set_dir)? {
            let project_dir = project_entry?.path();
            let work_file = project_dir.join("project.work");
            if !work_file.exists() { continue; }

            match parser.parse_project_file(&work_file) {
                Ok(project) => {
                    db::projects::upsert_project(conn, &ProjectRow {
                        id: uuid::Uuid::new_v4().to_string(),
                        set_name: set_dir.file_name().unwrap().to_string_lossy().into(),
                        project_name: project_dir.file_name().unwrap().to_string_lossy().into(),
                        card_path: project_dir.to_string_lossy().into(),
                        tempo_bpm: Some(project.tempo as f32 / 10.0), // OT stores tempo * 10
                        bank_count: Some(count_populated_banks(&project)),
                        last_modified: project_dir.metadata()?.modified().ok()
                            .map(|t| format_modified_time(t)),
                    })?;
                    count += 1;
                }
                Err(e) => {
                    tracing::warn!("Could not parse {}: {e}", work_file.display());
                }
            }
        }
    }
    Ok(count)
}
```

### Anti-Patterns to Avoid

- **Re-parsing binary files on every list view:** The project list (BROW-02) must query the SQLite index, not re-read binary files. Index is populated once on card mount. Only project detail / health check reads binary files on demand.
- **Loading audio sample data for format check:** `hound::WavReader::open()` reads the header immediately; `.spec()` gives you everything needed. Never call `.samples()` — that loads all audio data into memory. DETC-02 is a header-only operation.
- **Blocking health check:** `run_health_check` MUST return `Ok(())` immediately and spawn the check as a background task. Health check can take seconds on large projects (256 slots × file stat + header read). Blocking would freeze the UI.
- **Running health check on every re-render:** Health check is triggered once per project open (in a `useEffect` with the project ID as dependency), not on every query invalidation.
- **Showing raw card paths to users:** Display only the filename portion of sample paths in slot rows. Full path is available in the tooltip and health tab detail. The full `/CF/AUDIO/PROJECTS/...` path is too long for a table cell.
- **Losing navigation state on re-render:** Navigation state (current view, selected project, active tab, selected bank) lives in zustand — not in component state. This ensures tab state persists across re-renders.
- **Tempo display without decimal:** OT stores tempo as integer × 10 (e.g., 1200 = 120.0 BPM). Always divide by 10 and display with one decimal: "120.0 BPM". UI-SPEC.md is explicit on this.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| WAV header reading | Manual byte offset parsing | hound 3.5.1 `WavReader::open().spec()` | Handles all WAV format variants including float and PCM; header read is ~10 microseconds |
| AIFF header reading | RIFF chunk walker | aifc 0.7.0 `AifcReader` | Handles AIFF and AIFF-C; compression variants; COMM chunk extraction |
| File type detection | Extension sniffing | infer 0.19.0 `get_from_path()` | Detects by magic bytes — handles misnamed files (a .wav file with AIFF content) |
| Component primitives | CSS-from-scratch table/tabs/collapsible | shadcn + Radix UI | WAI-ARIA, keyboard nav, Tauri-compatible (no SSR dependency) |
| Background task → UI notification | Polling via invoke | Tauri `AppHandle::emit()` + `listen()` | Push model: no polling; results delivered exactly once; matches Wallflower pattern |
| Navigation state machine | React useState chains | zustand store | Prevents navigation state loss across re-renders; shareable across component tree without prop drilling |
| Project search | SQLite FTS5 virtual table | LIKE query on `projects.project_name` | Project names are short, counts are low (< 100 typical). FTS5 adds complexity for no benefit at this scale. |

**Key insight:** Phase 2 has no novel technical problems. Every challenge (typed IPC, background events, header-only audio reads, indexed search) has a battle-tested solution in the established stack. The value is in assembling these pieces correctly against the OT domain model.

---

## OT Domain Model Reference

This section captures the OT binary data structures as exposed by the parser (from Phase 1 research) as they apply to Phase 2 display needs.

### Data Flow: What Phase 2 Reads from ot-parser

```
project.work → ProjectFile
  ├── name: [u8; 32]         → display as project name
  ├── tempo_bpm: u16         → divide by 10 for display (1200 → 120.0)
  ├── flex_sample_slots[128] → SampleSlot { path: [u8; N], ... }
  └── static_sample_slots[128] → SampleSlot { path: [u8; N], ... }

bank01.work ... bank16.work → BankFile (one per bank, 16 total)
  └── parts[4] → Part
      ├── name: [u8; N]     → bank/part name for display
      └── tracks[8] → Track
          ├── machine_type: u8   → Flex/Static/Thru/Neighbor/Pickup
          └── sample_slot: u8   → index into Flex or Static slot list
```

### Machine Types (OT MkII)
| Code | Name | Sample Source |
|------|------|---------------|
| 0x00 | Thru | No sample slot |
| 0x01 | Flex | Flex slot list |
| 0x02 | Static | Static slot list |
| 0x04 | Neighbor | No sample slot |
| 0x05 | Pickup | Flex slot |

[ASSUMED] — machine type byte values not confirmed from official docs; derived from ot-tools-io struct names and community documentation. Verify during clean-room spec creation in Phase 1.

### OT Sample Format Requirements (DETC-02 implementation reference)
| Property | Required | Flagged As |
|----------|----------|------------|
| Sample rate | 44100 Hz | Warning if ≠ 44100 |
| Bit depth | 16-bit or 24-bit | Warning if neither |
| Format | WAV or AIFF | Error if neither (device cannot load) |
| Channels | Mono or stereo | No flag (both supported) |

[CITED: Elektronauts community threads, OT MkII user manual via ManualsLib, samplestack.app OT MkII specs]

Note: 48 kHz is flagged Warning (not Error) because the OT will load the file but it plays at wrong pitch. Non-WAV/AIFF is flagged Error because the OT cannot load the file at all.

---

## Common Pitfalls

### Pitfall 1: Re-Parsing Binary Files on Project List View

**What goes wrong:** Developer calls `ot-parser` for every project on each list render instead of reading the SQLite index. With 20+ projects this takes several seconds on each navigation.

**Why it happens:** The SQLite project index (Phase 1 schema) must be populated on card mount. If the index population step is skipped or incomplete, devs reach for the parser as the data source for the list.

**How to avoid:** Wave 0 of Phase 2 must include the project indexing command (walk SETS/, parse project.work, upsert to projects table). The list_projects command must be a pure SQLite query — no parser calls.

**Warning signs:** Noticeable delay (> 200ms) when navigating back to the project list from a detail view.

---

### Pitfall 2: Health Check Blocking the UI Thread

**What goes wrong:** `run_health_check` is implemented as a synchronous Tauri command that reads 256 files and their audio headers. The UI freezes while the command executes.

**Why it happens:** Tauri async commands run on the async runtime, but if the implementation blocks (e.g., using blocking file I/O inside a sync handler), the Tauri IPC thread stalls.

**How to avoid:** `run_health_check` returns `Ok(())` immediately after spawning `tauri::async_runtime::spawn(async move { ... })`. The async block does all file I/O and emits `health-complete` when done. See Pattern 2.

**Warning signs:** Health tab shows no progress bar and the whole UI is unresponsive when a project with many slots is opened.

---

### Pitfall 3: OT Tempo Stored as Integer × 10

**What goes wrong:** Tempo 120.0 BPM is stored as `1200` in the binary file. Displaying the raw value shows "1200 BPM" in the project list — obviously wrong, looks like a bug.

**Why it happens:** OT uses fixed-point integer for tempo with one decimal implied. The convention is undocumented in many sources.

**How to avoid:** Parser should expose tempo as `f32` after dividing by 10, or the command layer should do the conversion. Either way, the TypeScript type for `ProjectSummary.tempo_bpm` should be `number` representing the actual BPM value (e.g., 120.0).

**Warning signs:** Project list shows suspiciously large BPM values (1200 instead of 120).

---

### Pitfall 4: Sample Slot Path Encoding

**What goes wrong:** OT sample paths in project.work are stored as null-terminated ASCII relative paths using backslash separators (`\AUDIO\Kicks\kick.wav`). Treating them as native macOS paths without conversion fails file existence checks.

**Why it happens:** OT firmware uses Windows-style backslashes and stores paths relative to the card root. When checking existence on macOS, the path must be: resolve against OT volume mount point + convert backslash to forward slash.

**How to avoid:** The health check path resolution function must: (1) replace `\` with `/`, (2) strip any leading `/` or backslash from the stored path, (3) join with the OT volume mount point path. Verify against a real OT project file during Phase 1 fixture work.

**Warning signs:** All health checks show "missing" for every slot even when files clearly exist on the card.

---

### Pitfall 5: React-Query Cache Stale on Project Switch

**What goes wrong:** User navigates to Project A (health check runs, results cached), then to Project B, then back to Project A — sees stale health data from the earlier run without triggering a new check.

**Why it happens:** react-query caches by query key. If `["health", projectId]` is set from the event listener and never invalidated, the cached result is returned immediately without re-running the check.

**How to avoid:** Health check is triggered in a `useEffect` on `projectId` change. Every time the user opens a project, the check fires — even if cached results exist. Use `queryClient.removeQueries({ queryKey: ["health", projectId] })` before navigating to ensure a fresh run, or accept stale results as a feature (show "scanned at {time}" timestamp).

**Warning signs:** Health tab shows outdated results (e.g., a missing file that's been fixed still shows as missing after re-opening the project).

---

### Pitfall 6: Bank Grid Click Handler on Empty Cells

**What goes wrong:** Clicking an empty bank cell (no patterns in that bank) attempts to load bank detail data, gets an empty response, and renders a confusing empty drill-down panel.

**Why it happens:** Click handlers attached to all cells without distinguishing populated vs empty state.

**How to avoid:** Empty cells have `cursor-default` and no click handler. Only populated cells (dot = filled) get click handlers. This is specified in UI-SPEC.md and must be enforced in `BankGridCell.tsx`.

**Warning signs:** Clicking empty cells in the bank grid causes a flash of empty content or a loading state.

---

## Code Examples

### SQLite Schema Addition (Phase 2 — no schema changes needed)

The Phase 1 schema's `projects` table already contains all fields needed for BROW-02 and MGMT-04:

```sql
-- From Phase 1 V1__initial_schema.sql (already defined)
-- Phase 2 adds no new tables — it queries and populates the existing projects table

-- Index already present:
CREATE INDEX idx_projects_card_path ON projects(card_path);

-- Phase 2 will query:
SELECT id, set_name, project_name, card_path, tempo_bpm, bank_count, last_modified
FROM projects
WHERE project_name LIKE '%query%'  -- MGMT-04 text search
  AND tempo_bpm BETWEEN 90 AND 120  -- MGMT-04 BPM filter
  AND last_modified >= '2026-01-01' -- MGMT-04 date filter
ORDER BY last_modified DESC;

-- For performance on text search, add this index in a new migration:
-- CREATE INDEX idx_projects_name ON projects(project_name COLLATE NOCASE);
```

### Tauri IPC Wrapper (tauri.ts additions)

```typescript
// Additions to src/lib/tauri.ts — follow existing Wallflower pattern
import { invoke } from "@tauri-apps/api/core";
import type {
  ProjectSummary,
  ProjectFilter,
  ProjectDetail,
  SampleSlot,
  HealthCheckResult,
} from "./types";

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

### React-Query Project List Hook

```typescript
// In src/components/projects/ProjectTable.tsx (or custom hook)
import { useQuery } from "@tanstack/react-query";
import { listProjects } from "@/lib/tauri";
import { useProjectFilter } from "@/lib/stores/filters";

export function useProjectList() {
  const filter = useProjectFilter();

  return useQuery({
    queryKey: ["projects", filter],
    queryFn: () => listProjects(filter),
    // No staleTime — re-fetch when filter changes or on mount
    // Projects list should refresh when user returns to list view
  });
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Polling Tauri commands for async results | `AppHandle::emit()` + `listen()` event model | Tauri v2 (2024) | No polling; push model; results arrive exactly once |
| tauri-specta v1 manual type export | v2 Builder + `collect_commands!` macro | tauri-specta v2-rc | Types auto-regenerate on build; no manual sync |
| React Context for global state | zustand 5.x stores | stable (2024) | Smaller bundles, no provider wrapping, selector-based re-render |
| react-query v4 | TanStack Query v5 (react-query) | 2024 | `useQuery` options restructured; `status` values changed; `isLoading` → `isPending` for no-data state |

**Deprecated/outdated in this context:**
- `@tanstack/react-query` v4 `isLoading` flag: In v5, `isLoading` means "fetching AND no cached data"; use `isPending` for "no data yet". This affects skeleton states.
- Tauri `window.__TAURI__.invoke()` global: Replaced by `import { invoke } from "@tauri-apps/api/core"`. The global still exists for debugging but module import is canonical.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | OT machine type byte codes (Thru=0x00, Flex=0x01, Static=0x02, Neighbor=0x04, Pickup=0x05) are accurate | OT Domain Model | Bank drill-down shows wrong machine type labels; cross-reference for DETC-03 is wrong |
| A2 | OT stores tempo as integer × 10 in project.work (e.g., 1200 = 120.0 BPM) | OT Domain Model, Pitfall 3 | Tempo display is off by 10x in both directions |
| A3 | OT sample paths in project.work use backslash separators and are relative to the card root | OT Domain Model, Pitfall 4 | Health check DETC-01 always shows "missing" even when files exist |
| A4 | aifc crate's `AifcReader::comm()` method returns sample_rate as a numeric type that can be cast to u32 | Pattern 3 | AIFF header reading code does not compile; need alternative AIFF approach |
| A5 | 24-bit WAV files are accepted by OT MkII (only 48 kHz triggers playback issues, not 24-bit depth) | OT Sample Format Requirements | Health check emits incorrect warnings for valid 24-bit samples |
| A6 | The projects table from Phase 1 schema is sufficient for Phase 2 queries (no additional columns needed) | SQLite Schema, Code Examples | A schema migration is needed before Phase 2 list commands work |

**If this table is empty:** All claims in this research were verified or cited — no user confirmation needed.

---

## Open Questions (RESOLVED — guarded by Plan 01 assumption guards; definitive validation pending Phase 1 OT binary fixtures)

1. **OT tempo encoding**
   - What we know: OT BPM range is 30–300 BPM. If stored as integer, integer × 10 gives range 300–3000, which requires u16. ot-tools-io parses a u16 tempo field.
   - What's unclear: Is the scale factor exactly 10 (e.g., 120.0 BPM → 1200) or some other factor?
   - Recommendation: Verify during Phase 1 clean-room spec creation using a known project file with a known tempo. Set up one test project on the OT at exactly 120.0 BPM and hex-dump the project.work to find the tempo bytes.

2. **OT sample path encoding in project.work**
   - What we know: OT uses FAT32 with Windows-style paths. Sample paths are stored relative to the card root.
   - What's unclear: Are paths null-terminated? Fixed length or variable? Backslash or forward slash separator? What encoding (ASCII? UTF-8? Latin-1)?
   - Recommendation: Extract from ot-tools-io source study (format study, not code copy) during Phase 1 clean-room spec work. This is critical for DETC-01 (file existence check) — get this right before building the health check.

3. **Project.work vs project.strd: which to read for the project index?**
   - What we know: `.work` is autosaved; `.strd` is last manual save. Both exist per project. They may contain different data if the user autosaved after their last manual save.
   - What's unclear: Should the project index reflect the autosave (.work) or the last manual save (.strd)? Does this affect what the health check should validate?
   - Recommendation: Read `.work` for the project index — it reflects the most current state. Document this choice in the parser layer. If a user has only a `.strd` and no `.work`, fall back to `.strd`. Verify assumption A5 (from Phase 1 research) during Phase 1 fixture work.

4. **How many patterns are "populated" in a bank?**
   - What we know: Each bank has 16 patterns. "Populated" for the bank grid (D-05, D-06) means the bank has any non-empty patterns.
   - What's unclear: What does the bank binary file store that indicates whether a pattern is non-empty? A step count? An explicit flag?
   - Recommendation: Research during Phase 1 clean-room spec work. The display in Phase 2 only needs "populated or not" per bank — a single boolean per bank. If the binary has no explicit flag, treat any bank file with a parseable non-zero pattern as populated.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust / Cargo | ot-parser + Tauri backend | ✓ | rustc 1.95.0 | — |
| Node.js | Frontend build | ✓ | v24.6.0 | — |
| npm | Package management | ✓ | 11.5.1 | — |
| Tauri CLI | App build | ✓ | tauri-cli 2.10.1 | — |
| OT binary fixture files | Verify tempo/path encoding | ✗ (must be provided) | — | Use hex editor on any real OT project.work |
| Real OT CF card (for health check integration test) | DETC-01 file existence verification | ✗ (not verified) | — | Mock OT directory structure on any FAT32 volume |

**Missing with no fallback:** None — all dependencies for code development are available.

**Missing with fallback:**
- OT project fixtures: Required to verify tempo encoding and sample path format (Open Questions 1–2). User's own OT project files serve this role; they should be copied into `tests/fixtures/` as part of Phase 1 completion.
- Real CF card for health check integration test: A mock directory tree (SETS/{name}/{project}/project.work with sample files in AUDIO/) can validate the logic without a real card.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness + cargo test |
| Config file | none — `cargo test` works directly |
| Quick run command | `cargo test -p takoyaki-app health` |
| Full suite command | `cargo test --workspace` |
| Frontend | No automated frontend tests in Phase 2 — manual verification via `cargo tauri dev` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| BROW-02 | project index populated; list_projects returns rows | unit | `cargo test -p takoyaki-app test_list_projects` | ❌ Wave 0 |
| BROW-02 | list_projects with name filter returns matching subset | unit | `cargo test -p takoyaki-app test_list_projects_filter_name` | ❌ Wave 0 |
| BROW-03 | get_project_banks returns 16 entries with populated flags | unit | `cargo test -p takoyaki-app test_get_project_banks` | ❌ Wave 0 |
| BROW-04 | get_project_samples returns Flex[128] + Static[128] | unit | `cargo test -p takoyaki-app test_get_project_samples` | ❌ Wave 0 |
| BROW-05 | get_project_detail returns tempo, bank names, part names, machine types | unit | `cargo test -p takoyaki-app test_get_project_detail` | ❌ Wave 0 |
| DETC-01 | health check: missing file → Error issue in results | unit | `cargo test -p takoyaki-app test_health_missing_file` | ❌ Wave 0 |
| DETC-02 | health check: 48 kHz WAV → Warning issue in results | unit | `cargo test -p takoyaki-app test_health_wrong_sample_rate` | ❌ Wave 0 |
| DETC-02 | health check: non-WAV/AIFF file → Error issue in results | unit | `cargo test -p takoyaki-app test_health_unsupported_format` | ❌ Wave 0 |
| DETC-03 | health check: slot with no track references → Info issue | unit | `cargo test -p takoyaki-app test_health_unused_sample` | ❌ Wave 0 |
| MGMT-04 | list_projects with BPM range filter returns correct subset | unit | `cargo test -p takoyaki-app test_list_projects_filter_bpm` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p takoyaki-app health`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full workspace tests green + manual walkthrough of all 6 success criteria before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `crates/takoyaki-app/tests/projects.rs` — covers BROW-02, MGMT-04
- [ ] `crates/takoyaki-app/tests/project_detail.rs` — covers BROW-03, BROW-04, BROW-05
- [ ] `crates/takoyaki-app/tests/health_check.rs` — covers DETC-01, DETC-02, DETC-03
- [ ] `tests/fixtures/mock_ot_volume/` — mock OT card directory structure for health check tests (SETS/, AUDIO/ with sample WAV files at various sample rates)

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Desktop app, no user auth |
| V3 Session Management | no | No sessions |
| V4 Access Control | partial | Tauri capabilities — read-only; no write commands in this phase |
| V5 Input Validation | yes | All project IDs and filter strings from frontend must be validated; path traversal prevention on health check |
| V6 Cryptography | no | No encryption |

### Known Threat Patterns for Phase 2

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal via sample slot path from OT binary | Tampering | Resolve slot paths against OT volume mount point; reject any path that escapes the volume root via `canonicalize()` comparison |
| Malicious project.work with crafted path | Spoofing | Parser rejects paths outside the OT volume; health check validates resolved paths are under the mount point |
| Filter injection via name search string | Tampering | Use parameterized queries (rusqlite `params![]` macro) — never string interpolation for user-supplied filter values |
| Health check on path supplied from frontend | Tampering | Frontend supplies project_id (opaque UUID), not a path; backend resolves path from DB — user never controls raw file paths |

**Phase 2 threat profile is LOW:** No write operations; no auth; no network. The main surface is the SQLite query layer (filter injection) and the health check path resolution. Both are straightforward with parameterized queries and path validation.

---

## Project Constraints (from CLAUDE.md)

| Directive | Category | Impact on Phase 2 |
|-----------|----------|--------------------|
| Tech stack: Tauri v2 + Rust + React/Next.js | Required | Non-negotiable; all Phase 2 work follows this stack |
| Database: SQLite via rusqlite (bundled) | Required | MGMT-04 search/filter is a SQLite query; no FTS5 complexity needed |
| OT format: clean-room Rust, no GPL | Required | Health check reads audio files (WAV/AIFF) not OT binary — hound/aifc are clean MIT/Apache crates |
| Data safety: no writes to OT card | Core constraint | Phase 2 is strictly read-only; no write commands are implemented in this phase |
| File access: USB disk mode only | Architecture | OT card is a mounted volume; paths resolve against mount point |
| Licensing: MIT; no GPL dependencies | Required | hound (Apache-2.0 — compatible), aifc (Apache-2.0), infer (MIT) — all compatible |
| Testing: full test coverage | Required | All 8 requirements have corresponding test stubs in Wave 0 gaps |
| GSD workflow enforcement | Process | All file changes through GSD commands |

---

## Sources

### Primary (HIGH confidence)
- [Wallflower codebase](file:///Users/albair/src/wallflower) — IPC patterns (tauri.ts, TauriEventListener, commands/samples.rs), react-query usage, zustand stores [VERIFIED: Read tool]
- [Phase 1 RESEARCH.md](file:///Users/albair/src/takoyaki/.planning/phases/01-foundation/01-RESEARCH.md) — Standard stack, SQLite schema, binrw patterns, Wallflower blueprint reference [VERIFIED: Read tool]
- [Phase 2 UI-SPEC.md](file:///Users/albair/src/takoyaki/.planning/phases/02-read-only-browser/02-UI-SPEC.md) — Full component inventory, layout contracts, shadcn component list [VERIFIED: Read tool]
- [docs.rs/hound WavSpec](https://docs.rs/hound/latest/hound/struct.WavSpec.html) — sample_rate, bits_per_sample, channels, sample_format fields [VERIFIED: WebFetch]
- [v2.tauri.app/develop/calling-frontend/](https://v2.tauri.app/develop/calling-frontend/) — AppHandle emit pattern, async task spawn [VERIFIED: WebFetch]
- hound 3.5.1 — cargo search 2026-04-29 [VERIFIED]
- aifc 0.7.0 — cargo search 2026-04-29 [VERIFIED]
- infer 0.19.0 — cargo search 2026-04-29 [VERIFIED]
- @tanstack/react-query 5.100.6 — npm view 2026-04-29 [VERIFIED]
- zustand 5.0.12 — npm view 2026-04-29 [VERIFIED]

### Secondary (MEDIUM confidence)
- [Elektronauts OT sample rate thread](https://www.elektronauts.com/t/octatrack-sample-rate-stuff-importing-stems/177037) — confirms 48 kHz plays at wrong pitch
- [OT MkII ManualsLib page 108](https://www.manualslib.com/manual/1309767/Elektron-Octatrack-Mkii.html?page=108) — "preparing samples" section [CITED]
- [samplestack.app OT MkII specs](https://samplestack.app/instruments/octatrack-mkii/) — confirmed 44.1 kHz, 16/24-bit, WAV/AIFF [CITED]
- OT MkII user manual (general) — WAV/AIFF format requirement, Flex/Static 128-slot structure [CITED]

### Tertiary (LOW confidence)
- Machine type byte codes (Thru/Flex/Static values) — derived from ot-tools-io struct names; not from official documentation. Marked [ASSUMED] in Assumptions Log.

---

## Metadata

**Confidence breakdown:**
- Standard stack (inherited): HIGH — all versions verified via cargo search/npm view; Wallflower blueprint confirmed
- New crates (hound, aifc, infer): HIGH — cargo search confirmed versions; hound API verified via docs.rs
- IPC patterns: HIGH — Wallflower provides identical working implementation
- OT sample format requirements: MEDIUM — community-confirmed; official manual excerpts partial
- OT binary field encoding (tempo, paths, machine types): LOW/ASSUMED — Phase 1 clean-room spec work must confirm these before Phase 2 health check can be implemented correctly

**Research date:** 2026-04-29
**Valid until:** 2026-05-29 (stable stack; hound/aifc/infer are stable crates; OT format knowledge stable indefinitely)
