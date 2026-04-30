# Phase 4: Advanced Management - Context

**Gathered:** 2026-04-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can perform the full range of project management operations — duplicate, rename, export, and copy banks across projects — with the same safety guarantees as Phase 3 (mandatory dry-run preview, automatic pre-operation snapshots, atomic writes).

</domain>

<decisions>
## Implementation Decisions

### Project Duplication
- **D-01:** Full copy — duplicate copies all referenced audio files (+ .ot sidecars) into the new project directory. Produces a fully independent, self-contained copy. No shared references.
- **D-02:** Default name: original name + `_copy` suffix (e.g., LIVESET_01 → LIVESET_01_copy).
- **D-03:** If the auto-generated name exceeds the OT directory name length limit, fall back to prompting the user to type a name that fits. No auto-truncation.

### Project Export
- **D-04:** Export produces a self-contained zip with the complete `/SETS/PROJECT_NAME/` directory AND all referenced audio files in `/AUDIO/`, preserving the OT directory structure. Unzipping to a blank CF card produces a playable project.
- **D-05:** Export includes `.ot` sidecar files for every referenced sample — slice points, loop settings, and trim data are preserved. Truly play-ready export.
- **D-06:** Exports saved to `~/takoyaki/exports/`, consistent with the Phase 3 backup convention (`~/takoyaki/backups/`). Organized by project name and date.

### Bank Copy & Conflict Resolution
- **D-07:** When copying a bank to another project, missing samples are copied automatically. If the target already has the same filename with identical content (hash match), skip. No user prompt for unambiguous cases.
- **D-08:** When a filename exists in the target with different content (hash mismatch), surface the conflict in the dry-run preview with three options: keep target's version, overwrite with source's version, or rename the incoming file.
- **D-09:** If the target bank slot is populated, warn the user and show available empty slots. User can pick an empty slot or explicitly confirm overwrite. No silent overwrites.
- **D-10:** Bank copy target selection uses a two-step picker dialog: Step 1 — select target project from a list; Step 2 — select target bank slot with a 4×4 grid showing populated vs. empty slots.

### Project Rename
- **D-11:** Clicking "Rename" makes the project name editable inline in the project detail header. User types the new name, confirms, then sees the mandatory dry-run preview showing directory rename + internal name field update.

### Management Actions UX
- **D-12:** Project-level actions (Duplicate, Rename, Export) appear as toolbar buttons in the project detail view header, alongside the project name and metadata.
- **D-13:** Bank copy is a per-bank action accessed by right-clicking a bank in the bank grid → "Copy to project..."
- **D-14:** All operations go through the mandatory dry-run preview (Phase 3 D-08/D-09) and automatic pre-operation snapshot (Phase 3 D-11). Success feedback uses the inline auto-dismissing banner (Phase 3 D-13).

### Claude's Discretion
- Exact toolbar button styling and icon choices for Duplicate/Rename/Export
- Bank grid right-click context menu implementation (native OS menu vs. custom)
- Export progress indicator during zip creation
- How the two-step bank copy picker dialog is styled
- Whether the rename inline edit validates OT-legal characters in real-time
- Zip compression level (speed vs. size tradeoff for audio files)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project-Level
- `.planning/PROJECT.md` — Core value (safety-first), constraints (atomic writes, snapshot-before-write, MIT license)
- `.planning/REQUIREMENTS.md` — MGMT-01, MGMT-02, MGMT-03, SMPL-02 map to this phase
- `.planning/ROADMAP.md` — Phase 4 success criteria (4 criteria) and dependency on Phase 3

### Prior Phase Context
- `.planning/phases/01-foundation/01-CONTEXT.md` — Visual identity (warm dark palette, monospace-forward typography, sidebar nav), parser crate architecture, atomic write engine, snapshot infrastructure
- `.planning/phases/02-read-only-browser/02-CONTEXT.md` — Project list layout (D-01), breadcrumb navigation (D-03), project detail tabs (D-04), bank grid layout (D-06), sample slot display (D-08-D-10)
- `.planning/phases/03-write-path-and-backup/03-CONTEXT.md` — Backup directory convention ~/takoyaki/ (D-01), dry-run preview modal (D-08-D-10), pre-operation snapshots (D-11), success banner (D-13), disconnect safety (D-12)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/ot-parser/` — Standalone parser for all OT file types. ProjectFile (text-based key=value), BankFile (binary with opaque body), SampleSettingsFile (.ot sidecar, 832 bytes), MarkersFile, ArrangementFile. All support from_bytes/to_bytes round-trip.
- `crates/takoyaki-app/src/atomic/mod.rs` — `atomic_write()` and `atomic_write_batch()` for safe file writes with staging, fsync, atomic rename.
- `crates/takoyaki-app/src/atomic/snapshot.rs` — Pre-write snapshot engine.
- `crates/takoyaki-app/src/commands/samples.rs` — `normalize_ot_path()` for OT sample path normalization from raw binary bytes.
- `crates/takoyaki-app/src/health/mod.rs` — `resolve_ot_path()` resolves OT-style paths to absolute filesystem paths with traversal prevention.

### Established Patterns
- Tauri v2 IPC commands with tauri-specta for auto-generated TypeScript types
- Atomic write engine: stage to temp on same volume → fsync → rename (SAFE-04)
- OT volume detection by `/AUDIO` + `/SETS` directory structure sniffing
- Project files live under `/SETS/PROJECT_NAME/` on the OT volume
- Audio files live under `/AUDIO/` at volume root (shared across projects)

### Integration Points
- Phase 3 dry-run preview modal — Phase 4 operations reuse this for all destructive actions
- Phase 3 snapshot engine — automatic pre-operation snapshots for all Phase 4 writes
- Phase 2 project detail view — toolbar buttons added to existing header component
- Phase 2 bank grid — right-click context menu added to existing bank grid cells
- Phase 2 project list — updates after rename/duplicate to reflect changes
- `zip` crate (already in Cargo.toml) — used for export packaging

</code_context>

<specifics>
## Specific Ideas

- Export zip preserves OT directory structure (`/SETS/` + `/AUDIO/`) so unzipping to a blank CF card produces a playable project — this is the key differentiator from a simple file archive
- Bank copy auto-resolves unambiguous sample conflicts (hash match = skip, missing = copy) and only surfaces true conflicts (same filename, different content) — minimize friction for the common case
- The two-step bank copy picker with the 4×4 grid showing populated vs. empty slots mirrors the OT's own bank mental model (Phase 2 D-06)
- Rename inline edit in the header is lightweight and fast — no modal for a simple name change, but the dry-run preview still shows the full impact before applying

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 04-advanced-management*
*Context gathered: 2026-04-30*
