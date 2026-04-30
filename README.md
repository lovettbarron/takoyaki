# Takoyaki

A desktop backup, versioning, and file management tool for the [Elektron Octatrack](https://www.elektron.se/products/octatrack-mkii/). Browse, back up, version, and manage your Octatrack projects and samples from a Mac — with a clean-room Rust parser for OT binary formats and a three-layer safety model that protects creative work from data loss or corruption.

## Why

The Octatrack community has been underserved by tooling since OctaEdit was abandoned. Existing tools are either incomplete, narrowly focused, or unstable research projects. Takoyaki aims to be the reliable, open-source project management tool OT users have been asking for.

Every destructive operation is snapshot-protected, previewed, and atomically applied. Your creative work is never at risk.

## Features

**Available now (Phases 1-2):**

- Automatic OT volume detection when connected via USB disk mode
- Clean-room binary parser for all OT file types (.ot, .work, .strd, bank, markers, arrangement)
- Byte-exact round-trip fidelity — parse, serialize, re-parse produces identical results
- Project browser with searchable, filterable project list
- Project detail view with 4x4 bank grid, sample slot tables, and metadata display
- Health check engine — detects missing samples, wrong audio formats, incompatible sample rates
- Atomic write engine with F_FULLFSYNC on macOS (safe for CF card hot-unplug)
- Snapshot engine — copies all affected files before any write operation
- SQLite database for project index, backup history, and snapshot records
- Wallflower DB read-only integration (optional)

**Planned:**

- Backup and restore with chronological snapshot history (Phase 3)
- Dry-run preview for all destructive operations (Phase 3)
- Project export, bank copy, rename, and duplicate (Phase 4)
- Sample assignment from desktop UI and Wallflower library integration (Phase 5)

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | [Tauri v2](https://tauri.app/) (macOS native) |
| Backend | Rust (binrw, rusqlite, atomic-write-file, sysinfo) |
| Frontend | React 19, Next.js 15 (static export), TypeScript, Tailwind CSS v4 |
| State | Zustand (client), TanStack React Query (server) |
| OT parser | Clean-room Rust, MIT licensed, zero GPL dependencies |
| Database | SQLite (bundled via rusqlite) |

## Project Structure

```
takoyaki/
  crates/
    ot-parser/            # Standalone OT binary format parser library
      src/
        sample.rs         # .ot sample settings file parser
        project.rs        # .work/.strd project file parser
        bank.rs           # Bank file parser (bank01-16)
        markers.rs        # Markers file parser
        arrangement.rs    # Arrangement file parser (arr01-08)
        types.rs          # Index newtypes (ProjectSlotId, BankSlotId, BankNumber)
        error.rs          # ParseError types
      tests/
        round_trip.rs     # Byte-exact round-trip tests for all file types
        indexing.rs       # Boundary tests for index newtypes
    takoyaki-app/         # Tauri desktop application
      src/
        commands/         # Tauri IPC commands (device, projects, samples, health)
        db/               # SQLite database layer + Wallflower read-only access
        atomic/           # Atomic write engine + snapshot engine
        device/           # OT volume detection and polling
        health/           # Sample health check engine
  src/                    # React frontend
    app/                  # Next.js app router
    components/
      projects/           # Project list view (table, search, row)
      project-detail/     # Detail view (banks, samples, health, metadata)
      health/             # Health check display components
      ui/                 # shadcn component library
    lib/
      stores/             # Zustand stores (device, navigation, filter)
      tauri.ts            # TypeScript IPC wrappers
      types.ts            # Shared TypeScript types
  migrations/             # SQLite schema migrations
  tests/fixtures/         # OT binary test fixtures
```

## Development

### Prerequisites

- macOS (Tauri v2 target)
- Rust toolchain (stable)
- Node.js 20+
- An Elektron Octatrack in USB disk mode (for runtime testing)

### Setup

```bash
git clone https://github.com/lovettbarron/takoyaki.git
cd takoyaki
npm install
```

### Run

```bash
# Frontend dev server only (no Tauri shell)
npm run dev

# Full desktop app (requires Rust toolchain)
cargo tauri dev
```

### Test

```bash
# Rust tests (parser + app)
cargo test

# Frontend build check
npm run build
```

## Safety Model

Takoyaki uses a three-layer safety model for all write operations:

1. **Snapshot before write** — The snapshot engine copies all affected files to a timestamped directory with SHA-256 integrity hashes before any modification begins.
2. **Dry-run preview** — Every destructive operation can be previewed first, showing exactly which files will change and how. *(Phase 3)*
3. **Atomic staged writes** — Files are written to a temp location on the same volume, fsynced (F_FULLFSYNC on macOS), then atomically renamed. If anything fails, the original is untouched.

## License

MIT
