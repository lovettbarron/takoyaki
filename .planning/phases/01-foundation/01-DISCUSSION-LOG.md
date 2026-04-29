# Phase 1: Foundation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-29
**Phase:** 01-foundation
**Areas discussed:** Clean-room parser strategy, App scaffold & initial feel, Test fixture sourcing, Volume detection UX

---

## Clean-room Parser Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Community knowledge only | Elektronauts forum posts, OT user manuals, community wiki pages. No reading ot-tools-io source code or documentation at all. | |
| Read about, don't copy | Read forum discussions referencing ot-tools-io findings, but never look at source code. Build own implementation from descriptions + hex analysis. | |
| Full clean-room wall | One person reads ot-tools-io and writes a format spec document. A separate person implements from that spec only. | |

**User's choice:** Read ot-tools-io for format knowledge, write own implementation independently. (User asked about GPL implications first — confirmed that format specs are facts, not copyrightable expression, so no clean-room process needed.)
**Notes:** User initially asked whether using ot-tools-io would require their whole project to be GPL. After discussing the GPL v3 copyleft spectrum (use as dependency → fork → read and reimplement → full clean-room), user chose option 3: read to learn, write independently.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Preserve verbatim | Store unknown byte regions as opaque blobs, write them back unchanged. | |
| Hex-analyze your files | Use own OT project files to compare hex dumps across different project states. | |
| Both — preserve now, analyze over time | Ship with verbatim preservation, incrementally decode using own files as living test corpus. | ✓ |

**User's choice:** Both — preserve now, analyze over time
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| All core types | Parse everything: .work, .strd, .ot, bank files, marker files. | ✓ |
| Project + bank first | Start with .work and bank files. Defer .ot and marker files to Phase 2. | |
| You decide | Claude determines scoping. | |

**User's choice:** All core types
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| Standalone crate | Separate ot-parser crate in a Cargo workspace. No Tauri/IO dependency. | ✓ |
| Inline in Tauri | Parser modules live directly in src-tauri/src/. | |
| You decide | Claude determines architecture. | |

**User's choice:** Standalone crate
**Notes:** None

---

## App Scaffold & Initial Feel

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal status screen | Single screen showing OT connection status. No navigation. | |
| Structured shell | Sidebar navigation skeleton with placeholder sections. | |
| Connection-first flow | Prominent 'Connect your Octatrack' screen that transitions on connect. | |

**User's choice:** Clear application shell borrowing Octatrack/Elektron visual language with monome design influence.
**Notes:** User typed freeform — wanted OT + Elektron visual language with a hint of monome design. Not any of the three presented options directly.

---

| Option | Description | Selected |
|--------|-------------|----------|
| OT OLED dark | Near-black background, high contrast, teal/cyan accents. | |
| Monome warm dark | Dark but warm grays, off-white text, subtle warmth. Refined desktop app. | ✓ |
| Elektron muted | Dark charcoal, muted accents. Overbridge/Transfer style. | |

**User's choice:** Monome warm dark
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| Full sidebar skeleton | Sidebar with sections visible from Phase 1, inactive sections disabled/grayed. | ✓ |
| Grow organically | No sidebar in Phase 1, appears in Phase 2. | |
| Top bar only | Horizontal navigation at top. | |

**User's choice:** Full sidebar skeleton
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| Monospace-forward | Primary monospace font. Proportional only for longer descriptions. | ✓ |
| Clean sans-serif | Modern sans-serif primary. Monospace for values/paths only. | |
| You decide | Claude picks typography. | |

**User's choice:** Monospace-forward
**Notes:** None

---

## Test Fixture Sourcing

| Option | Description | Selected |
|--------|-------------|----------|
| My own projects | User provides own OT project directories. | |
| Synthetic + mine | Own projects for baseline + synthetic for edge cases. | ✓ |
| You decide | Claude determines fixture strategy. | |

**User's choice:** Synthetic + mine
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| Committed in repo | Binary fixtures checked into git under tests/fixtures/. | ✓ |
| Git LFS | Store via Git LFS. | |
| You decide | Claude picks approach. | |

**User's choice:** Committed in repo
**Notes:** OT binary files are small (KB-range), no need for LFS.

---

## Volume Detection UX

| Option | Description | Selected |
|--------|-------------|----------|
| Subtle status change | Indicator quietly updates. No interruption. | |
| Active notification | Brief toast notification, sidebar updates. | |
| Auto-navigate | App switches to Projects view automatically. Connection is the trigger. | ✓ |

**User's choice:** Auto-navigate
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| Empty state prompt | Main area shows 'Connect your Octatrack' message. Sidebar grayed out. | |
| Always-ready shell | App looks normal, content shows 'No device' inline. Sidebar stays active. | ✓ |
| You decide | Claude picks disconnected state. | |

**User's choice:** Always-ready shell
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| Single device | One OT volume at a time. | ✓ |
| Multi-device aware | Detect and list all mounted OT volumes. | |

**User's choice:** Single device
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| Directory structure sniff | Look for OT folder structure. No user config needed. | |
| User points to volume | Manual volume path selection in settings. | |
| Auto-detect + confirm | Auto-detect by structure, then confirm with user. | ✓ |

**User's choice:** Auto-detect + confirm
**Notes:** None

---

## Claude's Discretion

- Exact monospace font selection
- Specific accent color within warm dark palette
- Sidebar section icons and disabled state styling
- Volume detection polling interval vs filesystem event approach
- Synthetic test fixture generation strategy
- Snapshot storage format and directory structure
- SQLite schema design

## Deferred Ideas

None — discussion stayed within phase scope
