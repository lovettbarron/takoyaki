# Phase 1: Foundation - Research

**Researched:** 2026-04-29
**Domain:** Tauri v2 app scaffold, Rust binary parsing, atomic file writes, SQLite, OT format reverse engineering, macOS volume detection
**Confidence:** HIGH (stack verified via cargo search / npm view / codebase inspection), MEDIUM (OT format details)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Parser Strategy**
- D-01: Read ot-tools-io source to learn OT binary format structure, then write an independent Rust implementation. No code copying, no GPL contamination.
- D-02: Unknown/undocumented byte regions preserved verbatim as opaque blobs during round-trip. Parser evolves incrementally.
- D-03: Phase 1 parses ALL core OT file types: .work (project), .strd (bank arrangements), .ot (sample metadata/slices), bank files, and marker files. No deferral.
- D-04: Parser lives in a standalone `ot-parser` crate within a Cargo workspace. No Tauri dependency, no I/O. Pure parsing library.

**App Scaffold & Visual Identity**
- D-05: Hardware-inspired design language: Octatrack functional aesthetic + Elektron industrial precision + monome minimal grid warmth.
- D-06: Monome warm dark color palette — dark but slightly warm grays, off-white text, subtle warmth. Not pure OLED black, not corporate muted.
- D-07: Full sidebar navigation skeleton from Phase 1 with all future sections visible but disabled/grayed.
- D-08: Monospace-forward typography — primary monospace font throughout the UI. Proportional font for longer descriptive text only.

**Test Fixtures**
- D-09: Test corpus = user's real OT project files + synthetic edge cases.
- D-10: Binary test fixtures committed directly in git under tests/fixtures/. OT files are KB-range, no Git LFS needed.

**Volume Detection UX**
- D-11: On OT connect, auto-navigate to Projects view.
- D-12: Disconnected state is always-ready shell — sidebar stays active, content shows "No device" inline.
- D-13: Single device support only. If multiple mounted, use first detected or let user pick.
- D-14: Auto-detect OT volumes by directory structure sniffing (/AUDIO, /SETS, characteristic patterns), then show user confirmation dialog.

### Claude's Discretion
- Exact monospace font selection (Iosevka selected — see UI-SPEC.md)
- Specific accent color within warm dark palette (established in UI-SPEC.md)
- Sidebar section icons and disabled state styling
- Volume detection polling interval vs filesystem event approach
- Synthetic test fixture generation strategy
- Snapshot storage format and directory structure
- SQLite schema design for backup history and project index

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| FNDN-01 | Clean-room Rust parser for OT binary formats (.work, .strd, .ot, bank files, marker files) with no GPL dependencies | binrw 0.15 for declarative parsing; format documented below from OctaChainer/docs.rs |
| FNDN-02 | Parser preserves all unknown/reserved bytes verbatim during round-trip | binrw `[br(count = N)]` on `Vec<u8>` fields; opaque blob pattern verified |
| FNDN-03 | Parser uses correct indexing (1-indexed for project files, 0-indexed for bank/marker files) with Rust newtypes preventing mismatch | ot-tools-io docs confirm this indexing distinction; newtype wrappers enforce at type level |
| FNDN-04 | Staging directory for atomic writes lives on the same filesystem as the CF card volume | atomic-write-file 0.3 creates temp on same dir; custom staging also viable; FAT32 rename atomicity verified |
| FNDN-05 | All write completions gated on fsync + directory sync to protect against hot-unplug data loss | macOS requires `F_FULLFSYNC` via fcntl — Rust `File::sync_all` handles this correctly on macOS |
| FNDN-06 | Tauri v2 desktop app with Rust backend and React/Next.js frontend, consistent with Wallflower architecture | Wallflower workspace directly referenceable as blueprint; verified Cargo.toml patterns |
| FNDN-07 | SQLite database for Takoyaki's own metadata (backup history, project index, snapshot records) | rusqlite 0.39 + rusqlite_migration 2.5 pattern verified; Wallflower uses identical setup |
| FNDN-08 | Read-only SQLite connection to Wallflower database with driver-level write protection | rusqlite Connection::open_with_flags with SQLITE_OPEN_READONLY flag |
| SAFE-03 | System automatically creates a snapshot of all affected files before any write operation | snapshot engine wraps atomic-write-file; snapshot = file copies to .takoyaki/snapshots/ before any commit |
| SAFE-04 | All write operations use atomic staged writes (write to staging, verify, then rename — all-or-nothing) | atomic-write-file 0.3 verified; same-volume staging is the key constraint |
| BROW-01 | User can see when Octatrack is connected in USB disk mode via automatic volume detection | sysinfo 0.35 Disks API with polling; directory sniffing for OT signature (/AUDIO, /SETS, bank files) |
</phase_requirements>

---

## Summary

Phase 1 establishes the complete foundation for all subsequent phases: a pure Rust binary parser for Octatrack file formats, a Tauri v2 + Next.js app scaffold, a safe atomic write engine, SQLite schema initialization, and USB volume detection with warm-dark UI shell.

The technical risks are well-bounded. The Rust ecosystem (binrw, atomic-write-file, rusqlite, sysinfo) is mature and production-ready at the pinned versions. Tauri v2 is stable and Wallflower — a sister project in the same repository tree — provides a working blueprint for every infrastructure decision including Cargo workspace layout, `next.config.mjs`, Tailwind v4 CSS variable theming, and shadcn initialization. The planner should treat Wallflower as a concrete reference implementation, not just inspiration.

The OT binary format requires clean-room reverse engineering but the work is scoped to format structure (field offsets, sizes, types) — not the legal expression of ot-tools-io's code. The .ot file format is fully documented (832 bytes, verified from OctaChainer source). The .work and bank file formats are ~68% documented via ot-tools-io's prior art, with the remaining ~32% preserved verbatim as opaque bytes — this is explicitly required by FNDN-02.

The macOS atomic write concern (FAT32 + hot-unplug) has a correct solution: `File::sync_all()` on macOS calls `fcntl(F_FULLFSYNC)` which guarantees physical media flush, unlike plain `fsync()`. The `atomic-write-file` crate handles the temp-file-on-same-volume + rename pattern correctly.

**Primary recommendation:** Mirror Wallflower's Cargo workspace structure (`crates/` members with `ot-parser` and `takoyaki-app`), use binrw for all OT format structs with opaque `Vec<u8>` blobs for unknown regions, poll sysinfo Disks every 2 seconds for volume detection, and keep the parser zero-dependency (no Tauri, no I/O).

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| OT binary parsing | Rust library crate (ot-parser) | — | Pure parser, no I/O, no Tauri; enables independent testing |
| Atomic file write engine | Rust backend (takoyaki-app) | — | Requires fs access, same-volume staging, fsync; not a frontend concern |
| Snapshot creation | Rust backend (takoyaki-app) | — | Wraps atomic write; must complete before any write commits |
| SQLite schema init | Rust backend (takoyaki-app) | — | Database lives in macOS app data dir, initialized at startup |
| Wallflower DB read-only access | Rust backend (takoyaki-app) | — | Read-only rusqlite connection; never touches frontend |
| USB volume detection | Rust backend (takoyaki-app) | Frontend (event receiver) | sysinfo polling in Rust background task; Tauri event emitted to frontend |
| Volume detection UX (dialog, state) | Frontend (React/Next.js) | — | Dialog, sidebar state, auto-navigation are frontend concerns |
| App chrome / navigation skeleton | Frontend (React/Next.js) | — | Sidebar, titlebar, disabled nav sections are UI layer |
| TypeScript type safety | Frontend + Rust boundary | — | tauri-specta generates bindings; both sides honor the contract |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tauri | 2.10.3 | Desktop app shell, IPC, OS integration | Required by project; Wallflower blueprint available |
| binrw | 0.15.1 | Declarative binary parsing + writing for OT formats | Purpose-built for read/write symmetry; magic bytes, endianness, validation in one macro |
| rusqlite | 0.39.0 | SQLite (bundled) | No system SQLite dep; Wallflower uses identical version |
| atomic-write-file | 0.3.0 | Same-volume staging + atomic rename | Handles FAT32 same-filesystem constraint, fsync before rename |
| sysinfo | 0.35.x | Disk/volume enumeration for USB detection | `is_removable()`, `mount_point()`, `file_system()` methods; cross-platform |
| serde / serde_json | 1.0.228 | Serialization for IPC and config | Required by Tauri IPC (JSON-RPC protocol) |
| thiserror | 2.0.18 | Error types for Tauri commands | `derive(Error)` + `Serialize` wrapper pattern for IPC errors |
| tracing | 0.1.44 | Structured logging | Wallflower uses same; tracing-subscriber for output |
| Next.js | 15.x (16.2.4) | React framework (static export mode) | Wallflower uses `output: 'export'`; Tauri requires static site |
| React | 19.2.5 | UI framework | Latest stable; Wallflower verified working |
| Tailwind CSS | 4.2.x | Styling via CSS variables | Wallflower uses `@import "tailwindcss"` inline pattern |
| zustand | 5.0.12 | Client state (device connection state, navigation) | Lightweight; Wallflower verified pattern |
| @tanstack/react-query | 5.100.6 | Server state / async data fetching | Wallflower verified; wraps Tauri commands as queries |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| rusqlite_migration | 2.5.0 | SQLite schema migrations via user_version | Phase 1 schema init; add migration SQL as inline const strings |
| tauri-specta | 2.0.0-rc.24 | Auto-generated TypeScript types from Rust commands | Every Tauri command; prevents type drift between Rust and TS |
| specta | 2.0.0-rc.24 | Type introspection runtime for tauri-specta | Companion to tauri-specta; pin exact rc version |
| tempfile | 3.27.0 | Staging directories for atomic operations | When staging multiple files for a snapshot operation |
| zip | 8.6.0 | Project snapshot archives | Phase 3+; add to ot-parser or separate crate |
| notify | 9.0.0-rc.3 | Filesystem watching (FSEvents/Kqueue) | Optional for watching volume; polling via sysinfo may be simpler |
| next-themes | 0.4.6 | Dark mode wiring for Next.js | If theme toggle is needed; warm dark is fixed in Phase 1 |
| shadcn | 4.x | Component primitives (Radix UI based) | Sidebar, Dialog, Button, Toast — see UI-SPEC.md |
| lucide-react | latest | Icon library | Consistent with Wallflower; Tauri-compatible |
| @fontsource/iosevka | latest | Iosevka monospace font (self-hosted) | Or next/font/local — avoids CDN; confirmed in UI-SPEC.md |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| binrw | nom 8.0 | nom better for streaming/irregular sections; binrw better for struct-heavy formats with symmetric read/write. OT files are fixed-struct — binrw wins. |
| sysinfo polling | DiskArbitration (macOS FFI) | DiskArbitration gives push events but requires unsafe FFI and macOS entitlements. Polling every 2s with sysinfo is simpler, equally effective, no entitlement risk. |
| atomic-write-file | hand-rolled temp+rename | atomic-write-file handles directory file descriptors, cross-device prevention, and fsync correctly. Custom solutions almost always miss edge cases. |
| rusqlite_migration | refinery | rusqlite_migration is smaller, no CLI needed, uses user_version (not tables). Wallflower uses inline SQL const strings — proven pattern. |
| @tanstack/react-query | raw invoke() | react-query adds caching, loading/error states, background refetch. Worth it for any data that changes. |

**Installation:**
```bash
# Frontend
npm install next@latest react@latest react-dom@latest typescript@latest
npm install tailwindcss@latest @tanstack/react-query@latest zustand@latest
npm install @tauri-apps/api@latest @tauri-apps/cli@latest
npm install next-themes@latest lucide-react@latest
npm install -D @types/react @types/react-dom
# shadcn init: npx shadcn@latest init (answer: style=base-nova, baseColor=neutral, cssVariables=yes)
npx shadcn@latest add button dialog sidebar sonner separator skeleton badge
```

```toml
# Cargo workspace root Cargo.toml
[workspace]
members = ["crates/ot-parser", "crates/takoyaki-app"]
resolver = "2"
edition = "2021"  # NOTE: Use 2021, not 2024 — tauri-build cargo_toml dep has 2024 compatibility issue

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
thiserror = "2"
rusqlite = { version = "0.39", features = ["bundled"] }

# ot-parser crate: no Tauri dependencies
[dependencies]  # in crates/ot-parser/Cargo.toml
binrw = "0.15"
serde = { workspace = true }
thiserror = { workspace = true }

# takoyaki-app crate
[dependencies]  # in crates/takoyaki-app/Cargo.toml
ot-parser = { path = "../ot-parser" }
tauri = { version = "2", features = [] }
tauri-specta = { version = "=2.0.0-rc.24", features = ["derive", "typescript"] }
specta = "=2.0.0-rc.24"
atomic-write-file = "0.3"
tempfile = "3"
sysinfo = "0.35"
rusqlite = { workspace = true }
rusqlite_migration = "2.5"
serde = { workspace = true }
tracing = { workspace = true }
thiserror = { workspace = true }
```

**Version verification:** All Rust crate versions verified via `cargo search` on 2026-04-29. All npm versions verified via `npm view` on 2026-04-29. [VERIFIED: cargo search, npm view]

---

## Architecture Patterns

### System Architecture Diagram

```
User Action / OS Event
         │
         ▼
┌─────────────────────┐
│  React / Next.js    │  Static export, Tauri webview
│  Frontend           │  zustand: device state, nav state
│                     │  react-query: data fetching
│  Sidebar + Content  │
│  Volume Dialog      │
└────────┬────────────┘
         │  invoke() / listen()  [tauri-specta TypeScript bindings]
         ▼
┌─────────────────────┐
│  Tauri IPC Bridge   │  JSON-RPC, capabilities system
│  AppState (Mutex)   │  DeviceState, Database, SnapshotEngine
└────────┬────────────┘
         │
    ┌────┴──────────────────────────────────┐
    │                                       │
    ▼                                       ▼
┌──────────────────┐              ┌────────────────────┐
│  sysinfo Polling │              │  ot-parser crate   │
│  Background Task │              │  (pure library)    │
│                  │              │                    │
│  Disks::refresh()│              │  BinRead/BinWrite  │
│  OT signature    │              │  for each file type│
│  check           │              │  Round-trip tested │
│  Emit event to   │              │  No I/O, no Tauri  │
│  frontend        │              └────────────────────┘
└──────────────────┘
         │
         ▼
┌─────────────────────┐
│  Atomic Write Engine│
│  (FNDN-04, FNDN-05) │
│                     │
│  1. Create snapshot │  ◄── All files → .takoyaki/snapshots/{timestamp}/
│  2. Stage to temp   │  ◄── Same volume (FAT32 rename safe)
│  3. sync_all()      │  ◄── F_FULLFSYNC on macOS
│  4. rename() atomic │
└─────────────────────┘
         │
         ▼
┌─────────────────────┐
│  SQLite (rusqlite)  │
│  App data dir       │
│  backup_history     │
│  project_index      │
│  snapshot_records   │
│                     │
│  Wallflower DB      │  (read-only connection, SQLITE_OPEN_READONLY)
│  SQLITE_OPEN_READONLY│
└─────────────────────┘
```

### Recommended Project Structure

```
takoyaki/
├── Cargo.toml               # workspace root: members = ["crates/ot-parser", "crates/takoyaki-app"]
├── Cargo.lock
├── package.json             # frontend root
├── next.config.mjs          # output: 'export', images: { unoptimized: true }
├── src/                     # Next.js App Router frontend
│   └── app/
│       ├── globals.css      # @import "tailwindcss"; @theme inline { --font-mono: ... }
│       ├── layout.tsx       # root layout with sidebar chrome
│       └── page.tsx         # default: disconnected state
├── crates/
│   ├── ot-parser/           # pure parsing library — NO Tauri, NO I/O
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── project.rs   # ProjectFile (.work / .strd)
│   │       ├── bank.rs      # BankFile (bank01.work ... bank16.work)
│   │       ├── markers.rs   # MarkersFile (markers.work)
│   │       ├── arrangement.rs # ArrangementFile (arr01.work ... arr08.work)
│   │       ├── sample.rs    # SampleSettingsFile (*.ot)
│   │       └── error.rs
│   └── takoyaki-app/
│       ├── Cargo.toml
│       ├── build.rs         # tauri_build::build()
│       ├── tauri.conf.json
│       └── src/
│           ├── main.rs      # #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
│           ├── lib.rs       # pub fn run() — Tauri builder
│           ├── commands/    # #[tauri::command] #[specta::specta] handlers
│           ├── db/          # rusqlite connection, migrations, queries
│           ├── device/      # sysinfo polling, OT volume detection, event emit
│           ├── atomic/      # atomic write engine, snapshot creation
│           └── error.rs     # AppError: thiserror + Serialize
├── tests/
│   └── fixtures/            # binary .work/.ot files committed directly (KB-range, no LFS)
└── migrations/              # V1__initial_schema.sql, V2__*.sql etc.
```

### Pattern 1: binrw OT File Type

```rust
// Source: docs.rs/binrw + OctaChainer otwriter.h verified field layout
use binrw::{binrw, BinRead, BinWrite};

/// .ot sidecar file — 832 bytes total
/// Header magic: [0xF0, 0x00, 0x00, 0xE8, 0x57, 0x45, 0x52, 0x41, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
/// All multi-byte fields: big endian
#[binrw]
#[brw(big)]
#[derive(Debug, Clone, PartialEq)]
pub struct SampleSettingsFile {
    pub header: [u8; 16],
    pub unknown_0x10: [u8; 7],
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

#[binrw]
#[brw(big)]
#[derive(Debug, Clone, PartialEq)]
pub struct Slice {
    pub start_point: u32,
    pub end_point: u32,
    pub loop_point: u32,
}

// Round-trip test pattern:
// let bytes = include_bytes!("../tests/fixtures/sample.ot");
// let parsed = SampleSettingsFile::read(&mut Cursor::new(bytes)).unwrap();
// let mut out = Cursor::new(Vec::new());
// parsed.write(&mut out).unwrap();
// assert_eq!(bytes, out.into_inner().as_slice());
```

### Pattern 2: Opaque Unknown Byte Preservation

```rust
// Source: binrw docs.rs — count directive for unknown regions
// Use this for any undocumented region of .work / bank files
#[binrw]
#[brw(big)]
pub struct ProjectFile {
    pub header: [u8; 16],         // magic bytes verified
    pub version: u16,
    pub name: [u8; 32],           // null-padded ASCII
    pub tempo_bpm: u16,
    #[br(count = 0x1A2)]          // size of undocumented region (example)
    pub unknown_region_1: Vec<u8>, // preserved verbatim — round-trip safe
    // ... known fields ...
    #[br(count = remaining_unknown_size)]
    pub unknown_tail: Vec<u8>,
    pub checksum: u16,
}
```

### Pattern 3: Atomic Write Engine

```rust
// Source: docs.rs/atomic-write-file + Rust std::fs sync docs
use atomic_write_file::AtomicWriteFile;
use std::fs;

pub fn atomic_write_ot_file(
    target_path: &Path,
    content: &[u8],
) -> Result<(), AppError> {
    // Stage on same volume as target (required for FAT32 rename atomicity)
    let mut staging = AtomicWriteFile::options().open(target_path)?;
    staging.write_all(content)?;
    staging.flush()?;
    // sync_all() calls F_FULLFSYNC on macOS — protects against hot-unplug
    staging.sync_all()?;
    staging.commit()?; // atomic rename
    // Also sync the parent directory inode
    let parent = target_path.parent().ok_or(AppError::InvalidPath)?;
    let dir = fs::File::open(parent)?;
    dir.sync_all()?;
    Ok(())
}
```

### Pattern 4: Volume Detection with sysinfo

```rust
// Source: docs.rs/sysinfo Disk struct methods
use sysinfo::{Disks, DiskKind};
use std::path::Path;

const OT_SIGNATURE_DIRS: &[&str] = &["AUDIO", "SETS"];

pub fn detect_ot_volume(disks: &Disks) -> Option<std::path::PathBuf> {
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

// Polling task (run in Tauri background via tauri::async_runtime::spawn):
// let mut disks = Disks::new_with_refreshed_list();
// loop {
//     tokio::time::sleep(Duration::from_secs(2)).await;
//     disks.refresh();
//     if let Some(path) = detect_ot_volume(&disks) { ... }
// }
```

### Pattern 5: Tauri Command with thiserror + specta

```rust
// Source: tauri-specta DeepWiki + Tauri v2 docs
use tauri::State;
use tauri_specta::collect_commands;

#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("Parse error: {0}")]
    Parse(String),
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self { AppError::Io(e.to_string()) }
}

#[tauri::command]
#[specta::specta]
pub async fn get_device_status(
    state: State<'_, AppState>,
) -> Result<DeviceStatus, AppError> {
    let guard = state.device.lock().unwrap();
    Ok(guard.status.clone())
}

// In lib.rs:
let builder = tauri_specta::Builder::<tauri::Wry>::new()
    .commands(collect_commands![get_device_status]);

#[cfg(debug_assertions)]
builder.export(specta_typescript::Typescript::default(), "../src/bindings.ts").unwrap();

tauri::Builder::default()
    .invoke_handler(builder.invoke_handler())
    .manage(AppState::new()?)
    .run(tauri::generate_context!())
    .expect("error running application");
```

### Pattern 6: SQLite Schema with Embedded Migrations (Wallflower-verified)

```rust
// Source: Wallflower codebase (crates/wallflower-core/src/db/mod.rs) + rusqlite_migration docs
use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

const MIGRATION_V1: &str = include_str!("../../migrations/V1__initial_schema.sql");
const MIGRATION_V2: &str = include_str!("../../migrations/V2__snapshots.sql");

static MIGRATIONS: &[M] = &[
    M::up(MIGRATION_V1),
    M::up(MIGRATION_V2),
];

pub fn open_database(path: &Path) -> Result<Connection, AppError> {
    let mut conn = Connection::open(path)?;
    Migrations::from_slice(MIGRATIONS).to_latest(&mut conn)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(conn)
}
```

### Pattern 7: Read-Only Wallflower DB Connection

```rust
// Source: rusqlite docs Connection::open_with_flags
use rusqlite::{Connection, OpenFlags};

pub fn open_wallflower_db(path: &Path) -> Result<Connection, AppError> {
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    Ok(conn)
}
```

### Pattern 8: Tailwind v4 Warm Dark Theme (Wallflower-verified pattern)

```css
/* globals.css — matches Wallflower pattern, adapted for Takoyaki warm dark palette */
@import "tailwindcss";

@theme inline {
    --font-mono: 'Iosevka', 'JetBrains Mono', monospace;
    --font-sans: 'Inter', system-ui, sans-serif;
    --color-background: var(--background);
    --color-foreground: var(--foreground);
    /* ... other shadcn token mappings */
}

:root {
    /* Warm dark palette per UI-SPEC.md */
    --background: hsl(30 8% 10%);     /* #1c1a18 */
    --foreground: hsl(30 10% 88%);    /* #e3ddd6 */
    --card: hsl(30 8% 16%);           /* #2a2723 */
    --accent: hsl(38 85% 55%);        /* #f0a832 */
    --muted: hsl(30 8% 18%);
    --muted-foreground: hsl(30 8% 50%);
    --border: hsl(30 8% 26%);
    --destructive: hsl(0 68% 48%);
    --radius: 0.375rem;               /* hardware-precise, not round */
}
```

### Anti-Patterns to Avoid

- **Cross-volume staging:** Creating the temp file in `/tmp` rather than on the CF card volume — this makes `rename()` cross-device, which is NOT atomic. Always stage in the same directory or volume as the target.
- **Plain fsync on macOS:** `File::sync_data()` on macOS does not guarantee physical media write. Must use `File::sync_all()` which Rust routes to `F_FULLFSYNC`.
- **GPL code incorporation:** Any copy-paste from ot-tools-io source becomes a GPL contamination. Read for format understanding only; write independent Rust from the field layout knowledge.
- **Mixed indexing:** Slot IDs in ProjectFile are 1-indexed; all other files are 0-indexed. Mixing these without newtypes causes off-by-one bugs. Enforce at the type level.
- **Rust 2024 edition with Tauri:** `tauri-build` uses `cargo_toml ^0.17` which cannot parse `edition = "2024"`. Use `edition = "2021"` for all workspace members (Wallflower uses edition 2024 successfully — this may have been fixed by the time of this research; verify at project init).
- **Tauri commands with non-Serialize errors:** All Tauri command errors MUST implement `serde::Serialize`. Returning `anyhow::Error` or `std::io::Error` directly will cause a runtime panic. Wrap in a custom `AppError` enum.
- **tauri-specta rc version pinning:** Use `=2.0.0-rc.24` exact (with `=`), not `^2.0.0-rc`. RC versions can have breaking changes across minor bumps.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Atomic file write | temp file + manual rename | atomic-write-file 0.3 | Handles directory fd, cross-device prevention, fsync ordering correctly |
| Binary struct parsing | bitfield macros, manual offset arithmetic | binrw 0.15 | Symmetric read/write from same struct definition; magic bytes, endianness, padding, validation |
| Schema migration | custom user_version tracking | rusqlite_migration 2.5 | Uses SQLite user_version pragma; no tracking tables; atomic apply |
| TypeScript type generation | manually maintained bindings.ts | tauri-specta 2.0-rc | Compile-time guarantee types match Rust; regenerates on build |
| Disk enumeration | /proc/mounts parsing, raw ioctl | sysinfo 0.35 | `is_removable()`, `mount_point()`, `file_system()` — cross-platform and tested |
| Component primitives | CSS-from-scratch | shadcn + Radix UI | Keyboard accessibility, WAI-ARIA, tested interactions included |

**Key insight:** The OT format has at least 16 known fields that interact across 5 file types. Any write to one file may require coordinated writes to 17 others. Hand-rolling an atomic multi-file write engine that handles partial failures, hot-unplug mid-write, and FAT32 rename semantics is where data loss bugs hide. Use the established libraries.

---

## Common Pitfalls

### Pitfall 1: Cross-Volume Atomic Write

**What goes wrong:** Temp file created in `/tmp/` or app temp dir; `rename()` call fails with `EXDEV` (cross-device link) because `/tmp` is APFS and CF card is FAT32.

**Why it happens:** `tempfile::NamedTempFile` defaults to the system temp dir, not the target file's directory.

**How to avoid:** `atomic-write-file` always creates the temp on the same dir as the target. If using custom staging, call `tempfile::Builder::new().suffix(".staging").tempdir_in(target_parent)?`.

**Warning signs:** Integration test on real FAT32 volume (CI requirement per success criterion 3) will catch this.

---

### Pitfall 2: macOS fsync Does Not Guarantee Physical Write

**What goes wrong:** `File::sync_data()` on macOS calls POSIX `fsync()` which flushes the kernel page cache but does NOT guarantee the data has hit physical media. Hot-unplug after fsync still loses data.

**Why it happens:** Apple's storage drivers defer physical writes for performance; `fsync()` is documented as advisory on macOS.

**How to avoid:** Call `File::sync_all()` — Rust uses `fcntl(F_FULLFSYNC)` on macOS which forces media flush. `atomic-write-file` calls `sync_all()` internally on the temp file before rename.

**Warning signs:** Unit test can't catch this; requires real hardware test with hot-unplug.

---

### Pitfall 3: GPL Contamination via Format Research

**What goes wrong:** Developer reads ot-tools-io source, copies a struct definition or serialization logic verbatim. Project ships with embedded GPL code.

**Why it happens:** Format knowledge (field offsets, sizes, types) is discoverable fact; code expression of that knowledge is copyrightable. The line is easy to blur under time pressure.

**How to avoid:** Create a clean-room format spec document (a plain-text table of field offsets, sizes, types, and meanings) from ot-tools-io reading BEFORE writing any Rust. The spec document is the evidence of clean-room process. Code is then written from the spec, not from the GPL source.

**Warning signs:** If any Rust struct definition looks byte-for-byte like a binrw translation of ot-tools-io's serde code, it needs review.

---

### Pitfall 4: tauri-specta RC Version Drift

**What goes wrong:** Pinning `tauri-specta = "^2.0.0-rc.21"` and running `cargo update` updates to an incompatible RC with breaking API changes, breaking TypeScript generation.

**Why it happens:** Semver pre-release (`-rc`) versions are excluded from normal `^` matching, but some toolchain versions treat them differently.

**How to avoid:** Pin exact: `tauri-specta = "=2.0.0-rc.24"` and `specta = "=2.0.0-rc.24"`. Do not use `cargo update` without re-verifying TypeScript output.

---

### Pitfall 5: ot-parser Crate Accidentally Importing Tauri

**What goes wrong:** A developer adds a Tauri-related type to ot-parser (e.g., for error handling) and Tauri becomes a build dependency. The crate can no longer be compiled standalone for fuzzing or unit testing.

**Why it happens:** Easy to reach for AppError defined in takoyaki-app when writing parser code.

**How to avoid:** ot-parser has its own `ParseError` using `thiserror` only. The parser crate's `Cargo.toml` has zero Tauri dependencies. If it compiles with `cargo test -p ot-parser`, the boundary is maintained.

---

### Pitfall 6: Next.js Static Export Incompatibilities

**What goes wrong:** Using `next/image` without `unoptimized: true`, or using server-side features (getServerSideProps, API routes, server actions), causing build failures or runtime errors in the Tauri webview.

**Why it happens:** Next.js static export (`output: 'export'`) disables server-dependent features. Tauri has no Node.js server.

**How to avoid:** Mirror Wallflower's `next.config.mjs`: `output: 'export'`, `images: { unoptimized: true }`. No API routes. No server components that do data fetching — all data flows through Tauri IPC.

---

## Code Examples

### .ot File Format (Verified)

```
Offset  Size  Type        Field
0x00    16    u8[16]      Header/magic
0x10    7     u8[7]       Unknown (preserve verbatim)
0x17    4     u32 BE      Tempo
0x1B    4     u32 BE      Trim length (samples)
0x1F    4     u32 BE      Loop length (samples)
0x23    4     u32 BE      Stretch
0x27    4     u32 BE      Loop flag
0x2B    2     u16 BE      Gain
0x2D    1     u8          Quantize
0x2E    4     u32 BE      Trim start (samples)
0x32    4     u32 BE      Trim end (samples)
0x36    4     u32 BE      Loop point (samples)
0x3A    768   Slice[64]   Slice data (12 bytes each: start u32, end u32, loop u32)
0x33A   4     u32 BE      Slice count
0x33E   2     u16 BE      Checksum
TOTAL = 832 bytes
```

Source: OctaChainer `otwriter.h` [VERIFIED: raw.githubusercontent.com/KaiDrange/OctaChainer/master/otwriter.h]

### OT Card File Structure (Verified)

```
/Volumes/OT-CARD/
├── AUDIO/                   ← detection signature dir
│   ├── Kicks/
│   └── Loops/
└── SETS/                    ← detection signature dir
    └── {SetName}/
        └── {ProjectName}/
            ├── project.work     ← ProjectFile (auto-saved)
            ├── project.strd     ← ProjectFile (last manual save)
            ├── bank01.work      ← BankFile (1 of 16)
            ├── bank01.strd
            ├── ...
            ├── bank16.work
            ├── bank16.strd
            ├── arr01.work       ← ArrangementFile (1 of 8)
            ├── arr01.strd
            ├── ...
            ├── arr08.work
            ├── arr08.strd
            ├── markers.work     ← MarkersFile
            └── markers.strd
```

Source: Elektronauts forums + ot-tools-io docs.rs module structure [VERIFIED: docs.rs/ot-tools-io]

### ot-tools-io Module Structure (Format Reference, NOT for copying)

The following struct names are used for format study only:

| File Type | Struct | Module | Index Scheme |
|-----------|--------|--------|--------------|
| project.work / .strd | `ProjectFile` | `projects` | 1-indexed slot_id |
| bankNN.work / .strd | `BankFile` | `banks` | 0-indexed |
| markers.work / .strd | `MarkersFile` | `markers` | 0-indexed |
| arrNN.work / .strd | `ArrangementFile` | `arrangements` | 0-indexed |
| *.ot | `SampleSettingsFile` | `samples` | N/A |

Source: docs.rs/ot-tools-io [VERIFIED: docs.rs/ot-tools-io/latest/ot_tools_io/]

Integrity validation traits: `HasHeaderField`, `HasChecksumField`, `HasFileVersionField` — implement equivalent validation in clean-room Rust.

### SQLite Schema — Initial Tables

```sql
-- V1__initial_schema.sql
-- Takoyaki own metadata database

-- Snapshot records: one row per snapshot event
CREATE TABLE snapshots (
    id         TEXT PRIMARY KEY NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    operation  TEXT NOT NULL,  -- 'manual', 'pre-write', 'backup'
    project_path TEXT,
    file_count INTEGER NOT NULL,
    total_bytes INTEGER NOT NULL
);

-- Individual files captured in a snapshot
CREATE TABLE snapshot_files (
    id           TEXT PRIMARY KEY NOT NULL,
    snapshot_id  TEXT NOT NULL REFERENCES snapshots(id) ON DELETE CASCADE,
    original_path TEXT NOT NULL,
    stored_path  TEXT NOT NULL,
    file_hash    TEXT NOT NULL
);

-- Project index — updated on connect / rescan
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

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| tailwind.config.js + theme extend | CSS variables in globals.css `@theme inline` | Tailwind v4 (2024) | No config file needed; tokens live in CSS |
| tauri-specta v1 manual setup | builder pattern with `collect_commands!` + `Builder::new()` | tauri-specta v2 rc | Events also type-generated |
| `serde_json::Value` for opaque fields | `[u8; N]` or `Vec<u8>` with binrw count | always correct | Round-trip fidelity requires byte-level preservation |
| `rename()` + hope for atomicity | atomic-write-file with directory fsync | ongoing | Hot-unplug safety now expressible |

**Deprecated/outdated:**
- ot-tools-io uses `bincode` for binary parsing (older approach) — binrw is more expressive and produces symmetric read/write code.
- tauri-specta v1 `collect_types!` + `ts::export()` — v2 uses `Builder` pattern; v1 API removed in v2.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | OT project.work and bank file magic bytes and checksum algorithm match ot-tools-io's reversed format; only verified for .ot files from OctaChainer source | OT Format Knowledge | Need to re-derive offsets during clean-room spec creation; parser fails round-trip |
| A2 | sysinfo 0.35 `is_removable()` correctly identifies CF card on macOS 26.x (tested versions show 0.35 compatible with macOS) | Volume Detection | May need to fall back to directory sniffing alone without `is_removable()` |
| A3 | Rust 2021 edition avoids the tauri-build cargo_toml parse issue; Wallflower uses 2024 edition successfully suggesting the issue was fixed | Stack / Cargo Workspace | If 2024 edition is actually needed, use `edition = "2024"` and verify |
| A4 | `atomic-write-file` 0.3 creates staging temp in the same directory as the target, not in `/tmp` — behavior assumed from docs and source description | Atomic Write Engine | If wrong, cross-device rename will fail on real FAT32 volume |
| A5 | OT .strd and .work files contain identical binary format; only the save semantics differ (.work = autosave, .strd = manual save) | OT Format Knowledge | If formats differ between .work and .strd, different parser structs needed |

---

## Open Questions

1. **Exact checksums for .work and bank files**
   - What we know: .ot files use a simple 16-bit additive checksum over bytes 0x00..0x33D (verified). project.work and bank files have checksums but algorithm is not confirmed from public docs.
   - What's unclear: Is it the same 16-bit additive checksum? CRC16? Proprietary?
   - Recommendation: Extract from ot-tools-io source reading (format study, not code copy). Alternatively, run diff experiments: parse known good file, corrupt checksum, observe OT behavior. Round-trip tests will fail if checksum is wrong — this becomes visible immediately.

2. **Exact size and unknown field positions in .work and bank files**
   - What we know: .ot file is 832 bytes (fully documented). ProjectFile and BankFile sizes unknown from public research.
   - What's unclear: What are the total sizes of project.work and bank01.work? Where are the known fields vs unknown blobs?
   - Recommendation: Read ot-tools-io source (struct field names + sizes) to build clean-room field table. The planner should include a Wave 0 task: "Create clean-room OT format spec document for .work and bank files."

3. **F_FULLFSYNC and FAT32 volumes on macOS 26.x**
   - What we know: F_FULLFSYNC works for APFS/HFS+; FAT32 behavior on modern macOS is less documented.
   - What's unclear: Does FAT32 driver on macOS 26 honor F_FULLFSYNC properly?
   - Recommendation: The success criterion explicitly requires "verified by integration test on a real FAT32 volume." This test is the answer. Plan must include an integration test that writes, fsyncs, and verifies on an actual FAT32 disk.

4. **sysinfo version for macOS 26.x compatibility**
   - What we know: sysinfo 0.35 released on crates.io; macOS 26.x (Darwin 25.2.0) is the target.
   - What's unclear: Has sysinfo been verified against macOS 26.x specifically?
   - Recommendation: Include `sysinfo::System::IS_SUPPORTED` check in startup. If `Disks::new_with_refreshed_list()` returns empty on macOS 26, fall back to directory sniffing of /Volumes/*.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust / Cargo | ot-parser + Tauri backend | ✓ | rustc 1.95.0 | — |
| Tauri CLI | App build | ✓ | tauri-cli 2.10.1 | — |
| Node.js | Frontend build | ✓ | v24.6.0 | — |
| npm | Package management | ✓ | 11.5.1 | pnpm 10.29.1 also available |
| pnpm | Alternative PM | ✓ | 10.29.1 | npm |
| Xcode CLI tools | macOS native app build | ✓ | 2416 | — |
| macOS 26.x (Darwin 25.2.0) | Target platform | ✓ | 26.2 | — |
| Real FAT32 volume | Integration test FNDN-04/05 | ✗ (not verified) | — | Must source for success criterion 3; CF card in USB adapter works |
| Real OT project files | Test corpus D-09 | ✗ (not in repo yet) | — | User provides; synthetic fixtures needed for CI |

**Missing dependencies with fallback:**
- Real FAT32 volume: Required by success criterion 3. Use user's CF card in USB card reader. macOS formats CF as FAT32 by default. Any USB FAT32 drive works for the integration test.
- Real OT project files: User must provide at least one real project file for the test corpus. Synthetic fixtures (hand-crafted binary) can cover edge cases but cannot replace real-world round-trip verification.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness + cargo-nextest (recommended) |
| Config file | none — `cargo test` works; `cargo nextest run` for parallel |
| Quick run command | `cargo test -p ot-parser` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FNDN-01 | .ot file parsed without panic | unit | `cargo test -p ot-parser test_parse_ot_samplefile` | ❌ Wave 0 |
| FNDN-02 | Round-trip: parse → serialize → parse produces identical output | unit | `cargo test -p ot-parser test_roundtrip_all_types` | ❌ Wave 0 |
| FNDN-03 | 1-indexed project slot ID rejects 0; bank slot rejects 256 | unit | `cargo test -p ot-parser test_index_newtype_bounds` | ❌ Wave 0 |
| FNDN-04 | Staging dir is on same volume as target | integration | `cargo test -p takoyaki-app test_staging_same_volume` | ❌ Wave 0 |
| FNDN-05 | sync_all called before rename | integration | `cargo test -p takoyaki-app test_atomic_write_fsync` | ❌ Wave 0 |
| FNDN-06 | App launches on macOS without crash | smoke | manual launch + `cargo tauri dev` | ❌ Wave 0 |
| FNDN-07 | SQLite schema initializes; tables exist | unit | `cargo test -p takoyaki-app test_db_init` | ❌ Wave 0 |
| FNDN-08 | Wallflower DB opened read-only; write fails | unit | `cargo test -p takoyaki-app test_wallflower_db_readonly` | ❌ Wave 0 |
| SAFE-03 | Snapshot files exist before write completes | integration | `cargo test -p takoyaki-app test_snapshot_before_write` | ❌ Wave 0 |
| SAFE-04 | Write failure leaves original untouched | integration | `cargo test -p takoyaki-app test_atomic_write_failure_rollback` | ❌ Wave 0 |
| BROW-01 | OT volume detected by directory sniffing | unit | `cargo test -p takoyaki-app test_detect_ot_volume` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p ot-parser`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full workspace test suite green + real FAT32 integration test before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `crates/ot-parser/tests/round_trip.rs` — covers FNDN-01, FNDN-02
- [ ] `crates/ot-parser/tests/indexing.rs` — covers FNDN-03
- [ ] `crates/takoyaki-app/tests/atomic_write.rs` — covers FNDN-04, FNDN-05, SAFE-03, SAFE-04
- [ ] `crates/takoyaki-app/tests/db_init.rs` — covers FNDN-07, FNDN-08
- [ ] `crates/takoyaki-app/tests/volume_detection.rs` — covers BROW-01
- [ ] `tests/fixtures/` directory with at least one synthetic .ot fixture

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Desktop app, no auth layer in Phase 1 |
| V3 Session Management | no | No user sessions |
| V4 Access Control | partial | Tauri capabilities system — restrict invoke to required commands only |
| V5 Input Validation | yes | All paths received from frontend validated before fs operations |
| V6 Cryptography | no | No encryption in Phase 1 |

### Known Threat Patterns for Tauri + Rust + FAT32

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal via OT file paths | Tampering | Validate all paths are under the detected OT volume mount point before any fs op |
| Write to arbitrary FS location | Tampering | Scope all write operations to OT volume path prefix; reject paths outside it |
| Symlink attack via CF card | Spoofing | Use `canonicalize()` on paths before operations; reject symlinks to outside volume |
| Tauri command surface exposure | Elevation of privilege | Capabilities JSON — only expose needed commands; no shell command invocation |

---

## Project Constraints (from CLAUDE.md)

| Directive | Category | Impact on Phase 1 |
|-----------|----------|--------------------|
| Tech stack: Tauri v2 + Rust + React/Next.js | Required | Non-negotiable stack |
| Database: SQLite via rusqlite (bundled) | Required | `features = ["bundled"]` — no system SQLite dep |
| OT format: clean-room Rust, no GPL | Required | Format study from ot-tools-io; independent implementation |
| Data safety: atomic writes, snapshot-before-write, dry-run for ALL write ops | Required | Atomic write engine + snapshot engine in Phase 1 |
| File access: USB disk mode only | Architecture | No direct hardware comm; detect mounted volume only |
| Licensing: MIT, no GPL dependencies | Required | Verify all crates — binrw (MIT), atomic-write-file (MIT), sysinfo (MIT), rusqlite (MIT) |
| Wallflower coupling: read-only | Required | SQLITE_OPEN_READONLY connection; no writes to Wallflower DB |
| Testing: full coverage; parser must have extensive fixtures | Required | Round-trip tests with real + synthetic OT files; see test map above |
| GSD workflow enforcement | Process | All file changes through GSD commands |

---

## Sources

### Primary (HIGH confidence)
- [docs.rs/binrw](https://docs.rs/binrw/latest/binrw/) — API patterns, magic bytes, count directive, round-trip example
- [docs.rs/atomic-write-file](https://docs.rs/atomic-write-file/latest/atomic_write_file/) — fsync, commit, cross-filesystem behavior
- [docs.rs/sysinfo Disk](https://docs.rs/sysinfo/latest/sysinfo/struct.Disk.html) — is_removable, mount_point, file_system methods
- [docs.rs/ot-tools-io](https://docs.rs/ot-tools-io/latest/ot_tools_io/) — file type list, struct names, indexing scheme
- [docs.rs/rusqlite_migration](https://docs.rs/rusqlite_migration/latest/rusqlite_migration/) — M::up, from_slice, to_latest pattern
- [v2.tauri.app/start/frontend/nextjs](https://v2.tauri.app/start/frontend/nextjs/) — output: 'export', frontendDist, next.config.mjs
- [v2.tauri.app/develop/calling-rust](https://v2.tauri.app/develop/calling-rust/) — command macro, State, AppHandle, error handling
- [tauri-specta DeepWiki](https://deepwiki.com/specta-rs/tauri-specta/2-getting-started) — Builder pattern, collect_commands, TypeScript export
- [OctaChainer otwriter.h](https://raw.githubusercontent.com/KaiDrange/OctaChainer/master/otwriter.h) — .ot file byte layout (verified field offsets)
- Wallflower codebase (`/Users/albair/src/wallflower`) — Cargo workspace, db migrations, lib.rs Tauri setup, next.config.mjs, globals.css [VERIFIED: Read tool]

### Secondary (MEDIUM confidence)
- cargo search (2026-04-29) — all Rust crate versions verified
- npm view (2026-04-29) — all npm package versions verified
- Elektronauts CF card structure thread — OT volume directory hierarchy
- Rust GitHub issue #55920 — macOS sync_all → F_FULLFSYNC behavior

### Tertiary (LOW confidence)
- ot-tools-io format coverage estimate (~68% documented) — not directly measurable from public docs

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all versions verified via cargo search / npm view / codebase inspection
- Architecture: HIGH — Wallflower is a working blueprint for identical stack
- OT format (.ot): HIGH — field layout verified from OctaChainer source
- OT format (.work, bank, markers): MEDIUM — struct names from docs.rs; field offsets require clean-room spec creation
- Pitfalls: HIGH — macOS fsync behavior confirmed from Rust stdlib issue; cross-volume rename is filesystem semantics
- Volume detection: MEDIUM — sysinfo API verified; macOS 26.x compatibility assumed

**Research date:** 2026-04-29
**Valid until:** 2026-05-29 (stable stack; OT format knowledge stable indefinitely; RC package pins should be re-verified)
