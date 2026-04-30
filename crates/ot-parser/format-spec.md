# Octatrack Binary Format Specification (Clean-Room)

## Methodology

Format facts extracted by reading ot-tools-io 0.6.0 public API documentation on docs.rs
(struct field names, sizes, types) and its public source code viewer (constant values,
field ordering). No code was copied. This document records FORMAT FACTS (not copyrightable)
that inform independent Rust implementation.

Source: https://docs.rs/ot-tools-io/0.6.0/ot_tools_io/  [accessed 2026-04-30]
Repository: https://gitlab.com/ot-tools/ot-tools-io (GPL v3 — used for format study only)

All multi-byte integer fields are big-endian unless otherwise noted.
ot-tools-io uses bincode for binary serialization (little-endian by default), but the
OT hardware format appears to be big-endian based on the .ot file verification.

> NOTE on endianness: The .ot file is verified big-endian (from OctaChainer otwriter.h).
> The ot-tools-io crate uses bincode with its default settings. Bincode uses little-endian
> integers. This creates ambiguity for the other file types. The resolution strategy:
> use big-endian for all known numeric fields (matching .ot) and validate with real OT files.
> If round-trip tests fail on real files, switch to little-endian.
> UPDATE: After implementation testing, use the endianness that produces passing round-trips.

---

## .ot (SampleSettingsFile) — VERIFIED

Total: 832 bytes
Header magic (first 16 bytes): [0xF0, 0x00, 0x00, 0xE8, 0x57, 0x45, 0x52, 0x41, 0x00×8]
Checksum: 16-bit additive over bytes 0x00..0x33D (inclusive), big-endian

| Offset | Size | Type     | Field              | Notes                              |
|--------|------|----------|--------------------|------------------------------------|
| 0x00   | 16   | u8[16]   | header             | Magic bytes (first 4: F0 00 00 E8) |
| 0x10   | 7    | u8[7]    | unknown_0x10       | Preserve verbatim (D-02)           |
| 0x17   | 4    | u32 BE   | tempo              | Tempo in BPM × 6 (range: 30–300)  |
| 0x1B   | 4    | u32 BE   | trim_len           | Trim length in audio samples       |
| 0x1F   | 4    | u32 BE   | loop_len           | Loop length in audio samples       |
| 0x23   | 4    | u32 BE   | stretch            | Time stretch amount                |
| 0x27   | 4    | u32 BE   | loop_flag          | Loop enable flag                   |
| 0x2B   | 2    | u16 BE   | gain               | Gain value                         |
| 0x2D   | 1    | u8       | quantize           | Quantize setting                   |
| 0x2E   | 4    | u32 BE   | trim_start         | Trim start in audio samples        |
| 0x32   | 4    | u32 BE   | trim_end           | Trim end in audio samples          |
| 0x36   | 4    | u32 BE   | loop_point         | Loop point in audio samples        |
| 0x3A   | 768  | Slice[64]| slices             | 64 × 12 bytes: start/end/loop u32  |
| 0x33A  | 4    | u32 BE   | slice_count        | Number of active slices            |
| 0x33E  | 2    | u16 BE   | checksum           | 16-bit additive, bytes 0x00..0x33D |

Source: OctaChainer `otwriter.h` [VERIFIED]

---

## bankNN.work / bankNN.strd (BankFile)

Total: Unverified — treat body as opaque blob (D-02)
Files: bank01.work through bank16.work (1-indexed filenames, 0-indexed internally)

Header magic (21 bytes):
- bytes 0x00..0x03: "FORM" = [0x46, 0x4F, 0x52, 0x4D]
- bytes 0x04..0x07: [0x00, 0x00, 0x00, 0x00] (4 null bytes)
- bytes 0x08..0x0B: "DPS1" = [0x44, 0x50, 0x53, 0x31]
- bytes 0x0C..0x0F: "BANK" = [0x42, 0x41, 0x4E, 0x4B]
- bytes 0x10..0x14: [0x00, 0x00, 0x00, 0x00, 0x00] (5 null bytes)

File version field: byte 0x15 = 0x17 (decimal 23)

Source: ot-tools-io source, `BANK_HEADER: [u8; 21] = [70, 79, 82, 77, 0, 0, 0, 0, 68, 80, 83, 49, 66, 65, 78, 75, 0, 0, 0, 0, 0]`
        `BANK_FILE_VERSION: u8 = 23`

| Offset | Size     | Type    | Field             | Notes                                                   |
|--------|----------|---------|-------------------|---------------------------------------------------------|
| 0x00   | 21       | u8[21]  | header            | Magic bytes (see above)                                 |
| 0x15   | 1        | u8      | datatype_version  | Version = 23 (0x17) for OS 1.40B                        |
| 0x16   | ?        | u8[N]   | opaque_body       | Patterns + Parts data — size unknown, preserve verbatim |
| last-2 | 2        | u16     | checksum          | Non-trivial algorithm (see Checksum section)            |

Sub-structures present (from ot-tools-io API docs — sizes not independently verified):
- PatternArray: 16 × Pattern structs
- Pattern: header[u8;8] + audio_track_trigs + midi_track_trigs + scale + chain_behaviour + unknown(u8) + part_assignment(u8) + tempo_1(u8) + tempo_2(u8)
- Parts: 4 × Part structs (each Part has ~24 fields including FX, scenes, tracks)
- parts_saved_state: [u8; 4]
- parts_edited_bitmask: u8
- part_names: [[u8; 7]; 4] (4 parts × 7-byte ASCII names)

Default part names: "ONE\0\0\0\0", "TWO\0\0\0\0", "THREE\0\0", "FOUR\0\0\0"
= [0x4F,0x4E,0x45,0,0,0,0], [0x54,0x57,0x4F,0,0,0,0],
  [0x54,0x48,0x52,0x45,0x45,0,0], [0x46,0x4F,0x55,0x52,0,0,0]

Unknown regions:
- 0x16..last-2: The full body between header+version and checksum is treated as an opaque
  blob. Sizes of Pattern, Part, and sub-structures cannot be independently verified without
  running ot-tools-io against real OT files. Preserve verbatim per D-02.

Checksum algorithm: Complex (non-trivial) — see Checksum Algorithm section below.
Default checksum value for blank bank: 48022 (0xBB76)

**Implementation strategy:** Parse as header[21] + version(u8) + opaque_body(Vec<u8>) +
checksum(u16). The body size = total_file_size - 21 - 1 - 2. For round-trip fidelity,
store checksum verbatim (do not recalculate).

---

## markers.work / markers.strd (MarkersFile)

Total: 207,000 bytes (0x32898)
Calculation: 21 + 1 + (136 × 784) + (128 × 784) + 2

Header magic (21 bytes):
- bytes 0x00..0x03: "FORM" = [0x46, 0x4F, 0x52, 0x4D]
- bytes 0x04..0x07: [0x00, 0x00, 0x00, 0x00] (4 null bytes)
- bytes 0x08..0x0B: "DPS1" = [0x44, 0x50, 0x53, 0x31]
- bytes 0x0C..0x0F: "SAMP" = [0x53, 0x41, 0x4D, 0x50]
- bytes 0x10..0x14: [0x00, 0x00, 0x00, 0x00, 0x00] (5 null bytes)

Source: ot-tools-io source, `MARKERS_HEADER: [u8; 21] = [0x46, 0x4f, 0x52, 0x4d, 0x00, 0x00, 0x00, 0x00, 0x44, 0x50, 0x53, 0x31, 0x53, 0x41, 0x4d, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00]`
        `MARKERS_FILE_VERSION: u8 = 4`

| Offset   | Size     | Type              | Field            | Notes                              |
|----------|----------|-------------------|------------------|------------------------------------|
| 0x00     | 21       | u8[21]            | header           | Magic bytes (see above)            |
| 0x15     | 1        | u8                | datatype_version | Version = 4                        |
| 0x16     | 106,624  | SlotMarkers[136]  | flex_slots       | 136 flex machine slots             |
| 0x1A116  | 100,352  | SlotMarkers[128]  | static_slots     | 128 static machine slots           |
| 0x32896  | 2        | u16               | checksum         | Non-trivial algorithm              |

SlotMarkers sub-structure (784 bytes each):

| Sub-offset | Size | Type       | Field       | Notes                               |
|------------|------|------------|-------------|-------------------------------------|
| +0x00      | 4    | u32        | trim_offset | Main sample trim start (in samples) |
| +0x04      | 4    | u32        | trim_end    | Main sample trim end (in samples)   |
| +0x08      | 4    | u32        | loop_point  | Main sample loop point (in samples) |
| +0x0C      | 768  | Slice[64]  | slices      | 64 × 12 bytes slice data           |
| +0x30C     | 4    | u32        | slice_count | Number of active slices             |

Slice sub-structure (12 bytes each):

| Sub-offset | Size | Type | Field       | Notes                                      |
|------------|------|------|-------------|---------------------------------------------|
| +0x00      | 4    | u32  | trim_start  | Slice start position in audio samples       |
| +0x04      | 4    | u32  | trim_end    | Slice end position in audio samples         |
| +0x08      | 4    | u32  | loop_start  | Slice loop point (0xFFFFFFFF = no loop)     |

**Implementation strategy:** Parse full structure — SlotMarkers is fully characterized.
Store checksum verbatim for round-trip fidelity.

---

## arrNN.work / arrNN.strd (ArrangementFile)

Total: Unverified — treat body as opaque blob (D-02)
Files: arr01.work through arr08.work (8 arrangement files per project)

Header magic (21 bytes):
- bytes 0x00..0x03: "FORM" = [0x46, 0x4F, 0x52, 0x4D]
- bytes 0x04..0x07: [0x00, 0x00, 0x00, 0x00] (4 null bytes)
- bytes 0x08..0x0B: "DPS1" = [0x44, 0x50, 0x53, 0x31]
- bytes 0x0C..0x0F: "ARRA" = [0x41, 0x52, 0x52, 0x41]
- bytes 0x10..0x14: [0x00, 0x00, 0x00, 0x00, 0x00] (5 null bytes)

Source: ot-tools-io source, `ARRANGEMENT_FILE_HEADER: [u8; 21] = [70, 79, 82, 77, 0, 0, 0, 0, 68, 80, 83, 49, 65, 82, 82, 65, 0, 0, 0, 0, 0]`
        `ARRANGEMENT_FILE_VERSION: u8 = 6`

| Offset | Size | Type     | Field                           | Notes                             |
|--------|------|----------|---------------------------------|-----------------------------------|
| 0x00   | 21   | u8[21]   | header                          | Magic bytes (see above)           |
| 0x15   | 1    | u8       | datatype_version                | Version = 6                       |
| 0x16   | ?    | u8[N]    | opaque_body                     | Body — size unknown, preserve verbatim |
| last-2 | 2    | u16      | checksum                        | Non-trivial algorithm             |

Sub-structures present (from ot-tools-io API docs — sizes not independently verified):
- unk1: [u8; 2] (unknown — "Dunno. Example data: [0, 0]")
- arrangement_state_current: ArrangementBlock (current active arrangement)
- unk2: u8 (unknown)
- saved_flag: u8 (whether arrangement saved)
- arrangement_state_previous: ArrangementBlock (previous saved state)
- arrangements_saved_state: [u8; 8]

ArrangementBlock sub-structure (size unverified):
- name: [u8; 15] (ASCII arrangement name)
- unknown_1: [u8; 2] (unknown, usually [0,0] or [0,1])
- n_rows: u8 (number of active rows; 256 rows → value 0 per docs)
- rows: Array<ArrangeRow, 256> (fixed array of 256 arrangement rows)

ArrangeRow enum (fixed binary size, discriminant determines type):
- PatternRow: pattern_id(u8) + repetitions(u8) + mute_mask(u8) + tempo_1(u8) + tempo_2(u8) + scene_a(u8) + scene_b(u8) + offset(u8) + length(u8) + midi_transpose[u8;8]
- LoopOrJumpOrHaltRow: loop_count(u8) + row_target(u8)
- ReminderRow: String (variable length — exact binary layout unknown)
- EmptyRow: no fields

Default arrangement name: "OCTATOOLS-ARR  "
= [0x4F,0x43,0x54,0x41,0x54,0x4F,0x4F,0x4C,0x53,0x2D,0x41,0x52,0x52,0x20,0x20]

Unknown regions:
- 0x16..last-2: Full body treated as opaque blob (sizes of ArrangeRow fixed representation
  and ArrangementBlock are not independently determinable from docs alone).

Checksum algorithm: Complex (non-trivial) — see Checksum Algorithm section below.

**Implementation strategy:** Parse as header[21] + version(u8) + opaque_body(Vec<u8>) +
checksum(u16). For round-trip fidelity, store checksum verbatim.

---

## project.work / project.strd (ProjectFile)

**IMPORTANT NOTE:** The ot-tools-io ProjectFile is TEXT-BASED, not binary. From the docs:
"project files are actually string data being parsed directly without any serde-ing or
bincode-ing." The project.work and project.strd files contain KEY=VALUE text data, not
a fixed binary layout.

Structure: The file is a text file (likely UTF-8 or ASCII) with key-value pairs grouped
into sections: metadata, settings, states, slots.

Fields identified from ot-tools-io API (all text-formatted):
- metadata: OS version, project name, creation date
- settings: tempo, quantize, and other project-level settings
- states: mute state and other runtime state
- slots: 128 sample slot assignments (flex × 128 + static × 128)

**Implementation strategy (REVISED):** Since project.work is text-based, it cannot be
parsed with binrw. Implement ProjectFile as:
1. Store raw bytes verbatim (opaque Vec<u8>) for full round-trip fidelity
2. Optionally parse key=value sections for data extraction
3. No binary struct field decomposition possible without text parsing

File size: Variable (text content, no fixed size)
Header magic: None (text file, no binary magic bytes)
Checksum: None identified in ot-tools-io docs (text file)

> ASSUMPTION A-PROJ: project.work/.strd are text files. This is inferred from the
> ot-tools-io docs statement about "string data being parsed directly". If they contain
> a binary prefix or suffix, this must be revised with real OT files.

---

## Checksum Algorithm

### Background

The ot-tools-io checksum for binary file types (BankFile, MarkersFile, ArrangementFile)
is NOT a simple additive checksum. From the source code study:

The algorithm (for BankFile as reference):
1. Encode the struct with `datatype_version` field set (using bincode)
2. Encode a default instance of the struct (using bincode)
3. Compare encoded bytes from index 16 to len-2 (body without header/checksum)
4. Compute sum of (current_byte as i64) - (default_byte as i64) for each byte pair
5. Apply complex modular arithmetic with 256 and BANK_DEFAULT_CHECKSUM (48022 = 0xBB76)
6. Result is a u16 checksum

**For round-trip fidelity:** We do NOT need to recalculate checksums. Since we:
- Read the file verbatim (including its stored checksum)
- Store the checksum in the struct as a u16 field
- Write the checksum back verbatim when serializing

This guarantees byte-exact round-trip WITHOUT needing to implement the checksum algorithm.

**Checksum verification:** When reading real OT files in later phases, we can add an
optional checksum validator that:
1. Reads the stored checksum
2. Recalculates (if algorithm is implemented)
3. Warns if mismatch (never rejects — OT files from older firmware may have different checksums)

### Reference Constants

| File Type       | Default Checksum | File Version |
|-----------------|------------------|--------------|
| BankFile        | 48022 (0xBB76)   | 23 (0x17)    |
| MarkersFile     | Unknown          | 4            |
| ArrangementFile | Unknown          | 6            |

---

## Assumptions

| ID   | Assumption | Risk if Wrong | Resolution |
|------|------------|---------------|------------|
| A-01 | project.work/.strd are text (key=value) format | Binary format; wrong implementation | Verify with real OT file (hex dump) |
| A-02 | Multi-byte integers in bank/markers/arr files follow big-endian (matching .ot) | Wrong byte order; checksum tests fail | Round-trip test on real OT files |
| A-03 | Checksum stored verbatim; no recalculation needed for round-trip | N/A — storing verbatim always round-trips | No risk for storage; risk only if checksum validation is needed |
| A-04 | SlotMarkers is 784 bytes (4+4+4+64×12+4) | Parser fails if size wrong | Verify with real markers.work (wc -c) |
| A-05 | MarkersFile total is 207,000 bytes | Parser fails if size wrong | Verify with real markers.work (wc -c) |
| A-06 | BankFile and ArrangementFile body sizes are unknowable without running ot-tools-io | Can't size-validate; only round-trip test catches this | Real file test |

---

## Index Schemes

| File Type  | Index Scheme | Notes                              |
|------------|-------------|-------------------------------------|
| project.work/.strd | N/A (text) | slot_id references are 1-indexed |
| bankNN.work/.strd  | 0-indexed internal | Filename is 1-indexed (bank01=0) |
| markers.work/.strd | 0-indexed   | flex_slots[0..135], static_slots[0..127] |
| arrNN.work/.strd   | 0-indexed   | arr01=arrangement[0]              |
| *.ot               | N/A         | Per-sample sidecar file           |

Source: ot-tools-io docs — "HasFileVersionField", index scheme confirmed in module docs.
