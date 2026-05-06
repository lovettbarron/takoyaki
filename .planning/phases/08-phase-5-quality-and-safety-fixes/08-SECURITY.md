---
phase: 08
slug: phase-5-quality-and-safety-fixes
status: verified
threats_open: 0
asvs_level: 1
created: 2026-05-06
---

# Phase 08 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| Frontend -> assign_sample | User-supplied file_path, overwrite flag, and from_wallflower bool cross into Rust backend via Tauri IPC | String paths, booleans (low sensitivity) |
| OT card filesystem | Writes to FAT32 volume must be atomic to prevent corruption on USB disconnect | Binary audio files, OT project metadata |
| Error string parsing | Frontend matches "CONFLICT:" prefix in error messages from backend | Error strings containing filenames (no full paths) |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-08-01 | Tampering | assign_sample format gate | mitigate | `health::read_audio_spec` + `check_format_compatibility` called before any file write or snapshot; blocks `UnsupportedFormat` with `AppError::Parse` (samples.rs:451-458) | closed |
| T-08-02 | Tampering | Wallflower file copy | mitigate | Temp-then-rename pattern: copy to `.filename.tmp` in same `AUDIO/` dir, then `rename`; best-effort cleanup on rename failure (samples.rs:488-498) | closed |
| T-08-03 | Information Disclosure | CONFLICT error message | accept | Error message contains only filename, not full path — consistent with existing error patterns (samples.rs:483) | closed |
| T-08-04 | Denial of Service | overwrite parameter bypass | mitigate | `overwrite` param defaults to `false` in TypeScript wrapper (tauri.ts:186); Rust requires explicit `true` to overwrite — no silent data loss (samples.rs:481) | closed |
| T-08-05 | Spoofing | overwrite param from frontend | mitigate | TypeScript wrapper defaults `overwrite` to `false` via `overwrite ?? false`; only `handleConflictOverwrite` passes `true` after explicit user confirmation (SamplesTab.tsx:456) | closed |
| T-08-06 | Information Disclosure | conflict prompt filename | accept | Shows only filename extracted from error string via regex, not full path — same information already visible in slot list (SamplesTab.tsx:387) | closed |
| T-08-07 | Tampering | CONFLICT: string matching | accept | String prefix match is fragile but acceptable — both Rust producer (samples.rs:483) and TS consumer (SamplesTab.tsx:384) are in same codebase; no cross-team boundary; typed error variant alternative has higher complexity for identical security posture | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-08-01 | T-08-03 | CONFLICT error exposes only filename (e.g. "kick.wav"), not filesystem path. Same filename is already visible in the UI slot list. No additional information disclosure. | gsd-secure-phase | 2026-05-06 |
| AR-08-02 | T-08-06 | Conflict prompt shows filename extracted from error string. Same data already displayed in the samples tab slot rows. No path traversal or sensitive path exposure. | gsd-secure-phase | 2026-05-06 |
| AR-08-03 | T-08-07 | CONFLICT: string prefix matching between Rust backend and TypeScript frontend is fragile but both sides are maintained in the same codebase by the same team. A typed error variant would add IPC complexity for no security improvement. | gsd-secure-phase | 2026-05-06 |

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-05-06 | 7 | 7 | 0 | gsd-secure-phase |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-05-06
