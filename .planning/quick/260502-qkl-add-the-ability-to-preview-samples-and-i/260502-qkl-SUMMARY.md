---
phase: quick
plan: 260502-qkl
subsystem: samples-audio-preview
tags: [audio, preview, playback, samples, ux]
dependency_graph:
  requires: [health/resolve_ot_path, commands/samples, SamplesTab]
  provides: [get_sample_audio_bytes command, useAudioPreview hook, play button UI]
  affects: [SlotRow, SamplesTab, tauri.ts, lib.rs]
tech_stack:
  added: []
  patterns: [blob-url-audio-playback, tauri-vec-u8-to-uint8array]
key_files:
  created:
    - src/hooks/useAudioPreview.ts
  modified:
    - crates/takoyaki-app/src/commands/samples.rs
    - crates/takoyaki-app/src/lib.rs
    - src/lib/tauri.ts
    - src/components/project-detail/SlotRow.tsx
    - src/components/project-detail/SamplesTab.tsx
decisions:
  - "Used AppError::Io instead of NotFound (variant does not exist) for file-not-found errors"
  - "Strip leading ../ from sample_path before resolve_ot_path since OT stores relative paths"
  - "Blob URL + HTMLAudioElement for playback (no external audio library needed)"
metrics:
  duration: 209s
  completed: 2026-05-02
---

# Quick Plan 260502-qkl: Audio Preview for Sample Slots Summary

Audio preview play/stop buttons on occupied sample slot rows, backed by a Rust command that reads WAV/AIFF bytes from the mounted OT volume and streams them to the browser audio API via blob URLs.

## Task Completion

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Add Rust get_sample_audio_bytes command | 213a049 | samples.rs, lib.rs |
| 2 | Frontend audio preview hook + play button UI | d7d08c4 | useAudioPreview.ts, SlotRow.tsx, SamplesTab.tsx, tauri.ts |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] AppError::NotFound does not exist**
- **Found during:** Task 1
- **Issue:** Plan referenced `AppError::NotFound` but the error enum only has Io, Parse, Database, Lock, Device, InvalidPath, Cancelled
- **Fix:** Used `AppError::Io` with descriptive error messages for all not-found cases
- **Files modified:** crates/takoyaki-app/src/commands/samples.rs

**2. [Rule 1 - Bug] OT relative paths with ../ prefix**
- **Found during:** Task 1
- **Issue:** `full_path` field contains paths like `../AUDIO/Alb/sample.WAV` which would fail resolve_ot_path since it expects paths relative to volume root without traversal
- **Fix:** Added `trim_start_matches("../")` to strip the relative prefix before passing to resolve_ot_path
- **Files modified:** crates/takoyaki-app/src/commands/samples.rs

## Verification

- Rust: `cargo check -p takoyaki-app` -- compiles with 0 errors (2 pre-existing dead code warnings)
- Frontend: `npx next build` -- compiles successfully, 0 type errors

## Self-Check: PASSED
