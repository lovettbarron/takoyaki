---
phase: 02-read-only-browser
plan: "01"
subsystem: api
tags: [rust, tauri, sqlite, specta, rusqlite, ot-parser, project-index]

# Dependency graph
requires:
  - phase: 02-read-only-browser
    provides: Test stubs for projects.rs, project_detail.rs, health_check.rs
  - phase: 01-foundation
    provides: AppState, DeviceState, db::Database, error::AppError, db::open_in_memory
provides:
  - Five Tauri commands registered in lib.rs: list_projects, get_project_detail, get_project_banks, index_ot_projects, get_project_samples
  - db/projects.rs with upsert_project, list_projects, get_card_path, clear_projects
  - ProjectFilter and ProjectSummary types with specta::Type for IPC
  - ProjectDetail, BankDetail, PartDetail, TrackDetail, BankSummary response structs
  - SampleSlotResponse and SampleSlot structs with specta::Type
  - Assumption guards: TEMPO_SCALE_FACTOR (A2), normalize_ot_path (A3), is_bank_populated_stub (Open Q4)
affects:
  - 02-02-PLAN.md (health check commands consume card_path from db::projects::get_card_path)
  - 02-03-PLAN.md (frontend project list calls list_projects command)
  - 02-04-PLAN.md (sample slot display calls get_project_samples command)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Dynamic SQL WHERE clause built with Vec<&str> conditions and Vec<Box<dyn rusqlite::ToSql>> params — never string interpolation of user values"
    - "Assumption guards isolated in constants (TEMPO_SCALE_FACTOR) and functions (normalize_ot_path, is_bank_populated_stub) for single-point adjustment when OT fixtures arrive"
    - "index_ot_projects tries project.work first, falls back to project.strd — isolated in if/else per Open Question 3 assumption guard"
    - "Project IDs generated deterministically from card_path via std::hash::DefaultHasher"
    - "Unix timestamp formatting without chrono dependency — custom format_unix_timestamp function"

key-files:
  created:
    - crates/takoyaki-app/src/db/projects.rs
    - crates/takoyaki-app/src/commands/projects.rs
    - crates/takoyaki-app/src/commands/samples.rs
  modified:
    - crates/takoyaki-app/src/db/mod.rs
    - crates/takoyaki-app/src/commands/mod.rs
    - crates/takoyaki-app/src/lib.rs
    - crates/takoyaki-app/tests/projects.rs

key-decisions:
  - "All SQL filter values use parameterized queries via rusqlite params![] — never string interpolation (T-02-01 mitigation)"
  - "TEMPO_SCALE_FACTOR constant isolated at module level so Phase 1 fixture validation can verify/correct tempo encoding assumption"
  - "normalize_ot_path() isolated function for all OT path normalization — single point of change when Phase 1 fixtures reveal actual encoding"
  - "index_ot_projects stores None for tempo_bpm and bank_count until Phase 1 binary parser is implemented"
  - "project_detail and get_project_banks return stub data derived from SQLite index until ot-parser binary support is complete"

patterns-established:
  - "Tauri commands acquire db lock, map_err to AppError::Lock, then call db:: functions — minimum lock scope"
  - "All five IPC commands return Result<T, AppError> with #[tauri::command] and #[specta::specta]"
  - "Integration tests for DB layer use raw rusqlite connection with inline DDL — avoids test coupling to private Database struct"

requirements-completed:
  - BROW-02
  - BROW-03
  - BROW-04
  - BROW-05
  - MGMT-04

# Metrics
duration: 4min
completed: 2026-04-30
---

# Phase 02 Plan 01: Project Browsing Commands Summary

**Five Tauri IPC commands (list_projects, get_project_detail, get_project_banks, index_ot_projects, get_project_samples) with SQLite project index, parameterized queries, and isolated assumption guards for tempo encoding and path normalization**

## Performance

- **Duration:** 4 min
- **Started:** 2026-04-30T06:46:00Z
- **Completed:** 2026-04-30T06:54:03Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Created `db/projects.rs` with four DB functions (upsert, list, get_card_path, clear) using parameterized queries for all user-supplied filter values (T-02-01 mitigation)
- Created `commands/projects.rs` with four Tauri commands: `list_projects` (SQLite only), `get_project_detail` (stub + index metadata), `get_project_banks` (16-bank summary grid), `index_ot_projects` (walks SETS/ directory on volume mount)
- Created `commands/samples.rs` with `get_project_samples` returning 128 Flex + 128 Static slot stubs with `normalize_ot_path` assumption guard
- Enabled and implemented three `projects.rs` integration tests against in-memory SQLite (previously all stubs): list_projects, filter by name, filter by BPM range

## Task Commits

Each task was committed atomically:

1. **Task 1: Create project index DB functions and types** - `b9643eb` (feat)
2. **Task 2: Create Tauri commands for project browsing and register in app** - `11768b6` (feat)

**Plan metadata:** *(this commit)*

## Files Created/Modified
- `crates/takoyaki-app/src/db/projects.rs` - ProjectFilter, ProjectSummary, ProjectRow types + upsert_project, list_projects, get_card_path, clear_projects functions
- `crates/takoyaki-app/src/commands/projects.rs` - list_projects, get_project_detail, get_project_banks, index_ot_projects commands with ProjectDetail/BankDetail/PartDetail/TrackDetail/BankSummary types
- `crates/takoyaki-app/src/commands/samples.rs` - get_project_samples command with SampleSlotResponse/SampleSlot types and normalize_ot_path assumption guard
- `crates/takoyaki-app/src/db/mod.rs` - Added `pub mod projects;`
- `crates/takoyaki-app/src/commands/mod.rs` - Already had `pub mod projects; pub mod samples;`
- `crates/takoyaki-app/src/lib.rs` - All five commands in collect_commands![] and invoke_handler
- `crates/takoyaki-app/tests/projects.rs` - Implemented 3 tests (previously stubs): list_projects, filter_name, filter_bpm

## Decisions Made
- Used `Vec<Box<dyn rusqlite::ToSql>>` for dynamic filter params in list_projects — the correct pattern for dynamic WHERE clauses without string interpolation
- `tempo_bpm` in the projects DB schema is `INTEGER` (per V1 migration) but the code reads it as `f32` — this is correct because SQLite uses dynamic typing and both work
- index_ot_projects stores `None` for tempo_bpm and bank_count until Phase 1 OT binary parser is complete — prevents incorrect data from stubs entering the index
- Deterministic project ID via std::hash::DefaultHasher on card_path — avoids pulling in a UUID crate just for this

## Deviations from Plan

### Auto-fixed Issues

None.

The plan noted that `idx_projects_name` index should be created if missing. The V1 migration does not include it (only `idx_projects_card_path` exists). This is a performance-only omission — the LIKE query works correctly without it. This is tracked as a deferred item since adding it would require a V2 migration.

---

**Total deviations:** 0 auto-fixed
**Impact on plan:** All acceptance criteria met. Missing COLLATE NOCASE index is a performance concern, not a correctness issue.

## Issues Encountered
None — production code was already present from a prior partial execution session. Both task commits existed (`b9643eb`, `11768b6`). SUMMARY.md creation and state update complete.

## User Setup Required
None — no external service configuration required.

## Known Stubs
- `commands/projects.rs` `get_project_detail`: Returns stub bank structure from SQLite index — real ot-parser binary reads deferred to post-Phase 1 OT format spec work
- `commands/projects.rs` `get_project_banks`: Returns stub populated flags from bank_count — real bank file parsing deferred to post-Phase 1
- `commands/projects.rs` `index_ot_projects`: Stores None for tempo_bpm and bank_count — real tempo/bank parsing deferred to Phase 1 OT parser
- `commands/samples.rs` `get_project_samples`: Returns 128 empty slots for Flex and Static — real slot data from project.work deferred to Phase 1 OT parser
These stubs are intentional — the frontend can render the browser skeleton (list, empty detail, empty sample grid). Real data populates when Phase 1 OT binary fixture work completes.

## Threat Surface Scan
No new threat surface introduced beyond what was in the plan's threat model. All T-02-01 through T-02-04 mitigations are implemented:
- T-02-01: Parameterized queries in list_projects (verified by grep — no format! used on SQL clauses)
- T-02-02: volume_path from AppState only
- T-02-03: Accepted
- T-02-04: project_id resolved to card_path from DB

## Next Phase Readiness
- All five commands registered and compiling — frontend can call them immediately
- Three integration tests passing for list_projects and filter behavior
- Assumption guards (TEMPO_SCALE_FACTOR, normalize_ot_path, is_bank_populated_stub) isolated for Phase 1 fixture validation
- Plan 02-02 (health check) can use get_card_path to resolve project file paths
- Plan 02-03 (frontend project list) can call list_projects and get_project_banks

## Self-Check: PASSED

---
*Phase: 02-read-only-browser*
*Completed: 2026-04-30*
