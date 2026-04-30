---
phase: 01-foundation
plan: 04
subsystem: parser
tags: [rust, binrw, ot-parser, binary-format, round-trip, tdd]

# Dependency graph
requires:
  - phase: 01-03
    provides: "SampleSettingsFile parser, binrw opaque blob pattern, round_trip.rs test infrastructure"

provides:
  - "Clean-room format specification for all OT binary file types (format-spec.md)"
  - "ProjectFile parser: text-based project.work/.strd, verbatim byte storage"
  - "BankFile parser: FORM+DPS1BANK header, opaque body, verbatim checksum, round-trip exact"
  - "MarkersFile parser: FORM+DPS1SAMP header, opaque body, verbatim checksum, round-trip exact"
  - "ArrangementFile parser: FORM+DPS1ARRA header, opaque body, verbatim checksum, round-trip exact"
  - "Integration test covering all five OT file types"
  - "Synthetic fixtures: project.work, bank01.work, markers.work, arr01.work"

affects:
  - "02-read-only-browser (reads all five file types)"
  - "03-write-engine (writes back all five file types with round-trip guarantee)"
  - "04-project-management (ProjectFile, BankFile are primary edit targets)"

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Opaque blob parser: header[N] + version(u8) + opaque_body(Vec<u8>) + checksum(u16 BE) — universal pattern for OT binary files with non-trivial checksum"
    - "Verbatim checksum storage: non-trivial checksums stored as parsed u16, not recalculated — guarantees round-trip without understanding the algorithm"
    - "Text-as-opaque: project.work is text key=value, stored as raw bytes verbatim"

key-files:
  created:
    - crates/ot-parser/format-spec.md
    - crates/ot-parser/src/project.rs
    - crates/ot-parser/src/bank.rs
    - crates/ot-parser/src/markers.rs
    - crates/ot-parser/src/arrangement.rs
    - tests/fixtures/project.work
    - tests/fixtures/bank01.work
    - tests/fixtures/markers.work
    - tests/fixtures/arr01.work
  modified:
    - crates/ot-parser/src/lib.rs
    - crates/ot-parser/tests/round_trip.rs

key-decisions:
  - "project.work/.strd is a text key=value file — confirmed from ot-tools-io docs. Stored as opaque raw bytes, not parsed with binrw."
  - "All binary OT file types use FORM+DPS1+TYPE header (21 bytes) followed by version byte, opaque body, and u16 checksum"
  - "Checksum stored verbatim (not recalculated) — the algorithm requires bincode serialization comparison to a default instance, which is non-trivial to implement clean-room and not needed for round-trip"
  - "BankFile, MarkersFile, ArrangementFile body sizes are not independently verifiable from docs alone — treat as opaque Vec<u8> per D-02"

patterns-established:
  - "Pattern: OT binary opaque blob — header[21] + version(u8) + opaque_body(Vec<u8>) + checksum(u16) for binary file types with FORM magic"
  - "Pattern: format-spec.md as clean-room evidence document before any parser code is written"

requirements-completed: [FNDN-01, FNDN-02]

# Metrics
duration: 9min
completed: 2026-04-30
---

# Phase 01 Plan 04: OT Binary Format Parsers Summary

**Clean-room format spec and five OT file type parsers (ProjectFile, BankFile, MarkersFile, ArrangementFile, SampleSettingsFile) all byte-exact round-trip via opaque blob + verbatim checksum pattern**

## Performance

- **Duration:** 9 min
- **Started:** 2026-04-30T05:26:33Z
- **Completed:** 2026-04-30T05:35:33Z
- **Tasks:** 3 (Task 0 + Task 1 TDD + Task 2 TDD)
- **Files modified:** 10 (4 new parser modules, 4 new fixtures, lib.rs, round_trip.rs)

## Accomplishments

- Created clean-room format spec document (`format-spec.md`) documenting header magic, version constants, and field layouts for all four remaining OT file types — from ot-tools-io 0.6.0 public API docs (format facts, no code copied)
- Implemented all four remaining parsers using the opaque blob pattern: header magic validation, verbatim body storage, verbatim checksum — byte-exact round-trip for all types
- Integration test `test_all_types_round_trip` covers all five OT file types simultaneously
- 29 total tests pass (10 indexing + 19 round-trip including 5 new file types)

## Task Commits

Each task was committed atomically, with TDD red/green phases:

0. **Task 0: Clean-room format spec** - `f309cc3` (docs)
1. **Task 1 RED: Failing ProjectFile + BankFile tests** - `581ba1b` (test)
2. **Task 1 GREEN: ProjectFile + BankFile implementation** - `91c7e07` (feat)
3. **Task 2 RED: Failing MarkersFile + ArrangementFile tests** - `5a02e29` (test)
4. **Task 2 GREEN: MarkersFile + ArrangementFile implementation** - `5ecd6dc` (feat)

## Files Created/Modified

- `crates/ot-parser/format-spec.md` — Clean-room format spec for all OT binary file types; includes header magic constants, version numbers, checksum algorithm analysis
- `crates/ot-parser/src/project.rs` — ProjectFile: text-based project.work stored as opaque Vec<u8>
- `crates/ot-parser/src/bank.rs` — BankFile: FORM+DPS1BANK header, version=23, opaque body, u16 checksum
- `crates/ot-parser/src/markers.rs` — MarkersFile: FORM+DPS1SAMP header, version=4, opaque body, u16 checksum
- `crates/ot-parser/src/arrangement.rs` — ArrangementFile: FORM+DPS1ARRA header, version=6, opaque body, u16 checksum
- `crates/ot-parser/src/lib.rs` — Updated to export all five OT file types
- `crates/ot-parser/tests/round_trip.rs` — Added 14 new tests for four file types + integration test
- `tests/fixtures/project.work` — Synthetic text key=value fixture
- `tests/fixtures/bank01.work` — Synthetic binary fixture (FORM header, 100-byte body, zero checksum)
- `tests/fixtures/markers.work` — Synthetic binary fixture (FORM+SAMP header, 100-byte body)
- `tests/fixtures/arr01.work` — Synthetic binary fixture (FORM+ARRA header, 100-byte body)

## Decisions Made

- **project.work is text, not binary:** The ot-tools-io docs state project files are "string data being parsed directly without any serde-ing or bincode-ing." Implemented as opaque raw bytes — the cleanest approach that guarantees round-trip fidelity regardless of OT OS version.

- **Universal FORM header pattern:** All four binary OT file types share the same 21-byte FORM header structure: "FORM" + 4×0x00 + "DPS1" + 4-char type-code + 5×0x00. Type codes: BANK, SAMP, ARRA.

- **Verbatim checksum storage:** The ot-tools-io checksum algorithm is non-trivial (involves bincode serialization + comparison to default instance + modular arithmetic with type-specific constants). Rather than attempt a clean-room reimplementation, checksum is stored verbatim as a u16 field. This guarantees round-trip byte-exactness without implementing the algorithm. Validation can be added in a later phase when real OT files are available for testing.

- **Opaque body pattern (D-02 compliance):** BankFile, MarkersFile, and ArrangementFile body sizes cannot be independently verified from public docs alone. Treating the entire body as a `Vec<u8>` opaque blob per D-02 is both correct and safe — no data interpretation means no risk of corruption.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug/Discovery] project.work is text-based, not a binrw binary struct**
- **Found during:** Task 0 (format spec research) and Task 1 (implementation)
- **Issue:** The plan specified implementing ProjectFile with `#[binrw]` and field offsets, but ot-tools-io docs explicitly state "project files are actually string data being parsed directly." There are no magic bytes, no binary field layout, no checksum.
- **Fix:** Implemented ProjectFile as a verbatim opaque byte store (raw: Vec<u8>). This fully satisfies the plan's round-trip requirement and D-02. The plan's TDD tests pass with from_bytes/to_bytes/raw field as specified.
- **Files modified:** crates/ot-parser/src/project.rs (new file)
- **Verification:** test_project_round_trip, test_project_preserves_unknown_regions, test_project_parse all pass
- **Committed in:** 91c7e07 (Task 1 GREEN)

---

**Total deviations:** 1 auto-fixed (Rule 1 — format discovery, plan was designed from incomplete format knowledge)
**Impact on plan:** The deviation improved the implementation. Text-as-opaque-bytes is more correct than a speculative binary parser for a text file. All plan acceptance criteria met.

## Issues Encountered

The ot-tools-io checksum algorithm is visible in the source but non-trivial to implement clean-room. The algorithm diff-encodes against a default instance using bincode byte comparison. For Phase 1 scope (round-trip fidelity), verbatim storage is the correct approach. If checksum validation is needed in a later phase, it can be added independently.

## Known Stubs

None — synthetic fixtures are minimal (100-byte opaque body) but functionally complete for round-trip testing. Real OT files will be required for integration validation (Phase 3+). The fixtures serve their stated purpose: byte-exact round-trip with synthetic data.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes introduced. Threat model mitigations implemented as specified:
- T-01-05 (DoS): All parsers validate minimum file size before parsing
- T-01-06 (Tampering): Opaque regions preserved verbatim — accepted by design
- T-01-07 (Spoofing): All binary parsers validate first 4 magic bytes ("FORM")

## Next Phase Readiness

- The ot-parser crate is a complete, standalone Rust library with no Tauri or I/O dependencies
- All five OT file types have working parsers with byte-exact round-trip fidelity
- The crate is ready for use by the Tauri app (Phase 2 read-only browser)
- Real OT project files are needed for integration validation — synthetic fixtures cover CI only

---
*Phase: 01-foundation*
*Completed: 2026-04-30*
