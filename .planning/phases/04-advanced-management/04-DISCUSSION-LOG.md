# Phase 4: Advanced Management - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-30
**Phase:** 04-advanced-management
**Areas discussed:** Duplication & sample handling, Export packaging, Bank copy conflict resolution, Management actions UX

---

## Duplication & Sample Handling

| Option | Description | Selected |
|--------|-------------|----------|
| Full copy (Recommended) | Copy all referenced audio files into new project directory. Independent, self-contained duplicate. | ✓ |
| Shared references | New project references same audio files. No extra disk space but coupled to original. | |
| User chooses per-duplicate | Show choice in dry-run preview each time. | |

**User's choice:** Full copy
**Notes:** None — clear preference for independent duplicates.

### Naming

| Option | Description | Selected |
|--------|-------------|----------|
| Suffix with _copy | LIVESET_01 → LIVESET_01_copy | ✓ |
| User names it upfront | Text input in dry-run preview dialog | |
| Numeric suffix | LIVESET_01 → LIVESET_02 | |

**User's choice:** Suffix with _copy

### Length Limit Handling

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-truncate with suffix | Truncate original name to fit _copy | |
| Fall back to user input | Prompt user to type a name that fits | ✓ |

**User's choice:** Fall back to user input

---

## Export Packaging

| Option | Description | Selected |
|--------|-------------|----------|
| Full project + samples (Recommended) | Zip with complete project dir AND all referenced audio files with OT directory structure | ✓ |
| Project files only | Only /SETS/PROJECT_NAME/ directory, no audio | |
| User chooses at export time | Toggle in export dialog | |

**User's choice:** Full project + samples

### Export Destination

| Option | Description | Selected |
|--------|-------------|----------|
| ~/takoyaki/exports/ (Recommended) | App-managed directory consistent with backup convention | ✓ |
| macOS save dialog | Standard file picker each time | |
| Default dir + save dialog option | Default location with option to choose elsewhere | |

**User's choice:** ~/takoyaki/exports/

### OT Sidecar Files

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, include .ot files (Recommended) | Include .ot sidecar for every referenced sample | ✓ |
| No, audio only | Just raw audio files | |

**User's choice:** Yes, include .ot files

---

## Bank Copy Conflict Resolution

### Missing Samples

| Option | Description | Selected |
|--------|-------------|----------|
| Copy samples automatically (Recommended) | Auto-copy missing samples, skip matching files (hash match) | ✓ |
| Surface all conflicts for review | Show every missing sample in dry-run preview for manual resolution | |
| Copy with conflict list | Auto-copy but surface hash mismatches for user resolution | |

**User's choice:** Copy samples automatically

### Hash Mismatch Handling

| Option | Description | Selected |
|--------|-------------|----------|
| Surface in dry-run preview (Recommended) | Show conflict with keep/overwrite/rename options | ✓ |
| Always rename incoming | Auto-rename conflicting file with suffix | |
| Always keep target | Never overwrite, bank may sound different | |

**User's choice:** Surface in dry-run preview

### Populated Target Bank Slot

| Option | Description | Selected |
|--------|-------------|----------|
| Warn and let user pick slot (Recommended) | Show warning, offer empty slots or explicit overwrite | ✓ |
| Always overwrite | Overwrite with snapshot protection | |
| Only allow empty slots | Block operation on populated slots | |

**User's choice:** Warn and let user pick slot

---

## Management Actions UX

### Action Placement

| Option | Description | Selected |
|--------|-------------|----------|
| Actions toolbar in project detail (Recommended) | Buttons in project detail header for Duplicate/Rename/Export; bank copy via right-click in bank grid | ✓ |
| Context menu on project list | Right-click project in list for actions | |
| Both toolbar + context menu | Actions in both places | |

**User's choice:** Actions toolbar in project detail

### Rename UX

| Option | Description | Selected |
|--------|-------------|----------|
| Inline edit in header (Recommended) | Click Rename makes name editable in-place, then dry-run preview | ✓ |
| Modal dialog | Dialog with text input and preview | |
| Double-click project name | Double-click to enter rename mode | |

**User's choice:** Inline edit in header

### Bank Copy Target Selection

| Option | Description | Selected |
|--------|-------------|----------|
| Two-step picker dialog (Recommended) | Step 1: pick project, Step 2: pick bank slot with 4×4 grid | ✓ |
| Drag-and-drop between projects | Split view with drag-and-drop | |
| Command palette style | Searchable dropdown with slot picker | |

**User's choice:** Two-step picker dialog

---

## Claude's Discretion

- Toolbar button styling and icons
- Context menu implementation approach
- Export progress indicator
- Bank copy picker dialog styling
- Rename character validation
- Zip compression level

## Deferred Ideas

None — discussion stayed within phase scope
