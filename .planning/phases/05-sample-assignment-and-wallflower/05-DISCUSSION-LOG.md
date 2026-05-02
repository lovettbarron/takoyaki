# Phase 5: Sample Assignment and Wallflower - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-02
**Phase:** 05-sample-assignment-and-wallflower
**Areas discussed:** Assignment workflow, Wallflower discovery, Wallflower browser UX, Slot validation UX

---

## Assignment Workflow

### How should the user initiate assigning a sample to a slot?

| Option | Description | Selected |
|--------|-------------|----------|
| Click slot → file picker | User clicks slot on SamplesTab, gets native macOS file picker | ✓ |
| Drag-and-drop onto slot | Drag audio file from Finder onto a slot row | |
| Both: click + drag-and-drop | Click as primary, drag as power-user shortcut | |

**User's choice:** Click slot → file picker
**Notes:** Simple, discoverable, reuses existing SlotRow component

### How should the dry-run preview present the multi-file complexity?

| Option | Description | Selected |
|--------|-------------|----------|
| Summary + expandable detail | Clear summary at top with expandable file list | ✓ |
| Summary only | Just operation summary and file count | |
| Full file list always | Always show every affected file, no collapse | |

**User's choice:** Summary + expandable detail
**Notes:** Matches Phase 3 dry-run modal pattern without overwhelming

### What should happen when the user clicks an already-occupied slot?

| Option | Description | Selected |
|--------|-------------|----------|
| Same flow: file picker to replace | Opens file picker, dry-run shows old → new replacement | ✓ |
| Context menu with options | Right-click for Replace, Clear, View details | |
| Assign button per slot row | Explicit button keeps expand and assign separate | |

**User's choice:** Same flow: file picker to replace
**Notes:** Dry-run preview IS the confirmation — no extra step needed

### How should we handle both expand and assign on the same row?

| Option | Description | Selected |
|--------|-------------|----------|
| Assign button on each row | Small [↑] button for assignment, click body to expand | ✓ |
| Click row = assign, expand via chevron | Repurpose click to assign, chevron for expand | |
| Double-click to assign | Single click expands, double-click assigns | |

**User's choice:** Assign button on each row
**Notes:** Preserves Phase 2 click-to-expand behavior; clear action separation

---

## Wallflower Discovery

### How should Takoyaki find the Wallflower database?

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-discover + Settings override | Check Settings path first, then auto-discover from known default | ✓ |
| User configures in Settings only | No auto-discovery, user must set path | |
| Auto-discover only | Only look in known default location | |

**User's choice:** Auto-discover + Settings override
**Notes:** Covers "just works" and non-standard installs

### What should the graceful degradation look like when Wallflower is not available?

| Option | Description | Selected |
|--------|-------------|----------|
| Hide Wallflower panel entirely | No error, no empty state — panel simply doesn't appear | ✓ |
| Greyed "not connected" state | Show greyed section with link to Settings | |
| Sidebar badge indicator | Small indicator, panel hidden when disconnected | |

**User's choice:** Hide Wallflower panel entirely
**Notes:** Clean and non-confusing for users who don't use Wallflower

---

## Wallflower Browser UX

### Where should the Wallflower library browser live within Takoyaki?

| Option | Description | Selected |
|--------|-------------|----------|
| Panel below SamplesTab slots | Collapsible panel below Flex/Static lists | ✓ |
| Separate Samples sidebar tab | Own top-level sidebar section | |
| Slide-over panel from right | Right-side overlay panel | |

**User's choice:** Panel below SamplesTab slots
**Notes:** Slots visible above while browsing below — short visual distance for push-to-slot

### How should the push to slot flow work from the Wallflower library panel?

| Option | Description | Selected |
|--------|-------------|----------|
| Click sample → pick target slot | Click sample, slot picker appears (type + slot dropdown) | ✓ |
| Drag sample to slot row above | Drag from Wallflower panel to slot row | |
| Select slot first, then browse | Click slot assign first, then choose source | |

**User's choice:** Click sample → pick target slot
**Notes:** Sample file copied from Wallflower location to OT /AUDIO/ as part of atomic write

### What metadata fields should the Wallflower library panel show for each sample?

| Option | Description | Selected |
|--------|-------------|----------|
| Filename, key, BPM, tags | Compact row with tags as small badges | ✓ |
| Filename, key, BPM only | Minimal three-field display | |
| Full metadata card | Expanded view with all metadata fields | |

**User's choice:** Filename, key, BPM, tags
**Notes:** Most useful for OT sample selection — picking sounds to perform with

---

## Slot Validation UX

### How should the app handle Flex vs Static slot type mismatches?

| Option | Description | Selected |
|--------|-------------|----------|
| Block with inline error + suggest correct type | Inline error with one-click redirect to correct slot type | ✓ |
| Block with modal error | Modal dialog with full mismatch details | |
| Warn but allow | Warning but let user proceed | |

**User's choice:** Block with inline error + suggest correct type
**Notes:** Turns error into one-click fix — [Assign to Static #003 instead?]

### How should audio format issues be handled during assignment?

| Option | Description | Selected |
|--------|-------------|----------|
| Block incompatible, warn on non-ideal | Hard block non-WAV/AIFF; soft warning for wrong rate/depth | ✓ |
| Block everything non-standard | Hard block on anything not 44.1kHz 16/24-bit WAV/AIFF | |
| Warn on everything, block nothing | Warnings only, always allow | |

**User's choice:** Block incompatible formats, warn on non-ideal
**Notes:** Hard block: MP3/FLAC/etc. Soft warning: 48kHz/32-bit shown in dry-run with [Assign Anyway]

---

## Claude's Discretion

- Exact assign button icon and styling on slot rows
- Wallflower search/filter UX details
- Wallflower panel collapse/expand animation and default state
- Slot picker dropdown styling in push-to-slot flow
- Wallflower auto-discovery exact default path
- Search result sorting

## Deferred Ideas

None — discussion stayed within phase scope
