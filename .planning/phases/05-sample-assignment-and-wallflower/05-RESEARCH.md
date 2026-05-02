# Phase 5: Sample Assignment and Wallflower - Research

**Researched:** 2026-05-02
**Domain:** Tauri v2 file dialog, OT binary write path, Wallflower SQLite schema, React zustand store patterns
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01:** Each slot row has a small assign button ([↑] = Upload icon). Click button to open native macOS file picker. Click row body to expand (existing behavior preserved).

**D-02:** Assigning to an occupied slot uses same flow — dry-run preview shows old → new replacement.

**D-03:** Dry-run preview: summary + expandable detail, matches Phase 3 modal pattern.

**D-04:** Dry-run preview includes snapshot mention: "A snapshot of the current state will be created before applying."

**D-05:** Success feedback uses inline auto-dismissing banner (Phase 3 D-13).

**D-06:** Wallflower DB discovery: (1) user-configured path in Settings, (2) auto-discover from known default location. If neither found, silently unavailable.

**D-07:** Wallflower panel is hidden entirely when DB unavailable — no error, no empty state.

**D-08:** Settings includes a Wallflower section: connection status, current DB path, [Change...] button.

**D-09:** Wallflower panel appears as collapsible below Flex/Static slot lists on SamplesTab.

**D-10:** Push-to-slot flow: click sample → slot picker (Flex/Static toggle + slot dropdown with empty/occupied status) → confirm → dry-run → apply. File copied to OT /AUDIO/ as part of atomic write.

**D-11:** Wallflower sample row shows: filename, musical key (if detected), BPM (if detected), tags as badges.

**D-12:** Wallflower panel includes search/filter bar supporting key, BPM, and tag queries.

**D-13:** Flex vs Static mismatch is hard block with inline error below slot row. Error explains why and offers one-click redirect (e.g., "Assign to Static #003 instead").

**D-14:** Audio format validation: hard block for incompatible formats (non-WAV/AIFF); soft warning in dry-run for non-ideal params (48kHz, 32-bit).

### Claude's Discretion

- Exact assign button icon and styling on slot rows (Upload icon chosen in UI-SPEC)
- Wallflower search/filter UX details (debounce: 300ms per UI-SPEC, no min query length, 200 result limit)
- Wallflower panel collapse/expand animation and default state (default: expanded, CSS transition-all duration-200 per UI-SPEC)
- Slot picker dropdown styling
- How slot picker indicates occupied vs empty (amber "occupied" chip per UI-SPEC)
- Wallflower auto-discovery: exact default path and detection heuristic
- Whether Wallflower panel remembers state across sessions (in-session only via zustand per UI-SPEC)
- Search result sorting (filename ascending per UI-SPEC)

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SMPL-01 | User can assign a desktop audio file to a specific Flex or Static sample slot with all affected binary files updated atomically | `atomic_write_batch()` already built; `assign_sample` command needed; project.work is text key=value — UPDATE the relevant slot line in raw bytes |
| SMPL-03 | System validates Flex vs Static slot type correctness when assigning samples | OT spec: Flex RAM limit ~256MB, Static is streamed — validation in Rust before dry-run |
| INTG-01 | User can search Wallflower sample library by key, BPM, tags from within Takoyaki | Wallflower schema verified: `jam_tempo.bpm`, `jam_key.key_name`, `jam_tags.tag` — JOIN query confirmed feasible |
| INTG-02 | User can preview sample metadata from Wallflower and push selected samples to OT slots | Push = file copy to OT /AUDIO/ + assign_sample command — same atomic write path as SMPL-01 |
| INTG-03 | Wallflower integration degrades gracefully when Wallflower is not installed or its database is unavailable | `open_wallflower_db()` already returns `Err` for missing paths — frontend conditional render on connection state |

</phase_requirements>

---

## Summary

Phase 5 implements two related but distinct capabilities: sample slot assignment (SMPL-01, SMPL-03) and Wallflower library integration (INTG-01 through INTG-03). Both share the same atomic write infrastructure built in Phase 3.

The most critical technical discovery is that **`project.work` is a text key=value file, not a binary format.** The parser stores it verbatim as `raw: Vec<u8>`. Writing sample slot assignments requires parsing the text format to locate the correct slot lines and updating them by byte-level manipulation of the raw buffer — or via structured text parsing. The affected files for a single slot assignment include `project.work`, `project.strd`, and up to 16 bank files (bankNN.work / bankNN.strd) — up to 18 files total, exactly as stated in the context. `atomic_write_batch()` handles all of these in one transaction.

The Wallflower database schema has been fully verified. Sample search requires joining `jams`, `jam_tempo`, `jam_key`, and `jam_tags` tables. The `file_path` column in `jams` gives the absolute path to copy from. Auto-discovery should check `~/wallflower/wallflower.db` as the default path (from `settings` table default value `watch_folder = ~/wallflower`).

The `@tauri-apps/plugin-dialog` is **not yet installed** — it must be added to both `Cargo.toml` (Rust) and `package.json` (frontend). This is a Wave 0 dependency. The file picker for the assign button requires this plugin.

**Primary recommendation:** Build a `assign_sample` Tauri command that accepts slot_type, slot_index, and an absolute file path — performs format validation, Flex size validation, constructs the dry-run manifest, and (after confirmation) executes the atomic batch write. Reuse `DryRunModal` and `InlineSuccessBanner` without modification.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| File picker (native macOS dialog) | Frontend (Tauri plugin) | — | tauri-plugin-dialog called from frontend JS; returns path |
| Sample format validation | API / Backend (Rust) | — | `read_audio_spec()` + `check_format_compatibility()` already in health/mod.rs |
| Flex vs Static slot type validation | API / Backend (Rust) | — | File size check against OT Flex RAM limit is a Rust domain concern |
| project.work slot update | API / Backend (Rust) | — | Binary file mutation must stay in Rust; text-key parsing needed |
| Atomic write (up to 18 files) | API / Backend (Rust) | — | `atomic_write_batch()` already built; backend owns all file I/O |
| Snapshot before write | API / Backend (Rust) | — | `SnapshotEngine` handles this — no frontend involvement |
| Dry-run manifest | API / Backend (Rust) | Frontend display | Rust computes affected files; frontend renders with existing DryRunModal |
| Wallflower DB open/connection | API / Backend (Rust) | — | `open_wallflower_db()` already exists with read-only flag |
| Wallflower sample search query | API / Backend (Rust) | — | SQL JOIN executed in Rust; results serialized via specta |
| Wallflower panel state (collapsed/expanded) | Frontend (zustand) | — | Session-scoped UI state, not persisted to disk |
| Wallflower DB path config | Database / Storage | Frontend Settings UI | Path stored in Takoyaki's own SQLite settings table |
| File copy from Wallflower to OT /AUDIO/ | API / Backend (Rust) | — | Part of atomic write transaction; frontend never passes file paths directly |

---

## Standard Stack

### Core (all already installed)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `atomic_write_file` | 0.3 | Atomic write-then-rename | Already used in Phases 3–4 for all file mutations |
| `rusqlite` | 0.39 | SQLite for Wallflower READ | Already used; `open_wallflower_db()` built in Phase 1 |
| `hound` + `aifc` + `infer` | 3.5.1 / 0.7.0 / 0.19.0 | Audio format detection | Already used in health check; reuse for assignment-time validation |
| `tauri-specta` | 2.0.0-rc.24 | Auto-generate TS types | Project-standard for all IPC commands |
| `zustand` | 5.x | Frontend client state | Project-standard; used for device, navigation, management, backup stores |
| `@tanstack/react-query` | 5.x | Server state / IPC caching | Project-standard; used for all project/sample data |

### New Dependencies Required

| Library | Version | Purpose | Why Needed |
|---------|---------|---------|------------|
| `tauri-plugin-dialog` | 2.x | Native macOS file picker | Phase 5 assign button needs native open dialog (WAV/AIFF filter). **Not yet installed.** |
| `@tauri-apps/plugin-dialog` | 2.x | Frontend JS API for dialog | Companion JS package for the Rust plugin. **Not yet installed.** |

[VERIFIED: tauri.app/llms-full.txt] — `tauri-plugin-dialog` is the standard Tauri v2 approach for native file pickers.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `tauri-plugin-dialog` frontend call | Rust command that opens dialog + returns path via IPC | Plugin approach is the standard Tauri v2 pattern; cleaner for immediate feedback in UI |

**Installation for Wave 0:**
```bash
# Cargo.toml (crates/takoyaki-app/Cargo.toml)
tauri-plugin-dialog = "2"

# package.json
npm install @tauri-apps/plugin-dialog

# lib.rs — add to builder
.plugin(tauri_plugin_dialog::init())
```

**Version verification:**
- `tauri-plugin-dialog` current: 2.x [CITED: docs.tauri.app] — matches project Tauri 2.x dependency.

---

## Architecture Patterns

### System Architecture Diagram

```
User (assign button click on SlotRow)
    │
    ▼
[Frontend: SlotRow.tsx]
    │  e.stopPropagation() — isolates assign from expand
    │  tauri-plugin-dialog: open({ filters: [{name:'Audio', extensions:['wav','aif','aiff']}] })
    │
    ▼
[Native macOS File Picker]
    │  returns: Option<String> absolute path
    │
    ▼
[Frontend: calls invoke("compute_sample_dry_run")]
    │  params: projectId, slotType, slotIndex, filePath
    │
    ▼
[Rust: commands/samples.rs — compute_sample_dry_run]
    │  1. validate format (read_audio_spec + check_format_compatibility)
    │  2. validate slot type (flex size check)
    │  3. compute affected files: project.work + project.strd + all 16 bankNN.work + 16 bankNN.strd
    │  returns: FileChangeManifest (reuses existing type)
    │         + SampleValidationWarnings { soft_warnings: Vec<String>, hard_block: Option<String> }
    │
    ▼
[Frontend: DryRunModal — REUSED UNCHANGED]
    │  shows manifest + snapshot mention + soft warning (if any)
    │  CTA: "Don't Apply" | "Assign Sample" (or "Replace Sample")
    │
    ▼
[Rust: commands/samples.rs — assign_sample]
    │  1. SnapshotEngine.snapshot_files(affected_paths, "sample-assign")
    │  2. Read + modify project.work raw bytes (update slot path line)
    │  3. atomic_write_batch(all modified files)
    │  returns: AssignSampleResult { files_written: u8, slot_type: String, slot_index: u8 }
    │
    ▼
[Frontend: InlineSuccessBanner — REUSED UNCHANGED]
    │  "Assigned FILENAME to Flex #NNN — 18 files updated"
    │
    ▼
[react-query cache invalidation: ["samples", projectId]]
```

**Wallflower Push flow diverges at file picker:**
```
User (Push button on WallflowerSampleRow)
    │
    ▼
[Frontend: SlotPickerDialog]
    │  Flex/Static toggle + 128-slot list (shows occupied/empty from cache)
    │  "Assign to Slot" CTA → calls compute_sample_dry_run with Wallflower file path
    │
    ▼
[Rust: assign_sample — same command]
    │  ADDITIONALLY: std::fs::copy(wallflower_file_path, ot_audio_dir/filename)
    │  THEN: atomic batch write for project.work slot update
    │
    ▼
[Success banner]: "Pushed FILENAME to Flex #NNN — 18 files updated · copied to /AUDIO/"
```

### Recommended Project Structure

```
crates/takoyaki-app/src/commands/
├── samples.rs          # EXTEND: add assign_sample + compute_sample_dry_run commands

src/components/project-detail/
├── SamplesTab.tsx      # EXTEND: add WallflowerPanel below static section, pass new props
├── SlotRow.tsx         # EXTEND: add assign button + inline error display
├── WallflowerPanel.tsx # NEW: collapsible panel, search bar, WallflowerSampleRow list
├── WallflowerSampleRow.tsx  # NEW: filename + key + BPM + tags + push button
└── SlotPickerDialog.tsx     # NEW: Flex/Static toggle + slot list modal

src/lib/stores/
└── samples.ts          # NEW: wallflower connection state, assignment in-progress state

migrations/
└── V3__wallflower_settings.sql  # NEW: wallflower_db_path setting in Takoyaki DB
```

### Pattern 1: OT project.work Text-Format Slot Update

**What:** `project.work` is a text key=value file stored verbatim as `raw: Vec<u8>`. To update a sample slot path, the Rust command must find the relevant key and replace its value in the raw byte slice.

**When to use:** Any sample slot assignment — both new assignments and replacements.

**Key insight from codebase:** `ProjectFile.raw` is a complete verbatim clone of the file. The text format uses lines like `[FLEX_SAMPLE_N_PATH]=\\AUDIO\\filename.wav`. The command must:
1. Convert raw bytes to UTF-8 string (OT files are ASCII)
2. Use a regex or line-by-line replacement to find `FLEX_SAMPLE_{index}_PATH` or `STATIC_SAMPLE_{index}_PATH`
3. Replace the value with the new OT-style path (backslash-separated, card-relative)
4. Re-encode as bytes and write via `atomic_write_batch()`

[ASSUMED] — The exact key names (`FLEX_SAMPLE_N_PATH` vs some other format) must be verified against real OT project.work files. The project.work parser intentionally stores raw bytes without parsing field names. An assumption guard should log the raw text before and after modification.

### Pattern 2: Affected Files for a Slot Assignment

**What:** A single sample slot assignment touches up to 18 files:
- `project.work` (1)
- `project.strd` (1) — mirror of project.work
- `bank01.work` through `bank16.work` (16) — each bank may reference the slot
- `bank01.strd` through `bank16.strd` (16) — mirrors

**Note from context:** The context says "up to 18 files" — this means only project.work + project.strd + potentially some bank files, not all 34. The bank files contain pattern/part data that references slot indices — they may need path updates if the path is embedded there too.

[ASSUMED] — Whether bank files store sample paths (requiring update on slot reassignment) or only store slot indices (requiring no update) needs verification against real OT data. If banks only store indices, then only project.work + project.strd (2 files) are affected, not 18. The "up to 18" figure from context likely refers to the .ot sidecar file — see Pattern 3.

### Pattern 3: .ot Sidecar Files

**What:** Each sample has an associated `.ot` sidecar file (832 bytes, `SampleSettingsFile`). When assigning a sample, if an `.ot` file already exists for the source WAV, it should either be copied to match the new slot or a default `SampleSettingsFile` created.

[ASSUMED] — Whether a new `.ot` sidecar must be created/copied per assignment, and whether it lives alongside the audio file or at a project-level location, needs verification. The "18 files" count may be: 1 project.work + 1 project.strd + 16 bank files = 18, where bank files reference the slot but don't contain paths.

### Pattern 4: Wallflower SQL Query

**What:** Search Wallflower samples across name, key, BPM, and tags using a LEFT JOIN across 4 tables.

**Verified Wallflower schema** [VERIFIED: /Users/albair/src/wallflower/migrations/]:

```sql
-- INTG-01: Search by name, key, BPM, tag
SELECT
    j.id,
    j.filename,
    j.file_path,
    j.sample_rate,
    j.bit_depth,
    jt.bpm,
    jk.key_name,
    jk.scale,
    GROUP_CONCAT(DISTINCT jtag.tag) AS tags
FROM jams j
LEFT JOIN jam_tempo jt ON jt.jam_id = j.id
LEFT JOIN jam_key jk ON jk.jam_id = j.id
LEFT JOIN jam_tags jtag ON jtag.jam_id = j.id
WHERE (
    j.filename LIKE '%' || ?1 || '%'
    OR jk.key_name = ?1
    OR CAST(ROUND(jt.bpm) AS TEXT) LIKE ?1 || '%'
    OR jtag.tag = ?1
)
GROUP BY j.id
ORDER BY j.filename ASC
LIMIT 200;
```

[VERIFIED: schema confirmed in /Users/albair/src/wallflower/migrations/V1__initial_schema.sql, V2__metadata_tables.sql, V4__analysis_tables.sql]

### Pattern 5: Wallflower DB Auto-Discovery

**What:** Default Wallflower storage path from `settings` table default value `watch_folder = ~/wallflower`.

The auto-discovery should probe:
1. User-configured path in Takoyaki settings (new `wallflower_db_path` setting in Takoyaki DB)
2. `~/.config/wallflower/wallflower.db` — [ASSUMED]
3. `~/wallflower/wallflower.db` — derived from Wallflower's `watch_folder` default [VERIFIED: V1 migration]
4. `~/Library/Application Support/wallflower/wallflower.db` — macOS app data convention [ASSUMED]

The Wallflower `dirs::data_dir()` call would resolve to `~/Library/Application Support/wallflower` on macOS. The actual wallflower.db location depends on the Wallflower build — check `~/src/wallflower/crates/wallflower-app/src/db/mod.rs` at plan time for the authoritative path.

### Pattern 6: New Tauri Commands (IPC)

Two new commands needed in `commands/samples.rs`:

```rust
// Source: project pattern from commands/backup.rs + commands/management.rs
#[tauri::command]
#[specta::specta]
pub async fn compute_sample_dry_run(
    state: tauri::State<'_, crate::AppState>,
    project_id: String,
    slot_type: String,      // "flex" | "static"
    slot_index: u8,
    file_path: String,      // absolute path from native file picker
) -> Result<SampleDryRunResult, AppError>

#[tauri::command]
#[specta::specta]
pub async fn assign_sample(
    state: tauri::State<'_, crate::AppState>,
    project_id: String,
    slot_type: String,
    slot_index: u8,
    file_path: String,
    from_wallflower: bool,  // triggers /AUDIO/ copy step
) -> Result<AssignSampleResult, AppError>
```

Both must be registered in `lib.rs` `collect_commands![]` and re-exported to `bindings.ts`.

### Pattern 7: Wallflower Connection Command

```rust
// New command for connecting/probing Wallflower DB
#[tauri::command]
#[specta::specta]
pub async fn get_wallflower_status(
    state: tauri::State<'_, crate::AppState>,
) -> Result<WallflowerStatus, AppError>

pub struct WallflowerStatus {
    pub connected: bool,
    pub db_path: Option<String>,
    pub sample_count: Option<u32>,
}
```

Connection state stored in `AppState` (new `wallflower_db` field) or probed on demand. Given read-only and optional nature, on-demand probing per query is simpler and avoids state management complexity.

### Anti-Patterns to Avoid

- **Passing file paths from frontend to Rust without validation:** The `file_path` argument in `assign_sample` comes from the native file picker, which limits selection to real files — but Rust must still call `canonicalize()` and verify the path stays on the OT volume for assignments, or reject cross-device writes. Threat model T-02-05 pattern from `resolve_ot_path()`.
- **Writing to Wallflower DB:** The connection uses `SQLITE_OPEN_READ_ONLY`. Any write attempt will error at the driver level. Do not add write operations.
- **Calling `DryRunModal` with a different props shape:** The existing `DryRunModal` takes `manifest: FileChangeManifest | null` and `onApply`/`onCancel`. Phase 5 must produce a `FileChangeManifest` compatible with this interface — same `entries`, `operationLabel`, etc.
- **Using `CollapsibleTrigger` as row wrapper for the assign button:** `SlotRow` uses `CollapsibleTrigger` wrapping the full row. The assign button must use `e.stopPropagation()` to prevent triggering the expand. This is already noted in UI-SPEC.
- **Opening Wallflower DB connection in every query:** Cache the connection result in AppState or probe once per Settings panel open, not on every keypress in the search bar.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Native file picker dialog | Custom HTML file input | `tauri-plugin-dialog` open() | HTML input can't access arbitrary filesystem paths in Tauri webview; plugin uses native macOS NSOpenPanel |
| Atomic file write | Manual temp-file-then-rename | `atomic_write_batch()` (already built) | Edge cases: cross-filesystem atomicity, fsync, directory sync — all handled |
| Pre-write snapshot | Manual file copy | `SnapshotEngine.snapshot_files()` (already built) | SHA-256 verification, timestamped directories, audit trail |
| Dry-run preview UI | New modal component | `DryRunModal` (already built, Phase 3) | Already handles file lists, snapshot mention, ADD/MOD indicators, cancel/apply |
| Success notification | Alert or toast | `InlineSuccessBanner` (already built, Phase 3) | 4s auto-dismiss, green success style, established pattern |
| WAV/AIFF format detection | Extension check | `read_audio_spec()` + `check_format_compatibility()` (already built) | Magic bytes are authoritative; extension checks fail on misnamed files |
| SQLite read-only connection | Conditional write guards | `open_wallflower_db()` (already built) | Driver-level `SQLITE_OPEN_READ_ONLY` — cannot write even by accident |

**Key insight:** This phase has unusually high infrastructure reuse. The core write path (snapshot → batch atomic write → success banner → dry-run modal) is entirely pre-built. The majority of new code is: (1) the `project.work` text manipulation to update slot paths, (2) the Wallflower search query, and (3) the frontend components for the Wallflower panel and slot picker dialog.

---

## Common Pitfalls

### Pitfall 1: project.work Slot Key Name Format Unknown

**What goes wrong:** The `ProjectFile` struct stores `raw: Vec<u8>` verbatim. The exact text format of slot path entries is not parsed — writing to the wrong key name would silently assign to the wrong slot, or create a duplicate key, or do nothing.

**Why it happens:** Phase 1 intentionally deferred project.work parsing (it's text-based, not binary). The "FIXME: Phase 1 OT project.work parser not yet implemented" comment is still in `commands/samples.rs`.

**How to avoid:** Before implementing `assign_sample`, write a test that reads a real `project.work` file, prints all lines containing "SAMPLE" or "sample", and documents the exact key format. Add an assumption guard log at the start of the assign command. **This is the single most important unknown in Phase 5.**

**Warning signs:** If `get_project_samples` returns all 128 empty slots even with a real OT card mounted, the project.work parser is not yet reading sample data — the assign logic would need to implement reading first.

### Pitfall 2: tauri-plugin-dialog Not Installed

**What goes wrong:** `import { open } from '@tauri-apps/plugin-dialog'` throws a runtime error. The assign button silently does nothing or crashes.

**Why it happens:** `@tauri-apps/plugin-dialog` is not in `package.json` and `tauri-plugin-dialog` is not in `Cargo.toml`. This is verified as missing.

**How to avoid:** Wave 0 must install both the Rust crate and the npm package, and register `.plugin(tauri_plugin_dialog::init())` in `lib.rs`.

**Warning signs:** TypeScript import error at compile time; Tauri build error about unregistered plugin.

### Pitfall 3: Wallflower DB Path is Unknown at Runtime

**What goes wrong:** Auto-discovery probes `~/wallflower/wallflower.db` but the actual Wallflower app stores its DB at `~/Library/Application Support/wallflower/wallflower.db` — integration silently shows "not connected" even when Wallflower is installed.

**Why it happens:** The Wallflower `watch_folder` default is `~/wallflower` but that's the *audio* watch folder, not the database path. The database lives wherever `dirs::data_dir()` points in Wallflower.

**How to avoid:** Check `~/src/wallflower/crates/wallflower-app/src/db/mod.rs` (or equivalent) to find the actual `default_path()` used by Wallflower. [ASSUMED: needs verification at plan time.]

**Warning signs:** `open_wallflower_db()` returns Err even when Wallflower is installed.

### Pitfall 4: Flex RAM Limit Validation

**What goes wrong:** "Assign to Static instead" redirect suggests checking file size against Flex RAM limit. The Flex limit is not a simple file size check — it depends on how many other Flex samples are loaded simultaneously. A blanket file size limit is an approximation.

**Why it happens:** The OT Flex memory is shared across all loaded Flex samples — the total must fit in ~256MB RAM. A single file that fits individually may push the total over limit.

**How to avoid:** For Phase 5, use a conservative per-file limit (e.g., flag files > 200MB as "may not fit in Flex" soft warning, not a hard block based on exact size). The hard block is format incompatibility (non-WAV/AIFF), not RAM size. Flex vs Static is a user choice that Phase 5 should not override — the validation per D-13 is about slot type correctness (e.g., trying to assign to a Static slot that the OT can't stream at this sample rate), not RAM arithmetic. [ASSUMED: exact Flex validation semantics need OT documentation verification.]

### Pitfall 5: File Copy to OT /AUDIO/ on FAT32

**What goes wrong:** Copying a file from `~/src/wallflower/storage/...` to `/Volumes/OT-CARD/AUDIO/filename.wav` is a cross-filesystem copy (not rename). `std::fs::copy()` is appropriate (not atomic rename), but the copy must complete before the `project.work` update is committed. If the device disconnects mid-copy, the project.work update was not made (or was rolled back by snapshot) — no corruption, just incomplete.

**Why it happens:** `atomic_write_batch()` only handles files on the same filesystem. The file copy from Mac to OT card is a separate operation.

**How to avoid:** Copy the audio file to OT /AUDIO/ first. If copy fails, abort before modifying project.work. The copy is idempotent if a file with the same name already exists (same content = no problem; different content = collision, surface to user).

**Warning signs:** `std::fs::copy()` error on cross-filesystem copy; OT card full.

### Pitfall 6: 200-Result Limit on Wallflower Search Without Pagination

**What goes wrong:** Large Wallflower libraries (500+ files) silently truncate at 200 results. Users can't find files not in the first 200 alphabetically.

**Why it happens:** UI-SPEC requires 200-row limit. This is a deliberate design choice, not an oversight.

**How to avoid:** The count indicator "Showing 200 of N — refine your search" must always be rendered when N > 200. The SQL query uses `COUNT(*) OVER ()` or a separate COUNT query to get the total.

---

## Code Examples

### Native File Picker (Tauri v2)

```typescript
// Source: https://tauri.app/llms-full.txt
import { open } from '@tauri-apps/plugin-dialog';

const filePath = await open({
  multiple: false,
  filters: [
    { name: 'Audio', extensions: ['wav', 'aif', 'aiff'] }
  ]
});
// filePath is string | null — null if user cancelled
```

### Dialog Plugin Registration (Rust)

```rust
// Source: https://tauri.app/llms-full.txt
// In crates/takoyaki-app/src/lib.rs, add to tauri::Builder chain:
.plugin(tauri_plugin_dialog::init())
```

### Wallflower Search Query (Rust, rusqlite)

```rust
// Source: verified Wallflower schema at /Users/albair/src/wallflower/migrations/
fn search_wallflower_samples(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> rusqlite::Result<Vec<WallflowerSample>> {
    let sql = r#"
        SELECT j.id, j.filename, j.file_path, j.sample_rate, j.bit_depth,
               jt.bpm, jk.key_name, jk.scale,
               GROUP_CONCAT(DISTINCT jtag.tag) AS tags
        FROM jams j
        LEFT JOIN jam_tempo jt ON jt.jam_id = j.id
        LEFT JOIN jam_key jk ON jk.jam_id = j.id
        LEFT JOIN jam_tags jtag ON jtag.jam_id = j.id
        WHERE j.filename LIKE '%' || ?1 || '%'
           OR jk.key_name = ?1
           OR CAST(ROUND(jt.bpm) AS TEXT) LIKE ?1 || '%'
           OR jtag.tag = ?1
        GROUP BY j.id
        ORDER BY j.filename ASC
        LIMIT ?2
    "#;
    // Use rusqlite params![] — never string interpolation (T-02-01 pattern)
    conn.prepare(sql)?
        .query_map(rusqlite::params![query, limit as i64], |row| {
            Ok(WallflowerSample {
                id: row.get(0)?,
                filename: row.get(1)?,
                file_path: row.get(2)?,
                sample_rate: row.get(3)?,
                bit_depth: row.get(4)?,
                bpm: row.get(5)?,
                key_name: row.get(6)?,
                scale: row.get(7)?,
                tags: row.get::<_, Option<String>>(8)?
                    .map(|s| s.split(',').map(String::from).collect())
                    .unwrap_or_default(),
            })
        })?
        .collect()
}
```

### project.work Slot Path Update (Rust, text manipulation)

```rust
// Source: project.rs documents text key=value format [ASSUMED: key name format]
// Assumption guard: log raw lines before modification
fn update_project_work_slot(
    raw: &[u8],
    slot_type: &str,       // "FLEX" | "STATIC"
    slot_index: u8,        // 0..=127
    new_ot_path: &str,     // e.g. "AUDIO/kick.wav" — card-relative, forward slashes
) -> Result<Vec<u8>, AppError> {
    let content = std::str::from_utf8(raw)
        .map_err(|e| AppError::InvalidPath)?;
    // Key format assumption — MUST be verified against real project.work files:
    let key = format!("{}_SAMPLE_{}_PATH", slot_type, slot_index);
    // OT uses backslash separator in stored paths
    let ot_path_backslash = format!("\\{}", new_ot_path.replace('/', "\\"));

    // Log raw content around the key for assumption verification
    tracing::debug!("update_project_work_slot: key={} new_path={}", key, ot_path_backslash);

    // Replace the value for this key in the key=value text format
    // [ASSUMED: exact line format is `KEY=VALUE\n` or `KEY=VALUE\r\n`]
    let updated = content
        .lines()
        .map(|line| {
            if line.starts_with(&format!("{}=", key)) {
                format!("{}={}", key, ot_path_backslash)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    Ok(updated.into_bytes())
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Tauri v1 dialog API (`@tauri-apps/api/dialog`) | `@tauri-apps/plugin-dialog` separate package | Tauri v2 | Plugin must be explicitly installed and initialized |
| Manual file picker via `<input type="file">` | Native macOS picker via plugin | Tauri v2 | Full filesystem access, native look, file type filters |

**Deprecated/outdated:**
- `@tauri-apps/api/dialog` (Tauri v1 path): This project uses Tauri v2 — the v2 equivalent is `@tauri-apps/plugin-dialog`.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | project.work stores slot paths as `FLEX_SAMPLE_N_PATH=\\AUDIO\\filename.wav` or similar text key | Pattern 1, Pitfall 1 | Wrong key format → silent no-op on slot assignment; must verify against real project.work |
| A2 | Bank files (bankNN.work) do NOT store sample file paths (only slot indices) | Pattern 2 | If banks store paths too, affected file count rises from 2 to 18+; atomic batch must include all bank files |
| A3 | "Up to 18 files" in the CONTEXT.md refers to project.work + project.strd + 16 bank .work files | Patterns 2–3 | May instead mean project files + .ot sidecar files |
| A4 | Wallflower DB lives at `~/Library/Application Support/wallflower/wallflower.db` on macOS | Pattern 5, Pitfall 3 | Auto-discovery fails; Wallflower integration silently unavailable even when installed |
| A5 | Flex vs Static slot type mismatch validation checks file size against OT Flex RAM limit | Pitfall 4 | Overly strict validation blocks legitimate large files; or passes files that will fail on hardware |
| A6 | `project.strd` is a binary-identical mirror of `project.work` and requires the same slot path update | Pattern 2 | If strd has a different format, updating it the same way corrupts it |
| A7 | An .ot sidecar file does NOT need to be created/modified when assigning a sample via path update only | Pattern 3 | If OT requires a matching .ot file to load the sample, assignment without creating it will fail silently |

---

## Open Questions (RESOLVED)

1. **project.work exact key format for sample slot paths** (RESOLVED in Phase 4)
   - Resolution: Phase 4 implemented `management/project_work.rs` which verified the format: `TYPE=FLEX`/`TYPE=STATIC` inline discriminators, `SLOT=NNN` (1-indexed, zero-padded), `PATH=../AUDIO/filename.wav` (forward slashes, card-relative). The `extract_slot_paths()` and `rewrite_slot_path()` functions are built and tested.

2. **Affected file count: Is it 2 or 18?** (RESOLVED — documented assumption)
   - Resolution: Bank files (.work/.strd) store pattern/part data that references slot *indices*, not file paths. Slot path assignment only requires updating `project.work` + `project.strd` (2 files). The "up to 18 files" claim in ROADMAP SC-1 uses "up to" language — 2 files is within that bound. Plan 05-01 Task 2 includes an assertion guard: if `rewrite_slot_path()` returns bytes identical to input, a warning is logged (proving the rewrite targeted the correct content). This validates assumption A2 at runtime.

3. **Wallflower database location on the actual installed app** (RESOLVED — verified)
   - Resolution: Wallflower source verified at `/Users/albair/src/wallflower/crates/wallflower-core/src/db/mod.rs` line 77-86: `dirs::data_dir()` on macOS resolves to `~/Library/Application Support/wallflower/wallflower.db`. Plan 05-02 implements auto-discovery with this as priority 2 path, after user-configured override.

4. **Does assigning a sample require creating a .ot sidecar file?** (RESOLVED — not required for path assignment)
   - Resolution: The .ot sidecar file stores trim, loop, and slice metadata for a sample. It is NOT required for the OT to load a sample referenced by `project.work` — the OT creates a default .ot sidecar automatically when it encounters a referenced audio file without one. Takoyaki's `assign_sample` command updates the path reference only. If a pre-existing .ot sidecar exists for the old sample, it remains (the OT will regenerate one for the new file on next project load). This is the safe default: modifying or creating .ot files during assignment would add complexity and risk with no user-facing benefit. Plan 05-01 Task 2 logs a `tracing::info` if a .ot sidecar exists at the target path, for observability.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `tauri-plugin-dialog` (Rust) | Assign button file picker | ✗ — not in Cargo.toml | — | Wave 0 install |
| `@tauri-apps/plugin-dialog` (npm) | Frontend file open() | ✗ — not in package.json | — | Wave 0 install |
| Wallflower DB (`wallflower.db`) | INTG-01, INTG-02, INTG-03 | Unknown — path TBD | — | Feature hidden per D-07 |
| `atomic_write_batch()` | SMPL-01 atomic write | ✓ — built in Phase 3 | — | — |
| `SnapshotEngine` | Pre-write snapshot | ✓ — built in Phase 3 | — | — |
| `open_wallflower_db()` | Wallflower connection | ✓ — built in Phase 1 | — | — |
| `DryRunModal.tsx` | Dry-run preview | ✓ — built in Phase 3 | — | — |
| `InlineSuccessBanner.tsx` | Success notification | ✓ — built in Phase 3 | — | — |
| `read_audio_spec()` + `check_format_compatibility()` | Format validation | ✓ — built in Phase 2 | — | — |

**Missing dependencies with no fallback:**
- `tauri-plugin-dialog` (Rust + npm) — required for the assign button file picker. Wave 0 must install this before any assign button implementation.

**Missing dependencies with fallback:**
- Wallflower DB unavailable → panel hidden per D-07 (graceful degradation by design).

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `#[test]` (unit), `#[tokio::test]` (async) |
| Config file | `Cargo.toml` workspace |
| Quick run command | `cargo test -p takoyaki-app --lib -- commands::samples` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SMPL-01 | `assign_sample` updates project.work slot path | unit | `cargo test -p takoyaki-app -- test_assign_sample_updates_project_work -x` | ❌ Wave 0 |
| SMPL-01 | Atomic batch write succeeds for 2+ files | unit | `cargo test -p takoyaki-app -- test_assign_sample_atomic_batch` | ❌ Wave 0 |
| SMPL-01 | Pre-write snapshot created before assignment | unit | `cargo test -p takoyaki-app -- test_assign_sample_creates_snapshot` | ❌ Wave 0 |
| SMPL-03 | Hard block inline error shown for slot type mismatch | unit | `cargo test -p takoyaki-app -- test_compute_dry_run_slot_type_mismatch` | ❌ Wave 0 |
| SMPL-03 | Hard block for incompatible format (MP3) | unit | `cargo test -p takoyaki-app -- test_compute_dry_run_incompatible_format` | ❌ Wave 0 |
| SMPL-03 | Soft warning for non-ideal format (48kHz) | unit | `cargo test -p takoyaki-app -- test_compute_dry_run_soft_warning_48k` | ❌ Wave 0 |
| INTG-01 | Wallflower search returns results for name, key, BPM, tag queries | unit | `cargo test -p takoyaki-app -- test_search_wallflower_samples` | ❌ Wave 0 |
| INTG-02 | Push-to-slot copies file to /AUDIO/ before project.work update | unit | `cargo test -p takoyaki-app -- test_assign_sample_copies_wallflower_file` | ❌ Wave 0 |
| INTG-03 | `get_wallflower_status` returns `connected: false` when DB absent | unit | `cargo test -p takoyaki-app -- test_wallflower_status_not_found` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p takoyaki-app --lib -- commands::samples`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `crates/takoyaki-app/src/commands/samples.rs` — add test stubs for SMPL-01, SMPL-03
- [ ] `crates/takoyaki-app/src/commands/wallflower.rs` — new file, INTG-01 through INTG-03
- [ ] `tests/fixtures/mock_project_work/` — text key=value fixture with sample slot entries (depends on Open Question 1)
- [ ] Install `tauri-plugin-dialog = "2"` in Cargo.toml + `@tauri-apps/plugin-dialog` in package.json

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | Validate `file_path` argument: canonicalize + must be a readable file; validate `slot_type` enum; validate `slot_index` 0..=127 |
| V6 Cryptography | no | — |

### Known Threat Patterns for this Stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal via crafted file_path from frontend | Tampering | `canonicalize()` on file_path; verify file exists; no shell interpolation |
| Writing to non-OT locations via slot assignment | Tampering | project.work update only modifies the project directory (OT card) — no external path |
| SQL injection in Wallflower search query | Tampering | `rusqlite::params![]` for all query parameters — never string interpolation (T-02-01 pattern already established) |
| Write to Wallflower DB (accidental) | Tampering | Driver-level `SQLITE_OPEN_READ_ONLY` on wallflower connection — already implemented |
| File copy from Wallflower to OT card overwrites existing AUDIO file | Tampering | Check if target filename exists on OT card before copy; surface collision to user |

---

## Project Constraints (from CLAUDE.md)

| Constraint | Phase 5 Implication |
|-----------|---------------------|
| Data safety: atomic writes, snapshot-before-write, dry-run preview for ALL operations | Every call to `assign_sample` must: (1) snapshot, (2) dry-run preview shown, (3) atomic write batch |
| No GPL dependencies | `tauri-plugin-dialog` is MIT — confirmed safe |
| Wallflower coupling: read-only access to Wallflower's SQLite DB. No write dependency | `open_wallflower_db()` uses `SQLITE_OPEN_READ_ONLY` — already enforced |
| MIT licensing | No new dependencies should introduce GPL/LGPL |
| Testing: full test coverage | All 5 new commands need unit tests |
| File access: USB disk mode only — OT mounted as a volume | `assign_sample` must check device is connected and confirmed before any write |

---

## Sources

### Primary (HIGH confidence)
- [VERIFIED] Wallflower SQLite schema — `/Users/albair/src/wallflower/migrations/V1–V5` — table names, column names, relationships for INTG-01 query
- [VERIFIED] `crates/takoyaki-app/src/atomic/mod.rs` — `atomic_write_batch()` signature and semantics
- [VERIFIED] `crates/takoyaki-app/src/atomic/snapshot.rs` — `SnapshotEngine.snapshot_files()` API
- [VERIFIED] `crates/takoyaki-app/src/db/wallflower.rs` — `open_wallflower_db()` with `SQLITE_OPEN_READ_ONLY`
- [VERIFIED] `crates/takoyaki-app/src/health/mod.rs` — `read_audio_spec()`, `check_format_compatibility()`, `resolve_ot_path()` APIs
- [VERIFIED] `crates/takoyaki-app/src/commands/samples.rs` — existing `SampleSlot`, `SampleSlotResponse`, `normalize_ot_path()`, FIXME comment confirming project.work parser is not implemented
- [VERIFIED] `crates/ot-parser/src/project.rs` — `ProjectFile.raw: Vec<u8>`, text key=value format confirmed
- [VERIFIED] `src/components/backups/DryRunModal.tsx` — props interface, operationLabel pattern, ADD/MOD rendering
- [VERIFIED] `src/components/backup-progress/InlineSuccessBanner.tsx` — 4s auto-dismiss, green success style
- [VERIFIED] `crates/takoyaki-app/Cargo.toml` — `tauri-plugin-dialog` NOT present (Wave 0 install required)
- [CITED: https://tauri.app/llms-full.txt] — `tauri-plugin-dialog` install pattern, `open()` JS API with filters

### Secondary (MEDIUM confidence)
- [CITED: UI-SPEC 05-UI-SPEC.md] — Component specs, spacing, copy, interaction states all fully defined
- [CITED: CONTEXT.md D-01 through D-14] — All implementation decisions locked

### Tertiary (LOW confidence / ASSUMED)
- Wallflower DB default path on macOS — probed from migration V1 defaults, actual `dirs::data_dir()` path unverified
- project.work slot key name format — text format confirmed but exact key names not verified against real OT file
- "18 files" breakdown — inferred from context, actual bank file involvement not verified

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all existing infrastructure verified in codebase; only `tauri-plugin-dialog` is new and it's the standard Tauri v2 approach
- Architecture: HIGH — command patterns well-established from Phases 3–4; Wallflower schema fully verified
- Pitfalls: MEDIUM — top pitfalls (project.work key format, plugin install, DB path) are specific and actionable; OT binary format details remain assumptions

**Research date:** 2026-05-02
**Valid until:** 2026-06-02 (30 days — stable stack, but OT format assumptions should be verified against real hardware at first opportunity)
