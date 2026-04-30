---
phase: 01-foundation
plan: "07"
subsystem: device-detection
tags: [rust, tauri, sysinfo, volume-detection, frontend, dialog, toast]
dependency_graph:
  requires: ["01-05", "01-06"]
  provides: ["volume-detection-stack", "device-confirm-flow"]
  affects: ["AppState", "device-commands", "frontend-shell"]
tech_stack:
  added:
    - tokio time feature (polling sleep in poll_loop)
  patterns:
    - sysinfo Disks::new_with_refreshed_list() for volume enumeration
    - tauri::async_runtime::spawn for background polling task
    - Tauri Emitter trait for ot-device-changed events
    - base-ui Dialog controlled with open prop
    - sonner toast for connect/disconnect notifications
    - 500ms setTimeout debounce before showing confirmation dialog
key_files:
  created:
    - crates/takoyaki-app/src/device/mod.rs
    - crates/takoyaki-app/src/commands/device.rs
    - src/lib/tauri.ts
    - src/components/volume-confirm-dialog.tsx
  modified:
    - crates/takoyaki-app/src/lib.rs (AppState + DeviceState moved here, polling started)
    - crates/takoyaki-app/src/commands/mod.rs (added device module)
    - crates/takoyaki-app/src/commands/projects.rs (uses crate::AppState, volume_path -> device.mount_point)
    - crates/takoyaki-app/src/commands/samples.rs (uses crate::AppState)
    - src/app/page.tsx (confirmation dialog + auto-navigate wired)
    - src/components/tauri-event-listener.tsx (toast notifications added)
    - crates/takoyaki-app/Cargo.toml (added tokio time feature)
decisions:
  - "AppState moved from commands/projects.rs to lib.rs — commands/projects.rs had a local AppState with volume_path field; refactored to crate-level AppState with DeviceState (mount_point + confirmed) so device commands can share state"
  - "tokio time feature added explicitly — Tauri provides tokio runtime but does not re-export tokio::time; explicit dep with features=[time] needed for poll_loop sleep"
  - "sysinfo disks.list() returns &[Disk] slice, not iterator — use .iter() not .collect() directly"
  - "base-ui Dialog onOpenChange receives boolean (isOpen: boolean) — type annotation needed to avoid TypeScript inference issue"
  - "VolumeConfirmDialog uses showCloseButton=false — footer buttons provide full dismiss affordance; X button would be redundant"
metrics:
  duration_minutes: 3
  completed_date: "2026-04-30"
  tasks_completed: 3
  files_changed: 12
---

# Phase 01 Plan 07: Volume Detection Stack Summary

**One-liner:** sysinfo-based OT volume polling with Tauri event emission, confirmation dialog ("Octatrack Found"), toast notifications, and auto-navigate to Projects on device confirm.

## What Was Built

The complete volume detection stack wiring all Phase 1 subsystems into an end-to-end flow:

**Rust backend:**
- `device/mod.rs`: `is_ot_volume()` checks for `/AUDIO` + `/SETS` directories. `detect_ot_volume()` scans removable disks via sysinfo with two fallbacks (all non-system mounts, then `/Volumes` direct scan) for macOS CF card reader compatibility. `poll_loop()` runs every 2 seconds and emits `ot-device-changed` event when state changes. `start_polling()` spawns the background task via `tauri::async_runtime::spawn`.
- `commands/device.rs`: Three Tauri commands — `get_device_status`, `confirm_device` (validates path with `is_ot_volume` per T-01-14), `dismiss_device`. All decorated with `#[tauri::command]` + `#[specta::specta]`.
- `lib.rs`: `AppState` (crate-level, replacing the local definition in commands/projects.rs) with `device: Mutex<DeviceState>` where `DeviceState` holds `mount_point` and `confirmed`. Polling started in `tauri::Builder::setup`.

**TypeScript frontend:**
- `src/lib/tauri.ts`: `getDeviceStatus`, `confirmDevice`, `dismissDevice` invoke wrappers.
- `src/components/volume-confirm-dialog.tsx`: "Octatrack Found" dialog with exact UI-SPEC.md copy, "Use this device" (accent) and "Not Now" (ghost) buttons.
- `src/components/tauri-event-listener.tsx`: Toast notifications — "Octatrack connected at {volumeName}." and "Octatrack disconnected."
- `src/app/page.tsx`: 500ms debounce before showing dialog, auto-navigate to Projects section on confirm (D-11), content area shows disconnected state until both `connected && confirmed`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] AppState location mismatch**
- **Found during:** Task 1
- **Issue:** Plan assumed `AppState` with `DeviceState` lived in `lib.rs`, but prior plans had defined `AppState` locally in `commands/projects.rs` with `volume_path: Mutex<Option<PathBuf>>` — incompatible with `device.mount_point` access pattern in device commands.
- **Fix:** Moved `AppState` + new `DeviceState` struct to `lib.rs` as crate-level types. Updated `commands/projects.rs` (remove local definition, change `volume_path` -> `device.mount_point`), `commands/samples.rs` (use `crate::AppState`), and `commands/device.rs` (import `crate::AppState`).
- **Files modified:** `lib.rs`, `commands/projects.rs`, `commands/samples.rs`, `commands/device.rs`
- **Commit:** 6c9d62a

**2. [Rule 3 - Blocking] tokio::time not in scope**
- **Found during:** Task 1 build
- **Issue:** `poll_loop` uses `tokio::time::sleep` but `tokio` was not a direct dependency — Tauri provides the runtime but does not re-export `tokio::time`.
- **Fix:** Added `tokio = { version = "1", features = ["time"] }` to `crates/takoyaki-app/Cargo.toml`.
- **Files modified:** `Cargo.toml`
- **Commit:** 6c9d62a

**3. [Rule 1 - Bug] sysinfo Disks::list() returns slice not iterator**
- **Found during:** Task 1 build
- **Issue:** `disks.list().collect()` fails — `list()` returns `&[Disk]`, not an `IntoIterator`. Required `.iter()` first.
- **Fix:** Changed `disks.list().collect()` to `disks.list().iter().collect()`.
- **Files modified:** `device/mod.rs`
- **Commit:** 6c9d62a

## Threat Model Coverage

All T-01-14 mitigations implemented: `confirm_device` validates mount path with `is_ot_volume()` before accepting, rejecting paths without `/AUDIO` and `/SETS`. T-01-15, T-01-16, T-01-17 accepted as documented in plan.

## Known Stubs

None that prevent the plan's goal. Content area shows "Project browser will be available in Phase 2." when connected and confirmed — this is intentional (Phase 2 deliverable).

## Checkpoint Note

Task 3 (visual verification of volume detection flow) was auto-approved per `<checkpoint_override>` directive. Full end-to-end flow requires running `cargo tauri dev` with a real OT card or simulated volume. The UI compiles and all code paths are wired.

## Self-Check: PASSED

Files exist:
- crates/takoyaki-app/src/device/mod.rs: FOUND
- crates/takoyaki-app/src/commands/device.rs: FOUND
- src/lib/tauri.ts: FOUND
- src/components/volume-confirm-dialog.tsx: FOUND

Commits exist:
- 6c9d62a (Task 1): FOUND
- 31ef760 (Task 2): FOUND
