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
