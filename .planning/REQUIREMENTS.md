# Requirements: Takoyaki

**Defined:** 2026-04-29
**Core Value:** An Octatrack user can manage their projects and samples with complete confidence that their creative work is never at risk — every destructive operation is snapshot-protected, previewed, and atomically applied.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Browsing & Visualization

- [ ] **BROW-01**: User can see when their Octatrack is connected in USB disk mode via automatic volume detection
- [ ] **BROW-02**: User can list all OT projects on a mounted card with metadata (name, bank count, tempo, last modified)
- [ ] **BROW-03**: User can view which banks and patterns are populated within a project
- [ ] **BROW-04**: User can view all Flex and Static sample slots (128 each) with assigned file paths
- [ ] **BROW-05**: User can view project-level metadata including tempo, bank names, part names, and active machine types per track

### Detection & Validation

- [ ] **DETC-01**: User can detect missing or broken sample references across all slots in a project
- [ ] **DETC-02**: User can validate audio file format compatibility (flag non-44.1kHz, wrong bit depth, non-WAV/AIFF samples)
- [ ] **DETC-03**: User can detect unused samples (assigned to slots but never triggered in any pattern)

### Backup & Data Safety

- [ ] **SAFE-01**: User can back up any OT project to a Mac-side location (full directory tree copy)
- [ ] **SAFE-02**: User can verify backup integrity via checksum comparison between source and backup
- [ ] **SAFE-03**: System automatically creates a snapshot of all affected files before any write operation
- [ ] **SAFE-04**: All write operations use atomic staged writes (write to staging, verify, then rename — all-or-nothing)
- [ ] **SAFE-05**: User can browse snapshot history chronologically with timestamps and operation labels
- [ ] **SAFE-06**: User can restore any previous snapshot to roll back a project to a prior state
- [ ] **SAFE-07**: User can preview exactly what files will change before any destructive operation is committed (dry-run mode)

### Project Management

- [ ] **MGMT-01**: User can duplicate/copy an OT project with automatic sample path remapping
- [ ] **MGMT-02**: User can rename an OT project on disk with internal name field updated
- [ ] **MGMT-03**: User can export a project as a self-contained zip with all referenced samples collected
- [ ] **MGMT-04**: User can search and filter projects by name, tempo, or date via indexed metadata

### Sample Management

- [ ] **SMPL-01**: User can assign a desktop audio file to a specific Flex or Static sample slot with all affected binary files updated atomically
- [ ] **SMPL-02**: User can copy banks between projects with automatic sample slot remapping and conflict resolution
- [ ] **SMPL-03**: System validates Flex vs Static slot type correctness when assigning samples

### Wallflower Integration

- [ ] **INTG-01**: User can search Wallflower sample library by key, BPM, tags, and other metadata from within Takoyaki
- [ ] **INTG-02**: User can preview sample metadata from Wallflower and push selected samples to OT slots
- [ ] **INTG-03**: Wallflower integration degrades gracefully when Wallflower is not installed or its database is unavailable

### Foundation (Non-Functional)

- [ ] **FNDN-01**: Clean-room Rust parser for OT binary formats (.work, .strd, .ot, bank files, marker files) with no GPL dependencies
- [ ] **FNDN-02**: Parser preserves all unknown/reserved bytes verbatim during round-trip (parse → serialize → parse produces identical output)
- [ ] **FNDN-03**: Parser uses correct indexing (1-indexed for project files, 0-indexed for bank/marker files) with Rust newtypes preventing mismatch
- [ ] **FNDN-04**: Staging directory for atomic writes lives on the same filesystem as the CF card volume
- [ ] **FNDN-05**: All write completions are gated on fsync + directory sync to protect against hot-unplug data loss
- [ ] **FNDN-06**: Tauri v2 desktop app with Rust backend and React/Next.js frontend, consistent with Wallflower architecture
- [ ] **FNDN-07**: SQLite database for Takoyaki's own metadata (backup history, project index, snapshot records)
- [ ] **FNDN-08**: Read-only SQLite connection to Wallflower database with driver-level write protection

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Extended Features

- **V2-01**: Sample audio preview with playback transport within Takoyaki (requires audio engine)
- **V2-02**: Windows platform support
- **V2-03**: OT MkI format support (binary format differences from MkII)
- **V2-04**: Arrangement file editing and visualization
- **V2-05**: Local backup to NAS or external drive with path configuration

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Pattern/sequencer editing | Most complex and least documented part of OT format; separate product scope; OctaStudio is building this |
| Direct MIDI/SysEx hardware control | Different architecture; USB disk mode covers all file management use cases |
| Sample editing (trim/normalize/transcode) | Out of scope for backup/management tool; DigiChain and other tools handle this |
| Sample chain building | Useful but not core to backup/management value; OctaChainer/DigiChain exist |
| Cloud sync / remote backup | Introduces auth, privacy, cloud dependencies; local-first is the right default |
| Real-time monitoring / auto-backup on save | OT doesn't signal saves; polling during OT use is dangerous; manual backup on USB mount is safe |
| Mobile app | Desktop-first, Mac-first |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| BROW-01 | — | Pending |
| BROW-02 | — | Pending |
| BROW-03 | — | Pending |
| BROW-04 | — | Pending |
| BROW-05 | — | Pending |
| DETC-01 | — | Pending |
| DETC-02 | — | Pending |
| DETC-03 | — | Pending |
| SAFE-01 | — | Pending |
| SAFE-02 | — | Pending |
| SAFE-03 | — | Pending |
| SAFE-04 | — | Pending |
| SAFE-05 | — | Pending |
| SAFE-06 | — | Pending |
| SAFE-07 | — | Pending |
| MGMT-01 | — | Pending |
| MGMT-02 | — | Pending |
| MGMT-03 | — | Pending |
| MGMT-04 | — | Pending |
| SMPL-01 | — | Pending |
| SMPL-02 | — | Pending |
| SMPL-03 | — | Pending |
| INTG-01 | — | Pending |
| INTG-02 | — | Pending |
| INTG-03 | — | Pending |
| FNDN-01 | — | Pending |
| FNDN-02 | — | Pending |
| FNDN-03 | — | Pending |
| FNDN-04 | — | Pending |
| FNDN-05 | — | Pending |
| FNDN-06 | — | Pending |
| FNDN-07 | — | Pending |
| FNDN-08 | — | Pending |

**Coverage:**
- v1 requirements: 33 total
- Mapped to phases: 0
- Unmapped: 33

---
*Requirements defined: 2026-04-29*
*Last updated: 2026-04-29 after initial definition*
