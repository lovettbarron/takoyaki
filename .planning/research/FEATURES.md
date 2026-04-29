# Feature Research

**Domain:** Hardware sampler management — desktop backup/versioning/file management for Elektron Octatrack
**Researched:** 2026-04-29
**Confidence:** HIGH (Elektronauts forum threads, GitHub repos, OctaEdit documentation, OctaStudio beta discussion all cross-verified)

---

## Competitive Landscape Summary

Before features, the competitive landscape as of April 2026:

| Tool | Status | Core Capability | Gap |
|------|--------|-----------------|-----|
| OctaEdit | Abandoned (commercial) | Full-featured: 13 modules, project/bank/sample/sequencer editing | Never open-source; dead |
| OctaZip | Abandoned (part of OctaEdit) | Project archiving with sample integrity | Dead with OctaEdit |
| OctaLib | Active (C#, Windows) | Project visualization, bank swapping within a project | Windows-only; Phase 1 partial |
| OctaStudio | Active beta (macOS) | Sample chaining, cross-project bank copy, pattern editing | No backup/versioning; beta unstable |
| Project Manager (imacactus) | Active development | Bank copy, fix missing samples, project overview | Early; no backup/versioning |
| OctaChainer | Maintained | Sample chain .wav + .ot generation | Single workflow only |
| DigiChain | Active (browser) | Sample chain creation + format conversion | Browser-based; no project management |
| ot-tools-io | Active (Rust lib) | Binary format read/write | GPL v3; no GUI; developer warns unstable |

**Key insight:** No existing tool provides backup, versioning, snapshot protection, or data integrity validation. This is Takoyaki's primary differentiation vector.

---

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume any Octatrack management tool provides. Missing = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Project listing and browsing | Every librarian tool does this; it is the entry point to all other features | LOW | Read-only scan of Set directory on mounted CF card; display project name, bank count, last-modified date |
| Project file backup to Mac | Manual "copy the card" is current best practice; users have lost months of work to CF card failure | LOW | Copy entire project directory tree; straightforward filesystem operation |
| Project duplicate/copy | Multiple tools (OctaEdit Manager, OctaLib) implement this; needed before any destructive edit | MEDIUM | Must remap all sample slot paths to new project location; 18-file coordination required per slot change |
| Project rename on disk | Part of basic project organization; users currently do this manually in Finder | LOW | Rename project folder; update internal project name if stored in binary header |
| Bank visualization (which banks/patterns are populated) | OctaLib implements this; users need to see what is in a project before touching it | MEDIUM | Parse .work bank files to detect non-empty patterns; must handle OctaLib's known bug: empty detection requires checking all pages, not just page 1 trigs |
| Sample slot listing (Flex + Static, 128 each) | Users consistently ask for visibility into what samples are assigned to which slots | MEDIUM | Parse project binary for flex/static slot assignments; display file path + slot number |
| Detect missing/broken sample references | Explicitly requested by users; OctaStudio has "Missing Sample Auto-Fix"; Project Manager has "Fix Missing Samples" | MEDIUM | Walk all slot assignments, verify file exists at referenced path; report missing files |
| Audio file format validation | OT only accepts 16/24-bit WAV/AIFF at 44.1kHz stereo/mono; incompatible files silently fail | LOW | Check headers of samples in audio pool and assigned slots; flag non-44.1kHz, non-WAV/AIFF, wrong bit depth |
| Project-level metadata display | Users want to see tempo, bank names, part names, active machine types per track | MEDIUM | Parse project binary for tempo (stored in project file), bank/part names (stored in .work files) |
| USB mount detection and guidance | Users need to know when their OT is connected in USB mode and mounted | LOW | Watch for volume mount matching Octatrack card structure (presence of Audio, Project folders); show status |

### Differentiators (Competitive Advantage)

Features that set Takoyaki apart. No existing tool provides these.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Automatic snapshot before every write | Zero other tools protect against corrupt writes; CF card data loss is a known, recurring community trauma | HIGH | Before any operation that modifies OT files: snapshot affected files; store in Takoyaki's SQLite history with timestamp + operation label; enable rollback |
| Snapshot history browser and rollback | Users have lost months of work; version history lets them recover any previous state | HIGH | List snapshots chronologically; diff-style preview showing what changed; one-click restore; depends on snapshot feature |
| Dry-run preview before destructive operations | No existing tool shows what will change before committing; users modify projects blind | MEDIUM | Model the complete set of file changes for an operation; present summary ("will modify 18 files across 3 banks"); require confirmation; only then write |
| Atomic staged writes (all-or-nothing) | Partial writes leave projects in corrupt state; this is the root cause of several documented data loss incidents | HIGH | Write to temp files first; verify all writes succeeded; then rename to final paths atomically; on failure, restore from pre-operation snapshot |
| Wallflower library integration | Musicians who use Wallflower can search samples by key, BPM, tags and push directly to OT slots from their analyzed library | HIGH | Read-only SQLite query against Wallflower DB; display metadata-rich sample browser; one-click assign to flex/static slot with automatic path handling; must degrade gracefully when Wallflower is absent |
| Sample assignment from desktop (drag & drop to slot) | Explicitly requested on Elektronauts; OctaStudio has bank-level copy but not slot-level assignment from arbitrary desktop files | HIGH | Map a local WAV/AIFF file to a specific flex or static slot number; copy file to project's audio pool; update all affected binary files; 18-file coordination |
| Unused sample detection and reporting | OT has a "purge samples" function but users want desktop visibility before doing it | MEDIUM | Cross-reference all slot assignments against what is used in patterns; list orphaned slots (assigned but no pattern trig) and audio pool files not referenced by any project |
| Backup verification / integrity check | No tool validates that a backup is actually readable and complete | MEDIUM | After backup: compute checksums of source vs backup; verify file count matches; alert on any discrepancy; store verification record in SQLite |
| Project search and filtering | With many projects on a card, users need to find by name, tempo, date | LOW | Index project metadata in SQLite on first scan; query index; no binary re-parsing per search |
| Export project with collected samples | OctaEdit had OctaZip; users want to archive a project as a self-contained zip with all its samples | MEDIUM | Run equivalent of OT's "collect samples" operation: resolve all sample paths, copy to project folder, zip with consistent structure; useful for sharing or offload |
| Cross-project bank copy with sample remap | Both OctaEdit Manager and OctaStudio implement this; it is highly requested | HIGH | Copy bank (patterns + parts) from source project to target; resolve slot conflicts (auto-merge or user-directed); remap all sample file paths to target project's sample slots |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem desirable but should be explicitly excluded or deferred.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Pattern/sequencer editing | OctaEdit had a full sequencer module; power users want it | Binary format for pattern data (parameter locks, trig conditions, micro-timing) is the most complex and least documented part of the OT format; reverse-engineering risk is very high; would dwarf backup/management scope | OctaStudio is building this; focus Takoyaki on data safety and management, leave sequencer editing as a future phase once format is fully understood |
| Direct MIDI/SysEx hardware control | Users ask for "real-time" control from desktop | Requires MIDI stack, real-time sync, and a completely different architecture; USB disk mode covers all file management use cases | USB disk mode is sufficient; MIDI control adds no value for backup/management |
| Sample audio preview with transport | Users expect to preview samples before assigning them | Requires audio engine in the desktop app; adds significant complexity and platform integration work | Defer to v1.x; use system "Quick Look" (spacebar) as bridge; or document that users should preview in Finder before assigning |
| Sample editing (trim/normalize/transcode) | OctaEdit's Samples module had this; users want to prep samples without leaving the tool | Requires audio processing pipeline; transcoding for format compatibility (48kHz → 44.1kHz) is high complexity; out of scope for a backup/management tool | DigiChain handles format conversion; recommend users prep samples externally before assigning |
| OT MkI support | Some users still run MkI | Binary format differences create edge cases and parser complexity; MkII has the larger active user base | Document MkII-only scope clearly; accept GitHub issues for MkI differences to inform future support |
| Windows support | Most desktop tools are cross-platform | Tauri supports Windows but initial focus should be Mac-first (consistent with Wallflower); Windows adds QA surface and distribution complexity | Mac-first, evaluate Windows demand post-launch |
| Cloud sync / remote backup | Power users want off-site backup | Introduces auth, privacy, and cloud provider dependencies; local-first is the right default for a tool managing creative work | Support local backup to any mounted volume (NAS, external drive) via standard path configuration |
| Real-time monitoring / auto-backup on save | Appealing UX idea | The OT does not signal when it saves; USB disk mode disconnects during OT use; polling the card while it is also being written to by the OT is dangerous | Offer manual "backup now" with connection status indicator; automatic backup on USB mount event is safe and sufficient |

---

## Feature Dependencies

```
[USB Mount Detection]
    └──required by──> [Project Listing / Browsing]
                          └──required by──> [Bank Visualization]
                          └──required by──> [Sample Slot Listing]
                          └──required by──> [Missing Sample Detection]
                          └──required by──> [Unused Sample Detection]
                          └──required by──> [Project Metadata Display]

[OT Binary Parser (Rust)]
    └──required by──> [Bank Visualization]
    └──required by──> [Sample Slot Listing]
    └──required by──> [Project Metadata Display]
    └──required by──> [Cross-Project Bank Copy]
    └──required by──> [Sample Assignment from Desktop]
    └──required by──> [Dry-Run Preview]

[Project File Backup]
    └──required by──> [Automatic Snapshot Before Write]
                          └──required by──> [Snapshot History + Rollback]
                          └──required by──> [Atomic Staged Writes]

[Automatic Snapshot Before Write]
    └──required by──> [Sample Assignment from Desktop]
    └──required by──> [Cross-Project Bank Copy]
    └──required by──> [Project Duplicate / Copy]

[Sample Slot Listing]
    └──enhances──> [Sample Assignment from Desktop]
    └──enhances──> [Missing Sample Detection]
    └──enhances──> [Unused Sample Detection]

[Wallflower SQLite Read]
    └──required by──> [Wallflower Library Integration]
    └──enhances──> [Sample Assignment from Desktop]
                       (provides metadata-rich sample browser as source)

[Audio File Format Validation]
    └──enhances──> [Sample Assignment from Desktop]
                       (validate before assigning incompatible file)
    └──enhances──> [Missing Sample Detection]
                       (also flag format-incompatible assigned samples)

[Project Search / Index (SQLite)]
    └──enhances──> [Project Listing / Browsing]

[Export Project with Collected Samples]
    └──requires──> [Sample Slot Listing]
    └──requires──> [Project File Backup]
```

### Dependency Notes

- **OT Binary Parser required for nearly everything:** The parser is the foundation. It must be production-quality before any write features are built. Read-only features can ship before write-path is complete.
- **Snapshot required before all writes:** The three-layer safety model makes snapshot a prerequisite for every destructive operation. Write features must not ship without it.
- **Wallflower integration is enhancement, not blocker:** Takoyaki must stand alone. Wallflower integration adds a metadata-rich source for sample assignment but does not change the core assignment workflow.
- **Sample assignment from desktop is the hardest operation:** It requires the binary parser, snapshot, atomic write, format validation, and (optionally) Wallflower integration. It should be built last among write-path features.

---

## MVP Definition

### Launch With (v1)

Minimum viable product. Validates that the parser is correct and the safety model works. Delivers immediate value for backup use case.

- [ ] USB mount detection — users can see when their OT card is available
- [ ] Project listing with metadata (name, bank count, tempo, last modified) — replaces manual Finder browsing
- [ ] Bank and pattern visualization (which banks/patterns are populated) — users can see project structure at a glance
- [ ] Sample slot listing (Flex + Static, both 128 slots) with file path display — addresses the #1 workflow complaint
- [ ] Missing sample detection (report broken references) — addresses documented data loss scenarios
- [ ] Project backup to Mac (copy project directory tree) — core value: data safety
- [ ] Backup verification with checksums — differentiates from manual Finder copy; builds trust
- [ ] Project duplicate / copy with path remapping — needed for safe experimentation
- [ ] Automatic snapshot before every write operation — non-negotiable; the safety model cannot be partial

### Add After Validation (v1.x)

Add once v1 is stable and parser is proven on real-world project files.

- [ ] Snapshot history browser and rollback — activate after snapshot storage is proven reliable
- [ ] Dry-run preview for destructive operations — activate once write-path is built
- [ ] Atomic staged writes — activate alongside first write operation (duplicate/copy)
- [ ] Audio file format validation — add to sample slot listing view; low effort, high value
- [ ] Unused sample detection — straightforward cross-reference query once slot listing is working
- [ ] Export project with collected samples (archive/zip) — OctaZip equivalent; addresses missing archival use case
- [ ] Project search and filtering via SQLite index — quality-of-life once user has many projects indexed
- [ ] Cross-project bank copy with sample remap — high-value, high-complexity; requires write-path maturity

### Future Consideration (v2+)

Defer until product-market fit is established and parser format coverage is complete.

- [ ] Sample assignment from desktop (drag & drop to slot) — highest value but highest complexity; requires all write-path pieces to be proven first
- [ ] Wallflower library integration — high-value differentiator for Wallflower users; defer until core is stable; must not block launch
- [ ] Sample audio preview — quality-of-life; requires audio engine; build after core workflow is validated
- [ ] Pattern/sequencer editing — separate product scope; deep format complexity; do not tackle in v1 or v2
- [ ] Windows support — evaluate demand post-launch

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Project listing / browsing | HIGH | LOW | P1 |
| Bank + pattern visualization | HIGH | MEDIUM | P1 |
| Sample slot listing | HIGH | MEDIUM | P1 |
| Missing sample detection | HIGH | MEDIUM | P1 |
| Project backup to Mac | HIGH | LOW | P1 |
| Backup verification | HIGH | LOW | P1 |
| Automatic snapshot before write | HIGH | HIGH | P1 |
| Project duplicate / copy | HIGH | MEDIUM | P1 |
| Snapshot history + rollback | HIGH | HIGH | P2 |
| Dry-run preview | HIGH | MEDIUM | P2 |
| Atomic staged writes | HIGH | HIGH | P2 |
| Audio file format validation | MEDIUM | LOW | P2 |
| Unused sample detection | MEDIUM | LOW | P2 |
| Export project with collected samples | MEDIUM | MEDIUM | P2 |
| Cross-project bank copy | HIGH | HIGH | P2 |
| Project search / filtering | MEDIUM | LOW | P2 |
| Sample assignment from desktop | HIGH | HIGH | P3 |
| Wallflower library integration | HIGH | HIGH | P3 |
| Sample audio preview | MEDIUM | MEDIUM | P3 |
| Pattern / sequencer editing | HIGH | VERY HIGH | Out of scope |

**Priority key:**
- P1: Must have for launch (v1)
- P2: Should have, add after v1 validation (v1.x)
- P3: High-value, build when core is proven (v2+)

---

## Competitor Feature Analysis

| Feature | OctaEdit (dead) | OctaStudio (beta, macOS) | OctaLib (Windows) | Project Manager (active) | Takoyaki |
|---------|-----------------|--------------------------|-------------------|--------------------------|----------|
| Project browsing | Yes (Project module) | Yes | Yes (grid view) | Yes | P1 |
| Bank visualization | Yes | Yes | Yes | Yes | P1 |
| Sample slot listing | Yes (Samples module) | Yes | No | Partial | P1 |
| Missing sample fix | Yes | Yes (auto-fix) | No | Yes (v0.29.0) | P1 |
| Automatic snapshot before write | No | No | No | No | P1 — UNIQUE |
| Backup + restore | Yes (OctaZip) | No | No | No | P1 — UNIQUE |
| Backup integrity verification | No | No | No | No | P2 — UNIQUE |
| Snapshot history + rollback | No | No | No | No | P2 — UNIQUE |
| Dry-run preview | No | No | No | No | P2 — UNIQUE |
| Atomic staged writes | No | No | No | No | P2 — UNIQUE |
| Cross-project bank copy | Yes (Manager) | Yes (Copy Bank) | Planned | Yes | P2 |
| Sample chain creation | Yes (Chainer) | Yes (chains + Lazy Chop) | No | No | Anti-feature (defer) |
| Pattern / sequencer editing | Yes (Sequencer module) | Partial (toggle trigs) | No | No | Anti-feature (out of scope) |
| Wallflower integration | No | No | No | No | P3 — UNIQUE |
| Sample assignment (desktop → slot) | Yes (Samples module) | No | No | Planned | P3 |
| Audio format validation | No | No | No | No | P2 — UNIQUE |
| Sample preview | Yes | No | No | No | P3 |
| macOS support | Yes | Yes (Apple Silicon + Intel) | No | Yes | Yes |
| Open source | No | No (commercial TBD) | Yes (MIT) | Yes | Yes (MIT) |
| Active development | No (abandoned) | Yes | Partial | Yes | — |

---

## OT-Specific Feature Notes

### The 18-File Problem
Moving or re-assigning a single sample slot in a project requires coordinated changes across: the project file itself, all bank files (.work) that reference that slot, and associated marker files. A desktop tool that only updates one file will leave the project in a corrupt state. Every write operation must be modeled as a transaction across all affected files.

### Flex vs Static Distinction
The OT has separate slot lists for Flex (RAM, 128 slots) and Static (streamed, 128 slots) machines. A desktop tool must distinguish between them. Assigning a file intended for a Flex slot to a Static slot (or vice versa) is a user error the tool should catch.

### .ot Sidecar Files
Every sample can have a companion `.ot` file storing trim points, loop settings, gain, and up to 64 slice definitions. A tool that moves or renames a sample must move the corresponding `.ot` file. Format: binary with sample positions encoded as sample-count integers. OT self-corrects certain calculated values on first use (checksum is recalculated on device), but the fields must be structurally valid.

### Audio Pool vs Project Folder
The audio pool is shared across all projects in a Set. A project's own folder holds project-specific copies after "collect samples." The distinction matters for backup: a full backup must include the audio pool, not just the project folder.

### Purge vs Collect
The OT device has two built-in functions: PURGE SAMPLES (removes slot assignments for samples not triggered in any pattern) and COLLECT SAMPLES (copies all assigned samples into the project folder). A desktop tool should model the same concepts: detect purgeable samples (assigned but untriggered) and offer collection/export with all dependencies resolved.

---

## Sources

- [Elektronauts: Project Manager for Octatrack (active thread)](https://www.elektronauts.com/t/project-manager-for-octatrack/233672) — pages 1, 6, 11, 13
- [Elektronauts: OctaStudio macOS app beta thread](https://www.elektronauts.com/t/octastudio-macos-desktop-app-for-the-octatrack-sample-chaining-bank-management-pattern-editor/249502) — April 2026
- [Elektronauts: OctaLib Simple Librarian](https://www.elektronauts.com/t/octalib-a-simple-octatrack-librarian/225192)
- [Elektronauts: ot-tools-io Rust library](https://www.elektronauts.com/t/ot-tools-io-open-source-rust-library-for-reading-writing-modifying-octatrack-files/232508)
- [GitHub: OctaLib](https://github.com/snugsound/OctaLib/)
- [GitHub: OctaChainer](https://github.com/KaiDrange/OctaChainer)
- [GitHub: DigiChain](https://github.com/brian3kb/digichain)
- [Elektronauts: Dead CF card thread (data loss)](https://www.elektronauts.com/t/dead-cf-card-back-up-your-projects/178488)
- [Elektronauts: Project crashed, missing samples](https://www.elektronauts.com/t/project-crashed-missing-samples-i-might-be-done-with-it/15290)
- [Elektronauts: Several months lost during backup](https://www.elektronauts.com/t/several-months-of-work-lost-during-backup/11628)
- [Elektronauts: .OT format definition](https://www.elektronauts.com/t/ot-format-definition/160601)
- [Elektronauts: How to organize flex sample slots](https://www.elektronauts.com/t/how-to-organize-flex-sample-slots/236523)
- [ManualsLib: Octatrack MKII User Manual — Sample slots, Audio pool, File compatibility](https://www.manualslib.com/manual/1309767/Elektron-Octatrack-Mkii.html?page=29)
- [OctaEdit feature description (Gearspace / MOD Wiggler threads)](https://gearspace.com/board/electronic-music-instruments-and-electronic-music-production/995201-octaedit-elektron-octatrack-software-editor.html)

---

*Feature research for: Octatrack desktop backup/versioning/file management (Takoyaki)*
*Researched: 2026-04-29*
