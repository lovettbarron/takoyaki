# Phase 3: Write Path and Backup - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-30
**Phase:** 03-write-path-and-backup
**Areas discussed:** Backup destination & organization, Snapshot history & timeline UX, Dry-run preview presentation, Restore workflow & safety

---

## Backup Destination & Organization

| Option | Description | Selected |
|--------|-------------|----------|
| App-managed directory | Takoyaki manages a dedicated backup folder | |
| User-chosen directory | User picks any folder via native file picker | |
| Both — app default with override | Starts with app-managed default, settings lets you change | |

**User's choice:** Custom — `~/takoyaki/` as top-level directory, following Wallflower's convention
**Notes:** User specifically referenced Wallflower's approach of using a top-level home directory folder

---

| Option | Description | Selected |
|--------|-------------|----------|
| By project, then by date | ~/takoyaki/backups/PROJECT/DATE_LABEL/ | ✓ |
| By date, then by project | ~/takoyaki/backups/DATE/PROJECT/ | |
| Flat with metadata | ~/takoyaki/backups/PROJECT_DATE/ | |

**User's choice:** By project, then by date
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| Full project directory | Everything including AUDIO folder with samples | ✓ |
| Binary files only, samples referenced | .work, .strd, bank files, markers only | |
| User chooses per backup | Offer full and project-files-only each time | |

**User's choice:** Full project directory
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| Manual only | User clicks "Back Up" explicitly | ✓ |
| Auto on connect + manual | Auto backup all projects on OT mount | |
| Scheduled + manual | User sets backup schedule | |

**User's choice:** Manual only
**Notes:** None

---

## Snapshot History & Timeline UX

| Option | Description | Selected |
|--------|-------------|----------|
| Chronological list per project | Reverse-chrono, timestamp + label + count + size | ✓ |
| Global timeline across all projects | Single chrono view of all snapshots | |
| Calendar/grid view | Visual calendar with clickable days | |

**User's choice:** Chronological list per project
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| File listing with change summary | Added/modified/removed/unchanged indicators + Restore button | ✓ |
| Summary card only | Just metadata and Restore button | |
| Side-by-side diff view | Full comparison with binary-level changes | |

**User's choice:** File listing with change summary
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| Sidebar section | Top-level sidebar, works when disconnected | ✓ |
| Tab in project detail | Tab alongside Banks, Samples, Health | |
| Both — sidebar + project tab | Global sidebar + quick-access in project detail | |

**User's choice:** Sidebar section
**Notes:** Aligns with Phase 1 D-07 which already planned a Backups sidebar slot

---

## Dry-Run Preview Presentation

| Option | Description | Selected |
|--------|-------------|----------|
| Modal confirmation with file change list | Operation summary + file list + Apply/Cancel | ✓ |
| Inline preview panel | Expandable panel in current view | |
| Dedicated preview page | Full-page preview with detailed breakdown | |

**User's choice:** Modal confirmation with file change list
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| Always mandatory | Every destructive op shows preview, no skip | ✓ |
| Mandatory with quick-apply shortcut | Preview shows but keyboard shortcut to fast-apply | |
| Optional per-operation type | User can disable per operation type in Settings | |

**User's choice:** Always mandatory
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| Explicit mention | Modal shows "A snapshot will be created before applying" | ✓ |
| Silent — just do it | Snapshot happens but modal doesn't mention it | |

**User's choice:** Explicit mention
**Notes:** Builds trust in the safety model

---

## Restore Workflow & Safety

| Option | Description | Selected |
|--------|-------------|----------|
| Always snapshot before restore | Pre-restore snapshot automatic, can undo restore | ✓ |
| Ask each time | Modal asks whether to snapshot first | |
| No pre-restore snapshot | Restore directly | |

**User's choice:** Always snapshot before restore
**Notes:** Safety net for the safety net. Consistent with Phase 1 snapshot-before-write guarantee.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Abort cleanly, show status | Atomic write prevents damage; staging cleaned up on next launch | ✓ |
| Resume on reconnect | Detect reconnect, offer to resume | |
| Keep partial + warn | Keep partial writes, let user decide | |

**User's choice:** Abort cleanly, show status
**Notes:** Atomic write engine guarantees project files untouched on mid-restore disconnect. Partial backups deleted on mid-backup disconnect.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Inline success banner | Brief banner at top, auto-dismisses | ✓ |
| Toast notification | OS-level toast | |
| Success modal with details | Modal with full summary, click to dismiss | |

**User's choice:** Inline success banner
**Notes:** Calm, informative, auto-dismisses. No modal for success.

---

## Claude's Discretion

- Progress indicator style during long operations
- Checksum verification UX (SAFE-02)
- Backup deletion/cleanup UX
- Snapshot retention policy
- Banner styling and auto-dismiss timing
- SQLite schema for backup history
- Backup button placement in Projects view

## Deferred Ideas

None — discussion stayed within phase scope
