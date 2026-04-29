# Phase 2: Read-Only Browser - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-29
**Phase:** 02-read-only-browser
**Areas discussed:** Project list & navigation, Project detail structure, Sample slot display, Health check UX

---

## Project List & Navigation

| Option | Description | Selected |
|--------|-------------|----------|
| Compact table | Dense rows with columns for name, tempo, bank count, last modified | ✓ |
| Cards | Larger tiles per project with more visual space | |
| You decide | Claude picks layout | |

**User's choice:** Compact table
**Notes:** Fits monospace aesthetic, scannable with many projects

---

| Option | Description | Selected |
|--------|-------------|----------|
| Replace view with breadcrumb | Click project replaces list, breadcrumb for back nav | ✓ |
| Side panel / split view | Project list stays visible, detail in right panel | |
| You decide | Claude picks navigation pattern | |

**User's choice:** Replace view with breadcrumb
**Notes:** Linear flow mirrors OT navigation

---

| Option | Description | Selected |
|--------|-------------|----------|
| Tabbed sections | Banks, Samples, Health as tabs | ✓ |
| Single scrolling page | All info on one long page | |
| Dashboard layout | Grid of panels showing all simultaneously | |

**User's choice:** Tabbed sections (Banks, Samples, Health)
**Notes:** Compartmentalized concerns

---

| Option | Description | Selected |
|--------|-------------|----------|
| Search bar at top of project list | Always-visible search/filter with dropdowns | ✓ |
| Keyboard shortcut filter | Hidden by default, Cmd+F to activate | |
| You decide | Claude picks based on typical project counts | |

**User's choice:** Search bar at top of project list
**Notes:** Always visible, instant filtering

---

## Project Detail Structure

| Option | Description | Selected |
|--------|-------------|----------|
| Banks → parts → tracks | Banks as grid, click to see parts/tracks. Patterns shown as populated/empty dots | ✓ |
| Banks only (flat) | High-level bank list, no drill-down | |
| Full hierarchy (expandable tree) | Nested tree Bank → Pattern → Part → Track | |

**User's choice:** Banks → parts → tracks
**Notes:** Patterns noted as populated/empty but not individually expandable (sequencer data is out of scope)

---

| Option | Description | Selected |
|--------|-------------|----------|
| 4×4 grid with status indicators | 16 banks in grid matching OT layout | ✓ |
| Numbered list | Simple list with details per row | |
| You decide | Claude picks grid layout | |

**User's choice:** 4×4 grid with status indicators
**Notes:** Matches OT's own 16-bank layout mental model

---

| Option | Description | Selected |
|--------|-------------|----------|
| Compact header, always visible | Slim bar with name, tempo, bank count, date | ✓ |
| Expandable detail panel | Full metadata, collapsible | |
| You decide | Claude picks metadata presentation | |

**User's choice:** Compact header, always visible
**Notes:** Detailed metadata (bank names, part names) shown contextually when drilling into banks

---

## Sample Slot Display

| Option | Description | Selected |
|--------|-------------|----------|
| Two tables, hide empty | Flex and Static as separate sections, empty hidden by default | ✓ |
| Unified list, type column | All 256 in one table with Flex/Static column | |
| Visual grid (slot numbers) | 128-cell grid for each type | |

**User's choice:** Two tables, hide empty
**Notes:** Most projects use 10–40 of 256 slots; hiding empties keeps it scannable

---

| Option | Description | Selected |
|--------|-------------|----------|
| Slot #, filename, sample rate, file status | Slot number, filename, sample rate, status icon | ✓ |
| Slot # and filename only | Minimal info per row | |
| Full detail (path, size, format, duration) | Everything shown per row | |

**User's choice:** Slot #, filename, sample rate, file status
**Notes:** Status icons (✓ ✘ ⚠) provide quick visual scan for problems

---

| Option | Description | Selected |
|--------|-------------|----------|
| Show on click/expand | Click slot to see bank/part/track references | ✓ |
| Always show reference count | Column with reference count | |
| No cross-references | Only visible when drilling into banks | |

**User's choice:** Show on click/expand
**Notes:** Keeps list clean but cross-reference info available when needed

---

## Health Check UX

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-run on project open | Background health check, results populate tab and inline icons | ✓ |
| Manual (button/action) | User triggers health check explicitly | |
| You decide | Claude picks trigger model | |

**User's choice:** Auto-run on project open
**Notes:** Badge on Health tab shows issue count automatically

---

| Option | Description | Selected |
|--------|-------------|----------|
| Three severity tiers | Error (missing), Warning (format), Info (unused) grouped by severity | ✓ |
| Flat list, all equal | All issues in one list, no severity grouping | |
| You decide | Claude picks severity model | |

**User's choice:** Three severity tiers
**Notes:** Error/Warning/Info with counts per tier

---

| Option | Description | Selected |
|--------|-------------|----------|
| Both — inline icons + Health tab | Status icons on Samples tab AND full detail on Health tab | ✓ |
| Health tab only | Samples tab stays clean, diagnostics only on Health tab | |
| You decide | Claude picks inline vs dedicated | |

**User's choice:** Both — inline icons + Health tab
**Notes:** Users see problems wherever they are without tab switching

---

| Option | Description | Selected |
|--------|-------------|----------|
| Clean confirmation | "All clear" message with timestamp | ✓ |
| Summary stats | Show slot counts and check details even when healthy | |
| You decide | Claude picks healthy state presentation | |

**User's choice:** Clean confirmation
**Notes:** Calm, reassuring — matches Phase 1 "no device" empty state vibe

---

## Claude's Discretion

- Table column widths and truncation behavior
- "Show all" toggle design for empty slots
- Breadcrumb styling and back-navigation interaction
- Tab styling (underline, pill, segmented control)
- Health check loading state
- Sort order defaults
- Bank grid hover behavior
- Pattern grid dot layout
- View transition animations

## Deferred Ideas

None — discussion stayed within phase scope
