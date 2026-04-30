# Phase 1: Foundation - Pattern Map

**Mapped:** 2026-04-29
**Files analyzed:** 22 new files (greenfield project)
**Analogs found:** 22 / 22 (all from Wallflower sister project at /Users/albair/src/wallflower)

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `Cargo.toml` (workspace root) | config | — | `Cargo.toml` (Wallflower root) | exact |
| `crates/ot-parser/Cargo.toml` | config | — | `crates/wallflower-core/Cargo.toml` | role-match |
| `crates/ot-parser/src/lib.rs` | utility | transform | `crates/wallflower-core/src/lib.rs` | role-match |
| `crates/ot-parser/src/sample.rs` | utility | transform | `crates/wallflower-core/src/export/writer.rs` | role-match |
| `crates/ot-parser/src/project.rs` | utility | transform | `crates/wallflower-core/src/export/writer.rs` | role-match |
| `crates/ot-parser/src/bank.rs` | utility | transform | `crates/wallflower-core/src/export/writer.rs` | role-match |
| `crates/ot-parser/src/error.rs` | utility | — | `crates/wallflower-core/src/error.rs` | exact |
| `crates/takoyaki-app/Cargo.toml` | config | — | `crates/wallflower-app/Cargo.toml` | exact |
| `crates/takoyaki-app/build.rs` | config | — | `crates/wallflower-app/build.rs` | exact |
| `crates/takoyaki-app/tauri.conf.json` | config | — | `crates/wallflower-app/tauri.conf.json` | exact |
| `crates/takoyaki-app/capabilities/default.json` | config | — | `crates/wallflower-app/capabilities/default.json` | exact |
| `crates/takoyaki-app/src/main.rs` | config | — | `crates/wallflower-app/src/main.rs` | exact |
| `crates/takoyaki-app/src/lib.rs` | provider | request-response | `crates/wallflower-app/src/lib.rs` | exact |
| `crates/takoyaki-app/src/error.rs` | utility | — | `crates/wallflower-core/src/error.rs` | exact |
| `crates/takoyaki-app/src/commands/device.rs` | service | event-driven | `crates/wallflower-app/src/commands/status.rs` | role-match |
| `crates/takoyaki-app/src/db/mod.rs` | service | CRUD | `crates/wallflower-core/src/db/mod.rs` | exact |
| `crates/takoyaki-app/src/atomic/mod.rs` | service | file-I/O | `crates/wallflower-core/src/export/writer.rs` | role-match |
| `migrations/V1__initial_schema.sql` | config | — | `migrations/V1__initial_schema.sql` (Wallflower) | exact |
| `src/app/globals.css` | config | — | `src/app/globals.css` (Wallflower) | exact |
| `src/app/layout.tsx` | component | request-response | `src/app/layout.tsx` (Wallflower) | exact |
| `src/app/page.tsx` | component | request-response | `src/app/page.tsx` (Wallflower) | exact |
| `src/components/providers.tsx` | provider | — | `src/components/providers.tsx` (Wallflower) | exact |
| `src/lib/tauri.ts` | utility | request-response | `src/lib/tauri.ts` (Wallflower) | exact |
| `src/lib/stores/device.ts` | store | event-driven | `src/lib/stores/library.ts` (Wallflower) | role-match |
| `src/components/tauri-event-listener.tsx` | component | event-driven | `src/components/tauri-event-listener.tsx` (Wallflower) | exact |
| `next.config.mjs` | config | — | `next.config.mjs` (Wallflower) | exact |
| `package.json` | config | — | `package.json` (Wallflower) | exact |
| `tsconfig.json` | config | — | `tsconfig.json` (Wallflower) | exact |
| `components.json` | config | — | `components.json` (Wallflower) | exact |

---

## Pattern Assignments

### `Cargo.toml` (workspace root)

**Analog:** `/Users/albair/src/wallflower/Cargo.toml`

**Workspace config pattern** (lines 1-18):
```toml
[workspace]
members = [
    "crates/ot-parser",
    "crates/takoyaki-app",
]
resolver = "2"

[workspace.dependencies]
rusqlite = { version = "0.39", features = ["bundled"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
thiserror = "2"
```

**Key difference from Wallflower:** No `tokio`, no `anyhow` in workspace deps for Phase 1 (Tauri uses its own async runtime). Use `edition = "2021"` initially — Wallflower uses `"2024"` successfully but verify per RESEARCH.md assumption A3.

---

### `crates/ot-parser/Cargo.toml` (pure library crate)

**Analog:** `/Users/albair/src/wallflower/crates/wallflower-core/Cargo.toml` (lines 1-8 structure, not full deps)

**Library crate pattern:**
```toml
[package]
name = "ot-parser"
version = "0.1.0"
edition = "2021"

[dependencies]
binrw = "0.15"
serde = { workspace = true }
thiserror = { workspace = true }

# Zero Tauri dependencies — enforced constraint (FNDN-04 boundary)
# If `cargo test -p ot-parser` compiles cleanly, the boundary is maintained
```

**Key difference from Wallflower-core:** No rusqlite, no tokio, no cpal, no dirs — parser is pure transform with no I/O.

---

### `crates/ot-parser/src/error.rs` (parser error type)

**Analog:** `/Users/albair/src/wallflower/crates/wallflower-core/src/error.rs` (lines 1-18, exact pattern)

**Error type pattern** (all 18 lines):
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Checksum mismatch: expected {expected:#06x}, got {actual:#06x}")]
    ChecksumMismatch { expected: u16, actual: u16 },

    #[error("Invalid magic bytes")]
    InvalidMagic,
}

pub type Result<T> = std::result::Result<T, ParseError>;
```

**Key difference from Wallflower:** No `Db` variant (parser has no DB). No `Config` variant. Add `ChecksumMismatch` and `InvalidMagic` for OT-specific validation.

---

### `crates/ot-parser/src/lib.rs` (crate root)

**Analog:** `/Users/albair/src/wallflower/crates/wallflower-core/src/lib.rs`

**Crate root pattern:**
```rust
pub mod error;
pub mod sample;
pub mod project;
pub mod bank;
pub mod markers;
pub mod arrangement;

pub use error::{ParseError, Result};
```

No `pub use` re-exports beyond error types at the lib root — keep modules explicit for a clean API surface.

---

### `crates/ot-parser/src/sample.rs` (binrw parser for .ot files)

**Analog:** RESEARCH.md Pattern 1 (no direct Wallflower analog — binary parsing is new territory). Use RESEARCH.md code examples directly.

**binrw struct pattern** (from RESEARCH.md Pattern 1 + Pattern 2):
```rust
use binrw::{BinRead, BinWrite, binrw};
use crate::error::Result;
use std::io::Cursor;

/// .ot sidecar file — 832 bytes total (verified from OctaChainer otwriter.h)
/// Magic: [0xF0, 0x00, 0x00, 0xE8, 0x57, 0x45, 0x52, 0x41, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
/// All multi-byte fields: big endian
#[binrw]
#[brw(big)]
#[derive(Debug, Clone, PartialEq)]
pub struct SampleSettingsFile {
    pub header: [u8; 16],
    pub unknown_0x10: [u8; 7],   // preserve verbatim — D-02
    pub tempo: u32,
    pub trim_len: u32,
    pub loop_len: u32,
    pub stretch: u32,
    pub loop_flag: u32,
    pub gain: u16,
    pub quantize: u8,
    pub trim_start: u32,
    pub trim_end: u32,
    pub loop_point: u32,
    #[br(count = 64)]
    pub slices: Vec<Slice>,
    pub slice_count: u32,
    pub checksum: u16,
}

impl SampleSettingsFile {
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        Ok(Self::read(&mut cursor).map_err(|e| crate::error::ParseError::Parse(e.to_string()))?)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut cursor = Cursor::new(Vec::new());
        self.write(&mut cursor).map_err(|e| crate::error::ParseError::Parse(e.to_string()))?;
        Ok(cursor.into_inner())
    }
}
```

**Round-trip test pattern** (add to `tests/round_trip.rs`):
```rust
#[test]
fn test_sample_round_trip() {
    let bytes = include_bytes!("../../../tests/fixtures/sample.ot");
    let parsed = SampleSettingsFile::from_bytes(bytes).unwrap();
    let rewritten = parsed.to_bytes().unwrap();
    assert_eq!(bytes.as_ref(), rewritten.as_slice());
}
```

**Opaque unknown bytes pattern** (from RESEARCH.md Pattern 2):
```rust
// For undocumented regions in .work and bank files:
#[br(count = 0x1A2)]
pub unknown_region_1: Vec<u8>,  // preserved verbatim — round-trip safe
```

---

### `crates/takoyaki-app/Cargo.toml` (app crate)

**Analog:** `/Users/albair/src/wallflower/crates/wallflower-app/Cargo.toml`

**App crate pattern** (adapt from Wallflower, removing unneeded deps):
```toml
[package]
name = "takoyaki-app"
version = "0.1.0"
edition = "2021"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
ot-parser = { path = "../ot-parser" }
tauri = { version = "2", features = [] }
tauri-specta = { version = "=2.0.0-rc.24", features = ["derive", "typescript"] }
specta = "=2.0.0-rc.24"
atomic-write-file = "0.3"
tempfile = "3"
sysinfo = "0.35"
rusqlite = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
thiserror = { workspace = true }
dirs = "6"

# tauri-specta rc pinning: use exact `=` prefix, not `^`
# Wallflower does NOT use tauri-specta — Takoyaki adds it for TypeScript type safety
```

**Critical note:** Wallflower uses `tauri::generate_handler![]` directly. Takoyaki uses `tauri-specta`'s `Builder` + `collect_commands!` pattern instead — see lib.rs pattern below.

---

### `crates/takoyaki-app/build.rs`

**Analog:** `/Users/albair/src/wallflower/crates/wallflower-app/build.rs` (lines 1-12, strip tonic)

**Build script pattern:**
```rust
fn main() {
    tauri_build::build();
}
```

Wallflower's build.rs also compiles protobuf (lines 2-9) — omit that for Takoyaki. The core `tauri_build::build()` call is the required pattern.

---

### `crates/takoyaki-app/tauri.conf.json`

**Analog:** `/Users/albair/src/wallflower/crates/wallflower-app/tauri.conf.json` (all 39 lines)

**Configuration pattern:**
```json
{
  "productName": "Takoyaki",
  "version": "0.1.0",
  "identifier": "com.takoyaki.app",
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:3000",
    "frontendDist": "../../out"
  },
  "bundle": {
    "active": true,
    "targets": ["dmg", "app"],
    "macOS": {
      "signingIdentity": null,
      "minimumSystemVersion": "13.0"
    }
  },
  "app": {
    "windows": [
      {
        "title": "Takoyaki",
        "width": 1200,
        "height": 800,
        "minWidth": 900,
        "minHeight": 600
      }
    ],
    "security": {
      "csp": null,
      "assetProtocol": {
        "enable": true,
        "scope": ["$APPDATA/**"]
      }
    }
  }
}
```

**Key differences from Wallflower:** No `$HOME/wallflower/**` in asset scope. No tray icon feature. No entitlements initially (add when needed).

---

### `crates/takoyaki-app/capabilities/default.json`

**Analog:** `/Users/albair/src/wallflower/crates/wallflower-app/capabilities/default.json` (all 25 lines)

**Capabilities pattern** (minimal for Phase 1):
```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default capabilities for Takoyaki",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:default",
    "core:window:allow-close",
    "core:window:allow-set-title"
  ]
}
```

Wallflower's capabilities (lines 8-23) include notification, global-shortcut, autostart — omit all of these for Takoyaki Phase 1. Add only what commands require.

---

### `crates/takoyaki-app/src/main.rs`

**Analog:** `/Users/albair/src/wallflower/crates/wallflower-app/src/main.rs` (all 5 lines, exact copy)

**Entry point pattern** (lines 1-5 — verbatim):
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    takoyaki_app::run()
}
```

This is identical in every Tauri app. Copy verbatim, change crate name only.

---

### `crates/takoyaki-app/src/lib.rs` (Tauri app setup)

**Analog:** `/Users/albair/src/wallflower/crates/wallflower-app/src/lib.rs`

**AppState pattern** (lines 33-43, adapt):
```rust
use std::sync::Mutex;
use rusqlite::Connection;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub device: Mutex<DeviceState>,
}

pub struct DeviceState {
    pub mount_point: Option<std::path::PathBuf>,
}
```

**Tauri builder pattern** — Wallflower uses `tauri::generate_handler![]` (line 438). Takoyaki uses tauri-specta's builder instead (from RESEARCH.md Pattern 5):
```rust
pub fn run() {
    let db = db::open_database(&db::default_path()).expect("failed to open database");

    let builder = tauri_specta::Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            commands::device::get_device_status,
            commands::device::confirm_device,
        ]);

    #[cfg(debug_assertions)]
    builder
        .export(specta_typescript::Typescript::default(), "../src/bindings.ts")
        .expect("Failed to export TypeScript bindings");

    tauri::Builder::default()
        .setup(move |app| {
            // Start device polling background task
            device::start_polling(app.handle().clone());
            Ok(())
        })
        .manage(AppState {
            db: Mutex::new(db),
            device: Mutex::new(DeviceState { mount_point: None }),
        })
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Background task spawn pattern** (from Wallflower lib.rs lines 322-324):
```rust
tauri::async_runtime::spawn(async move {
    device::poll_loop(app_handle, tx).await;
});
```

---

### `crates/takoyaki-app/src/error.rs` (Tauri command errors)

**Analog:** `/Users/albair/src/wallflower/crates/wallflower-core/src/error.rs` (lines 1-18)

**Critical difference:** Wallflower's `WallflowerError` does NOT implement `serde::Serialize` — Wallflower commands return `Result<T, String>` (via `.map_err(|e| e.to_string())`). Takoyaki uses tauri-specta which requires a serializable error type. Pattern from RESEARCH.md Pattern 5:

```rust
use thiserror::Error;
use serde::Serialize;

/// AppError is used as the error type for all #[tauri::command] functions.
/// It MUST implement serde::Serialize — returning non-serializable errors
/// causes a runtime panic in Tauri IPC.
#[derive(Debug, Error, Serialize)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Database error: {0}")]
    Db(String),

    #[error("Device error: {0}")]
    Device(String),

    #[error("Invalid path")]
    InvalidPath,
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self { AppError::Io(e.to_string()) }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self { AppError::Db(e.to_string()) }
}

impl From<ot_parser::ParseError> for AppError {
    fn from(e: ot_parser::ParseError) -> Self { AppError::Parse(e.to_string()) }
}

pub type Result<T> = std::result::Result<T, AppError>;
```

---

### `crates/takoyaki-app/src/commands/device.rs` (device status commands)

**Analog:** `/Users/albair/src/wallflower/crates/wallflower-app/src/commands/status.rs` (all 38 lines)

**Command pattern** (lines 15-38):
```rust
use tauri::State;
use serde::Serialize;
use crate::AppState;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DeviceStatus {
    pub connected: bool,
    pub mount_point: Option<String>,
    pub confirmed: bool,
}

#[tauri::command]
#[specta::specta]
pub async fn get_device_status(
    state: State<'_, AppState>,
) -> Result<DeviceStatus, AppError> {
    let device = state.device.lock().map_err(|e| AppError::Device(e.to_string()))?;
    Ok(DeviceStatus {
        connected: device.mount_point.is_some(),
        mount_point: device.mount_point.as_ref().map(|p| p.to_string_lossy().to_string()),
        confirmed: device.confirmed,
    })
}
```

**Key differences from Wallflower status.rs:**
- Add `#[specta::specta]` on each command (Wallflower omits this — it doesn't use tauri-specta)
- Return `Result<T, AppError>` not `Result<T, String>`
- Response struct derives `specta::Type` for TypeScript generation

---

### `crates/takoyaki-app/src/db/mod.rs` (database layer)

**Analog:** `/Users/albair/src/wallflower/crates/wallflower-core/src/db/mod.rs` (lines 62-193)

**Database open pattern** (Wallflower lines 65-87, adapt):
```rust
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use crate::error::AppError;
use tracing::info;

const MIGRATION_V1: &str = include_str!("../../../migrations/V1__initial_schema.sql");

pub fn open_database(path: &Path) -> Result<Connection, AppError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(path)?;
    initialize(&conn)?;
    Ok(conn)
}

pub fn default_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("takoyaki")
        .join("takoyaki.db")
}

pub fn open_in_memory() -> Result<Connection, AppError> {
    let conn = Connection::open_in_memory()?;
    initialize(&conn)?;
    Ok(conn)
}

fn initialize(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    let current_version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    if current_version < 1 {
        info!("Running V1 migration: initial schema");
        conn.execute_batch(MIGRATION_V1)?;
        conn.execute_batch("PRAGMA user_version = 1;")?;
    }

    Ok(())
}
```

**Wallflower uses a `schema_version` table for V1-V2 migrations** (lines 112-155) then switches to `user_version` pragma for V3+ (lines 158-193). For Takoyaki, start clean with `user_version` pragma only from the beginning (simpler and correct).

**Read-only Wallflower DB connection** (from RESEARCH.md Pattern 7):
```rust
pub fn open_wallflower_db(path: &Path) -> Result<Connection, AppError> {
    use rusqlite::OpenFlags;
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    Ok(conn)
}
```

---

### `crates/takoyaki-app/src/device/mod.rs` (volume detection + polling)

**Analog:** `/Users/albair/src/wallflower/crates/wallflower-core/src/device/mod.rs` (lines 27-84)

Wallflower scans `/Volumes` directly (lines 33-83). Takoyaki uses `sysinfo` polling instead (per D-14 and RESEARCH.md Pattern 4). The detection logic structure is the same:

**Volume detection pattern** (from RESEARCH.md Pattern 4):
```rust
use sysinfo::Disks;
use std::path::PathBuf;

const OT_SIGNATURE_DIRS: &[&str] = &["AUDIO", "SETS"];

pub fn detect_ot_volume() -> Option<PathBuf> {
    let disks = Disks::new_with_refreshed_list();
    for disk in disks.list() {
        if !disk.is_removable() { continue; }
        let mount = disk.mount_point();
        let is_ot = OT_SIGNATURE_DIRS.iter().all(|d| mount.join(d).is_dir());
        if is_ot {
            return Some(mount.to_path_buf());
        }
    }
    None
}
```

**Polling loop pattern** (from Wallflower lib.rs background task lines 83-196, adapt for sysinfo):
```rust
use tauri::{AppHandle, Emitter};
use std::time::Duration;

pub async fn poll_loop(app: AppHandle) {
    let mut last_state: Option<PathBuf> = None;
    loop {
        tokio::time::sleep(Duration::from_secs(2)).await;
        let current = detect_ot_volume();
        if current != last_state {
            let _ = app.emit("ot-device-changed", &current.as_ref().map(|p| p.to_string_lossy().to_string()));
            last_state = current;
        }
    }
}

pub fn start_polling(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        poll_loop(app).await;
    });
}
```

**Test pattern** (from Wallflower device/mod.rs lines 180-190):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_detect_ot_volume_negative() {
        // On a dev machine without an OT card, detection should return None
        let result = detect_ot_volume();
        // Can't assert None (user might have OT connected), but must not panic
        let _ = result;
    }

    #[test]
    fn test_ot_signature_check() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("AUDIO")).unwrap();
        std::fs::create_dir_all(tmp.path().join("SETS")).unwrap();
        // Simulate the signature check logic directly (not full detect_ot_volume)
        let is_ot = OT_SIGNATURE_DIRS.iter().all(|d| tmp.path().join(d).is_dir());
        assert!(is_ot);
    }
}
```

---

### `crates/takoyaki-app/src/atomic/mod.rs` (atomic write engine)

**Analog:** No direct Wallflower analog (Wallflower does not do atomic writes to external volumes). Use RESEARCH.md Pattern 3 directly.

**Atomic write pattern** (from RESEARCH.md Pattern 3):
```rust
use atomic_write_file::AtomicWriteFile;
use std::path::Path;
use std::io::Write;
use crate::error::AppError;

pub fn atomic_write(target_path: &Path, content: &[u8]) -> Result<(), AppError> {
    // Stage on same volume as target (required for FAT32 rename atomicity)
    let mut staging = AtomicWriteFile::options()
        .open(target_path)
        .map_err(|e| AppError::Io(e.to_string()))?;
    staging.write_all(content)?;
    staging.flush()?;
    // sync_all() calls F_FULLFSYNC on macOS — protects against hot-unplug
    staging.sync_all()?;
    staging.commit().map_err(|e| AppError::Io(e.to_string()))?;
    // Also sync the parent directory inode
    let parent = target_path.parent().ok_or(AppError::InvalidPath)?;
    let dir = std::fs::File::open(parent)?;
    dir.sync_all()?;
    Ok(())
}
```

**Snapshot pattern** (wraps atomic_write, runs before any write operation):
```rust
use std::path::{Path, PathBuf};
use chrono::Utc;

pub struct SnapshotEngine {
    snapshot_root: PathBuf,
}

impl SnapshotEngine {
    pub fn snapshot_files(&self, files: &[&Path]) -> Result<PathBuf, AppError> {
        let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
        let snapshot_dir = self.snapshot_root.join(&timestamp);
        std::fs::create_dir_all(&snapshot_dir)?;
        for src in files {
            let filename = src.file_name().ok_or(AppError::InvalidPath)?;
            let dest = snapshot_dir.join(filename);
            std::fs::copy(src, &dest)?;
        }
        Ok(snapshot_dir)
    }
}
```

---

### `migrations/V1__initial_schema.sql`

**Analog:** `/Users/albair/src/wallflower/migrations/V1__initial_schema.sql` (all 29 lines — structure, not content)

**Migration file pattern** (from RESEARCH.md Code Examples — SQLite Schema):
```sql
-- V1__initial_schema.sql
-- Takoyaki own metadata database

CREATE TABLE snapshots (
    id         TEXT PRIMARY KEY NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    operation  TEXT NOT NULL,
    project_path TEXT,
    file_count INTEGER NOT NULL,
    total_bytes INTEGER NOT NULL
);

CREATE TABLE snapshot_files (
    id           TEXT PRIMARY KEY NOT NULL,
    snapshot_id  TEXT NOT NULL REFERENCES snapshots(id) ON DELETE CASCADE,
    original_path TEXT NOT NULL,
    stored_path  TEXT NOT NULL,
    file_hash    TEXT NOT NULL
);

CREATE TABLE projects (
    id           TEXT PRIMARY KEY NOT NULL,
    set_name     TEXT NOT NULL,
    project_name TEXT NOT NULL,
    card_path    TEXT NOT NULL UNIQUE,
    tempo_bpm    INTEGER,
    bank_count   INTEGER,
    last_modified TEXT,
    indexed_at   TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_projects_card_path ON projects(card_path);
CREATE INDEX idx_snapshots_project_path ON snapshots(project_path);
CREATE INDEX idx_snapshots_created_at ON snapshots(created_at);
```

Wallflower's V1 uses `TEXT PRIMARY KEY` with `uuid` generated in Rust (not SQLite autoincrement) — follow the same pattern for `id` fields.

---

### `src/app/globals.css` (Tailwind v4 theme)

**Analog:** `/Users/albair/src/wallflower/src/app/globals.css` (all 114 lines — structure exact, values differ)

**CSS structure pattern** (lines 1-33 structure, lines 35-61 token pattern):
```css
@import "tailwindcss";
@import "tw-animate-css";
@import "shadcn/tailwind.css";

@theme inline {
    --font-mono: 'Iosevka', 'JetBrains Mono', monospace;
    --font-sans: 'Inter', system-ui, sans-serif;
    --color-ring: var(--ring);
    --color-input: var(--input);
    --color-border: var(--border);
    /* ... full shadcn token map as in Wallflower lines 8-32 ... */
}

:root {
    /* Warm dark palette per UI-SPEC.md — different from Wallflower's cool dark */
    --background: hsl(30 8% 10%);      /* #1c1a18 — warm dark base */
    --foreground: hsl(30 10% 88%);     /* #e3ddd6 — off-white */
    --card: hsl(30 8% 16%);
    --card-foreground: hsl(30 10% 88%);
    --popover: hsl(30 8% 16%);
    --popover-foreground: hsl(30 10% 88%);
    --primary: hsl(38 85% 55%);        /* amber accent */
    --primary-foreground: hsl(0 0% 100%);
    --secondary: hsl(30 8% 16%);
    --secondary-foreground: hsl(30 10% 88%);
    --muted: hsl(30 8% 18%);
    --muted-foreground: hsl(30 8% 50%);
    --accent: hsl(38 85% 55%);
    --accent-foreground: hsl(0 0% 100%);
    --destructive: hsl(0 68% 48%);
    --destructive-foreground: hsl(0 0% 100%);
    --border: hsl(30 8% 26%);
    --input: hsl(30 8% 26%);
    --ring: hsl(38 85% 55%);
    --radius: 0.375rem;                /* hardware-precise per D-06 */
}
```

**Base layer pattern** (Wallflower lines 92-114 — copy exactly):
```css
@layer base {
  * {
    @apply border-border outline-ring/50;
  }
  body {
    @apply bg-background text-foreground;
    -webkit-user-select: none;
    user-select: none;
    -webkit-tap-highlight-color: transparent;
    overflow: hidden;
    height: 100vh;
    font-family: var(--font-mono), monospace;  /* monospace-forward per D-08 */
  }
  html {
    overflow: hidden;
    overscroll-behavior: none;
    height: 100vh;
  }
  input, textarea, [contenteditable="true"] {
    -webkit-user-select: text;
    user-select: text;
  }
}
```

**Key difference:** Takoyaki uses `--font-mono` as the body font (D-08 monospace-forward). Wallflower uses `--font-sans`. Also omit Wallflower's custom waveform tokens; add OT-specific tokens as needed.

---

### `src/app/layout.tsx` (root layout)

**Analog:** `/Users/albair/src/wallflower/src/app/layout.tsx` (all 49 lines)

**Layout pattern** (lines 1-49, adapt):
```tsx
import type { Metadata } from "next";
import "./globals.css";
import "@fontsource/iosevka/400.css";
import "@fontsource/iosevka/500.css";
import { Providers } from "@/components/providers";
import { Toaster } from "@/components/ui/sonner";
import { TauriEventListener } from "@/components/tauri-event-listener";

export const metadata: Metadata = {
  title: "Takoyaki",
  description: "Octatrack backup and project manager",
};

export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en">
      <head>
        <meta name="color-scheme" content="dark" />
      </head>
      <body className="min-h-screen">
        <Providers>
          <div id="main-content" className="h-screen overflow-hidden">
            {children}
          </div>
          <TauriEventListener />
          <Toaster position="bottom-right" />
        </Providers>
      </body>
    </html>
  );
}
```

**Key differences from Wallflower:**
- `@fontsource/iosevka` not `@fontsource/plus-jakarta-sans` (D-08)
- No `TransportBar` (Takoyaki has no persistent transport bar in Phase 1)
- No `HighContrastProvider` / `SkipLink` in Phase 1 (accessibility stretch goals)
- `overflow: hidden` on body matches D-12 disconnected shell behavior

---

### `src/app/page.tsx` (home page / shell)

**Analog:** `/Users/albair/src/wallflower/src/app/page.tsx` (all 155 lines — structure, not content)

**Page pattern** (lines 1-22 imports + state pattern):
```tsx
"use client";

import { useState } from "react";
import { useDeviceStore } from "@/lib/stores/device";

type ActiveSection = "projects" | "samples" | "backups" | "settings";

export default function Home() {
  const [activeSection, setActiveSection] = useState<ActiveSection>("projects");
  const { connected, mountPoint } = useDeviceStore();
  // ...
}
```

**Sidebar nav pattern** (from Wallflower page.tsx lines 65-113 — tab bar becomes sidebar):
```tsx
{/* Sidebar navigation — D-07: all sections visible, inactive = disabled */}
<nav className="flex flex-col gap-1 p-3 border-r border-border w-56">
  {NAV_SECTIONS.map(({ key, label, icon: Icon, available }) => (
    <button
      key={key}
      onClick={() => available && setActiveSection(key)}
      disabled={!available}
      className={`flex items-center gap-2 px-3 py-2 rounded text-sm font-mono
        transition-colors
        ${activeSection === key
          ? "bg-accent/20 text-accent"
          : available
            ? "text-foreground hover:bg-muted"
            : "text-muted-foreground/40 cursor-not-allowed"
        }`}
    >
      <Icon size={14} />
      {label}
    </button>
  ))}
</nav>
```

**No-device state pattern** (D-12 — inline, not modal):
```tsx
{/* Content area — D-12: show inline "No device" when disconnected */}
<main className="flex-1 flex items-center justify-center">
  {!connected ? (
    <div className="text-center text-muted-foreground font-mono">
      <p className="text-sm">No Octatrack detected</p>
      <p className="text-xs mt-1 opacity-60">Connect via USB disk mode to begin</p>
    </div>
  ) : (
    <ProjectsView mountPoint={mountPoint!} />
  )}
</main>
```

---

### `src/components/providers.tsx`

**Analog:** `/Users/albair/src/wallflower/src/components/providers.tsx` (all 22 lines — copy exactly)

**Providers pattern** (lines 1-22, verbatim):
```tsx
"use client";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";

export function Providers({ children }: { children: ReactNode }) {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            retry: 1,
            refetchOnWindowFocus: false,
          },
        },
      }),
  );

  return (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
}
```

This is identical. Copy verbatim.

---

### `src/lib/tauri.ts` (Tauri invoke wrappers)

**Analog:** `/Users/albair/src/wallflower/src/lib/tauri.ts` (lines 1-30 import pattern, then function pattern)

**Invoke wrapper pattern** (lines 1-2, 34-41):
```typescript
import { invoke } from "@tauri-apps/api/core";
import type { DeviceStatus } from "./bindings";  // tauri-specta generated

export async function getDeviceStatus(): Promise<DeviceStatus> {
  return invoke("get_device_status");
}

export async function confirmDevice(mountPoint: string): Promise<void> {
  return invoke("confirm_device", { mountPoint });
}
```

**Key difference from Wallflower:** Takoyaki imports types from `./bindings` (tauri-specta generated file at `src/bindings.ts`). Wallflower maintains `src/lib/types.ts` manually. The bindings file is auto-generated on `cargo tauri dev` — do not hand-edit.

---

### `src/lib/stores/device.ts` (device connection state)

**Analog:** `/Users/albair/src/wallflower/src/lib/stores/library.ts` (all 40 lines — zustand store pattern)

**Zustand store pattern** (lines 1-40, adapt):
```typescript
import { create } from "zustand";

export interface DeviceState {
  connected: bool;
  mountPoint: string | null;
  confirmed: bool;
  setConnected: (connected: bool, mountPoint: string | null) => void;
  setConfirmed: (confirmed: bool) => void;
  reset: () => void;
}

export const useDeviceStore = create<DeviceState>((set) => ({
  connected: false,
  mountPoint: null,
  confirmed: false,
  setConnected: (connected, mountPoint) => set({ connected, mountPoint }),
  setConfirmed: (confirmed) => set({ confirmed }),
  reset: () => set({ connected: false, mountPoint: null, confirmed: false }),
}));
```

Wallflower's library store (lines 23-40) shows the `create<State>((set) => ({...}))` pattern — copy this structure exactly, substituting device state fields.

---

### `src/components/tauri-event-listener.tsx` (backend event bridge)

**Analog:** `/Users/albair/src/wallflower/src/components/tauri-event-listener.tsx` (all 339 lines — structure exact)

**Event listener pattern** (lines 80-120, adapt):
```tsx
"use client";

import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { useDeviceStore } from "@/lib/stores/device";

export function TauriEventListener() {
  const queryClient = useQueryClient();
  const { setConnected, reset } = useDeviceStore();

  useEffect(() => {
    const cleanupFns: (() => void)[] = [];

    async function setupListeners() {
      try {
        const { listen } = await import("@tauri-apps/api/event");

        const unlistenDevice = await listen<string | null>(
          "ot-device-changed",
          (event) => {
            if (event.payload) {
              setConnected(true, event.payload);
            } else {
              reset();
            }
          }
        );
        cleanupFns.push(unlistenDevice);

      } catch {
        // Not running in Tauri context (SSR / dev in browser)
      }
    }

    setupListeners();
    return () => { for (const cleanup of cleanupFns) cleanup(); };
  }, [setConnected, reset, queryClient]);

  return null;
}
```

**Key patterns from Wallflower lines 84-319:**
- Dynamic import `await import("@tauri-apps/api/event")` inside the async function (prevents SSR errors)
- `cleanupFns` array pattern with cleanup on unmount (lines 83, 323-326)
- Return `null` — this component renders nothing, it only registers listeners
- Wrap entire `setupListeners()` in `try/catch {}` — Tauri API throws when not in Tauri context

---

### `next.config.mjs`

**Analog:** `/Users/albair/src/wallflower/next.config.mjs` (all 7 lines — copy exactly)

**Config pattern** (lines 1-7, verbatim):
```js
/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'export',
  images: { unoptimized: true },
};

export default nextConfig;
```

This is identical. Copy verbatim. The `output: 'export'` and `images: { unoptimized: true }` are required for Tauri static export.

---

### `package.json`

**Analog:** `/Users/albair/src/wallflower/package.json` (all 45 lines — structure, adapted deps)

**Package.json pattern** (adapt from Wallflower, remove Wallflower-specific deps):
```json
{
  "name": "takoyaki",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "dev": "next dev --turbopack",
    "build": "next build",
    "tauri": "tauri"
  },
  "dependencies": {
    "@fontsource/iosevka": "latest",
    "@tanstack/react-query": "^5.100.6",
    "@tauri-apps/api": "^2.10.1",
    "class-variance-authority": "^0.7.1",
    "clsx": "^2.1.1",
    "lucide-react": "^1.8.0",
    "next": "^16.2.4",
    "react": "^19.2.5",
    "react-dom": "^19.2.5",
    "shadcn": "^4.3.0",
    "sonner": "^2.0.7",
    "tailwind-merge": "^3.5.0",
    "tw-animate-css": "^1.4.0",
    "typescript": "^6.0.3",
    "zustand": "^5.0.12"
  },
  "devDependencies": {
    "@tailwindcss/postcss": "^4.2.2",
    "@tauri-apps/cli": "^2.10.1",
    "@types/react": "^19.2.14",
    "@types/react-dom": "^19.2.3",
    "@types/node": "latest",
    "postcss": "^8.5.10",
    "tailwindcss": "^4.2.2"
  }
}
```

**Removed from Wallflower:** `@base-ui/react`, `@wavesurfer/react`, `wavesurfer.js`, `cmdk`, `next-themes`, `@fontsource/plus-jakarta-sans`, `@tauri-apps/plugin-autostart`, `@tauri-apps/plugin-notification`.

---

### `tsconfig.json`

**Analog:** `/Users/albair/src/wallflower/tsconfig.json` (all 41 lines — copy exactly)

Copy verbatim. The `@/*` path alias (line 26) is the key convention used throughout the frontend.

---

### `components.json` (shadcn config)

**Analog:** `/Users/albair/src/wallflower/components.json` (all 26 lines — copy exactly, change css path)

**shadcn config pattern:**
```json
{
  "$schema": "https://ui.shadcn.com/schema.json",
  "style": "base-nova",
  "rsc": true,
  "tsx": true,
  "tailwind": {
    "config": "",
    "css": "src/app/globals.css",
    "baseColor": "neutral",
    "cssVariables": true,
    "prefix": ""
  },
  "iconLibrary": "lucide",
  "rtl": false,
  "aliases": {
    "components": "@/components",
    "utils": "@/lib/utils",
    "ui": "@/components/ui",
    "lib": "@/lib",
    "hooks": "@/hooks"
  }
}
```

Identical to Wallflower. Copy verbatim.

---

### `src/lib/utils.ts`

**Analog:** `/Users/albair/src/wallflower/src/lib/utils.ts` (all 6 lines — copy exactly)

```typescript
import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}
```

Copy verbatim. Required by all shadcn components.

---

## Shared Patterns

### Rust: Tauri Command Structure
**Source:** `/Users/albair/src/wallflower/crates/wallflower-app/src/commands/status.rs` lines 15-33
**Apply to:** All files in `crates/takoyaki-app/src/commands/`

Wallflower pattern (returns `Result<T, String>`):
```rust
#[tauri::command]
pub async fn get_status(state: tauri::State<'_, AppState>) -> Result<StatusResponse, String> {
    let db_guard = state.db.lock().map_err(|e| e.to_string())?;
    // ...
}
```

Takoyaki adaptation (returns `Result<T, AppError>` with `#[specta::specta]`):
```rust
#[tauri::command]
#[specta::specta]
pub async fn get_device_status(state: tauri::State<'_, AppState>) -> Result<DeviceStatus, AppError> {
    let device = state.device.lock().map_err(|e| AppError::Device(e.to_string()))?;
    // ...
}
```

The `#[specta::specta]` annotation is the critical addition. Every command that is registered in `collect_commands![]` must have it.

---

### Rust: AppState Mutex Access
**Source:** `/Users/albair/src/wallflower/crates/wallflower-app/src/commands/status.rs` lines 17-18
**Apply to:** All command files

```rust
let guard = state.some_field.lock().map_err(|e| AppError::Device(e.to_string()))?;
```

Always use `.map_err(|e| AppError::SomeVariant(e.to_string()))?` — never `.unwrap()` on Mutex locks in command handlers.

---

### Rust: tracing for logging
**Source:** `/Users/albair/src/wallflower/crates/wallflower-core/src/db/mod.rs` lines 85, 128, 165
**Apply to:** All Rust modules

```rust
tracing::info!("Opening database at: {}", path.display());
tracing::warn!("Something unexpected: {e}");
tracing::error!("Fatal: {e}");
```

No `println!` or `eprintln!` in library code. All logging goes through `tracing`.

---

### Rust: Background task spawn
**Source:** `/Users/albair/src/wallflower/crates/wallflower-app/src/lib.rs` lines 322-324
**Apply to:** `crates/takoyaki-app/src/lib.rs` and `src/device/mod.rs`

```rust
tauri::async_runtime::spawn(async move {
    some_async_task(app_handle).await;
});
```

Use `tauri::async_runtime::spawn` not `tokio::spawn` directly — Tauri manages its own runtime.

---

### TypeScript: Tauri event listen pattern
**Source:** `/Users/albair/src/wallflower/src/components/tauri-event-listener.tsx` lines 84-100, 322-326
**Apply to:** `src/components/tauri-event-listener.tsx`

```typescript
const cleanupFns: (() => void)[] = [];
// Inside useEffect:
const { listen } = await import("@tauri-apps/api/event");
const unlisten = await listen<PayloadType>("event-name", (event) => { ... });
cleanupFns.push(unlisten);
// Cleanup:
return () => { for (const cleanup of cleanupFns) cleanup(); };
```

---

### TypeScript: React Query integration
**Source:** `/Users/albair/src/wallflower/src/components/tauri-event-listener.tsx` lines 259, 274
**Apply to:** Any component that fetches data from Tauri commands

```typescript
import { useQueryClient } from "@tanstack/react-query";
const queryClient = useQueryClient();
// After mutation:
queryClient.invalidateQueries({ queryKey: ["projects"] });
```

---

### TypeScript: Zustand store pattern
**Source:** `/Users/albair/src/wallflower/src/lib/stores/library.ts` lines 23-40
**Apply to:** All stores in `src/lib/stores/`

```typescript
import { create } from "zustand";

interface SomeState {
  field: string | null;
  setField: (val: string | null) => void;
}

export const useSomeStore = create<SomeState>((set) => ({
  field: null,
  setField: (val) => set({ field: val }),
}));
```

---

### CSS: Tailwind v4 shadcn token mapping
**Source:** `/Users/albair/src/wallflower/src/app/globals.css` lines 1-33
**Apply to:** `src/app/globals.css`

The `@theme inline { ... }` block maps shadcn CSS variable names to Tailwind color utilities. Copy the full block from Wallflower lines 5-33, then override `:root` values for the warm dark palette.

---

## No Analog Found

All files have analogs from the Wallflower sister project. The following files have no direct line-for-line analog but use RESEARCH.md patterns instead:

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `crates/ot-parser/src/sample.rs` | utility | transform | Binary parsing with binrw is new — no Wallflower analog. Use RESEARCH.md Pattern 1 + Pattern 2 |
| `crates/ot-parser/src/project.rs` | utility | transform | Same as above |
| `crates/ot-parser/src/bank.rs` | utility | transform | Same as above |
| `crates/ot-parser/src/markers.rs` | utility | transform | Same as above |
| `crates/ot-parser/src/arrangement.rs` | utility | transform | Same as above |
| `crates/takoyaki-app/src/atomic/mod.rs` | service | file-I/O | Wallflower has no atomic write engine — use RESEARCH.md Pattern 3 |
| `tests/fixtures/` (binary files) | test | — | Binary test fixtures — generate synthetically per D-09/D-10 |

---

## Metadata

**Analog search scope:** `/Users/albair/src/wallflower/crates/` and `/Users/albair/src/wallflower/src/`
**Files scanned:** ~30 source files across Wallflower codebase
**Pattern extraction date:** 2026-04-29

**Critical version notes:**
- Wallflower uses `edition = "2024"` successfully in both crates — Takoyaki should follow (assumption A3 in RESEARCH.md is likely resolved)
- Wallflower does NOT use `tauri-specta` — all type bindings are manual in `src/lib/types.ts`. Takoyaki adds tauri-specta; follow RESEARCH.md Pattern 5 for the builder setup, not Wallflower's `generate_handler![]` pattern
- Wallflower's `tauri.conf.json` uses `"$schema"` pointing to a non-standard URL — omit that field or use the official Tauri schema URL
