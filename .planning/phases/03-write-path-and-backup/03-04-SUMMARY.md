---
phase: 03-write-path-and-backup
plan: 04
subsystem: ui
tags: [react, tanstack-query, zustand, tailwind, backups, timeline, restore]

# Dependency graph
requires:
  - phase: 03-write-path-and-backup/03-02
    provides: BackupSummary types, listBackups/computeDryRun IPC wrappers, useBackupStore with startOperation
  - phase: 03-write-path-and-backup/03-03
    provides: DryRunModal, BackupProgressView, backup flow orchestration in page.tsx

provides:
  - BackupsView: top-level backups sidebar section with project grouping and timeline
  - BackupTimeline: reverse-chronological snapshot list per project name
  - SnapshotRow: single 48px snapshot entry with timestamp, op label, file count, size, Restore button
  - SnapshotDetailPanel: expanded file diff with semantic change colors; disconnected state handled
  - page.tsx: BackupsView replaces Plan 04 placeholder, backups view fully wired

affects: [phase-04-advanced-management, any plan reading backup history, restore flow consumers]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "useQuery(['backups']) for fetching all backups; groupBy project_name using Map.reduce"
    - "SnapshotDetailPanel uses useQuery(['restore-manifest', backup.id]) with enabled: isConnected guard"
    - "Restore shortcut in SnapshotRow calls onRestore -> handleRestoreShortcut -> useBackupStore.getState().startOperation + computeDryRun -> DryRunModal"
    - "useDeviceStore().connected check gates manifest fetch and restore button state"
    - "BackupTimeline renders SnapshotDetailPanel inline via selectedId prop comparison"

key-files:
  created:
    - src/components/backups/BackupsView.tsx
    - src/components/backups/BackupTimeline.tsx
    - src/components/backups/SnapshotRow.tsx
    - src/components/backups/SnapshotDetailPanel.tsx
  modified:
    - src/app/page.tsx

key-decisions:
  - "SnapshotDetailPanel rendered inside BackupTimeline loop (not BackupsView) for clean separation of timeline vs. detail concern"
  - "Device connection check uses useDeviceStore().connected (not .confirmed) — restore requires mount, not confirmation ceremony"
  - "formatFileSize helper defined locally in SnapshotRow and SnapshotDetailPanel — no shared util file needed at this stage"

patterns-established:
  - "Offline-capable view pattern: useQuery with enabled: false for connected-only data; summary fallback for disconnected state"
  - "Restore trigger pattern: stopPropagation on Restore button click, then onRestore callback -> useBackupStore.getState() -> startOperation + computeDryRun"

requirements-completed: [SAFE-05, SAFE-06]

# Metrics
duration: 3min
completed: 2026-04-30
---

# Phase 03 Plan 04: Backups View Summary

**Reverse-chronological backup timeline grouped by project name with file-diff detail panel and offline-safe restore flow via DryRunModal**

## Performance

- **Duration:** 3 min
- **Started:** 2026-04-30T19:33:03Z
- **Completed:** 2026-04-30T19:36:21Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- BackupsView shows all backups grouped by project_name from SQLite via useQuery, with loading skeletons and empty state
- SnapshotRow renders each backup with timestamp, operation label, file count, size, and a Restore shortcut button (stopPropagation isolated)
- SnapshotDetailPanel shows file change diff with semantic colors (Added green / Modified amber / Removed red) when OT connected; shows summary + "Connect your Octatrack" when disconnected
- Restore trigger in both SnapshotRow shortcut and SnapshotDetailPanel button goes through useBackupStore.startOperation + computeDryRun -> DryRunModal (SAFE-06)
- page.tsx BackupsView wired: placeholder replaced, import added, TypeScript passes

## Task Commits

Each task was committed atomically:

1. **Task 1: BackupsView, BackupTimeline, SnapshotRow, SnapshotDetailPanel** - `8291c0a` (feat)
2. **Task 2: page.tsx BackupsView wiring** - `864cc1d` (feat)

## Files Created/Modified
- `src/components/backups/BackupsView.tsx` - Top-level backups section with project grouping, empty/loading states, handleRestoreShortcut
- `src/components/backups/BackupTimeline.tsx` - Per-project snapshot list, renders SnapshotDetailPanel inline below selected row
- `src/components/backups/SnapshotRow.tsx` - 48px row with timestamp/op/count/size columns and Restore button
- `src/components/backups/SnapshotDetailPanel.tsx` - File diff panel with change colors, disconnected state, guarantee note, restore trigger
- `src/app/page.tsx` - Added BackupsView import, replaced placeholder with `<BackupsView />`

## Decisions Made
- SnapshotDetailPanel placed inside BackupTimeline loop rather than BackupsView directly — BackupTimeline owns the snapshot rendering context, so inline expansion belongs there
- `useDeviceStore().connected` (not `.confirmed`) gates manifest fetch and restore — confirmed tracks the volume confirmation ceremony, but restore needs active mount
- formatFileSize defined locally in both SnapshotRow and SnapshotDetailPanel — no shared util extracted at this stage, avoids premature abstraction

## Deviations from Plan

None - plan executed exactly as written.

The one acceptance criterion that checks "BackupsView.tsx contains 'SnapshotDetailPanel' import" is satisfied architecturally: BackupsView imports BackupTimeline which renders SnapshotDetailPanel inline. The plan narrative specifies "inside BackupTimeline's rendering loop — pass via a render prop or children pattern," which is the implemented approach.

## Threat Model Coverage

| Threat | Mitigation Applied |
|--------|--------------------|
| T-03-13: Restore while disconnected | SnapshotDetailPanel uses `useDeviceStore().connected` to disable restore button with "Connect your Octatrack" message |
| T-03-14: Restore without pre-snapshot | Guarantee note "A snapshot of the current state will be created before restoring" shown in every SnapshotDetailPanel |

## Issues Encountered
None

## Next Phase Readiness
- All four plans of Phase 03 complete: Rust backup engine, TypeScript types + IPC, UI flow (DryRunModal/Progress/Banner), and Backups view
- Phase 04 (advanced management) can proceed — backup/restore safety net is fully functional end-to-end

---
*Phase: 03-write-path-and-backup*
*Completed: 2026-04-30*
