---
phase: 01-foundation
plan: "02"
subsystem: frontend
tags: [next.js, tailwind, shadcn, iosevka, design-system, react-query]
dependency_graph:
  requires: []
  provides: [frontend-build-pipeline, design-system-tokens, shadcn-components, react-query-provider]
  affects: [all-future-frontend-plans]
tech_stack:
  added:
    - next.js 16.2.4 (static export mode)
    - react 19.2.5
    - tailwind 4.2.x (CSS variable @theme inline pattern)
    - shadcn base-nova style (button, dialog, sidebar, sonner, separator, skeleton, badge)
    - "@base-ui/react (shadcn base-nova peer dependency)"
    - "@fontsource/iosevka (weights 400, 500, 600)"
    - "@tanstack/react-query 5.x"
    - zustand 5.x
    - lucide-react
  patterns:
    - Tailwind v4 @theme inline CSS variable mapping (no tailwind.config.js)
    - shadcn components with base-nova style + cssVariables
    - React Query provider wrapping via Providers component
    - Static export (output='export') for Tauri webview
key_files:
  created:
    - package.json
    - package-lock.json
    - next.config.mjs
    - tsconfig.json
    - postcss.config.mjs
    - components.json
    - next-env.d.ts
    - src/lib/utils.ts
    - src/components/providers.tsx
    - src/components/ui/button.tsx
    - src/components/ui/dialog.tsx
    - src/components/ui/sidebar.tsx
    - src/components/ui/sonner.tsx
    - src/components/ui/separator.tsx
    - src/components/ui/skeleton.tsx
    - src/components/ui/badge.tsx
    - src/components/ui/input.tsx
    - src/components/ui/tooltip.tsx
    - src/components/ui/sheet.tsx
    - src/hooks/use-mobile.ts
    - src/app/globals.css
    - src/app/layout.tsx
    - src/app/page.tsx
  modified: []
decisions:
  - "@base-ui/react peer dependency required by shadcn base-nova style — installed as explicit dependency"
  - "next.config.mjs uses ESM-compatible import.meta.url for turbopack.root (not __dirname)"
  - "turbopack.root set to project directory to silence Next.js workspace lockfile warning"
  - "shadcn installs next-themes as a peer; left in package.json as it may be needed for future theme work"
metrics:
  duration: "4 min"
  completed: "2026-04-30"
  tasks_completed: 2
  tasks_total: 2
  files_created: 23
  files_modified: 0
---

# Phase 01 Plan 02: Frontend Foundation (Next.js + Tailwind v4 + shadcn) Summary

**One-liner:** Next.js 16 static export with Tailwind v4 CSS variable theming, shadcn base-nova components, Iosevka monospace font, and warm dark palette matching the Elektron/monome visual identity.

## What Was Built

The complete frontend build pipeline and design system for Takoyaki. Every future UI plan builds on these tokens and components.

**Build pipeline:** Next.js 16.2 configured for static export (`output: 'export'`) targeting the Tauri webview. TypeScript strict mode. Tailwind v4 via `@tailwindcss/postcss`. `npm run build` produces `out/` in ~4 seconds.

**Design system:** Tailwind v4 `@theme inline` block maps all shadcn CSS variable names to Tailwind color utilities — no `tailwind.config.js` needed. The warm dark palette is defined in `:root` with HSL values:
- Background: `hsl(30 8% 10%)` — warm dark base (#1c1a18)
- Foreground: `hsl(30 10% 88%)` — off-white (#e3ddd6)
- Accent: `hsl(38 85% 55%)` — amber (#f0a832)
- All shadcn sidebar variables defined for sidebar component compatibility

**Typography:** Iosevka loaded via `@fontsource/iosevka` (weights 400, 500, 600). Set as `font-family: var(--font-mono)` on `body` — monospace-forward per D-08.

**Components installed:** button, dialog, sidebar, sonner, separator, skeleton, badge (plus input, tooltip, sheet as transitive dependencies of sidebar).

**React Query provider:** `src/components/providers.tsx` wraps children with `QueryClientProvider` (retry: 1, no window focus refetch). Wired in `layout.tsx` wrapping all page content.

**Layout:** Root layout imports Iosevka CSS, wraps in Providers and Toaster (bottom-right position). Desktop UX constraints applied: `overflow: hidden`, `user-select: none`, `overscroll-behavior: none` on html/body.

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| Task 1 | `6df4d1f` | Initialize Next.js frontend with dependencies and shadcn |
| Task 2 | `3875763` | Create globals.css with warm dark palette and root layout with Iosevka |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Missing @base-ui/react peer dependency**
- **Found during:** Task 2 (first `npm run build`)
- **Issue:** shadcn `base-nova` style components import from `@base-ui/react/merge-props` and `@base-ui/react/use-render`. The package was not pulled in by `npm install` because it was not listed in `package.json`. Build failed with TypeScript error: "Cannot find module '@base-ui/react/merge-props'".
- **Fix:** Ran `npm install @base-ui/react` to install the peer dependency explicitly.
- **Files modified:** `package.json`, `package-lock.json`
- **Commit:** Included in `3875763`

**2. [Rule 3 - Blocking] next.config.mjs turbopack.root used __dirname in ESM module**
- **Found during:** Task 2 (turbopack workspace warning suppression)
- **Issue:** Next.js warned about multiple lockfiles and suggested setting `turbopack.root`. Used `__dirname` which is not defined in ES module scope (`.mjs` files are ESM).
- **Fix:** Used `import.meta.url` with `fileURLToPath` + `dirname` to derive `__dirname` in ESM context.
- **Files modified:** `next.config.mjs`
- **Commit:** `3875763`

**3. [Rule 2 - Auto-added] next-themes package added by shadcn**
- **Found during:** Task 1 (shadcn install)
- **Issue:** `shadcn@latest add sonner` automatically added `next-themes` to `package.json`. This is not a breaking change — next-themes is listed as a supporting library in RESEARCH.md and may be needed in future plans.
- **Fix:** Left in place; not removed.
- **Files modified:** `package.json`

## Known Stubs

**src/app/page.tsx** — Minimal placeholder showing only "Takoyaki" text. This is intentional per the plan: "This placeholder will be replaced by Plan 06 with the full sidebar + disconnected state layout." Not a blocker for plan completion.

## Threat Flags

None. This plan creates a static frontend build pipeline only. No new network endpoints, auth paths, or trust boundaries introduced. Consistent with T-01-02a and T-01-02b in the plan threat model.

## Self-Check: PASSED

All files verified present on disk. Both task commits (6df4d1f, 3875763) confirmed in git log. `npm run build` produces `out/` directory with static export. All acceptance criteria met.
