---
phase: 04-advanced-management
plan: "02"
subsystem: frontend-data-layer
tags: [typescript, zustand, ipc, shadcn, management]
dependency_graph:
  requires: []
  provides:
    - "ManagementOperation, ManagementEvent, ConflictResolution, ConflictEntry TypeScript types"
    - "useManagementStore zustand store with full operation lifecycle"
    - "5 management IPC wrappers in tauri.ts"
    - "shadcn context-menu component"
  affects:
    - "src/lib/types.ts"
    - "src/lib/stores/management.ts"
    - "src/lib/tauri.ts"
    - "src/components/ui/context-menu.tsx"
tech_stack:
  added: []
  patterns:
    - "ManagementEvent discriminated union matching Rust enum variant serialization shape"
    - "Management store follows established backup store lifecycle pattern (idle -> dry-running -> in-progress -> complete -> failed)"
    - "shadcn component installed via CLI using @base-ui/react primitives with base-nova styling"
key_files:
  created:
    - src/lib/stores/management.ts
    - src/components/ui/context-menu.tsx
  modified:
    - src/lib/types.ts
    - src/lib/tauri.ts
decisions:
  - "ManagementEvent uses snake_case field names (total_files, files_processed, current_file) matching Rust enum variant serialization shape"
  - "ChangeType extended with 'Conflict' variant to support bank-copy conflict detection"
  - "context-menu installed via shadcn CLI (not manually) — CLI produced full implementation using @base-ui/react/context-menu primitives"
metrics:
  duration: "~2 min"
  completed_date: "2026-05-01"
  tasks_completed: 2
  files_modified: 4
---

# Phase 04 Plan 02: Frontend Foundation — Types, Store, IPC, Context Menu Summary

Frontend data layer for Phase 4 advanced management: ManagementEvent/ConflictEntry types, operation lifecycle zustand store, 5 management IPC wrappers, and shadcn context-menu component installed via base-ui primitives.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | TypeScript types, management store, and IPC wrappers | 5b32926 | src/lib/types.ts, src/lib/stores/management.ts, src/lib/tauri.ts |
| 2 | Install shadcn context-menu component | e9dd570 | src/components/ui/context-menu.tsx |

## What Was Built

### Task 1: TypeScript Types, Management Store, and IPC Wrappers

**`src/lib/types.ts`** extended with:
- `ChangeType` union extended to include `"Conflict"` variant
- `ManagementOperation` type: `"duplicate" | "rename" | "export" | "bank-copy"`
- `ManagementEvent` discriminated union matching Rust enum variant serialization
- `ConflictResolution` type: `"keep-target" | "use-source" | "rename-incoming"`
- `ConflictEntry` interface with filename, source/target hashes, and optional resolution

**`src/lib/stores/management.ts`** created following exact backup.ts store shape:
- `ManagementStatus` type with 5 lifecycle states: `idle | dry-running | in-progress | complete | failed`
- `useManagementStore` with fields: status, operation, activeProjectId, activeProjectName, dryRunManifest, conflicts, progress, successMessage
- Actions: setStatus, startOperation, setDryRunManifest, setConflicts, setProgress, setSuccessMessage, reset

**`src/lib/tauri.ts`** extended with 5 management IPC wrappers:
- `computeManagementDryRun(projectId, operation, targetProjectId?, bankIndex?, newName?)` → `FileChangeManifest`
- `duplicateProject(projectId, newName, onEvent)` → `void`
- `renameProject(projectId, newName)` → `void`
- `exportProject(projectId, onEvent)` → `void`
- `copyBank(sourceProjectId, sourceBankIndex, targetProjectId, targetBankIndex, conflictResolutions, onEvent)` → `void`

### Task 2: shadcn context-menu

`src/components/ui/context-menu.tsx` installed via `npx shadcn@latest add context-menu`. The CLI used `@base-ui/react/context-menu` primitives with base-nova styling consistent with `dialog.tsx`. Full export set includes: ContextMenu, ContextMenuTrigger, ContextMenuContent, ContextMenuItem, ContextMenuCheckboxItem, ContextMenuRadioItem, ContextMenuLabel, ContextMenuSeparator, ContextMenuShortcut, ContextMenuGroup, ContextMenuPortal, ContextMenuSub, ContextMenuSubContent, ContextMenuSubTrigger, ContextMenuRadioGroup.

## Deviations from Plan

### Pre-existing Out-of-Scope TypeScript Errors

Two pre-existing TypeScript errors exist in Phase 3 components not modified by this plan:
- `src/components/backups/DryRunModal.tsx(31,45)`: Function lacks ending return statement
- `src/components/backups/SnapshotDetailPanel.tsx(22,43)`: Function lacks ending return statement

These errors were present before this plan's changes (confirmed via git diff) and are not caused by any work in this plan. Logged to deferred items for Phase 3 cleanup.

No other deviations — plan executed as written.

## Known Stubs

None. This plan is the data layer only (no visual components) — no UI stubs introduced.

## Threat Flags

None. This plan adds TypeScript type definitions and store/IPC scaffolding only. No new network endpoints, auth paths, file access patterns, or schema changes are introduced. IPC wrappers only pass UUIDs and enum-like strings to the Rust backend (T-04-04 accepted).

## Self-Check: PASSED

- `src/lib/types.ts` — exists, contains ManagementOperation, ManagementEvent, ConflictResolution, ConflictEntry, "Conflict" in ChangeType
- `src/lib/stores/management.ts` — exists, exports useManagementStore, contains startOperation, setDryRunManifest, reset
- `src/lib/tauri.ts` — exists, contains computeManagementDryRun, duplicateProject, renameProject, exportProject, copyBank
- `src/components/ui/context-menu.tsx` — exists, contains ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger
- Commit 5b32926 — Task 1
- Commit e9dd570 — Task 2
- New files have zero TypeScript errors (pre-existing errors in unrelated Phase 3 files documented above)
