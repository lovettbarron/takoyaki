---
phase: 02-read-only-browser
plan: "04"
subsystem: frontend
tags: [react, nextjs, typescript, zustand, react-query, shadcn, base-ui, project-detail, bank-grid, sample-slots]

# Dependency graph
requires:
  - phase: 02-read-only-browser
    plan: "01"
    provides: get_project_detail, get_project_samples, run_health_check Tauri commands
  - phase: 02-read-only-browser
    plan: "03"
    provides: useNavigationStore (selectedProjectId, activeTab, selectBank), getProjectDetail/getProjectSamples IPC wrappers, page.tsx view routing

provides:
  - src/components/project-detail/ProjectDetailView.tsx with breadcrumb, metadata header, three-tab layout, skeleton loading, auto health check
  - src/components/project-detail/MetadataHeader.tsx with project name (Display size), tempo toFixed(1), bank count, last modified
  - src/components/project-detail/BanksTab.tsx with 4x4 bank grid, drill-down panel for parts/tracks
  - src/components/project-detail/BankGridCell.tsx with populated/empty dot states, selected accent border, aria attributes
  - src/components/project-detail/SamplesTab.tsx with FLEX/STATIC sections, show/hide empty toggle, cross-reference map
  - src/components/project-detail/SlotRow.tsx with collapsible cross-reference expansion, status icons with tooltips
  - src/app/page.tsx updated to render ProjectDetailView for project-detail view (Plan 03 placeholder replaced)

affects:
  - 02-05-PLAN.md (HealthEventListener mounts in page.tsx; HealthTab replaces the "Health tab (Plan 05)" placeholder in ProjectDetailView)

# Tech tracking
tech-stack:
  added:
    - shadcn/ui tabs component (src/components/ui/tabs.tsx)
    - shadcn/ui breadcrumb component (src/components/ui/breadcrumb.tsx)
    - shadcn/ui collapsible component (src/components/ui/collapsible.tsx) — backed by @base-ui/react/collapsible
    - shadcn/ui toggle component (src/components/ui/toggle.tsx)
  patterns:
    - "base-ui Collapsible does not support asChild prop — use CollapsibleTrigger as a direct wrapper div/button, not as a render prop"
    - "base-ui Tooltip does not support asChild prop on TooltipTrigger — wrap the target element as children directly"
    - "SlotRow uses flex div rows instead of TableRow/TableCell to avoid asChild incompatibility with base-ui Collapsible"
    - "SamplesTab builds cross-ref map from cached ['project', projectId] react-query data — no extra IPC call"
    - "BankGridCell: onClick only attached when bank is populated; disabled={!populated} prevents keyboard activation of empty cells"

key-files:
  created:
    - src/components/project-detail/ProjectDetailView.tsx
    - src/components/project-detail/MetadataHeader.tsx
    - src/components/project-detail/BanksTab.tsx
    - src/components/project-detail/BankGridCell.tsx
    - src/components/project-detail/SamplesTab.tsx
    - src/components/project-detail/SlotRow.tsx
    - src/components/ui/tabs.tsx
    - src/components/ui/breadcrumb.tsx
    - src/components/ui/collapsible.tsx
    - src/components/ui/toggle.tsx
  modified:
    - src/app/page.tsx

key-decisions:
  - "base-ui Collapsible/Tooltip components do not support asChild — switched SlotRow from TableRow-based to flex div layout to avoid the incompatibility"
  - "SamplesTab builds cross-reference map from already-cached ProjectDetail query data rather than a new IPC call — zero extra round-trips"
  - "SlotRow uses local useState for open/closed — collapsible state is per-row, not hoisted to SamplesTab"
  - "Health tab is a placeholder div in ProjectDetailView — wired to Plan 05"

# Metrics
duration: 4min
completed: 2026-04-30
---

# Phase 02 Plan 04: Project Detail View Summary

**Project detail view with breadcrumb navigation, metadata header (Display-size name, tempo toFixed(1), bank count, modified date), three-tab layout (Banks/Samples/Health), 4x4 bank grid matching OT hardware layout, parts/tracks drill-down, and sample slot tables with collapsible cross-reference expansion**

## Performance

- **Duration:** 4 min (201 seconds)
- **Started:** 2026-04-30T06:06:06Z
- **Completed:** 2026-04-30T06:09:27Z
- **Tasks:** 2
- **Files modified:** 11 (10 created, 1 modified)

## Accomplishments

- Created `MetadataHeader.tsx` — compact always-visible strip with project name (Display/24px monospace semibold), tempo as `toFixed(1) BPM`, bank count, and `Modified {date}` — all separated by middle-dot
- Created `ProjectDetailView.tsx` — breadcrumb (`Projects > project_name > Bank N` when bank selected), MetadataHeader, shadcn Tabs for Banks/Samples/Health, skeleton loading states for metadata and tab content, auto-triggered `runHealthCheck` on project open via `useEffect`, Health tab badge from react-query cache `["health", projectId]`
- Updated `src/app/page.tsx` to replace the Plan 03 placeholder div with `<ProjectDetailView />` when `view === "project-detail"`
- Created `BankGridCell.tsx` — 48x48px button with populated/empty dot (filled `bg-foreground` or outlined `border border-muted-foreground`), bank number below dot, selected state with accent border, `disabled={!populated}`, `aria-label` and `aria-pressed` for accessibility
- Created `BanksTab.tsx` — 16-cell 4x4 grid (`grid grid-cols-4 gap-2`), padding to 16 entries, bank count summary (`N of 16 banks used`), hint text when no bank selected, drill-down panel with bank heading, 4-column parts layout, 8 tracks per part showing track index / machine type / sample filename
- Created `SlotRow.tsx` — collapsible row using base-ui Collapsible, 3-digit zero-padded slot number (`padStart(3, "0")`), filename with full-path tooltip, sample rate formatter (44100→44.1k, 48000→48k), `CircleCheck`/`CircleX`/`CircleAlert` Lucide icons with color-coded tooltips, cross-reference expansion panel
- Created `SamplesTab.tsx` — FLEX SAMPLES and STATIC SAMPLES sections with matching column headers (#/FILENAME/RATE/STATUS), Show/Hide empty slots Toggle (default off), cross-reference map built from already-cached ProjectDetail banks data, "No samples assigned" empty state
- Installed shadcn tabs, breadcrumb, collapsible, and toggle components

## Task Commits

Each task was committed atomically:

1. **Task 1: ProjectDetailView shell with breadcrumb, metadata header, and tabs** — `4fc4522` (feat)
2. **Task 2: Banks tab (4x4 grid + drill-down) and Samples tab (slot tables + cross-reference)** — `9474dd8` (feat)

**Plan metadata:** *(this commit)*

## Files Created/Modified

- `src/components/project-detail/ProjectDetailView.tsx` — project detail shell
- `src/components/project-detail/MetadataHeader.tsx` — metadata header strip
- `src/components/project-detail/BanksTab.tsx` — 4x4 grid with drill-down
- `src/components/project-detail/BankGridCell.tsx` — individual bank grid cell
- `src/components/project-detail/SamplesTab.tsx` — FLEX/STATIC sample tables with toggle
- `src/components/project-detail/SlotRow.tsx` — collapsible slot row with status icons
- `src/components/ui/tabs.tsx` — shadcn tabs (installed)
- `src/components/ui/breadcrumb.tsx` — shadcn breadcrumb (installed)
- `src/components/ui/collapsible.tsx` — shadcn collapsible (installed, base-ui backed)
- `src/components/ui/toggle.tsx` — shadcn toggle (installed)
- `src/app/page.tsx` — view routing updated (Plan 03 placeholder replaced)

## Decisions Made

- base-ui Collapsible and Tooltip do not support `asChild` — switched SlotRow from shadcn TableRow to a flex div layout so CollapsibleTrigger can wrap the row directly without asChild
- Cross-reference map built from the already-cached `["project", projectId]` query data — no additional IPC call, zero round-trip cost
- Slot row expand/collapse state is local per-row (`useState`) — no need to hoist to SamplesTab

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] base-ui components do not support `asChild` prop**
- **Found during:** Task 2 TypeScript compilation
- **Issue:** `SlotRow.tsx` used `asChild` on `CollapsibleTrigger`, `CollapsibleContent`, and `TooltipTrigger`, but `@base-ui/react` components (used by this project's shadcn installation) do not expose `asChild`. TypeScript reported 7 errors for this.
- **Fix:** Rewrote `SlotRow.tsx` to use a flex div layout instead of `<TableRow>`. `CollapsibleTrigger` wraps the entire row div directly (no asChild needed). `SamplesTab.tsx` updated to use the new div-based `SlotRow` with a matching div column header row instead of `<Table>`.
- **Files modified:** `src/components/project-detail/SlotRow.tsx`, `src/components/project-detail/SamplesTab.tsx`
- **Commit:** included in `9474dd8`

## Known Stubs

- `ProjectDetailView.tsx` Health tab content: renders `<div>Health tab (Plan 05)</div>` placeholder. Health badge reads from `["health", projectId]` react-query cache but the HealthEventListener (Plan 05) is not yet mounted, so the badge will always show 0 until Plan 05 is complete.
- All project/sample data is from the Phase 01/02 Rust backend — tempo_bpm and bank data are null in stub Rust implementations until the OT binary parser work completes. UI correctly shows "--" for null values.

## Threat Surface Scan

No new threat surface beyond the plan's threat model (T-02-10 and T-02-11 accepted). SlotRow renders `slot.full_path` in a tooltip — local filesystem path on user's own machine, desktop app, no network exposure. All data comes from the Rust backend (trusted source per T-02-11). No user-editable fields in Phase 2.

## Self-Check: PASSED

Files exist:
- src/components/project-detail/ProjectDetailView.tsx: FOUND
- src/components/project-detail/MetadataHeader.tsx: FOUND
- src/components/project-detail/BanksTab.tsx: FOUND
- src/components/project-detail/BankGridCell.tsx: FOUND
- src/components/project-detail/SamplesTab.tsx: FOUND
- src/components/project-detail/SlotRow.tsx: FOUND
- src/app/page.tsx: FOUND (modified)

Commits exist:
- 4fc4522: FOUND (Task 1)
- 9474dd8: FOUND (Task 2)

TypeScript: PASS (npx tsc --noEmit — no errors)

---
*Phase: 02-read-only-browser*
*Completed: 2026-04-30*
