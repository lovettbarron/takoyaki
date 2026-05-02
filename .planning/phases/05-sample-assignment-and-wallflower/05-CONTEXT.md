# Phase 5: Sample Assignment and Wallflower - Context

**Gathered:** 2026-05-02
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can assign any desktop audio file to a specific Flex or Static sample slot with all affected OT binary files updated atomically, and optionally browse and deploy samples from the Wallflower library — with graceful degradation when Wallflower is not present.

</domain>

<decisions>
## Implementation Decisions

### Assignment Workflow
- **D-01:** Each slot row on the SamplesTab has a small assign button ([↑]). Click the button to open a native macOS file picker. Click the row body to expand (existing Phase 2 behavior preserved). Clear separation of assign vs. inspect actions.
- **D-02:** Assigning to an occupied slot uses the same flow — file picker opens, dry-run preview explicitly shows old → new replacement (e.g., "kick_808.wav → new_kick.wav").
- **D-03:** Dry-run preview uses summary + expandable detail: clear summary at top ("Assigning kick_808.wav to Flex #003 — 18 files will be updated") with an expandable section listing every affected file with ADD/MOD indicators. Matches Phase 3 dry-run modal pattern.
- **D-04:** The dry-run preview includes the standard snapshot mention: "A snapshot of the current state will be created before applying." (Phase 3 D-10).
- **D-05:** Success feedback uses the inline auto-dismissing banner (Phase 3 D-13).

### Wallflower Discovery & Degradation
- **D-06:** Takoyaki finds the Wallflower DB using a priority order: (1) user-configured path in Settings, (2) auto-discover from Wallflower's known default location (~/wallflower/ or equivalent). If neither found, Wallflower integration is silently unavailable.
- **D-07:** When Wallflower is not available, the Wallflower panel is hidden entirely — no error, no empty state, no "not connected" message. The app works as a standalone OT tool. The panel appears only when the Wallflower DB is successfully found and opened.
- **D-08:** Settings includes a Wallflower section showing connection status, current DB path, and a [Change...] button for user override.

### Wallflower Browser UX
- **D-09:** Wallflower library appears as a collapsible panel below the Flex/Static slot lists on the SamplesTab. Slots are visible above while browsing samples below — short visual distance for push-to-slot.
- **D-10:** Push-to-slot flow: click a sample in the Wallflower panel → a slot picker appears (Flex/Static toggle + slot number dropdown showing which slots are empty/occupied) → confirm → dry-run preview → apply. The sample file is copied from Wallflower's location to the OT's /AUDIO/ directory as part of the atomic write transaction.
- **D-11:** Each Wallflower sample row shows: filename, musical key (if detected), BPM (if detected), and tags as small badges. Compact, scannable, monospace-forward — matches the established visual identity.
- **D-12:** Wallflower panel includes a search/filter bar supporting key, BPM, and tag queries.

### Slot Validation
- **D-13:** Flex vs Static slot type mismatch is a hard block with inline error below the slot row. Error explains why (e.g., "Sample exceeds Flex RAM limit") and suggests the equivalent slot number in the correct type with a one-click redirect: [Assign to Static #003] [Cancel].
- **D-14:** Audio format validation uses two tiers: hard block for incompatible formats (non-WAV/AIFF — MP3, FLAC, etc.) with inline error explaining OT only supports WAV and AIFF; soft warning for non-ideal parameters (48kHz, 32-bit) shown in the dry-run preview with [Cancel] [Assign Anyway]. Users who know what they're doing can proceed past soft warnings.

### Claude's Discretion
- Exact assign button icon and styling on slot rows
- Wallflower search/filter UX details (debounce, minimum query length, result limit)
- Wallflower panel collapse/expand animation and default state
- Slot picker dropdown styling in the push-to-slot flow
- How the slot picker indicates occupied vs empty slots (color, icon, text)
- Wallflower auto-discovery: exact default path and detection heuristic
- Whether the Wallflower panel remembers its collapsed/expanded state across sessions
- Search result sorting (by relevance, name, BPM, key)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project-Level
- `.planning/PROJECT.md` — Core value (safety-first), constraints (atomic writes, snapshot-before-write, MIT license, read-only Wallflower DB)
- `.planning/REQUIREMENTS.md` — SMPL-01, SMPL-03, INTG-01, INTG-02, INTG-03 map to this phase
- `.planning/ROADMAP.md` — Phase 5 success criteria (4 criteria) and dependency on Phase 4

### Prior Phase Context
- `.planning/phases/01-foundation/01-CONTEXT.md` — Visual identity (warm dark palette, monospace-forward typography, sidebar nav), parser crate architecture (`ot-parser`), atomic write engine, snapshot infrastructure, read-only Wallflower DB connection (FNDN-08)
- `.planning/phases/02-read-only-browser/02-CONTEXT.md` — SamplesTab layout (Flex/Static tables, D-08-D-10), SlotRow expand behavior (D-10), health check severity tiers (D-12), hide-empty-slots default (D-08)
- `.planning/phases/03-write-path-and-backup/03-CONTEXT.md` — Dry-run preview modal (D-08-D-10), pre-operation snapshots (D-11), success banner (D-13), disconnect safety (D-12)
- `.planning/phases/04-advanced-management/04-CONTEXT.md` — Export convention ~/takoyaki/exports/ (D-06), bank copy conflict resolution pattern (D-07-D-09), toolbar button pattern in MetadataHeader (D-12)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/ot-parser/` — Full parser for all OT file types. `SampleSettingsFile` (.ot sidecar, 832 bytes) handles sample metadata. `BankFile` for bank-level slot data. All support `from_bytes`/`to_bytes` round-trip.
- `crates/takoyaki-app/src/atomic/mod.rs` — `atomic_write()` and `atomic_write_batch()` for safe multi-file writes with staging, fsync, atomic rename. Phase 5 assignment uses `atomic_write_batch()` for the up-to-18-file transaction.
- `crates/takoyaki-app/src/atomic/snapshot.rs` — Pre-write snapshot engine. Automatically snapshots all affected files before any write.
- `crates/takoyaki-app/src/db/wallflower.rs` — `open_wallflower_db()` with `SQLITE_OPEN_READ_ONLY` flag. Driver-level write protection already built (FNDN-08).
- `crates/takoyaki-app/src/commands/samples.rs` — `SampleSlotResponse` with `flex`/`static_slots` vectors, `SampleSlot` struct with `slot_index`, `normalize_ot_path()` for OT path normalization.
- `crates/takoyaki-app/src/health/mod.rs` — `resolve_ot_path()` resolves OT-style paths to absolute filesystem paths with traversal prevention. Audio format validation patterns (sample rate, bit depth, WAV/AIFF checks).
- `src/components/project-detail/SamplesTab.tsx` — Existing Flex/Static sample slot display with SlotRow components.
- `src/components/project-detail/SlotRow.tsx` — Individual slot row with expand/collapse for cross-reference view. Phase 5 adds assign button to this component.

### Established Patterns
- Tauri v2 IPC commands with tauri-specta for auto-generated TypeScript types
- Atomic write engine: stage to temp on same volume → fsync → rename (SAFE-04)
- Dry-run preview modal with file list and snapshot mention (Phase 3)
- Inline auto-dismissing success banner (Phase 3)
- OT volume detection and device state management via `AppState`

### Integration Points
- Phase 3 dry-run preview modal — Phase 5 assignment reuses this for all sample operations
- Phase 3 snapshot engine — automatic pre-operation snapshots for all Phase 5 writes
- Phase 2 SamplesTab — assign button added to existing SlotRow component
- Phase 2 health check — format validation logic reused for assignment-time checks
- `crates/takoyaki-app/src/db/wallflower.rs` — Read-only connection to Wallflower DB, ready to query

</code_context>

<specifics>
## Specific Ideas

- The assign button on each slot row preserves Phase 2's click-to-expand interaction while adding a clear, discoverable assignment trigger — no ambiguity about what clicking does
- The Wallflower panel below the slot lists keeps assignment in-context: you can see your slots above and browse Wallflower samples below without context-switching to a separate view
- The push-to-slot picker showing empty/occupied status mirrors the bank copy picker pattern from Phase 4 (D-10) — consistent interaction for slot selection across the app
- Slot type validation with inline redirect ("Assign to Static #003 instead?") turns an error into a one-click fix — friction only when there's a real problem, and the fix is immediate
- Two-tier format validation (hard block vs soft warning) respects power users who intentionally use non-standard sample rates while protecting against genuinely incompatible formats

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 05-sample-assignment-and-wallflower*
*Context gathered: 2026-05-02*
