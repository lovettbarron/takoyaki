---
phase: 08-phase-5-quality-and-safety-fixes
plan: "01"
subsystem: samples
tags: [safety, atomic-write, format-validation, conflict-detection, tdd]
dependency_graph:
  requires: []
  provides: [assign_sample-format-gate, assign_sample-atomic-copy, assign_sample-overwrite-param]
  affects: [crates/takoyaki-app/src/commands/samples.rs]
tech_stack:
  added: []
  patterns: [temp-then-rename atomic copy, health::read_audio_spec format gate, CONFLICT error prefix]
key_files:
  modified:
    - crates/takoyaki-app/src/commands/samples.rs
decisions:
  - "Format gate only blocks FormatIssue::UnsupportedFormat — WrongSampleRate and WrongBitDepth remain soft warnings (user already saw them in dry-run preview and chose to proceed)"
  - "Temp file staged as .filename.tmp in same audio_dir — same FAT32 volume guarantees atomic rename semantics on macOS"
  - "Best-effort cleanup (let _ = std::fs::remove_file) on rename failure prevents orphaned .tmp files"
  - "CONFLICT error message contains only filename (not full path) per T-08-03 accept disposition"
metrics:
  duration: "8 min"
  completed: "2026-05-06"
  tasks_completed: 2
  files_modified: 1
---

# Phase 08 Plan 01: Format Gate, Atomic Copy, and Conflict Detection Summary

**One-liner:** Format gate via health::read_audio_spec, temp-then-rename Wallflower copy, and overwrite/CONFLICT param in assign_sample — closing three safety gaps before any OT write occurs.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | RED — Write failing tests for format gate, atomic copy, conflict detection | 53d4f23 | crates/takoyaki-app/src/commands/samples.rs |
| 2 | GREEN — Implement format gate, atomic copy, overwrite param in assign_sample | 6a6692e | crates/takoyaki-app/src/commands/samples.rs |

## What Was Built

Three surgical fixes to `assign_sample` in `crates/takoyaki-app/src/commands/samples.rs`:

**WR-02: Format validation gate**
- Calls `health::read_audio_spec(&canonical_source)` immediately after path resolution, before any snapshot or file write
- Iterates `check_format_compatibility` results — returns `AppError::Parse` on first `FormatIssue::UnsupportedFormat`
- Soft issues (WrongSampleRate, WrongBitDepth) are not blocked here — user already accepted them in dry-run

**WR-03: Atomic Wallflower copy**
- Stages file to `.filename.tmp` in the same `AUDIO/` directory (same FAT32 volume = kernel-level atomic rename)
- Replaces bare `std::fs::copy` which could leave a partial file on USB disconnect
- Best-effort cleanup removes `.tmp` if `rename` fails

**WR-04: Overwrite/conflict parameter**
- `assign_sample` now takes `overwrite: bool`
- Returns `Err(AppError::Io("CONFLICT: {filename} already exists on OT card"))` when `dest.exists() && !overwrite`
- When `overwrite=true`, proceeds through atomic copy (clobbers existing file atomically)

## Tests Added (4 new, all in existing `#[cfg(test)] mod tests` block)

| Test | Validates |
|------|-----------|
| `test_assign_rejects_unsupported_format` | `read_audio_spec` + `check_format_compatibility` returns `UnsupportedFormat` for non-audio file |
| `test_wallflower_atomic_copy_no_partial` | Copy-to-.tmp then rename leaves final file, no .tmp remnant |
| `test_wallflower_conflict_when_dest_exists` | `dest.exists() && !overwrite` is true when file exists and overwrite=false |
| `test_wallflower_overwrite_when_flag_true` | `dest.exists() && !overwrite` is false when overwrite=true |

All 25 unit tests pass. Full workspace green.

## Deviations from Plan

None — plan executed exactly as written.

## Threat Surface Scan

No new network endpoints, auth paths, or trust boundaries introduced. Changes are entirely within existing `assign_sample` function body. Threat model entries T-08-01 through T-08-04 are addressed:
- T-08-01 (format gate): mitigated — `read_audio_spec` + `UnsupportedFormat` check before any write
- T-08-02 (atomic copy): mitigated — temp-then-rename pattern in same AUDIO dir
- T-08-03 (CONFLICT message): accepted — error message contains only filename, not full path
- T-08-04 (overwrite bypass): mitigated — `overwrite` param is explicit bool, defaults to false in Plan 02 TS wrapper

## Known Stubs

None. The `overwrite` parameter defaults are enforced in the TypeScript frontend wrapper — Plan 02 covers that side.

## TDD Gate Compliance

- RED gate commit: `53d4f23` — `test(08-01): add unit tests for format gate, atomic copy, and conflict detection`
- GREEN gate commit: `6a6692e` — `feat(08-01): add format gate, atomic copy, and overwrite param to assign_sample`
- REFACTOR gate: not needed (implementation was clean on first pass)

## Self-Check: PASSED

| Check | Result |
|-------|--------|
| `samples.rs` exists | FOUND |
| `08-01-SUMMARY.md` exists | FOUND |
| Commit `53d4f23` (RED) | FOUND |
| Commit `6a6692e` (GREEN) | FOUND |
| `overwrite: bool` in signature | FOUND |
| `health::read_audio_spec(&canonical_source)` in assign_sample | FOUND |
| `CONFLICT:` error string | FOUND |
| `.tmp` staging pattern | FOUND |
