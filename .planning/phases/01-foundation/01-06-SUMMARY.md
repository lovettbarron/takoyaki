---
phase: 01-foundation
plan: 06
subsystem: ui
tags: [react, nextjs, zustand, tauri, sidebar, device-status, tailwind]

# Dependency graph
requires:
  - phase: 01-02
    provides: Next.js frontend scaffold with shadcn, Tailwind CSS, warm dark theme, and CSS variables

provides:
  - App shell layout with 220px sidebar and content area
  - Sidebar navigation skeleton with Projects/Samples/Backups/Settings (Projects active, rest disabled)
  - Device status badge showing connected/disconnected state
  - Zustand device store with connected/mountPoint/confirmed state
  - TauriEventListener component bridging ot-device-changed events to device store
  - Calm disconnected state display with correct UI-SPEC.md copy

affects: [01-07, 02-03, 02-04, 02-05]

# Tech tracking
tech-stack:
  added: [zustand (device store)]
  patterns:
    - Zustand store for device state (useDeviceStore hook pattern)
    - Dynamic import of @tauri-apps/api/event inside async function to prevent SSR errors
    - cleanupFns array pattern for Tauri event listener cleanup on unmount
    - Disabled nav items use opacity-50 + pointer-events-none + cursor-not-allowed (UI-SPEC.md contract)
    - Active sidebar item uses amber accent left-border indicator (3px rounded-r strip)

key-files:
  created:
    - src/lib/stores/device.ts
    - src/components/device-status-badge.tsx
    - src/components/sidebar-nav.tsx
    - src/components/tauri-event-listener.tsx
  modified:
    - src/app/page.tsx
    - src/app/layout.tsx

key-decisions:
  - "TauriEventListener uses dynamic import + try/catch so Next.js dev server (browser context) doesn't crash when Tauri API is unavailable"
  - "Sidebar nav items use custom button elements (not shadcn Sidebar) to satisfy 44px WCAG 2.5.5 touch target without fighting shadcn's default sizing"
  - "Device status badge shows volume name extracted from mount path tail (split('/').pop()) — minimal and informative"

patterns-established:
  - "Tauri event listener pattern: dynamic import inside async setupListeners(), cleanupFns array, return null"
  - "Disabled nav: opacity-50 pointer-events-none cursor-not-allowed — enforced per UI-SPEC Disabled State Contract"
  - "Active nav: amber accent left-border (3px strip, absolute-positioned) + accent text + accent/15 bg"

requirements-completed: [FNDN-06, BROW-01]

# Metrics
duration: 15min
completed: 2026-04-30
---

# Phase 1 Plan 06: Frontend UI Shell Summary

**Warm-dark app shell with 220px sidebar skeleton, zustand device store, amber-accent active nav, and Tauri event bridge wired to ot-device-changed events**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-04-30T05:09:00Z
- **Completed:** 2026-04-30T05:24:24Z
- **Tasks:** 3 (2 auto + 1 checkpoint, approved)
- **Files modified:** 6

## Accomplishments

- Full app shell layout: 220px sidebar (warm card bg) + flex-1 content area, matching UI-SPEC.md Layout Contract
- Sidebar navigation with all four sections (Projects active, Samples/Backups/Settings disabled) with amber accent indicator and WCAG 2.5.5 touch targets
- Zustand device store with connected/mountPoint/confirmed state and typed setters
- TauriEventListener component using safe dynamic import pattern for ot-device-changed events
- Calm "No Device Connected" disconnected state with exact UI-SPEC.md copywriting
- Device status badge with green (hsl 140 60% 42%) / gray (hsl 30 8% 38%) indicator dots
- Human visual verification approved: warm dark theme, monospace typography, disabled nav, v0.1.0 footer

## Task Commits

Each task was committed atomically:

1. **Task 1: Create sidebar navigation, device status badge, and zustand device store** - `4b3462f` (feat)
2. **Task 2: Wire app shell layout with sidebar, content area, event listener, and disconnected state** - `9c3aff9` (feat)
3. **Task 3: Visual verification of app shell** - APPROVED by user (no code commit — checkpoint only)

## Files Created/Modified

- `src/lib/stores/device.ts` - Zustand store: connected, mountPoint, confirmed state + setConnected/reset actions
- `src/components/device-status-badge.tsx` - Color-coded dot + volume name, reads from useDeviceStore
- `src/components/sidebar-nav.tsx` - 4-section nav, Projects active, others disabled per UI-SPEC contracts
- `src/components/tauri-event-listener.tsx` - Tauri event bridge: dynamic import, ot-device-changed listener, cleanup
- `src/app/page.tsx` - App shell: aside sidebar + main content area, disconnected/connected states
- `src/app/layout.tsx` - Added TauriEventListener to layout providers

## Decisions Made

- TauriEventListener uses `await import("@tauri-apps/api/event")` inside an `async setupListeners()` wrapped in try/catch — prevents SSR crash when Next.js dev server runs in browser context (Tauri API only available inside webview).
- Sidebar nav uses custom `<button>` elements rather than the shadcn Sidebar component — gives direct control over 44px touch targets, active indicator, and disabled styling per UI-SPEC.md Disabled State Contract.
- Device status badge shows `mountPoint?.split("/").pop()` as connected label — shows volume name like "OT-CARD" rather than full path, minimal and unambiguous.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None — both auto tasks built and verified cleanly. `npm run build` passed. Visual checkpoint approved on first presentation.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- App shell is the permanent chrome for all future phases. Plan 07 (volume detection backend) can now wire up the Rust `ot-device-changed` event emitter — TauriEventListener is already listening.
- Phase 2 plans (02-03, 02-04, 02-05) build content into the main area; the sidebar structure and device store are ready.
- No blockers. The 220px sidebar, device store, and event listener are stable foundations.

---
*Phase: 01-foundation*
*Completed: 2026-04-30*
