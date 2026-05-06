# Phase 7: Parser Integration — Replace Stub Data - Pattern Map

**Mapped:** 2026-05-06
**Files analyzed:** 5 files to be modified
**Analogs found:** 5 / 5

---

## File Classification

| Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---------------|------|-----------|----------------|---------------|
| `crates/takoyaki-app/src/commands/samples.rs` | command/service | file-I/O → request-response | itself (replace inner function `parse_sample_slots`) | exact (self-modification) |
| `crates/takoyaki-app/src/commands/projects.rs` | command/service | file-I/O → request-response | `commands/samples.rs` get_project_samples | role-match |
| `crates/takoyaki-app/src/commands/health.rs` | command/service | file-I/O → event-driven | `commands/samples.rs` get_project_samples (lock-drop pattern) | role-match |
| `tests/fixtures/mock_ot_volume/SETS/LIVESET/PROJECT_01/project.work` | fixture | file-I/O | `tests/fixtures/project.work` (the real synthetic fixture) | exact |
| `crates/takoyaki-app/tests/project_detail.rs` | test | batch | `crates/takoyaki-app/tests/health_check.rs` | role-match |

---

## Pattern Assignments

### `crates/takoyaki-app/src/commands/samples.rs` (replace `parse_sample_slots`)

**Change scope:** Replace the `parse_sample_slots` private function and its call site in `get_project_samples`. The new parser (`parse_project_work`) parses the real `FLEX0:path` / `STAT0:path` format under a `[SLOTS]` section header. The existing `get_project_samples` command structure, DB lock pattern, and `SampleSlotResponse` shape stay unchanged.

**Analog for new parser:** `management/project_work.rs` lines 47–75 — this is the closest existing parse loop (line-by-line, `String::from_utf8_lossy`, state machine, `strip_prefix`).

**Existing section-state machine pattern** (`management/project_work.rs` lines 47–75):
```rust
pub fn extract_slot_paths(project_work_bytes: &[u8]) -> Vec<SlotPath> {
    let text = String::from_utf8_lossy(project_work_bytes);
    let mut results = Vec::new();
    let mut current_type: Option<SlotType> = None;
    let mut current_slot: Option<u8> = None;

    for line in text.lines() {
        let line = line.trim();
        if line == "TYPE=FLEX" {
            current_type = Some(SlotType::Flex);
            current_slot = None;
        } else if let Some(rest) = line.strip_prefix("SLOT=") {
            current_slot = rest.trim().parse().ok();
        } else if let Some(rest) = line.strip_prefix("PATH=") {
            if let (Some(slot_type), Some(slot_number)) = (current_type, current_slot) {
                results.push(SlotPath { slot_type, slot_number, path: rest.to_string() });
            }
        }
    }
    results
}
```

**New parser struct to add** (modeled on RESEARCH.md Pattern 1, same idiom as above):
```rust
struct ParsedProjectWork {
    tempo_raw: Option<u32>,
    flex_slots: Vec<Option<String>>,   // 128 entries, None = empty slot
    static_slots: Vec<Option<String>>, // 128 entries, None = empty slot
}

fn parse_project_work(raw: &[u8]) -> ParsedProjectWork {
    let text = String::from_utf8_lossy(raw);
    let mut tempo_raw: Option<u32> = None;
    let mut flex_slots: Vec<Option<String>> = vec![None; 128];
    let mut static_slots: Vec<Option<String>> = vec![None; 128];
    let mut in_settings = false;
    let mut in_slots = false;

    for line in text.lines() {
        let trimmed = line.trim();
        match trimmed {
            "[SETTINGS]" => { in_settings = true; in_slots = false; }
            "[SLOTS]" => { in_slots = true; in_settings = false; }
            s if s.starts_with('[') => { in_settings = false; in_slots = false; }
            s if in_settings => {
                if let Some(rest) = s.strip_prefix("TEMPO:") {
                    tempo_raw = rest.trim().parse().ok();
                }
            }
            s if in_slots => {
                if let Some(rest) = s.strip_prefix("FLEX") {
                    if let Some(colon) = rest.find(':') {
                        let idx: usize = rest[..colon].parse().unwrap_or(999);
                        if idx < 128 {  // bounds check — security requirement
                            let path = rest[colon + 1..].trim();
                            flex_slots[idx] = if path.is_empty() { None } else { Some(path.to_string()) };
                        }
                    }
                } else if let Some(rest) = s.strip_prefix("STAT") {
                    if let Some(colon) = rest.find(':') {
                        let idx: usize = rest[..colon].parse().unwrap_or(999);
                        if idx < 128 {  // bounds check — security requirement
                            let path = rest[colon + 1..].trim();
                            static_slots[idx] = if path.is_empty() { None } else { Some(path.to_string()) };
                        }
                    }
                }
            }
            _ => {}
        }
    }

    ParsedProjectWork { tempo_raw, flex_slots, static_slots }
}
```

**Existing `get_project_samples` DB lock-drop + file-read pattern** (`commands/samples.rs` lines 125–150) — structure to keep unchanged:
```rust
pub async fn get_project_samples(
    state: tauri::State<'_, crate::AppState>,
    project_id: String,
) -> Result<SampleSlotResponse, AppError> {
    // 1. DB lookup — lock acquired and immediately dropped
    let card_path = {
        let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        db::projects::get_card_path(&db.conn, &project_id)
            .map_err(|e| AppError::Database(e.to_string()))?
    };
    // lock dropped here — file I/O follows

    let project_dir = std::path::PathBuf::from(&card_path);
    let work_file = project_dir.join("project.work");
    let strd_file = project_dir.join("project.strd");

    let file_to_read = if work_file.exists() {
        work_file
    } else if strd_file.exists() {
        strd_file
    } else {
        return Ok(SampleSlotResponse {
            flex: make_empty_slots(128),
            static_slots: make_empty_slots(128),
        });
    };

    let content = std::fs::read_to_string(&file_to_read).map_err(AppError::from)?;
    // REPLACE: let parsed = parse_sample_slots(&content);
    // WITH:    let raw = content.as_bytes(); let parsed = parse_project_work(raw);
    // ...
}
```

**Call site replacement** — replace the slot-population loop (lines 151–207) to index directly into `parsed.flex_slots[i]` / `parsed.static_slots[i]` using the 0-indexed slot position from the new parser. The `SampleSlot` construction pattern below stays:
```rust
// Pattern for building each SampleSlot from a parsed Option<String>
let slot = SampleSlot {
    slot_index: i as u8,
    occupied: path_opt.is_some(),
    filename: path_opt.as_deref().map(filename_from_path),
    full_path: path_opt.clone(),
    sample_rate: None,
    status: if path_opt.is_some() { "ok" } else { "unknown" }.to_string(),
};
```

**Test pattern to add** (inline `#[cfg(test)]` matching existing style in `samples.rs` lines 743–942):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_project_work_occupied_slot() {
        let raw = b"[SLOTS]\nFLEX1:../AUDIO/kick.wav\nSTAT0:../AUDIO/pad.wav\n";
        let parsed = parse_project_work(raw);
        assert_eq!(parsed.flex_slots[1], Some("../AUDIO/kick.wav".to_string()));
        assert_eq!(parsed.static_slots[0], Some("../AUDIO/pad.wav".to_string()));
    }

    #[test]
    fn test_parse_project_work_empty_slot() {
        let raw = b"[SLOTS]\nFLEX0:\n";
        let parsed = parse_project_work(raw);
        assert_eq!(parsed.flex_slots[0], None);
    }

    #[test]
    fn test_parse_project_work_tempo() {
        let raw = b"[SETTINGS]\nTEMPO:1200\n[SLOTS]\n";
        let parsed = parse_project_work(raw);
        assert_eq!(parsed.tempo_raw, Some(1200));
    }

    #[test]
    fn test_parse_project_work_bounds_check() {
        // Malformed slot index (> 127) must not panic or write out of bounds
        let raw = b"[SLOTS]\nFLEX999:../AUDIO/bad.wav\n";
        let parsed = parse_project_work(raw);
        // All 128 flex slots should be None — the bad index was ignored
        assert!(parsed.flex_slots.iter().all(|s| s.is_none()));
    }
}
```

---

### `crates/takoyaki-app/src/commands/projects.rs` (replace stubs in `get_project_detail` and `get_project_banks`)

**Change scope:**
1. `get_project_banks` — replace `is_bank_populated_stub()` with `is_bank_populated()` that reads bank files from disk.
2. `get_project_detail` — read `project.work` and call `parse_project_work()` to get real `tempo_raw`, divide by `TEMPO_SCALE_FACTOR`. Note: `bank_name`, `part_name`, `machine_type` remain stub values (bank body is opaque).

**Analog — existing lock-drop + file-read pattern** (`commands/projects.rs` lines 100–120):
```rust
pub async fn get_project_detail(
    state: tauri::State<'_, AppState>,
    project_id: String,
) -> Result<ProjectDetail, AppError> {
    let (card_path, project_name, tempo_bpm, bank_count, last_modified) = {
        let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        // ... DB query ...
    };
    // lock dropped — file I/O follows
```

**New `is_bank_populated` to replace `is_bank_populated_stub`** (lines 226–230):
```rust
// OLD stub (lines 226–230) — DELETE:
fn is_bank_populated_stub(bank_index: u8, bank_count: u8) -> bool {
    bank_index < bank_count
}

// NEW implementation:
fn is_bank_populated(project_dir: &std::path::Path, bank_index: u8) -> bool {
    // bank files are 1-indexed on disk: bank01.work .. bank16.work
    let filename = format!("bank{:02}.work", bank_index + 1);
    let bank_path = project_dir.join(&filename);
    match std::fs::read(&bank_path) {
        Ok(data) => ot_parser::BankFile::from_bytes(&data).is_ok(),
        Err(_) => false,
    }
}
```

**Import to add** (`commands/projects.rs` top of file, following existing import style):
```rust
// Add alongside existing `use crate::db;` and `use crate::error::AppError;`
use ot_parser;  // for BankFile::from_bytes in is_bank_populated
```

**`get_project_banks` call-site change** (lines 199–217) — replace `is_bank_populated_stub(bank_index, bank_count)` with `is_bank_populated(&project_path, bank_index)`. Requires `card_path` to be kept in scope (currently dropped inside the `{}` block at line 181). Restructure: keep `card_path` outside the lock scope:
```rust
// Pattern from health.rs lines 33–51 — grab path from DB, drop lock, then do file I/O:
let card_path = {
    let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db::projects::get_card_path(&db.conn, &project_id)
        .map_err(|e| AppError::Database(e.to_string()))?
};
// lock dropped
let project_path = std::path::PathBuf::from(&card_path);
// ... then for each bank:
let populated = is_bank_populated(&project_path, bank_index);
```

**`get_project_detail` tempo change** — replace `tempo_bpm.unwrap_or(0.0)` (line 123) with real parse. Pattern:
```rust
// After dropping DB lock, read project.work:
let raw = std::fs::read(project_path.join("project.work"))
    .or_else(|_| std::fs::read(project_path.join("project.strd")))
    .unwrap_or_default();
let parsed_work = parse_project_work(&raw);  // call the new function from samples.rs or move to shared module
let display_tempo = parsed_work.tempo_raw
    .map(|r| r as f32 / TEMPO_SCALE_FACTOR)
    .unwrap_or(0.0);
```

Note: `parse_project_work` will need to be either moved to a shared module (e.g., `management/project_work.rs` as a new free function, separate from the old write-path functions) or made `pub(crate)` in `commands/samples.rs` and imported from there. The shared-module approach is cleaner; the inline approach avoids adding a new public surface.

**TEMPO_SCALE_FACTOR note** (line 16 in `projects.rs`): The fixture has `TEMPO:12000`, which at 10.0 gives 1200 BPM (out of range). The planner must decide: either update the fixture to `TEMPO:1200` (120 BPM at /10.0) or change the constant to 100.0. The constant is already isolated for this reason (the comment on line 14 names it "Assumption guard A2").

**Test to add in `crates/takoyaki-app/tests/project_detail.rs`** — un-ignore existing `#[ignore]` tests and fill in bodies. Pattern from `health_check.rs` lines 6–15:
```rust
fn fixture_path(relative: &str) -> std::path::PathBuf {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    std::path::Path::new(&manifest)
        .parent().unwrap()  // crates/
        .parent().unwrap()  // project root
        .join("tests/fixtures/mock_ot_volume")
        .join(relative)
}
```

---

### `crates/takoyaki-app/src/commands/health.rs` (replace empty `slot_inputs`)

**Change scope:** Replace lines 57–63 (the stub `Vec::new()`) with a real `Vec<SlotCheckInput>` built from `parse_project_work()` output.

**Existing lock-drop pattern** (`health.rs` lines 31–51) — this is the canonical reference for all Phase 7 I/O:
```rust
// 1. Grab project path and volume path from state, then DROP the locks.
//    All file I/O happens outside the lock (RESEARCH.md SQLite Lock Pattern).
let (project_path, volume_path) = {
    let card_path = {
        let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        db::projects::get_card_path(&db.conn, &project_id)
            .map_err(|e| AppError::Database(e.to_string()))?
    };
    // DB lock dropped here.

    let vol = {
        let device = state.device.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        device.mount_point.clone()
            .ok_or_else(|| AppError::Io("No OT volume mounted".to_string()))?
    };
    // Device lock dropped here.

    (card_path, vol)
};
```

**Stub to replace** (`health.rs` lines 56–63):
```rust
// CURRENT (STUB):
tauri::async_runtime::spawn(async move {
    // FIXME comment block...
    let slot_inputs: Vec<crate::health::SlotCheckInput> = Vec::new();
    // ...
```

**Replacement pattern** (inside the `spawn` block, after lock-drop):
```rust
tauri::async_runtime::spawn(async move {
    // Read project.work (or project.strd as fallback)
    let raw = std::fs::read(format!("{}/project.work", project_path))
        .or_else(|_| std::fs::read(format!("{}/project.strd", project_path)))
        .unwrap_or_default();

    let parsed = parse_project_work(&raw);  // from commands/samples.rs or shared module

    let mut slot_inputs: Vec<crate::health::SlotCheckInput> = Vec::new();

    for (idx, path_opt) in parsed.flex_slots.iter().enumerate() {
        slot_inputs.push(crate::health::SlotCheckInput {
            slot_type: "flex".to_string(),
            slot_index: idx as u8,
            occupied: path_opt.is_some(),
            raw_path: path_opt.clone(),
            track_references: vec![], // bank body opaque — DETC-03 limitation
        });
    }
    for (idx, path_opt) in parsed.static_slots.iter().enumerate() {
        slot_inputs.push(crate::health::SlotCheckInput {
            slot_type: "static".to_string(),
            slot_index: idx as u8,
            occupied: path_opt.is_some(),
            raw_path: path_opt.clone(),
            track_references: vec![],
        });
    }

    let issues = crate::health::perform_health_check(
        &project_path,
        &volume_path,
        &slot_inputs,
    ).await;
    // ...
```

**`SlotCheckInput` definition for reference** (`health/mod.rs` lines 88–98):
```rust
pub struct SlotCheckInput {
    pub slot_type: String,     // "flex" or "static"
    pub slot_index: u8,
    pub occupied: bool,
    pub raw_path: Option<String>,  // normalized OT path (already string, not bytes)
    pub track_references: Vec<TrackRef>,
}
```

**DETC-03 decision note:** With `track_references: vec![]` for all slots, `perform_health_check` will emit `HealthIssue::Info` for every occupied slot (lines 378–390 of `health/mod.rs`). This is correct-but-noisy behavior. The planner must decide whether to suppress the DETC-03 branch in Phase 7 or accept the flood of Info items.

---

### `tests/fixtures/mock_ot_volume/SETS/LIVESET/PROJECT_01/project.work` (fix placeholder)

**Change scope:** Replace `PLACEHOLDER_PROJECT_WORK` with real synthetic content in the `FLEX0:path` / `STAT0:path` format. The integration tests in `project_detail.rs` and `health_check.rs` read from this file.

**Analog:** `tests/fixtures/project.work` (the full 286-line synthetic fixture). Copy that format but make at least one slot occupied so DETC-01/02 tests have something to check:

```text
[META]
VERSION:1.40B
OS:1.40B
NAME:LIVESET_TEST

[SETTINGS]
TEMPO:1200
QUANTIZE:3
MASTERVOL:100

[SLOTS]
FLEX0:../AUDIO/kick_44100.wav
FLEX1:
...
FLEX127:
STAT0:../AUDIO/pad_48000.wav
STAT1:
...
STAT127:
```

Key points:
- `TEMPO:1200` — at `TEMPO_SCALE_FACTOR=10.0` gives 120.0 BPM (valid range). The main fixture at `tests/fixtures/project.work` uses `TEMPO:12000` which is suspicious; use 1200 here and update the planner to reconcile.
- `FLEX0:../AUDIO/kick_44100.wav` — maps to the existing `mock_ot_volume/AUDIO/kick_44100.wav` fixture for DETC-01 (file exists) and DETC-02 (correct format).
- `STAT0:../AUDIO/pad_48000.wav` — maps to existing `mock_ot_volume/AUDIO/pad_48000.wav` for DETC-02 wrong-rate test.
- All other slots empty (`: ` with no path).

---

### `crates/takoyaki-app/tests/project_detail.rs` (un-ignore and implement tests)

**Change scope:** Remove `#[ignore]` and implement the three test bodies.

**Analog — fixture helper pattern** (`health_check.rs` lines 6–15):
```rust
fn fixture_path(relative: &str) -> std::path::PathBuf {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    std::path::Path::new(&manifest)
        .parent().unwrap()  // crates/
        .parent().unwrap()  // project root
        .join("tests/fixtures/mock_ot_volume")
        .join(relative)
}
```

**Analog — unit call pattern** (`health_check.rs` lines 28–41 — direct function call, no Tauri state):
```rust
#[test]
fn test_health_wrong_sample_rate() {
    let path = fixture_path("AUDIO/pad_48000.wav");
    assert!(path.exists(), "Fixture file must exist: {}", path.display());
    let spec = takoyaki_app::health::read_audio_spec(&path).expect("Should read WAV spec");
    let issues = takoyaki_app::health::check_format_compatibility(&spec);
    // assert on issues
}
```

Note: The Tauri command functions (`get_project_samples`, `get_project_detail`, `get_project_banks`) cannot be called directly in unit tests because they require `tauri::State`. The test approach must test the underlying parsing functions directly (same pattern as `health_check.rs` testing `read_audio_spec` / `check_format_compatibility` directly). The planner should expose `parse_project_work` as `pub(crate)` or `pub` and test it directly.

---

## Shared Patterns

### Lock-Drop Before File I/O
**Source:** `crates/takoyaki-app/src/commands/health.rs` lines 31–51
**Apply to:** All three command modifications (`get_project_samples`, `get_project_detail`, `get_project_banks`, `run_health_check`)
```rust
// Pattern: acquire lock in a nested block, extract data, let block end (drop lock), THEN do I/O
let card_path = {
    let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db::projects::get_card_path(&db.conn, &project_id)
        .map_err(|e| AppError::Database(e.to_string()))?
};
// lock is dropped here — file I/O follows
```

### Infallible Parsing (Never Panic)
**Source:** `management/project_work.rs` lines 53–75 (`extract_slot_paths`) and `commands/samples.rs` lines 230–267 (`parse_sample_slots`)
**Apply to:** `parse_project_work()` — must use `unwrap_or(999)` / `unwrap_or_default()` for all parse operations, never `unwrap()` or `expect()`. Bad lines are silently skipped.

### AppError Mapping
**Source:** `commands/samples.rs` lines 130–133, `commands/health.rs` lines 35–38
**Apply to:** All new file I/O calls:
```rust
.map_err(|e| AppError::Lock(e.to_string()))?    // for Mutex::lock
.map_err(|e| AppError::Database(e.to_string()))? // for DB calls
.map_err(AppError::from)?                         // for std::io::Error (Io variant)
.map_err(|e| AppError::Io(e.to_string()))?        // for std::io::Error with context
```

### tracing::info! Logging
**Source:** `commands/samples.rs` lines 200–205, `commands/projects.rs` lines 343–344
**Apply to:** `get_project_samples` after parsing real slots, `get_project_banks` after bank file checks:
```rust
info!(
    "get_project_samples: found {} flex, {} static occupied slots",
    flex.iter().filter(|s| s.occupied).count(),
    static_slots.iter().filter(|s| s.occupied).count(),
);
```

### project.work / project.strd Fallback Read
**Source:** `commands/samples.rs` lines 136–148
**Apply to:** `get_project_detail` and `run_health_check` when reading project.work for tempo / slot inputs:
```rust
let file_to_read = if work_file.exists() { work_file } else if strd_file.exists() { strd_file } else {
    // return early with defaults
};
// OR for infallible read inside spawn:
let raw = std::fs::read(&work_file)
    .or_else(|_| std::fs::read(&strd_file))
    .unwrap_or_default();
```

### Test Fixture Helper
**Source:** `crates/takoyaki-app/tests/health_check.rs` lines 6–15, `crates/takoyaki-app/src/health/mod.rs` lines 407–416
**Apply to:** All integration tests in `tests/project_detail.rs`:
```rust
fn fixture_path(relative: &str) -> std::path::PathBuf {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    std::path::Path::new(&manifest)
        .parent().unwrap()  // crates/
        .parent().unwrap()  // project root
        .join("tests/fixtures/mock_ot_volume")
        .join(relative)
}
```

---

## No Analog Found

No files fall into this category. All modified files have close analogs in the codebase, and the core `parse_project_work` function follows the same idiom as the existing (wrong-format) parsers.

---

## Key Findings for Planner

1. **`parse_project_work` placement decision required:** The function is needed by three files (`commands/samples.rs`, `commands/projects.rs`, `commands/health.rs`). Options: (a) add as `pub(crate) fn` in `commands/samples.rs` and import from the other two, (b) add as a new free function in `management/project_work.rs` alongside the existing (wrong-format) functions, noting clearly it is a separate read-only parser. Option (b) is cleaner but puts a new function next to the wrong-format functions — risk of confusion. Option (a) avoids the confusion but requires cross-module imports.

2. **Do NOT modify `extract_slot_paths` or `rewrite_slot_path` in `management/project_work.rs`:** These feed `assign_sample` (Phase 5 write path). Changing them is Phase 8 scope. Phase 7 adds a parallel read-only parser only.

3. **TEMPO_SCALE_FACTOR:** The constant is `10.0` at `commands/projects.rs` line 16. The main fixture `tests/fixtures/project.work` has `TEMPO:12000` which gives 1200.0 BPM — out of valid range. The mock_ot_volume fixture replacement should use `TEMPO:1200` (120.0 BPM at /10.0). The planner should note this discrepancy and decide whether to also fix the main fixture.

4. **DETC-03 flooding decision required:** Once `run_health_check` feeds real slots, every occupied slot with `track_references: vec![]` emits `HealthIssue::Info`. The planner must decide: suppress the DETC-03 branch in Phase 7 (add a `if !slot.track_references.is_empty()` guard around lines 378–390 of `health/mod.rs` with a comment) or accept the flood and document it.

5. **`BankFile::from_bytes` is the bank-populated oracle:** It checks the `FORM` magic (first 4 bytes) and minimum size. An empty default bank file will still parse as `Ok` if it has the correct header — so file presence + parse success is the effective "bank was ever touched" heuristic (not "bank has content"). This is a known limitation (RESEARCH.md Assumption A3).

---

## Metadata

**Analog search scope:** `crates/takoyaki-app/src/commands/`, `crates/takoyaki-app/src/management/`, `crates/takoyaki-app/src/health/`, `crates/ot-parser/src/`, `tests/fixtures/`
**Files scanned:** 12 source files + 2 fixture files
**Pattern extraction date:** 2026-05-06
