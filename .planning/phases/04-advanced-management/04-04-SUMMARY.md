---
phase: 04-advanced-management
plan: "04"
subsystem: frontend-ui
tags: [management, ui, rename, duplicate, export, bank-copy, context-menu, dry-run]
dependency_graph:
  requires: [04-02]
  provides: [management-ui-complete]
  affects: [src/app/page.tsx, src/components/project-detail/MetadataHeader.tsx, src/components/project-detail/BankGridCell.tsx, src/components/project-detail/BanksTab.tsx, src/components/project-detail/ProjectDetailView.tsx, src/components/management/BankCopyPickerDialog.tsx]
tech_stack:
  added: []
  patterns:
    - "Management operation dry-run modal reuses DryRunModal with separate instance per operation type (backup vs management)"
    - "Callback threading: page.tsx -> ProjectDetailView -> BanksTab/MetadataHeader -> BankGridCell"
    - "Pending operation params captured in useRef (pendingRenameRef, pendingBankCopyRef) for use in async apply handler"
    - "Management success shown as inline banner div (not SuccessBanner type) with 4s auto-dismiss via useEffect"
key_files:
  created:
    - src/components/management/BankCopyPickerDialog.tsx
  modified:
    - src/components/project-detail/MetadataHeader.tsx
    - src/components/project-detail/BankGridCell.tsx
    - src/components/project-detail/BanksTab.tsx
    - src/components/project-detail/ProjectDetailView.tsx
    - src/app/page.tsx
decisions:
  - "BankCopyPickerDialog fetches target project banks via getProjectBanks() IPC on step transition (not pre-fetched) to ensure fresh populated state"
  - "Management success banner implemented as inline div (not SuccessBanner type from backup store) since management ops lack destination/checksum fields"
  - "Duplicate default name uses _COPY suffix truncated to 16 chars; window.prompt fallback if name already 16 chars"
  - "BankGridCell onCopyToProject only wired when populated=true; empty cells get undefined to match existing onClick pattern"
  - "ProjectDetailView props extended (onRename/onDuplicate/onExport/onCopyBankToProject) to thread callbacks without bypassing component boundary"
metrics:
  duration: "3 min 15 sec"
  completed: "2026-05-01"
  tasks: 2
  files: 6
---

# Phase 04 Plan 04: Management UI Components Summary

Frontend UI for all four Phase 4 management operations — MetadataHeader toolbar with Rename/Duplicate/Export ghost buttons, inline rename editor with real-time OT character validation, BankGridCell right-click context menu for bank copy, two-step BankCopyPickerDialog (project list -> 4x4 bank grid), and page.tsx orchestrating all operations through dry-run preview and success feedback.

## What Was Built

### Task 1: MetadataHeader toolbar buttons and inline rename (commit fb0f514)

Extended `MetadataHeader` with three ghost-variant toolbar buttons (Rename, Duplicate, Export) placed before the primary Back Up button. Clicking Rename toggles inline `<input>` that:
- Filters input to `[A-Z0-9_]` in real-time via `.replace(/[^A-Z0-9_]/g, "")`
- Forces uppercase
- Enforces `maxLength={16}`
- Enter confirms (calls `onRename` callback), Escape cancels and restores original name
- Blurring also cancels (safe default)

Props extended: `onRename?: (newName: string) => void`, `onDuplicate?: () => void`, `onExport?: () => void`

### Task 2: BankGridCell context menu, BankCopyPickerDialog, page.tsx wiring (commit 7813791)

**BankGridCell**: Wrapped existing `<button>` with `ContextMenu`/`ContextMenuTrigger`. Context menu content only renders when `populated=true` and shows a single "Copy to project..." item with `ArrowRightFromLine` icon. New prop: `onCopyToProject?: () => void`.

**BanksTab**: Extended with `onCopyBankToProject?: (bankIndex: number) => void` prop, passed down to each `BankGridCell` as `onCopyToProject={() => onCopyBankToProject?.(i)}` (only when populated).

**ProjectDetailView**: Extended with four new optional callback props (`onRename`, `onDuplicate`, `onExport`, `onCopyBankToProject`), threaded to `MetadataHeader` and `BanksTab`.

**BankCopyPickerDialog**: Two-step dialog component:
- Step 1: ScrollArea project list filtered to exclude source project. Selected row shows amber left-border highlight. "Next" advances to step 2 and fetches target bank states via `getProjectBanks()`.
- Step 2: 4x4 grid of bank slot buttons mirroring BankGridCell visual pattern (filled dot = populated, outlined = empty). Selecting an occupied slot shows overwrite warning in amber. "Copy Bank" calls `onConfirm(targetProjectId, targetBankSlot)`.
- Empty state: "No other projects on this card." per UI-SPEC copywriting.

**page.tsx**: Full management orchestration:
- `useManagementStore` destructured alongside existing `useBackupStore`
- `pendingRenameRef` / `pendingBankCopyRef` capture op params for use in async apply handler
- `projectList` state populated from `listProjects({})` on device confirm (feeds BankCopyPickerDialog)
- Handlers: `handleRename`, `handleDuplicate`, `handleExport`, `handleBankCopyTrigger`, `handleBankCopyConfirm`, `handleMgmtDryRunApply`, `handleMgmtDryRunCancel`
- Second `DryRunModal` instance for management ops (opens when `mgmtDryRunManifest !== null && mgmtStatus === "dry-running"`)
- Management success shown as fixed-top banner div with 4s auto-dismiss via `useEffect`
- `BankCopyPickerDialog` rendered at page level with `navProjectId` as `sourceProjectId`

## Deviations from Plan

### Auto-added: sourceProjectId prop on BankCopyPickerDialog

The plan's `BankCopyPickerDialogProps` interface did not include `sourceProjectId`, but filtering out the source project (so users can't copy to the same project) requires knowing the source project's ID. Added `sourceProjectId: string` prop — passed from page.tsx as `navProjectId ?? ""`.

### Auto-adapted: Management success banner as inline div

The plan suggested reusing `InlineSuccessBanner`, but that component accepts `SuccessBanner` type from the backup store which requires `destination`, `checksumOk`, `fileCount`, `totalBytes`, and `operation: "backup" | "restore"` — fields management operations don't have. Implemented a simpler inline banner div with identical visual styling (amber-on-dark-green, fixed-top) that displays the `mgmtSuccessMessage` string directly.

## Threat Model Compliance

| Threat ID | Mitigation | Status |
|-----------|-----------|--------|
| T-04-10 | Real-time input filter `/[^A-Z0-9_]/g` + `maxLength={16}` in rename input | Implemented |
| T-04-11 | All IPC calls pass `project_id` (UUID) only; path computation in Rust backend | Implemented |

## Known Stubs

None — all management operations are fully wired to IPC handlers. The `conflictResolutions` parameter in `copyBank` is passed as `{}` (empty object) since conflict resolution UI is deferred to a future plan (no conflict resolution step exists in this plan's scope).

## Self-Check: PASSED

| Item | Status |
|------|--------|
| BankCopyPickerDialog.tsx | FOUND |
| MetadataHeader.tsx | FOUND |
| BankGridCell.tsx | FOUND |
| page.tsx | FOUND |
| 04-04-SUMMARY.md | FOUND |
| Commit fb0f514 (Task 1) | FOUND |
| Commit 7813791 (Task 2) | FOUND |
