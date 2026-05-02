---
phase: 05-sample-assignment-and-wallflower
verified: 2026-05-02T14:00:00Z
status: human_needed
score: 4/4 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Launch the app with cargo tauri dev, connect an Octatrack in USB disk mode, navigate to a project Samples tab, click the Upload button on any Flex slot row — verify a macOS file picker opens filtered to WAV/AIF/AIFF, select a valid WAV file, verify the dry-run preview modal appears with affected files and snapshot guarantee text, click Assign Sample, verify green success banner with 4s auto-dismiss"
    expected: "File picker opens, dry-run modal shows project.work and project.strd as Modified, assignment succeeds, slot updates"
    why_human: "Tauri native file dialog requires macOS interaction; DryRunModal content is visual; slot refresh is visual"
  - test: "Attempt to assign a non-WAV/MP3 file to a slot — verify inline error appears below the slot row with the text 'Unsupported format: OT accepts WAV and AIFF only'"
    expected: "Inline error with destructive red styling appears below the slot row, modal does NOT open"
    why_human: "File picker interaction requires real file selection; inline error positioning is visual"
  - test: "Attempt to assign a file >200MB to a Flex slot — verify inline error with 'Assign to Static' redirect button appears"
    expected: "Error shows with amber redirect button; clicking redirect opens file picker targeting a Static slot"
    why_human: "Requires a large file; redirect button behavior requires interaction"
  - test: "With Wallflower installed: verify WALLFLOWER LIBRARY panel appears below Static slots section (collapsed/expanded per D-09 default expanded). Type in search bar and verify results update after ~300ms. Click Push on a sample, verify Slot Picker dialog opens with FLEX/STATIC toggle and 128 slots showing occupied/empty status"
    expected: "Panel visible; search debounces 300ms; dialog shows correct toggle and slot list"
    why_human: "Wallflower DB must exist on test machine; visual layout verification; 300ms timing is perceptual"
  - test: "With Wallflower NOT installed (remove DB or use machine without Wallflower): verify the WALLFLOWER LIBRARY panel is completely absent — no error message, no empty state, no visible panel trigger"
    expected: "Panel entirely hidden; no console error; no visible placeholder"
    why_human: "Requires controlling Wallflower installation state; absence verification is visual"
  - test: "Navigate to Settings in the sidebar. Verify the Wallflower section shows connection status (green CircleCheck icon when connected, grey CircleMinus when not). Click Change... and verify file picker opens filtered to .db files"
    expected: "Status icon matches connection state; Change... opens correct file picker"
    why_human: "Visual icon check; file picker interaction requires macOS"
  - test: "Verify the SlotRow Dismiss button (shown alongside inline errors that include a redirect button) actually clears the error. This was flagged in WR-01 of the code review — the onClick handler may be a no-op stub"
    expected: "Clicking Dismiss clears the inline error from the slot row"
    why_human: "Requires triggering a slot-type mismatch error (>200MB Flex file) to get both error and redirect button visible; then testing Dismiss separately from the redirect"
---

# Phase 5: Sample Assignment and Wallflower Verification Report

**Phase Goal:** Users can assign any desktop audio file to a specific Flex or Static sample slot with all affected OT binary files updated atomically, and optionally browse and deploy samples from the Wallflower library — with graceful degradation when Wallflower is not present.
**Verified:** 2026-05-02T14:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|---------|
| 1 | User can assign a desktop audio file to a Flex or Static slot with all affected binary files updated atomically with a pre-write snapshot | VERIFIED | `assign_sample` in samples.rs: snapshots `project.work` + `project.strd` via `SnapshotEngine::new(snapshot_root())`, rewrites both via `rewrite_slot_path`, writes both via `atomic_write_batch`. 13/13 unit tests pass |
| 2 | The app validates Flex vs Static slot type correctness before assigning and blocks incompatible assignments with a clear error | VERIFIED | `compute_sample_dry_run` validates `slot_type` enum (only "flex"/"static" accepted), validates `slot_index` 0..=127, blocks Flex slots with files >200MB with a hard_block message. `SamplesTab.handleAssign` shows inline error for hard_block. Test `test_flex_slot_size_check` and `test_format_validation_non_audio_produces_unsupported_issue` both pass |
| 3 | User can search the Wallflower sample library by key, BPM, and tags from within Takoyaki and push a selected sample to an OT slot | VERIFIED | `search_wallflower_samples` in wallflower.rs uses parameterized JOIN across jams/jam_tempo/jam_key/jam_tags. `WallflowerPanel.tsx` debounces 300ms, calls `searchWallflowerSamples`, renders `WallflowerSampleRow` with key/BPM/tags. `SlotPickerDialog` wired via `handleSlotPickerConfirm` -> `computeSampleDryRun` -> `assignSample(fromWallflower=true)`. 14/14 wallflower unit tests pass |
| 4 | When Wallflower is not installed or its database is unavailable, the Wallflower panel is hidden — no crash, no error dialog | VERIFIED | `get_wallflower_status` returns `connected: false` when no DB found (tested by `test_get_wallflower_status_connected_false_when_no_db`). `SamplesTab` checks on mount and sets `wallflowerConnected`. JSX renders `{wallflowerConnected && <WallflowerPanel .../>}` — panel completely absent when not connected |

**Score:** 4/4 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|---------|---------|--------|---------|
| `migrations/V3__wallflower_settings.sql` | Settings table with wallflower_db_path | VERIFIED | `CREATE TABLE IF NOT EXISTS settings` + `INSERT OR IGNORE INTO settings (key, value) VALUES ('wallflower_db_path', '')` |
| `crates/takoyaki-app/src/commands/samples.rs` | compute_sample_dry_run + assign_sample | VERIFIED | Both async functions present; SampleDryRunResult + AssignSampleResult structs; hard_block, soft_warnings, SnapshotEngine::new, atomic_write_batch, rewrite_slot_path, from_wallflower, std::fs::copy all present |
| `crates/takoyaki-app/src/commands/wallflower.rs` | get_wallflower_status, search_wallflower_samples, set_wallflower_db_path | VERIFIED | All 3 Tauri commands present; WallflowerSample + WallflowerStatus structs; discover_wallflower_db with auto-discovery chain; rusqlite::params![] used throughout |
| `crates/takoyaki-app/src/lib.rs` | tauri_plugin_dialog::init() + all 5 command registrations | VERIFIED | tauri_plugin_dialog::init() registered; all 5 commands in collect_commands![] macro |
| `crates/takoyaki-app/src/db/mod.rs` | V3 migration + get_setting/set_setting | VERIFIED | MIGRATION_V3 constant, user_version < 3 block, get_setting() and set_setting() public functions |
| `src/lib/types.ts` | SampleDryRunResult, AssignSampleResult, WallflowerStatus, WallflowerSample | VERIFIED | All 4 interfaces present at lines 155–185 |
| `src/lib/tauri.ts` | 5 IPC wrappers | VERIFIED | computeSampleDryRun, assignSample, getWallflowerStatus, searchWallflowerSamples, setWallflowerDbPath all present with correct invoke() calls |
| `src/lib/stores/samples.ts` | useSamplesStore with assignment flow + Wallflower state | VERIFIED | useSamplesStore with assignStatus, wallflowerConnected, slotPickerOpen; 10+ action methods |
| `src/components/project-detail/SlotRow.tsx` | Assign button with stopPropagation, inline error display | VERIFIED | onAssign?, assignError?, assignErrorRedirect? props; e.stopPropagation() on button click; Upload icon; bg-[hsl(0,68%,12%)] inline error |
| `src/components/project-detail/SamplesTab.tsx` | Full assignment flow orchestration | VERIFIED | computeSampleDryRun, assignSample, handleAssign, handleApplyAssign, DryRunModal, AssignSuccessBanner, invalidateQueries, deviceConnected, WallflowerPanel, SlotPickerDialog, handlePushToSlot, wallflowerConnected, getWallflowerStatus, pendingFromWallflower, "copied to /AUDIO/" all present |
| `src/components/project-detail/WallflowerPanel.tsx` | Collapsible panel with search | VERIFIED | searchWallflowerSamples, WALLFLOWER LIBRARY heading, search placeholder, setTimeout for 300ms debounce, Showing 200 results indicator, No samples match empty state |
| `src/components/project-detail/WallflowerSampleRow.tsx` | Sample row with push button | VERIFIED | onPush callback, key_name, Math.round(sample.bpm), sample.tags.slice(0, 3) |
| `src/components/project-detail/SlotPickerDialog.tsx` | Slot selection dialog | VERIFIED | Assign to Slot, Close Picker, occupied chip, slotTypeTab, FLEX/STATIC buttons, onConfirm callback |
| `src/components/settings/WallflowerSettings.tsx` | Settings panel with connection status | VERIFIED | CircleCheck, CircleMinus, Change... button, setWallflowerDbPath, Connected/Not connected text |

### Key Link Verification

| From | To | Via | Status | Details |
|------|-----|-----|--------|---------|
| commands/samples.rs | management/project_work.rs | rewrite_slot_path() | WIRED | `use crate::management::project_work::{self, SlotType}` at line 17; `project_work::rewrite_slot_path(...)` called at lines 482 and 503 |
| commands/samples.rs | atomic/mod.rs | atomic_write_batch() | WIRED | `crate::atomic::atomic_write_batch(...)` called at line 524 |
| commands/samples.rs | health/mod.rs | read_audio_spec + check_format_compatibility | WIRED | `health::read_audio_spec` + `health::check_format_compatibility` + `health::FormatIssue` all used in compute_sample_dry_run |
| commands/wallflower.rs | db/wallflower.rs | open_wallflower_db() | WIRED | `use crate::db::wallflower::open_wallflower_db` at line 14; called in get_wallflower_status and search_wallflower_samples |
| commands/wallflower.rs | db/mod.rs | get_setting() for wallflower_db_path | WIRED | `db::get_setting(&db.conn, "wallflower_db_path")` called in get_wallflower_status and search_wallflower_samples |
| src/lib/tauri.ts | commands/wallflower.rs | invoke() IPC calls | WIRED | `invoke("search_wallflower_samples", ...)`, `invoke("get_wallflower_status")`, `invoke("set_wallflower_db_path", ...)` |
| SlotRow.tsx | SamplesTab.tsx | onAssign callback prop | WIRED | `onAssign?:` in SlotRowProps; passed as `onAssign={assignHandler}` from SamplesTab |
| SamplesTab.tsx | tauri.ts | computeSampleDryRun + assignSample | WIRED | Both imported and called in handleAssign and handleApplyAssign |
| SamplesTab.tsx | DryRunModal.tsx | manifest prop from dry-run result | WIRED | `manifest={dryRunManifest}` at line 475 |
| WallflowerPanel.tsx | tauri.ts | searchWallflowerSamples | WIRED | Imported and used in useQuery queryFn |
| WallflowerSampleRow.tsx | WallflowerPanel.tsx | onPush callback prop | WIRED | `onPush={onPushToSlot}` passed; `onPush(sample)` called in button handler |
| SlotPickerDialog.tsx | SamplesTab.tsx | onConfirm with slotType + slotIndex | WIRED | `onConfirm={handleSlotPickerConfirm}` at line 488; handler receives slotType + slotIndex |
| SamplesTab.tsx | commands/samples.rs | assignSample with fromWallflower=true | WIRED | `assignSample(..., pendingFromWallflower)` at line 302; pendingFromWallflower set to true via `setPendingAssign(slotType, slotIndex, path, true)` in handleSlotPickerConfirm |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|---------|-------------|--------|-------------------|--------|
| WallflowerPanel.tsx | samples (from useQuery) | searchWallflowerSamples -> search_wallflower_samples Rust command -> search_samples() SQL JOIN on jams/jam_tempo/jam_key/jam_tags | Yes (real DB query with rusqlite) | FLOWING |
| SamplesTab.tsx | dryRunManifest | computeSampleDryRun -> compute_sample_dry_run reads project.work, builds FileChangeManifest | Yes (reads project directory from DB, builds manifest) | FLOWING |
| SamplesTab.tsx | samples (for SlotPickerDialog) | getProjectSamples -> get_project_samples — returns 128 empty stub slots | No (pre-existing Phase 2 stub, FIXME documented) | HOLLOW_STUB (pre-existing, not Phase 5 regression) |
| WallflowerSettings.tsx | wallflowerConnected, sampleCount | getWallflowerStatus -> get_wallflower_status -> discover_wallflower_db -> open_wallflower_db -> COUNT(*) | Yes (real DB connection check) | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|---------|--------|--------|-------|
| Rust samples unit tests (format validation, Flex size, slot rewrite) | `cargo test -p takoyaki-app --lib -- commands::samples` | 13/13 passed | PASS |
| Rust wallflower unit tests (auto-discovery, search, graceful degradation) | `cargo test -p takoyaki-app --lib -- commands::wallflower` | 14/14 passed | PASS |
| Full workspace tests | `cargo test --workspace` | 89/89 passed (0 failed) | PASS |
| TypeScript compilation | `npx tsc --noEmit` | Exit 0, zero errors | PASS |
| Native file picker open / visual UX | Requires `cargo tauri dev` + macOS interaction | — | SKIP (needs human) |
| Wallflower panel hide/show | Requires Wallflower installed/uninstalled state | — | SKIP (needs human) |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|------------|------------|------------|--------|---------|
| SMPL-01 | 05-01, 05-03 | User can assign a desktop audio file to a specific Flex or Static sample slot with all affected binary files updated atomically | SATISFIED | assign_sample command: snapshot + rewrite_slot_path + atomic_write_batch; SamplesTab file picker flow wired end-to-end |
| SMPL-03 | 05-01, 05-03 | System validates Flex vs Static slot type correctness when assigning samples | SATISFIED | compute_sample_dry_run validates slot_type enum; hard block for Flex >200MB with redirect suggestion; format validation per D-14 |
| INTG-01 | 05-02, 05-04 | User can search Wallflower sample library by key, BPM, tags from within Takoyaki | SATISFIED | search_wallflower_samples uses parameterized JOIN query; WallflowerPanel renders results; 300ms debounce per D-12 |
| INTG-02 | 05-02, 05-04 | User can preview sample metadata from Wallflower and push selected samples to OT slots | SATISFIED | WallflowerSampleRow shows filename/key/BPM/tags; SlotPickerDialog -> handleSlotPickerConfirm -> DryRunModal -> assignSample(fromWallflower=true) wires full push-to-slot flow |
| INTG-03 | 05-02, 05-04 | Wallflower integration degrades gracefully when Wallflower is not installed or database is unavailable | SATISFIED | get_wallflower_status returns connected:false when no DB; wallflowerConnected=false hides WallflowerPanel entirely; no error or empty state shown |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|---------|--------|
| samples.rs | 148 | FIXME: get_project_samples stub (pre-existing Phase 2) | Info | SlotPickerDialog shows all slots as empty (occupied/empty chips non-functional). Pre-existing issue, not Phase 5 regression — documented in Plan 01 SUMMARY |
| SlotRow.tsx | ~244 | Dismiss button onClick is no-op (only calls e.stopPropagation, never calls clearSlotError) | Warning | When a slot-type mismatch error shows both a redirect button AND a Dismiss button, clicking Dismiss does not clear the error. Only the redirect button works. Format errors (no redirect) show no Dismiss button at all, so they cannot be cleared without reloading. Flagged as WR-01 in code review |
| samples.rs | 407–541 | assign_sample does not independently validate audio format (trusts dry-run was called) | Warning | A direct IPC call bypassing dry-run could assign a non-audio file. Flagged as WR-02 in code review. Defense-in-depth gap, not a typical user flow |
| samples.rs | 457–459 | Non-atomic Wallflower file copy using std::fs::copy (WR-03) | Warning | If USB disconnects during copy, partial file remains on OT card. Project.work not modified yet (copy happens first), so slot won't reference corrupt file — but partial file exists on card |
| samples.rs | 451–455 | Silently skips Wallflower copy when destination exists (WR-04) | Warning | If Wallflower source and destination have same filename but different content, the OT card keeps the stale file silently |

### Human Verification Required

#### 1. End-to-End Desktop Assignment Flow (SMPL-01, SMPL-03)

**Test:** Launch app (`cargo tauri dev`), connect OT in USB disk mode, confirm device, navigate to any project Samples tab. Click the Upload button on any Flex slot row. Verify a macOS file picker opens filtered to WAV/AIF/AIFF. Select a valid WAV file. Verify the dry-run preview modal appears listing `project.work` and `project.strd` as Modified with snapshot guarantee text. Click "Assign Sample". Verify the green success banner appears and auto-dismisses after ~4 seconds. Verify the slot now shows the assigned filename.

**Expected:** Modal shows 2 affected files (project.work + project.strd), assignment succeeds, slot updates in the list.

**Why human:** Tauri native file dialog requires real macOS interaction; modal content is visual; slot refresh requires live data.

#### 2. Format Validation Error (SMPL-03)

**Test:** Try assigning a non-WAV file (e.g., an MP3 or text file) to any slot.

**Expected:** Inline error appears below the slot row: "Unsupported format: OT accepts WAV and AIFF only. Convert this file first." No modal opens.

**Why human:** File selection requires macOS interaction; inline error display and positioning is visual.

#### 3. Flex Size Block with Redirect (SMPL-03)

**Test:** Try assigning a file >200MB to a Flex slot. Also test the Dismiss button on the resulting error — this verifies or disproves WR-01.

**Expected:** Error shows "This sample is too large for a Flex slot. Assign to Static instead." with an amber "Assign to Static #NNN" redirect button. Clicking Dismiss should clear the error (WR-01 risk: it may not).

**Why human:** Requires a large audio file; WR-01 Dismiss button behavior requires interaction testing.

#### 4. Wallflower Panel Visibility (INTG-01, INTG-02, INTG-03)

**Test (connected):** With Wallflower installed, verify the "WALLFLOWER LIBRARY" panel appears below the Static slots section, default expanded. Type a search term and verify results update after ~300ms delay. Verify rows show filename, key, BPM, and tag badges. Click "Push" on a sample and verify the Slot Picker dialog opens with FLEX/STATIC toggle and 128 slots listing occupied/empty status.

**Test (disconnected):** Remove the Wallflower DB (or test on a machine without Wallflower). Verify the panel is completely absent — no trigger bar, no error, no empty state.

**Expected connected:** Panel visible, search debounces 300ms, slot picker shows correct structure. **Expected disconnected:** Panel entirely absent.

**Why human:** Requires Wallflower installation state control; visual layout; 300ms timing is perceptual; slot picker occupied status depends on pre-existing Phase 2 stub (all slots will show as empty).

#### 5. Push-to-Slot Flow (INTG-02)

**Test:** Click Push on a Wallflower sample, select a slot in the slot picker, click "Assign to Slot". Verify the dry-run modal appears. Confirm. Verify the success banner shows "copied to /AUDIO/" suffix.

**Expected:** Full flow completes; success banner shows the Wallflower-specific copy message.

**Why human:** Requires Wallflower installation; OT device connection; success banner copy requires visual inspection.

#### 6. Settings Wallflower Section (INTG-03)

**Test:** Click "Settings" in the sidebar. Verify the Wallflower section shows a green CircleCheck icon when connected (or grey CircleMinus when not). Click "Change..." and verify the file picker opens filtered to .db files.

**Expected:** Icon matches connection state; file picker opens for .db files.

**Why human:** Visual icon check; file picker filter requires macOS interaction.

### Gaps Summary

No automated gaps block goal achievement — all 4 roadmap success criteria are verified programmatically through code inspection and passing tests. The `human_needed` status reflects that Plan 04 included a mandatory human verification checkpoint (Task 3, `checkpoint:human-verify`, `gate: blocking`) that has not yet been completed.

**Known issues from code review (WR-01 through WR-04):** Four warnings from the code review represent quality concerns but do not prevent the core goal from being achieved:
- WR-01 (Dismiss button non-functional) is a UX gap that makes format errors unresolvable without reloading
- WR-02 (no independent format validation in assign_sample) is a defense-in-depth gap for direct IPC access
- WR-03 (non-atomic Wallflower copy) is a data safety concern for edge-case USB disconnect scenarios
- WR-04 (silent skip on existing destination) can silently use stale Wallflower samples

These are tracked in the code review (05-REVIEW.md) and should be evaluated before marking Phase 5 complete.

**Pre-existing Phase 2 stub:** `get_project_samples` returns 128 empty stub slots. This causes the SlotPickerDialog to show all slots as empty (occupied/empty chips are non-functional). This predates Phase 5 and is documented. It degrades but does not block the Wallflower push-to-slot flow (the assignment itself writes the correct bytes regardless of the display).

---

_Verified: 2026-05-02T14:00:00Z_
_Verifier: Claude (gsd-verifier)_
