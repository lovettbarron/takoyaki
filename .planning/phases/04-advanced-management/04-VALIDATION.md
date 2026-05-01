---
phase: 04
slug: advanced-management
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-01
---

# Phase 04 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust integration tests) |
| **Config file** | none — standard Cargo test runner |
| **Quick run command** | `cargo test -p takoyaki-app management` |
| **Full suite command** | `cargo test -p takoyaki-app` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p takoyaki-app management`
- **After every plan wave:** Run `cargo test -p takoyaki-app`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 04-01-01 | 01 | 1 | MGMT-01 | — | Duplicate copies all files + remaps paths | integration | `cargo test -p takoyaki-app management::tests::test_duplicate_project` | ❌ W0 | ⬜ pending |
| 04-01-02 | 01 | 1 | MGMT-01 | — | Name collision detected + incremented | unit | `cargo test -p takoyaki-app management::tests::test_duplicate_name_collision` | ❌ W0 | ⬜ pending |
| 04-02-01 | 02 | 1 | MGMT-02 | — | Rename updates directory name on disk | integration | `cargo test -p takoyaki-app management::tests::test_rename_project` | ❌ W0 | ⬜ pending |
| 04-02-02 | 02 | 1 | MGMT-02 | — | OT-legal character validation | unit | `cargo test -p takoyaki-app management::tests::test_rename_validation` | ❌ W0 | ⬜ pending |
| 04-03-01 | 03 | 1 | MGMT-03 | — | Export zip contains SETS/ + AUDIO/ structure | integration | `cargo test -p takoyaki-app management::tests::test_export_zip_structure` | ❌ W0 | ⬜ pending |
| 04-03-02 | 03 | 1 | MGMT-03 | — | Export includes .ot sidecar files | integration | `cargo test -p takoyaki-app management::tests::test_export_includes_sidecars` | ❌ W0 | ⬜ pending |
| 04-04-01 | 04 | 2 | SMPL-02 | — | Bank copy merges slots, hash-match = skip | integration | `cargo test -p takoyaki-app management::tests::test_bank_copy_no_conflict` | ❌ W0 | ⬜ pending |
| 04-04-02 | 04 | 2 | SMPL-02 | — | Bank copy surfaces hash-mismatch conflicts | integration | `cargo test -p takoyaki-app management::tests::test_bank_copy_conflict_detection` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/takoyaki-app/src/management/mod.rs` — management module skeleton
- [ ] `crates/takoyaki-app/tests/management.rs` — integration tests for all 4 operations
- [ ] Add `zip = "2"` to `crates/takoyaki-app/Cargo.toml`
- [ ] `npx shadcn@latest add context-menu` — frontend prerequisite
- [ ] `src/lib/stores/management.ts` — zustand store for management operations

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Context menu appears on right-click of populated bank cell | SMPL-02 | Browser interaction | Right-click a populated bank in the bank grid; verify "Copy to project..." appears |
| Two-step bank copy picker dialog | SMPL-02 | Multi-step UI flow | Trigger bank copy; verify project list step then bank slot grid step |
| Inline rename in header | MGMT-02 | Browser interaction | Click Rename button; verify name becomes editable inline |
| Export progress indicator | MGMT-03 | Visual feedback | Trigger export; verify progress bar appears and updates |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
