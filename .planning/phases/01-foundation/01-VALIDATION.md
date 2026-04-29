---
phase: 1
slug: foundation
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-29
---

# Phase 1 -- Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness + cargo-nextest |
| **Config file** | none -- `cargo test` works; `cargo nextest run` for parallel |
| **Quick run command** | `cargo test -p ot-parser` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ot-parser`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

Task IDs follow `{plan}-{task}` convention matching plan filenames.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 03-T1 | 03 | 2 | FNDN-01, FNDN-02 | T-01-03 | Reject malformed input early | unit | `cargo test -p ot-parser test_sample` | Plan 03 creates | pending |
| 03-T2 | 03 | 2 | FNDN-03 | -- | N/A | unit | `cargo test -p ot-parser -- indexing` | Plan 03 creates | pending |
| 04-T0 | 04 | 3 | FNDN-01 | -- | N/A | -- | `test -f crates/ot-parser/format-spec.md` | Plan 04 creates | pending |
| 04-T1 | 04 | 3 | FNDN-01, FNDN-02 | T-01-05 | Validate file size before parse | unit | `cargo test -p ot-parser test_project test_bank` | Plan 04 creates | pending |
| 04-T2 | 04 | 3 | FNDN-01, FNDN-02 | T-01-07 | Validate magic bytes per file type | unit | `cargo test -p ot-parser test_markers test_arrangement test_all_types` | Plan 04 creates | pending |
| 05-T1 | 05 | 2 | FNDN-07, FNDN-08 | T-01-09 | Wallflower DB write returns error | unit | `cargo test -p takoyaki-app -- db` | Plan 05 creates | pending |
| 05-T2 | 05 | 2 | FNDN-04, FNDN-05, SAFE-03, SAFE-04 | T-01-08 | Atomic write with fsync; snapshot before write | integration | `cargo test -p takoyaki-app -- atomic snapshot` | Plan 05 creates | pending |
| 06-T3 | 06 | 2 | FNDN-06 | -- | N/A | smoke | manual `cargo tauri dev` | checkpoint | pending |
| 07-T1 | 07 | 3 | BROW-01 | T-01-14 | Path validated with is_ot_volume before accept | unit | `cargo test -p takoyaki-app -- device` | Plan 07 creates | pending |
| 07-T3 | 07 | 3 | BROW-01 | -- | N/A | smoke | manual end-to-end flow | checkpoint | pending |

*Status: pending / green / red / flaky*

---

## Wave 0 Requirements

All test files are created by their respective plans (TDD tasks create tests before implementation). No separate Wave 0 scaffolding is needed because each TDD plan produces its own test files.

- [x] `crates/ot-parser/tests/round_trip.rs` -- created by Plan 03 Task 1
- [x] `crates/ot-parser/tests/indexing.rs` -- created by Plan 03 Task 2
- [x] `crates/takoyaki-app/src/atomic/mod.rs` (inline tests) -- created by Plan 05 Task 2
- [x] `crates/takoyaki-app/src/atomic/snapshot.rs` (inline tests) -- created by Plan 05 Task 2
- [x] `crates/takoyaki-app/src/db/mod.rs` (inline tests) -- created by Plan 05 Task 1
- [x] `crates/takoyaki-app/src/db/wallflower.rs` (inline tests) -- created by Plan 05 Task 1
- [x] `crates/takoyaki-app/src/device/mod.rs` (inline tests) -- created by Plan 07 Task 1
- [x] `tests/fixtures/sample.ot` -- created by Plan 03 Task 1

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| App launches on macOS without crash | FNDN-06 | Requires GUI window, Tauri webview | Run `cargo tauri dev`, verify window opens and sidebar renders |
| Visual identity matches warm dark palette | D-05, D-06 | Visual / subjective | Plan 06 Task 3 checkpoint |
| Real FAT32 volume write | FNDN-04/05 | Requires physical USB FAT32 device | Insert CF card via USB reader, run targeted integration test |
| End-to-end volume detection flow | BROW-01 | Requires physical OT card or USB drive | Plan 07 Task 3 checkpoint |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 15s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** signed off (revision 1)
