---
phase: 05-sample-assignment-and-wallflower
plan: 03
subsystem: sample-assignment-ui
tags: [react, typescript, tauri, zustand, sample-assignment, dry-run, file-picker]
dependency_graph:
  requires:
    - 05-01 (compute_sample_dry_run, assign_sample Tauri commands)
    - 05-02 (useSamplesStore, computeSampleDryRun/assignSample IPC wrappers, SampleDryRunResult type)
  provides:
    - SlotRow assign button with stopPropagation and inline error display
    - SamplesTab full assignment flow: file picker -> dry-run -> apply -> success
    - DryRunModal applyLabel/isApplying/softWarnings extension props
    - AssignSuccessBanner inline success component
  affects:
    - src/components/project-detail/SlotRow.tsx
    - src/components/project-detail/SamplesTab.tsx
    - src/components/backups/DryRunModal.tsx
tech_stack:
  added: []
  patterns:
    - e.stopPropagation() on assign button to prevent CollapsibleTrigger expand
    - assignStatus guard prevents concurrent assign triggers (T-05-11)
    - Hard block inline error below slot row (D-13, D-14) via assignError/assignErrorRedirect props
    - Soft warnings injected into DryRunModal via softWarnings prop
    - AssignSuccessBanner: simple inline success (4s auto-dismiss) separate from backup InlineSuccessBanner
    - deviceConnected gates onAssign — undefined prop disables button visually (opacity-40)
    - cache invalidation via queryClient.invalidateQueries after successful assignment
key_files:
  created: []
  modified:
    - src/components/project-detail/SlotRow.tsx
    - src/components/project-detail/SamplesTab.tsx
    - src/components/backups/DryRunModal.tsx
decisions:
  - "AssignSuccessBanner created inline in SamplesTab — backup InlineSuccessBanner has SuccessBanner interface with operation/projectName/fileCount/totalBytes/checksumOk fields incompatible with simple string message"
  - "DryRunModal extended with applyLabel, isApplying, softWarnings props — hardcoded backup/restore button text was incompatible with assignment flow; extension is backward-compatible (all props optional)"
  - "assignStatus guard reads useSamplesStore.getState() directly to check idle before opening file picker — prevents concurrent picks"
  - "SlotTableHeader gains trailing w-8 shrink-0 empty column to align with new assign button column in SlotRow"
  - "Dismiss button in inline error is visual-only; clearSlotError in SlotSection onDismissError prop provides the actual clear action"
metrics:
  duration: "~12 min"
  completed: "2026-05-02T12:00:00Z"
  tasks_completed: 2
  files_modified: 3
---

# Phase 5 Plan 03: Sample Assignment UI Summary

**One-liner:** SlotRow assign button (Upload icon, stopPropagation) wired to SamplesTab full assignment flow — native file picker, format-validated dry-run preview, atomic assignment, inline success banner with 4s auto-dismiss.

## What Was Built

### Task 1: SlotRow assign button and inline error display

**`src/components/project-detail/SlotRow.tsx`** — extended with:

- `Upload` icon imported from lucide-react
- Three new optional props: `onAssign?`, `assignError?`, `assignErrorRedirect?`
- Trailing `w-8 shrink-0` assign button column inside `CollapsibleTrigger`
  - `e.stopPropagation()` prevents expand/collapse when assign button is clicked
  - `opacity-40 pointer-events-none` when `onAssign` is undefined (device disconnected)
  - `aria-label="Assign sample to Flex/Static slot NNN"` for accessibility
- Inline error block below `CollapsibleContent` renders when `assignError` is set
  - `bg-[hsl(0,68%,12%)] border border-destructive rounded` dark-red background
  - Optional amber redirect button (`text-[hsl(38,85%,55%)] underline`) for slot-type mismatches (D-13)
  - Dismiss button clears via parent `onDismissError` callback

### Task 2: SamplesTab full assignment flow + DryRunModal extension

**`src/components/project-detail/SamplesTab.tsx`** — full rewrite with:

- Imports: `open` from `@tauri-apps/plugin-dialog`, `useQueryClient`, `computeSampleDryRun`, `assignSample`, `useSamplesStore`, `useDeviceStore`, `DryRunModal`
- `AssignSuccessBanner` inline component: green success bar with `CircleCheck` icon, 4s auto-dismiss via `setTimeout(() => reset(), 4000)` (D-05)
- `SlotSection` component extended with error props passed through to `SlotRow`
- `SlotTableHeader` extended with trailing `w-8 shrink-0` column for header alignment
- `handleAssign(slotIndex, slotType)`:
  - T-05-11 guard: returns early if `assignStatus !== "idle"`
  - Opens native macOS file picker filtered to WAV/AIF/AIFF (D-01)
  - Calls `computeSampleDryRun` for format validation
  - Hard block: shows inline error — slot-type mismatch gets redirect button (D-13), format error has no redirect (D-14)
  - Success: sets `pendingApplyLabel` ("Assign Sample" vs "Replace Sample"), opens DryRunModal (D-03)
- `handleApplyAssign()`: calls `assignSample` → success banner → `queryClient.invalidateQueries(["samples", projectId])` → 4s auto-reset
- `handleCancelAssign()`: closes modal, resets store
- `handleSlotRedirect()`: clears error, re-triggers `handleAssign` with redirect target slot
- `onAssign={deviceConnected ? handleAssign : undefined}` disables all assign buttons when OT not connected

**`src/components/backups/DryRunModal.tsx`** — backward-compatible extension:

- `applyLabel?: string` — overrides apply button text (used for "Assign Sample" / "Replace Sample")
- `isApplying?: boolean` — disables both cancel and apply buttons during in-progress assignment
- `softWarnings?: string[]` — renders amber italic warnings between snapshot guarantee line and change summary strip (D-14 soft block)
- All existing backup/restore callers unaffected (all new props optional)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] InlineSuccessBanner has incompatible interface**
- **Found during:** Task 2 implementation
- **Issue:** Plan stated `InlineSuccessBanner` takes `message: string` and `onDismiss: () => void` props. Actual component (`src/components/backup-progress/InlineSuccessBanner.tsx`) takes `banner: SuccessBanner` where `SuccessBanner` has backup-specific fields: `operation`, `projectName`, `fileCount`, `totalBytes`, `destination`, `checksumOk`. Incompatible with sample assignment success message.
- **Fix:** Created `AssignSuccessBanner` component inline in SamplesTab with the simple `message/onDismiss` interface described in the plan. Identical visual style (green success bar, CircleCheck icon, 4s auto-dismiss, X dismiss button).
- **Files modified:** `src/components/project-detail/SamplesTab.tsx`
- **Commit:** `451e985`

**2. [Rule 1 - Bug] DryRunModal footer button text hardcoded for backup/restore**
- **Found during:** Task 2 implementation
- **Issue:** Plan said to reuse DryRunModal "without modification" and use operation_label to control title. However, the modal's footer button text is determined by `manifest.operationLabel.startsWith("Back Up")` — for sample assignment this would show "Restore Snapshot" (wrong) or "Back Up 0 files" (wrong).
- **Fix:** Extended DryRunModal with optional `applyLabel` prop. When provided, renders a default-variant button with that label instead of the backup/restore branch. Also added `isApplying` (disables buttons) and `softWarnings` (renders between snapshot guarantee and change summary) as companion props. All new props are optional — existing backup/restore usage is unaffected.
- **Files modified:** `src/components/backups/DryRunModal.tsx`
- **Commit:** `451e985`

## Known Stubs

None — all data flows are wired. The assignment flow is fully connected from SlotRow button through to atomic Rust command and cache invalidation.

## Threat Flags

No new threat surface beyond the plan's threat model.

- T-05-11 (concurrent assign clicks): Mitigated via `assignStatus !== "idle"` guard at the top of `handleAssign` — reads store state directly before opening file picker.
- T-05-12 (file path from dialog): Accepted — native macOS dialog returns real paths; Rust backend canonicalizes.
- T-05-13 (success banner path disclosure): Accepted — only filename shown in success message, not full path.

## Self-Check: PASSED

| Item | Status |
|------|--------|
| src/components/project-detail/SlotRow.tsx | FOUND |
| src/components/project-detail/SamplesTab.tsx | FOUND |
| src/components/backups/DryRunModal.tsx | FOUND |
| Commit f99f57a (Task 1: SlotRow assign button) | FOUND |
| Commit 451e985 (Task 2: SamplesTab assignment flow) | FOUND |
| npx tsc --noEmit | PASSED (exit 0, zero errors) |
| grep onAssign SlotRow.tsx | FOUND (4 occurrences) |
| grep stopPropagation SlotRow.tsx | FOUND (2 occurrences) |
| grep computeSampleDryRun SamplesTab.tsx | FOUND (2 occurrences) |
| grep DryRunModal SamplesTab.tsx | FOUND (3 occurrences) |
| grep SuccessBanner SamplesTab.tsx | FOUND (3 occurrences) |
| grep invalidateQueries SamplesTab.tsx | FOUND (1 occurrence) |
| grep deviceConnected SamplesTab.tsx | FOUND (2 occurrences) |
