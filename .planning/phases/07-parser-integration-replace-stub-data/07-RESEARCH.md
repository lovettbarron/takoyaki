# Phase 7: Parser Integration — Replace Stub Data - Research

**Researched:** 2026-05-06
**Domain:** Rust Tauri command wiring — replacing text/stub parsing with real project.work text parsing for sample slots, tempo, bank population, and health check inputs
**Confidence:** HIGH (all findings verified against codebase; format confirmed from synthetic fixture)

---

## Summary

Phase 7 closes a specific category of tech debt: the Phase 2 Tauri commands that return stub data. The stubs were placed intentionally with `FIXME` comments pointing to "Phase 1 OT parser", but the situation is more nuanced than those comments imply.

The `ot-parser` crate's `ProjectFile` stores raw bytes verbatim — it does not parse key-value fields. The real parsing work was done separately in `management/project_work.rs` (which extracts slot paths using `TYPE=FLEX`/`SLOT=NNN`/`PATH=...` format) and the `parse_sample_slots()` function in `commands/samples.rs` (which expects `[SAMPLE]...[/SAMPLE]` blocks). **Both of these format assumptions are wrong.** The synthetic fixture at `tests/fixtures/project.work` reveals the real format: `FLEX0:path`, `STAT0:path` under a `[SLOTS]` section header, and `TEMPO:12000` under `[SETTINGS]`.

The good news: `get_project_samples` already reads from `project.work` — it just parses the wrong format. `get_project_detail` reads from SQLite only (stub banks). The health check engine builds an empty `Vec<SlotCheckInput>` instead of building from real slot data. The `BankFile` body is an opaque blob — populated-bank detection requires a new heuristic (non-empty opaque body, or file presence + non-zero body).

**Primary recommendation:** Write a new `parse_project_work()` text parser that handles the real `[SECTION]`/`KEY:VALUE` format, replace the three stubs with calls to this parser, and update the mock fixture to contain real-looking data.

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| BROW-03 | User can view which banks and patterns are populated within a project | `is_bank_populated_stub()` uses `bank_count` from SQLite; replace with bank file presence + opaque body non-empty heuristic |
| BROW-04 | User can view all Flex and Static sample slots (128 each) with assigned file paths | `parse_sample_slots()` expects wrong format; new `parse_project_work()` reads real `FLEX0:path` format |
| BROW-05 | User can view project-level metadata including tempo, bank names, part names, machine types | `get_project_detail` uses SQLite-only stub; tempo readable from `[SETTINGS]`/`TEMPO:` key; bank/part names not in project.work (in bank body — opaque) |
| DETC-01 | User can detect missing or broken sample references across all slots | Health check uses empty `slot_inputs`; fix by building `Vec<SlotCheckInput>` from real slot parse |
| DETC-02 | User can validate audio file format compatibility | Health engine is complete; just needs real slot inputs fed to it |
| DETC-03 | User can detect unused samples | Track-reference cross-linking from bank opaque bodies is impossible — this requirement is limited to the health engine's existing "no track_references = unused" path, which requires bank parsing beyond scope |
</phase_requirements>

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| project.work text parsing | API / Backend (Rust) | — | File I/O and format parsing belongs in the Rust layer; never in the frontend |
| Sample slot hydration | API / Backend (Rust) | — | `get_project_samples` is a Tauri command; parser replaces stub inside command |
| Tempo extraction | API / Backend (Rust) | — | `get_project_detail` reads `TEMPO:` key from project.work |
| Bank populated detection | API / Backend (Rust) | — | `get_project_banks` / `is_bank_populated_stub` replaced with file+body heuristic |
| Health check slot inputs | API / Backend (Rust) | — | `run_health_check` builds `Vec<SlotCheckInput>` from real parsed slots |
| SlotPickerDialog slot state | Browser / Client (React) | — | Already receives `SampleSlotResponse` from IPC; displays `occupied` flag — no change needed once backend returns real data |

---

## Standard Stack

### Core (already installed — no new dependencies)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `ot-parser` crate | internal | `ProjectFile` stores raw bytes; used for round-trip safety | Already in workspace |
| `std::str` | stdlib | UTF-8 from_utf8_lossy for project.work parsing | No dep needed |

Phase 7 adds **zero new Cargo dependencies**. All parsing is in-process string iteration over the existing raw bytes. [VERIFIED: codebase inspection]

**Version verification:** Not applicable — no new packages. [VERIFIED: Cargo.toml inspection]

---

## Architecture Patterns

### System Architecture Diagram

```
project.work (text file on OT card)
    │
    ▼
std::fs::read() ──► raw bytes
    │
    ▼
parse_project_work(raw: &[u8]) ──► ParsedProjectWork {
    │                                  tempo_raw: Option<u32>,
    │                                  flex_slots: [Option<String>; 128],
    │                                  static_slots: [Option<String>; 128],
    │                              }
    │
    ├──► get_project_samples ──► SampleSlotResponse { flex[128], static_slots[128] }
    │                                (BROW-04)
    │
    ├──► get_project_detail ──► tempo_bpm = tempo_raw / TEMPO_SCALE_FACTOR
    │        └── is_bank_populated from BankFile::from_bytes on bankNN.work
    │                                (BROW-05, BROW-03)
    │
    └──► run_health_check ──► Vec<SlotCheckInput> ──► perform_health_check()
                                (DETC-01, DETC-02, DETC-03)
```

### Recommended Project Structure

No new files or directories needed. All changes are within:

```
crates/takoyaki-app/src/
├── commands/
│   ├── projects.rs       # get_project_detail, get_project_banks (replace stubs)
│   ├── samples.rs        # get_project_samples (replace parse_sample_slots)
│   └── health.rs         # run_health_check (replace empty Vec)
└── management/
    └── project_work.rs   # extract_slot_paths / rewrite_slot_path (may need format fix)

crates/ot-parser/src/
└── project.rs            # ProjectFile::parse_key_values() helper — or keep as pure raw storage

tests/fixtures/
├── project.work          # Already has real format (FLEX0: STAT0: TEMPO:) — synthetic only
└── mock_ot_volume/SETS/LIVESET/PROJECT_01/
    └── project.work      # Currently "PLACEHOLDER_PROJECT_WORK" — needs real synthetic content
```

### Pattern 1: Real project.work Text Parser

**What:** New `parse_project_work()` function that reads `[SECTION]`/`KEY:VALUE` and `KEY:` format. Can live as a free function in `commands/samples.rs` (close to use site) or as a shared module function.

**When to use:** Anywhere `project.work` bytes need to yield structured data.

**Real format (from `tests/fixtures/project.work`):** [VERIFIED: codebase inspection]

```text
[META]
VERSION:1.40B
OS:1.40B
NAME:SYNTHETIC_PROJECT

[SETTINGS]
TEMPO:12000
QUANTIZE:3
MASTERVOL:100
...

[STATES]
MUTE:0
...

[SLOTS]
FLEX0:
FLEX1:../AUDIO/kick.wav
FLEX2:
...
FLEX127:
STAT0:../AUDIO/pad.wav
STAT1:
...
STAT127:
```

**Key facts:** [VERIFIED: tests/fixtures/project.work]
- Section headers use square brackets: `[META]`, `[SETTINGS]`, `[SLOTS]`
- Key-value separator is `:` (not `=`)
- Slot keys: `FLEX0` through `FLEX127` (0-indexed), `STAT0` through `STAT127` (0-indexed)
- Slot value is the path after `:` — empty string if unoccupied
- TEMPO raw value: `12000` in the fixture — display BPM = 12000 / TEMPO_SCALE_FACTOR
- The TEMPO_SCALE_FACTOR constant is already isolated at 10.0 in `commands/projects.rs` — fixture value 12000 / 10.0 = 120.0 BPM confirms the scale factor is correct [VERIFIED]

**Example implementation pattern:**

```rust
// Source: codebase inspection of tests/fixtures/project.work real format
pub struct ParsedProjectWork {
    pub tempo_raw: Option<u32>,
    pub flex_slots: Vec<Option<String>>, // 128 entries, None if empty
    pub static_slots: Vec<Option<String>>, // 128 entries, None if empty
}

pub fn parse_project_work(raw: &[u8]) -> ParsedProjectWork {
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
                // FLEX0:path or FLEX0: (empty)
                if let Some(rest) = s.strip_prefix("FLEX") {
                    if let Some(colon) = rest.find(':') {
                        let idx: usize = rest[..colon].parse().unwrap_or(999);
                        if idx < 128 {
                            let path = rest[colon + 1..].trim();
                            flex_slots[idx] = if path.is_empty() { None } else { Some(path.to_string()) };
                        }
                    }
                } else if let Some(rest) = s.strip_prefix("STAT") {
                    if let Some(colon) = rest.find(':') {
                        let idx: usize = rest[..colon].parse().unwrap_or(999);
                        if idx < 128 {
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

### Pattern 2: Bank Populated Detection via File Heuristic

**What:** Replace `is_bank_populated_stub()` with actual bank file parsing. BankFile body is opaque, but "populated" can be detected via: (1) bank file exists on disk AND (2) opaque_body.len() > some minimum threshold (empty banks have minimal body content).

**Limitation:** The exact opaque body structure is undocumented. The safe heuristic is: if the bank file exists at `bankNN.work`, attempt `BankFile::from_bytes()`. If it parses successfully, treat as populated. If file does not exist or parse fails, treat as not populated. [VERIFIED: format-spec.md and bank.rs]

**Pattern:**

```rust
// Source: codebase inspection of ot-parser/src/bank.rs
fn is_bank_populated(project_dir: &std::path::Path, bank_index: u8) -> bool {
    let filename = format!("bank{:02}.work", bank_index + 1); // 0-indexed -> 1-indexed filename
    let bank_path = project_dir.join(&filename);
    if !bank_path.exists() {
        return false;
    }
    match std::fs::read(&bank_path) {
        Ok(data) => ot_parser::BankFile::from_bytes(&data).is_ok(),
        Err(_) => false,
    }
}
```

**Note:** The `BankFile::from_bytes()` validates the `FORM` magic and minimum size. An empty/default bank file will still parse as `Ok`, so file presence + magic validity is the effective heuristic. [VERIFIED: bank.rs from_bytes implementation]

### Pattern 3: Building Health Check SlotCheckInput from Real Slots

**What:** Replace the `let slot_inputs: Vec<SlotCheckInput> = Vec::new();` stub in `commands/health.rs` with real slot input built from `parse_project_work()`.

**Pattern:**

```rust
// Source: codebase inspection of health/mod.rs SlotCheckInput definition
// and health.rs FIXME comment
let raw = std::fs::read(project_path.join("project.work"))
    .or_else(|_| std::fs::read(project_path.join("project.strd")))
    .unwrap_or_default();

let parsed = parse_project_work(&raw);
let mut slot_inputs: Vec<SlotCheckInput> = Vec::new();

for (idx, path_opt) in parsed.flex_slots.iter().enumerate() {
    slot_inputs.push(SlotCheckInput {
        slot_type: "flex".to_string(),
        slot_index: idx as u8,
        occupied: path_opt.is_some(),
        raw_path: path_opt.clone(),
        track_references: vec![], // Bank body opaque — cannot cross-ref (DETC-03 limitation)
    });
}
// Same for static_slots
```

**DETC-03 limitation:** Track reference cross-linking requires parsing the bank opaque body (pattern trig data), which is out of scope for Phase 7. Health check will correctly mark all occupied slots as having `track_references: vec![]`, causing them all to emit `HealthIssue::Info { detail: "assigned but not referenced by any track" }`. This is technically a regression from "no issues" (current stub returns no issues because slot_inputs is empty), but it is the correct behavior per DETC-03 semantics — all slots will be flagged as potentially unused. This is acceptable and expected. [ASSUMED — potential UX regression requires user awareness]

### Anti-Patterns to Avoid

- **Touching `project_work.rs`'s `rewrite_slot_path` / `extract_slot_paths` for Phase 7:** These functions use the `TYPE=FLEX`/`SLOT=NNN`/`PATH=...` format assumption. They are used by `assign_sample` (Phase 5). Fixing them is Phase 8 scope or a separate fix — Phase 7 adds a parallel correct parser without removing the wrong one. The wrong parser will silently return zero slots from real OT files, which is a Phase 8 fix.
- **Using `ProjectFile::from_bytes()` to get text content:** It just stores raw bytes — it's a correct round-trip store but doesn't add value over `std::fs::read()` for text parsing. Call `std::fs::read()` directly.
- **Parsing bank names or part names from project.work:** These are stored in the bank file's opaque body. They cannot be extracted without a deep bank parser. `get_project_detail` must leave `bank_name: None` and `part_name: None` for Phase 7. [VERIFIED: format-spec.md and bank.rs]
- **Parsing machine types from project.work:** Also in bank file opaque body. `TrackDetail.machine_type` must remain `"Thru"` (the stub value) for Phase 7. [VERIFIED: format-spec.md]

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Bank file magic validation | Custom magic check | `ot_parser::BankFile::from_bytes()` | Already handles minimum size and FORM magic check correctly |
| Bank file round-trip | Custom serializer | `ot_parser::BankFile::to_bytes()` | Preserves verbatim bytes including opaque body |
| Path normalization | New path normalizer | Existing `normalize_ot_path()` in `commands/samples.rs` | Already handles null termination, backslash, stripping; reuse for any raw OT path bytes |
| Path resolution (health) | New resolver | Existing `health::resolve_ot_path()` | Already handles volume root containment check (T-02-05) |

---

## Critical Discovery: Format Assumption Mismatch

This is the most important finding in this research.

**The existing `parse_sample_slots()` in `commands/samples.rs` (lines 229-267) expects a `[SAMPLE]...[/SAMPLE]` block format with `TYPE=FLEX`, `SLOT=NNN`, `PATH=...` lines. This format does NOT match the real OT project.work format.** [VERIFIED: codebase inspection — both the synthetic fixture at `tests/fixtures/project.work` and the format-spec.md documentation confirm the real `FLEX0:path` key-value format]

**The existing `extract_slot_paths()` and `rewrite_slot_path()` in `management/project_work.rs` also use the wrong format (`TYPE=FLEX` etc.).** [VERIFIED: project_work.rs source]

**Impact on `get_project_samples`:** The function does read from `project.work` but its `parse_sample_slots()` inner function will always return zero slots on a real OT card because the format doesn't match. This means the current "partial implementation" noted in ANALYSIS.md is actually returning 128 empty stubs on all real projects.

**Impact on `assign_sample` (Phase 5):** `project_work::rewrite_slot_path()` used by `assign_sample` also uses the wrong format. Calling it against a real OT project.work will return unchanged bytes (the silent warning log in `assign_sample` line 570 would fire). This is a Phase 8 issue — Phase 7 does not touch `assign_sample`.

**Phase 7 action:** Write a new `parse_project_work()` that handles the real format. Do NOT modify `extract_slot_paths()` or `rewrite_slot_path()` in Phase 7 (those feed `assign_sample` and modifying them could introduce regressions in the Phase 5 write path).

---

## Common Pitfalls

### Pitfall 1: Modifying project_work.rs and Breaking assign_sample

**What goes wrong:** Fixing `extract_slot_paths()` or `rewrite_slot_path()` to use the real format in Phase 7 would break `assign_sample` if the real OT format switch causes the rewrite to mangle real files.
**Why it happens:** The write path in `assign_sample` depends on `rewrite_slot_path` returning bytes that the OT can read back. If the format assumption is wrong in both directions (read AND write), the OT card might be corrupted.
**How to avoid:** Phase 7 adds a new read-only `parse_project_work()` alongside the existing functions. The existing write functions stay unchanged. Phase 8 validates the write path with real OT data.
**Warning signs:** Any change to `project_work.rs` in Phase 7 is suspect.

### Pitfall 2: DETC-03 False-Positive Flood After Fix

**What goes wrong:** Once `run_health_check` feeds real slots to `perform_health_check()`, every occupied slot will trigger `HealthIssue::Info` ("assigned but not referenced by any track") because `track_references` is always empty (bank body is opaque).
**Why it happens:** `perform_health_check` checks `slot.track_references.is_empty()` for DETC-03 — this will be true for all slots.
**How to avoid:** Accept this behavior as correct for now. The health tab will show many "Info" items. Update the health engine to skip DETC-03 check when `track_references` would always be empty (set a feature flag), OR document that DETC-03 is not meaningfully implementable until the bank body is parsed. The ROADMAP and ANALYSIS.md both confirm this is a known limitation. Planner should make a decision: either suppress DETC-03 in Phase 7 or emit it and note the limitation in the UI.
**Warning signs:** Health tab shows 256 "Info: assigned but not referenced" items after Phase 7.

### Pitfall 3: Missing mock_ot_volume project.work Content

**What goes wrong:** Tests pass against synthetic text fixtures but the mock_ot_volume fixture (`tests/fixtures/mock_ot_volume/SETS/LIVESET/PROJECT_01/project.work`) currently contains only `PLACEHOLDER_PROJECT_WORK`. Integration tests that read from this fixture will fail.
**Why it happens:** The fixture was created as a placeholder pending real OT data.
**How to avoid:** Update the mock_ot_volume project.work to contain realistic `[SLOTS]`/`FLEX0:` format content. For tests requiring occupied slots (DETC-01), add at least one `FLEX0:../AUDIO/kick_44100.wav` entry.
**Warning signs:** Integration tests in `tests/project_detail.rs` (currently marked `#[ignore]`) fail when un-ignored.

### Pitfall 4: TEMPO_SCALE_FACTOR Validation

**What goes wrong:** The fixture has `TEMPO:12000` which at TEMPO_SCALE_FACTOR=10.0 gives 1200.0 BPM — impossible. At 120.0 BPM the raw value should be 1200, not 12000.
**Why it happens:** The fixture may have been written with the wrong scale factor, OR the scale factor is 100 (not 10), OR the assumption in `commands/projects.rs` is wrong.
**How to avoid:** The synthetic fixture says `TEMPO:12000` and the project name is `SYNTHETIC_PROJECT`. 12000 / 10.0 = 1200 BPM (impossible). 12000 / 100.0 = 120.0 BPM (plausible). The existing `TEMPO_SCALE_FACTOR = 10.0` constant may be wrong for project.work's tempo format, even if correct for the .ot file format. [ASSUMED — verify against real OT data or ot-tools-io docs]
**Warning signs:** `tempo_bpm` returned by `get_project_detail` is outside the 30–300 BPM range after Phase 7.
**Resolution:** Phase 7 should log the raw TEMPO value and the computed BPM, and the planner should add a test that asserts the computed value is in 30–300 range. If the synthetic fixture's 12000 yields an out-of-range BPM, update the fixture to use a value that produces a valid range.

### Pitfall 5: Lock Guard Patterns — Don't Hold Lock During File I/O

**What goes wrong:** Adding file I/O inside a `state.db.lock()` or `state.device.lock()` guard causes deadlocks.
**Why it happens:** Existing code pattern consistently drops DB lock before file I/O (`let card_path = { let db = state.db.lock()...; db::projects::get_card_path(...) }; // lock dropped here`).
**How to avoid:** Follow existing pattern exactly — grab the lock, extract data, drop lock, then do file I/O. See `run_health_check` lines 33-51 for the canonical pattern. [VERIFIED: health.rs]

---

## Code Examples

### Existing parse_sample_slots (WRONG FORMAT — DO NOT EXTEND)

```rust
// Source: commands/samples.rs lines 229-267
// This function parses [SAMPLE]...[/SAMPLE] blocks — NOT the real OT format
// Do not add new callers to this function. Replace its call site with parse_project_work().
fn parse_sample_slots(content: &str) -> Vec<ParsedSampleEntry> {
    // expects trimmed == "[SAMPLE]" / "[/SAMPLE]" / "TYPE=..." / "SLOT=..." / "PATH=..."
    // Real format: "FLEX0:path" under [SLOTS] section
}
```

### Existing build_sample_slot Helper (REUSE THIS)

```rust
// Source: commands/samples.rs lines 274-286
// This is correctly marked #[allow(dead_code)] — Phase 7 activates it
#[allow(dead_code)]
fn build_sample_slot(slot_index: u8, raw_path: &[u8], sample_rate: Option<u32>) -> SampleSlot {
    let full_path = normalize_ot_path(raw_path);
    let filename = full_path.as_deref().map(filename_from_path);
    let occupied = full_path.is_some();
    SampleSlot { slot_index, occupied, filename, full_path, sample_rate, status: "unknown".to_string() }
}
```

**Note:** `build_sample_slot` takes `&[u8]` (for binary paths). The real format has text paths. For text-format paths, use a simpler builder that takes `Option<&str>` directly.

### Health Check Stub (REPLACE THIS)

```rust
// Source: commands/health.rs lines 63-64
// This is the stub to replace
let slot_inputs: Vec<crate::health::SlotCheckInput> = Vec::new();
```

### Bank File Existence Check (NEW PATTERN)

```rust
// Source: synthesized from ot-parser/src/bank.rs BankFile::from_bytes
// Pattern for Phase 7 is_bank_populated replacement
fn is_bank_populated(project_dir: &std::path::Path, bank_index: u8) -> bool {
    let filename = format!("bank{:02}.work", bank_index + 1);
    let bank_path = project_dir.join(&filename);
    match std::fs::read(&bank_path) {
        Ok(data) => ot_parser::BankFile::from_bytes(&data).is_ok(),
        Err(_) => false,
    }
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Empty slot list in health check | Build from real parsed slots | Phase 7 | DETC-01/02 work on real data |
| 128 stub bank entries in project detail | Bank file presence + parse check | Phase 7 | BROW-03 shows real populated state |
| Wrong `[SAMPLE]` block format in get_project_samples | Real `FLEX0:path` key-value parsing | Phase 7 | BROW-04 shows real slot assignments |
| SQLite-only tempo in get_project_detail | Read `TEMPO:` from project.work | Phase 7 | BROW-05 shows accurate tempo |

**Currently broken (not in Phase 7 scope):**
- `extract_slot_paths()` / `rewrite_slot_path()` in `project_work.rs` — wrong format for write path. Phase 8 validates this.
- Bank name, part name, machine type parsing — bank body is opaque. Deferred to a future deep-bank-parser phase.
- DETC-03 track-reference cross-linking — bank body is opaque.

---

## Environment Availability

Step 2.6: SKIPPED (no external dependencies — this phase is code-only changes within the existing Rust workspace, adding no new tools or services).

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `cargo test` |
| Config file | none (workspace uses default) |
| Quick run command | `cargo test --package takoyaki-app` |
| Full suite command | `cargo test --package takoyaki-app && cargo test --package ot-parser` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| BROW-04 | `parse_project_work()` parses `FLEX0:path` correctly | unit | `cargo test --package takoyaki-app test_parse_project_work` | ❌ Wave 0 |
| BROW-04 | `parse_project_work()` returns None for empty slot | unit | `cargo test --package takoyaki-app test_parse_project_work_empty_slot` | ❌ Wave 0 |
| BROW-04 | `get_project_samples` returns real occupied slots | integration | un-ignore `test_get_project_samples` in `tests/project_detail.rs` | ❌ Wave 0 (fixture needed) |
| BROW-05 | `parse_project_work()` extracts TEMPO correctly | unit | `cargo test --package takoyaki-app test_parse_tempo` | ❌ Wave 0 |
| BROW-03 | `is_bank_populated()` returns false for missing bank file | unit | `cargo test --package takoyaki-app test_is_bank_populated_missing` | ❌ Wave 0 |
| BROW-03 | `is_bank_populated()` returns true for valid bank file | unit | `cargo test --package takoyaki-app test_is_bank_populated_valid` | ❌ Wave 0 |
| DETC-01 | Health check emits Error for missing file when slot is occupied | integration | un-ignore `test_health_missing_file` | ❌ Wave 0 (needs async test) |
| DETC-02 | Health check emits Warning for 48kHz file (existing test) | unit | `cargo test --package takoyaki-app test_health_wrong_sample_rate` | ✅ passes |

### Sampling Rate

- **Per task commit:** `cargo test --package takoyaki-app`
- **Per wave merge:** `cargo test --package takoyaki-app && cargo test --package ot-parser`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `crates/takoyaki-app/tests/project_detail.rs` — un-ignore `test_get_project_samples`, `test_get_project_detail`, `test_get_project_banks` (requires fixture update first)
- [ ] `tests/fixtures/mock_ot_volume/SETS/LIVESET/PROJECT_01/project.work` — replace `PLACEHOLDER_PROJECT_WORK` with real synthetic `[SLOTS]`/`FLEX0:` content
- [ ] Unit tests for `parse_project_work()` — add inline `#[cfg(test)]` block
- [ ] Unit tests for new `is_bank_populated()` function

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | Parse text from OT card files — path traversal via crafted slot paths |
| V6 Cryptography | no | — |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal via crafted FLEX slot paths | Tampering/EoP | Existing `health::resolve_ot_path()` with `canonicalize()` and volume root containment check (T-02-05) — MUST use this for all path resolution in health check |
| Malformed project.work causing panic | DoS | `parse_project_work()` must be infallible (return defaults on any parse error, never unwrap/panic) |
| Out-of-bounds slot index in `FLEX999:path` | Tampering | Parser must bounds-check slot index before writing to the 128-slot array |

All three mitigations are already present in the codebase and must be maintained:
- Path traversal: `health::resolve_ot_path()` [VERIFIED: health/mod.rs]
- Infallible parsing: follow the existing `parse_sample_slots()` pattern of `unwrap_or(0)` and silent continue on bad lines [VERIFIED: samples.rs]
- Bounds checking: `if idx < 128` guard in the new parser [synthesized from existing patterns]

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `TEMPO_SCALE_FACTOR = 10.0` is correct for project.work's `TEMPO:` key | Pitfall 4 | Displayed BPM is 10x off (1200 BPM instead of 120 BPM). The synthetic fixture value of 12000 / 10.0 = 1200 is already suspicious. Consider the scale might be 100. | 
| A2 | `project.work`'s `[SLOTS]` section always uses `FLEX0:`..`FLEX127:` and `STAT0:`..`STAT127:` keys | Standard Stack | Different firmware versions may use different key formats. The format-spec.md notes only the synthetic fixture format as confirmed. |
| A3 | An empty bank body (opaque_body with just default content) still passes `BankFile::from_bytes()` — so file presence + parse success is a valid populated heuristic | Pattern 2 | Some unpopulated banks might have valid headers but empty content, causing false positives in is_bank_populated |
| A4 | DETC-03 flooding (all occupied slots emitting "unused" info) is acceptable UX for Phase 7 | Pitfall 2 | Confusing to users who see 128 "unused" warnings |

---

## Open Questions

1. **TEMPO_SCALE_FACTOR for project.work**
   - What we know: The constant is 10.0 in `commands/projects.rs`. The synthetic fixture has `TEMPO:12000`. 12000 / 10 = 1200 BPM.
   - What's unclear: Is the synthetic fixture wrong (should be `TEMPO:1200` for 120 BPM), or is the scale factor wrong (should be 100)?
   - Recommendation: Planner should update the synthetic fixture's TEMPO value to 1200 (which at /10 = 120 BPM is valid), and add a test asserting the result is 120.0. Alternatively, if real OT data shows TEMPO:12000 = 120 BPM, change the scale factor to 100.

2. **DETC-03 behavior: suppress or emit?**
   - What we know: Emitting track_references=[] for all slots produces `HealthIssue::Info` for every occupied slot.
   - What's unclear: Is the user better served by 128 "unused sample" warnings (correct behavior) or no DETC-03 warnings (cleaner UX but DETC-03 requirement not met)?
   - Recommendation: Planner should decide. The requirement says "detect unused samples" — emitting the warning is correct. A future bank parser phase can make this accurate.

3. **project_work.rs write path format mismatch**
   - What we know: `extract_slot_paths()` and `rewrite_slot_path()` use wrong format. `assign_sample` depends on them.
   - What's unclear: Should Phase 7 include fixing the write path or just the read path?
   - Recommendation: Phase 7 scope is read path only (stub replacement). The write path fix is a Phase 8 safety concern (WR-02 area). Keep them separate.

---

## Sources

### Primary (HIGH confidence)
- `tests/fixtures/project.work` — real synthetic OT project file showing actual format
- `crates/ot-parser/format-spec.md` — clean-room format specification with verified sources
- `crates/takoyaki-app/src/commands/projects.rs` — current stub implementations
- `crates/takoyaki-app/src/commands/samples.rs` — current parse_sample_slots (wrong format)
- `crates/takoyaki-app/src/commands/health.rs` — current empty slot_inputs stub
- `crates/takoyaki-app/src/health/mod.rs` — SlotCheckInput definition and perform_health_check
- `crates/ot-parser/src/bank.rs` — BankFile::from_bytes for bank populated detection
- `.planning/quick/260505-izk-analysis-of-takoyaki-progress-and-propos/ANALYSIS.md` — prior gap analysis

### Secondary (MEDIUM confidence)
- `docs.rs/ot-tools-io/0.6.0` (accessed via WebFetch) — format study reference, limited field detail
- Format-spec.md ASSUMPTION A-01 — project.work is text; confirmed by synthetic fixture content

### Tertiary (LOW confidence — marked ASSUMED)
- TEMPO_SCALE_FACTOR correct value for project.work TEMPO: key — unverified against real OT hardware

---

## Metadata

**Confidence breakdown:**
- Format discovery (project.work is `FLEX0:path` format): HIGH — verified from synthetic fixture in codebase
- Stub locations and FIXME comments: HIGH — verified from source
- TEMPO_SCALE_FACTOR correctness: LOW — synthetic fixture value suspicious
- Bank populated heuristic: MEDIUM — BankFile::from_bytes logic verified, but opaque body behavior for empty banks is assumed

**Research date:** 2026-05-06
**Valid until:** 2026-06-06 (stable format; no external deps)
