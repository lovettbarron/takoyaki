# Takoyaki Progress Analysis and Proposed Next Steps

**Date:** 2026-05-05
**Scope:** Phases 1-5 complete, plus quick task 260502-qkl (audio preview)
**Codebase:** 9,375 lines Rust / 7,493 lines TypeScript/React / 139 passing tests

---

## 1. Current State Assessment

### 1.1 Built vs Planned (Phase-by-Phase)

| Phase | Goal | Status | Completeness |
|-------|------|--------|-------------|
| 1: Foundation | Parser, safety infra, app scaffold, USB detection | Complete | High (see parser caveats below) |
| 2: Read-Only Browser | Full read-only visibility into OT projects | Complete | High (sample slots are text-parse, not binary-parse) |
| 3: Write Path and Backup | Backup, snapshot, dry-run, atomic writes | Complete | High |
| 4: Advanced Management | Duplicate, rename, export, bank copy | Complete | High |
| 5: Sample Assignment and Wallflower | Desktop-to-slot assignment, Wallflower integration | Complete (code) | Human verification pending |

### 1.2 Feature Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| OT volume detection (USB disk mode) | Fully implemented | Polling every 2s, sysinfo + /Volumes fallback, multi-layout detection |
| .ot file parser (SampleSettingsFile) | Fully implemented | binrw, 832 bytes, byte-exact round-trip verified |
| Bank file parser (BankFile) | Opaque blob | Header + version + checksum parsed; body stored verbatim (D-02) |
| Markers file parser (MarkersFile) | Opaque blob | Header + version + checksum parsed; body stored verbatim (D-02) |
| Arrangement file parser (ArrangementFile) | Opaque blob | Header + version + checksum parsed; body stored verbatim (D-02) |
| Project file parser (ProjectFile) | Raw bytes | project.work is text-based (key=value), stored verbatim |
| project.work text parser | Implemented | TYPE=/SLOT=/PATH= extraction and rewrite (used by samples and management) |
| Atomic write engine | Fully implemented | Temp-to-rename, fsync, batch writes |
| Snapshot engine | Fully implemented | SHA-256 hashes, timestamped dirs, pre-write snapshots |
| SQLite database | Fully implemented | V1 (projects/snapshots), V2 (backups), V3 (wallflower settings) |
| Project list and detail | Fully implemented | List, filter, sort, bank grid, metadata header |
| Sample slot browsing | Partially implemented | Reads text format from project.work; returns real data for projects with text-format slots |
| Health check engine | Fully implemented | Audio format validation (WAV/AIFF, sample rate, bit depth), missing refs |
| Backup and restore | Fully implemented | Full project backup, snapshot history timeline, chronological restore |
| Dry-run preview | Fully implemented | FileChangeManifest before any destructive operation |
| Project duplicate | Fully implemented | Name validation, sample path remapping |
| Project rename | Fully implemented | OT name validation (16 char, A-Z/0-9/_) |
| Project export (zip) | Fully implemented | Self-contained zip with all referenced audio |
| Bank copy across projects | Fully implemented | Conflict detection, resolution UI |
| Sample assignment (desktop) | Fully implemented | File picker, dry-run, atomic write, snapshot |
| Wallflower integration | Fully implemented | Auto-discovery, search by key/BPM/tags, push-to-slot |
| Wallflower graceful degradation | Fully implemented | Panel hidden when DB unavailable |
| Audio preview (play/stop) | Fully implemented | rodio-based native playback, dedicated audio thread |

### 1.3 Architecture Summary

```
takoyaki/
  crates/
    ot-parser/        -- Clean-room binary format library (MIT, no I/O)
    takoyaki-app/     -- Tauri app crate
      src/
        commands/     -- 6 modules: device, projects, samples, backup, management, wallflower
        db/           -- SQLite (rusqlite): projects, backups, wallflower
        atomic/       -- Atomic writes + snapshot engine
        health/       -- Audio format validation
        management/   -- project_work parser, rename, duplicate, export, bank_copy
        device/       -- USB volume detection polling
  src/                -- Next.js 15 + React 19 frontend
    components/       -- 30+ components across project-detail, backups, management, health, settings
    lib/stores/       -- 5 Zustand stores: device, navigation, backup, management, samples
    lib/tauri.ts      -- 20+ IPC wrappers
    lib/types.ts      -- All TypeScript interfaces
```

**Key architectural decisions that held up well:**
- Separate ot-parser crate (no I/O, no Tauri dependency) enables independent testing
- Opaque blob storage for undocumented binary sections ensures round-trip fidelity
- Tauri-specta auto-generates TypeScript bindings (debug builds)
- Zustand stores partition state cleanly (device, navigation, backup, management, samples)
- Snapshot-before-write pattern applied consistently across all write operations

---

## 2. Known Issues and Polish Needed

### 2.1 Phase 5 Code Review Warnings (WR-01 through WR-04)

| ID | Severity | Issue | Impact | Fix Effort |
|----|----------|-------|--------|-----------|
| WR-01 | Medium | SlotRow "Dismiss" button onClick is a no-op | Users cannot dismiss format errors without page reload | 15 min |
| WR-02 | Medium | assign_sample lacks independent format validation | Direct IPC call could bypass dry-run and assign non-audio file | 20 min |
| WR-03 | Low | Wallflower file copy uses std::fs::copy (not atomic) | Partial file on OT card if USB disconnects mid-copy | 20 min |
| WR-04 | Low | Silent skip when Wallflower destination file already exists | Stale file used if Wallflower source updated with same filename | 20 min |

### 2.2 Parser Completeness Gaps

| File Type | Parsing Depth | Gap | Impact |
|-----------|--------------|-----|--------|
| .ot (SampleSettingsFile) | Full structured parse | None | Fully usable |
| bankNN.work/.strd | Opaque blob (header + checksum only) | Cannot read/write individual patterns, parts, machine settings | No pattern/part editing possible |
| markers.work/.strd | Opaque blob (header + checksum only) | Cannot modify individual slot markers (trim, loop, slices) | Marker editing not possible |
| arrNN.work/.strd | Opaque blob (header + checksum only) | Cannot read/edit arrangement data | Arrangement features blocked |
| project.work/.strd | Text parser (TYPE/SLOT/PATH extraction) | Only sample slot paths parsed; other settings opaque | Cannot modify tempo, machine assignments, or other project settings from binary |

**Key insight:** The binary format parser was designed for round-trip safety, not deep editing. Bank/markers/arrangement bodies are opaque -- they can be copied, backed up, and restored byte-exactly, but their internal structure cannot be modified field-by-field. This is the correct architectural choice for now (safety over features), but it limits future functionality.

### 2.3 UI/UX Gaps

| Area | Issue | Impact |
|------|-------|--------|
| SlotPickerDialog | Shows all 128 slots as empty (pre-existing Phase 2 stub) | Users cannot see which slots are occupied when choosing a target via Wallflower push |
| get_project_samples | Reads from text format only; returns stub data if project.work is not text-format | Some OT firmware versions may use different formats |
| No frontend tests | Zero component tests, zero integration tests | No automated UI regression detection |
| No error boundary | No global React error boundary | Unhandled errors could crash the entire UI |
| Offline-first UX | No explicit offline indicator or retry logic | App assumes OT is always mounted when performing operations |

### 2.4 Testing Gaps

| Category | Coverage | Gap |
|----------|----------|-----|
| ot-parser unit tests | 29 tests | All file types covered; round-trip tests verify byte-exactness |
| takoyaki-app unit tests | 90 tests | Commands, DB, atomic writes, health, management, snapshots |
| Integration tests (test files) | 7 files, 20 tests | backup_db, dry_run, backup, restore, project_detail, projects, health_check |
| FAT32 integration | None | No tests against real FAT32 volumes (research flag from Phase 1) |
| End-to-end tests | None | No automated E2E with Tauri webdriver |
| Frontend tests | None | No React component tests |
| Real OT fixture tests | None | Tests use synthetic fixtures, not real OT project files |

### 2.5 Safety Model Assessment

The three-layer safety model is consistently implemented:

1. **Auto-snapshot before writes** -- Every destructive command (assign_sample, restore_snapshot, duplicate, rename, export, copy_bank) calls SnapshotEngine before modifying files.
2. **Dry-run preview** -- Every destructive UI flow shows a FileChangeManifest before committing. DryRunModal is reused across backup, management, and sample assignment.
3. **Atomic staged writes** -- atomic_write_batch stages to temp files on the same volume, fsyncs, then renames.

**Gap in the model:** The FAT32 rename atomicity assumption (Phase 1 research flag) has never been validated on a real FAT32 volume. On FAT32, rename may not be truly atomic if the filesystem journal is incomplete. This is the single biggest unvalidated assumption in the safety model.

---

## 3. Proposed Next Steps

### P0: Ship Blockers (Must Fix Before Any User Touches This)

These issues would cause confusion, data loss, or broken UX in real use.

| # | Issue | Fix | Effort |
|---|-------|-----|--------|
| P0-1 | WR-02: assign_sample accepts any file without format check | Add independent format validation in assign_sample before snapshot/write | 20 min |
| P0-2 | WR-01: Dismiss button non-functional | Wire onDismissError prop to clearSlotError in SlotRow | 15 min |
| P0-3 | Test with REAL OT project files | Obtain real .work, .strd, markers, bank files from an actual OT card; add as test fixtures; verify parsers handle them correctly | 2-4 hours |
| P0-4 | Validate project.work text format assumption | The parser assumes TYPE=/SLOT=/PATH= text format. Verify this against real OT card data. If binary, the sample assignment feature is broken | 1-2 hours |
| P0-5 | FAT32 atomicity validation | Create integration test that writes to a real FAT32 volume and verifies rename behavior under simulated interruption | 2-4 hours |
| P0-6 | WR-03: Non-atomic Wallflower file copy | Use temp-then-rename pattern for AUDIO/ file deployment | 20 min |
| P0-7 | Global error boundary | Add React error boundary to prevent full-app crashes on unhandled exceptions | 30 min |

### P1: Core Value Delivery (Complete the Promise)

These features are needed to make Takoyaki genuinely useful for day-to-day OT workflow.

| # | Feature | Why | Effort |
|---|---------|-----|--------|
| P1-1 | Real sample slot data in get_project_samples | Currently returns data only from text-format project.work. Need to handle both text and binary variants, and cross-reference with markers.work for trim/loop metadata | 1-2 days |
| P1-2 | Unused sample detection | The health engine can check format compatibility but does not detect orphaned audio files on the card (present in /AUDIO but not referenced by any project) | 1 day |
| P1-3 | Multi-project backup (batch) | Users often want to back up their entire card at once, not project by project | 1 day |
| P1-4 | Backup scheduling / auto-backup on mount | The single most requested feature in OT communities. When OT is plugged in, auto-snapshot everything | 1-2 days |
| P1-5 | Backup diff view | Show what changed between two snapshots (file-level diff). Users want to understand project evolution | 1 day |
| P1-6 | Drag-and-drop sample assignment | Currently uses native file picker. Drag-and-drop from Finder would be much faster for bulk assignment | 1 day |
| P1-7 | Sample waveform display | Show waveform in slot row (or on hover/click) so users can visually identify samples without previewing audio | 2-3 days |

### P2: Differentiation (What Makes Takoyaki Uniquely Valuable)

These features go beyond "another librarian" and leverage Takoyaki's unique position (safety model + Wallflower + clean-room parser).

| # | Feature | Why | Effort |
|---|---------|-----|--------|
| P2-1 | Set list preparation mode | OT users prepare for gigs by organizing banks across projects. A "set list" view that shows tonight's projects in order, with quick-swap of banks between them, would be transformative. Leverages bank_copy infrastructure | 3-5 days |
| P2-2 | Project template system | Save a "blank" project with preferred machine assignments, effect settings (once pattern/part parsing exists), and sample assignments. Create new projects from templates | 2-3 days |
| P2-3 | Wallflower smart suggestions | When a slot is empty and the user opens the assignment flow, suggest Wallflower samples based on the project's tempo, key, and existing sample characteristics | 2-3 days |
| P2-4 | Sample chain builder | The OT's sample chain workflow (concatenating audio files + generating .ot slices) is currently manual and painful. OctaChainer exists but is unmaintained. Takoyaki already has the .ot parser and audio health engine to do this better | 3-5 days |
| P2-5 | Deep bank parser (Pattern/Part) | Parse the opaque bank body to expose patterns, parts, machine assignments. Enables pattern browsing, part comparison, and eventual editing | 2-4 weeks |
| P2-6 | Cross-project sample deduplication | Identify identical audio files across multiple projects and show how much card space could be recovered by consolidating them | 1-2 days |
| P2-7 | OT firmware version detection | Read project file version bytes to detect which firmware created a project. Warn about compatibility when moving projects between OT units | 1 day |

### P3: Community and Polish

| # | Feature | Why | Effort |
|---|---------|-----|--------|
| P3-1 | DMG packaging and code signing | Ship a proper macOS app. Currently requires building from source | 1-2 days |
| P3-2 | Elektronauts beta program | The OT community lives on Elektronauts. Post a beta thread, gather real-world test data (OT project files, bug reports, feature requests) | Ongoing |
| P3-3 | Frontend test suite | Add Vitest + React Testing Library. Start with SamplesTab (most complex component, 528 lines) and DryRunModal | 2-3 days |
| P3-4 | Keyboard shortcuts | Power users want to navigate quickly. Cmd+B for backup, Cmd+R for restore, arrow keys for slot navigation | 1 day |
| P3-5 | Dark/light theme toggle | Currently hardcoded warm dark theme. Some users work in bright environments | 4-8 hours |
| P3-6 | Menu bar integration | Quick-access to backup status, device connection, recent projects from the macOS menu bar | 1-2 days |
| P3-7 | Onboarding flow | First-run wizard: detect OT, explain the safety model, run first backup. Builds trust immediately | 1-2 days |
| P3-8 | Telemetry / crash reporting (opt-in) | Understand real usage patterns and catch errors in the wild | 1 day |

---

## 4. Strategic Observations

### 4.1 The Parser Strategy Was Correct

Storing undocumented binary sections as opaque blobs (D-02) was the right call. It enables:
- Safe backup/restore without understanding internal structure
- Bank copy that preserves pattern data verbatim
- Zero risk of corruption from incorrect field interpretation

The cost is that deeper features (pattern editing, part comparison) require future parser work. But shipping safety features first builds trust in the tool.

### 4.2 The Community Opportunity Window

OctaEdit has been dead for years. OctaLib never shipped beyond Phase 1. The Elektronauts community has 50,000+ members actively using Octatracks with no reliable tooling. Key community pain points:

1. **Backup anxiety** -- "What if my CF card corrupts?" (Takoyaki solves this NOW)
2. **Sample management across projects** -- "I have 200 kicks across 12 projects, which ones am I actually using?" (P1-2 + P2-6 address this)
3. **Set list preparation** -- "I need to reorganize 4 projects for Friday's gig" (P2-1 would be transformative)
4. **Sample chain workflow** -- "Chaining samples for the OT is tedious and error-prone" (P2-4 fills the OctaChainer gap)

### 4.3 The Wallflower Integration is a Genuine Differentiator

No other OT tool integrates with a sample library. The pipeline of "capture/analyze in Wallflower -> deploy to OT via Takoyaki" is unique and solves a real workflow: musicians collect samples from various sources, want them organized/tagged, then want to push specific ones to their hardware.

### 4.4 What to Ship First

The fastest path to community value:
1. Fix P0 items (1-2 days total)
2. Package as DMG (P3-1)
3. Post beta on Elektronauts (P3-2)
4. Gather real OT project files from beta testers (validates P0-3, P0-4, P0-5)
5. Implement P1-3 and P1-4 (auto-backup) based on community feedback

The auto-backup-on-mount feature (P1-4) is likely the single highest-value feature for the community. "Plug in my OT and everything is automatically safe" is the core promise distilled to one action.

### 4.5 Technical Debt to Address

| Debt | Risk | When to Address |
|------|------|----------------|
| No real OT fixture data | Parser could fail on real files | Before beta release |
| FAT32 atomicity unvalidated | Safety model's foundation is assumed, not proven | Before beta release |
| project.work format assumption | Could be wrong for some firmware versions | Before beta release |
| In-memory DB (development) | Data lost on app restart | Before beta release (already has file-based DB code, just needs wiring) |
| No frontend tests | UI regressions invisible | Before adding new UI features |
| Checksum algorithm not implemented | Cannot modify bank/markers/arrangement bodies | Before any deep editing features |

---

## 5. Recommended Immediate Action Plan (Next 2 Weeks)

**Week 1: Validate and Fix**
- Day 1-2: Obtain real OT project files (from your own OT or community volunteers). Run parsers against them. Fix any failures.
- Day 3: Fix P0-1 through P0-3 (all quick fixes from code review)
- Day 4: FAT32 integration test on a real formatted volume
- Day 5: DMG packaging and basic code signing

**Week 2: Ship Beta**
- Day 1: Global error boundary, onboarding flow basics
- Day 2: Multi-project backup (batch "Back Up All")
- Day 3: Write Elektronauts beta post, prepare beta release
- Day 4-5: Respond to first beta feedback, fix critical issues

This gets Takoyaki into real users' hands within 2 weeks while addressing the most important validation gaps.

---

## Appendix: File Counts and Metrics

| Metric | Value |
|--------|-------|
| Total Rust source files | 43 |
| Total TypeScript/React source files | 59 |
| Rust lines of code | 9,375 |
| TypeScript/React lines of code | 7,493 |
| Passing tests | 139 |
| Test files (integration) | 7 |
| Tauri commands registered | 25 |
| Zustand stores | 5 |
| React components (non-UI-lib) | 30+ |
| SQLite migrations | 3 |
| Cargo dependencies (app crate) | 22 |
| Phase plans completed | 28 |
| Quick tasks completed | 2 |
