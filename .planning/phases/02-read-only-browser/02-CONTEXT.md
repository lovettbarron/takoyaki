# Phase 2: Read-Only Browser - Context

**Gathered:** 2026-04-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can browse every OT project on a mounted card in full detail — projects, banks, patterns, sample slots, and metadata — and detect problems in their project files, all without touching a single byte on the CF card. This phase builds the complete read-only UI on top of the parser and app scaffold from Phase 1.

</domain>

<decisions>
## Implementation Decisions

### Project List & Navigation
- **D-01:** Projects displayed as a compact monospace table with columns: name, BPM, banks (used/total), last modified date. Dense and scannable — fits the hardware aesthetic.
- **D-02:** Always-visible search/filter bar above the project table. Text search filters by name; dropdown filters for BPM range and date. Instant filtering, no separate search page. Backed by SQLite index (MGMT-04).
- **D-03:** Clicking a project replaces the list with a project detail view. Breadcrumb trail at top (Projects › LIVESET_01 › Bank 03) for navigation back. Linear flow mirrors how you'd navigate on the OT itself.
- **D-04:** Project detail view uses tabbed sections: Banks, Samples, Health. Each tab is a distinct concern with its own presentation.

### Project Detail Structure
- **D-05:** Bank drill-down goes to banks → parts → tracks depth. Patterns are shown as a populated/empty indicator grid (dots) but are not individually expandable — pattern data is mostly sequencer info which is out of scope.
- **D-06:** Banks displayed as a 4×4 grid matching the OT's own 16-bank layout. Filled dot for populated banks, empty dot for unused. Click a bank to drill into its 4 parts and 8 tracks per part with machine type and assigned sample.
- **D-07:** Compact metadata header always visible below breadcrumb: project name, tempo, bank count, last modified. Detailed metadata (bank names, part names, machine types) shown contextually when drilling into specific banks.

### Sample Slot Display
- **D-08:** Flex and Static samples displayed as two separate table sections on the Samples tab. Empty slots hidden by default with a "show all" toggle. Most projects use 10–40 of 256 slots, so hiding empties keeps it scannable.
- **D-09:** Each sample slot row shows: slot number (#001–#128), filename (truncated if long), sample rate (44.1k/48k), and a status icon (✓ OK, ✘ missing, ⚠ format issue).
- **D-10:** Click/expand a slot row to see which banks, parts, and tracks reference that sample. Not visible by default — keeps the list clean but cross-reference info is there when needed.

### Health Check UX
- **D-11:** Health check runs automatically in the background when a project is opened. Results populate the Health tab badge count and inline status icons on the Samples tab. No manual trigger needed.
- **D-12:** Three severity tiers: Error (missing files — project won't play correctly), Warning (wrong format — 48kHz, wrong bit depth, non-WAV/AIFF), Info (unused samples — assigned to slot but not referenced by any track). Grouped by severity with counts.
- **D-13:** Health issues appear in two places: inline status icons (✓ ✘ ⚠) next to each slot on the Samples tab, and a full grouped detail view on the dedicated Health tab with descriptions and file paths.
- **D-14:** Healthy project state shows a calm "All clear" message with timestamp — no issues found. Matches the Phase 1 "no device" empty state approach: reassuring, not noisy.

### Claude's Discretion
- Exact table column widths and truncation behavior for long filenames
- How the "show all" toggle for empty slots works (button, switch, or link)
- Breadcrumb styling and back-navigation interaction
- Tab styling (underline, pill, segmented control)
- Health check loading state while background scan is in progress
- Sort order defaults for project list and sample slot tables
- Whether bank grid cells show additional info on hover
- Pattern grid dot layout within a bank detail view
- Animation/transition behavior when navigating between views

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project-Level
- `.planning/PROJECT.md` — Core value, constraints, key decisions, prior art analysis
- `.planning/REQUIREMENTS.md` — Full v1 requirement list with traceability (BROW-02 through BROW-05, DETC-01 through DETC-03, MGMT-04 map to this phase)
- `.planning/ROADMAP.md` — Phase 2 success criteria and dependency on Phase 1

### Phase 1 Context (predecessor)
- `.planning/phases/01-foundation/01-CONTEXT.md` — Visual identity decisions (warm dark palette, monospace typography, sidebar nav, volume detection UX) that Phase 2 must follow

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- None yet — greenfield project. Phase 1 (Foundation) establishes the app scaffold, parser crate, and UI patterns that Phase 2 builds on.

### Established Patterns
- Wallflower (sister project) uses the same Tauri v2 + React/Next.js + tauri-specta stack — patterns from Wallflower should inform Phase 2 component architecture
- Phase 1 establishes: warm dark palette, monospace-forward typography, sidebar navigation, breadcrumb-style drill-down, calm empty states

### Integration Points
- `ot-parser` crate (Phase 1) — Phase 2 calls parser to read all project/bank/sample data. Parser is the data source; Phase 2 is the presentation layer.
- SQLite database (Phase 1) — Phase 2 populates and queries the project index for search/filter functionality (MGMT-04)
- Tauri commands + tauri-specta — IPC bridge between Rust parser/DB and React frontend with auto-generated TypeScript types
- Volume detection (Phase 1) — Phase 2's project list activates when an OT volume is detected; shows "no device" state otherwise

</code_context>

<specifics>
## Specific Ideas

- Bank grid should mirror the OT's own 4×4 layout mental model — users think in bank numbers 1–16 arranged in a grid, not a list
- Status icons on the Samples tab serve as a quick visual scan — you should be able to glance and see if anything is wrong without switching to the Health tab
- The "hide empty slots" default is critical — most OT projects use a fraction of the 256 available slots, and showing all 256 would bury the useful data
- Health check "all clear" state should feel like the app is watching your back, not like the absence of a feature

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 02-read-only-browser*
*Context gathered: 2026-04-29*
