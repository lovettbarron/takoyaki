---
phase: 08-phase-5-quality-and-safety-fixes
plan: "02"
subsystem: samples-frontend
tags: [safety, conflict-detection, dismiss-button, overwrite, frontend]
dependency_graph:
  requires: [08-01]
  provides: [dismiss-button-wired, assignSample-overwrite-param, conflict-prompt-ui]
  affects:
    - src/components/project-detail/SlotRow.tsx
    - src/components/project-detail/SamplesTab.tsx
    - src/lib/tauri.ts
tech_stack:
  added: []
  patterns: [inline-banner-component, optional-callback-prop, error-string-prefix-detection]
key_files:
  modified:
    - src/components/project-detail/SlotRow.tsx
    - src/components/project-detail/SamplesTab.tsx
    - src/lib/tauri.ts
decisions:
  - "Dismiss button wired via optional onDismiss?: () => void prop — threaded from SlotSection's existing onDismissError callback, no new store action needed"
  - "CONFLICT: string prefix detection chosen over new error variant — both sides in same codebase, complexity of new variant outweighs benefit (per threat model T-08-07 accept disposition)"
  - "ConflictPrompt renders before AssignSuccessBanner so amber prompt appears above green banner if both states briefly overlap"
  - "handleApplyAssign accepts optional overwrite?: boolean param — default false preserves existing DryRunModal onApply call site without changes"
metrics:
  duration: "12 min"
  completed: "2026-05-06"
  tasks_completed: 2
  files_modified: 3
---

# Phase 08 Plan 02: Dismiss Button, Overwrite Param, and Conflict Prompt Summary

**One-liner:** SlotRow Dismiss button wired to clearSlotError via onDismiss prop, assignSample IPC wrapper updated with optional overwrite param, and amber ConflictPrompt UI added to SamplesTab to surface Wallflower destination conflicts instead of silently skipping them.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Wire SlotRow Dismiss button and update assignSample IPC wrapper | fc4316a | src/components/project-detail/SlotRow.tsx, src/components/project-detail/SamplesTab.tsx, src/lib/tauri.ts |
| 2 | Add conflict prompt UI in SamplesTab for Wallflower destination conflicts | 3981a13 | src/components/project-detail/SamplesTab.tsx |

## What Was Built

**WR-01: SlotRow Dismiss button (was a no-op)**
- Added `onDismiss?: () => void` to `SlotRowProps` interface in `SlotRow.tsx`
- Added `onDismiss` to destructured props on `export function SlotRow`
- Replaced `// Parent clears via clearSlotError — this button is visual-only` comment with `onDismiss?.()` call in the Dismiss button onClick
- `SlotSection` now passes `onDismiss={isErrorSlot ? onDismissError : undefined}` to each `SlotRow` — chains to `clearSlotError` from the samples store

**WR-04 (TypeScript side): assignSample overwrite param**
- Added optional `overwrite?: boolean` param to `assignSample` in `src/lib/tauri.ts`
- Passes `overwrite: overwrite ?? false` in the invoke call, matching the Rust backend signature added in Plan 08-01

**WR-04 (Frontend UI): Conflict prompt in SamplesTab**
- Added `conflictPending` and `conflictFilename` state variables to `SamplesTab`
- Updated `handleApplyAssign` to accept optional `overwrite?: boolean` param and detect `CONFLICT:` prefix errors from backend — sets `conflictPending=true` and extracts filename from error message instead of showing generic slot error
- Added `handleConflictOverwrite` (re-calls `handleApplyAssign(true)`) and `handleConflictCancel` (resets state) handlers
- Added `ConflictPrompt` inline component with amber theme: CircleAlert icon, filename display, Cancel and Overwrite buttons
- Conflict prompt rendered conditionally (`{conflictPending && (...)`) before the AssignSuccessBanner in the JSX

## Checkpoint: Task 3 Pending Human Verification

Task 3 (`checkpoint:human-verify`) requires running the app with `cargo tauri dev` to verify:
- **WR-01:** Dismiss button clears slot errors (was a no-op before this plan)
- **WR-02:** assign_sample rejects non-WAV/AIFF before any write (Rust backend from Plan 08-01)
- **WR-03:** Wallflower file copy uses atomic temp-then-rename (Rust backend from Plan 08-01)
- **WR-04:** Wallflower conflict prompt appears instead of silent skip

## Deviations from Plan

None — plan executed exactly as written. The `assignErrorRedirect` fallback `{ label: "Dismiss", onRedirect: onDismissError }` already existed in SlotSection; the new `onDismiss` prop provides a direct path that doesn't rely on the redirect mechanism.

## Known Stubs

None. Both the Dismiss button and conflict prompt are fully wired to real state and IPC calls. No placeholder data flows to the UI.

## Threat Surface Scan

No new network endpoints, auth paths, or trust boundaries introduced. Changes are entirely in existing frontend components. Threat model entries T-08-05 through T-08-07 addressed:
- T-08-05 (overwrite param spoofing): mitigated — TypeScript wrapper defaults overwrite to false; only `handleConflictOverwrite` passes true after explicit user confirmation
- T-08-06 (conflict prompt filename): accepted — shows only filename extracted from error string, not full path
- T-08-07 (CONFLICT: string matching): accepted — prefix match is fragile but acceptable; both sides in same codebase

## Self-Check: PASSED

| Check | Result |
|-------|--------|
| `SlotRow.tsx` contains `onDismiss?: () => void` | FOUND (line 32) |
| `SlotRow.tsx` contains `onDismiss?.()` in Dismiss button | FOUND (line 256) |
| `SamplesTab.tsx` contains `onDismiss={isErrorSlot ? onDismissError : undefined}` | FOUND (line 138) |
| `tauri.ts` contains `overwrite?: boolean` | FOUND (line 184) |
| `tauri.ts` contains `overwrite: overwrite ?? false` | FOUND (line 186) |
| `SamplesTab.tsx` contains `conflictPending` state | FOUND (line 195) |
| `SamplesTab.tsx` contains `CONFLICT:` detection | FOUND (line 384) |
| `SamplesTab.tsx` contains `function ConflictPrompt` | FOUND (line 165) |
| `SamplesTab.tsx` contains `{conflictPending && (` | FOUND (line 497) |
| Commit `fc4316a` (Task 1) | FOUND |
| Commit `3981a13` (Task 2) | FOUND |
| `npx tsc --noEmit` exits 0 | PASSED |
| `cargo test --workspace` exits 0 (104 tests) | PASSED |
