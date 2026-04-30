# Phase 3: Write Path and Backup - Context

**Gathered:** 2026-04-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can back up projects, browse snapshot history, restore any prior state, and preview exactly what will change before any destructive operation is committed — with every write going through the atomic staged-write engine built in Phase 1.

</domain>

<decisions>
## Implementation Decisions

### Backup Destination & Organization
- **D-01:** Backups live in `~/takoyaki/backups/`, following the same top-level home directory convention as Wallflower. App-managed, predictable location.
- **D-02:** Backups organized by project, then by date: `~/takoyaki/backups/PROJECT_NAME/YYYY-MM-DD_HH-MM_label/`. Project-first structure mirrors how musicians think about their OT work.
- **D-03:** Full project directory copy — every backup is a complete, self-contained snapshot including all samples in the AUDIO folder. A backup can be restored and played immediately without needing original source files.
- **D-04:** Manual backups only. User explicitly clicks "Back Up" on a project. No auto-backup on OT connect, no scheduled backups. User is always in control of when disk space is used.

### Snapshot History & Timeline UX
- **D-05:** Snapshot timeline is a reverse-chronological list per project. Each entry shows: timestamp, operation label (e.g., "manual backup", "pre-rename", "pre-restore"), file count, and total size. Dense, scannable, monospace-forward.
- **D-06:** Clicking a snapshot shows a file listing with change indicators: added/modified/removed/unchanged compared to the current project state on the OT card (or last known state if disconnected). Includes a [Restore This Snapshot] button.
- **D-07:** Backups live in the top-level sidebar section (Phase 1 D-07 already planned this slot). Independent from the project browser — accessible even when the OT is disconnected. Users can browse their entire backup history without a connected device.

### Dry-Run Preview
- **D-08:** Dry-run preview appears as a modal confirmation dialog. Shows: operation summary at top, then a list of files that will be added/modified/removed with sizes. [Cancel] and [Apply] buttons. Blocks other interaction until the user decides.
- **D-09:** Dry-run preview is always mandatory for ALL destructive operations. No skip option, no "don't show again" checkbox. Slight friction is intentional — matches the safety-first core value.
- **D-10:** The preview modal explicitly mentions the automatic pre-write snapshot: "A snapshot of the current state will be created before applying." This reinforces the safety model and builds user trust.

### Restore Workflow & Safety
- **D-11:** Every restore automatically creates a "pre-restore" snapshot of the current state before applying. You can always undo a restore by restoring the pre-restore snapshot. Safety net for the safety net, consistent with Phase 1's snapshot-before-write guarantee.
- **D-12:** If OT disconnects mid-restore: staging dir has partial writes but rename never happened, so project files are untouched. Staging dir cleaned up on next launch. App shows: "Restore aborted. Project unchanged." If OT disconnects mid-backup: partial backup is deleted. App shows: "Backup incomplete. Try again."
- **D-13:** Success feedback is an inline banner at the top of the current view: "✓ Backed up LIVESET_01 — 42 files · 128 MB · ~/takoyaki". Auto-dismisses after a few seconds. No modal for success — the operation is done, don't block.

### Claude's Discretion
- Progress indicator style during long backup/restore operations (progress bar, spinner, file-by-file counter)
- Checksum verification UX — how/whether to surface the SAFE-02 checksum comparison result to the user
- Backup deletion/cleanup UX — how users manage or prune old backups
- Snapshot retention policy (keep all forever, or configurable limits)
- Exact banner styling, animation, and auto-dismiss timing
- SQLite schema for backup history records (timestamps, operation types, file manifests)
- How the backup button is presented in the Projects view (toolbar, context menu, inline action)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project-Level
- `.planning/PROJECT.md` — Core value (safety-first), constraints (atomic writes, snapshot-before-write), prior art analysis
- `.planning/REQUIREMENTS.md` — SAFE-01 through SAFE-07 map to this phase; SAFE-03 and SAFE-04 are Phase 1 infrastructure this phase builds on
- `.planning/ROADMAP.md` — Phase 3 success criteria (5 criteria) and dependency on Phase 2

### Phase 1 Context (foundation)
- `.planning/phases/01-foundation/01-CONTEXT.md` — Visual identity (warm dark palette, monospace typography), sidebar nav with Backups section (D-07), volume detection UX (D-11 through D-14), snapshot storage format and SQLite schema are Claude's Discretion items from Phase 1

### Phase 2 Context (read-only browser)
- `.planning/phases/02-read-only-browser/02-CONTEXT.md` — Project list and navigation patterns (D-01 through D-04), health check UX patterns (D-11 through D-14) — Phase 3 should be visually consistent

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/ot-parser/` — Standalone OT binary parser crate (Phase 1). Phase 3 uses this to read project files for backup and to understand file structure for change detection.
- `crates/takoyaki-app/` — Tauri app crate with specta builder (Phase 1). Phase 3 adds new Tauri commands for backup, restore, snapshot operations.

### Established Patterns
- Tauri v2 IPC commands with tauri-specta for auto-generated TypeScript types — Phase 3 follows the same pattern for backup/restore commands
- Atomic write engine (Phase 1 SAFE-04) — staging to temp dir on same volume, fsync, rename. Phase 3 restore operations use this engine.
- SQLite via rusqlite (Phase 1 FNDN-07) — Phase 3 extends the schema with backup history, snapshot records

### Integration Points
- Phase 1 atomic write engine — restore operations go through this. No bypass.
- Phase 1 snapshot infrastructure (SAFE-03) — Phase 3 surfaces this to the user via the snapshot timeline UI
- Phase 1 volume detection — backup/restore commands check for mounted OT volume; Backups sidebar section works even when disconnected
- Phase 2 project list/detail views — backup button likely appears alongside or within the project browser UI

</code_context>

<specifics>
## Specific Ideas

- `~/takoyaki/` as the top-level data directory follows Wallflower's convention — consistent ecosystem feel for users of both apps
- The dry-run preview modal is the heart of the safety UX — it should feel deliberate and trustworthy, not like a nag dialog
- "A snapshot of the current state will be created before applying" is a specific line that should appear in every dry-run preview modal
- Success banners should be calm and informative (file count, size, destination) — not celebratory. Matches the Phase 1 "calm and ready" empty state philosophy.
- The Backups sidebar section working while disconnected is important — users should be able to browse their backup history and verify they have coverage without needing to plug in their OT

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 03-write-path-and-backup*
*Context gathered: 2026-04-30*
