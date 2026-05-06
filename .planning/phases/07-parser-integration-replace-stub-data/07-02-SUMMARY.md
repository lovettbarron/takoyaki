---
phase: 07-parser-integration-replace-stub-data
plan: 02
subsystem: tauri-commands
tags: [parser-wiring, stub-replacement, health-check, integration-tests]
dependency_graph:
  requires: [parse_project_work, ParsedProjectWork, realistic-fixtures]
  provides: [real-slot-data, real-tempo, real-bank-checks, detc03-suppression]
  affects: [commands/samples.rs, commands/projects.rs, commands/health.rs, health/mod.rs]
tech_stack:
  added: []
  patterns: [lock-drop-before-io, infallible-fallback-reads, detc03-suppression-guard]
key_files:
  created: []
  modified:
    - crates/takoyaki-app/src/commands/samples.rs
    - crates/takoyaki-app/src/commands/projects.rs
    - crates/takoyaki-app/src/commands/health.rs
    - crates/takoyaki-app/src/health/mod.rs
    - crates/takoyaki-app/src/lib.rs
    - crates/takoyaki-app/Cargo.toml
    - crates/takoyaki-app/tests/project_detail.rs
    - crates/takoyaki-app/tests/health_check.rs
decisions:
  - "commands module made pub in lib.rs to allow integration tests to access parse_project_work"
  - "parse_project_work and ParsedProjectWork changed from pub(crate) to pub for integration test access"
  - "DETC-03 suppressed via !track_references.is_empty() guard -- original logic preserved for future bank parser"
  - "tokio macros+rt features added to enable #[tokio::test] for async integration tests"
metrics:
  duration: 413s
  completed: "2026-05-06T19:40:31Z"
  tasks: 3
  files: 8
---

# Phase 7 Plan 02: Wire Parse Into Tauri Commands Summary

**One-liner:** Replaced all stub/empty data in get_project_samples, get_project_detail, get_project_banks, and run_health_check with real parse_project_work output, suppressed DETC-03 false-positive flood, and activated all ignored integration tests.

## What Was Done

### Task 1: Wire parse_project_work into get_project_samples, get_project_detail, and get_project_banks
- Replaced `parse_sample_slots` (wrong `[SAMPLE]...[/SAMPLE]` format) with `parse_project_work` in `get_project_samples`
- Removed `ParsedSampleEntry` struct and `parse_sample_slots` function entirely
- Replaced `is_bank_populated_stub` (index < bank_count heuristic) with `is_bank_populated` using `ot_parser::BankFile::from_bytes` validation
- Wired real tempo extraction from project.work `[SETTINGS]/TEMPO:` into `get_project_detail`, with fallback to SQLite value
- Made `commands` module `pub` in `lib.rs` for integration test access
- Added tokio `macros` and `rt` features for `#[tokio::test]` support

### Task 2: Wire health check with real slot inputs and suppress DETC-03 false-positive flood
- Replaced empty `Vec::new()` stub in `run_health_check` with real `Vec<SlotCheckInput>` built from `parse_project_work` output
- Builds 128 flex + 128 static slot inputs with real occupied/path data from project.work
- Suppressed DETC-03 unused sample detection when `track_references` is empty (bank body opaque -- cannot determine track-to-slot references)
- Original DETC-03 logic preserved verbatim behind `!track_references.is_empty()` guard for future bank parser activation

### Task 3: Un-ignore integration tests and implement test bodies
- Rewrote `project_detail.rs` with 3 working tests: samples from fixture, tempo from fixture, bank file validity check
- Rewrote `health_check.rs` replacing 2 `#[ignore]`/`todo!()` tests with async implementations: missing file detection (DETC-01), DETC-03 suppression verification
- All 126 tests pass (100 lib + 26 integration), 0 failures, 0 ignored

## Commits

| Task | Commit | Type | Message |
|------|--------|------|---------|
| 1 | 6cf296b | feat | wire parse_project_work into get_project_samples, get_project_detail, and get_project_banks |
| 2 | 3ead099 | feat | wire health check with real slot inputs and suppress DETC-03 false-positive flood |
| 3 | 9e72904 | test | un-ignore integration tests and implement test bodies |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Made commands module pub and parse_project_work pub**
- **Found during:** Task 1
- **Issue:** Integration tests in `tests/` directory are external crates that cannot access `pub(crate)` items through private `mod commands`. The plan specified `pub(crate)` visibility from Plan 01.
- **Fix:** Changed `mod commands` to `pub mod commands` in `lib.rs` and changed `pub(crate)` to `pub` on `ParsedProjectWork` and `parse_project_work`.
- **Files modified:** `crates/takoyaki-app/src/lib.rs`, `crates/takoyaki-app/src/commands/samples.rs`
- **Commit:** 6cf296b

**2. [Rule 3 - Blocking] Added tokio macros and rt features**
- **Found during:** Task 1 (preparation for Task 3)
- **Issue:** `tokio` dependency only had `time` feature. `#[tokio::test]` requires `macros` and `rt` features.
- **Fix:** Added `macros` and `rt` to tokio features in `Cargo.toml`.
- **Files modified:** `crates/takoyaki-app/Cargo.toml`
- **Commit:** 6cf296b

## Threat Mitigations Applied

| Threat ID | Mitigation | Verified |
|-----------|------------|----------|
| T-07-04 | Slot paths resolved through existing `health::resolve_ot_path()` with canonicalize + volume containment check (unchanged) | Yes -- test_health_missing_file passes |
| T-07-05 | Bank file reads bounded by 16 bank max per project; small files (<100KB each) | Yes -- is_bank_populated reads at most 16 files |
| T-07-06 | parse_project_work is infallible; file read uses unwrap_or_default() for missing files | Yes -- all lib tests pass |
| T-07-07 | card_path in tracing::debug only; same pattern as existing code; no PII, local app | Yes -- debug log pattern preserved |

## Known Stubs

None -- all stubs replaced with real parser output. The following are known limitations (not stubs):
- Bank names, part names, machine types remain None/"Thru" -- bank body is opaque, not parseable in Phase 7
- DETC-03 track references always empty -- bank body opaque, suppression guard active

## Self-Check: PASSED

- [x] `crates/takoyaki-app/src/commands/samples.rs` contains `parse_project_work(&raw)` in get_project_samples
- [x] `crates/takoyaki-app/src/commands/samples.rs` does NOT contain `fn parse_sample_slots`
- [x] `crates/takoyaki-app/src/commands/projects.rs` contains `fn is_bank_populated`
- [x] `crates/takoyaki-app/src/commands/projects.rs` does NOT contain `fn is_bank_populated_stub`
- [x] `crates/takoyaki-app/src/commands/health.rs` contains `crate::commands::samples::parse_project_work`
- [x] `crates/takoyaki-app/src/health/mod.rs` contains `if !slot.track_references.is_empty()`
- [x] `crates/takoyaki-app/tests/project_detail.rs` does NOT contain `#[ignore` or `todo!(`
- [x] `crates/takoyaki-app/tests/health_check.rs` does NOT contain `#[ignore` or `todo!(`
- [x] Commit 6cf296b exists in git log
- [x] Commit 3ead099 exists in git log
- [x] Commit 9e72904 exists in git log
