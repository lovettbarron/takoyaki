---
phase: 05-sample-assignment-and-wallflower
plan: 02
subsystem: wallflower-integration-backend-and-frontend-foundation
tags: [rust, tauri, sqlite, wallflower, zustand, typescript, tdd]
dependency_graph:
  requires:
    - 05-01 (compute_sample_dry_run, assign_sample, V3 migration, get_setting/set_setting)
  provides:
    - get_wallflower_status Tauri command
    - search_wallflower_samples Tauri command
    - set_wallflower_db_path Tauri command
    - WallflowerSample / WallflowerStatus TypeScript types
    - computeSampleDryRun / assignSample / getWallflowerStatus / searchWallflowerSamples / setWallflowerDbPath IPC wrappers
    - useSamplesStore zustand store (assignment flow, Wallflower state, slot picker)
  affects:
    - crates/takoyaki-app/src/commands/wallflower.rs
    - crates/takoyaki-app/src/commands/mod.rs
    - crates/takoyaki-app/src/lib.rs
    - src/lib/types.ts
    - src/lib/tauri.ts
    - src/lib/stores/samples.ts
tech_stack:
  added: []
  patterns:
    - Wallflower DB auto-discovery priority order (user-configured → data_dir → home)
    - read-only Wallflower connection via open_wallflower_db (SQLITE_OPEN_READ_ONLY)
    - rusqlite::params![] for all SQL parameters (T-05-08 SQL injection prevention)
    - DB lock release before file I/O (T-03-04 pattern)
    - Borrow checker safe pattern: collect MappedRows into Vec before returning from if/else
key_files:
  created:
    - crates/takoyaki-app/src/commands/wallflower.rs
    - src/lib/stores/samples.ts
  modified:
    - crates/takoyaki-app/src/commands/mod.rs
    - crates/takoyaki-app/src/lib.rs
    - src/lib/types.ts
    - src/lib/tauri.ts
decisions:
  - "search_samples uses shared map_row closure assigned before if/else split to avoid stmt borrow lifetime issues"
  - "discover_wallflower_db tests are defensive — they assert only that returned paths exist, not that None is returned (Wallflower may actually be installed)"
  - "Empty query returns all samples ordered by filename; non-empty query uses LIKE + exact matches on key/BPM/tag"
  - "wallflowerPanelExpanded defaults to true per D-09 / UI-SPEC"
metrics:
  duration: "~6 min"
  completed: "2026-05-02T11:08:00Z"
  tasks_completed: 2
  files_modified: 6
---

# Phase 5 Plan 02: Wallflower Commands and Frontend Foundation Summary

**One-liner:** Three Tauri commands for Wallflower DB auto-discovery/search/settings, plus complete TypeScript foundation (4 types, 5 IPC wrappers, zustand samples store with assignment flow and Wallflower state).

## What Was Built

### Task 1: commands/wallflower.rs (TDD)

**`crates/takoyaki-app/src/commands/wallflower.rs`** — new file with:

- `WallflowerStatus` struct: `connected`, `db_path`, `sample_count` (specta-typed for IPC)
- `WallflowerSample` struct: `id`, `filename`, `file_path`, `sample_rate`, `bit_depth`, `bpm`, `key_name`, `scale`, `tags: Vec<String>` (specta-typed)
- `discover_wallflower_db(user_configured_path)` — priority order per D-06:
  1. User-configured path from settings table
  2. `~/Library/Application Support/wallflower/wallflower.db` (VERIFIED: `dirs::data_dir()` in Wallflower source)
  3. `~/wallflower/wallflower.db` (Wallflower watch_folder default)
- `search_samples(conn, query, limit)` — LEFT JOIN across `jams`, `jam_tempo`, `jam_key`, `jam_tags`; empty query returns all; parameterized query on `?2` prevents SQL injection
- `get_wallflower_status` — auto-discovers DB, returns connection state + sample count
- `search_wallflower_samples` — calls discover then search; returns `Vec<WallflowerSample>`
- `set_wallflower_db_path` — validates `path.exists()` (T-05-10), writes to settings, returns updated status
- All 3 commands registered in `lib.rs collect_commands![]`
- **14 unit tests** covering: auto-discovery priority, nonexistent path fall-through, empty/filtered search, BPM/key/tag search, tag splitting, limit enforcement, struct construction

**`crates/takoyaki-app/src/commands/mod.rs`** — added `pub mod wallflower`

**`crates/takoyaki-app/src/lib.rs`** — registered 3 wallflower commands in `collect_commands![]`

### Task 2: TypeScript types, IPC wrappers, zustand store

**`src/lib/types.ts`** — appended Phase 5 types:
- `SampleDryRunResult` (manifest + hard_block + soft_warnings)
- `AssignSampleResult` (files_written, slot_type, slot_index, filename)
- `WallflowerStatus` (connected, db_path, sample_count)
- `WallflowerSample` (id, filename, file_path, audio specs, bpm, key_name, scale, tags)

**`src/lib/tauri.ts`** — added 5 IPC wrappers:
- `computeSampleDryRun(projectId, slotType, slotIndex, filePath)`
- `assignSample(projectId, slotType, slotIndex, filePath, fromWallflower)`
- `getWallflowerStatus()`
- `searchWallflowerSamples(query)`
- `setWallflowerDbPath(path)`

**`src/lib/stores/samples.ts`** — new `useSamplesStore` with:
- Assignment flow state: `assignStatus` (7 states), `dryRunManifest`, `hardBlock`, `softWarnings`, `successMessage`, pending slot/path fields
- Per-slot error state: `slotError` + `slotErrorRedirect` (D-13 inline redirect)
- Wallflower state: `wallflowerConnected`, `wallflowerDbPath`, `wallflowerPanelExpanded` (default true per D-09)
- Slot picker dialog: `slotPickerOpen`, `slotPickerSampleFilename`, `slotPickerSampleFilePath`
- 10 action methods: setAssignStatus, setDryRunResult, setPendingAssign, setSlotError, clearSlotError, setSuccessMessage, setWallflowerConnected, setWallflowerPanelExpanded, openSlotPicker, closeSlotPicker, reset

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Borrow checker rejected stmt in if/else block returning value**
- **Found during:** Task 1 first compilation attempt
- **Issue:** The `search_samples` function used an `if/else` expression returning collected rows from `stmt.query_map(...)`. Rust's borrow checker rejected this because `stmt` was declared inside the `if` arm but the temporary `MappedRows` iterator extended the borrow past `stmt`'s drop point at the closing `}`.
- **Fix:** Extracted a shared `map_row` closure defined before the `if/else`, then stored `query_map(...).collect()` result into a local `rows` variable before returning `Ok(rows)`. This drops `stmt` and the MappedRows iterator before the function returns.
- **Files modified:** `crates/takoyaki-app/src/commands/wallflower.rs`
- **Commit:** `26b89c1`

## Known Stubs

None — all data flows are wired. The Wallflower integration degrades gracefully per D-07: when no DB is found, commands return `connected: false` or `Err(Io("Wallflower database not found"))`. Frontend Plans 03/04 will conditionally render the Wallflower panel based on `wallflowerConnected` state.

## Threat Flags

No new threat surface beyond the plan's threat model. All three new Tauri commands are local IPC (no network exposure). Wallflower DB is opened read-only via `open_wallflower_db` (SQLITE_OPEN_READ_ONLY enforced at driver level — T-05-07). Path validation in `set_wallflower_db_path` rejects nonexistent paths (T-05-10). SQL parameters use `rusqlite::params![]` exclusively (T-05-08).

## Self-Check: PASSED

| Item | Status |
|------|--------|
| crates/takoyaki-app/src/commands/wallflower.rs | FOUND |
| src/lib/stores/samples.ts | FOUND |
| src/lib/types.ts contains WallflowerSample | FOUND |
| src/lib/tauri.ts contains searchWallflowerSamples | FOUND |
| Commit 26b89c1 (Task 1: wallflower.rs) | FOUND |
| Commit 2245c26 (Task 2: TS types + store) | FOUND |
| cargo test commands::wallflower | 14/14 PASSED |
| cargo check --workspace | PASSED (0 errors, pre-existing warnings only) |
| npx tsc --noEmit | PASSED (exit 0, zero errors) |
