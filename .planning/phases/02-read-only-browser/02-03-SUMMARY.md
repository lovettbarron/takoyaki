---
phase: 02-read-only-browser
plan: "03"
subsystem: frontend
tags: [react, nextjs, typescript, zustand, react-query, shadcn, tauri-ipc, project-list]

# Dependency graph
requires:
  - phase: 02-read-only-browser
    plan: "01"
    provides: list_projects, get_project_detail, get_project_banks, get_project_samples, run_health_check Tauri commands
  - phase: 01-foundation
    provides: AppState, zustand, react-query, shadcn scaffold, page.tsx layout chrome

provides:
  - src/lib/types.ts with all Phase 2 TypeScript types matching Rust specta structs
  - src/lib/tauri.ts extended with five IPC wrappers (listProjects, getProjectDetail, getProjectBanks, getProjectSamples, runHealthCheck)
  - src/lib/stores/navigation.ts with useNavigationStore (view/selectedProjectId/selectedBankIndex/activeTab) and useFilterStore (filter/hasActiveFilters)
  - src/components/projects/ProjectTable.tsx with useQuery, skeleton loading, empty states, client-side sort
  - src/components/projects/ProjectSearchBar.tsx with debounced name search, BPM range dropdown, date filter dropdown
  - src/components/projects/ProjectRow.tsx with keyboard navigation and toFixed(1) tempo display
  - src/app/page.tsx updated to render ProjectTable or project-detail placeholder based on navigation store view

affects:
  - 02-04-PLAN.md (ProjectDetailView replaces the placeholder; uses useNavigationStore.selectedProjectId)
  - 02-05-PLAN.md (HealthEventListener mounts in page.tsx comment placeholder location)

# Tech tracking
tech-stack:
  added:
    - shadcn/ui table component (src/components/ui/table.tsx)
    - shadcn/ui select component (src/components/ui/select.tsx)
  patterns:
    - "useQuery with queryKey: ['projects', filter] — filter object in key ensures re-fetch on any filter change"
    - "useFilterStore setFilter deletes undefined/null/empty-string keys to keep filter object lean for query key comparison"
    - "shadcn Select onValueChange typed as (string | null) — handle null as 'any' case"
    - "Client-side sort applied after react-query data arrives — SQLite always returns last_modified DESC; resort for other columns without a new IPC call"
    - "150ms debounce on search input via useRef<ReturnType<typeof setTimeout>>"
    - "Navigation view state in zustand persists across re-renders — no prop drilling"

key-files:
  created:
    - src/lib/types.ts
    - src/lib/stores/navigation.ts
    - src/components/projects/ProjectTable.tsx
    - src/components/projects/ProjectSearchBar.tsx
    - src/components/projects/ProjectRow.tsx
    - src/components/ui/table.tsx
    - src/components/ui/select.tsx
  modified:
    - src/lib/tauri.ts
    - src/app/page.tsx

key-decisions:
  - "shadcn Select onValueChange signature is (string | null) not (string) — handlers accept null and treat as 'any'"
  - "Client-side sort rather than additional IPC calls for non-default columns — keeps list query simple and avoids round-trips for UI-only sort preference"
  - "useFilterStore removes empty/undefined keys from filter object so react-query sees {} (no filters) vs {name: 'foo'} as distinct cache keys"
  - "Date select controlled value tracked as local string value separate from filter store — avoids needing to reverse-engineer date strings back to dropdown labels"

# Metrics
duration: 3min
completed: 2026-04-30
---

# Phase 02 Plan 03: Project List View Summary

**Project list view with TypeScript IPC layer, zustand navigation/filter stores, debounced search bar, BPM and date filter dropdowns, sortable table with skeleton loading and correct empty states — all wired to the Phase 01 list_projects Tauri command**

## Performance

- **Duration:** 3 min
- **Started:** 2026-04-30T07:00:00Z
- **Completed:** 2026-04-30T07:03:00Z
- **Tasks:** 2
- **Files modified:** 9 (7 created, 2 modified)

## Accomplishments

- Created `src/lib/types.ts` with all Phase 2 TypeScript types: ProjectFilter, ProjectSummary, ProjectDetail, BankDetail, PartDetail, TrackDetail, SampleSlotResponse, SampleSlot, HealthIssue, HealthCheckComplete
- Extended `src/lib/tauri.ts` with five typed IPC wrappers: listProjects, getProjectDetail, getProjectBanks, getProjectSamples, runHealthCheck
- Created `src/lib/stores/navigation.ts` with useNavigationStore (view state, selectedProjectId, selectedBankIndex, activeTab, navigate actions) and useFilterStore (ProjectFilter state, setFilter with auto-cleanup of empty keys, clearFilter)
- Created `ProjectSearchBar.tsx` with 150ms-debounced name search, BPM range Select (60-90 / 90-120 / 120-140 / 140+), date range Select (7/30/90 days), result count display when active, Escape to clear, role="search" + aria-label, screen reader live region
- Created `ProjectRow.tsx` with h-9 row height, cursor-pointer, hover bg, tabIndex=0, Enter/Space keyboard navigation to project detail, toFixed(1) tempo display, four cells (NAME/BPM/BANKS/MODIFIED)
- Created `ProjectTable.tsx` with useQuery(["projects", filter]), 5-row skeleton loading, client-side sort toggle with ChevronUp/Down indicators, empty states matching UI-SPEC copy exactly
- Updated `page.tsx` to import useNavigationStore, render ProjectTable for project-list view, placeholder div for project-detail (Plan 04), HealthEventListener comment placeholder (Plan 05)
- Installed shadcn table and select components

## Task Commits

Each task was committed atomically:

1. **Task 1: Create TypeScript types, IPC wrappers, and navigation store** — `bc6ad77` (feat)
2. **Task 2: Build project list view with search bar, filter dropdowns, and table** — `a678ebf` (feat)

**Plan metadata:** *(this commit)*

## Files Created/Modified

- `src/lib/types.ts` — all Phase 2 TypeScript types matching Rust specta structs
- `src/lib/tauri.ts` — extended with five Phase 2 IPC wrappers
- `src/lib/stores/navigation.ts` — useNavigationStore + useFilterStore
- `src/components/projects/ProjectTable.tsx` — project list table with react-query, skeleton, empty states, sort
- `src/components/projects/ProjectSearchBar.tsx` — search + BPM/date filter bar
- `src/components/projects/ProjectRow.tsx` — single project row with keyboard navigation
- `src/components/ui/table.tsx` — shadcn table component (installed)
- `src/components/ui/select.tsx` — shadcn select component (installed)
- `src/app/page.tsx` — view routing via useNavigationStore

## Decisions Made

- shadcn Select `onValueChange` signature is `(string | null)` not `(string)` — handlers treat null as "any"
- Client-side sort rather than server-side for non-default columns — avoids extra IPC round-trips for a UI preference
- `useFilterStore.setFilter` removes keys with undefined/null/"" values so the filter object stays clean for react-query cache key comparison
- Date select value state not derived from filter store — avoids reverse-engineering ISO date strings back to dropdown option labels

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed shadcn Select onValueChange type incompatibility**
- **Found during:** Task 2 TypeScript compilation
- **Issue:** shadcn Select's `onValueChange` expects `(value: string | null) => void` but handler functions declared `(value: string) => void`
- **Fix:** Updated `handleBpmChange` and `handleDateChange` signatures to accept `string | null`, with null treated as "any" (clear filter)
- **Files modified:** `src/components/projects/ProjectSearchBar.tsx`
- **Commit:** included in `a678ebf`

None beyond the type fix above — plan executed as written.

## Known Stubs

- `page.tsx` project-detail branch: renders a placeholder div ("Project detail view — coming in Plan 04"). NavigationStore `view` is set correctly by ProjectRow clicks; the detail component will be wired in Plan 04.
- All data shown in ProjectTable comes from the Phase 01 SQLite index — tempo_bpm and bank_count are null in stubs until Phase 01 OT binary parser work completes. The UI correctly shows "--" for null tempo and "0/16" for null bank_count.

## Threat Surface Scan

No new threat surface beyond the plan's threat model (T-02-08 and T-02-09 accepted). Frontend accepts user filter input but passes it as typed ProjectFilter to Rust backend — no SQL or path operations in the frontend. card_path is available in ProjectSummary but is not rendered in any table cell.

## Self-Check: PASSED

Files exist:
- src/lib/types.ts: FOUND
- src/lib/tauri.ts: FOUND (modified)
- src/lib/stores/navigation.ts: FOUND
- src/components/projects/ProjectTable.tsx: FOUND
- src/components/projects/ProjectSearchBar.tsx: FOUND
- src/components/projects/ProjectRow.tsx: FOUND
- src/app/page.tsx: FOUND (modified)

Commits exist:
- bc6ad77: FOUND (Task 1)
- a678ebf: FOUND (Task 2)

TypeScript: PASS (npx tsc --noEmit — no errors)

---
*Phase: 02-read-only-browser*
*Completed: 2026-04-30*
