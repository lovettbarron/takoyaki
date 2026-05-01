---
phase: 04-advanced-management
plan: "07"
subsystem: management-ui
tags: [conflict-resolution, bank-copy, ui, rust, typescript]
dependency_graph:
  requires:
    - 04-03  # management commands (bank-copy dry-run, copyBank IPC)
    - 04-04  # management UI (DryRunModal, BankCopyPickerDialog patterns)
  provides:
    - conflict-resolution-dialog
    - bank-copy-conflict-flow
  affects:
    - src/app/page.tsx
    - crates/takoyaki-app/src/commands/backup.rs
    - crates/takoyaki-app/src/commands/management.rs
tech_stack:
  added: []
  patterns:
    - ConflictResolutionDialog follows BankCopyPickerDialog warm-dark palette conventions
    - executeBankCopy extracted as helper to handle both conflict and no-conflict paths
key_files:
  created:
    - src/components/management/ConflictResolutionDialog.tsx
  modified:
    - crates/takoyaki-app/src/commands/backup.rs
    - crates/takoyaki-app/src/commands/management.rs
    - src/lib/types.ts
    - src/app/page.tsx
decisions:
  - "conflict_details defaulted to Vec::new() in all non-bank-copy manifests — no breaking change to existing operations"
  - "executeBankCopy extracted from handleMgmtDryRunApply so both conflict and no-conflict paths share one implementation"
  - "pendingConflicts stored as local state (camelCase) rather than management store (snake_case) — avoids impedance mismatch between Rust ConflictEntry and ConflictResolutionDialog props"
metrics:
  duration: "12 min"
  completed: "2026-05-01T20:57:43Z"
  tasks_completed: 2
  files_changed: 5
---

# Phase 04 Plan 07: Conflict Resolution UI for Bank Copy Summary

**One-liner:** Per-conflict resolution dialog (keep-target / use-source / rename-incoming) inserted between bank-copy dry-run and execution, with resolved map passed to copyBank IPC.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Extend FileChangeManifest with conflict details | eb16621 | backup.rs, management.rs, types.ts |
| 2 | Create ConflictResolutionDialog and wire into page.tsx | 34f5b27 | ConflictResolutionDialog.tsx, page.tsx |

## What Was Built

### Task 1: FileChangeManifest conflict_details field

Added `ConflictDetail` struct to `backup.rs`:
- `filename`, `source_hash`, `target_hash` fields
- Serializes to camelCase via serde (matching TypeScript `conflictDetails`)

Added `conflict_details: Vec<ConflictDetail>` to `FileChangeManifest`:
- Defaulted to `Vec::new()` in all 5 existing construction sites (backup dry-run, restore dry-run, duplicate, rename, export)
- Populated from `BankCopyAnalysis.conflicts` in the bank-copy branch of `compute_management_dry_run`

TypeScript: added `conflictDetails: Array<{ filename: string; sourceHash: string; targetHash: string }>` to `FileChangeManifest` interface in `types.ts`.

### Task 2: ConflictResolutionDialog + page.tsx wiring

`ConflictResolutionDialog.tsx` (186 lines):
- Props: `open`, `conflicts`, `onResolve`, `onCancel`
- Local state: `Record<string, ConflictResolution | null>` map initialized with all filenames null
- Per-conflict rows: filename in conflict purple (`hsl(280,60%,55%)`), truncated source/target hash snippets, three resolution buttons
- Bulk resolution row at top: "Apply to all" sets all conflicts to one resolution
- Apply button disabled until all conflicts resolved
- Resets state when `open` changes to true
- Follows BankCopyPickerDialog styling: `bg-[hsl(30,8%,20%)] border-[hsl(38,85%,55%)]` for selected state

`page.tsx` changes:
- Added `conflictDialogOpen` and `pendingConflicts` state
- Extracted `executeBankCopy(resolutions)` helper — contains the channel/copyBank call
- `handleMgmtDryRunApply`: when bank-copy AND `conflictDetails.length > 0`, closes dry-run modal, stores conflicts, opens ConflictResolutionDialog (returns early)
- Bank-copy with no conflicts: calls `executeBankCopy({})` directly — no dialog
- `handleConflictResolve`: closes dialog, calls `executeBankCopy(resolutions)` with user choices
- `handleConflictCancel`: closes dialog, resets management state

## Verification

- `cargo check -p takoyaki-app`: 0 errors, 2 pre-existing dead_code warnings
- `npx tsc --noEmit`: 0 errors
- `cargo test -p takoyaki-app`: all tests pass (2 restore, 3 project filter tests)
- Acceptance criteria: all 7 criteria satisfied (see plan)

## Deviations from Plan

None — plan executed exactly as written.

## Threat Surface Scan

No new network endpoints, auth paths, or schema changes introduced. The `conflict_details` field in `FileChangeManifest` exposes SHA-256 hashes of audio files to the frontend — this is covered by T-04-10 (accept disposition, hashes are integrity identifiers not secrets).

## Self-Check
