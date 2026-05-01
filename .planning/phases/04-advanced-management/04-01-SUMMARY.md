---
phase: 04-advanced-management
plan: "01"
subsystem: rust-backend
tags: [management, project-work, rename, duplicate, ot-parser]
dependency_graph:
  requires:
    - "03-write-path-and-backup (atomic writes, SnapshotEngine)"
    - "01-foundation (AppError, health::resolve_ot_path)"
  provides:
    - "management::project_work: extract_slot_paths, rewrite_slot_path, validate_ot_name"
    - "management::rename: rename_project"
    - "management::duplicate: duplicate_project, compute_default_name"
  affects:
    - "04-02 (management Tauri commands will call rename_project + duplicate_project)"
    - "04-03 (export.rs will use extract_slot_paths for audio path discovery)"
    - "04-04 (bank_copy.rs will use extract_slot_paths + rewrite_slot_path)"
tech_stack:
  added:
    - "zip = 2 (Cargo.toml — required for Plan 04-03 export)"
  patterns:
    - "project.work TYPE=FLEX/TYPE=STATIC inline discriminator parsing (not bracketed headers)"
    - "WalkDir follow_links(false) copy-tree pattern (from backup.rs)"
    - "OT name validation: A-Z a-z 0-9 underscore max 16 chars"
key_files:
  created:
    - "crates/takoyaki-app/src/management/mod.rs"
    - "crates/takoyaki-app/src/management/project_work.rs"
    - "crates/takoyaki-app/src/management/rename.rs"
    - "crates/takoyaki-app/src/management/duplicate.rs"
    - "crates/takoyaki-app/src/management/export.rs (stub)"
    - "crates/takoyaki-app/src/management/bank_copy.rs (stub)"
  modified:
    - "crates/takoyaki-app/Cargo.toml (zip = 2 added)"
    - "crates/takoyaki-app/src/lib.rs (pub mod management added)"
decisions:
  - "project.work slot section uses TYPE=FLEX/TYPE=STATIC inline discriminators (Assumption A1 — needs real OT file verification)"
  - "Audio files in /AUDIO/ are shared across projects — duplicate preserves original PATH= entries (../AUDIO/... remains valid from any /SETS/PROJECT/ dir)"
  - "rename_project does not modify project.work/.strd — directory name IS the authoritative project name (A3, verified against ot-tools-io OsMetadata)"
  - "export.rs and bank_copy.rs are empty stubs to satisfy mod.rs declarations; implementation in Plans 04-03 and 04-04"
metrics:
  duration: "5 min"
  completed_date: "2026-05-01"
  tasks_completed: 2
  files_changed: 8
---

# Phase 04 Plan 01: Management Module Foundation Summary

## One-liner

project.work text parser with TYPE=FLEX/STATIC slot extraction, OT name validator, and atomic rename/duplicate operations backed by 17 unit tests.

## What Was Built

**Task 1: Management module scaffold + project_work parser**

- `management/mod.rs`: declares 5 submodules (project_work, rename, duplicate, export, bank_copy)
- `management/project_work.rs`: three public functions:
  - `extract_slot_paths(bytes) -> Vec<SlotPath>`: parses TYPE=FLEX/STATIC + SLOT= + PATH= lines
  - `rewrite_slot_path(raw, slot_type, slot_number, new_path) -> Vec<u8>`: targeted PATH= line replacement
  - `validate_ot_name(name) -> Result<(), AppError>`: enforces A-Z/a-z/0-9/underscore, max 16 chars
- `SlotType` enum and `SlotPath` struct with Serialize/Type derives for IPC
- 7 unit tests: extract, empty input, rewrite (correct slot only), 4 validate tests
- `zip = "2"` added to Cargo.toml (prerequisite for Plan 04-03 export)
- `pub mod management` added to lib.rs

**Task 2: Rename and duplicate business logic**

- `management/rename.rs`:
  - `rename_project(project_dir, new_name) -> Result<PathBuf, AppError>`
  - Validates name, checks collision, calls `std::fs::rename` (same-volume FAT32 safe)
  - No project.work modification needed — directory name IS the project name (A3)
  - 4 unit tests: success, invalid name, name too long, collision guard
- `management/duplicate.rs`:
  - `duplicate_project(project_dir, new_name, card_volume_path) -> Result<DuplicateResult, AppError>`
  - `compute_default_name(original) -> String`: returns `{name}_copy` (D-02)
  - `DuplicateResult { new_project_dir, files_copied }` struct with Serialize/Type
  - WalkDir copy-tree with follow_links(false) — preserves original PATH= entries
  - 5 unit tests: default name, D-03 overflow detection, copy files, content preservation, collision

## Test Results

```
cargo test -p takoyaki-app management
running 17 tests ... test result: ok. 17 passed; 0 failed
```

## Deviations from Plan

None — plan executed exactly as written.

The RESEARCH.md Assumption A1 (TYPE= inline discriminators vs [FLEX]/[STATIC] section headers) was correctly followed as specified in the task action ("NOT bracketed section headers — see RESEARCH.md A1").

## Known Stubs

| File | Reason |
|------|--------|
| `crates/takoyaki-app/src/management/export.rs` | Empty stub — required by mod.rs declaration; implementation in Plan 04-03 |
| `crates/takoyaki-app/src/management/bank_copy.rs` | Empty stub — required by mod.rs declaration; implementation in Plan 04-04 |

These stubs do not affect this plan's goals (they are not called by any code in this plan).

## Threat Surface Scan

No new network endpoints, auth paths, or trust boundary crossings introduced. All path computation stays in Rust. T-04-01 mitigated: `validate_ot_name` called as first step in both `rename_project` and `duplicate_project`, preventing any path-separator characters from reaching filesystem operations.

## Self-Check: PASSED

All 6 management module files found on disk. Both task commits (1f94d1d, f3fedea) confirmed in git log.
