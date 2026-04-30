---
phase: 01-foundation
verified: 2026-04-30T08:00:00Z
status: human_needed
score: 8/10 must-haves verified
overrides_applied: 0
gaps: []
human_verification:
  - test: "App launches on macOS and shows Octatrack connection status"
    expected: "cargo tauri dev opens a 1200x800 window with warm dark theme, sidebar with Projects active, 'No Device Connected' state displayed, and 'No Device' badge in sidebar header"
    why_human: "Requires the Tauri desktop app to launch in a macOS GUI environment. All code paths are wired and the build compiles, but the SC-1 runtime guarantee can only be confirmed visually."
  - test: "Volume detection flow end-to-end"
    expected: "Connecting an OT card (or directory with /AUDIO and /SETS) triggers a toast 'Octatrack connected at {name}.', then shows 'Octatrack Found' dialog after 500ms. Clicking 'Use this device' switches content to Projects view and updates the device status badge to a green dot. Disconnecting triggers 'Octatrack disconnected.' toast and resets to disconnected state."
    why_human: "Requires a real OT card or simulated volume (mkdir /tmp/test-ot/AUDIO /tmp/test-ot/SETS) and running cargo tauri dev. The Plan 07 visual checkpoint was auto-approved via checkpoint_override directive — not verified by the user."
  - test: "Round-trip parser fidelity against real OT project files"
    expected: "parse(serialize(parse(bytes))) == parse(bytes) passes for every file in a real OT project directory: project.work, project.strd, bank01-16.work, bank01-16.strd, markers.work, markers.strd, arr01-08.work, arr01-08.strd, and .ot sidecar files"
    why_human: "SC-2 requires verification against a 'corpus of real OT project files'. All 19 round-trip tests pass against synthetic fixtures. Real OT files may reveal edge cases in the opaque-blob pattern (e.g. variable body sizes in bank/markers/arr files, or unknown field values in .ot files). User must run cargo test -p ot-parser with real files added to tests/fixtures/."
  - test: "Atomic write staging on same filesystem as target (FAT32 volume)"
    expected: "Writing to a file on the OT CF card: temp file is created in the same directory on the card (same FAT32 volume), not in /tmp or Mac-side tmpfs. Atomic rename succeeds. If the card is ejected mid-write, the original file is intact."
    why_human: "SC-3 requires verification 'by integration test on a real FAT32 volume'. The unit tests use a macOS temp directory (same volume by default, but not FAT32). AtomicWriteFile is documented to create temp in the same directory, which ensures same-volume behavior — but this must be verified on an actual OT card."
---

# Phase 1: Foundation Verification Report

**Phase Goal:** Scaffold the complete development environment — Cargo workspace, Tauri v2 app shell, Next.js frontend, OT binary parser crate, SQLite database, and atomic write primitives. Everything compiles, the app window opens, and the first OT file type (.ot sample settings) parses with a round-trip test.
**Verified:** 2026-04-30T08:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| #  | Truth                                                                        | Status           | Evidence                                                                                                                                                             |
|----|------------------------------------------------------------------------------|------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 1  | Cargo workspace compiles with both crates as members                         | VERIFIED         | `cargo build --workspace` exits 0. Cargo.toml has `[workspace]` with `crates/ot-parser` and `crates/takoyaki-app`.                                                  |
| 2  | ot-parser crate has zero Tauri dependencies                                  | VERIFIED         | `crates/ot-parser/Cargo.toml` contains only `binrw`, `serde`, `thiserror`. No `tauri` string present. Confirmed by grep.                                            |
| 3  | takoyaki-app crate depends on ot-parser and Tauri                            | VERIFIED         | `crates/takoyaki-app/Cargo.toml` contains `ot-parser = { path = "../ot-parser" }` and `tauri = { version = "2"...}`. `tauri_specta::Builder` used in `lib.rs::run()`. |
| 4  | cargo tauri dev launches a window (app window opens)                         | HUMAN NEEDED     | Build succeeds. Tauri config declares 1200x800 window. Runtime launch requires macOS GUI environment — not verifiable programmatically.                              |
| 5  | npm run build produces a static export in out/                               | VERIFIED         | `npm run build` exits 0. `out/` directory exists with static export. Tauri `frontendDist: "../../out"` configured.                                                  |
| 6  | SampleSettingsFile parses a .ot file with byte-exact round-trip              | VERIFIED         | `cargo test -p ot-parser` — `test_sample_round_trip`, `test_sample_round_trip_parse_equality` pass. `tests/fixtures/sample.ot` is exactly 832 bytes.                |
| 7  | Round-trip works for all five OT file types (test_all_types_round_trip)      | VERIFIED         | 19 tests pass including `test_all_types_round_trip`. Synthetic fixtures for project.work, bank01.work, markers.work, arr01.work.                                     |
| 8  | Round-trip verified against real OT project files                            | HUMAN NEEDED     | SC-2 explicitly says "corpus of real OT project files". Tests pass on synthetic fixtures only. Real files must be added to `tests/fixtures/` to satisfy this.        |
| 9  | SQLite database initializes with snapshots, snapshot_files, and projects     | VERIFIED         | 13 DB tests pass. Tables confirmed by `test_all_three_tables`. PRAGMA user_version=1, WAL mode, foreign_keys=ON.                                                     |
| 10 | Atomic write stages temp file on same volume, fsyncs, then renames           | VERIFIED (code)  | `atomic_write()` uses `AtomicWriteFile::options().open(target_path)` (same-dir temp), then `sync_all()`, then `commit()`, then parent `dir.sync_all()`. 5 unit tests pass. FAT32 verification needs human. |

**Score:** 8/10 truths verified (2 require human verification)

### Deferred Items

No items were deferred to later phases. SC-2 (real OT files) and SC-3 (FAT32 integration) are roadmap Phase 1 success criteria — they are not scheduled in later phases and are genuine gaps pending human action.

### Required Artifacts

| Artifact                                              | Expected                                        | Status     | Details                                                                    |
|-------------------------------------------------------|-------------------------------------------------|------------|----------------------------------------------------------------------------|
| `Cargo.toml`                                          | Workspace root with two members                 | VERIFIED   | Contains `[workspace]` and both crate members                              |
| `crates/ot-parser/Cargo.toml`                         | Pure parser library crate                       | VERIFIED   | `binrw = "0.15"`, zero Tauri deps                                          |
| `crates/ot-parser/src/lib.rs`                         | Crate root with all module declarations         | VERIFIED   | Exports all 5 OT file types + ParseError + 3 index newtypes                |
| `crates/ot-parser/src/error.rs`                       | ParseError enum                                 | VERIFIED   | `pub enum ParseError` with ChecksumMismatch, InvalidMagic, UnexpectedSize  |
| `crates/ot-parser/src/sample.rs`                      | SampleSettingsFile parser                       | VERIFIED   | Full 832-byte .ot parser with `from_bytes`/`to_bytes`, `unknown_0x10`      |
| `crates/ot-parser/src/types.rs`                       | Newtype wrappers                                | VERIFIED   | ProjectSlotId(1..=256), BankSlotId(0..=255), BankNumber(0..=15)            |
| `crates/ot-parser/src/project.rs`                     | ProjectFile parser                              | VERIFIED   | Text-as-opaque-bytes verbatim round-trip                                   |
| `crates/ot-parser/src/bank.rs`                        | BankFile parser                                 | VERIFIED   | FORM+DPS1BANK header, opaque body, u16 checksum                            |
| `crates/ot-parser/src/markers.rs`                     | MarkersFile parser                              | VERIFIED   | FORM+DPS1SAMP header, opaque body                                          |
| `crates/ot-parser/src/arrangement.rs`                 | ArrangementFile parser                          | VERIFIED   | FORM+DPS1ARRA header, opaque body                                          |
| `crates/ot-parser/format-spec.md`                     | Clean-room format spec                          | VERIFIED   | Exists, contains 4+ "Offset" tables for all file types                     |
| `tests/fixtures/sample.ot`                            | 832-byte .ot test fixture                       | VERIFIED   | `wc -c` = 832 bytes                                                        |
| `tests/fixtures/project.work`                         | Synthetic project fixture                       | VERIFIED   | Exists, round-trip tests pass                                              |
| `tests/fixtures/bank01.work`                          | Synthetic bank fixture                          | VERIFIED   | Exists, round-trip tests pass                                              |
| `tests/fixtures/markers.work`                         | Synthetic markers fixture                       | VERIFIED   | Exists, round-trip tests pass                                              |
| `tests/fixtures/arr01.work`                           | Synthetic arrangement fixture                   | VERIFIED   | Exists, round-trip tests pass                                              |
| `crates/ot-parser/tests/round_trip.rs`                | Round-trip tests                                | VERIFIED   | 19 tests including `test_all_types_round_trip`, all pass                   |
| `crates/ot-parser/tests/indexing.rs`                  | Indexing boundary tests                         | VERIFIED   | 10 tests including `test_project_slot_id_rejects_zero`, all pass           |
| `crates/takoyaki-app/Cargo.toml`                      | Tauri app with all deps                         | VERIFIED   | `tauri-specta = "=2.0.0-rc.24"`, `ot-parser = { path = "../ot-parser" }`  |
| `crates/takoyaki-app/src/lib.rs`                      | Tauri app entrypoint                            | VERIFIED   | `pub fn run()`, `collect_commands!`, `device::start_polling`, AppState     |
| `crates/takoyaki-app/src/error.rs`                    | AppError enum with Serialize                    | VERIFIED   | `#[derive(Debug, Error, Serialize)]`, all From impls                       |
| `migrations/V1__initial_schema.sql`                   | Initial schema migration                        | VERIFIED   | snapshots, snapshot_files, projects tables + 3 indexes                     |
| `crates/takoyaki-app/src/db/mod.rs`                   | DB open, init, migration functions              | VERIFIED   | `open_database`, `open_in_memory`, include_str! V1 migration               |
| `crates/takoyaki-app/src/db/wallflower.rs`            | Read-only Wallflower DB connection              | VERIFIED   | `SQLITE_OPEN_READ_ONLY`, write-fail test passes                            |
| `crates/takoyaki-app/src/atomic/mod.rs`               | Atomic write engine                             | VERIFIED   | `atomic_write`, `atomic_write_batch`, AtomicWriteFile, sync_all            |
| `crates/takoyaki-app/src/atomic/snapshot.rs`          | Snapshot engine                                 | VERIFIED   | `SnapshotEngine`, `snapshot_files`, SHA-256 hash, timestamped dirs         |
| `crates/takoyaki-app/src/device/mod.rs`               | OT volume detection + polling                   | VERIFIED   | `detect_ot_volume`, `is_ot_volume`, `poll_loop`, `start_polling`           |
| `crates/takoyaki-app/src/commands/device.rs`          | Device Tauri commands                           | VERIFIED   | `get_device_status`, `confirm_device`, `dismiss_device` with specta        |
| `src/app/globals.css`                                 | Tailwind v4 warm dark palette                   | VERIFIED   | `--background: hsl(30 8% 10%)`, `--accent: hsl(38 85% 55%)`               |
| `src/app/layout.tsx`                                  | Root layout with Iosevka and Providers          | VERIFIED   | `@fontsource/iosevka` 400/500/600, `<Providers>`, `<TauriEventListener>`   |
| `src/components/providers.tsx`                        | React Query provider                            | VERIFIED   | `QueryClientProvider` with retry:1, no windowFocus refetch                 |
| `src/components/sidebar-nav.tsx`                      | Sidebar navigation                              | VERIFIED   | Projects/Samples/Backups/Settings, disabled states, 44px touch targets     |
| `src/components/device-status-badge.tsx`              | Device status badge                             | VERIFIED   | Green `hsl(140_60%_42%)` / gray `hsl(30_8%_38%)` dot indicators           |
| `src/lib/stores/device.ts`                            | Zustand device store                            | VERIFIED   | `useDeviceStore`, connected/mountPoint/confirmed state                     |
| `src/components/tauri-event-listener.tsx`             | Tauri event bridge                              | VERIFIED   | `ot-device-changed` listener, dynamic import, setConnected/reset wired     |
| `src/components/volume-confirm-dialog.tsx`            | Volume confirmation dialog                      | VERIFIED   | "Octatrack Found", "Use this device", "Not Now" buttons                    |
| `src/lib/tauri.ts`                                    | TypeScript invoke wrappers                      | VERIFIED   | `getDeviceStatus`, `confirmDevice`, `dismissDevice`                        |
| `src/app/page.tsx`                                    | App shell with sidebar + content                | VERIFIED   | SidebarNav, DeviceStatusBadge, "No Device Connected", VolumeConfirmDialog  |

### Key Link Verification

| From                                           | To                                      | Via                               | Status     | Details                                                                       |
|------------------------------------------------|-----------------------------------------|-----------------------------------|------------|-------------------------------------------------------------------------------|
| `crates/takoyaki-app/Cargo.toml`               | `crates/ot-parser`                      | path dependency                   | VERIFIED   | `ot-parser = { path = "../ot-parser" }`                                       |
| `crates/takoyaki-app/src/main.rs`              | `crates/takoyaki-app/src/lib.rs`        | `takoyaki_app::run()`             | VERIFIED   | `fn main() { takoyaki_app::run() }`                                           |
| `crates/takoyaki-app/src/db/mod.rs`            | `migrations/V1__initial_schema.sql`     | `include_str!`                    | VERIFIED   | `include_str!("../../../../migrations/V1__initial_schema.sql")`               |
| `crates/takoyaki-app/src/atomic/mod.rs`        | `atomic_write_file`                     | AtomicWriteFile API               | VERIFIED   | `AtomicWriteFile::options().open()` + `sync_all()` + `commit()`               |
| `crates/takoyaki-app/src/lib.rs`               | `db::open_in_memory` (not open_database)| AppState initialization           | WARNING    | `run()` uses `open_in_memory()` not `default_path()`; no data persistence     |
| `crates/takoyaki-app/src/lib.rs`               | `device::start_polling`                 | `.setup()` closure                | VERIFIED   | `device::start_polling(app.handle().clone())` called in setup                 |
| `crates/takoyaki-app/src/device/mod.rs`        | `sysinfo`                               | `Disks::new_with_refreshed_list()`| VERIFIED   | `Disks::new_with_refreshed_list()` with `.list().iter()`                      |
| `src/app/page.tsx`                             | `src/lib/stores/device.ts`              | `useDeviceStore`                  | VERIFIED   | `const { connected, mountPoint, confirmed, setConfirmed } = useDeviceStore()` |
| `src/components/tauri-event-listener.tsx`      | `src/lib/stores/device.ts`              | `setConnected` on event           | VERIFIED   | `setConnected(true, event.payload)` and `reset()` called on event             |
| `src/app/page.tsx`                             | `src/components/sidebar-nav.tsx`        | `<SidebarNav>` component          | VERIFIED   | `<SidebarNav activeSection={...} onSectionChange={...} />`                    |
| `src/components/volume-confirm-dialog.tsx`     | `src/lib/tauri.ts`                      | `confirmDevice` invoke            | VERIFIED   | `confirmDevice(mountPoint)` called in `handleConfirm`                         |
| `crates/ot-parser/tests/round_trip.rs`         | `tests/fixtures/sample.ot`              | `include_bytes!` macro            | VERIFIED   | `include_bytes!("../../../tests/fixtures/sample.ot")`                         |
| `crates/ot-parser/tests/round_trip.rs`         | `tests/fixtures/project.work`           | `include_bytes!` macro            | VERIFIED   | `include_bytes!("../../../tests/fixtures/project.work")`                      |
| `crates/ot-parser/src/sample.rs`               | `binrw`                                 | `#[binrw]` derive macro           | VERIFIED   | `#[binrw] #[brw(big)] pub struct SampleSettingsFile`                          |

### Data-Flow Trace (Level 4)

| Artifact                                      | Data Variable         | Source                                 | Produces Real Data | Status     |
|-----------------------------------------------|-----------------------|----------------------------------------|--------------------|------------|
| `src/app/page.tsx`                            | `connected`           | `useDeviceStore` + `ot-device-changed` | Runtime — event-driven | VERIFIED  |
| `src/components/device-status-badge.tsx`      | `connected`           | `useDeviceStore`                       | Same as above      | VERIFIED   |
| DB AppState in `lib.rs`                       | project/snapshot data | `db::Database::open_in_memory()`       | In-memory only, no persistence | WARNING |

Note on DB persistence: `lib.rs::run()` initializes `AppState` with `db::Database::open_in_memory()`. This satisfies SC-5 ("SQLite database initializes with schema") but data is lost on app restart. FNDN-07 says "SQLite database for Takoyaki's own metadata (backup history, project index, snapshot records)" — the persistence aspect will become a practical issue in Phase 2 when indexing is required. However, no Phase 1 write commands yet store data, so this is not blocking the Phase 1 goal.

### Behavioral Spot-Checks

| Behavior                                          | Command                                                                       | Result               | Status  |
|---------------------------------------------------|-------------------------------------------------------------------------------|----------------------|---------|
| ot-parser tests pass (19 tests)                   | `cargo test -p ot-parser`                                                     | 19 passed, 0 failed  | PASS    |
| takoyaki-app unit tests pass (33 tests)           | `cargo test -p takoyaki-app -- --test-threads=1` (lib unit tests)             | 33 passed, 0 failed  | PASS    |
| sample.ot fixture is exactly 832 bytes            | `wc -c tests/fixtures/sample.ot`                                              | 832 bytes            | PASS    |
| Next.js static export builds                      | `npm run build`                                                               | out/ directory created | PASS  |
| Workspace compiles                                | `cargo build --workspace`                                                     | Finished dev profile | PASS    |
| App window launch (runtime)                       | `cargo tauri dev`                                                             | Requires macOS GUI   | SKIP    |
| Real OT files round-trip                          | Add real files to `tests/fixtures/`, run `cargo test -p ot-parser`           | Not run              | SKIP    |
| AtomicWriteFile on FAT32 volume                   | Write to OT card path via `atomic_write`, verify temp stays on card           | Requires real card   | SKIP    |

### Requirements Coverage

| Requirement | Source Plan | Description                                                          | Status         | Evidence                                                                          |
|-------------|-------------|----------------------------------------------------------------------|----------------|-----------------------------------------------------------------------------------|
| FNDN-01     | 01-03, 01-04| Clean-room Rust OT binary parser, no GPL                            | SATISFIED      | binrw-based parsers for all 5 file types; format-spec.md documents clean-room methodology |
| FNDN-02     | 01-03, 01-04| Parser preserves unknown bytes verbatim during round-trip            | SATISFIED      | `unknown_0x10: [u8; 7]` in sample.rs; opaque blob pattern for binary types; 19 round-trip tests pass |
| FNDN-03     | 01-03       | 1-indexed vs 0-indexed newtypes with Rust type safety               | SATISFIED      | ProjectSlotId(1..=256), BankSlotId(0..=255), BankNumber(0..=15); 10 boundary tests pass |
| FNDN-04     | 01-05       | Staging dir on same filesystem as CF card volume                     | SATISFIED (code)| `AtomicWriteFile::options().open(target_path)` stages in same dir as target; same-filesystem by design. FAT32 runtime verification needed. |
| FNDN-05     | 01-05       | fsync + directory sync before completion                             | SATISFIED      | `staging.sync_all()` (F_FULLFSYNC on macOS) + `dir.sync_all()` in `atomic_write()` |
| FNDN-06     | 01-01, 01-02| Tauri v2 desktop app with Rust backend + React/Next.js frontend      | SATISFIED      | Tauri 2.x + `tauri-specta` builder + Next.js 16 static export; `cargo build --workspace` and `npm run build` succeed |
| FNDN-07     | 01-05       | SQLite database for metadata (backup history, project index, snapshots) | PARTIAL       | Schema with all 3 tables + 3 indexes created and verified by tests. Production `run()` uses in-memory DB — data is not persisted across restarts. No Phase 1 commands write data, so this does not block Phase 1 goals. |
| FNDN-08     | 01-05       | Read-only Wallflower DB connection at driver level                   | SATISFIED      | `SQLITE_OPEN_READ_ONLY` flag + test that confirms write attempt fails        |
| SAFE-03     | 01-05       | Snapshot before any write operation                                  | SATISFIED (unit)| SnapshotEngine exists with `snapshot_files()`, SHA-256 hash, 6 tests pass. No production write command yet uses it — no write commands exist in Phase 1. |
| SAFE-04     | 01-05       | All writes use atomic staged writes (stage, sync, rename)            | SATISFIED (unit)| `atomic_write()` + `atomic_write_batch()` with AtomicWriteFile, sync_all, commit. 5 unit tests pass. Production wire-up deferred to Phase 3 write commands. |
| BROW-01     | 01-06, 01-07| User can see OT connected status via automatic volume detection      | SATISFIED (code)| Full stack wired: `detect_ot_volume()` → `ot-device-changed` event → `useDeviceStore` → `DeviceStatusBadge`. Volume detection flow human verification needed (auto-approved, not user-approved). |

### Anti-Patterns Found

| File                                                    | Line | Pattern                          | Severity | Impact                                                                                       |
|---------------------------------------------------------|------|----------------------------------|----------|----------------------------------------------------------------------------------------------|
| `crates/takoyaki-app/src/lib.rs`                        | 46   | `Database::open_in_memory()` in production `run()` | WARNING | DB data lost on restart. Acceptable for Phase 1 (no write commands exist), but must be changed before Phase 2 indexing commands store data. |
| `crates/takoyaki-app/src/atomic/mod.rs` (warning)       | —    | `atomic_write`, `atomic_write_batch` never used in production code | INFO | Functions exist but are not called by any command. Correct for Phase 1 (no write commands). Will be used in Phase 3. |
| `crates/takoyaki-app/src/atomic/snapshot.rs` (warning)  | —    | `SnapshotEngine` never constructed in production code | INFO | Same as above — correct for Phase 1. |

### Human Verification Required

#### 1. App Launch and Connection Status Display

**Test:** Run `cargo tauri dev` from the project root. Wait for the app window to open.
**Expected:** A 1200x800 window opens with warm dark theme. Sidebar shows "Takoyaki" title, "No Device" gray dot badge, four nav items (Projects active with amber left-border, Samples/Backups/Settings dimmed). Content area shows "No Device Connected" heading and descriptive body text. Footer shows "v0.1.0".
**Why human:** Requires macOS GUI runtime. Tauri desktop app launch cannot be verified programmatically.

#### 2. Volume Detection End-to-End Flow

**Test:** With the app running (`cargo tauri dev`), connect an OT card or run `mkdir -p /tmp/test-ot/AUDIO /tmp/test-ot/SETS` and mount it as a volume (note: directory-only simulation may not work with sysinfo's removable disk scan — a real CF card or USB drive with these dirs is more reliable).
**Expected:**
- Within ~2 seconds: toast "Octatrack connected at {volumeName}." appears bottom-right
- After 500ms debounce: "Octatrack Found" dialog appears with mount path displayed and "Use this device" (amber) / "Not Now" (ghost) buttons
- Clicking "Use this device": dialog closes, content area switches to "Projects" placeholder, device status badge shows green dot with volume name
- Disconnecting: toast "Octatrack disconnected." appears, content resets to "No Device Connected"
- Clicking "Not Now": dialog closes, stays in disconnected state
**Why human:** Plan 07 visual checkpoint was auto-approved via `checkpoint_override` directive — not user-verified. Runtime event emission, dialog timing, and UI state transitions require live testing.

#### 3. Round-Trip Parser Against Real OT Project Files

**Test:** Copy real OT project files to `tests/fixtures/` (project.work, bank01.work through bank16.work, markers.work, arr01.work through arr08.work, and some .ot sidecar files), then run `cargo test -p ot-parser`.
**Expected:** All round-trip tests pass. The opaque-blob pattern should handle any body size since body is `Vec<u8>` parsed by `BinRead` to fill remaining bytes. If tests fail, it may indicate endianness assumptions or variable-length field issues.
**Why human:** SC-2 explicitly requires "a corpus of real OT project files." Synthetic fixtures were designed from public documentation; real files may reveal undocumented edge cases.

#### 4. Atomic Write on Real FAT32 Volume

**Test:** With an OT card mounted, use the Rust test harness to call `atomic_write(card_path.join("test.bin"), b"hello")` and verify the temp file and final file are on the card (same FAT32 volume), not on the Mac's local filesystem.
**Expected:** Temp file (`.atomic-write-*`) appears transiently in the same directory as the target, not in `/tmp`. Final file `test.bin` appears on the card. `sync_all()` guarantees flush to FAT32.
**Why human:** SC-3 says "verified by integration test on a real FAT32 volume." Unit tests run on macOS tmpfs. The behavior should be correct (AtomicWriteFile creates temp in same dir by design), but FAT32 filesystem semantics differ.

### Gaps Summary

No blocking gaps were found. All code exists and is substantive. All required artifacts are present. The two major open items are runtime/environmental verifications that require human action:

1. **App window launch** — Code compiles and Tauri is configured; runtime visual verification needed.
2. **Volume detection flow** — All code paths are wired; the visual checkpoint in Plan 07 was auto-approved, not user-approved. Needs user to run `cargo tauri dev` with a real or simulated OT volume.
3. **Real OT file corpus** — SC-2 requires real project files. Synthetic fixtures pass all tests but don't constitute "a corpus of real OT project files."
4. **FAT32 atomic write** — SC-3 requires an integration test on a real FAT32 volume.

Items 3 and 4 are the most substantive gaps against the roadmap success criteria. Items 1 and 2 are expected human-in-the-loop verification steps that could not be automated.

One notable observation: the production `run()` function uses `open_in_memory()` for the SQLite DB (line 46, `lib.rs`). This satisfies SC-5 (schema initializes) but means no data persistence. This is not blocking for Phase 1 (no write commands store data), but must be addressed in Phase 2 before project indexing commands are added.

---

_Verified: 2026-04-30T08:00:00Z_
_Verifier: Claude (gsd-verifier)_
