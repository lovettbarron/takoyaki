---
phase: 04-advanced-management
verified: 2026-05-02T00:00:00Z
status: passed
score: 4/4 must-haves verified
overrides_applied: 0
gaps: []
human_verification:
  - test: "Rename operation end-to-end"
    expected: "Clicking Rename in project detail, typing a new name, pressing Enter shows dry-run preview 'Rename OLD -> NEW', confirming renames the directory visible in the project list"
    why_human: "Directory rename on a mounted OT volume and navigation state refresh require a running Tauri app"
  - test: "Duplicate operation end-to-end"
    expected: "Clicking Duplicate shows dry-run preview with list of files to be created, confirming creates the new project visible in the project list"
    why_human: "Requires mounted OT volume with real project data; verifying the new project is loadable"
  - test: "Export creates a valid playable zip"
    expected: "Clicking Export shows dry-run listing project files + audio files, confirming writes ~/takoyaki/exports/PROJECT_NNNN.zip containing SETS/ and AUDIO/ structure openable by unzip"
    why_human: "Requires verifying the zip file can be extracted and produces a valid OT structure; filesystem write to ~/takoyaki/exports/"
  - test: "Bank copy shows conflict entries in dry-run when filenames collide with different content"
    expected: "If source and target projects share a slot with same filename but different audio content, dry-run shows a Conflict entry for that file"
    why_human: "Requires real OT project data with known hash-mismatching files; conflict display depends on DryRunModal rendering ChangeType.Conflict entries"
---

# Phase 4: Advanced Management Verification Report

**Phase Goal:** Users can perform the full range of project management operations — duplicate, rename, export, copy banks across projects — with the same safety guarantees as Phase 3.
**Verified:** 2026-05-02
**Status:** passed
**Re-verification:** Yes — gap closure plans 04-06 and 04-07 executed

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|---------|
| 1 | User can duplicate an OT project and have all sample paths correctly remapped to the new project directory | VERIFIED | `duplicate.rs` copies entire project directory tree via WalkDir; `../AUDIO/` relative paths are valid from any `/SETS/PROJECT/` location — no rewriting needed per OT architecture. 5 unit tests pass including content preservation test. |
| 2 | User can rename an OT project directory on disk; no binary header modification required | VERIFIED | Plan 04-06 corrected SC-2 wording to match reality. ROADMAP.md and REQUIREMENTS.md now accurately state directory rename only. Implementation in `rename.rs` verified correct. |
| 3 | User can export a project as a self-contained zip with all referenced audio samples collected inside | VERIFIED | `export.rs` creates zip with `SETS/{project_name}/` tree + `AUDIO/` files + `.ot` sidecars. `zip.finish()` called (Pitfall 2 avoided). Stored compression for WAV/AIFF. 5 unit tests pass including zip validity and audio inclusion checks. |
| 4 | User can copy a bank from one project to another with sample slots automatically remapped and conflicts surfaced for resolution | VERIFIED | Plan 04-07 added ConflictResolutionDialog (186 lines). Rust FileChangeManifest extended with `conflict_details`. Page.tsx now shows per-conflict resolution UI (keep-target/use-source/rename-incoming) when hash mismatches detected. `copyBank` receives resolved conflict map instead of `{}`. |

**Score:** 4/4 truths verified (SC-2 corrected by 04-06; SC-4 completed by 04-07)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `crates/takoyaki-app/src/management/mod.rs` | Module scaffold with 5 submodule declarations | VERIFIED | Declares project_work, rename, duplicate, export, bank_copy |
| `crates/takoyaki-app/src/management/project_work.rs` | project.work text parser + OT name validation | VERIFIED | 313 lines; exports extract_slot_paths, rewrite_slot_path, validate_ot_name, SlotType, SlotPath; 7 unit tests |
| `crates/takoyaki-app/src/management/rename.rs` | Project rename business logic | VERIFIED | 124 lines; rename_project validates name, checks collision, calls std::fs::rename; 4 unit tests |
| `crates/takoyaki-app/src/management/duplicate.rs` | Project duplication with path preservation | VERIFIED | 231 lines; duplicate_project + compute_default_name + DuplicateResult; WalkDir copy; 5 unit tests |
| `crates/takoyaki-app/src/management/export.rs` | Export to self-contained zip | VERIFIED | 417 lines; export_project + compute_export_dest + ExportResult; ZipWriter with Stored/Deflated; zip.finish(); 5 unit tests |
| `crates/takoyaki-app/src/management/bank_copy.rs` | Bank copy with conflict detection | VERIFIED | ~680 lines; copy_bank + compute_bank_copy_conflicts + SlotConflict + BankCopyAnalysis + BankCopyResult; SHA-256 comparison; 4 unit tests |
| `crates/takoyaki-app/src/commands/management.rs` | 5 Tauri IPC commands | VERIFIED | 5 commands with #[tauri::command] #[specta::specta]; ManagementEvent enum; SnapshotEngine pre-op snapshots; DB lock release before I/O |
| `src/lib/types.ts` | ManagementOperation, ManagementEvent, ConflictResolution, ConflictEntry types | VERIFIED | Contains all 4 types; ChangeType extended with "Conflict" |
| `src/lib/stores/management.ts` | useManagementStore zustand store | VERIFIED | Full lifecycle: idle -> dry-running -> in-progress -> complete -> failed; all actions present |
| `src/lib/tauri.ts` | 5 management IPC wrappers | VERIFIED | computeManagementDryRun, duplicateProject, renameProject, exportProject, copyBank all present with correct signatures |
| `src/components/ui/context-menu.tsx` | shadcn context-menu component | VERIFIED | Installed via shadcn CLI; exports ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger |
| `src/components/project-detail/MetadataHeader.tsx` | Toolbar buttons + inline rename | VERIFIED | Rename/Duplicate/Export ghost buttons; inline input with /[^A-Z0-9_]/g filter; maxLength=16; Enter/Escape/blur handling |
| `src/components/project-detail/BankGridCell.tsx` | Right-click context menu | VERIFIED | ContextMenu wraps existing button; content only when populated=true; "Copy to project..." with ArrowRightFromLine icon |
| `src/components/management/BankCopyPickerDialog.tsx` | Two-step bank copy picker | VERIFIED | Step 1: project list with empty state "No other projects on this card."; Step 2: 4x4 grid; overwrite warning; sourceProjectId prop filters current project |
| `src/app/page.tsx` | Management operation orchestration | VERIFIED | useManagementStore wired; all 5 handlers (handleRename, handleDuplicate, handleExport, handleBankCopyTrigger, handleBankCopyConfirm); second DryRunModal for management; success banner with 4s auto-dismiss |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `duplicate.rs` | `project_work.rs` | validate_ot_name | WIRED | Line 65: project_work::validate_ot_name called first |
| `rename.rs` | `project_work.rs` | validate_ot_name | WIRED | Line 31: project_work::validate_ot_name called first |
| `export.rs` | `project_work.rs` | extract_slot_paths | WIRED | Line 88: project_work::extract_slot_paths called on project.work bytes |
| `bank_copy.rs` | `project_work.rs` | extract_slot_paths + rewrite_slot_path | WIRED | Lines 77-78 (extract); lines 313-319 (rewrite for auto-copy) |
| `commands/management.rs` | `management::export` | export_project call | WIRED | Calls management::export::export_project |
| `commands/management.rs` | `management::bank_copy` | copy_bank call | WIRED | Calls management::bank_copy::copy_bank |
| `lib.rs` | `commands::management` | collect_commands! registration | WIRED | Lines 47-51: all 5 commands registered |
| `page.tsx` | `tauri.ts` | computeManagementDryRun + execute calls | WIRED | All 5 IPC functions imported and called in handlers |
| `page.tsx` | `stores/management.ts` | useManagementStore | WIRED | Lines 59-72: full store destructuring and usage |
| `MetadataHeader.tsx` | `page.tsx` | onRename/onDuplicate/onExport callbacks | WIRED | Props passed at line 432-435 in page.tsx |
| `BankGridCell.tsx` | `page.tsx` | onCopyToProject -> handleBankCopyTrigger | WIRED | Via BanksTab -> ProjectDetailView -> page.tsx callback chain |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|--------------------|--------|
| `BankCopyPickerDialog.tsx` | `targetBanks` | `getProjectBanks(selectedProjectId)` IPC call on step advance | Yes — fetches real bank data from Rust backend | FLOWING |
| `BankCopyPickerDialog.tsx` | `otherProjects` | `projects` prop from page.tsx (populated from `listProjects({})` on device confirm) | Yes — real project list from indexed DB | FLOWING |
| `page.tsx` management dry-run | `mgmtDryRunManifest` | `computeManagementDryRun()` IPC returning `FileChangeManifest` | Yes — walks actual project directory for entries | FLOWING |
| `copyBank` (page.tsx) | `conflictResolutions` | ConflictResolutionDialog `onResolve` callback | Yes — user-selected per-conflict resolutions | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All management unit tests pass | `cargo test -p takoyaki-app management` | 26 passed, 0 failed | PASS |
| Rust compilation clean | `cargo check -p takoyaki-app` | 0 errors, 2 pre-existing warnings | PASS |
| TypeScript compilation clean | `npx tsc --noEmit` | 0 errors | PASS |
| Context menu only on populated cells | Code inspection: `{populated && <ContextMenuContent>}` | Content gated on populated=true | PASS |
| export.rs calls zip.finish() | Code inspection | Line 217: `zip.finish()` before return | PASS |
| Rename validates name before filesystem op | Code inspection | `project_work::validate_ot_name(new_name)?` first line | PASS |

### Requirements Coverage

| Requirement | Source Plans | Description | Status | Evidence |
|-------------|-------------|-------------|--------|---------|
| MGMT-01 | Plans 01, 02, 03, 04 | User can duplicate/copy an OT project with automatic sample path remapping | SATISFIED | `duplicate_project` copies tree; relative paths valid from new location; IPC command wired; UI trigger present |
| MGMT-02 | Plans 01, 02, 03, 04, 06 | User can rename an OT project directory on disk (directory name is authoritative) | SATISFIED | Plan 04-06 corrected REQUIREMENTS.md MGMT-02 to reflect reality. `rename_project` correctly renames directory. No binary header field exists (research A3). |
| MGMT-03 | Plans 01, 02, 03, 04 | User can export a project as a self-contained zip with all referenced samples collected | SATISFIED | `export_project` creates SETS/ + AUDIO/ zip with sidecars; verified by test |
| SMPL-02 | Plans 01, 02, 03, 04, 07 | User can copy banks between projects with automatic sample slot remapping and conflict resolution | SATISFIED | Plan 04-07 added ConflictResolutionDialog. Users can choose per-conflict resolution (keep-target/use-source/rename-incoming) before applying. Resolved map passed to Rust backend. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src/app/page.tsx` | 310 | `copyBank(..., resolutions, channel)` — resolved by 04-07 | Resolved | ConflictResolutionDialog now provides per-conflict resolution before copyBank is called |
| `crates/takoyaki-app/src/management/bank_copy.rs` | 496 | Comment "Write bank01.work placeholder" | Info | Inside test helper function only; not production code |
| `src/components/project-detail/MetadataHeader.tsx` | 73 | `placeholder="Project name..."` | Info | Intentional UX — input placeholder text, not a stub |

### Human Verification Required

#### 1. Rename Workflow End-to-End

**Test:** Open a project in project detail view. Click "Rename" button. Type a new valid name (e.g., "NEW_NAME"). Press Enter. Verify dry-run modal appears with label "Rename OLD_NAME -> NEW_NAME". Click Apply.
**Expected:** Project directory is renamed on the OT card; project list refreshes showing the new name; snapshot exists in ~/takoyaki/snapshots/
**Why human:** Requires mounted OT card with real project; navigation store refresh after rename not verifiable statically

#### 2. Duplicate Workflow End-to-End

**Test:** Click "Duplicate" button. Verify dry-run shows list of files to be created. Click Apply.
**Expected:** New project appears in project list with _COPY suffix; both projects loadable with valid bank/slot data
**Why human:** Requires mounted OT card; verifying duplicated project is fully functional in the OT

#### 3. Export Creates Valid OT-Playable Zip

**Test:** Click "Export" button. Confirm dry-run. Verify ~/takoyaki/exports/{PROJECT}_{TIMESTAMP}.zip exists.
**Expected:** Zip contains SETS/{PROJECT}/ directory with project files AND AUDIO/ with all referenced wav/aiff files. Unzipping to a blank CF card and loading in OT MkII produces a playable project.
**Why human:** OT playability verification requires actual hardware; zip structure inspection requires manual extraction

#### 4. Bank Copy Conflict Display in Dry-Run

**Test:** Set up two projects with the same audio filename in the same slot but different content. Right-click a populated bank, select "Copy to project...", choose the target project and bank slot. Click "Copy Bank".
**Expected:** Dry-run modal shows a Conflict entry for the colliding file (not just Added or Unchanged).
**Why human:** Requires specific project fixture with known hash-mismatching files; conflict display depends on DryRunModal rendering Conflict change type entries

#### 5. Conflict Resolution Gap (Functional Concern)

**Test:** When a bank copy dry-run shows Conflict entries, click Apply.
**Expected per SC-4:** User should be able to choose "keep-target", "use-source", or "rename-incoming" before applying.
**Actual behavior:** Conflicts are silently treated as "keep-target" with no user choice.
**Why human:** Confirm whether this gap is acceptable for initial release or requires the conflict resolution UI to be implemented before shipping Phase 4.

### Gaps Summary

All gaps resolved by gap closure plans:

**Gap 1 — SC-2 wording (MGMT-02): RESOLVED by Plan 04-06.** ROADMAP.md SC-2 and REQUIREMENTS.md MGMT-02 updated to accurately state that OT project rename is directory-only — no binary header contains a name field.

**Gap 2 — Conflict resolution UI (SMPL-02, SC-4): RESOLVED by Plan 04-07.** ConflictResolutionDialog (186 lines) created. Rust FileChangeManifest extended with `conflict_details` field populated in bank-copy dry-run. Page.tsx now shows per-conflict resolution UI when hash mismatches detected, passing resolved map to `copyBank` instead of empty `{}`.

---

_Verified: 2026-05-01_
_Verifier: Claude (gsd-verifier)_
