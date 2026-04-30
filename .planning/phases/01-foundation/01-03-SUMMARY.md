---
phase: 01-foundation
plan: "03"
subsystem: parser
tags: [rust, binrw, ot-parser, binary-format, newtypes, round-trip, tdd]

requires:
  - phase: 01-foundation/01-01
    provides: Cargo workspace with ot-parser crate, error.rs with ParseError

provides:
  - SampleSettingsFile parser: binrw struct with from_bytes/to_bytes for .ot format (832 bytes)
  - Slice struct: 12-byte slice entry within .ot files
  - ProjectSlotId newtype: 1-indexed slot IDs (1..=256) with range enforcement
  - BankSlotId newtype: 0-indexed slot IDs (0..=255) with type-level distinction
  - BankNumber newtype: bank 0..=15 with filename 1..=16 conversion
  - tests/fixtures/sample.ot: 832-byte synthetic .ot test fixture
  - Round-trip test suite (5 tests) + indexing boundary test suite (10 tests)

affects: [01-04, 01-05, all future ot-parser plans]

tech-stack:
  added: []
  patterns:
    - "binrw + from_bytes/to_bytes pattern established for all OT file type parsers"
    - "TDD cycle: failing tests committed first, then implementation"
    - "Opaque blob preservation: unknown_0x10 [u8; 7] preserved verbatim (D-02)"
    - "Newtype boundary enforcement at type level (FNDN-03)"

key-files:
  created:
    - crates/ot-parser/src/sample.rs
    - crates/ot-parser/src/types.rs
    - crates/ot-parser/tests/round_trip.rs
    - crates/ot-parser/tests/indexing.rs
    - tests/fixtures/sample.ot
  modified:
    - crates/ot-parser/src/lib.rs

key-decisions:
  - "Fixture generated as 832-byte Python binary blob with real field values (not all-zeros) to test field parsing"
  - "BankSlotId::new() accepts any u8 (no runtime check needed since u8 range == valid range 0..=255)"
  - "Checksum field preserved verbatim in round-trip (0x0000 placeholder) — algorithm deferred to Plan 04 when real files available"

patterns-established:
  - "Pattern: binrw OT file type — #[binrw] #[brw(big)] struct + from_bytes/to_bytes methods"
  - "Pattern: unknown byte preservation — [u8; N] fixed-size arrays for undocumented regions"
  - "Pattern: indexing newtypes — separate types for 1-indexed vs 0-indexed prevent off-by-one bugs"

requirements-completed: [FNDN-01, FNDN-02, FNDN-03]

duration: 7min
completed: 2026-04-30
---

# Phase 01 Plan 03: .ot Parser and Indexing Newtypes Summary

**binrw-based SampleSettingsFile parser with byte-exact round-trip, 832-byte synthetic fixture, and ProjectSlotId/BankSlotId/BankNumber newtypes enforcing 1-indexed vs 0-indexed distinction**

## Performance

- **Duration:** ~7 min
- **Started:** 2026-04-30T04:36:00Z
- **Completed:** 2026-04-30T04:43:36Z
- **Tasks:** 2 (both TDD with RED/GREEN commits)
- **Files modified:** 6

## Accomplishments

- Implemented SampleSettingsFile parser using binrw with all 832-byte .ot format fields including 7-byte unknown region preserved verbatim
- Created synthetic test fixture with real field values (tempo, trim/loop points, two populated slices) to exercise actual parsing
- Established the binrw + from_bytes/to_bytes pattern that all future OT file type parsers will follow
- Implemented ProjectSlotId (1..=256), BankSlotId (0..=255), and BankNumber (0..=15) newtypes preventing index confusion at compile time
- All 15 tests pass: 5 round-trip tests + 10 indexing boundary tests

## Task Commits

Each task was committed atomically (TDD: test commit then implementation commit):

1. **Task 1 RED: Failing round-trip tests** - `3676a5f` (test)
2. **Task 1 GREEN: SampleSettingsFile parser + fixture** - `2619f6f` (feat)
3. **Task 2 RED: Failing indexing tests** - `b06a04e` (test)
4. **Task 2 GREEN: Indexing newtypes** - `ec12214` (feat)

**Plan metadata:** _(docs commit follows)_

## Files Created/Modified

- `crates/ot-parser/src/sample.rs` - SampleSettingsFile and Slice binrw structs with from_bytes/to_bytes
- `crates/ot-parser/src/types.rs` - ProjectSlotId, BankSlotId, BankNumber newtypes with boundary enforcement
- `crates/ot-parser/src/lib.rs` - Added pub mod sample, pub mod types and re-exports
- `crates/ot-parser/tests/round_trip.rs` - 5 tests: parse, round-trip, parse equality, wrong size, wrong magic
- `crates/ot-parser/tests/indexing.rs` - 10 tests: boundary enforcement for all three newtype types
- `tests/fixtures/sample.ot` - 832-byte synthetic .ot fixture with valid magic and field values

## Decisions Made

- Generated fixture with Python rather than a Rust build script — simpler for a one-time synthetic fixture; real OT files will supplement later (D-09/D-10)
- `BankSlotId::new()` accepts any u8 without error return because `u8` range already enforces 0..=255 — the newtype provides type-level distinction, not additional range validation
- Checksum field (offset 0x33E) stored as 0x0000 placeholder in fixture — the checksum algorithm will be validated against real files in Plan 04 when the clean-room format spec is created

## Deviations from Plan

None — plan executed exactly as written. All binrw patterns, struct fields, test cases, and fixture layout matched the plan spec verbatim.

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- `.ot` parser pattern established; Plans 04+ can follow the same binrw + from_bytes/to_bytes + round-trip test pattern for .work, bank, markers, and arrangement files
- Indexing newtypes ready for use in all parser modules that reference project slots or bank slots
- Synthetic fixture in `tests/fixtures/` — user can add real OT files alongside it at any time
- Remaining concern: checksum algorithm for .ot and other file types is a placeholder (0x0000) — Plan 04 will derive the correct algorithm from real files

---
*Phase: 01-foundation*
*Completed: 2026-04-30*

## Self-Check: PASSED

- FOUND: crates/ot-parser/src/sample.rs
- FOUND: crates/ot-parser/src/types.rs
- FOUND: crates/ot-parser/tests/round_trip.rs
- FOUND: crates/ot-parser/tests/indexing.rs
- FOUND: tests/fixtures/sample.ot
- FOUND: .planning/phases/01-foundation/01-03-SUMMARY.md
- FOUND: 3676a5f (test RED round-trip)
- FOUND: 2619f6f (feat GREEN parser)
- FOUND: b06a04e (test RED indexing)
- FOUND: ec12214 (feat GREEN newtypes)
