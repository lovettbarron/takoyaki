# Stack Research

**Domain:** Tauri v2 desktop app — binary file parsing, filesystem management, backup/versioning, cross-SQLite queries
**Researched:** 2026-04-29
**Confidence:** HIGH (all versions verified against crates.io API; architecture patterns verified against official Tauri docs)

---

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Tauri | 2.10.3 | Desktop app shell, IPC, OS integration | Already decided. Native macOS WebView, Rust backend, fine-grained capability permissions. v2 is stable and actively maintained (latest as of 2026-04). |
| Rust | stable (1.78+) | Backend runtime — parsing, filesystem, DB, business logic | Memory safety with zero-cost abstractions is the right tool for binary format parsing and atomic file ops. Tauri requires it. |
| React 19 | 19.x | Frontend UI framework | Decided (same as Wallflower). Ecosystem is dominant; TanStack Query and Zustand both target it. |
| Next.js | 15.x | React meta-framework | Decided (same as Wallflower). Used in static/SPA export mode for Tauri — no SSR needed. |

### Binary Format Parsing (Rust)

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| binrw | 0.15.1 | Declarative binary struct read/write via derive macros | **Primary choice for OT format parsing.** The `#[binrw]` derive macro maps directly to fixed-layout binary structs — exactly what OT's `.work`, `.strd`, bank files, and marker files are. Supports big/little endian, magic bytes, alignment, temp fields, computed counts, and both read and write in one annotation. Active: 0.15.x released March 2026, 8M+ downloads. Confidence: HIGH. |
| nom | 8.0.0 | Parser combinator for stream-oriented or irregular binary data | Use alongside binrw for any OT sub-formats that don't fit clean struct shapes (e.g., variable-length sections, embedded strings). nom 8 (Jan 2025) updated API; use `nom::Parser` trait pattern. Confidence: HIGH. |

**Do NOT use:**
- `deku` (0.20.3) — Bit-level focus is overkill for OT's byte-aligned structures; uses slice-based model rather than IO streams, making it harder to parse streaming data from large project files.
- `serde + bincode` (the ot-tools-io approach) — `bincode` does not map to an externally-specified binary format; it serializes Rust types into its own wire format. This works only when you control both ends of the wire. OT file formats are fixed hardware formats; you need a parser that reads to spec, not a serializer. Also: bincode development has ceased following a maintainer incident.
- `binary-layout` — Type-map overlay approach; fine for small fixed buffers, but lacks the ergonomics of binrw for complex nested struct trees with conditional fields.

### Filesystem Watching

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| notify | 9.0.0-rc.3 | Cross-platform filesystem event watching | The standard Rust crate for FS watching. v9 RC (latest stable: 6.1.1 is old; 9.x RC is the active line). Use `notify-debouncer-full` (0.8.0-rc.1) alongside it. |
| notify-debouncer-full | 0.8.0-rc.1 | Debounce, merge, and deduplicate FS events | OT CF card files may generate bursts of events on mount/unmount and during OT saves. The full debouncer handles rename tracking correctly (critical: OT sometimes replaces files by rename-over). Use 2–4 second debounce window for CF card events. |

**Note on RC versions:** Both notify and notify-debouncer-full are at RC status for v9. The 6.x stable line is functional but lacks improved macOS FSEvents handling. RC9 is the only actively maintained line; it is safe to use for a new project — no stable 9.x release exists yet.

### Atomic File Operations

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| atomic-write-file | 0.3.0 | Write-then-rename atomic file replacement | The canonical approach for atomic writes on POSIX: write to temp file in same directory, then `rename()` which is atomic on POSIX filesystems. This crate encapsulates that pattern. Critical for the three-layer safety model. |
| tempfile | 3.27.0 | Secure temporary file and directory creation | Use for staging areas: create a temp dir for a snapshot's working tree, build it up, then atomically commit. Integrates cleanly with `atomic-write-file`. |

**Atomic write strategy for OT project files:**
1. Auto-snapshot: zip the entire project directory to `.takoyaki/snapshots/<uuid>.zip` before any mutation.
2. Stage all writes in a `tempfile::TempDir` (same filesystem as OT volume — copy the original files there first).
3. Validate staged files (parse them back with binrw to confirm round-trip integrity).
4. Apply: use `atomic-write-file` to replace each target file via rename. All renames succeed or the operation rolls back to snapshot.

### Backup / Snapshot Archives

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| zip | 8.6.0 | Create and read ZIP archives for project snapshots | **Preferred over tar for snapshots.** ZIP is universally accessible to musicians on any OS without tooling. A snapshot is a self-contained ZIP of the OT project directory tree. zip crate is actively maintained (8.6.0 released April 2026), supports deflate and stored modes, and handles arbitrary path structures cleanly. |

**Why ZIP over tar:** tar+gz is a byte stream — partial extraction requires full scan. ZIP has a central directory, so individual files can be extracted randomly. This matters for "show me what changed between snapshot A and B" without extracting both archives fully. Musicians can also open snapshots in Finder.

**Why not a custom binary format:** Unnecessary complexity, no tooling compatibility, harder to debug.

### SQLite — Application Database

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| rusqlite | 0.39.0 | Takoyaki's own SQLite database (backup history, project index, sample slot assignments) | Synchronous, low-overhead, ergonomic. No async runtime required for DB-only access. Use the `bundled` feature to statically link SQLite — avoids macOS system SQLite version variance and distribution headaches. |
| rusqlite_migration | 2.5.0 | Schema migrations for Takoyaki's DB | Minimal, correct, uses SQLite's `user_version` pragma for fast version checks. Avoids the complexity of diesel-migrations or sea-orm for a single-file SQLite schema. |

**Why rusqlite over sqlx:** sqlx 0.8.6 is the current stable (0.9.0-alpha.1 is pre-release). sqlx's async model adds complexity for what is ultimately synchronous local file I/O. rusqlite is synchronous and maps naturally to Tauri's `#[tauri::command]` pattern where you spawn blocking work on the thread pool. For a single-user desktop app with one SQLite file, async DB is overhead with no benefit.

**Why rusqlite over Tauri's official SQL plugin (`tauri-plugin-sql`):** The Tauri SQL plugin exposes DB access to the frontend layer. For Takoyaki, all DB logic belongs in Rust — the frontend asks the Rust backend for data via typed commands, not raw SQL. This keeps business logic server-side and prevents the frontend from constructing arbitrary queries against the schema.

### Cross-Database Queries (Wallflower Integration)

**Pattern:** ATTACH DATABASE with rusqlite, read-only.

```rust
// Open Takoyaki's own DB
let conn = Connection::open_with_flags("takoyaki.db", OpenFlags::SQLITE_OPEN_READ_WRITE)?;

// Attach Wallflower's DB read-only
conn.execute_batch(&format!(
    "ATTACH DATABASE '{}' AS wallflower",
    wallflower_path.display()
))?;

// Cross-DB query
let rows = conn.prepare(
    "SELECT w.path, w.bpm, w.key FROM wallflower.samples w
     LEFT JOIN sample_assignments sa ON sa.wallflower_id = w.id
     WHERE sa.project_id = ?1"
)?;
```

This is standard SQLite ATTACH behavior — no extra crates needed. rusqlite exposes `execute_batch` for the ATTACH statement. Keep the Wallflower connection read-only by using `OpenFlags::SQLITE_OPEN_READ_ONLY` on the attached path to prevent accidental writes. The Wallflower DB is discovered via a user-configured path stored in Takoyaki's settings table.

### Tauri Plugins

| Plugin | Version | Purpose | Why |
|--------|---------|---------|-----|
| tauri-plugin-fs | 2.5.0 | Frontend-accessible filesystem operations | For the React UI to read directory listings, file metadata, and small text files. Configure `fs:scope` with `**/*` pattern for desktop (users select their OT CF card path via dialog). |
| tauri-plugin-dialog | 2.7.0 | Native file/folder picker dialogs | **Required for USB drive access.** User selects OT CF card root via native folder picker — no pre-configured path needed. The user-selected path is then passed back to Rust for all subsequent operations. No `fs:scope` path-guessing needed. |
| tauri-plugin-shell | 2.3.5 | Shell command execution | May be needed for any `ffmpeg` or audio format conversion (optional; defer until needed). |

**Tauri v2 permission pattern for external volumes:**
- Do NOT hardcode `/Volumes/**` in capabilities — this grants broad access and bypasses user consent.
- DO use `tauri-plugin-dialog` to let the user pick the CF card root once. Store the path in app settings. Pass it to Rust backend commands as a parameter.
- For desktop: add `"identifier": "fs:scope", "allow": [{ "path": "**/*" }]` to capabilities only if the frontend needs direct FS access. All OT binary file I/O should go through Rust commands, not frontend plugin calls.
- On macOS sandboxed builds: use `NSOpenPanel` (what the dialog plugin wraps) — it grants security-scoped bookmark access automatically.

### IPC / Type Safety

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| tauri-specta | 2.0.0-rc.24 | Auto-generate TypeScript types from Rust command signatures | Eliminates the class of bugs where the frontend passes the wrong shape to a Rust command. Annotate `#[tauri::command]` with specta's `#[specta::specta]` and export a binding file. Works with tauri-specta v2 RC (actively maintained, latest RC March 2026). |

### Frontend State Management

| Library | Purpose | Why |
|---------|---------|-----|
| Zustand | Global UI state (active project, selected slots, UI preferences) | Simpler than Redux for a single-window desktop app. Store maps cleanly to Tauri's window-level state. Grew 150% in adoption in 2025. |
| TanStack Query (React Query) | Server-state caching for Rust command results | Treats Rust backend commands as "server endpoints" — automatic caching, loading states, background refetch. Pairs perfectly with tauri-specta typed commands. Standard pattern in Tauri community templates. |

### Error Handling

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| thiserror | 2.0.18 | Define typed error enums for Tauri commands | Tauri commands must return `Result<T, E>` where E: `serde::Serialize`. `thiserror` derive makes this ergonomic. Use for all public command errors and OT parser errors. |
| anyhow | 1.0.102 | Internal error propagation in non-command code | Use inside Rust modules where errors don't cross the IPC boundary. Do NOT return `anyhow::Error` from Tauri commands — it doesn't implement Serialize. Convert to `thiserror` at the command boundary. |

### Logging / Observability

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| tracing | 0.1.44 | Structured async-aware logging and spans | Standard for Rust async/Tauri apps. Use `tracing::instrument` on command handlers to trace slow OT parse operations. Integrates with `tauri-plugin-log` for writing to file. |

---

## Installation

```toml
# Cargo.toml — Rust dependencies

[dependencies]
# Tauri
tauri = { version = "2", features = [] }
tauri-plugin-fs = "2"
tauri-plugin-dialog = "2"
tauri-plugin-shell = "2"
tauri-specta = { version = "=2.0.0-rc.24", features = ["derive", "tauri"] }
specta-typescript = "0.0.7"

# Binary parsing
binrw = "0.15"
nom = "8"

# Filesystem watching
notify = "9.0.0-rc.3"
notify-debouncer-full = "0.8.0-rc.1"

# Atomic file operations
atomic-write-file = "0.3"
tempfile = "3"

# Archive / backup
zip = { version = "8", features = ["deflate"] }

# SQLite
rusqlite = { version = "0.39", features = ["bundled"] }
rusqlite_migration = "2.5"

# Error handling
thiserror = "2"
anyhow = "1"

# Serialization (IPC types)
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

```bash
# Frontend dependencies (npm)
npm install @tauri-apps/api @tauri-apps/plugin-fs @tauri-apps/plugin-dialog @tauri-apps/plugin-shell
npm install zustand @tanstack/react-query
```

---

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| binrw | nom only | Use nom only if format is non-seekable or deeply recursive. For OT's fixed structs, binrw's derive macros are far less boilerplate. |
| binrw | deku | Use deku if you need bit-level field access (e.g., packed bitfields). OT format is byte-aligned. |
| rusqlite (sync) | sqlx (async) | Use sqlx if you want async DB and are comfortable with macro-time query checking. sqlx 0.9 is still in alpha. rusqlite is simpler for single-user desktop apps. |
| rusqlite | tauri-plugin-sql | Use tauri-plugin-sql only if you want the frontend to run SQL directly. Takoyaki's SQL stays in Rust. |
| zip | tar + flate2 | Use tar if you need POSIX permissions and symlinks. ZIP is better for cross-platform archive accessibility. |
| atomic-write-file | manual rename | Use atomic-write-file to avoid reimplementing the same pattern for every file write path. |
| thiserror + anyhow | eyre | eyre is ergonomic but less common in Tauri community. thiserror + anyhow is the documented Tauri pattern. |

---

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `serde + bincode` for OT format parsing | bincode encodes Rust types into its own wire format — it cannot parse an externally-specified binary format. ot-tools-io uses this approach and the result is fragile (field ordering and repr are implicitly coupled). | `binrw` with explicit layout annotations |
| `deku` for OT format | Slice-based (not stream-based) model; poor ergonomics for large file parsing; overkill for byte-aligned structures | `binrw` |
| `tauri-plugin-sql` for core DB access | Moves SQL to the frontend layer; breaks separation of concerns; frontend cannot be trusted with raw DB access | `rusqlite` in Rust commands |
| `diesel` ORM | Heavy, requires schema macro codegen, poor ergonomics for `ATTACH DATABASE` cross-DB queries | `rusqlite` with raw SQL |
| `sea-orm` | Async-first ORM adds complexity; overkill for single-file local SQLite | `rusqlite` |
| `anyhow` in Tauri command return types | `anyhow::Error` does not implement `serde::Serialize` — Tauri will refuse to serialize it | `thiserror` for command error types |
| `notify` v6.x stable | Old stable line; v9 RC is the active maintained line with better macOS FSEvents | `notify` 9.0.0-rc.3 |
| Writing OT files from the frontend | Frontend has no knowledge of OT format interdependencies; a single sample slot assignment changes up to 18 files | All OT mutations in Rust commands only |

---

## Stack Patterns by Variant

**For OT binary format parsing (clean-room implementation):**
- Define one `#[binrw]` struct per OT file type (`.work`, `.strd`, `.ot`, bank, marker)
- Annotate each field with `#[br(big)]` or `#[br(little)]` as the format requires (most OT files are big-endian)
- Use `#[br(magic = b"...")]` to assert file type signatures
- Use `#[br(temp)]` for length fields that should not be stored in the struct
- Write round-trip tests: parse real OT files → serialize back → compare bytes exactly

**For the three-layer safety model:**
- Layer 1 (auto-snapshot): call `snapshot_project()` at the start of every mutating command. This zips the project dir to `.takoyaki/snapshots/`. Non-blocking: spawn in Tauri's thread pool.
- Layer 2 (dry-run preview): implement `apply_operation(op: OtOperation, dry_run: bool)` — when `dry_run: true`, compute the diff and return it to the frontend without writing. Frontend displays the preview.
- Layer 3 (atomic write): use `atomic-write-file` for every individual file write. Wrap multi-file operations in a transaction struct that tracks pending renames; on error, issue counter-renames back.

**For Wallflower integration (optional feature):**
- Check for Wallflower DB path in settings on startup
- If present, ATTACH at connection open time
- All Wallflower queries are SELECT only — enforce with `OpenFlags::SQLITE_OPEN_READ_ONLY` on the attached file path
- Gate all Wallflower-reading commands with `#[cfg(feature = "wallflower")]` or runtime flag — do not break app startup if Wallflower is absent

**For USB/CF card detection:**
- Do NOT auto-discover mounted volumes — user picks the OT root via dialog plugin
- Store the selected path in app settings (rusqlite settings table)
- Re-validate on startup: check if the path exists and contains expected OT directory structure
- Watch the parent mount point with `notify` to detect unmount events; disable write operations immediately on unmount

---

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| tauri 2.10.3 | tauri-plugin-fs 2.5, tauri-plugin-dialog 2.7 | All v2 plugins track tauri v2 major. |
| tauri-specta 2.0.0-rc.24 | tauri 2.x | v2 RC is required for Tauri v2 support. v1 (1.0.2) only supports Tauri v1. |
| rusqlite 0.39 | bundled SQLite 3.47.x | `bundled` feature pins the SQLite version; avoids macOS system SQLite (typically 3.39). |
| notify 9.0.0-rc.3 | notify-debouncer-full 0.8.0-rc.1 | Both are RC; must be updated together as they share internal event types. |
| binrw 0.15.1 | Rust stable 1.78+ | No nightly features required. |
| nom 8.0.0 | Rust stable | nom 8 introduced breaking API changes from nom 7 — use nom 8's `Parser` trait combinator pattern, not the nom 7 function style. |
| sqlx 0.8.6 | (not recommended for this project) | Listed for reference: stable sqlx. 0.9.x is pre-release. |

---

## Sources

- crates.io API (verified 2026-04-29): binrw 0.15.1, nom 8.0.0, deku 0.20.3, notify 9.0.0-rc.3, notify-debouncer-full 0.8.0-rc.1, atomic-write-file 0.3.0, tempfile 3.27.0, zip 8.6.0, rusqlite 0.39.0, rusqlite_migration 2.5.0, sqlx 0.8.6, thiserror 2.0.18, anyhow 1.0.102, tracing 0.1.44, tauri 2.10.3, tauri-plugin-fs 2.5.0, tauri-plugin-dialog 2.7.0, tauri-specta 2.0.0-rc.24
- Context7 `/tauri-apps/tauri-docs` — filesystem plugin permissions, dialog plugin, state management, async commands
- Context7 `/jam1garner/binrw` — BinRead/BinWrite derive macro patterns, attribute reference
- Context7 `/notify-rs/notify` — debouncer-full usage, custom watcher config
- Context7 `/rusqlite/rusqlite` — connection flags, bundled feature, ATTACH DATABASE pattern
- [Tauri filesystem plugin docs](https://v2.tauri.app/plugin/file-system/) — permission scopes, BaseDirectory, capabilities config
- [Tauri dialog plugin docs](https://v2.tauri.app/plugin/dialog/) — file/folder picker, USB path access pattern
- [Tauri allowing file access discussion](https://github.com/orgs/tauri-apps/discussions/11792) — `**/*` scope and macOS entitlement for unrestricted path access — HIGH confidence
- [binrw comparison with deku](https://github.com/jam1garner/binrw/discussions/184) — confirmed byte vs bit level tradeoffs
- [ot-tools-io crate docs](https://docs.rs/ot-tools-io) — confirmed GPL-3.0 license, confirmed uses bincode (not binrw) — informs our clean-room approach
- [Tauri v2 SQLite patterns](https://tauritutorials.com/blog/building-a-todo-app-in-tauri-with-sqlite-and-sqlx) — sqlx vs rusqlite tradeoff — MEDIUM confidence
- [Rust error handling in Tauri](https://tauritutorials.com/blog/handling-errors-in-tauri) — thiserror for command errors, anyhow incompatibility — HIGH confidence
- [bincode development cessation notice](https://generalistprogrammer.com/comparisons/serde-vs-bincode) — MEDIUM confidence (single source)

---

*Stack research for: Takoyaki — Tauri v2 desktop app for Octatrack backup/versioning/file management*
*Researched: 2026-04-29*
