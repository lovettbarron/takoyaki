# Architecture Research

**Domain:** Tauri v2 desktop hardware manager — binary file management, versioning, safety model
**Researched:** 2026-04-29
**Confidence:** HIGH (Tauri v2 patterns), MEDIUM (multi-file atomic write strategy), HIGH (SQLite multi-pool)

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                        React / Next.js Frontend                      │
│  ┌───────────┐  ┌────────────┐  ┌──────────────┐  ┌─────────────┐  │
│  │ Project   │  │ Sample     │  │ Backup /     │  │ Wallflower  │  │
│  │ Browser   │  │ Browser    │  │ History UI   │  │ Search UI   │  │
│  └─────┬─────┘  └──────┬─────┘  └──────┬───────┘  └──────┬──────┘  │
│        │               │               │                  │         │
│        └───────────────┴───────────────┴──────────────────┘         │
│                             invoke() / listen()                      │
├─────────────────────────────────────────────────────────────────────┤
│                     Tauri IPC Bridge (JSON-RPC)                      │
├─────────────────────────────────────────────────────────────────────┤
│                     Rust Core (src-tauri/src/)                       │
│                                                                      │
│  ┌──────────────────────┐   ┌──────────────────────────────────────┐ │
│  │   Command Layer      │   │          Domain Services             │ │
│  │  (#[tauri::command]) │   │                                      │ │
│  │                      │   │  ┌──────────────┐  ┌─────────────┐  │ │
│  │  project_commands    │──▶│  │  OT Parser   │  │  Backup     │  │ │
│  │  backup_commands     │   │  │  (binrw)     │  │  Service    │  │ │
│  │  sample_commands     │   │  └──────┬───────┘  └──────┬──────┘  │ │
│  │  wallflower_commands │   │         │                  │         │ │
│  └──────────────────────┘   │  ┌──────▼───────┐  ┌──────▼──────┐  │ │
│                              │  │  Transaction │  │  Snapshot   │  │ │
│  ┌──────────────────────┐   │  │  Coordinator │  │  Store      │  │ │
│  │   App State          │   │  └──────┬───────┘  └──────┬──────┘  │ │
│  │  (Mutex<AppState>)   │   │         │                  │         │ │
│  │                      │   │  ┌──────▼──────────────────▼──────┐  │ │
│  │  - active_volume     │   │  │      Atomic Write Engine       │  │ │
│  │  - active_project    │   │  │  (stage → validate → rename)   │  │ │
│  │  - pending_tx        │   │  └────────────────────────────────┘  │ │
│  └──────────────────────┘   └──────────────────────────────────────┘ │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                     Storage Layer                             │   │
│  │  ┌─────────────────────┐    ┌────────────────────────────┐   │   │
│  │  │  Takoyaki SQLite DB │    │  Wallflower SQLite DB      │   │   │
│  │  │  (read/write pool)  │    │  (read-only pool)          │   │   │
│  │  └─────────────────────┘    └────────────────────────────┘   │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                     File System Layer                         │   │
│  │  ┌───────────────────────┐    ┌──────────────────────────┐   │   │
│  │  │  USB Volume (CF card) │    │  Local Snapshot Store    │   │   │
│  │  │  /Volumes/OCTATRACK/  │    │  ~/.takoyaki/snapshots/  │   │   │
│  │  └───────────────────────┘    └──────────────────────────┘   │   │
│  └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Communicates With |
|-----------|----------------|-------------------|
| React Frontend | UI rendering, user intent, display state | Tauri IPC (invoke/listen) |
| Command Layer | Thin Tauri entry points: validate inputs, call domain services, emit events | App State, Domain Services |
| OT Parser (`ot-parser` crate) | Read/write all OT binary formats (.work, .strd, .ot, bank, marker files) | Transaction Coordinator, raw filesystem |
| Transaction Coordinator | Orchestrate multi-file write plans: collect file mutations, validate consistency, hand to Atomic Write Engine | OT Parser, Atomic Write Engine, Backup Service |
| Atomic Write Engine | Stage files to temp dir, validate all, rename atomically into place | Filesystem (CF card volume) |
| Backup Service | Pre-write snapshot creation, snapshot indexing, revert logic | Snapshot Store, Takoyaki SQLite, OT Parser |
| Snapshot Store | Content-addressable storage of OT project file trees, keyed by SHA-256 | Filesystem (local app dir), Takoyaki SQLite |
| Volume Detector | Detect OT CF card mount/unmount on macOS via `sysinfo` + notify crate | App State, emits events to frontend |
| Takoyaki SQLite DB | Project index, backup history, snapshot manifest, sample assignments | Domain services via sqlx pool |
| Wallflower SQLite DB | Read-only: sample metadata (key, BPM, tags, file path) | Wallflower Service via read-only sqlx pool |
| App State (`Mutex<AppState>`) | Runtime state: active volume, active project, in-progress transactions | All Tauri commands via State extractor |

## Recommended Project Structure

```
takoyaki/
├── package.json                    # frontend deps
├── src/                            # Next.js / React frontend
│   ├── app/                        # Next.js app router pages
│   ├── components/
│   │   ├── project/                # Project browser, bank/pattern views
│   │   ├── samples/                # Sample slot assignment, Wallflower search
│   │   └── backup/                 # Snapshot history, revert UI
│   ├── hooks/                      # useTauriEvent, useVolume, useProject
│   ├── store/                      # Zustand / Jotai client state
│   └── lib/
│       └── tauri.ts                # typed wrappers around invoke()
└── src-tauri/
    ├── tauri.conf.json
    ├── capabilities/
    │   └── default.json            # filesystem + SQL permissions
    ├── Cargo.toml
    └── src/
        ├── main.rs                 # desktop entry point
        ├── lib.rs                  # Builder setup, plugin registration
        ├── commands/
        │   ├── project.rs          # project_open, project_list, project_copy
        │   ├── backup.rs           # snapshot_create, snapshot_list, revert
        │   ├── samples.rs          # slot_assign, sample_preview
        │   └── wallflower.rs       # wallflower_search, wallflower_connect
        ├── domain/
        │   ├── ot_parser/          # OT binary format crate (pure, no Tauri dep)
        │   │   ├── mod.rs
        │   │   ├── work_file.rs    # .work bank file parser
        │   │   ├── project_file.rs # .strd project file parser
        │   │   ├── ot_file.rs      # .ot slice metadata parser
        │   │   └── tests/          # fixture-based tests (real .work files)
        │   ├── transaction.rs      # TransactionPlan, multi-file write coordinator
        │   ├── atomic_write.rs     # stage-validate-rename engine
        │   ├── backup.rs           # snapshot creation, revert logic
        │   └── snapshot_store.rs   # content-addressable store, SHA-256 keyed
        ├── services/
        │   ├── volume_detector.rs  # macOS mount/unmount via sysinfo
        │   └── wallflower.rs       # read-only Wallflower DB access
        └── db/
            ├── mod.rs
            ├── migrations/         # SQLx compile-time migrations
            ├── project_repo.rs     # project index queries
            └── backup_repo.rs      # snapshot manifest queries
```

### Structure Rationale

- **`domain/ot_parser/` as isolated sub-module:** Pure Rust, zero Tauri dependency. Can be extracted to a standalone crate later or tested in isolation without spinning up Tauri. This boundary is the highest-risk code and benefits most from isolation.
- **`commands/` as thin wrappers:** Commands do no domain logic — they extract arguments, call domain services, and serialize results. Keeps `#[tauri::command]` boilerplate out of domain logic.
- **`domain/` vs `services/`:** `domain/` is application logic (OT-specific), `services/` is infrastructure (volume detection, external DB). Clean separation avoids coupling OT format knowledge to filesystem monitoring code.
- **`db/migrations/` with sqlx compile-time queries:** sqlx `query!` macro validates SQL at compile time against the schema. Migrations are embedded and run automatically at startup.

## Architectural Patterns

### Pattern 1: Three-Layer Safety Model

**What:** Every write operation passes through three gates before modifying OT files on the CF card.

**When to use:** Any operation that writes to the CF card volume — mandatory, no exceptions.

**Trade-offs:** Adds latency (snapshot + staging), but the OT's multi-file interdependency makes corruption recovery nearly impossible without this. The latency is acceptable for a desktop management tool.

**Flow:**

```
User intent (e.g. "assign sample to slot 3")
    │
    ▼
Layer 1 — Auto-Snapshot
    TransactionCoordinator::pre_snapshot()
    Reads all affected files → content-addresses each → records in SQLite snapshot manifest
    If snapshot fails → abort, surface error, no write attempted
    │
    ▼
Layer 2 — Dry-Run Preview (TransactionPlan)
    OT Parser builds a TransactionPlan: list of (file_path, new_bytes) pairs
    Plan is returned to frontend as a diff summary (files changed, slot deltas)
    Frontend renders preview; user confirms
    │
    ▼
Layer 3 — Atomic Staged Write
    AtomicWriteEngine::commit(plan):
      1. Write each new_bytes to temp file in same filesystem (e.g. /Volumes/OCTATRACK/.takoyaki_staging/)
      2. Validate all temp files (re-parse with OT Parser to confirm round-trip integrity)
      3. Rename each temp file over its target atomically (same-filesystem rename() is atomic on macOS)
      4. fsync the parent directory
    If any rename fails → revert from snapshot
```

### Pattern 2: TransactionPlan — Explicit Write Plans as Data

**What:** The OT Parser never writes directly to disk. It produces a `TransactionPlan` (a Vec of `FileWrite { path, bytes }`) that is handed to the Atomic Write Engine.

**When to use:** All multi-file OT operations. The 18-file write case (sample slot reassignment) is the canonical example.

**Trade-offs:** More code than direct writes, but decouples parsing from I/O, makes dry-run trivial (inspect the plan without executing it), and allows testing the parser completely without touching disk.

```rust
// Conceptual shape — not final API
pub struct FileWrite {
    pub path: PathBuf,
    pub content: Vec<u8>,
}

pub struct TransactionPlan {
    pub writes: Vec<FileWrite>,
    pub description: String,        // human-readable summary for UI
    pub affected_file_count: usize,
}

impl OtProject {
    pub fn plan_slot_assign(
        &self,
        track: u8,
        slot: u8,
        sample_path: &Path,
    ) -> Result<TransactionPlan, OtError> {
        // Returns plan — does NOT write anything
    }
}
```

### Pattern 3: Content-Addressable Snapshot Store

**What:** Snapshots store file content as SHA-256-keyed blobs in a local directory (`~/.takoyaki/objects/`), with a SQLite manifest mapping snapshot_id → (file_path, sha256, size). Identical content across snapshots shares storage.

**When to use:** Pre-write snapshots and version history. NOT for live file monitoring.

**Trade-offs:** SHA-256 of raw binary OT files is simple and robust. Does not require understanding the file format semantics. Deduplication is automatic for unchanged files. Revert is a content-addressed read + Atomic Write Engine commit (same safety path as forward writes).

**Manifest schema:**
```sql
CREATE TABLE snapshots (
    id          TEXT PRIMARY KEY,   -- UUID
    project_id  TEXT NOT NULL,
    created_at  INTEGER NOT NULL,   -- Unix timestamp
    label       TEXT,
    trigger     TEXT NOT NULL       -- 'auto_pre_write' | 'manual' | 'on_connect'
);

CREATE TABLE snapshot_files (
    snapshot_id TEXT NOT NULL REFERENCES snapshots(id),
    rel_path    TEXT NOT NULL,      -- path relative to project root
    sha256      TEXT NOT NULL,
    size_bytes  INTEGER NOT NULL,
    PRIMARY KEY (snapshot_id, rel_path)
);
```

### Pattern 4: Tauri State for Runtime Context

**What:** Active volume path and active project are stored in `Mutex<AppState>` registered with Tauri's `manage()`. All commands receive this via the `State<'_, T>` extractor.

**When to use:** Any shared mutable runtime data that commands need — current volume, in-progress transaction, connection state.

**Trade-offs:** Mutex contention is not a concern at Tauri's IPC throughput. Avoids global statics. State is naturally scoped to app lifetime.

```rust
pub struct AppState {
    pub active_volume: Option<PathBuf>,
    pub active_project: Option<OtProjectRef>,
    pub pending_transaction: Option<TransactionPlan>,
    pub takoyaki_db: SqlitePool,
    pub wallflower_db: Option<SqlitePool>,  // None if Wallflower not configured
}
```

### Pattern 5: Backend-to-Frontend Event Emission for Long Operations

**What:** Long Rust operations (snapshot creation, volume scan, large copy) emit progress events via `app_handle.emit("operation-progress", payload)`. Frontend listens with `listen()`.

**When to use:** Any operation taking more than ~200ms that the user should observe. Never make the frontend wait on a command that can stall.

**Trade-offs:** Requires frontend to manage listener lifecycle (unlisten on unmount). More wiring than a simple command, but essential for responsive UX.

## Data Flow

### Write Operation Flow (Sample Slot Assignment)

```
User clicks "Assign sample to slot"
    │
    ▼  invoke("plan_slot_assign", { track, slot, samplePath })
    │
    ▼  Rust: backup_service.snapshot_pre_write(project) → snapshot_id
    │
    ▼  Rust: ot_parser.plan_slot_assign(track, slot, path) → TransactionPlan
    │
    ▼  Return: { plan_id, files_changed: 18, description: "..." }  (to frontend)
    │
    ▼  Frontend renders diff preview, user confirms
    │
    ▼  invoke("commit_transaction", { plan_id })
    │
    ▼  Rust: atomic_write_engine.commit(plan)
         Stage → Validate → Rename → fsync
    │
    ▼  Rust: emit("write-complete", { snapshot_id, files_written: 18 })
    │
    ▼  Frontend shows success + snapshot reference
```

### Volume Detection Flow

```
macOS mounts USB disk
    │
    ▼  VolumeDetector (background task, notify + sysinfo)
         polls /Volumes or subscribes to DiskArbitration via IOKit
    │
    ▼  Detects volume matching OT signature (AUDIO/Projects/ directory exists)
    │
    ▼  app_handle.emit("volume-connected", { path: "/Volumes/OCTATRACK" })
    │
    ▼  Frontend: show volume banner, enable project list
    │
    ▼  auto-snapshot of connected projects ("on_connect" trigger)
```

### Wallflower Integration Flow

```
User opens Sample Browser with Wallflower search
    │
    ▼  invoke("wallflower_search", { query, key, bpm_range, tags })
    │
    ▼  Rust: WallflowerService::search(opts)
         Opens read-only pool to Wallflower DB path (from settings)
         SELECT ... FROM samples WHERE ...
    │
    ▼  Return: Vec<SampleResult> { path, key, bpm, tags, duration }
    │
    ▼  User selects sample → invoke("plan_slot_assign", { ..., samplePath })
         (continues as standard write flow above)
```

### Revert Flow

```
User selects snapshot to revert to
    │
    ▼  invoke("revert_to_snapshot", { snapshot_id })
    │
    ▼  Rust: auto-snapshot current state first (safety snapshot before revert)
    │
    ▼  Rust: snapshot_store.build_plan_from_snapshot(snapshot_id) → TransactionPlan
         Reads blobs from ~/.takoyaki/objects/ by SHA-256
         Produces same FileWrite list as any other write operation
    │
    ▼  Rust: atomic_write_engine.commit(plan)
         Identical path as forward writes — revert uses the same safety guarantees
    │
    ▼  emit("revert-complete", { snapshot_id, files_restored })
```

## Component Build Order

Build in this sequence — each layer depends only on what came before:

1. **OT Parser** (`domain/ot_parser/`) — Pure Rust, no Tauri, no async. Start here. Ship with fixture-based tests covering round-trip read/write for every file type. This is the highest-risk component; validate it first.

2. **Atomic Write Engine** (`domain/atomic_write.rs`) — Pure Rust filesystem primitives: temp file write, validate, atomic rename, fsync. Does not depend on OT Parser (takes `Vec<FileWrite>`). Test against a temp directory on the real filesystem.

3. **Snapshot Store + Backup Service** (`domain/snapshot_store.rs`, `domain/backup.rs`) — Depends on Atomic Write Engine (for revert) and SQLite schema. Implement schema migrations first, then snapshot creation and retrieval.

4. **Transaction Coordinator** (`domain/transaction.rs`) — Wires OT Parser → pre-snapshot → plan generation → Atomic Write Engine. First integration of the full write pipeline.

5. **Takoyaki SQLite DB** (`db/`) — Schema, migrations, repo queries. Needed by Backup Service. sqlx compile-time query macros enforce correctness.

6. **Tauri App State + Command Layer** (`lib.rs`, `commands/`) — Expose domain services via Tauri commands. Wire `AppState` with pools and active volume.

7. **Volume Detector** (`services/volume_detector.rs`) — Background task using `sysinfo` + notify crate for `/Volumes` changes. Emits Tauri events. Can be stubbed earlier with a manual "connect volume" command.

8. **Wallflower Service** (`services/wallflower.rs`) — Opened last; depends on sqlx read-only pool pattern being established. Read-only, so cannot corrupt data. Safest to defer.

9. **React Frontend** — Progresses in parallel with backend milestones. Can mock Tauri commands (`@tauri-apps/api/mocks`) during early UI development.

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| Octatrack CF Card | Direct filesystem reads/writes via `std::fs` in Rust; paths discovered by Volume Detector | USB disk mode only — no MIDI. Volumes appear at `/Volumes/<name>/` on macOS |
| Wallflower SQLite | `SqlitePool::connect_with(SqliteConnectOptions::new().read_only(true))` via sqlx | Path configured in Takoyaki settings. If path missing/invalid, feature gracefully disabled |
| macOS DiskArbitration | `sysinfo` crate for disk enumeration; `notify` crate for filesystem events on `/Volumes` | May require entitlement `com.apple.security.device.usb` in capabilities |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| Frontend ↔ Command Layer | Tauri `invoke()` (command) and `listen()` (events); all data JSON-serialized via serde | Commands are synchronous from frontend's perspective; backend async |
| Command Layer ↔ Domain Services | Direct Rust function calls; no serialization overhead | Commands are thin; domain services do not know about Tauri |
| OT Parser ↔ Transaction Coordinator | `TransactionPlan` as the handoff type; parser produces, coordinator consumes | Parser is pure; coordinator owns I/O decisions |
| Domain Services ↔ SQLite | sqlx async query macros; compile-time validated SQL | Two separate pools: Takoyaki (read/write) and Wallflower (read-only) |
| Backup Service ↔ Atomic Write Engine | Revert produces a `TransactionPlan` and passes it to the write engine — same path as forward writes | No special revert code path; reuse ensures revert has same safety guarantees |

## Anti-Patterns

### Anti-Pattern 1: Direct File Writes in the OT Parser

**What people do:** Have the parser write mutated bytes directly to the target file path.

**Why it's wrong:** Bypasses all three safety layers. A partial write or crash mid-operation leaves OT project files in an inconsistent state, which the OT hardware cannot recover from.

**Do this instead:** Parser produces `TransactionPlan` (data), never touches the filesystem. All writes flow through Atomic Write Engine.

### Anti-Pattern 2: One-Shot Snapshot at End of Operation

**What people do:** Take a "backup" by copying files after writing, to record what was changed.

**Why it's wrong:** If the write operation corrupts data, the snapshot already contains the corrupt state. The snapshot must be taken before any write attempt.

**Do this instead:** Snapshot always precedes write. Pre-write snapshot is created even before the TransactionPlan is generated, so any failure at any stage still has a clean rollback point.

### Anti-Pattern 3: Sharing the Wallflower DB Pool for Writes

**What people do:** Reuse the Wallflower connection pool for convenience, or accidentally open it as read-write.

**Why it's wrong:** Takoyaki has no schema ownership over Wallflower's database. Any write — even accidental — could corrupt Wallflower's data or cause conflicts with a running Wallflower instance.

**Do this instead:** Open the Wallflower pool with `.read_only(true)` in `SqliteConnectOptions`. Keep it as a separate named pool in `AppState`. Never pass it to any write-capable code path.

### Anti-Pattern 4: Blocking the Tauri Main Thread with Long Rust Operations

**What people do:** Write long synchronous `#[tauri::command]` functions for snapshot creation or full-project scans.

**Why it's wrong:** Blocks the Tauri core thread, freezing the UI and blocking other IPC messages.

**Do this instead:** Use `async fn` commands with `tauri::async_runtime::spawn()` for background work. Emit progress events to the frontend. The snapshot and scan pipelines are the two operations most likely to block on large projects.

### Anti-Pattern 5: Staging Files Across Filesystem Boundaries

**What people do:** Stage temp files in the macOS local filesystem (`/tmp`) when the target is the CF card volume.

**Why it's wrong:** `rename()` (which provides atomicity) only works within the same filesystem. Cross-filesystem rename falls back to copy-then-delete, which is not atomic and can leave partial files on failure.

**Do this instead:** Create the staging directory on the target volume itself (e.g., `/Volumes/OCTATRACK/.takoyaki_staging/`). Same filesystem → rename is atomic.

## Scaling Considerations

This is a single-user desktop tool. Scaling concerns are different from web services:

| Concern | At small projects (< 8 banks) | At large projects (8 banks, 128 patterns, 1000s of samples) |
|---------|-------------------------------|-------------------------------------------------------------|
| Snapshot size | Trivial (< 5 MB per snapshot) | May reach 50-100 MB; content-addressing keeps total storage manageable via dedup |
| Write plan generation | Instant | Still fast — 18-file plan is a bounded constant, not O(n) in project size |
| SQLite query perf | Not a concern | Not a concern — single user, local DB, indexed queries |
| Volume scan on connect | Fast | May take 1-3 seconds scanning thousands of sample files — emit progress events |
| Binary parsing correctness | Primary concern regardless of scale | Primary concern regardless of scale |

## Sources

- Tauri v2 Architecture: https://v2.tauri.app/concept/architecture/
- Tauri v2 Commands / Calling Rust: https://v2.tauri.app/develop/calling-rust/
- Tauri v2 IPC / Events: https://v2.tauri.app/concept/inter-process-communication/
- Tauri v2 State Management: https://github.com/tauri-apps/tauri-docs/blob/v2/src/content/docs/develop/state-management.mdx
- Tauri v2 Calling Frontend (events): https://v2.tauri.app/develop/calling-frontend/
- Tauri v2 Project Structure: https://v2.tauri.app/start/project-structure/
- Atomic file writes in Rust: https://crates.io/crates/atomic-write-file
- binrw (declarative binary read/write): https://jam1.re/blog/binread-a-declarative-rust-binary-parsing-library
- sqlx SqliteConnectOptions (read-only): https://docs.rs/sqlx/latest/sqlx/sqlite/struct.SqliteConnectOptions.html
- sysinfo crate (disk/volume enumeration): https://lib.rs/crates/sysinfo
- Long-running async tasks in Tauri v2: https://sneakycrow.dev/blog/2024-05-12-running-async-tasks-in-tauri-v2

---
*Architecture research for: Takoyaki — Octatrack desktop backup/versioning/file management tool*
*Researched: 2026-04-29*
