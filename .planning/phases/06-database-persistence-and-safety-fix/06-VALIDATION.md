---
phase: 6
slug: database-persistence-and-safety-fix
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-06
---

# Phase 6 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (cargo test) |
| **Config file** | Cargo.toml — no separate test config |
| **Quick run command** | `cargo test -p takoyaki-app` |
| **Full suite command** | `cargo test -p takoyaki-app` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p takoyaki-app`
- **After every plan wave:** Run `cargo test -p takoyaki-app`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 06-01-01 | 01 | 1 | SAFE-05 | — | DB persists across restarts | integration | `cargo test -p takoyaki-app test_open_database_creates_file` | ✅ | ⬜ pending |
| 06-01-02 | 01 | 1 | SAFE-05 | — | default_path() returns absolute path | unit | `cargo test -p takoyaki-app test_default_path_is_absolute` | ❌ W0 | ⬜ pending |
| 06-01-03 | 01 | 1 | INTG-03 | — | Settings persist via file DB | unit | `cargo test -p takoyaki-app test_settings_persist_on_file_db` | ❌ W0 | ⬜ pending |
| 06-01-04 | 01 | 1 | INTG-03 | — | mark_backup_complete rejects unknown ID | unit | `cargo test -p takoyaki-app test_mark_backup_complete_unknown_id_returns_err` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `test_mark_backup_complete_unknown_id_returns_err` — add to `tests/backup_db.rs`, covers CR-02 row-count guard
- [ ] `test_default_path_is_absolute` — add to `db/mod.rs` unit tests, covers SAFE-05 persistence initialization
- [ ] `test_settings_persist_on_file_db` — add to `db/mod.rs` unit tests, covers INTG-03 settings persistence

*Existing infrastructure covers all other aspects — 90 passing tests, no framework gaps.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Backup history visible after app restart | SAFE-05 | Requires full Tauri app restart cycle | 1. Create backup 2. Quit app 3. Relaunch 4. Check backup history still shows |
| Wallflower path survives restart | INTG-03 | Requires Tauri app restart | 1. Set Wallflower path 2. Quit app 3. Relaunch 4. Verify Wallflower panel connects |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
