# Phase 1: Foundation - Context

**Gathered:** 2026-04-29
**Status:** Ready for planning

<domain>
## Phase Boundary

A Tauri app exists that can detect a mounted OT volume, parse all OT binary file types with byte-exact fidelity, and safely write any file atomically — with snapshot infrastructure in place before any user-facing write operation is ever built. The app shell establishes the visual identity and navigation structure for all future phases.

</domain>

<decisions>
## Implementation Decisions

### Parser Strategy
- **D-01:** Read ot-tools-io source to learn OT binary format structure (field offsets, data types, layout), then write an independent Rust implementation. No code copying, no GPL contamination. Format specifications are facts, not copyrightable expression — this is legally sound without a formal clean-room process.
- **D-02:** Unknown/undocumented byte regions (~31.6% of format) are preserved verbatim as opaque blobs during round-trip. Parser evolves incrementally as understanding grows — user's own OT project files serve as a living corpus for hex analysis over time.
- **D-03:** Phase 1 parses ALL core OT file types: .work (project), .strd (bank arrangements), .ot (sample metadata/slices), bank files, and marker files. No deferral — full coverage needed for any write operation.
- **D-04:** Parser lives in a standalone `ot-parser` crate within a Cargo workspace. No Tauri dependency, no I/O. Pure parsing library that can be tested and potentially published independently.

### App Scaffold & Visual Identity
- **D-05:** Application shell draws from three design languages: Octatrack's functional hardware aesthetic, Elektron's industrial precision, and monome's minimal grid warmth. Hardware-inspired but refined as a desktop app.
- **D-06:** Monome warm dark color palette — dark but slightly warm grays, off-white text, subtle warmth. Not screen-emulation (no OLED-style pure black), not corporate muted. A refined, warm desktop app that feels native to the music hardware world.
- **D-07:** Full sidebar navigation skeleton from Phase 1 with all future sections visible (Projects, Samples, Backups, Settings). Inactive sections are disabled/grayed out — users see where the app is going.
- **D-08:** Monospace-forward typography — primary monospace font throughout the UI. Proportional font reserved for longer descriptive text only. Reinforces the hardware/technical aesthetic.

### Test Fixtures
- **D-09:** Test corpus combines user's own real OT project files (baseline coverage of real-world patterns) with synthetic edge cases (empty projects, maxed-out slots, unusual configurations, boundary conditions).
- **D-10:** Binary test fixtures committed directly in the git repo under tests/fixtures/. OT files are small (KB-range), no need for Git LFS.

### Volume Detection UX
- **D-11:** On OT connect, app auto-navigates to the Projects view. The connection event IS the trigger to start working — no extra clicks.
- **D-12:** Disconnected state is an always-ready shell — sidebar stays active, content areas show "No device" inline. Users can still access settings, backup history, or other non-device features without a connected OT.
- **D-13:** Single device support only. One OT volume at a time. If multiple mounted, use first detected or let user pick.
- **D-14:** Auto-detect OT volumes by directory structure sniffing (look for /AUDIO, /SETS, characteristic file patterns), then show user confirmation: "Found Octatrack at /Volumes/OT-CARD. Use this?"

### Claude's Discretion
- Exact monospace font selection (Iosevka, JetBrains Mono, Berkeley Mono, etc.)
- Specific accent color within the warm dark palette
- Sidebar section icons and disabled state styling
- Volume detection polling interval vs filesystem event approach
- Synthetic test fixture generation strategy
- Snapshot storage format and directory structure
- SQLite schema design for backup history and project index

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

No external specs or ADRs exist yet — requirements are fully captured in decisions above and project-level documents below.

### Project-Level
- `.planning/PROJECT.md` — Core value, constraints, key decisions, prior art analysis
- `.planning/REQUIREMENTS.md` — Full v1 requirement list with traceability to phases
- `.planning/ROADMAP.md` — Phase 1 success criteria and dependency chain

### OT Format Knowledge (to be read during research)
- ot-tools-io source code (GitHub) — Format structure reference for independent implementation
- Elektronauts forum — Community-documented OT binary format knowledge

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- None — greenfield project, no existing code

### Established Patterns
- Wallflower (sister project) uses the same Tauri v2 + React/Next.js stack — architectural patterns from Wallflower should inform scaffold decisions

### Integration Points
- Cargo workspace root for ot-parser crate + src-tauri crate
- Tauri v2 commands bridge Rust backend to React frontend
- tauri-specta generates TypeScript types from Rust command signatures
- SQLite via rusqlite (bundled) for Takoyaki metadata
- Read-only SQLite connection to Wallflower DB (FNDN-08) — driver-level write protection

</code_context>

<specifics>
## Specific Ideas

- Visual identity blends Octatrack/Elektron hardware aesthetic with monome's warm minimalism — think norns-meets-Overbridge
- Monospace-forward typography reinforces the hardware/technical feel throughout the UI, not just in data displays
- The disconnected state should feel calm and ready, not like an error — the app is useful even without a device

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-foundation*
*Context gathered: 2026-04-29*
