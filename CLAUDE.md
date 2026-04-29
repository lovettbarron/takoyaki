<!-- GSD:project-start source:PROJECT.md -->
## Project

**Takoyaki**

Takoyaki is a desktop backup, versioning, and file management tool for the Elektron Octatrack. It lets musicians safely browse, back up, version, and manage their Octatrack projects and samples from a Mac — with a clean-room Rust parser for OT binary formats and a three-layer safety model that protects creative work from data loss or corruption. Optional integration with the Wallflower sample library enables metadata-powered sample search and one-click deployment to OT slots.

**Core Value:** An Octatrack user can manage their projects and samples with complete confidence that their creative work is never at risk — every destructive operation is snapshot-protected, previewed, and atomically applied.

### Constraints

- **Tech stack**: Tauri v2 native macOS app with Rust backend + React/Next.js frontend (same architecture as Wallflower)
- **Database**: SQLite for Takoyaki's own metadata (backup history, project index, sample assignments)
- **OT format**: Clean-room Rust implementation — no GPL dependencies. Use community-documented format specs and independent reverse engineering.
- **Data safety**: Atomic writes, snapshot-before-write, dry-run preview for ALL operations that modify OT project files. No exceptions.
- **File access**: USB disk mode only — OT mounted as a volume on Mac
- **Licensing**: MIT for all project code. No GPL dependencies in core.
- **Wallflower coupling**: Read-only access to Wallflower's SQLite DB. No write dependency.
- **Testing**: Full test coverage. OT binary parser must have extensive test fixtures from real OT project files.
<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->
## Technology Stack

### Rust Backend
| Technology | Version | Purpose |
|------------|---------|---------|
| **Tauri** | 2.x | Desktop app shell, IPC, OS integration |
| **binrw** | 0.15.x | Declarative binary format parsing for OT file types |
| **nom** | 8.x | Streaming parser for irregular binary sections |
| **rusqlite** | 0.39.x | SQLite database (bundled, synchronous) |
| **atomic-write-file** | 0.3.x | Atomic write-then-rename for file safety |
| **tempfile** | 3.x | Staging directories for atomic operations |
| **zip** | 8.x | Project snapshot/export archives |
| **notify** | 9.x-rc | Filesystem watching (macOS FSEvents/Kqueue) |
| **tauri-specta** | 2.x-rc | Auto-generated TypeScript types from Rust commands |
| **serde** / **serde_json** | 1.x | Serialization for config and API |
| **thiserror** | 2.x | Error types for Tauri command IPC |
| **tracing** | 0.1.x | Structured logging |

### Frontend (React Web UI)
| Technology | Version | Purpose |
|------------|---------|---------|
| **Next.js** | 15.x | React framework (static export for Tauri webview) |
| **React** | 19.x | UI framework |
| **TypeScript** | 5.x | Type safety |
| **Tailwind CSS** | 4.x | Styling |
| **zustand** | 5.x | Client state management |
| **@tanstack/react-query** | 5.x | Server state / API data |
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

Conventions not yet established. Will populate as patterns emerge during development.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.
<!-- GSD:architecture-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd:quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd:debug` for investigation and bug fixing
- `/gsd:execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->
