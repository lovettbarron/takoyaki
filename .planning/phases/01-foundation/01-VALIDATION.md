---
phase: 1
slug: foundation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-29
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness + cargo-nextest |
| **Config file** | none — `cargo test` works; `cargo nextest run` for parallel |
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

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 1 | FNDN-01 | — | N/A | unit | `cargo test -p ot-parser test_parse_ot_samplefile` | ❌ W0 | ⬜ pending |
| 01-01-02 | 01 | 1 | FNDN-02 | — | N/A | unit | `cargo test -p ot-parser test_roundtrip_all_types` | ❌ W0 | ⬜ pending |
| 01-01-03 | 01 | 1 | FNDN-03 | — | N/A | unit | `cargo test -p ot-parser test_index_newtype_bounds` | ❌ W0 | ⬜ pending |
| 01-02-01 | 02 | 1 | FNDN-04 | T-01-01 | Staging dir on same volume as target | integration | `cargo test -p takoyaki-app test_staging_same_volume` | ❌ W0 | ⬜ pending |
| 01-02-02 | 02 | 1 | FNDN-05 | T-01-02 | sync_all (F_FULLFSYNC) called before rename | integration | `cargo test -p takoyaki-app test_atomic_write_fsync` | ❌ W0 | ⬜ pending |
| 01-02-03 | 02 | 1 | SAFE-03 | — | Snapshot exists before write commits | integration | `cargo test -p takoyaki-app test_snapshot_before_write` | ❌ W0 | ⬜ pending |
| 01-02-04 | 02 | 1 | SAFE-04 | T-01-03 | Write failure leaves original untouched | integration | `cargo test -p takoyaki-app test_atomic_write_failure_rollback` | ❌ W0 | ⬜ pending |
| 01-03-01 | 03 | 1 | FNDN-06 | — | N/A | smoke | manual `cargo tauri dev` | ❌ W0 | ⬜ pending |
| 01-03-02 | 03 | 1 | FNDN-07 | — | N/A | unit | `cargo test -p takoyaki-app test_db_init` | ❌ W0 | ⬜ pending |
| 01-03-03 | 03 | 1 | FNDN-08 | — | Wallflower DB opened read-only; write attempt fails | unit | `cargo test -p takoyaki-app test_wallflower_db_readonly` | ❌ W0 | ⬜ pending |
| 01-04-01 | 04 | 2 | BROW-01 | T-01-04 | Path validated under OT mount point before fs ops | unit | `cargo test -p takoyaki-app test_detect_ot_volume` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/ot-parser/tests/round_trip.rs` — stubs for FNDN-01, FNDN-02
- [ ] `crates/ot-parser/tests/indexing.rs` — stubs for FNDN-03
- [ ] `crates/takoyaki-app/tests/atomic_write.rs` — stubs for FNDN-04, FNDN-05, SAFE-03, SAFE-04
- [ ] `crates/takoyaki-app/tests/db_init.rs` — stubs for FNDN-07, FNDN-08
- [ ] `crates/takoyaki-app/tests/volume_detection.rs` — stubs for BROW-01
- [ ] `tests/fixtures/` directory with at least one synthetic .ot fixture

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| App launches on macOS without crash | FNDN-06 | Requires GUI window, Tauri webview | Run `cargo tauri dev`, verify window opens and sidebar renders |
| Real FAT32 volume write | FNDN-04/05 | Requires physical USB FAT32 device | Insert CF card via USB reader, run targeted integration test |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
