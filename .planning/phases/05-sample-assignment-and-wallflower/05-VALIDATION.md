---
phase: 5
slug: sample-assignment-and-wallflower
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-02
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | vitest (frontend) / cargo test (Rust backend) |
| **Config file** | `vitest.config.ts` / `Cargo.toml` |
| **Quick run command** | `cargo test -p takoyaki-app && npx vitest run --reporter=verbose` |
| **Full suite command** | `cargo test --workspace && npx vitest run` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p takoyaki-app && npx vitest run --reporter=verbose`
- **After every plan wave:** Run `cargo test --workspace && npx vitest run`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| TBD | TBD | TBD | SMPL-01 | — | Atomic write with snapshot | integration | `cargo test -p takoyaki-app` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | SMPL-03 | — | Flex/Static validation blocks mismatch | unit | `cargo test -p takoyaki-app` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | INTG-01 | — | Read-only Wallflower DB access | integration | `cargo test -p takoyaki-app` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | INTG-02 | — | Search by key/BPM/tags returns results | integration | `cargo test -p takoyaki-app` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | INTG-03 | — | Graceful degradation without Wallflower | unit | `cargo test -p takoyaki-app` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Test fixtures for OT sample assignment (real .work file excerpts)
- [ ] Wallflower test DB with sample data for search tests
- [ ] Shared test helpers for atomic write verification

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Native file picker opens on assign button click | SMPL-01 | Tauri dialog requires native macOS interaction | Click assign button on slot row, verify file picker opens |
| Wallflower panel hides when DB unavailable | INTG-03 | UI visibility requires browser inspection | Remove Wallflower DB path, reload, verify panel absent |
| Dry-run preview shows correct file count | SMPL-01 | Visual verification of modal content | Assign sample, verify preview shows affected files |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
