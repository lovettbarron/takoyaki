---
phase: 7
slug: parser-integration-replace-stub-data
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-06
---

# Phase 7 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| TBD | 01 | 1 | BROW-03 | — | N/A | unit | `cargo test parse_project_work` | ❌ W0 | ⬜ pending |
| TBD | 01 | 1 | BROW-04 | — | N/A | unit | `cargo test get_project_detail` | ❌ W0 | ⬜ pending |
| TBD | 01 | 1 | BROW-05 | — | N/A | unit | `cargo test get_project_samples` | ❌ W0 | ⬜ pending |
| TBD | 01 | 1 | DETC-01 | — | N/A | unit | `cargo test run_health_check` | ❌ W0 | ⬜ pending |
| TBD | 01 | 1 | DETC-02 | — | N/A | unit | `cargo test is_bank_populated` | ❌ W0 | ⬜ pending |
| TBD | 01 | 1 | DETC-03 | — | N/A | N/A | `cargo test health_check` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Test stubs for `parse_project_work()` real format parsing
- [ ] Test stubs for `get_project_detail` with real binary data
- [ ] Test stubs for `run_health_check` with real slot data
- [ ] Fixture files verified against real OT card format

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| SlotPickerDialog shows real slot state | BROW-05 | Frontend visual verification | Open SlotPickerDialog with a real OT project and confirm occupied/empty slots match binary data |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
