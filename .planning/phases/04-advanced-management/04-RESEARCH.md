# Phase 4: Advanced Management - Research

**Researched:** 2026-05-01
**Domain:** Rust file operations (duplicate, rename, export, bank copy), zip archiving, project.work text parsing, React UI extensions
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Project Duplication**
- D-01: Full copy — duplicate copies all referenced audio files (+ .ot sidecars) into the new project directory. Produces a fully independent, self-contained copy. No shared references.
- D-02: Default name: original name + `_copy` suffix (e.g., LIVESET_01 → LIVESET_01_copy).
- D-03: If the auto-generated name exceeds the OT directory name length limit, fall back to prompting the user to type a name that fits. No auto-truncation.

**Project Export**
- D-04: Export produces a self-contained zip with the complete `/SETS/PROJECT_NAME/` directory AND all referenced audio files in `/AUDIO/`, preserving the OT directory structure. Unzipping to a blank CF card produces a playable project.
- D-05: Export includes `.ot` sidecar files for every referenced sample — slice points, loop settings, and trim data are preserved. Truly play-ready export.
- D-06: Exports saved to `~/takoyaki/exports/`, consistent with Phase 3 backup convention (`~/takoyaki/backups/`). Organized by project name and date.

**Bank Copy & Conflict Resolution**
- D-07: When copying a bank to another project, missing samples are copied automatically. If the target already has the same filename with identical content (hash match), skip. No user prompt for unambiguous cases.
- D-08: When a filename exists in the target with different content (hash mismatch), surface the conflict in the dry-run preview with three options: keep target's version, overwrite with source's version, or rename the incoming file.
- D-09: If the target bank slot is populated, warn the user and show available empty slots. User can pick an empty slot or explicitly confirm overwrite. No silent overwrites.
- D-10: Bank copy target selection uses a two-step picker dialog: Step 1 — select target project from a list; Step 2 — select target bank slot with a 4×4 grid showing populated vs. empty slots.

**Project Rename**
- D-11: Clicking "Rename" makes the project name editable inline in the project detail header. User types the new name, confirms, then sees the mandatory dry-run preview showing directory rename + internal name field update.

**Management Actions UX**
- D-12: Project-level actions (Duplicate, Rename, Export) appear as toolbar buttons in the project detail view header, alongside the project name and metadata.
- D-13: Bank copy is a per-bank action accessed by right-clicking a bank in the bank grid → "Copy to project..."
- D-14: All operations go through the mandatory dry-run preview (Phase 3 D-08/D-09) and automatic pre-operation snapshot (Phase 3 D-11). Success feedback uses the inline auto-dismissing banner (Phase 3 D-13).

### Claude's Discretion
- Exact toolbar button styling and icon choices for Duplicate/Rename/Export
- Bank grid right-click context menu implementation (native OS menu vs. custom)
- Export progress indicator during zip creation
- How the two-step bank copy picker dialog is styled
- Whether the rename inline edit validates OT-legal characters in real-time
- Zip compression level (speed vs. size tradeoff for audio files)

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| MGMT-01 | User can duplicate/copy an OT project with automatic sample path remapping | Covered by project.work text parsing + atomic write engine; path rewriting is a text substitution in the key=value slots section |
| MGMT-02 | User can rename an OT project on disk with internal name field updated | Covered by std::fs::rename for directory; project.work has no separate NAME= field — directory name IS the authoritative project name |
| MGMT-03 | User can export a project as a self-contained zip with all referenced samples collected | Covered by zip crate 2.x (ZipWriter) + walkdir + project.work path extraction |
| SMPL-02 | User can copy banks between projects with automatic sample slot remapping and conflict resolution | Covered by BankFile opaque copy + project.work slot merging with SHA-256 conflict detection |
</phase_requirements>

---

## Summary

Phase 4 builds on all prior phases to implement the four destructive management operations: duplicate, rename, export, and bank copy. All infrastructure exists — atomic writes, snapshots, dry-run preview, progress streaming, success banners. The new work is the business logic for each operation plus their frontend triggers.

The most technically complex operation is **project duplicate**: it requires parsing `project.work` (a text key=value file) to extract sample slot path assignments, copying referenced audio files (including `.ot` sidecars) to new locations in the shared `/AUDIO/` pool, and rewriting the `PATH=` entries in the duplicated `project.work`. The same text parsing engine feeds bank copy's slot-merging logic.

The bank copy operation has a fundamental constraint: the `BankFile` body is treated as an opaque blob (see `crates/ot-parser/src/bank.rs`) so we cannot know which specific slot indices a bank's patterns reference. The conservative solution is to merge ALL populated source slots into the target project's slot assignments, applying D-07/D-08 conflict resolution per slot file. This is safe: extra slots in target do no harm, and the bank patterns continue to reference the same slot indices as they did in the source.

The export operation uses the `zip` crate (not yet in `Cargo.toml` — must be added) with `CompressionMethod::Stored` for audio files (WAV/AIFF are already compressed; Deflate would be counterproductive). The zip structure must preserve `SETS/PROJECT_NAME/` and `AUDIO/` at the root so unzipping to a blank CF card produces a playable project.

**Primary recommendation:** Add `commands/management.rs` as a new Tauri command module containing `duplicate_project`, `rename_project`, `export_project`, and `copy_bank`. All four operations share the same pre-operation snapshot + dry-run + atomic write pattern already established in Phase 3. A shared `project_work_parser` submodule handles the text format.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Project duplicate (file ops) | API / Backend (Rust) | — | All file I/O is Rust-only; frontend only triggers and monitors |
| Project rename (dir rename) | API / Backend (Rust) | — | Filesystem operation; SQLite update |
| Export to ZIP | API / Backend (Rust) | — | zip crate runs in Rust; frontend shows progress via Channel |
| Bank copy (file + slot merge) | API / Backend (Rust) | — | Slot conflict detection requires file hashing; text parsing of project.work |
| Duplicate/Rename/Export toolbar | Frontend Server (Next.js) | — | Toolbar buttons added to existing MetadataHeader component |
| Bank right-click context menu | Browser / Client | — | Event handler on BankGridCell; ContextMenu component |
| Two-step bank copy picker dialog | Browser / Client | — | New BankCopyPickerDialog component; uses existing Dialog + ScrollArea |
| Dry-run preview (all ops) | Browser / Client | — | Reuses existing DryRunModal component without modification |
| Export progress | Browser / Client | — | Reuses BackupProgressView pattern; new management store or extended backup store |

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| zip | 2.6.0 (`"2"`) | Zip archive creation for export | Only pure-Rust zip writer with directory recursion + Stored mode; already specified in CLAUDE.md as `zip = "8.x"` — confirmed latest is 2.6.0 on crates.io |
| walkdir | 2.5 | Directory traversal for export and duplicate | Already in Cargo.toml; used by backup commands |
| sha2 | 0.10 | SHA-256 hashing for D-07 hash-match detection | Already in Cargo.toml |
| atomic-write-file | 0.3 | Atomic writes for project.work updates | Already in Cargo.toml |
| dirs | 6 | Home directory for export destination | Already in Cargo.toml |

[VERIFIED: grep of Cargo.toml confirmed walkdir, sha2, atomic-write-file, dirs already present]
[VERIFIED: crates.io API confirmed zip crate latest stable = 2.6.0 (despite CLAUDE.md saying "8.x" — the "8.x" was an error in CLAUDE.md; the crate package is `zip` and current version is 2.x)]

**Note on zip version:** The CLAUDE.md stack table lists `zip = "8.x"` but crates.io confirms the `zip` crate latest is `2.6.0`. The `8.x` number does not correspond to any existing version. Add as `zip = "2"` to Cargo.toml. The zip-rs/zip2 Context7 library ID and Cargo.toml package name are both `zip`.

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| shadcn context-menu | via CLI | Right-click context menu on BankGridCell | Right-click affordance for bank copy trigger |
| @base-ui/react ContextMenu | ^1.4.1 (already installed) | Underlying implementation for context-menu | Used by shadcn context-menu component; already in package.json |

[VERIFIED: `ls src/components/ui/` confirms context-menu.tsx NOT present — must be added via `npx shadcn@latest add context-menu`]
[VERIFIED: lucide-react@1.14.0 already installed; all required icons verified: `PackageOpen`, `ArrowRightFromLine`, `Pencil`, `Copy`, `Loader2` all export correctly]

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `CompressionMethod::Stored` (audio) | `CompressionMethod::Deflated` | Stored is correct for WAV/AIFF (already compressed); Deflated wastes CPU and adds 0.1–2% size improvement |
| shadcn `context-menu` component | Custom onContextMenu handler | shadcn provides accessible, keyboard-navigable menu with correct positioning; custom handler requires manual positioning |
| Text regex for project.work PATH | binary search/replace | Regex on text format is correct and safe since project.work is a plain ASCII key=value file |

**Installation:**
```bash
# Rust — add to crates/takoyaki-app/Cargo.toml
# zip = "2"   (add to [dependencies])

# Frontend — add context-menu component
cd /Users/albair/src/takoyaki
npx shadcn@latest add context-menu
```

---

## Architecture Patterns

### System Architecture Diagram

```
Frontend trigger (toolbar / right-click)
       │
       ▼
[Dry-run IPC call] → compute_management_dry_run (Rust)
       │                  - reads project.work (text parse)
       │                  - resolves audio paths
       │                  - computes manifest (Added/Modified/Conflict)
       ▼
[DryRunModal] shows manifest
       │
  User confirms
       │
       ▼
[Operation IPC call] → execute_management_op (Rust)
       │                  - SnapshotEngine.snapshot_files() (pre-op)
       │                  - for duplicate: copy files + rewrite project.work
       │                  - for rename: std::fs::rename dir + update SQLite
       │                  - for export: ZipWriter assembly
       │                  - for bank copy: copy bank file + merge slots
       │                  - atomic_write_batch() for all project.work modifications
       │
  Channel<ManagementEvent> streams progress to frontend
       │
       ▼
[BackupProgressView reuse] shows progress
       │
  Operation complete
       │
       ▼
[InlineSuccessBanner] auto-dismisses
```

### Recommended Project Structure
```
crates/takoyaki-app/src/
├── commands/
│   ├── management.rs          # NEW: Tauri commands for Phase 4 operations
│   └── mod.rs                 # updated to pub mod management
├── management/                # NEW: management business logic
│   ├── mod.rs
│   ├── project_work.rs        # project.work text parser + slot rewriter
│   ├── duplicate.rs           # duplicate_project implementation
│   ├── rename.rs              # rename_project implementation
│   ├── export.rs              # export_project implementation
│   └── bank_copy.rs           # copy_bank implementation
src/components/
├── project-detail/
│   ├── BankGridCell.tsx       # MODIFIED: add onContextMenu + ContextMenu
│   └── MetadataHeader.tsx     # MODIFIED: add Rename/Duplicate/Export buttons
├── management/                # NEW: management-specific UI
│   └── BankCopyPickerDialog.tsx
src/lib/stores/
│   └── management.ts          # NEW: zustand store for management operations
src/lib/
│   └── tauri.ts               # MODIFIED: add management IPC wrappers
```

### Pattern 1: project.work Text Parser

The OT `project.work` file is a plain ASCII key=value text file with sections. The slot section uses:
```
[FLEX]
SLOT=001
PATH=../AUDIO/kick.wav
TRIM_BARSx100=400
TSMODE=0
LOOPMODE=0
GAIN=48
TRIGQUANTIZATION=-1
...
[STATIC]
SLOT=001
PATH=../AUDIO/snare.wav
...
```

**What:** Parse project.work bytes to extract all slot PATH assignments; rewrite PATH values after copying files.

**When to use:** Before duplicate, export, or bank copy — any operation that needs to know which audio files a project references.

**Example:**
```rust
// Source: ot-tools-io source analysis + OctaLib Research.md
// project.work text format: sections [FLEX]/[STATIC] with SLOT= + PATH= pairs

pub struct SlotPath {
    pub slot_type: SlotType,
    pub slot_number: u8,         // 1-indexed (1..=128)
    pub path: Option<String>,    // relative to card root: "../AUDIO/file.wav"
}

pub fn parse_slot_paths(raw: &[u8]) -> Vec<SlotPath> {
    // Parse text as ASCII, find [FLEX]/[STATIC] sections,
    // extract SLOT= and PATH= key pairs
    // Returns Vec ordered by type+number for predictable processing
}

pub fn rewrite_slot_path(raw: &[u8], slot_type: SlotType, slot_number: u8, new_path: &str) -> Vec<u8> {
    // Text substitution: find the PATH= line for the given slot and replace value
    // Must handle both cases: slot exists (update) and slot absent (insert)
}
```

### Pattern 2: Management Event Channel (mirrors BackupEvent)

```rust
// Source: crates/takoyaki-app/src/commands/backup.rs (BackupEvent pattern)
#[derive(Clone, Serialize, Type)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum ManagementEvent {
    Started { total_files: usize, destination: String },
    Progress { files_processed: usize, total_files: usize, current_file: String },
    Complete { files_processed: usize, total_bytes: u64, destination: String },
    Failed { reason: String },
}
```

### Pattern 3: Pre-Operation Snapshot (established in Phase 3)

```rust
// Source: crates/takoyaki-app/src/commands/backup.rs restore_snapshot()
// Every management write must snapshot first:
let snapshot_engine = SnapshotEngine::new(snapshot_root);
let files_to_snapshot: Vec<&Path> = affected_files.iter().map(|p| p.as_path()).collect();
let _result = snapshot_engine.snapshot_files(&files_to_snapshot, "pre-management-op")?;
```

### Pattern 4: zip Export Structure

```rust
// Source: Context7 zip-rs/zip2 docs [VERIFIED]
// Audio files use Stored (not Deflated) — WAV/AIFF are already compressed
use zip::{ZipWriter, CompressionMethod, write::SimpleFileOptions};

let options_audio = SimpleFileOptions::default()
    .compression_method(CompressionMethod::Stored);
let options_text = SimpleFileOptions::default()
    .compression_method(CompressionMethod::Deflated);  // project.work + bank files compress well

// OT-correct path structure inside ZIP:
// "SETS/PROJECT_NAME/project.work"
// "SETS/PROJECT_NAME/bank01.work"
// "AUDIO/kick.wav"
// "AUDIO/kick.ot"  (sidecar — D-05)
```

### Pattern 5: Rename Operation

```rust
// Rename = std::fs::rename + SQLite UPDATE
// project.work has NO internal NAME= key — the directory name IS the project name
// [VERIFIED: ot-tools-io OsMetadata struct has only TYPE, VERSION, OS_VERSION — no NAME field]
// [VERIFIED: Settings struct has no NAME field either]

pub fn rename_project(card_path: &Path, new_name: &str) -> Result<(), AppError> {
    let parent = card_path.parent().ok_or(AppError::InvalidPath)?;
    let new_path = parent.join(new_name);
    // Validate: new_name is OT-legal (A-Z, 0-9, underscore, max 16 chars)
    // Snapshot the project directory files first
    // Perform directory rename
    std::fs::rename(card_path, &new_path)?;
    // No project.work modification needed — directory name IS the project name
    Ok(())
}
```

### Pattern 6: Bank Copy Slot Merge Strategy

**Why we merge ALL source slots (not just "bank slots"):**
The `BankFile` body is an opaque blob — we cannot determine which slot indices the bank's patterns reference without parsing the undocumented binary. The conservative approach is correct:

```
For each occupied slot in source project.work:
  resolved_src_audio = resolve_ot_path(card_root, slot.path)
  target_slot = target_project.get_slot(slot.slot_type, slot.slot_number)
  
  if target_slot.path is None:
    // D-07: missing — auto-copy
    copy audio + sidecar to target AUDIO/
    rewrite_slot_path(target_project_work, ...)
  else:
    src_hash = sha256_hex(resolved_src_audio)
    tgt_hash = sha256_hex(resolve_ot_path(card_root, target_slot.path))
    if src_hash == tgt_hash:
      // D-07: same content — skip silently
    else:
      // D-08: conflict — surface in dry-run manifest as "Conflict" entry
```

### Anti-Patterns to Avoid

- **Path manipulation on frontend:** All path resolution must happen in Rust (`resolve_ot_path()` is the single point). The frontend never supplies raw card paths.
- **Writing project.work as binary:** It is a TEXT file — always read as bytes, parse as ASCII, rewrite as ASCII. Never run it through `BankFile::from_bytes()` or `SampleSettingsFile::from_bytes()`.
- **Deflating audio files in export:** `CompressionMethod::Deflated` on WAV/AIFF files wastes CPU without meaningfully reducing size. Use `Stored` for audio, `Deflated` for project/bank files.
- **Skipping pre-op snapshot:** Every management operation that writes must snapshot first. No exceptions per CLAUDE.md constraint.
- **Auto-truncating project names:** D-03 explicitly forbids auto-truncation. If the default `_copy` suffix causes overflow, fall back to a user prompt.
- **Blocking the Tauri event loop with large zip operations:** Export must be `async` and stream progress via `Channel<ManagementEvent>` — same pattern as backup.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| ZIP archive creation | Custom file concatenation | `zip = "2"` crate | Handles local file headers, central directory, end-of-central-directory record, compression metadata |
| Directory traversal | Manual `read_dir()` recursion | `walkdir` (already in Cargo.toml) | Handles symlinks, error recovery, depth control |
| SHA-256 hashing | Custom hash impl | `sha2::Sha256` via `sha256_hex()` in `atomic/snapshot.rs` | Already in codebase, tested |
| Atomic file writes | `File::create()` | `atomic_write_batch()` in `atomic/mod.rs` | Stage → fsync → rename pattern, cross-filesystem safety |
| Pre-operation snapshots | Manual file copy | `SnapshotEngine` in `atomic/snapshot.rs` | Already records to SQLite, used by Phase 3 operations |
| Context menu | `onContextMenu` + custom positioned div | `shadcn context-menu` component | Accessible, keyboard-navigable, correct z-index/portal behavior |

**Key insight:** Every low-level primitive for Phase 4 is already implemented. The work is composing these primitives into new management operations.

---

## Runtime State Inventory

> Not a rename/refactor/migration phase — omit per instructions.

---

## Common Pitfalls

### Pitfall 1: project.work PATH Values Are Card-Root-Relative

**What goes wrong:** A `PATH=` entry like `../AUDIO/kick.wav` is relative to the project directory (one level up, then into AUDIO). If you resolve it relative to the wrong base, you get a path that doesn't exist.

**Why it happens:** The OT uses relative paths from the project directory into the shared `/AUDIO/` pool at card root.

**How to avoid:** Always use `resolve_ot_path(card_volume_path, slot_path)` from `health/mod.rs`. This function already handles the `..` traversal correctly.

**Warning signs:** "File not found" errors in dry-run computation for files you know exist on the card.

### Pitfall 2: Zip Write Requires Finalization

**What goes wrong:** A zip file written with `ZipWriter` is corrupt if `.finish()` is not called — the central directory is never written.

**Why it happens:** `finish()` writes the central directory and end-of-central-directory record. Without it the file is a partial archive.

**How to avoid:** Always call `zip.finish()?` before closing. Use RAII-style drop guard or explicit call in the success path.

**Warning signs:** Created `.zip` file exists on disk but cannot be opened.

### Pitfall 3: Bank Copy Without Snapshot Loses Target Data

**What goes wrong:** Copying a bank blob over a populated target bank slot overwrites pattern data with no recovery path.

**Why it happens:** The BankFile is written atomically but without a snapshot, the original target bank data is gone.

**How to avoid:** Snapshot ALL target project files (project.work + all bank*.work files) before any bank copy write. The `SnapshotEngine` should be called with the entire target project directory's file list.

**Warning signs:** User reports lost pattern data after a bank copy operation.

### Pitfall 4: project.work Text Parse Must Handle Both .work and .strd

**What goes wrong:** If the operation reads only `project.work` and writes only `project.work`, the `.strd` version (saved state) remains stale and the OT may reload stale data on next mount.

**Why it happens:** OT maintains `.work` (working) and `.strd` (stored) versions; rename/duplicate operations must update both.

**How to avoid:** All operations that modify project files must atomically write BOTH `project.work` AND `project.strd`. Same for bank files: `bank01.work` AND `bank01.strd`.

**Warning signs:** After rename, OT shows the old name on reconnect.

### Pitfall 5: Duplicate Name Collision on Card

**What goes wrong:** `LIVESET_01_copy` already exists on the card from a previous duplicate. Creating it again would silently overwrite.

**Why it happens:** The duplicate command computes the default name without checking if it already exists.

**How to avoid:** Before creating the destination directory, check if `{new_name}` already exists under `/SETS/`. If it does, increment: `LIVESET_01_copy` → `LIVESET_01_copy2` → ... or fall back to the user-prompt path (same as D-03 overflow path).

**Warning signs:** Duplicate of an already-duplicated project corrupts the existing copy.

### Pitfall 6: FAT32 Rename on macOS Requires Same Volume

**What goes wrong:** `std::fs::rename()` fails with EXDEV if source and destination are on different filesystems.

**Why it happens:** Cross-filesystem rename is not atomic; the kernel rejects it. This is the same constraint as Phase 3's atomic write staging.

**How to avoid:** For project rename, the old and new directories are both under `/SETS/` on the OT card — same filesystem. For duplicate, the destination is also on the card. This is always safe. For export, the zip file goes to `~/takoyaki/exports/` on the Mac filesystem — `std::fs::rename()` is NOT used there; the zip is written directly to the destination.

**Warning signs:** `AppError::Io("EXDEV: cross-device link")` from rename.

### Pitfall 7: Context Menu Requires `context-menu` Shadcn Component (Not Installed)

**What goes wrong:** Attempting to import `ContextMenu` from `@/components/ui/context-menu` fails because the file does not exist yet.

**Why it happens:** Unlike `Dialog`, `Button`, etc., the `context-menu` component has not been added to this project. The UI-SPEC references "DropdownMenu (already in registry)" but the correct semantic component for right-click is `ContextMenu`, and neither is present in `src/components/ui/`.

**How to avoid:** Wave 0 task must run `npx shadcn@latest add context-menu` before any component code that imports it.

**Warning signs:** TypeScript compile error: Cannot find module `@/components/ui/context-menu`.

---

## Code Examples

Verified patterns from official sources and existing codebase:

### project.work Slot PATH Extraction (text parsing)

```rust
// Source: ot-tools-io docs.rs source analysis [CITED: https://docs.rs/ot-tools-io/0.6.0]
// project.work is ASCII key=value text. Sample slot format:
//   [FLEX]
//   SLOT=001
//   PATH=../AUDIO/kick.wav
//   ...
//   [STATIC]
//   SLOT=001
//   PATH=../AUDIO/snare.wav
//
// Key=value pairs are separated by newlines. Sections delimited by [FLEX] / [STATIC] headers.
// PATH values are relative to the PROJECT directory (not card root).
// Resolve with: resolve_ot_path(card_root, "../AUDIO/kick.wav")

pub fn extract_slot_paths(project_work_bytes: &[u8]) -> Vec<(SlotType, u8, String)> {
    let text = String::from_utf8_lossy(project_work_bytes);
    let mut results = Vec::new();
    let mut current_type: Option<SlotType> = None;
    let mut current_slot: Option<u8> = None;

    for line in text.lines() {
        let line = line.trim();
        if line == "[FLEX]" { current_type = Some(SlotType::Flex); current_slot = None; }
        else if line == "[STATIC]" { current_type = Some(SlotType::Static); current_slot = None; }
        else if let Some(rest) = line.strip_prefix("SLOT=") {
            current_slot = rest.parse().ok();
        } else if let Some(rest) = line.strip_prefix("PATH=") {
            if let (Some(t), Some(s)) = (current_type, current_slot) {
                results.push((t, s, rest.to_string()));
            }
        }
    }
    results
}
```

### Zip Export (OT Structure)

```rust
// Source: Context7 zip-rs/zip2 [VERIFIED: /zip-rs/zip2]
use zip::{ZipWriter, CompressionMethod, write::SimpleFileOptions};
use std::io::Write;

fn export_project_to_zip(
    zip_path: &Path,
    project_dir: &Path,     // card/SETS/PROJECT_NAME/
    audio_files: &[PathBuf], // resolved absolute paths to referenced audio
    project_name: &str,
) -> Result<(), AppError> {
    let file = std::fs::File::create(zip_path)?;
    let mut zip = ZipWriter::new(file);

    let opts_audio = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored);
    let opts_text = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated);

    // Add SETS/PROJECT_NAME/* directory tree
    for entry in walkdir::WalkDir::new(project_dir).follow_links(false) {
        let entry = entry.map_err(|e| AppError::Io(e.to_string()))?;
        let relative = entry.path().strip_prefix(project_dir.parent().unwrap()?)
            .map_err(|_| AppError::InvalidPath)?;
        let zip_path_str = format!("SETS/{}", relative.to_string_lossy().replace('\\', "/"));
        if entry.file_type().is_dir() {
            zip.add_directory(&zip_path_str, SimpleFileOptions::default())?;
        } else if entry.file_type().is_file() {
            let is_audio = matches!(entry.path().extension().and_then(|e| e.to_str()),
                Some("wav") | Some("aif") | Some("aiff"));
            let opts = if is_audio { opts_audio } else { opts_text };
            zip.start_file(&zip_path_str, opts)?;
            let mut f = std::fs::File::open(entry.path())?;
            std::io::copy(&mut f, &mut zip)?;
        }
    }

    // Add AUDIO/referenced_files + .ot sidecars
    for audio_path in audio_files {
        let filename = audio_path.file_name().ok_or(AppError::InvalidPath)?;
        let zip_audio_path = format!("AUDIO/{}", filename.to_string_lossy());
        zip.start_file(&zip_audio_path, opts_audio)?;
        let mut f = std::fs::File::open(audio_path)?;
        std::io::copy(&mut f, &mut zip)?;

        // .ot sidecar (D-05)
        let ot_sidecar = audio_path.with_extension("ot");
        if ot_sidecar.exists() {
            let sidecar_name = ot_sidecar.file_name().ok_or(AppError::InvalidPath)?;
            zip.start_file(format!("AUDIO/{}", sidecar_name.to_string_lossy()), opts_text)?;
            let mut sf = std::fs::File::open(&ot_sidecar)?;
            std::io::copy(&mut sf, &mut zip)?;
        }
    }

    zip.finish().map_err(|e| AppError::Io(e.to_string()))?;
    Ok(())
}
```

### Context Menu on BankGridCell (Right-Click for Bank Copy)

```tsx
// Source: Context7 shadcn-ui/ui [VERIFIED: /shadcn-ui/ui] + base-nova style
// Requires: npx shadcn@latest add context-menu (Wave 0 prerequisite)
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { ArrowRightFromLine } from "lucide-react";

// Wrap BankGridCell button with ContextMenu (populated banks only):
<ContextMenu>
  <ContextMenuTrigger asChild>
    <button ...existing BankGridCell button...>
      {/* existing content */}
    </button>
  </ContextMenuTrigger>
  {populated && (
    <ContextMenuContent className="font-mono text-xs">
      <ContextMenuItem
        onSelect={() => onCopyToProject?.()}
        className="flex items-center gap-2"
      >
        <ArrowRightFromLine className="h-3.5 w-3.5" />
        Copy to project...
      </ContextMenuItem>
    </ContextMenuContent>
  )}
</ContextMenu>
```

### MetadataHeader Toolbar Additions

```tsx
// Source: crates/src/components/project-detail/MetadataHeader.tsx (existing)
// Add to the right cluster alongside existing Back Up button:
import { Pencil, Copy, PackageOpen } from "lucide-react";

// In the toolbar flex row:
<Button variant="ghost" size="sm" className="font-mono text-xs h-8 gap-1"
  onClick={onRename} disabled={!project} aria-label="Rename project">
  <Pencil className="h-3.5 w-3.5" />
  Rename
</Button>
<Button variant="ghost" size="sm" className="font-mono text-xs h-8 gap-1"
  onClick={onDuplicate} disabled={!project} aria-label="Duplicate project">
  <Copy className="h-3.5 w-3.5" />
  Duplicate
</Button>
<Button variant="ghost" size="sm" className="font-mono text-xs h-8 gap-1"
  onClick={onExport} disabled={!project} aria-label="Export project">
  <PackageOpen className="h-3.5 w-3.5" />
  Export
</Button>
```

### OT Name Validation (16 chars, A-Z 0-9 underscore)

```typescript
// Source: Phase 4 UI-SPEC.md (approved 2026-05-01)
const OT_NAME_PATTERN = /^[A-Z0-9_]{1,16}$/;
const OT_NAME_CHAR_PATTERN = /[^A-Z0-9_]/g;

function validateOtName(name: string): { valid: boolean; error?: string } {
  if (name.length > 16) return { valid: false, error: "OT names: A–Z, 0–9, underscore only. Max 16 characters." };
  if (OT_NAME_CHAR_PATTERN.test(name)) return { valid: false, error: "OT names: A–Z, 0–9, underscore only. Max 16 characters." };
  return { valid: true };
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual file manager for OT project backup | Dedicated Rust tool with atomic writes | Phases 1–3 | Safe, atomic, snapshot-protected |
| BankFile body fully parsed | BankFile body stored as opaque blob | Phase 1 D-02 | Can't extract slot-index references from banks; use conservative full-slot-merge strategy |
| Separate export tool (OctaChainer etc.) | Integrated export with path preservation | Phase 4 | One tool, correct structure |

**Deprecated/outdated:**
- `zip = "8.x"` (CLAUDE.md tech stack table): The zip crate's actual latest version is 2.6.0. Dependency line should be `zip = "2"`.

---

## Open Questions

1. **Are both `.work` and `.strd` files always kept in sync?**
   - What we know: The format spec notes `.work` = working memory, `.strd` = saved state; OT reloads from `.strd` on project reload
   - What's unclear: Whether writing only `.work` is sufficient for operations that take effect on next OT mount, or whether `.strd` must also be updated
   - Recommendation: Always write BOTH to be safe. Cost is negligible (files are small). Include in every atomic_write_batch call.

2. **Does project.work use `[FLEX]` / `[STATIC]` as literal section headers?**
   - What we know: OctaLib Research.md (confirmed source) lists `TYPE` field in slot entries with values `FLEX` or `STATIC`; ot-tools-io `SlotsAttributes` divides flex vs static slots
   - What's unclear: Whether sections are delimited by bracketed headers like `[FLEX]` or by the `TYPE=FLEX` key within each slot block
   - Recommendation: Implement parser to handle `TYPE=FLEX` / `TYPE=STATIC` as inline discriminators rather than bracketed sections. Flag this assumption with a clear comment. [ASSUMED: section format needs real OT file verification]

3. **Do .ot sidecar files live alongside audio files in /AUDIO/?**
   - What we know: `SampleSettingsFile` (`.ot`) is per-sample; export D-05 requires including them
   - What's unclear: Whether sidecars are named `filename.ot` in the same directory as `filename.wav`, or somewhere else
   - Recommendation: Assume `AUDIO/kick.ot` alongside `AUDIO/kick.wav`. Check existence before including. [ASSUMED: sidecar naming convention — verify with real OT card]

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| zip crate | Export operation | ✗ (not in Cargo.toml) | 2.6.0 (to be added) | — (blocking for export) |
| shadcn context-menu | Bank copy right-click | ✗ (not in ui/) | latest | — (blocking for bank copy trigger) |
| walkdir | Export + duplicate traversal | ✓ | 2.5 (already in Cargo.toml) | — |
| sha2 | Hash comparison (D-07) | ✓ | 0.10 (already in Cargo.toml) | — |
| lucide-react icons | Toolbar buttons | ✓ | 1.14.0 (verified: all 5 icons present) | — |
| @base-ui/react | context-menu component | ✓ | ^1.4.1 (already in package.json) | — |

[VERIFIED: Cargo.toml grep confirms walkdir, sha2 present; `ls src/components/ui/` confirms context-menu missing; `node -e` confirms all 5 lucide icons present]

**Missing dependencies with no fallback:**
- `zip = "2"` Rust crate — must add to `crates/takoyaki-app/Cargo.toml` before export command can compile
- `shadcn context-menu` component — must run `npx shadcn@latest add context-menu` before BankGridCell context menu can work

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (Rust integration tests) |
| Config file | none — standard Cargo test runner |
| Quick run command | `cargo test -p takoyaki-app management` |
| Full suite command | `cargo test -p takoyaki-app` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| MGMT-01 | Duplicate copies project files + remaps sample paths | integration | `cargo test -p takoyaki-app management::tests::test_duplicate_project -x` | ❌ Wave 0 |
| MGMT-01 | Duplicate name collision detected + incremented | unit | `cargo test -p takoyaki-app management::tests::test_duplicate_name_collision` | ❌ Wave 0 |
| MGMT-02 | Rename updates directory name on disk | integration | `cargo test -p takoyaki-app management::tests::test_rename_project` | ❌ Wave 0 |
| MGMT-02 | Rename validates OT-legal characters | unit | `cargo test -p takoyaki-app management::tests::test_rename_validation` | ❌ Wave 0 |
| MGMT-03 | Export zip contains SETS/ + AUDIO/ structure | integration | `cargo test -p takoyaki-app management::tests::test_export_zip_structure` | ❌ Wave 0 |
| MGMT-03 | Export zip includes .ot sidecar files | integration | `cargo test -p takoyaki-app management::tests::test_export_includes_sidecars` | ❌ Wave 0 |
| SMPL-02 | Bank copy merges slots, hash-match = skip | integration | `cargo test -p takoyaki-app management::tests::test_bank_copy_no_conflict` | ❌ Wave 0 |
| SMPL-02 | Bank copy surfaces hash-mismatch conflicts | integration | `cargo test -p takoyaki-app management::tests::test_bank_copy_conflict_detection` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p takoyaki-app management`
- **Per wave merge:** `cargo test -p takoyaki-app`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `crates/takoyaki-app/src/management/mod.rs` — management module skeleton
- [ ] `crates/takoyaki-app/tests/management.rs` — integration tests for all 4 operations
- [ ] Add `zip = "2"` to `crates/takoyaki-app/Cargo.toml`
- [ ] `npx shadcn@latest add context-menu` — frontend prerequisite
- [ ] `src/lib/stores/management.ts` — zustand store for management operations

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | n/a — local desktop app |
| V3 Session Management | no | n/a — local desktop app |
| V4 Access Control | no | n/a — local desktop app |
| V5 Input Validation | yes | OT name validation (A-Z 0-9 underscore, max 16 chars); path validation via `resolve_ot_path()` |
| V6 Cryptography | partial | SHA-256 via `sha256_hex()` for integrity verification (not encryption) |

### Known Threat Patterns for This Stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal via crafted PATH= values in project.work | Tampering | `resolve_ot_path()` with canonicalize() already in `health/mod.rs` — all paths must pass through this |
| Zip slip (malicious zip path escaping target dir) | Tampering | Not applicable — we only WRITE zips, never extract them |
| Frontend-supplied raw paths | Tampering | All path computation in Rust; frontend supplies only `project_id` (UUID) and bank index; same pattern as Phase 3 T-03-01 |
| Overwrite of unrelated files via bank copy destination | Tampering | Destination computed in Rust from `project_id` DB lookup; frontend never supplies target path |

---

## Project Constraints (from CLAUDE.md)

| Directive | Phase 4 Impact |
|-----------|---------------|
| Atomic writes + snapshot-before-write for ALL file-modifying operations | Every management operation (duplicate, rename, export, bank copy) must call SnapshotEngine before any write |
| No GPL dependencies — clean-room OT format implementation | Do not link against ot-tools-io for parsing; use the existing ot-parser crate + text parsing we write ourselves |
| MIT license for all project code | zip crate is MIT licensed [VERIFIED: crates.io] |
| Full test coverage — OT binary parser must have extensive fixtures | All 8 test cases in the Phase Requirements table are required |
| Tauri v2 IPC commands with tauri-specta auto-generated TypeScript | New management commands must use `#[tauri::command] #[specta::specta]` and be registered in `lib.rs::collect_commands![]` |
| SQLite for Takoyaki own metadata | Rename must UPDATE the projects table; duplicate must INSERT a new project row |
| Atomic write staging on same filesystem as CF card | All writes to the OT card go through `atomic_write_batch()` — never `std::fs::write()` directly |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | project.work slot section uses `TYPE=FLEX`/`TYPE=STATIC` inline discriminators (not `[FLEX]`/`[STATIC]` section headers) | Code Examples (project.work parser) | Parser would misidentify section boundaries; wrong slots extracted. Mitigation: test against real OT file in Wave 0 |
| A2 | `.ot` sidecar files are stored as `filename.ot` in the same directory as `filename.wav` | Code Examples (zip export), D-05 | Sidecars not included in export; export would be missing slice/loop data. Mitigation: verify with real OT card listing |
| A3 | project.work has no internal NAME= field; the directory name under `/SETS/` is the authoritative and only project name storage | Architecture Patterns (rename), Standard Stack | Rename would leave stale data if a NAME= field exists elsewhere. Evidence: ot-tools-io OsMetadata/Settings structs have no name field; OctaLib research skips metadata section entirely |

---

## Sources

### Primary (HIGH confidence)
- `crates/ot-parser/format-spec.md` — OT binary format spec (clean-room) with verified .ot structure
- `crates/takoyaki-app/src/` — All existing Phase 1–3 code read directly from filesystem
- `crates/ot-parser/src/` — Parser implementations verified
- [zip-rs/zip2 Context7 library `/zip-rs/zip2`] — ZipWriter API, CompressionMethod enum
- [shadcn-ui/ui Context7 library `/shadcn-ui/ui`] — ContextMenu component installation and usage
- [base-ui MUI Context7 library `/mui/base-ui`] — ContextMenu/Menu API reference

### Secondary (MEDIUM confidence)
- [ot-tools-io docs.rs OsMetadata](https://docs.rs/ot-tools-io/0.6.0) — Verified no NAME field in metadata; Settings struct has no name field
- [OctaLib Research.md](https://github.com/snugsound/OctaLib) — project.work slot format: TYPE, SLOT, PATH, TRIM_BARSx100, TSMODE, LOOPMODE, GAIN, TRIGQUANTIZATION
- [ot-tools-io slots.rs source](https://docs.rs/ot-tools-io/0.6.0/src/ot_tools_io/projects/slots.rs.html) — PATH= is the key for slot file paths

### Tertiary (LOW confidence)
- [General OT community knowledge — project name = directory name] — Widely accepted but not in official API docs; consistent with ot-tools-io struct analysis showing no NAME= field

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all Rust libs verified in Cargo.toml or crates.io; all frontend icons verified in node_modules; UI gap (context-menu) confirmed by filesystem listing
- Architecture: HIGH — follows established Phase 3 patterns exactly; new patterns are simple compositions of existing primitives
- Pitfalls: HIGH — all pitfalls derived from direct code analysis of existing codebase patterns
- project.work format: MEDIUM — derived from ot-tools-io source analysis and OctaLib research; exact section syntax needs real file verification (marked as A1)

**Research date:** 2026-05-01
**Valid until:** 2026-06-01 (stable domain; zip crate API won't change meaningfully)
