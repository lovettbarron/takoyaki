---
phase: 2
slug: read-only-browser
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-29
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | vitest (frontend) / cargo test (backend) |
| **Config file** | vitest.config.ts / Cargo.toml |
| **Quick run command** | `cargo test -p ot-parser && npx vitest run --reporter=verbose` |
| **Full suite command** | `cargo test --workspace && npx vitest run` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ot-parser && npx vitest run --reporter=verbose`
- **After every plan wave:** Run `cargo test --workspace && npx vitest run`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| TBD | TBD | TBD | BROW-02 | — | N/A | integration | `cargo test --test project_list` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | BROW-03 | — | N/A | integration | `cargo test --test bank_view` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | BROW-04 | — | N/A | integration | `cargo test --test sample_slots` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | BROW-05 | — | N/A | integration | `cargo test --test metadata` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | DETC-01 | — | N/A | unit | `cargo test --test health_missing` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | DETC-02 | — | N/A | unit | `cargo test --test health_format` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | DETC-03 | — | N/A | unit | `cargo test --test health_unused` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | MGMT-04 | — | N/A | integration | `cargo test --test search_filter` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Test fixtures from real OT project files (established in Phase 1)
- [ ] Frontend test infrastructure (vitest + testing-library)
- [ ] Backend integration test harness for Tauri commands

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Visual bank grid layout matches OT 4x4 | BROW-03 | Visual layout verification | Open project, verify 4x4 grid matches OT bank numbering |
| Health check badge count updates in real-time | DETC-01 | UI state timing | Open project, observe Health tab badge populating |
| Breadcrumb navigation flow | BROW-03 | UI navigation path | Click through Projects > Project > Bank > Part, verify breadcrumb trail |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
