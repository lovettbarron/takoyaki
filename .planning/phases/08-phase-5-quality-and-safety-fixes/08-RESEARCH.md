# Phase 8: Phase 5 Quality & Safety Fixes — Research

**Researched:** 2026-05-06
**Domain:** Rust backend fix (atomic file copy, format validation) + React frontend fix (Dismiss button wiring, conflict prompt)
**Confidence:** HIGH — all four issues are precisely located in source code; fixes are mechanical with no new dependencies.

---

## Summary

Phase 8 closes four specific tech-debt items (WR-01 through WR-04) that were identified in the Phase 5 code review and preserved in the verification report. All four issues are in existing, working code — no new architecture is introduced. The fixes are surgical edits to two files:

- `crates/takoyaki-app/src/commands/samples.rs` (Rust — WR-02, WR-03, WR-04)
- `src/components/project-detail/SlotRow.tsx` (React — WR-01)

The codebase already has all the infrastructure needed: `health::read_audio_spec` + `health::check_format_compatibility` exist for WR-02; `atomic_write` / `atomic_write_batch` patterns exist for WR-03; the Zustand store already exposes `clearSlotError` that the Dismiss button just needs to call (WR-01); and the destination-exists check is already in place for WR-04, it just needs a conflict prompt instead of a silent skip.

**Primary recommendation:** Fix all four items in a single plan. The changes are co-located in two files, all 100 existing workspace tests pass as baseline, and new unit tests can be added inline alongside existing ones.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Dismiss button wires to store action | Browser / Client | — | Button click calls Zustand store action already available in parent — pure frontend state wiring |
| Audio format validation gate in assign_sample | API / Backend | — | Rust Tauri command; reuses existing `health::` functions already called in dry-run |
| Atomic file copy for Wallflower samples | API / Backend | — | Rust; replaces `std::fs::copy` with temp-then-rename using same-volume temp file |
| Conflict prompt for existing Wallflower destination | API / Backend + Browser/Client | — | Backend detects conflict and returns error/flag; frontend shows conflict prompt |

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SMPL-01 | User can assign a desktop audio file to a specific Flex or Static slot with all affected binary files updated atomically | WR-02 fix: `assign_sample` must independently validate format before writing; WR-03 fix: Wallflower copy must use atomic write |
| SMPL-03 | System validates Flex vs Static slot type correctness when assigning samples | WR-01 fix: Dismiss must clear validation error; WR-02 fix: format gate in `assign_sample` |
| INTG-01 | User can search Wallflower sample library from within Takoyaki | No direct fix needed — search already works; WR-04 affects push-to-slot reliability |
| INTG-02 | User can push Wallflower samples to OT slots | WR-03 + WR-04 fixes: atomic copy and conflict resolution for the push-to-slot path |
</phase_requirements>

---

## Precise Issue Diagnosis

### WR-01: SlotRow Dismiss Button Is a No-Op

**File:** `src/components/project-detail/SlotRow.tsx` lines 249–258

**Exact problem:** The Dismiss button inside the inline error block calls only `e.stopPropagation()`. There is a comment saying "Parent clears via clearSlotError — this button is visual-only; parent must wire dismiss."

**What the fix requires:**
1. Add an `onDismiss?: () => void` prop to `SlotRowProps` (alongside existing `assignErrorRedirect`).
2. Wire the Dismiss button `onClick` to call `onDismiss?.()`.
3. In `SlotSection` (in `SamplesTab.tsx`), pass `onDismissError` as the new `onDismiss` prop to `SlotRow`.

**Current wiring analysis (from reading SamplesTab.tsx):**
- `SamplesTab` passes `onDismissError={clearSlotError}` to `SlotSection`.
- `SlotSection` correctly passes `assignErrorRedirect` to `SlotRow`, which is either the redirect callback (when redirect exists) or `{ label: "Dismiss", onRedirect: onDismissError }` (when no redirect).
- The separate Dismiss button that appears alongside the redirect button is the broken one — it receives no callback at all.
- `clearSlotError` in the Zustand store correctly nulls out both `slotError` and `slotErrorRedirect`.

**Minimal prop addition needed:** `onDismiss?: () => void` on `SlotRowProps`. `SlotSection` already receives `onDismissError`, so the wire-through is: `SlotSection.onDismissError` → `SlotRow.onDismiss`.

**Verified:** `clearSlotError` exists at `samples.ts:92` — `set({ slotError: null, slotErrorRedirect: null })`. [VERIFIED: codebase grep]

---

### WR-02: assign_sample Does Not Independently Validate Audio Format

**File:** `crates/takoyaki-app/src/commands/samples.rs` lines 416–550

**Exact problem:** `assign_sample` performs slot type/index validation and path canonicalization but calls `health::read_audio_spec` and `health::check_format_compatibility` only in `compute_sample_dry_run` (lines 276–306), not in `assign_sample` itself.

**What the fix requires:**
Add a format gate immediately after `canonical_source` is resolved (before snapshot), using the same pattern already in `compute_sample_dry_run`:

```rust
// After canonicalize(), before snapshot:
match health::read_audio_spec(&canonical_source) {
    Ok(spec) => {
        let issues = health::check_format_compatibility(&spec);
        for issue in &issues {
            if matches!(issue, health::FormatIssue::UnsupportedFormat(_)) {
                return Err(AppError::Parse(
                    "Cannot assign non-audio file to OT slot".into()
                ));
            }
        }
    }
    Err(e) => return Err(AppError::Io(format!("Cannot read audio file: {}", e))),
}
```

**Scope note:** WR-02 blocks only `UnsupportedFormat` (non-WAV/AIFF). `WrongSampleRate` and `WrongBitDepth` are soft warnings in dry-run and should remain non-blocking in `assign_sample` — the user was warned during dry-run and chose to proceed.

**Functions available (verified):**
- `health::read_audio_spec(&Path) -> Result<AudioSpec, ...>` — reads header bytes only [VERIFIED: health/mod.rs]
- `health::check_format_compatibility(&AudioSpec) -> Vec<FormatIssue>` — returns format issues [VERIFIED: health/mod.rs]
- `health::FormatIssue::UnsupportedFormat(_)` — the match arm to block on [VERIFIED: health/mod.rs]
- `AppError::Parse(String)` and `AppError::Io(String)` — existing error variants [VERIFIED: codebase]

**Test needed:** Unit test verifying that calling `assign_sample` with a non-audio file returns `Err(AppError::Parse(...))`. This can use the `create_non_audio_fixture` helper already defined in the test module.

---

### WR-03: Non-Atomic Wallflower File Copy

**File:** `crates/takoyaki-app/src/commands/samples.rs` lines 449–471

**Exact problem:** Lines 465–469 use `std::fs::copy(&canonical_source, &dest)` directly. If USB disconnects mid-copy, a partial file is left at `dest`.

**What the fix requires:**
Replace `std::fs::copy` with the temp-then-rename pattern (copy to `.{filename}.tmp` in the same directory, then `std::fs::rename` to final destination). The temp file must be in the same directory as `dest` to ensure same-filesystem rename atomicity — consistent with how `atomic_write` works in `atomic/mod.rs`.

```rust
// Stage on same filesystem as destination
let temp_dest = audio_dir.join(format!(".{}.tmp", &filename));
std::fs::copy(&canonical_source, &temp_dest)
    .map_err(|e| AppError::Io(format!("Failed to stage Wallflower file: {}", e)))?;
std::fs::rename(&temp_dest, &dest)
    .map_err(|e| AppError::Io(format!("Failed to finalize Wallflower file: {}", e)))?;
```

**Why temp-then-rename is correct here (not `atomic_write_batch`):**
`atomic_write_batch` is designed for OT binary project files — it creates a temp file via `AtomicWriteFile` (which writes serialized content, not copies). For a file copy from the Mac filesystem to the OT card, `std::fs::copy`-to-temp + `std::fs::rename` is the correct pattern. Both source and temp/dest are on the same FAT32 volume (temp is in `audio_dir`), so the rename is atomic. [ASSUMED: FAT32 on macOS supports atomic same-directory rename; confirmed as expected behavior per Phase 1 design decision]

**Cleanup on failure:** If `copy` to temp succeeds but `rename` fails (edge case), the `.tmp` file is left on the card. The plan should include a best-effort cleanup: if rename fails, attempt to remove temp file before returning error. This keeps the card clean.

---

### WR-04: Silent Skip When Wallflower Destination File Exists

**File:** `crates/takoyaki-app/src/commands/samples.rs` lines 460–470

**Exact problem:** When `dest.exists()` is true, the code logs at `info!` level and skips the copy entirely, even if the source and destination have different content. The slot assignment proceeds with whatever file is already on the card.

**What the fix requires:**
The code review suggested a size-comparison heuristic. However, the Phase 8 success criterion says "the user sees a conflict prompt instead of a silent skip." This means:

1. When `dest.exists()`, the command should return an error (or a dedicated conflict signal) that the frontend can translate into a prompt.
2. The user then chooses: overwrite or cancel.

**Implementation approach:**
Return a new error variant or use a structured result. Given the existing `AppError` enum and the fact that `assign_sample` returns `Result<AssignSampleResult, AppError>`, the cleanest approach is:

Option A (simpler): Return `AppError::Io` with a message containing a distinguishable prefix (e.g., `"CONFLICT:"`) that the frontend detects. The frontend shows a confirmation dialog; if confirmed, re-calls `assign_sample` with a new `overwrite: bool` param.

Option B (cleaner): Add `overwrite: bool` parameter to `assign_sample`. When `false` and dest exists → return conflict error. When `true` → overwrite. Frontend gets the error, shows prompt, re-calls with `overwrite: true`.

**Recommendation: Option B (add `overwrite: bool` param).** This is explicit and avoids string-matching on error messages. The frontend already calls `assignSample(...)` via `tauri.ts`. The new param defaults to `false` for the normal flow; the conflict prompt passes `true` on confirm.

**Frontend changes needed:**
- `tauri.ts`: update `assignSample` wrapper to accept optional `overwrite?: boolean`.
- `SamplesTab.tsx`: add a conflict confirmation dialog (can use a simple `window.confirm` or a small React state modal — see discussion below).
- `handleApplyAssign`: catch the conflict error, show prompt, re-call with `overwrite: true` if confirmed.

**Conflict UI:** A modal or confirm dialog shown when `assign_sample` returns a conflict error. The user sees: "A file named `{filename}` already exists on the OT card. Overwrite it?" with Overwrite / Cancel buttons. This does NOT need a dry-run step — overwrite is a single file operation already wrapped in the existing snapshot + atomic batch.

---

## Standard Stack

No new libraries are needed. All required tools are already in `Cargo.toml` and installed:

| What | Available | Used For |
|------|-----------|---------|
| `health::read_audio_spec` | Yes (Phase 2) | WR-02 format gate |
| `health::check_format_compatibility` | Yes (Phase 2) | WR-02 format gate |
| `std::fs::copy` + `std::fs::rename` | Yes (stdlib) | WR-03 atomic copy |
| `AppError` variants | Yes (Phase 1) | WR-02/03/04 error returns |
| Zustand `clearSlotError` | Yes (Phase 5) | WR-01 Dismiss wiring |

[VERIFIED: codebase inspection]

---

## Architecture Patterns

### Pattern 1: Format Validation Gate in assign_sample (WR-02)

The same three-function pattern already used in `compute_sample_dry_run` (lines 276–306):

```rust
// Source: crates/takoyaki-app/src/commands/samples.rs (existing compute_sample_dry_run pattern)
match health::read_audio_spec(&canonical_source) {
    Ok(spec) => {
        for issue in health::check_format_compatibility(&spec) {
            if matches!(issue, health::FormatIssue::UnsupportedFormat(_)) {
                return Err(AppError::Parse("Cannot assign non-audio file to OT slot".into()));
            }
        }
    }
    Err(e) => return Err(AppError::Io(format!("Cannot read audio file: {}", e))),
}
```

Insertion point: after `canonical_source` is resolved at line 440, before the Wallflower copy block at line 447.

### Pattern 2: Atomic Copy via Temp-Then-Rename (WR-03)

Consistent with `atomic/mod.rs` philosophy (same-volume temp, then rename):

```rust
// Source: mirrors atomic/mod.rs temp-then-rename strategy
let temp_dest = audio_dir.join(format!(".{}.tmp", &filename));
std::fs::copy(&canonical_source, &temp_dest)
    .map_err(|e| AppError::Io(format!("Failed to stage Wallflower file: {}", e)))?;
std::fs::rename(&temp_dest, &dest)
    .map_err(|e| {
        // Best-effort cleanup of temp on rename failure
        let _ = std::fs::remove_file(&temp_dest);
        AppError::Io(format!("Failed to finalize Wallflower file: {}", e))
    })?;
```

### Pattern 3: Conflict Overwrite Param (WR-04)

Add `overwrite: bool` to `assign_sample` signature. Change the existing-file check:

```rust
// Current (broken):
if dest.exists() {
    info!("Wallflower file already exists at destination: {}", dest.display());
} else {
    // ... copy
}

// Fixed:
if dest.exists() && !overwrite {
    return Err(AppError::Io(format!(
        "CONFLICT: {} already exists on OT card",
        filename
    )));
}
// Proceed with atomic copy (overwrite or new file — same code path)
```

Frontend in `SamplesTab.tsx` catches the `"CONFLICT:"` prefix error from `handleApplyAssign` and shows a confirmation prompt. On confirm, re-calls `handleApplyAssign` with the overwrite flag threaded through to `assignSample`.

### Pattern 4: Dismiss Button Wiring (WR-01)

```tsx
// Source: SlotRow.tsx fix — add onDismiss prop
interface SlotRowProps {
  // ... existing props ...
  onDismiss?: () => void;  // ADD THIS
}

// In render:
<button
  type="button"
  className="font-mono text-xs text-muted-foreground whitespace-nowrap shrink-0"
  onClick={(e) => {
    e.stopPropagation();
    onDismiss?.();  // WAS: no-op
  }}
>
  Dismiss
</button>
```

In `SlotSection` (within `SamplesTab.tsx`), pass `onDismiss={onDismissError}` to `SlotRow`. `onDismissError` is already wired to `clearSlotError` in `SamplesTab`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Audio format detection | Custom byte-sniffing | `health::read_audio_spec` (already exists) | Already tested against WAV/AIFF headers; correct behavior verified in 3 unit tests |
| Atomic file write | Manual temp-file management | `atomic_write_batch` for project files; temp+rename for file copy | Established pattern with fsync; cross-filesystem rename issue already solved |
| State management | Local React state for slot error | Zustand `useSamplesStore` (already exists) | Error state, redirect state, and clear action are already in the store |

---

## Common Pitfalls

### Pitfall 1: WR-03 Temp File on Wrong Filesystem
**What goes wrong:** If the temp file is created in `/tmp` (Mac filesystem) and dest is on the OT card (FAT32), `std::fs::rename` fails with `EXDEV` (cross-device move).
**Why it happens:** `tempfile::TempDir::new()` creates in `/tmp` by default — a different filesystem from the CF card.
**How to avoid:** Create the temp file in `audio_dir` (same directory as dest), using `audio_dir.join(format!(".{}.tmp", &filename))`. Same-directory = same filesystem = atomic rename.
**Warning signs:** `EXDEV` error from `std::fs::rename` in logs.

### Pitfall 2: WR-04 Overwrite Flag Threading
**What goes wrong:** Adding `overwrite: bool` to `assign_sample` without updating the TypeScript IPC wrapper in `tauri.ts` causes a runtime type error or silent ignored param.
**Why it happens:** Tauri commands are called with `invoke("assign_sample", { ... })` — the param name must match the Rust function parameter name exactly.
**How to avoid:** Update `tauri.ts` `assignSample` wrapper signature to accept `overwrite?: boolean` (default `false`), and pass it in the `invoke` call object.

### Pitfall 3: WR-01 Double Dismiss Path
**What goes wrong:** SlotRow shows both a redirect button (via `assignErrorRedirect.onRedirect`) AND a Dismiss button. If Dismiss calls `onDismiss` which calls `clearSlotError`, but the redirect button is also wired to `clearSlotError` + redirect logic, both buttons work but the UX is confusing.
**Why it happens:** The existing `SlotSection` logic at `SamplesTab.tsx:131–138` uses `assignErrorRedirect` for either the redirect action or a standalone Dismiss action depending on whether a redirect exists. When a redirect exists, the `assignErrorRedirect.label` is the redirect label, not "Dismiss". The separate Dismiss button is additive. This is correct design — don't conflate them.
**How to avoid:** Add `onDismiss` as a separate prop, not via `assignErrorRedirect`. The redirect button keeps its existing wiring.

### Pitfall 4: WR-02 Over-Blocking
**What goes wrong:** Blocking on `WrongSampleRate` or `WrongBitDepth` in `assign_sample` — these are soft warnings, not hard blocks. If added to the gate in `assign_sample`, they would reject files the user explicitly approved during dry-run.
**Why it happens:** Copy-paste from the `compute_sample_dry_run` block which handles all three `FormatIssue` variants.
**How to avoid:** In `assign_sample`, match ONLY on `FormatIssue::UnsupportedFormat(_)` — return error for that only. Let `WrongSampleRate` and `WrongBitDepth` pass through silently (user accepted during dry-run).

---

## Code Examples

### Existing health check pattern (verified)

```rust
// Source: crates/takoyaki-app/src/commands/samples.rs lines 276-306
match health::read_audio_spec(&canonical_source) {
    Ok(spec) => {
        let issues = health::check_format_compatibility(&spec);
        for issue in &issues {
            match issue {
                health::FormatIssue::UnsupportedFormat(_) => {
                    hard_block = Some("Unsupported format: OT accepts WAV and AIFF only. Convert this file first.".into());
                }
                health::FormatIssue::WrongSampleRate(actual) => { /* soft warning */ }
                health::FormatIssue::WrongBitDepth(actual) => { /* soft warning */ }
            }
        }
    }
    Err(e) => { hard_block = Some(format!("Cannot read audio file: {}", e)); }
}
```

### Existing Dismiss + redirect pattern in SlotSection (verified)

```tsx
// Source: SamplesTab.tsx lines 131-138
assignErrorRedirect={
  isErrorSlot && slotErrorRedirect
    ? { label: slotErrorRedirect.label, onRedirect: onSlotRedirect }
    : isErrorSlot
    ? { label: "Dismiss", onRedirect: onDismissError }
    : null
}
```

Note: When `slotErrorRedirect` exists (slot-type mismatch), `assignErrorRedirect.label` is the redirect action (e.g., "Assign to Static #001") and `onRedirect` is `onSlotRedirect`. The separate Dismiss button in SlotRow (lines 249–258) is the additional dismiss button. When `slotErrorRedirect` is null (format error), `assignErrorRedirect` IS the Dismiss button — so format errors already dismiss correctly via this prop. The bug only affects the case where BOTH a redirect AND a Dismiss button are shown simultaneously.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) + TypeScript compilation check (`npx tsc --noEmit`) |
| Config file | `Cargo.toml` workspace |
| Quick run command | `cargo test -p takoyaki-app --lib -- commands::samples` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SMPL-01 / WR-02 | `assign_sample` rejects non-audio file before snapshot | unit | `cargo test -p takoyaki-app --lib -- commands::samples::tests::test_assign_sample_rejects_non_audio` | No — Wave 0 |
| SMPL-01 / WR-03 | Wallflower copy uses temp-then-rename (no partial file on failure) | unit | `cargo test -p takoyaki-app --lib -- commands::samples::tests::test_wallflower_copy_atomic` | No — Wave 0 |
| SMPL-01 / WR-04 | `assign_sample` with `overwrite=false` and existing dest returns conflict error | unit | `cargo test -p takoyaki-app --lib -- commands::samples::tests::test_wallflower_conflict_returns_error` | No — Wave 0 |
| SMPL-01 / WR-04 | `assign_sample` with `overwrite=true` and existing dest succeeds | unit | `cargo test -p takoyaki-app --lib -- commands::samples::tests::test_wallflower_overwrite_succeeds` | No — Wave 0 |
| SMPL-03 / WR-01 | SlotRow Dismiss prop wires to callback | manual (UI) | `cargo tauri dev` + trigger slot-type mismatch error + click Dismiss | No — visual |

### Sampling Rate
- **Per task commit:** `cargo test -p takoyaki-app --lib -- commands::samples`
- **Per wave merge:** `cargo test --workspace && npx tsc --noEmit`
- **Phase gate:** Full suite green + TypeScript clean before `/gsd-verify-work`

### Wave 0 Gaps
- `commands::samples::tests::test_assign_sample_rejects_non_audio` — covers WR-02 (new unit test in existing file)
- `commands::samples::tests::test_wallflower_copy_atomic` — covers WR-03
- `commands::samples::tests::test_wallflower_conflict_returns_error` — covers WR-04 (overwrite=false)
- `commands::samples::tests::test_wallflower_overwrite_succeeds` — covers WR-04 (overwrite=true)

Note: All tests go in the existing `#[cfg(test)]` block at the bottom of `samples.rs` — no new test files needed. The `create_non_audio_fixture` and `create_wav_fixture` helpers already exist in that test module.

---

## Environment Availability

Step 2.6: No new external dependencies identified. All fixes use existing stdlib and crate-level code.

---

## Runtime State Inventory

Step 2.5: Not applicable — this is a code-quality fix phase, not a rename/refactor/migration.

---

## Open Questions

1. **WR-04 conflict UI: modal vs confirm dialog**
   - What we know: Phase 5 already uses `DryRunModal` for the pre-assign confirmation flow. A conflict prompt needs to appear after `handleApplyAssign` catches the conflict error.
   - What's unclear: Should the conflict prompt be a new minimal modal component, or can we reuse `window.confirm` (synchronous, no React state needed)?
   - Recommendation: Use a simple React state flag (`conflictPending: boolean`) in `SamplesTab` local state + an inline confirmation UI or a small dialog. Avoid `window.confirm` (blocks event loop, inconsistent styling). A minimal inline confirmation block (similar to `AssignSuccessBanner` structure) is sufficient — it does not need to be a full modal.

2. **WR-04 TypeScript type update for assignSample**
   - What we know: `tauri.ts` exports `assignSample(...)`. Adding `overwrite: boolean` to the Rust signature requires updating the TS wrapper and possibly `types.ts` if a new result type is introduced.
   - What's unclear: Should `overwrite` be a positional param or named (passed as `invoke` object key)?
   - Recommendation: Named param in the `invoke` object (consistent with existing pattern where all params are passed as an object). Tauri automatically maps object keys to Rust function param names.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | FAT32 on macOS supports atomic same-directory rename (via `std::fs::rename`) for the temp-then-final pattern | WR-03 Pattern 2 | If wrong, WR-03 fix provides false safety. Mitigation: Phase 1 already validated same-filesystem atomic rename (decision logged in STATE.md), and audio dir temp lives in `audio_dir` on the same volume. Low risk. |
| A2 | Adding `overwrite: bool` to `assign_sample` Rust function is the right granularity for WR-04 | WR-04 Pattern 3 | If a different conflict resolution strategy is preferred (e.g., content hash comparison, auto-rename), the plan would need revision. Planner should confirm this approach. |

---

## Sources

### Primary (HIGH confidence)
- Codebase: `crates/takoyaki-app/src/commands/samples.rs` — direct source code inspection of all four issues
- Codebase: `src/components/project-detail/SlotRow.tsx` — confirmed Dismiss button no-op at lines 249–258
- Codebase: `src/components/project-detail/SamplesTab.tsx` — confirmed `onDismissError={clearSlotError}` wiring exists but not threaded to Dismiss button
- Codebase: `src/lib/stores/samples.ts` — confirmed `clearSlotError` action nulls both `slotError` and `slotErrorRedirect`
- Codebase: `crates/takoyaki-app/src/health/mod.rs` — confirmed `read_audio_spec`, `check_format_compatibility`, `FormatIssue` variants
- Codebase: `crates/takoyaki-app/src/atomic/mod.rs` — confirmed temp-then-rename strategy and same-filesystem requirement
- Phase 5 code review: `.planning/phases/05-sample-assignment-and-wallflower/05-REVIEW.md` — authoritative issue descriptions for WR-01 through WR-04

### Secondary (MEDIUM confidence)
- Phase 5 verification report: `.planning/phases/05-sample-assignment-and-wallflower/05-VERIFICATION.md` — confirms WR-01/02/03/04 are unresolved quality issues from Phase 5

---

## Metadata

**Confidence breakdown:**
- Issue diagnosis: HIGH — all four issues directly observed in source code
- Fix approach: HIGH (WR-01, WR-02, WR-03) — fixes are direct applications of existing patterns already in the codebase
- Fix approach: MEDIUM (WR-04) — conflict prompt requires a small UI addition; approach is reasonable but planner should confirm modal vs inline UI choice
- No new libraries needed: HIGH

**Research date:** 2026-05-06
**Valid until:** Stable — code does not change between research and planning in this project
