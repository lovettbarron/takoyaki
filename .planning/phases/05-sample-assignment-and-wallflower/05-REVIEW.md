---
phase: 05-sample-assignment-and-wallflower
reviewed: 2026-05-02T12:00:00Z
depth: standard
files_reviewed: 20
files_reviewed_list:
  - crates/takoyaki-app/Cargo.toml
  - crates/takoyaki-app/src/commands/mod.rs
  - crates/takoyaki-app/src/commands/samples.rs
  - crates/takoyaki-app/src/commands/wallflower.rs
  - crates/takoyaki-app/src/db/mod.rs
  - crates/takoyaki-app/src/db/wallflower.rs
  - crates/takoyaki-app/src/lib.rs
  - migrations/V3__wallflower_settings.sql
  - package.json
  - src/app/page.tsx
  - src/components/backups/DryRunModal.tsx
  - src/components/project-detail/SamplesTab.tsx
  - src/components/project-detail/SlotPickerDialog.tsx
  - src/components/project-detail/SlotRow.tsx
  - src/components/project-detail/WallflowerPanel.tsx
  - src/components/project-detail/WallflowerSampleRow.tsx
  - src/components/settings/WallflowerSettings.tsx
  - src/components/sidebar-nav.tsx
  - src/lib/stores/samples.ts
  - src/lib/tauri.ts
  - src/lib/types.ts
findings:
  critical: 0
  warning: 4
  info: 3
  total: 7
status: issues_found
---

# Phase 5: Code Review Report

**Reviewed:** 2026-05-02T12:00:00Z
**Depth:** standard
**Files Reviewed:** 20
**Status:** issues_found

## Summary

Phase 5 implements sample assignment (desktop file picker and Wallflower push-to-slot flows) with dry-run preview, atomic writes, and Wallflower library integration. The code is well-structured with clear threat model annotations, proper read-only DB access for Wallflower, and consistent patterns from prior phases.

Key concerns:
- The SlotRow "Dismiss" button has a non-functional onClick handler (visual-only stub that never calls the parent dismiss callback)
- The `assign_sample` command does not re-validate the audio format before writing, relying solely on the dry-run having been called first (a user could bypass dry-run via direct IPC)
- The `assign_sample` Wallflower file copy uses `std::fs::copy` which is not atomic and could leave a partial file on crash

## Warnings

### WR-01: SlotRow "Dismiss" button has a non-functional onClick handler

**File:** `src/components/project-detail/SlotRow.tsx:244-250`
**Issue:** The inline "Dismiss" button's onClick handler only calls `e.stopPropagation()` and has a comment saying "Parent clears via clearSlotError -- this button is visual-only; parent must wire dismiss." However, the parent `SlotSection` only passes `assignErrorRedirect.onRedirect` for the redirect button -- the Dismiss button never actually calls `clearSlotError`. When no redirect is available (format errors per UI-SPEC), the only way to dismiss the error is via the redirect button labeled "Dismiss" passed through `assignErrorRedirect`, which is wired correctly. But when there IS a redirect option, both the redirect button and a separate "Dismiss" button are shown, and the latter is non-functional.
**Fix:** Wire the Dismiss button to call a passed-in `onDismissError` prop, or remove the non-functional button:
```tsx
// Option A: Pass an onDismissError prop to SlotRow and wire it
<button
  type="button"
  className="font-mono text-xs text-muted-foreground whitespace-nowrap shrink-0"
  onClick={(e) => {
    e.stopPropagation();
    onDismissError?.();
  }}
>
  Dismiss
</button>
```

### WR-02: assign_sample does not independently validate audio format before writing

**File:** `crates/takoyaki-app/src/commands/samples.rs:406-541`
**Issue:** The `assign_sample` command trusts that the caller ran `compute_sample_dry_run` first and respects the hard_block result. However, since Tauri commands are individually invocable from the frontend (or devtools), a caller could skip dry-run and directly invoke `assign_sample` with an incompatible file. While this is a desktop app with no network exposure (reducing risk), the project constraint states "No exceptions" for safety on destructive operations. The file could be a non-audio file that gets assigned to a slot, corrupting the project metadata.
**Fix:** Add a lightweight format check in `assign_sample` before the snapshot/write steps:
```rust
// After canonicalize, before snapshot:
match health::read_audio_spec(&canonical_source) {
    Ok(spec) => {
        let issues = health::check_format_compatibility(&spec);
        for issue in &issues {
            if matches!(issue, health::FormatIssue::UnsupportedFormat(_)) {
                return Err(AppError::Parse("Cannot assign non-audio file to OT slot".into()));
            }
        }
    }
    Err(e) => return Err(AppError::Io(format!("Cannot read audio file: {}", e))),
}
```

### WR-03: Non-atomic file copy for Wallflower samples to /AUDIO/

**File:** `crates/takoyaki-app/src/commands/samples.rs:457-459`
**Issue:** The Wallflower file copy to `card_root/AUDIO/` uses `std::fs::copy` directly. If the system crashes mid-copy (e.g., USB disconnect during write), a partial/corrupt file is left on the OT card. The project uses `atomic-write-file` for project.work writes and the atomic_write_batch mechanism, but this copy path does not benefit from atomic semantics. While the project.work rewrite happens after the copy (so the slot won't reference the file if the app crashes during copy), the partial file on the card is unclean.
**Fix:** Use copy-to-temp-then-rename pattern for the AUDIO copy:
```rust
let temp_dest = audio_dir.join(format!(".{}.tmp", &filename));
std::fs::copy(&canonical_source, &temp_dest)
    .map_err(|e| AppError::Io(format!("Failed to copy file to OT AUDIO: {}", e)))?;
std::fs::rename(&temp_dest, &dest)
    .map_err(|e| AppError::Io(format!("Failed to finalize file in OT AUDIO: {}", e)))?;
```

### WR-04: Wallflower file copy silently skips when destination file already exists

**File:** `crates/takoyaki-app/src/commands/samples.rs:451-455`
**Issue:** When `from_wallflower` is true and the destination file already exists, the copy is skipped with just an `info!` log. If the existing file at the destination path has different content than the Wallflower source (e.g., the user replaced a sample in Wallflower with the same filename), the OT slot will reference the stale file. The user gets no feedback that their file was not actually deployed.
**Fix:** Compare file sizes or hashes and either overwrite or warn the user:
```rust
if dest.exists() {
    let source_len = std::fs::metadata(&canonical_source)
        .map(|m| m.len())
        .unwrap_or(0);
    let dest_len = std::fs::metadata(&dest)
        .map(|m| m.len())
        .unwrap_or(0);
    if source_len != dest_len {
        // Files differ — overwrite
        std::fs::copy(&canonical_source, &dest).map_err(|e| {
            AppError::Io(format!("Failed to update file in OT AUDIO: {}", e))
        })?;
        info!("Updated existing Wallflower file at: {}", dest.display());
    } else {
        info!("Wallflower file already exists with same size: {}", dest.display());
    }
}
```

## Info

### IN-01: FIXME comment in get_project_samples — stub implementation

**File:** `crates/takoyaki-app/src/commands/samples.rs:148`
**Issue:** The `get_project_samples` command contains a FIXME comment noting the Phase 1 OT project.work parser is not yet implemented, returning 128 empty slots. This is expected as a stub, but the FIXME should be tracked.
**Fix:** Track in a future phase backlog or convert to a GitHub issue. The stub is intentional and correctly documented.

### IN-02: `sample_rate` field typed as `Option<u32>` in SampleSlot but `Option<number | null>` in TypeScript

**File:** `src/lib/types.ts:60` and `crates/takoyaki-app/src/commands/samples.rs:66`
**Issue:** The TypeScript type declares `sample_rate: number | null` which correctly maps to Rust's `Option<u32>`. No actual bug, but the `formatSampleRate` function in SlotRow accepts `number | null` and the SampleSlot type uses `number | null`. This is consistent. However, the SlotRow prop `slot.sample_rate` is typed as `number | null` but the function signature says `rate: number | null` -- this is fine. Just noting the manual type maintenance burden.
**Fix:** When tauri-specta generates `bindings.ts` in debug builds, swap to the auto-generated types to eliminate drift risk.

### IN-03: Dead code path -- build_sample_slot function is #[allow(dead_code)]

**File:** `crates/takoyaki-app/src/commands/samples.rs:193-206`
**Issue:** `build_sample_slot` is defined and annotated with `#[allow(dead_code)]` as scaffolding for when the Phase 1 parser becomes available. It is not currently called anywhere.
**Fix:** No action needed now -- this is intentional scaffolding per the comment. Remove the `#[allow(dead_code)]` when the parser is integrated.

---

_Reviewed: 2026-05-02T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
