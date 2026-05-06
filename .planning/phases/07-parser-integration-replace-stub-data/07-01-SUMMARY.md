---
phase: 07-parser-integration-replace-stub-data
plan: 01
subsystem: ot-parser
tags: [parser, tdd, project-work, samples, fixtures]
dependency_graph:
  requires: []
  provides: [parse_project_work, ParsedProjectWork, realistic-fixtures]
  affects: [commands/samples.rs, commands/projects.rs, commands/health.rs]
tech_stack:
  added: []
  patterns: [section-state-machine, infallible-parsing, bounds-checked-indexing]
key_files:
  created: []
  modified:
    - crates/takoyaki-app/src/commands/samples.rs
    - tests/fixtures/mock_ot_volume/SETS/LIVESET/PROJECT_01/project.work
    - tests/fixtures/project.work
decisions:
  - "parse_project_work placed in commands/samples.rs as pub(crate) -- accessible to projects.rs and health.rs via crate-internal import"
  - "TEMPO fixture value corrected from 12000 to 1200 (120 BPM at TEMPO_SCALE_FACTOR=10.0)"
  - "Parser uses unwrap_or(999) for slot index parse failures -- silent rejection of malformed indices"
metrics:
  duration: 193s
  completed: "2026-05-06T19:30:21Z"
  tasks: 2
  files: 3
---

# Phase 7 Plan 01: Real project.work Text Parser (TDD) Summary

**One-liner:** Section-state-machine parser for OT project.work FLEX0:/STAT0: slot format with bounds checking and infallible error handling, replacing the stub parse_sample_slots format assumption.

## What Was Done

### Task 1: RED -- Write failing tests for parse_project_work()
- Added `ParsedProjectWork` struct with `tempo_raw`, `flex_slots[128]`, `static_slots[128]`
- Added stub `parse_project_work()` that returns all-None defaults
- Added 8 unit tests covering: occupied flex/static slots, empty slot, tempo extraction, no tempo, bounds check (FLEX999), empty input, multiple sections
- RED verification: 4 tests failed (occupied slots + tempo), 4 passed (empty/none cases)
- All 92 pre-existing tests continued to pass

### Task 2: GREEN -- Implement parse_project_work() and update fixtures
- Replaced stub with real section-state machine parser using `String::from_utf8_lossy`
- Parser iterates lines, tracks `[SETTINGS]`/`[SLOTS]` section state
- Extracts `TEMPO:` value from `[SETTINGS]` section
- Extracts `FLEX0:`..`FLEX127:` and `STAT0:`..`STAT127:` paths from `[SLOTS]` section
- Empty paths (e.g., `FLEX0:`) correctly map to `None`
- Bounds check: `if idx < 128` prevents out-of-bounds array writes (T-07-01)
- Infallible: `parse().ok()` and `unwrap_or(999)` prevent panics on malformed input (T-07-02)
- Updated mock_ot_volume fixture with realistic content: `FLEX0:../AUDIO/kick_44100.wav`, `STAT0:../AUDIO/pad_48000.wav`, `TEMPO:1200`
- Fixed main fixture `tests/fixtures/project.work` TEMPO from 12000 to 1200 (120 BPM at /10.0 scale factor)
- GREEN verification: all 8 tests pass, 100 total tests pass (0 failures)

## TDD Gate Compliance

- RED gate: `test(07-01)` commit `4cf0c86` -- 4 tests failing, 4 passing
- GREEN gate: `feat(07-01)` commit `bf316f8` -- all 8 tests passing
- REFACTOR gate: Not needed -- implementation is clean, no refactoring required

## Commits

| Task | Commit | Type | Message |
|------|--------|------|---------|
| 1 | 4cf0c86 | test | add failing tests for parse_project_work |
| 2 | bf316f8 | feat | implement parse_project_work and update fixtures |

## Deviations from Plan

None -- plan executed exactly as written.

## Threat Mitigations Applied

| Threat ID | Mitigation | Verified |
|-----------|------------|----------|
| T-07-01 | `if idx < 128` bounds check on slot index before array write | Yes -- test_parse_project_work_bounds_check passes |
| T-07-02 | Infallible parsing: `parse().ok()`, `unwrap_or(999)`, `String::from_utf8_lossy` | Yes -- test_parse_project_work_empty_input passes |
| T-07-03 | Path traversal accepted (parser reads paths as strings only) | N/A -- resolution uses existing resolve_ot_path() |

## Known Stubs

None -- `parse_project_work()` is fully implemented, not stubbed.

## Self-Check: PASSED

- [x] `crates/takoyaki-app/src/commands/samples.rs` exists and contains `pub(crate) fn parse_project_work`
- [x] `tests/fixtures/mock_ot_volume/SETS/LIVESET/PROJECT_01/project.work` contains `FLEX0:../AUDIO/kick_44100.wav`
- [x] `tests/fixtures/project.work` contains `TEMPO:1200`
- [x] Commit 4cf0c86 exists in git log
- [x] Commit bf316f8 exists in git log
