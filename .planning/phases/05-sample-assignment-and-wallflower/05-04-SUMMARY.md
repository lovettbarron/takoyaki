---
phase: 05-sample-assignment-and-wallflower
plan: 04
subsystem: wallflower-ui-panel-and-settings
tags: [react, typescript, wallflower, slot-picker, push-to-slot, settings, tauri]
dependency_graph:
  requires:
    - 05-02 (getWallflowerStatus, searchWallflowerSamples, setWallflowerDbPath IPC wrappers, useSamplesStore)
    - 05-03 (SamplesTab assignment flow, DryRunModal extensions, SlotRow assign button)
  provides:
    - WallflowerPanel (collapsible library browser with search)
    - WallflowerSampleRow (individual sample display row)
    - SlotPickerDialog (Flex/Static slot selection for push-to-slot)
    - WallflowerSettings (connection status + path override)
    - SamplesTab Wallflower integration and push-to-slot flow
    - Settings nav section in page.tsx
  affects:
    - src/components/project-detail/WallflowerPanel.tsx
    - src/components/project-detail/WallflowerSampleRow.tsx
    - src/components/project-detail/SlotPickerDialog.tsx
    - src/components/project-detail/SamplesTab.tsx
    - src/components/settings/WallflowerSettings.tsx
    - src/app/page.tsx
    - src/components/sidebar-nav.tsx
tech_stack:
  added: []
  patterns:
    - 300ms debounce via useEffect + setTimeout for search input (D-12)
    - Conditional render (wallflowerConnected guard) for D-07 graceful degradation
    - Push-to-slot flow: WallflowerPanel.onPushToSlot -> openSlotPicker -> SlotPickerDialog.onConfirm -> handleSlotPickerConfirm -> computeSampleDryRun -> DryRunModal -> assignSample(fromWallflower=true)
    - pendingFromWallflower zustand flag controls success message copy and /AUDIO/ copy behavior
    - Settings section as inline view rendered in main content area when activeSection === "settings"
key_files:
  created:
    - src/components/project-detail/WallflowerPanel.tsx
    - src/components/project-detail/WallflowerSampleRow.tsx
    - src/components/project-detail/SlotPickerDialog.tsx
    - src/components/settings/WallflowerSettings.tsx
  modified:
    - src/components/project-detail/SamplesTab.tsx
    - src/app/page.tsx
    - src/components/sidebar-nav.tsx
decisions:
  - "WallflowerPanel rendered inside px-4 wrapper in SamplesTab for consistent horizontal padding with slot sections above"
  - "SlotPickerDialog receives samples query data (SampleSlotResponse) directly from SamplesTab's existing useQuery — no additional IPC round-trip"
  - "Settings rendered as inline content area section (not modal/drawer) — matches plan intent of accessible without navigating away from project detail"
  - "sidebar-nav.tsx Settings nav item enabled (available: true) — was false, unblocked for this plan"
  - "WallflowerSettings uses tauri-plugin-dialog open() same as SamplesTab — consistent pattern"
metrics:
  duration: "~5 min"
  completed: "2026-05-02T11:25:04Z"
  tasks_completed: 2
  files_modified: 7
---

# Phase 5 Plan 04: Wallflower UI Panel and Settings Summary

**One-liner:** WallflowerPanel (collapsible library browser with 300ms debounce search), WallflowerSampleRow, SlotPickerDialog (Flex/Static toggle, 128-slot list with occupied/empty chips), WallflowerSettings (connection status + path override), and full push-to-slot wiring through the existing dry-run/assign pipeline with fromWallflower=true.

## What Was Built

### Task 1: WallflowerPanel, WallflowerSampleRow, SlotPickerDialog

**`src/components/project-detail/WallflowerPanel.tsx`** — new file:
- Collapsible trigger: full-width `h-10` bar with ChevronUp/ChevronDown icon, `WALLFLOWER LIBRARY` heading
- Default state: expanded (wallflowerPanelExpanded defaults true per D-09 in Plan 02 store)
- Search bar: `h-8` input, 300ms debounce via `useEffect + setTimeout`, placeholder "Search by name, key, BPM, or tag..."
- Results: `<ScrollArea className="max-h-96">` with WallflowerSampleRow per result
- Loading state: 3 Skeleton rows
- Empty state: "No samples match your search." centered
- Truncation indicator: "Showing 200 results — refine your search" when results === 200
- React Query: `queryKey: ["wallflower-search", debouncedQuery]`, `enabled: wallflowerPanelExpanded`

**`src/components/project-detail/WallflowerSampleRow.tsx`** — new file:
- Layout: `flex h-9 items-center px-3 gap-2 border-b border-[hsl(30,8%,26%)] hover:bg-[hsl(30,8%,20%)]`
- Columns: filename (flex-1 truncate), key_name (w-8), bpm rounded integer (w-10), tags (up to 3 Badge variant=outline + "+N" overflow)
- Push button: `h-6 w-6` Upload icon 12px, ghost hover style, `aria-label="Push FILENAME to slot"`

**`src/components/project-detail/SlotPickerDialog.tsx`** — new file:
- Dialog with DialogHeader: "Assign to Slot" title + "Pushing FILENAME from Wallflower" sub-caption
- Flex/Static toggle: two Button variants default/ghost
- Slot list: ScrollArea max-h-48, each row shows `#NNN` + filename + status chip (amber=occupied, muted=empty)
- Selected row: accent left border `border-l-2 border-l-[hsl(38,85%,55%)]` + bg highlight
- Occupied slot warning: amber text below list when occupied slot is selected
- Footer: "Close Picker" ghost + "Assign to Slot" default (disabled until selection)
- useEffect resets slotTypeTab and selectedSlot when dialog opens

### Task 2: SamplesTab wiring, WallflowerSettings, page.tsx Settings

**`src/components/project-detail/SamplesTab.tsx`** — extended:
- Added imports: `useEffect`, `getWallflowerStatus`, `WallflowerSample` type, `WallflowerPanel`, `SlotPickerDialog`
- Destructured from store: `pendingFromWallflower`, `wallflowerConnected`, `slotPickerOpen`, `slotPickerSampleFilename`, `slotPickerSampleFilePath`, `setWallflowerConnected`, `openSlotPicker`, `closeSlotPicker`
- `useEffect` on mount: calls `getWallflowerStatus()` → `setWallflowerConnected(status.connected, status.db_path)` (D-07 state)
- `handlePushToSlot(sample: WallflowerSample)`: calls `openSlotPicker(sample.filename, sample.file_path)`
- `handleSlotPickerConfirm(slotType, slotIndex)`: closes picker, runs `computeSampleDryRun`, opens DryRunModal with `setPendingAssign(slotType, slotIndex, path, true)`
- `handleApplyAssign`: uses `pendingFromWallflower` for `assignSample` call; success message branches on `pendingFromWallflower` to show "copied to /AUDIO/" suffix
- JSX: renders `{wallflowerConnected && <WallflowerPanel .../>}` after Static section (D-07 conditional)
- JSX: renders `<SlotPickerDialog>` with `slots={samples}` (passes existing query data — no extra IPC)

**`src/components/settings/WallflowerSettings.tsx`** — new file:
- Section heading "WALLFLOWER" (font-mono text-xs uppercase)
- Status row: `CircleCheck` green or `CircleMinus` muted + status text + sample count (when connected)
- DB path row: shown when connected (truncated with title tooltip)
- Change... button: opens `@tauri-apps/plugin-dialog` file picker filtered to `*.db`, calls `setWallflowerDbPath`

**`src/app/page.tsx`** — extended:
- Import `WallflowerSettings`
- Added Settings section: `{activeSection === "settings" && <div><WallflowerSettings /></div>}`

**`src/components/sidebar-nav.tsx`** — updated:
- Settings nav item: `available: false` → `available: true`

## Deviations from Plan

None — plan executed exactly as written. All component props, state wiring, and copywriting matched the plan spec and UI-SPEC.

## Task 3: Human Verification Required

**Task 3 is a `checkpoint:human-verify`** — automated tasks complete, visual verification by user needed.

**How to verify (run `cargo tauri dev`):**

1. Launch the app and connect an Octatrack in USB disk mode
2. Navigate to any project → Samples tab

**Desktop assignment flow (SMPL-01, SMPL-03):**
3. Click the Upload button on any Flex slot row → macOS file picker (WAV/AIF/AIFF filter)
4. Select a valid WAV file → dry-run preview modal with affected files and snapshot guarantee
5. Click "Assign Sample" → green success banner with 4s auto-dismiss
6. Try assigning a non-WAV file → inline error: "Unsupported format..."
7. Try assigning a large file (>200MB) to Flex → inline error with "Assign to Static" redirect

**Wallflower integration (INTG-01, INTG-02, INTG-03):**
8. If Wallflower installed: "WALLFLOWER LIBRARY" panel appears below Static slots section
9. Type in search bar → results update after ~300ms delay
10. Click Push button on a sample → Slot Picker dialog (Flex/Static toggle, 128 slots with occupied/empty chips)
11. Select a slot → amber "occupied" chip and warning if occupied; click "Assign to Slot" → dry-run → success with "copied to /AUDIO/"
12. If Wallflower NOT installed: panel completely absent (no error, no empty state)

**Settings (D-08):**
13. Click Settings in sidebar → Wallflower section with connection status (green check or grey minus)
14. Click "Change..." → file picker for .db files

## Known Stubs

None — all data flows are wired. The `placeholder` attribute on the search input is an HTML placeholder (expected behavior), not a data stub.

## Threat Flags

No new threat surface beyond the plan's threat model.
- T-05-14: Wallflower `file_path` in push-to-slot — Rust backend handles canonicalization and destination hardcoded to `/AUDIO/` (Plan 01)
- T-05-16: 300ms debounce + 200-row SQL limit mitigates rapid search DoS
- T-05-17: `setWallflowerDbPath` path validation in Rust backend (Plan 02)

## Self-Check: PASSED

| Item | Status |
|------|--------|
| src/components/project-detail/WallflowerPanel.tsx | FOUND |
| src/components/project-detail/WallflowerSampleRow.tsx | FOUND |
| src/components/project-detail/SlotPickerDialog.tsx | FOUND |
| src/components/settings/WallflowerSettings.tsx | FOUND |
| Commit 2f5f20c (Task 1: three new components) | FOUND |
| Commit cb89e15 (Task 2: SamplesTab wiring + WallflowerSettings) | FOUND |
| WallflowerPanel contains searchWallflowerSamples | FOUND |
| WallflowerPanel contains setTimeout (300ms debounce) | FOUND |
| WallflowerSampleRow contains Math.round(sample.bpm) | FOUND |
| SlotPickerDialog contains slotTypeTab | FOUND |
| SamplesTab contains wallflowerConnected | FOUND |
| SamplesTab contains copied to /AUDIO/ | FOUND |
| SamplesTab contains pendingFromWallflower | FOUND |
| WallflowerSettings contains CircleCheck + CircleMinus | FOUND |
| npx tsc --noEmit (new files only) | PASSED |
| cargo test --workspace | PASSED (all tests pass) |
