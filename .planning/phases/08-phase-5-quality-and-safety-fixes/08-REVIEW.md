---
phase: 08-phase-5-quality-and-safety-fixes
reviewed: 2026-05-06T20:42:00Z
depth: standard
files_reviewed: 4
files_reviewed_list:
  - crates/takoyaki-app/src/commands/samples.rs
  - src/components/project-detail/SamplesTab.tsx
  - src/components/project-detail/SlotRow.tsx
  - src/lib/tauri.ts
findings:
  critical: 0
  warning: 3
  info: 2
  total: 5
status: issues_found
---

# Phase 8: Code Review Report

**Reviewed:** 2026-05-06T20:42:00Z
**Depth:** standard
**Files Reviewed:** 4
**Status:** issues_found

## Summary

Reviewed the Phase 8 changes across the Rust backend and React frontend. The changes implement three key safety features: (WR-02) format validation gate in `assign_sample`, (WR-03) atomic temp-then-rename copy for Wallflower files, and (WR-04) conflict detection with overwrite parameter and a frontend `ConflictPrompt` UI.

Overall the implementation is solid. The atomic copy pattern (stage to `.tmp`, rename) is correctly implemented with best-effort cleanup on failure. The conflict detection uses a sentinel string pattern ("CONFLICT:") in the error message, which works but is fragile. The `onDismiss` prop wiring in SlotRow fixes a previously dead Dismiss button. No critical security issues found.

Three warnings and two info items identified below.

## Warnings

### WR-01: DryRunModal onApply type mismatch with handleApplyAssign

**File:** `src/components/project-detail/SamplesTab.tsx:578`
**Issue:** `handleApplyAssign` has signature `(overwrite?: boolean) => Promise<void>`, but `DryRunModal` declares `onApply: () => void`. When the DryRunModal calls `onApply()` with no arguments, `overwrite` is `undefined`, which gets defaulted to `false` via `overwrite ?? false` -- so this works at runtime. However, the type signatures are mismatched: a function accepting an optional parameter is being passed where a zero-arg function is expected. TypeScript allows this assignment direction (function with fewer params is assignable to one with more), but the modal can never pass `overwrite=true`, meaning the conflict-then-retry flow works only because `handleConflictOverwrite` calls `handleApplyAssign(true)` directly, bypassing the modal. This is correct behavior but the type mismatch signals a potential future footgun -- if someone wires a second "overwrite" button inside the modal, the type would silently drop the argument.
**Fix:** No immediate code change required since runtime behavior is correct. For clarity, consider either:
- Keeping `onApply: () => void` in DryRunModal and wrapping the call: `onApply={() => handleApplyAssign()}`
- Or documenting that the modal never triggers overwrite (the conflict prompt handles it)

### WR-02: Conflict detection relies on string parsing of error messages

**File:** `src/components/project-detail/SamplesTab.tsx:383-389`
**Issue:** The conflict detection parses the error string for `"CONFLICT:"` prefix and uses a regex to extract the filename. This couples the frontend to the exact error message format in the Rust backend (`samples.rs:482-485`). If the error message wording changes (e.g., localization, rewording), the conflict prompt silently breaks and falls through to the generic error handler. This is a fragile contract between the backend and frontend.
**Fix:** Consider using a structured error variant instead of string matching. For example, add a `Conflict` variant to `AppError`:
```rust
// In error.rs
#[error("Conflict: {0}")]
Conflict(String),
```
Then in the frontend, match on a structured error code rather than message text. Alternatively, if string matching is the chosen pattern, add an integration test or constant that keeps both sides in sync:
```rust
// In samples.rs
const CONFLICT_PREFIX: &str = "CONFLICT:";
```
```typescript
// In SamplesTab.tsx or a shared constants file
const CONFLICT_PREFIX = "CONFLICT:";
```

### WR-03: assignStatus left as "assigning" during conflict prompt

**File:** `src/components/project-detail/SamplesTab.tsx:384-389`
**Issue:** When a CONFLICT error is caught, `setIsApplying(false)` is called but `setAssignStatus` is never updated. The `assignStatus` remains `"assigning"` (set on line 353) throughout the conflict prompt flow. The concurrent-assign guard on line 277-278 (`if (currentStatus !== "idle") return`) would block the user from initiating a *new* assignment to a different slot while the conflict prompt is visible, which may be intentional. However, `handleConflictOverwrite` calls `handleApplyAssign(true)` which also checks `pendingSlotType/pendingSlotIndex/pendingFilePath` (not `assignStatus`), so it works. The issue is that if the user dismisses the conflict via `handleConflictCancel`, `reset()` sets status back to `"idle"`. But if the component unmounts or re-renders during the conflict prompt, the stuck `"assigning"` status could cause the assign button to appear permanently disabled until manual `reset()`.
**Fix:** Set `assignStatus` to a specific state when showing the conflict prompt:
```typescript
if (errMsg.includes("CONFLICT:")) {
  setIsApplying(false);
  setAssignStatus("idle"); // or a new "conflict" status
  // ... rest of conflict handling
}
```

## Info

### IN-01: Stale .tmp files could accumulate on repeated failures

**File:** `crates/takoyaki-app/src/commands/samples.rs:490-498`
**Issue:** If `std::fs::copy` to the `.tmp` file succeeds but `std::fs::rename` fails, the cleanup on line 496 (`let _ = std::fs::remove_file(&temp_dest)`) is best-effort. If both rename and cleanup fail (e.g., filesystem permissions change, USB ejected mid-operation), a `.kick.wav.tmp` file remains in the AUDIO directory. Additionally, if a previous failed attempt left a `.tmp` file and a new attempt begins, the `std::fs::copy` on line 491 will silently overwrite the stale `.tmp`, which is actually the correct behavior. No action needed unless `.tmp` file accumulation becomes an observable issue.
**Fix:** Consider adding a startup or pre-operation sweep that removes any `.*tmp` files from the AUDIO directory, or log a warning when a stale `.tmp` is found.

### IN-02: Unused import in SlotRow (pre-existing)

**File:** `src/components/project-detail/SlotRow.tsx:1`
**Issue:** The `useState` import is used (line 108), so no unused import in the new code. However, the `Loader2` import from `lucide-react` (line 4) is only used when `isPlaying && playbackState === "loading"`. This is fine and functional -- noting for completeness that all imports in the changed files are actively used.
**Fix:** No action needed.

---

_Reviewed: 2026-05-06T20:42:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
