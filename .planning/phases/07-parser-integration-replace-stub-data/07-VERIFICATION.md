---
phase: 07-parser-integration-replace-stub-data
verified: 2026-05-06T19:46:47Z
status: human_needed
score: 4/5 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Open the app with a real OT volume mounted. Navigate to a project detail view and confirm tempo displays correctly (not 0 or a stub value)."
    expected: "Tempo BPM should reflect the actual project tempo from the OT card."
    why_human: "Cannot verify Tauri IPC round-trip or real OT volume data without running the app."
  - test: "Open a project and click Samples tab. Verify that occupied slots show real filenames (e.g., kick_44100.wav) and empty slots show as empty."
    expected: "SlotPickerDialog and sample table should render real slot assignments, not 128 empty rows."
    why_human: "Visual verification of UI rendering real data from backend; grep cannot verify runtime rendering."
  - test: "Run health check on a project with a missing sample reference. Verify the health tab shows an Error for the missing file."
    expected: "Health issues list includes an Error-severity item for the missing sample reference."
    why_human: "Health check runs async via Tauri event emission; requires running app + event listener."
  - test: "Verify DETC-03 does NOT flood the health results with 'unused sample' Info items for every occupied slot."
    expected: "No 'assigned but not referenced by any track' messages in health results (suppression guard active)."
    why_human: "Requires running the full health check pipeline against a real or mock OT volume in the app."
---

# Phase 7: Parser Integration -- Replace Stub Data Verification Report

**Phase Goal:** Wire the Phase 1 binary parser into Phase 2 read commands so project detail, sample slots, and health checks return real parsed data instead of stubs.
**Verified:** 2026-05-06T19:46:47Z
**Status:** human_needed
**Re-verification:** No -- initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `get_project_samples` returns real slot assignments parsed from OT binary files, not 128 empty stubs | VERIFIED | `samples.rs:151` calls `parse_project_work(&raw)`, builds `SampleSlot` from `parsed.flex_slots` / `parsed.static_slots`. Integration test `test_get_project_samples_from_fixture` confirms FLEX0 = `../AUDIO/kick_44100.wav`, STAT0 = `../AUDIO/pad_48000.wav`. Old `parse_sample_slots` removed. |
| 2 | `get_project_detail` returns real tempo, bank names, and machine types from parsed binary data | PARTIAL | Tempo: VERIFIED -- `projects.rs:126` calls `parse_project_work`, divides `tempo_raw` by `TEMPO_SCALE_FACTOR`. Integration test confirms 1200 raw = 120.0 BPM. Bank names: NOT ACHIEVABLE -- research confirmed bank body is opaque; returns `None`. Machine types: NOT ACHIEVABLE -- bank body opaque; returns `"Thru"`. This is a technical impossibility documented in research, not a stub. |
| 3 | Health check scans actual slot inputs and detects real missing/incompatible samples | VERIFIED | `health.rs:56-88` reads `project.work`, builds 128 flex + 128 static `SlotCheckInput` from `parse_project_work`. Integration test `test_health_missing_file` confirms Error-severity issue for nonexistent file. DETC-02 tests confirm format validation works. |
| 4 | `is_bank_populated` derives from actual bank file parsing, not a stub integer | VERIFIED | `projects.rs:221-228` implements `is_bank_populated(project_dir, bank_index)` using `ot_parser::BankFile::from_bytes(&data).is_ok()`. Old `is_bank_populated_stub` removed (0 matches). Integration test `test_get_project_banks_bank_file_check` confirms placeholder bank file fails parse. |
| 5 | SlotPickerDialog shows real occupied/empty slot state | VERIFIED | Data flow traced: `SamplesTab.tsx:219` calls `getProjectSamples(projectId)` via `useQuery` -> passes `slots={samples}` to `SlotPickerDialog` at line 522. `SlotPickerDialog.tsx:106` renders `slot.occupied` with amber "occupied" / muted "empty" chips. Since backend now returns real data from `parse_project_work`, the dialog will render real state. |

**Score:** 4/5 truths verified (SC #2 is partial -- tempo verified, bank names/machine types are technically not achievable)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `crates/takoyaki-app/src/commands/samples.rs` | `parse_project_work` function and `ParsedProjectWork` struct | VERIFIED | Lines 678-738: full section-state-machine parser with bounds check `if idx < 128`, infallible via `parse().ok()` and `unwrap_or(999)`. `pub fn parse_project_work` at line 696. |
| `crates/takoyaki-app/src/commands/projects.rs` | Real `is_bank_populated` using `BankFile::from_bytes`, real tempo from `parse_project_work` | VERIFIED | `is_bank_populated` at line 221 using `ot_parser::BankFile::from_bytes`. Tempo at line 126-129 via `parse_project_work`. No `is_bank_populated_stub` (0 matches). |
| `crates/takoyaki-app/src/commands/health.rs` | Real `SlotCheckInput` built from `parse_project_work` | VERIFIED | Lines 56-88: reads `project.work`, calls `parse_project_work`, builds 256 slot inputs with real `occupied` and `raw_path` values. |
| `crates/takoyaki-app/src/health/mod.rs` | DETC-03 suppression guard | VERIFIED | Line 383: `if !slot.track_references.is_empty()` guard prevents false-positive flood. Comment documents Phase 7 limitation. |
| `crates/takoyaki-app/tests/project_detail.rs` | Integration tests without `#[ignore]` or `todo!()` | VERIFIED | 3 tests: `test_get_project_samples_from_fixture`, `test_get_project_detail_tempo_from_fixture`, `test_get_project_banks_bank_file_check`. No `#[ignore]` or `todo!()`. |
| `crates/takoyaki-app/tests/health_check.rs` | Integration tests without `#[ignore]` or `todo!()` | VERIFIED | 5 tests including `test_health_missing_file` and `test_health_unused_sample_suppressed_when_no_track_refs`. No `#[ignore]` or `todo!()`. |
| `tests/fixtures/mock_ot_volume/SETS/LIVESET/PROJECT_01/project.work` | Realistic fixture with occupied slots | VERIFIED | Contains `FLEX0:../AUDIO/kick_44100.wav`, `STAT0:../AUDIO/pad_48000.wav`, `TEMPO:1200`. |
| `tests/fixtures/project.work` | Corrected TEMPO value | VERIFIED | Contains `TEMPO:1200` (was 12000). |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `commands/samples.rs` `get_project_samples` | `parse_project_work()` | Direct call at line 151 | WIRED | `let parsed = parse_project_work(&raw);` -- replaces old `parse_sample_slots` |
| `commands/projects.rs` `get_project_detail` | `commands/samples::parse_project_work` | Cross-module import at line 126 | WIRED | `crate::commands::samples::parse_project_work(&raw)` |
| `commands/projects.rs` `get_project_banks` | `is_bank_populated` | Local function call at line 195 | WIRED | `is_bank_populated(&project_path, bank_index)` |
| `commands/projects.rs` `is_bank_populated` | `ot_parser::BankFile::from_bytes` | Direct call at line 225 | WIRED | `ot_parser::BankFile::from_bytes(&data).is_ok()` |
| `commands/health.rs` `run_health_check` | `commands/samples::parse_project_work` | Cross-module import at line 61 | WIRED | `crate::commands::samples::parse_project_work(&raw)` |
| `SlotPickerDialog.tsx` | `get_project_samples` backend | via `useQuery` in `SamplesTab.tsx:219` -> `slots` prop at line 522 | WIRED | `SamplesTab` fetches `getProjectSamples(projectId)` and passes `slots={samples}` to `SlotPickerDialog` |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|-------------------|--------|
| `commands/samples.rs` | `parsed.flex_slots` / `parsed.static_slots` | `std::fs::read(&file_to_read)` -> `parse_project_work(&raw)` | Yes -- reads actual project.work file from OT volume | FLOWING |
| `commands/projects.rs` | `display_tempo` | `std::fs::read(project_path.join("project.work"))` -> `parse_project_work` -> `tempo_raw / TEMPO_SCALE_FACTOR` | Yes -- reads actual project.work TEMPO field | FLOWING |
| `commands/projects.rs` | `populated` (banks) | `std::fs::read(&bank_path)` -> `BankFile::from_bytes(&data).is_ok()` | Yes -- reads actual bank files from disk | FLOWING |
| `commands/health.rs` | `slot_inputs` | `std::fs::read(project.work)` -> `parse_project_work(&raw)` -> 256 `SlotCheckInput` | Yes -- builds from actual project.work slot data | FLOWING |
| `SlotPickerDialog.tsx` | `slots` prop | `useQuery` -> `getProjectSamples(projectId)` -> Tauri IPC -> `get_project_samples` | Yes -- flows from backend parse_project_work | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All parse_project_work tests pass | `cargo test --package takoyaki-app test_parse_project_work` | 8 passed, 0 failed | PASS |
| Full package test suite green | `cargo test --package takoyaki-app` | 126 passed, 0 failed, 0 ignored | PASS |
| Old stubs removed (parse_sample_slots) | `grep -c "parse_sample_slots" samples.rs` | 0 (only in comment on line 669) | PASS |
| Old stubs removed (is_bank_populated_stub) | `grep -c "is_bank_populated_stub" projects.rs` | 0 | PASS |
| Real parser wired in samples.rs | `grep -c "parse_project_work" samples.rs` | Multiple matches (definition + tests + calls) | PASS |
| Real parser wired in projects.rs | `grep "parse_project_work" projects.rs` | Line 126: `crate::commands::samples::parse_project_work(&raw)` | PASS |
| Real parser wired in health.rs | `grep "parse_project_work" health.rs` | Line 61: `crate::commands::samples::parse_project_work(&raw)` | PASS |
| DETC-03 suppression guard present | `grep "track_references.is_empty" health/mod.rs` | Line 383: `if !slot.track_references.is_empty()` | PASS |
| Integration tests have no ignored tests | `cargo test --package takoyaki-app 2>&1 \| grep ignored` | All test suites show `0 ignored` | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-----------|-------------|--------|----------|
| BROW-03 | 07-02 | User can view which banks and patterns are populated within a project | SATISFIED | `is_bank_populated` uses `BankFile::from_bytes` to check actual bank files. Integration test `test_get_project_banks_bank_file_check` validates. |
| BROW-04 | 07-01, 07-02 | User can view all Flex and Static sample slots with assigned file paths | SATISFIED | `get_project_samples` returns real slot data from `parse_project_work`. Integration test `test_get_project_samples_from_fixture` validates. |
| BROW-05 | 07-02 | User can view project-level metadata including tempo, bank names, part names, machine types | PARTIAL | Tempo: satisfied. Bank names, part names, machine types: bank body is opaque (research-documented impossibility). Returns `None`/`"Thru"`. |
| DETC-01 | 07-02 | User can detect missing or broken sample references | SATISFIED | Health check builds real `SlotCheckInput` from `parse_project_work`. Integration test `test_health_missing_file` confirms Error-severity for missing files. |
| DETC-02 | 07-02 | User can validate audio file format compatibility | SATISFIED | Format validation unchanged (already working from Phase 2). Integration tests `test_health_wrong_sample_rate` and `test_health_unsupported_format` confirm. |
| DETC-03 | 07-02 | User can detect unused samples | PARTIAL | DETC-03 check is suppressed when `track_references` is empty (bank body opaque). Logic preserved behind guard for future bank parser. Test `test_health_unused_sample_suppressed_when_no_track_refs` validates suppression. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `commands/projects.rs` | 90-92 | Stale doc comment: "parsers are not yet implemented... returns a stub ProjectDetail" -- function now returns real data | Info | Misleading documentation; no functional impact |
| `commands/health.rs` | - | Comment on line 669 of samples.rs references "replaces parse_sample_slots" -- the old function name in a comment, not code | Info | No functional impact |
| `health/mod.rs` | 382 | `TODO: Re-enable when bank body parser provides real track references` | Info | Intentional design marker for future work; not a stub |

### Human Verification Required

### 1. Real OT Volume Tempo Display

**Test:** Open the app with a real OT volume mounted. Navigate to a project detail view and confirm tempo displays correctly.
**Expected:** Tempo BPM should reflect the actual project tempo from the OT card (e.g., 120.0 BPM, not 0 or a placeholder).
**Why human:** Cannot verify Tauri IPC round-trip or real OT volume data without running the app.

### 2. Slot Picker Shows Real Data

**Test:** Open a project and click Samples tab. Verify that occupied slots show real filenames and empty slots show as empty.
**Expected:** SlotPickerDialog and sample table should render real slot assignments, not 128 empty rows.
**Why human:** Visual verification of UI rendering real data from backend; static analysis cannot verify runtime rendering.

### 3. Health Check Detects Missing Files

**Test:** Run health check on a project with a missing sample reference. Verify the health tab shows an Error for the missing file.
**Expected:** Health issues list includes an Error-severity item for the missing sample reference.
**Why human:** Health check runs async via Tauri event emission; requires running app and event listener.

### 4. DETC-03 Suppression in Practice

**Test:** Verify DETC-03 does NOT flood the health results with "unused sample" Info items for every occupied slot.
**Expected:** No "assigned but not referenced by any track" messages in health results.
**Why human:** Requires running the full health check pipeline against a real or mock OT volume in the app.

### Gaps Summary

No blocking gaps found. All stubs have been replaced with real parser-backed implementations. The one partial truth (SC #2: bank names and machine types) is a documented technical impossibility -- the OT bank body is opaque binary and cannot be parsed for these fields without a deep bank body parser that is explicitly out of scope. This is not a stub or missing implementation; it is a format limitation confirmed by research.

The phase goal of "real parsed data instead of stubs" is achieved for all parseable data. Fields that are technically unparseable (bank names, part names, machine types from opaque bank body) are correctly documented as limitations with `None`/`"Thru"` placeholder values.

All 126 tests pass with 0 failures and 0 ignored. Test count increased from ~99 passing + 5 ignored to 126 passing + 0 ignored.

---

_Verified: 2026-05-06T19:46:47Z_
_Verifier: Claude (gsd-verifier)_
