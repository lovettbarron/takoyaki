# Roadmap: Takoyaki

## Overview

Takoyaki ships in five phases ordered by technical risk and dependency. The OT binary parser is built first — in isolation, with byte-exact round-trip tests — because every other feature depends on it being correct. Read-only browsing ships next, stress-testing the parser against real project diversity without write risk. The write path (backup, snapshot, atomic writes) follows once the parser is field-proven. Advanced management operations (export, bank copy, rename) layer on top of the proven write engine. Sample assignment from desktop and the optional Wallflower integration ship last, after the entire stack is stable — they are the most complex and depend on all prior pieces.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Foundation** - Parser, safety infrastructure, app scaffold, and USB volume detection
- [ ] **Phase 2: Read-Only Browser** - Complete read-only visibility into OT projects, slots, and samples
- [ ] **Phase 3: Write Path and Backup** - Backup, snapshot history, dry-run preview, and atomic write engine
- [ ] **Phase 4: Advanced Management** - Export, unused sample detection, bank copy, project rename
- [ ] **Phase 5: Sample Assignment and Wallflower** - Desktop-to-slot assignment and optional Wallflower library integration

## Phase Details

### Phase 1: Foundation
**Goal**: A Tauri app exists that can detect a mounted OT volume, parse all OT binary file types with byte-exact fidelity, and safely write any file atomically — with snapshot infrastructure in place before any user-facing write operation is ever built.
**Depends on**: Nothing (first phase)
**Requirements**: FNDN-01, FNDN-02, FNDN-03, FNDN-04, FNDN-05, FNDN-06, FNDN-07, FNDN-08, SAFE-03, SAFE-04, BROW-01
**Success Criteria** (what must be TRUE):
  1. The app launches on macOS and shows whether an Octatrack is connected in USB disk mode
  2. The OT parser reads every file type (.work, .strd, .ot, bank files, marker files) and round-trips them byte-for-exactly: `parse(serialize(parse(bytes))) == parse(bytes)` passes against a corpus of real OT project files
  3. Writing any OT file through the atomic write engine stages to a temp dir on the same volume, fsyncs, then renames — verified by integration test on a real FAT32 volume
  4. A snapshot of all affected files is automatically created before any write operation is committed — verified by test that confirms snapshot exists before file changes land
  5. The SQLite database initializes with schema for backup history, project index, and snapshot records
**Plans:** 7 plans
Plans:
- [x] 01-01-PLAN.md — Cargo workspace and Tauri app crate scaffold
- [x] 01-02-PLAN.md — Next.js frontend scaffold with shadcn and warm dark theme
- [x] 01-03-PLAN.md — .ot file parser with round-trip tests and indexing newtypes
- [x] 01-04-PLAN.md — Remaining OT file type parsers (.work, bank, markers, arrangement)
- [x] 01-05-PLAN.md — SQLite database, atomic write engine, and snapshot infrastructure
- [x] 01-06-PLAN.md — Frontend UI shell with sidebar navigation and disconnected state
- [x] 01-07-PLAN.md — Volume detection backend and frontend integration with confirmation dialog
**UI hint**: yes

### Phase 2: Read-Only Browser
**Goal**: Users can browse every OT project on a mounted card in full detail — projects, banks, patterns, sample slots, and metadata — and detect problems in their project files, all without touching a single byte on the CF card.
**Depends on**: Phase 1
**Requirements**: BROW-02, BROW-03, BROW-04, BROW-05, DETC-01, DETC-02, DETC-03, MGMT-04
**Success Criteria** (what must be TRUE):
  1. User can see a list of all OT projects on the mounted card with name, bank count, tempo, and last-modified date
  2. User can open a project and see which banks and patterns are populated
  3. User can view all 128 Flex and 128 Static sample slots with the assigned file path for each occupied slot
  4. User can see project-level metadata: tempo, bank names, part names, and active machine types per track
  5. User can run a health check on any project that flags missing sample references, incompatible audio formats (non-44.1kHz, wrong bit depth, non-WAV/AIFF), and unused samples
  6. User can search and filter projects by name, tempo, or date using the SQLite index
**Plans:** 5 plans
Plans:
- [x] 02-01-PLAN.md — Rust Tauri commands for project list, detail, banks, samples, and SQLite indexing
- [x] 02-02-PLAN.md — Rust health check engine with audio format validation and background event emission
- [x] 02-03-PLAN.md — Frontend TypeScript types, IPC wrappers, navigation store, and project list view
- [x] 02-04-PLAN.md — Frontend project detail view with bank grid, samples tab, and metadata header
- [x] 02-05-PLAN.md — Frontend health tab, event listener, and end-to-end visual verification
**UI hint**: yes

### Phase 3: Write Path and Backup
**Goal**: Users can back up projects, browse snapshot history, restore any prior state, and preview exactly what will change before any destructive operation is committed — with every write going through the atomic staged-write engine.
**Depends on**: Phase 2
**Requirements**: SAFE-01, SAFE-02, SAFE-05, SAFE-06, SAFE-07
**Success Criteria** (what must be TRUE):
  1. User can back up any OT project to a Mac-side location and verify it via checksum comparison between source and backup
  2. User can browse a chronological snapshot history for any project with timestamps and operation labels
  3. User can restore any snapshot to roll a project back to exactly that prior state
  4. User can trigger dry-run mode on any destructive operation and see exactly which files will change and how before committing
  5. A backup or restore that is interrupted mid-operation leaves the project in its pre-operation state (all-or-nothing guarantee)
**Plans:** 4 plans
Plans:
- [x] 03-01-PLAN.md — Rust backend: V2 migration, db::backups module, and five Tauri backup commands
- [x] 03-02-PLAN.md — Frontend foundation: TypeScript types, backup store, IPC wrappers, sidebar activation
- [x] 03-03-PLAN.md — Dry-run modal, backup progress view, success banner, and MetadataHeader Back Up button
- [x] 03-04-PLAN.md — Backups view with timeline, snapshot detail panel, and restore workflow
**UI hint**: yes

### Phase 4: Advanced Management
**Goal**: Users can perform the full range of project management operations — duplicate, rename, export, copy banks across projects — with the same safety guarantees as Phase 3.
**Depends on**: Phase 3
**Requirements**: MGMT-01, MGMT-02, MGMT-03, SMPL-02
**Success Criteria** (what must be TRUE):
  1. User can duplicate an OT project and have all sample paths correctly remapped to the new project directory
  2. User can rename an OT project directory on disk; no binary header modification is required as the directory name is the sole authoritative project name in OT
  3. User can export a project as a self-contained zip with all referenced audio samples collected inside
  4. User can copy a bank from one project to another with sample slots automatically remapped and conflicts surfaced for resolution
**Plans:** 7 plans
Plans:
- [ ] 04-01-PLAN.md — Rust management module: project.work parser, OT name validation, rename, duplicate
- [ ] 04-02-PLAN.md — Frontend foundation: TypeScript types, management store, IPC wrappers, context-menu install
- [ ] 04-03-PLAN.md — Rust export-to-zip, bank copy with conflict detection, Tauri command registration
- [ ] 04-04-PLAN.md — Frontend UI: MetadataHeader toolbar, inline rename, BankCopyPickerDialog, page.tsx wiring
- [ ] 04-05-PLAN.md — End-to-end integration verification checkpoint
- [ ] 04-06-PLAN.md — [gap closure] Correct ROADMAP SC-2 and REQUIREMENTS MGMT-02 wording
- [ ] 04-07-PLAN.md — [gap closure] Conflict resolution UI for bank copy flow
**UI hint**: yes

### Phase 5: Sample Assignment and Wallflower
**Goal**: Users can assign any desktop audio file to a specific Flex or Static sample slot with all affected OT binary files updated atomically, and optionally browse and deploy samples from the Wallflower library — with graceful degradation when Wallflower is not present.
**Depends on**: Phase 4
**Requirements**: SMPL-01, SMPL-03, INTG-01, INTG-02, INTG-03
**Success Criteria** (what must be TRUE):
  1. User can assign a desktop audio file to a specific Flex or Static slot and have all affected binary files (up to 18) updated atomically in a single transaction with a pre-write snapshot
  2. The app validates Flex vs Static slot type correctness before assigning and blocks incompatible assignments with a clear error
  3. User can search the Wallflower sample library by key, BPM, and tags from within Takoyaki and push a selected sample to an OT slot
  4. When Wallflower is not installed or its database is unavailable, the Wallflower panel is hidden or shows a graceful "not connected" state — no crash, no error dialog
**Plans**: TBD
**UI hint**: yes

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation | 6/7 | Executing | - |
| 2. Read-Only Browser | 0/5 | Planning complete | - |
| 3. Write Path and Backup | 0/4 | Planning complete | - |
| 4. Advanced Management | 0/7 | Planning complete | - |
| 5. Sample Assignment and Wallflower | 0/? | Not started | - |
