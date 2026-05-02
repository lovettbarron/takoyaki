# Takoyaki

## What This Is

Takoyaki is a desktop backup, versioning, and file management tool for the Elektron Octatrack. It lets musicians safely browse, back up, version, and manage their Octatrack projects and samples from a Mac — with a clean-room Rust parser for OT binary formats and a three-layer safety model that protects creative work from data loss or corruption. Optional integration with the Wallflower sample library enables metadata-powered sample search and one-click deployment to OT slots.

## Core Value

An Octatrack user can manage their projects and samples with complete confidence that their creative work is never at risk — every destructive operation is snapshot-protected, previewed, and atomically applied.

## Requirements

### Validated

- [x] Browse OT project structure on Mac (samples, banks, parts, patterns) — Validated in Phase 2: Read-Only Browser
- [x] Safe backup and versioning of OT projects (snapshot, revert, track evolution) — Validated in Phase 3: Write Path and Backup
- [x] Three-layer safety model: auto-snapshot before writes, dry-run preview, atomic staged writes — Validated in Phase 3: Write Path and Backup
- [x] Clean-room Rust parser for OT binary formats (.work, .strd, .ot, bank files, marker files) — Validated in Phase 1: Foundation

### Active

- [ ] Assign samples to OT track slots from desktop UI
- [ ] Assign samples to OT track slots from desktop UI
- [x] Move, copy, duplicate, and archive OT projects on the CF card — Validated in Phase 4: Advanced Management
- [ ] Clean-room Rust parser for OT binary formats (.work, .strd, .ot, bank files, marker files)
- [ ] USB mode support — read/write OT when mounted as USB disk
- [ ] Optional Wallflower integration — read Wallflower SQLite DB for metadata search (key, BPM, tags)
- [ ] Search and preview samples from Wallflower library, push to OT slots
- [ ] MIT-licensed, built for the Octatrack community

### Out of Scope

- Pattern/sequencer editing — high complexity, deep format reverse engineering, defer to future
- Sample chain building — useful but not core to backup/management value
- Direct OT hardware communication (MIDI/SysEx control) — USB disk mode is sufficient
- OT MkI support — focus on MkII initially (same binary format family but avoids edge cases)
- Mobile app — desktop-first, Mac-first

## Context

**Ecosystem:** The Octatrack community has been underserved by tooling since OctaEdit was abandoned. Existing tools are either incomplete (OctaLib), narrowly focused (OctaChainer, OctaSplit), or unstable research projects (ot-tools-io). There is strong demand on Elektronauts for a reliable project management tool.

**Prior art (key findings from research):**
- **ot-tools-io** (Rust, GPL v3) — Most complete reverse engineering of OT binary formats. Reads/writes .work, .strd, bank files, marker files, sample settings. Key insight: moving a single sample slot requires changing up to 18 files across project, markers, and all bank files. Described as unstable/learning project.
- **OctaChainer** (C++/Qt, public domain) — Chains audio files into .wav + generates .ot slice metadata. Solves sample chain workflow only.
- **ot_utils** (Rust) — Library for concatenating samples and generating .ot slice files. Scriptable sample chain creation.
- **OctaLib** (C#, 2024) — Basic librarian — viewing projects, swapping banks. Phase 1 partially complete.
- **OctArranger/OctaSplit** — Arrangement editing and sample slicing GUIs.
- **OctaEdit** (commercial, dead) — 13-module feature-complete editor/librarian. Never open source. Abandoned.

**Wallflower relationship:** Wallflower is a local-first jam and sample manager that records, imports, analyzes (source separation, key/BPM/tag detection), and organizes musical material. Takoyaki is the downstream consumer — musicians capture and analyze in Wallflower, then deploy to the Octatrack via Takoyaki. The Wallflower integration is optional; Takoyaki must stand alone for OT users who don't use Wallflower.

**Binary format challenge:** The OT stores project data across multiple interdependent binary files. A single logical operation (e.g., assigning a sample to a slot) can require coordinated writes across project files, bank files (.work), and marker files. This is the primary technical risk and the reason the safety model is non-negotiable.

## Constraints

- **Tech stack**: Tauri v2 native macOS app with Rust backend + React/Next.js frontend (same architecture as Wallflower)
- **Database**: SQLite for Takoyaki's own metadata (backup history, project index, sample assignments)
- **OT format**: Clean-room Rust implementation — no GPL dependencies from ot-tools-io. Use community-documented format specs and independent reverse engineering.
- **Data safety**: Atomic writes, snapshot-before-write, dry-run preview for ALL operations that modify OT project files. No exceptions.
- **File access**: USB disk mode only — OT mounted as a volume on Mac
- **Licensing**: MIT for all project code. No GPL dependencies in core.
- **Wallflower coupling**: Read-only access to Wallflower's SQLite DB. No write dependency. Wallflower integration is a feature, not a requirement.
- **Testing**: Full test coverage. OT binary parser must have extensive test fixtures from real OT project files.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Clean-room OT parser (not fork ot-tools-io) | MIT license compatibility; ot-tools-io is GPL v3 and self-described as unstable | — Pending |
| Same stack as Wallflower (Tauri/Rust/React/Next.js) | Consistent ecosystem, shared learnings, reuse architectural patterns | — Pending |
| Three-layer safety model | OT project corruption is unacceptable; belt-and-suspenders approach justified | — Pending |
| Wallflower integration as optional feature | Target audience is all OT users, not just Wallflower users | — Pending |
| USB disk mode only (no MIDI/SysEx) | Simpler, safer, covers all file management use cases | — Pending |
| Read-only Wallflower DB access | Loose coupling; Wallflower doesn't need to know about Takoyaki | — Pending |
| MIT license | Community-friendly, matches Wallflower, avoids GPL contamination | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

Last updated: 2026-05-02 (Phase 4 complete — duplicate, rename, export, bank copy with conflict resolution)

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-05-02 after Phase 4 completion*
