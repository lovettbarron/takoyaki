---
phase: 8
slug: phase-5-quality-and-safety-fixes
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-06
---

# Phase 8 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` + Vitest (frontend) |
| **Config file** | `src-tauri/Cargo.toml` / `vitest.config.ts` |
| **Quick run command** | `cd src-tauri && cargo test` |
| **Full suite command** | `cd src-tauri && cargo test && cd ../src && npx vitest run` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cd src-tauri && cargo test`
- **After every plan wave:** Run full suite command
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 08-01-01 | 01 | 1 | SMPL-01 | — | N/A | unit | `cargo test slot_row` | ❌ W0 | ⬜ pending |
| 08-02-01 | 02 | 1 | SMPL-03 | T-08-01 | Reject unsupported audio formats before write | unit | `cargo test format_validation` | ❌ W0 | ⬜ pending |
| 08-03-01 | 03 | 1 | INTG-01 | T-08-02 | Atomic write prevents partial file corruption | unit | `cargo test atomic_copy` | ❌ W0 | ⬜ pending |
| 08-04-01 | 04 | 1 | INTG-02 | — | N/A | integration | `cargo test conflict_prompt` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Unit tests for format validation in `assign_sample`
- [ ] Unit tests for atomic file copy
- [ ] Integration test for conflict prompt flow

*Existing test infrastructure covers framework needs — no new dependencies required.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Dismiss button clears pending assignment in UI | SMPL-01 | Requires visual confirmation of React state | Click dismiss on SlotRow, verify assignment clears |
| Conflict prompt appears when dest exists | INTG-02 | Requires Tauri IPC + UI interaction | Assign sample to occupied slot, verify prompt appears |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
