---
phase: 07-parser-integration-replace-stub-data
reviewed: 2026-05-06T12:00:00Z
depth: standard
files_reviewed: 7
files_reviewed_list:
  - crates/takoyaki-app/src/commands/samples.rs
  - crates/takoyaki-app/src/commands/projects.rs
  - crates/takoyaki-app/src/commands/health.rs
  - crates/takoyaki-app/src/health/mod.rs
  - crates/takoyaki-app/src/lib.rs
  - crates/takoyaki-app/tests/project_detail.rs
  - crates/takoyaki-app/tests/health_check.rs
findings:
  critical: 1
  warning: 2
  info: 3
  total: 6
status: issues_found
---

# Phase 7: Code Review Report

**Reviewed:** 2026-05-06T12:00:00Z
**Depth:** standard
**Files Reviewed:** 7
**Status:** issues_found

## Summary

Phase 7 replaces stub data with real parser integration across three commands: `get_project_detail` (tempo from project.work), `get_project_samples` (slot paths from [SLOTS] section), and `run_health_check` (real slot inputs from parsed project.work). The new `parse_project_work` text parser is well-structured, infallible, and bounds-checked. Most of the integration is clean.

However, there is a critical path resolution mismatch between the new parser output and the existing health check engine. Paths from `parse_project_work` include `../` relative prefixes (e.g., `../AUDIO/kick.wav`), but `resolve_ot_path` in the health engine only strips leading `/` characters, not `../` segments. This means health checks on real OT projects will incorrectly flag valid sample paths as "invalid or unsafe." There is also a dormant logic inversion in the DETC-03 unused sample detection that will become a bug once track references are populated.

## Critical Issues

### CR-01: Path resolution mismatch -- health check rejects valid `../`-prefixed OT paths

**File:** `crates/takoyaki-app/src/health/mod.rs:212-215`
**Issue:** `resolve_ot_path` normalizes OT paths by replacing backslashes and stripping leading `/`, but does NOT strip `../` prefixes. The new `parse_project_work` parser (samples.rs:696) returns paths verbatim from the project.work file, which use `../AUDIO/filename.wav` format (relative from the project directory to the card root). When `resolve_ot_path` joins `../AUDIO/kick.wav` with the volume root path, it produces `<volume_root>/../AUDIO/kick.wav`, which resolves to a path *outside* the volume root. The canonicalize-based traversal check then correctly rejects it -- but this means every valid occupied sample slot will be reported as "Invalid or unsafe path" in the health check.

The `run_health_check` command (health.rs:65-82) passes `path_opt.clone()` from parsed slots directly to `SlotCheckInput.raw_path`, and the comment on `SlotCheckInput.raw_path` (health/mod.rs:94) incorrectly states paths are "already passed through `normalize_ot_path` in samples.rs" -- they are not. The test `test_health_unused_sample_suppressed_when_no_track_refs` uses `AUDIO/kick_44100.wav` (no `../`) which sidesteps this bug, masking it.

**Fix:** Strip `../` prefixes in `resolve_ot_path` the same way `resolve_sample_path` in samples.rs does, or normalize the path before passing it to `SlotCheckInput`:

```rust
// In health/mod.rs, resolve_ot_path, after line 215:
let normalized = raw_path.replace('\\', "/");
let relative = normalized
    .trim_start_matches("../")
    .trim_start_matches('/');
```

Alternatively, normalize in the health command before building `SlotCheckInput`:

```rust
// In health.rs, when building slot_inputs:
raw_path: path_opt.as_deref()
    .map(|p| p.replace('\\', "/")
              .trim_start_matches("../")
              .trim_start_matches('/')
              .to_string()),
```

Also update the integration test to use `../AUDIO/kick_44100.wav` to catch this in the future.

## Warnings

### WR-01: DETC-03 logic inversion -- non-empty track_references unconditionally flagged as "unused"

**File:** `crates/takoyaki-app/src/health/mod.rs:383-399`
**Issue:** The DETC-03 unused sample detection block fires when `!slot.track_references.is_empty()` (i.e., when references exist), but then unconditionally pushes an "unused" info issue. The logic is inverted: slots WITH track references should NOT be flagged as unused. Currently dormant because `track_references` is always empty in Phase 7, but will produce false "unused" reports for every referenced slot when bank body parsing is implemented.

**Fix:**
```rust
// The condition should check that the slot has NO references:
if slot.track_references.is_empty() && slot.occupied {
    // This slot is assigned but not referenced by any track
    let filename = resolved_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| raw_path.clone());
    issues.push(HealthIssue::Info {
        slot_type: slot.slot_type.clone(),
        slot_index: slot.slot_index,
        filename: filename.clone(),
        detail: format!(
            "{filename} (slot #{}) -- assigned but not referenced by any track.",
            slot.slot_index
        ),
    });
}
```

Note: this also means the Phase 7 suppression guard (the `track_references.is_empty()` early-skip) needs to be rethought. A separate boolean flag like `track_refs_available: bool` on `SlotCheckInput` would make the intent clearer than overloading the empty-vec semantics.

### WR-02: Stale comment on `SlotCheckInput.raw_path` -- claims normalization that does not happen

**File:** `crates/takoyaki-app/src/health/mod.rs:94-95`
**Issue:** The doc comment says "Normalized OT path (already passed through `normalize_ot_path` in samples.rs)." In Phase 7, the health command (health.rs:70,79) passes `path_opt.clone()` directly from `parse_project_work`, which returns raw text values -- NOT normalized via `normalize_ot_path`. This misleading comment could cause future contributors to skip normalization, trusting it was already done.

**Fix:**
```rust
/// Raw OT path from the project.work parser. NOT yet normalized --
/// the health engine's `resolve_ot_path` handles normalization.
pub raw_path: Option<String>,
```

## Info

### IN-01: Misleading docstring on `generate_project_id` -- says SHA-256 but uses SipHash

**File:** `crates/takoyaki-app/src/commands/projects.rs:385`
**Issue:** The docstring says "using SHA-256 (first 16 bytes as hex)" but the implementation uses `std::collections::hash_map::DefaultHasher` which is SipHash-based. This is not a functional issue (the ID is still deterministic) but the documentation is misleading.

**Fix:**
```rust
/// Generate a deterministic project ID from its card path using SipHash (64-bit hex string).
```

### IN-02: Unused parameter `_project_path` in `perform_health_check`

**File:** `crates/takoyaki-app/src/health/mod.rs:279`
**Issue:** `_project_path: &str` is passed to `perform_health_check` but never used (underscore prefix confirms this). The caller in health.rs:90 still computes and passes it. This is likely leftover from when the function was expected to resolve `../`-relative paths from the project directory -- which is exactly what CR-01 identifies as missing. Once CR-01 is fixed, this parameter may become necessary.

**Fix:** Either remove the parameter if the path resolution fix goes into `resolve_ot_path`, or use it to resolve `../`-relative paths correctly within `perform_health_check`.

### IN-03: Duplicated `is_leap_year` and date formatting functions across modules

**File:** `crates/takoyaki-app/src/commands/projects.rs:432-434` and `crates/takoyaki-app/src/commands/health.rs:166-168`
**Issue:** `is_leap_year` is defined identically in both `projects.rs` and `health.rs`. The `format_unix_timestamp` (projects.rs:397-430) and `format_iso8601` (health.rs:125-164) functions share nearly identical calendar computation logic. This is code duplication that could drift over time.

**Fix:** Extract a shared `utils` or `time` module with a single `format_iso8601` function and `is_leap_year` helper. Both command modules can import from there.

---

_Reviewed: 2026-05-06T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
