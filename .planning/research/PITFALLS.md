# Pitfalls Research

**Domain:** Desktop hardware sampler file manager — Octatrack binary format tool
**Researched:** 2026-04-29
**Confidence:** HIGH (critical pitfalls verified against ot-tools-io source, Elektronauts community reports, and macOS filesystem documentation)

---

## Critical Pitfalls

### Pitfall 1: One/Zero Index Mismatch Corrupts Sample Slot References

**What goes wrong:**
The Octatrack project file stores sample slot IDs as one-indexed values. Every other file type — BankFile, MarkersFile, SampleSettingsFile — stores slot IDs as zero-indexed values. Code that fails to convert between these two coordinate systems writes slot references that are off by one, silently assigning the wrong sample to every slot. This is the most common source of subtle data corruption in OT tooling.

**Why it happens:**
The ot-tools-io docs explicitly warn: "Project Sample Slots store their slots with a one-indexed `slot_id` field. All other references to sample slots are zero-indexed." Developers reading one file type in isolation will infer an incorrect index model and propagate it across all file writes.

**How to avoid:**
Define distinct Rust newtypes: `ProjectSlotId(u8)` (one-indexed) and `SlotRef(u8)` (zero-indexed). Make conversion explicit and compile-enforced. Never use raw `u8` for slot IDs in the parser. Write round-trip tests that move a sample through project → bank → markers and verify every reference resolves to the same sample after a full read-write-read cycle.

**Warning signs:**
- Any sample-slot operation that "works" but loads one slot off from what the user assigned
- Tests comparing slot IDs from project and bank files without an explicit conversion step
- Raw integer arithmetic on slot IDs anywhere in the codebase

**Phase to address:**
Binary format parser (Phase 1 / Foundation). Must be correct before any write operations are built.

---

### Pitfall 2: Treating Multi-File Operations as Separate Writes

**What goes wrong:**
Moving a single sample slot on the Octatrack requires coordinated changes across up to 18 files: the project file (.work), all bank files that reference that slot, and all marker files that point to it. If the application writes some files and then crashes, or writes them sequentially with any failure in between, the project ends up in an inconsistent state. The Octatrack firmware has no repair mechanism for partial writes — it either loads a corrupt project or refuses to load it at all.

**Why it happens:**
The ot-tools-io author explicitly documents this: "moving a single sample slot requires changing up to 18 files." Developers building "edit one file at a time" approaches underestimate the cross-file dependency graph. Early prototypes that only write the project file appear to work on the OT until the bank references are loaded.

**How to avoid:**
Implement a changeset model from day one. An operation produces a `ChangeSet { files: Vec<(Path, Bytes)> }` value that is only committed together. The commit path is: (1) auto-snapshot all affected files to a staging area, (2) write all new files to a temp directory on the same volume, (3) atomically rename each file into place in dependency order, (4) verify checksums on all written files. Never write individual files outside this transaction abstraction.

**Warning signs:**
- Any code path that writes a single OT file without consulting a dependency graph
- Missing test coverage for crash-during-write scenarios
- Operations that skip bank file updates when changing project sample slots

**Phase to address:**
Atomic write engine (Phase 1 / Foundation). The changeset model must exist before the first write operation of any kind is implemented.

---

### Pitfall 3: Zeroing Unknown / Reserved Bytes Destroys Future Compatibility

**What goes wrong:**
The OT binary format has many fields that are unknown or undocumented. A naive parser that initializes all struct fields to zero, or that re-serializes only the fields it understands, will zero out reserved bytes on every write. When Elektron uses those bytes in a future firmware version — or when those bytes already carry unknown meaning — the written file is silently corrupted relative to the original. The OT firmware may then refuse to load the project, produce unexpected behavior, or exhibit data loss on the next OT-side save.

**Why it happens:**
Binary parsing libraries like `binrw` require you to explicitly capture and preserve bytes you don't understand. The default behavior of any struct-based parser is to drop unrecognized bytes. The ot-tools-io library itself notes that only 68.37% of the crate is documented — unknown fields remain in the format.

**How to avoid:**
Every struct in the parser must carry opaque `[u8; N]` fields for all unknown/reserved regions. Use `binrw`'s `#[br(map = ...)]` or raw `RestorePosition` to capture bytes verbatim. The fundamental invariant: `parse(serialize(parse(file_bytes))) == parse(file_bytes)` for every real OT project file. Build this as an automated test fixture that runs against a corpus of real project files before any write path is merged.

**Warning signs:**
- Any `_reserved: u8` or `_unknown: [u8; N]` field that is set to zero rather than read from input
- Missing round-trip tests against real OT project fixtures
- Parser that uses `#[br(ignore)]` on fields rather than preserving them

**Phase to address:**
Binary format parser (Phase 1 / Foundation). Enforce as a parser invariant from the first struct definition.

---

### Pitfall 4: OS Version Mismatch Causes Silent Incompatibility

**What goes wrong:**
OT project files embed an OS version number. The ot-tools-io library defines an `ALLOWED_OS_VERSIONS` constant and adds a specific error for OS version mismatches — because writing a project file with an incorrect or unrecognized OS version field causes the Octatrack firmware to refuse to load the project, displaying "project must be corrupt or saved with a newer version of octatrack OS." This is a hard failure with no recovery on the hardware side.

**Why it happens:**
Developers copy a known-good project structure without updating the OS version field. Or they hard-code a version from their test unit and ship it, not realizing users on different firmware versions will get load failures. New firmware releases may bump the version field even when no format changes occurred.

**How to avoid:**
Read the OS version from each project file before any write operation. Round-trip the version verbatim — never synthesize it. Add a pre-write validation step that checks the OS version field against a known-valid list and warns (but does not block) if the version is unrecognized. Surface a clear user-facing error that distinguishes "firmware version mismatch" from "file corruption" so users know to check their OT firmware version.

**Warning signs:**
- Project file structure that generates the OS version field from a constant rather than reading it from the source file
- Users reporting "project won't load on OT" after a write operation that appeared to succeed
- No test coverage for reading and re-writing projects at different OS version values

**Phase to address:**
Binary format parser and write validation (Phase 1 / Foundation).

---

### Pitfall 5: Loop Point Default Value Inconsistency Breaks Markers Round-Trip

**What goes wrong:**
Disabled loop points are represented as `0xFFFFFFFF` in markers files and sample settings files when a sample is loaded into a slot. However, newly created markers default to `0_u32`. Writing a newly-constructed markers entry with the wrong default for disabled loop points creates a markers file that differs from what the OT firmware would produce — and this difference may be interpreted as a valid (but wrong) loop point at offset 0.

**Why it happens:**
This is a documented quirk in ot-tools-io: "a `0xFFFFFFFF` value in markers or sample settings files... the disabled value only applies when samples are loaded into slots and sample settings files are generated." Developers creating new entries from struct defaults use `0_u32`, which means "loop at byte offset 0," not "disabled."

**How to avoid:**
Define a `LoopPoint` enum (`Disabled` | `At(u32)`) with explicit serialization: `Disabled` serializes to `0xFFFFFFFF`. Never use raw `u32` for loop points. All new markers entries must use `LoopPoint::Disabled` as the default, not zero.

**Warning signs:**
- Any `loop_point: u32 = 0` in struct initialization
- Missing test: create a new sample entry, write it, read it back, verify loop point is disabled not zero-offset

**Phase to address:**
Binary format parser (Phase 1 / Foundation). Enforce as a parser invariant.

---

### Pitfall 6: USB Hot-Unplug Leaves CF Card in Inconsistent State

**What goes wrong:**
The Octatrack CF card is a FAT32 volume. macOS buffers writes to FAT32 volumes. If the user physically unplugs the OT USB cable — or if the application fails to flush and unmount before the user removes the card — in-progress or buffered writes are lost, FAT directory entries become inconsistent, and projects that appeared to save correctly are corrupted. This is the most common real-world data loss vector reported in Elektronauts threads.

**Why it happens:**
macOS's write buffer is not synchronously flushed to FAT32 volumes by default. The application may have completed its write call without the data reaching the card. Users who eject the OT from hardware while it is still in USB mode, or who quit the app without an explicit "eject" step, lose data.

**How to avoid:**
After every atomic write commit: (1) call `fsync` on every written file descriptor, (2) call `fsync` on the parent directory, (3) wait for `diskutil unmount` or equivalent DiskArbitration API confirmation before reporting success to the user. Expose a prominent "Safely eject OT" button in the UI that blocks until the unmount completes. Never report a write as "done" to the user until `fsync` + directory sync is confirmed. Display a persistent warning banner whenever the OT volume is mounted but the application has pending writes.

**Warning signs:**
- Write operations that report success based on Rust's `File::write_all` return value alone
- No `fsync` calls after writes
- No volume-unmount step in the write confirmation flow
- UI that closes or dismisses the progress indicator before `fsync` completes

**Phase to address:**
Atomic write engine (Phase 1 / Foundation) and USB volume management (Phase 1 / Foundation).

---

### Pitfall 7: FSEvents Is Unreliable for Detecting OT Volume Changes

**What goes wrong:**
Tauri's default file watching on macOS uses FSEvents, which was designed for Time Machine backups — not real-time file monitoring. FSEvents coalesces multiple events into single notifications, delivers them out of order, and can miss events entirely. For detecting when the OT volume appears, disappears, or has files modified externally, FSEvents will silently miss events or deliver them seconds late.

**Why it happens:**
FSEvents is the default macOS file system notification mechanism, and many libraries use it transparently. Watchexec's own documentation states FSEvents "is not designed for the use cases Notify is used for" and notes that newer versions of Notify switched to Kqueue for this reason.

**How to avoid:**
Use the macOS DiskArbitration framework (via a Rust FFI binding or Tauri plugin) to detect volume mount/unmount events — not FSEvents. For file change detection within the OT volume, use Kqueue-based watchers (the `kqueue` crate or `notify` crate with Kqueue backend) rather than FSEvents. Treat any volume event as "rescan everything" rather than trying to track individual file changes.

**Warning signs:**
- Using `tauri-plugin-fs-watch` without explicitly selecting the Kqueue backend
- File watcher that appears to miss OT card insertions in manual testing
- No integration test for "volume appears mid-session" scenario

**Phase to address:**
USB volume management (Phase 1 / Foundation).

---

### Pitfall 8: Reading Wallflower's SQLite Without Concurrency Awareness Causes DB Locking

**What goes wrong:**
Wallflower holds an open write connection to its SQLite database during normal operation. Takoyaki opens a second read connection to the same file. Without WAL mode enabled on Wallflower's DB, Takoyaki's read connection can conflict with Wallflower's write lock, causing `SQLITE_BUSY` errors or stale reads. Takoyaki has no control over whether Wallflower enables WAL mode.

**Why it happens:**
SQLite's default journal mode (DELETE) allows only one writer or reader at a time. Even a read-only connection from Takoyaki can be blocked when Wallflower is mid-transaction. Developers assume "read-only" means "no locking concerns," which is incorrect in SQLite's default mode.

**How to avoid:**
Open Wallflower's DB with `PRAGMA journal_mode=WAL` if not already set (WAL allows concurrent readers with a single writer). Add `PRAGMA busy_timeout=3000` to tolerate brief write contention. Treat all Wallflower DB reads as best-effort: if the query returns `SQLITE_BUSY` or a timeout, surface a user-facing message ("Wallflower library temporarily unavailable") rather than crashing. Never write to Wallflower's DB. Add a graceful-degradation path so Takoyaki remains fully functional when Wallflower's DB is inaccessible.

**Warning signs:**
- Wallflower integration that crashes Takoyaki on `SQLITE_BUSY`
- No timeout configured on Wallflower DB connections
- Takoyaki's UI becoming unresponsive when Wallflower is performing a large import

**Phase to address:**
Wallflower integration (later phase, after core OT functionality works). The graceful-degradation contract should be defined at the feature boundary.

---

### Pitfall 9: Clean-Room Implementation Contaminated by GPL Source

**What goes wrong:**
The project requires a clean-room Rust parser — MIT licensed, no GPL code. ot-tools-io is GPL v3. If any contributor to Takoyaki has read ot-tools-io source in detail and then directly transcribes field names, struct layouts, or algorithmic patterns into Takoyaki code, the clean-room defense is compromised. If Elektron were to challenge the implementation, the project would need to demonstrate independent derivation.

**Why it happens:**
ot-tools-io is the most complete public reference for OT binary formats. Developers naturally read it as documentation. The line between "using it as a spec reference" and "copying its implementation" is easy to cross without noticing.

**How to avoid:**
Adopt a documented clean-room process: one researcher reads ot-tools-io and community documentation, writes a format specification document in plain language with no code, has that spec reviewed by a lawyer (or at minimum documented as independent), and then a separate implementer writes the parser from the spec only. Never include ot-tools-io as a build dependency — not even in dev/test targets. Use community documentation sources (Elektronauts threads, OctaLib research notes, independent hex analysis of real files) as the primary reference. Document all sources in the parser crate's README.

**Warning signs:**
- Variable/field names in Takoyaki's parser that match ot-tools-io naming verbatim
- `ot-tools-io` appearing in any Cargo.toml (workspace or per-crate)
- No documented specification artifact that predates the implementation

**Phase to address:**
Binary format parser (Phase 1 / Foundation). The specification document must exist before coding begins.

---

### Pitfall 10: Snapshot Storage Bloat From Naive Binary Snapshots

**What goes wrong:**
The three-layer safety model requires auto-snapshots before every write. A naive implementation that copies all project files verbatim for each snapshot will generate large amounts of redundant data. A project with 10 banks and marker files at ~500KB each, snapshotted before every sample slot change, bloats to gigabytes of snapshot history within a normal session. Users eventually disable the safety system to recover disk space, defeating its purpose.

**Why it happens:**
Binary files don't diff meaningfully with standard tools. Git LFS helps with storage but adds operational complexity. The easiest implementation is a flat copy, and developers underestimate how often users perform write operations.

**How to avoid:**
Use content-addressed storage for snapshot blobs: SHA-256 each file before snapshotting, and only store a new blob if the hash differs from the most recent snapshot of that file. A changeset then stores references (hashes) not full copies. This gives deduplication automatically — if only the project file changed, the 16 bank files are not re-stored. Store snapshots in Takoyaki's own SQLite DB as blobs with hash primary keys. Add a configurable retention policy (e.g., keep last N snapshots per project, or prune snapshots older than 30 days).

**Warning signs:**
- Snapshot directory growing by the full project size on each write operation
- No hash-based deduplication in the snapshot storage layer
- Users reporting "Takoyaki is eating my disk space" in early testing

**Phase to address:**
Snapshot/versioning engine (Phase 2, after atomic writes are stable).

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Use raw `u8` for slot IDs instead of newtypes | Faster initial parsing | One/zero index bugs in every write path; hard to audit | Never |
| Skip `fsync` after writes, rely on OS flush | Faster perceived write speed | Silent data loss on unexpected dismount | Never |
| Copy ot-tools-io field names directly | Faster reverse engineering | GPL contamination of the clean-room implementation | Never |
| Store full file copies for snapshots | Simple implementation | Disk bloat; users disable safety model | MVP only, with hard cap on snapshot count |
| Use FSEvents for volume detection | No extra dependencies | Missed mount/unmount events; unreliable OT detection | Never for mount detection; maybe for within-volume file tracking |
| Hard-code OS version constant in parser | Avoids version-check complexity | Project files rejected by OT after user firmware updates | Never |
| Write files sequentially outside changeset model | Simpler code path | Partial writes leave OT in unrecoverable inconsistent state | Never |
| Skip round-trip tests for binary parser | Faster initial development | Unknown fields get zeroed; format corruption on first real use | Never |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Wallflower SQLite DB | Opening with default journal mode and assuming reads are non-blocking | Open read-only with WAL hint and `busy_timeout=3000`; gracefully degrade if unavailable |
| OT CF card via USB | Relying on write success without `fsync` | `fsync` every written file + parent directory; block success confirmation until complete |
| macOS volume mount detection | Using FSEvents or `tauri-plugin-fs-watch` default | Use DiskArbitration framework for mount/unmount events; Kqueue for file changes |
| OT project files | Reading one file type to infer slot indexing convention | Verify index convention per file type; use newtypes to make conversion explicit |
| ot-tools-io as format reference | Treating it as documentation, importing its structs | Use only as a secondary reference; derive all format knowledge from independent analysis + community docs |

---

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Scanning all 16 bank files on every UI refresh | UI stalls when OT volume is mounted | Parse bank files lazily, cache parsed representation in SQLite, invalidate on file change | Immediately on any project with 16 banks |
| Synchronous `fsync` blocking the UI thread | App appears frozen during write operations | All write operations must run in Tauri async commands on a separate thread; UI stays responsive | Every write operation if not addressed |
| Snapshot storage without deduplication | Disk fills after moderate use | Content-addressed blob storage with hash deduplication | Projects with large sample sets, after ~20 writes |
| Parsing all OT files at volume mount time | Slow initial load | Parse project index only at mount; defer bank/marker parsing to on-demand | Projects with 16 banks and full marker files |

---

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Writing to arbitrary paths provided through UI | Path traversal — attacker crafts a project name that writes outside CF card mount point | Canonicalize all write paths; validate they are children of the mounted OT volume path before any write |
| Reading Wallflower SQLite path from user config without validation | User-provided path could point to system DB or sensitive file | Validate the SQLite file contains expected Wallflower schema tables before querying |
| Auto-executing files found on OT CF card | Malicious project file could contain an executable disguised as a .work file | Never execute files from the OT volume; treat all content as data only |

---

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Reporting write success before `fsync` completes | User unplugs OT immediately, loses work, blames the tool | Show "Syncing to card..." state that only resolves after `fsync` + directory sync confirmed |
| Showing technical slot indices (0–127) instead of OT's displayed numbering | Users can't correlate UI with what they see on the OT screen | Map all slot displays to OT's one-indexed, user-visible numbering (1–128) |
| Offering "revert to snapshot" without showing what changed | Users don't know which snapshot to pick; risk reverting to wrong version | Show a diff summary (which files changed, which samples changed) before confirming a revert |
| Requiring the OT to be in USB mode before launching the app | Users forget; app shows confusing "no OT found" state | Detect OT volume presence at any time; show clear "Connect OT in USB mode" instruction with a waiting indicator |
| Silently failing the Wallflower integration | Users think search is broken, not that Wallflower is temporarily busy | Show clear status: "Wallflower library connected" / "Wallflower unavailable — check if app is running" |

---

## "Looks Done But Isn't" Checklist

- [ ] **Binary parser round-trip:** Parser reads a real OT project file, serializes it back, and byte-for-byte matches the original — verify this with a fixture corpus before calling the parser "done"
- [ ] **Slot index conversion:** Every code path that moves a slot ID across file-type boundaries has an explicit index conversion — grep for raw `u8` slot arithmetic and confirm every instance goes through a typed conversion
- [ ] **Atomic writes:** "Write a sample slot assignment" test checks that a simulated crash after writing 9 of 18 files leaves the project loadable on the OT (or at least recoverable from snapshot) — not just that the happy path works
- [ ] **fsync confirmation:** Write operations do not return success to the UI until `fsync` is confirmed on every written file — unit test that mocks the syscall and verifies it is called
- [ ] **OS version passthrough:** A project written by Takoyaki and reloaded by the OT firmware does not show any version error — requires field testing with a real OT unit
- [ ] **Snapshot deduplication:** After 10 writes that each change only one file, snapshot storage has grown by only ~10x one-file-size, not 10x full-project-size
- [ ] **Wallflower degradation:** Wallflower DB unavailable (file locked, process holding write lock) does not crash or freeze Takoyaki — all OT management features continue to work
- [ ] **Volume unmount race:** User ejects OT volume while a write is in progress — Takoyaki detects the unmount, aborts the write, restores from snapshot, and reports a clear error

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| One/zero index bug shipped in write path | HIGH | Audit all write paths, add newtypes, re-test all operations against real OT; users may need to re-create affected projects |
| Partial write left project inconsistent | MEDIUM | Auto-snapshot means revert is available; surface "last known good" snapshot in UI; user reloads from snapshot |
| Unknown bytes zeroed by parser | HIGH | Cannot recover already-written projects; must re-derive correct byte values from fresh OT-generated versions; issue urgent patch |
| Hot-unplug data loss | HIGH (user-facing) | Nothing to recover if `fsync` was not called; snapshot-before-write protects the previous state on the card if the write was partial |
| GPL contamination | HIGH (legal/project) | Requires full clean-room rewrite of affected parser code; potential project shutdown if not caught early |
| Snapshot storage bloat | LOW | Add retention policy, run cleanup; no data lost |
| Wallflower DB lock crash | LOW | Restart Takoyaki; implement graceful degradation in next release |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| One/zero index mismatch | Phase 1: Binary parser | Automated round-trip tests; slot-move integration test across all file types |
| Multi-file partial writes | Phase 1: Atomic write engine | Crash-simulation test; verify OT loads project after simulated mid-write failure |
| Unknown bytes zeroed | Phase 1: Binary parser | Byte-exact round-trip test against real OT project corpus |
| OS version mismatch | Phase 1: Binary parser | Field test: write and reload on OT hardware at current firmware version |
| Loop point default confusion | Phase 1: Binary parser | Unit test: new marker entry with disabled loop reads back as 0xFFFFFFFF not 0x0 |
| USB hot-unplug data loss | Phase 1: USB volume management | Integration test: mock `fsync` + directory sync; UI only confirms after both return |
| FSEvents unreliability | Phase 1: USB volume management | Integration test: volume mount/unmount detection via DiskArbitration |
| Wallflower SQLite locking | Wallflower integration phase | Test: Takoyaki query while Wallflower holds write lock; verify graceful degradation |
| GPL contamination | Phase 1: Pre-implementation | Spec document exists before code; no ot-tools-io in any Cargo.toml |
| Snapshot storage bloat | Phase 2: Snapshot engine | Measure: 10 writes to one file produces <1.5x full-project snapshot growth |

---

## Sources

- ot-tools-io Rust documentation: https://docs.rs/ot-tools-io (confirmed: one/zero index mismatch, OS version constants, loop point defaults, arrangement checksum issues)
- Elektronauts thread on bank.work decoding: https://www.elektronauts.com/t/how-to-decode-bank-work-files-as-a-third-party-app/172666
- Elektronauts ot-tools-io announcement thread: https://www.elektronauts.com/t/ot-tools-io-open-source-rust-library-for-reading-writing-modifying-octatrack-files/232508
- Elektronauts OT file corruption reports: https://www.elektronauts.com/t/ot-files-corrupted/191275
- Watchexec FSEvents limitations: https://watchexec.github.io/docs/macos-fsevents.html
- SQLite locking documentation: https://sqlite.org/lockingv3.html
- LWN: Atomic file writes: https://lwn.net/Articles/789600/
- Clean-room reverse engineering legal analysis: https://www.retroreversing.com/clean-room-reversing
- binrw documentation: https://docs.rs/binrw

---
*Pitfalls research for: Takoyaki — Octatrack desktop backup/versioning/file management tool*
*Researched: 2026-04-29*
