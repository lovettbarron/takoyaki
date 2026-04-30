# Deferred UAT — Phase 1 (Foundation)

## Visual / Runtime Verification (deferred 2026-04-30)

1. **App launch verification** — Run `cargo tauri dev`, verify window opens with warm dark theme, sidebar, disconnected state
2. **Volume detection E2E** — Test with OT card: toast on connect, confirmation dialog, auto-navigate on confirm, disconnect toast
3. **Real OT file round-trip** — Add real .work, bank, markers, arr, .ot files to `tests/fixtures/` and run `cargo test -p ot-parser`
4. **FAT32 atomic write** — Verify AtomicWriteFile staging on a real FAT32/CF card volume

## Notes
- Phase 1 automated checks all pass (52 tests, cargo build, npm build)
- Wallflower read-only protection verified
- In-memory SQLite used in `run()` — acceptable for Phase 1, must switch to `default_path()` before Phase 2 project indexing

# Deferred UAT — Phase 2 (Read-Only Browser)

## Visual / Runtime Verification (deferred 2026-04-30)

1. **Project list view** — Connect OT card, verify project table renders with search/filter, click a project to navigate
2. **Project detail view** — Verify breadcrumb, metadata header, 4x4 bank grid drill-down, sample slot tables
3. **Health check display** — Verify health tab shows scanning → all-clear/issues, inline slot status icons
4. **Cross-reference expansion** — Expand sample slot rows, verify cross-reference data appears

## Notes
- All automated checks pass (cargo test + npm build)
- OT parser returns stub data (opaque blobs) — real structured data requires Phase 1 parser enrichment
- Health engine tested against real WAV fixtures (3 format tests pass)
- Security: parameterized SQL, canonicalize path traversal prevention verified
