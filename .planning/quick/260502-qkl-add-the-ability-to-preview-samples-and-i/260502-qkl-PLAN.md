---
phase: quick
plan: 260502-qkl
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/takoyaki-app/src/commands/samples.rs
  - crates/takoyaki-app/src/lib.rs
  - src/lib/tauri.ts
  - src/hooks/useAudioPreview.ts
  - src/components/project-detail/SlotRow.tsx
autonomous: true
requirements: [QUICK-audio-preview]
must_haves:
  truths:
    - "User can click a play button on any occupied sample slot and hear the audio"
    - "User can stop playback by clicking the button again"
    - "Play button only appears for occupied slots with a valid file path"
    - "Audio plays from files on the mounted OT volume (resolved from relative OT paths)"
  artifacts:
    - path: "crates/takoyaki-app/src/commands/samples.rs"
      provides: "get_sample_audio_bytes Tauri command"
    - path: "src/hooks/useAudioPreview.ts"
      provides: "Audio playback hook with play/stop/state"
    - path: "src/components/project-detail/SlotRow.tsx"
      provides: "Play button integrated into slot row UI"
  key_links:
    - from: "src/components/project-detail/SlotRow.tsx"
      to: "src/hooks/useAudioPreview.ts"
      via: "useAudioPreview hook call"
    - from: "src/hooks/useAudioPreview.ts"
      to: "src/lib/tauri.ts"
      via: "getSampleAudioBytes IPC wrapper"
    - from: "src/lib/tauri.ts"
      to: "crates/takoyaki-app/src/commands/samples.rs"
      via: "invoke get_sample_audio_bytes"
---

<objective>
Add audio preview (playback) functionality to the sample browser. Users can click a play button on any occupied slot row to hear the referenced WAV/AIFF file from the mounted OT volume.

Purpose: Musicians need to quickly audition samples assigned to slots without leaving the app — essential for verifying sample assignments and project exploration.

Output: Working play/stop button in each SlotRow, backed by a Rust command that resolves and reads audio file bytes from the OT card.
</objective>

<execution_context>
@.planning/quick/260502-qkl-add-the-ability-to-preview-samples-and-i/260502-qkl-PLAN.md
</execution_context>

<context>
@CLAUDE.md
@crates/takoyaki-app/src/commands/samples.rs (existing sample commands, has resolve_ot_path import pattern, get_card_path usage)
@crates/takoyaki-app/src/health/mod.rs (resolve_ot_path function at line 212 — resolves OT relative paths to absolute paths with traversal prevention)
@crates/takoyaki-app/src/lib.rs (command registration via collect_commands![])
@src/lib/tauri.ts (IPC wrapper functions)
@src/lib/stores/device.ts (useDeviceStore with mountPoint)
@src/components/project-detail/SlotRow.tsx (target component for play button)
@src/components/project-detail/SamplesTab.tsx (parent that renders SlotRow)

<interfaces>
From crates/takoyaki-app/src/commands/samples.rs:
```rust
// Existing pattern for getting card_path:
let card_path = {
    let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db::projects::get_card_path(&db.conn, &project_id)
        .map_err(|e| AppError::Database(e.to_string()))?
};
let project_dir = std::path::PathBuf::from(&card_path);
```

From crates/takoyaki-app/src/health/mod.rs:
```rust
pub fn resolve_ot_path(volume_path: &Path, raw_path: &str) -> Option<PathBuf>;
```

From src/lib/types.ts:
```typescript
export interface SampleSlot {
  slot_index: number;
  occupied: boolean;
  filename: string | null;
  full_path: string | null;  // normalized OT path (e.g., "AUDIO/Alb/Field/sample.WAV")
  sample_rate: number | null;
  status: string;
}
```

From src/lib/stores/device.ts:
```typescript
export interface DeviceState {
  connected: boolean;
  mountPoint: string | null;  // e.g., "/Volumes/OCTATRACK"
  confirmed: boolean;
}
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add Rust get_sample_audio_bytes command</name>
  <files>crates/takoyaki-app/src/commands/samples.rs, crates/takoyaki-app/src/lib.rs</files>
  <action>
Add a new Tauri command `get_sample_audio_bytes` to `crates/takoyaki-app/src/commands/samples.rs`:

```rust
#[tauri::command]
#[specta::specta]
pub async fn get_sample_audio_bytes(
    state: tauri::State<'_, crate::AppState>,
    project_id: String,
    sample_path: String,
) -> Result<Vec<u8>, AppError> {
    // 1. Get card_path from DB (same pattern as get_project_samples)
    let card_path = {
        let db = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        db::projects::get_card_path(&db.conn, &project_id)
            .map_err(|e| AppError::Database(e.to_string()))?
    };

    // 2. Derive the volume root from card_path.
    //    card_path is like "/Volumes/OCTATRACK/SET1/Project01"
    //    The OT volume root is the mount point (2 levels up from project).
    //    sample_path (full_path from SampleSlot) is relative to volume root.
    let project_dir = PathBuf::from(&card_path);
    // Volume root: go up until we hit the mount point. OT structure is /Volumes/X/SET/Project
    // so volume root = project_dir.parent().parent() (set dir -> volume root)
    let volume_root = project_dir
        .parent() // SET directory
        .and_then(|p| p.parent()) // Volume root
        .ok_or_else(|| AppError::NotFound("Cannot determine volume root from project path".into()))?;

    // 3. Resolve the sample path safely using resolve_ot_path (traversal prevention)
    let resolved = health::resolve_ot_path(volume_root, &sample_path)
        .ok_or_else(|| AppError::NotFound(format!("Cannot resolve sample path: {}", sample_path)))?;

    // 4. Verify file exists
    if !resolved.exists() {
        return Err(AppError::NotFound(format!("Audio file not found: {}", resolved.display())));
    }

    // 5. Read file bytes (WAV/AIFF files are typically < 50MB for OT)
    let bytes = std::fs::read(&resolved)
        .map_err(|e| AppError::Io(format!("Failed to read audio file: {}", e)))?;

    info!("Serving {} bytes for sample: {}", bytes.len(), resolved.display());
    Ok(bytes)
}
```

Also add the command to `crates/takoyaki-app/src/lib.rs` in the `collect_commands![]` macro — add `commands::samples::get_sample_audio_bytes` after the existing `commands::samples::assign_sample` line.

Note: Check that `AppError` has an `Io` variant and a `NotFound` variant. If `NotFound` doesn't exist, use `AppError::Database` or add a new variant. Look at `crates/takoyaki-app/src/error.rs` for available variants and match the pattern.

The command takes `sample_path` which is the `full_path` field from `SampleSlot` (already normalized, e.g., "AUDIO/Alb/Field/sample.WAV"). This avoids exposing arbitrary file read capability — the path must resolve under the OT volume root.
  </action>
  <verify>
    <automated>cd /Users/andrewlovettbarron/src/takoyaki && cargo check -p takoyaki-app 2>&1 | tail -5</automated>
  </verify>
  <done>Rust compiles without errors. New command `get_sample_audio_bytes` registered and accessible via IPC.</done>
</task>

<task type="auto">
  <name>Task 2: Add frontend audio preview hook and integrate play button into SlotRow</name>
  <files>src/lib/tauri.ts, src/hooks/useAudioPreview.ts, src/components/project-detail/SlotRow.tsx</files>
  <action>
**Step 1: Add IPC wrapper to `src/lib/tauri.ts`:**

Add at the end of the file:
```typescript
export async function getSampleAudioBytes(
  projectId: string,
  samplePath: string
): Promise<number[]> {
  return invoke("get_sample_audio_bytes", { projectId, samplePath });
}
```

Note: Tauri serializes `Vec<u8>` as a JSON number array. The hook will convert this to a Uint8Array.

**Step 2: Create `src/hooks/useAudioPreview.ts`:**

```typescript
"use client";

import { useState, useRef, useCallback } from "react";
import { getSampleAudioBytes } from "@/lib/tauri";

export type PlaybackState = "idle" | "loading" | "playing";

export function useAudioPreview() {
  const [playbackState, setPlaybackState] = useState<PlaybackState>("idle");
  const [activeSlotKey, setActiveSlotKey] = useState<string | null>(null);
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const blobUrlRef = useRef<string | null>(null);

  const stop = useCallback(() => {
    if (audioRef.current) {
      audioRef.current.pause();
      audioRef.current.currentTime = 0;
      audioRef.current = null;
    }
    if (blobUrlRef.current) {
      URL.revokeObjectURL(blobUrlRef.current);
      blobUrlRef.current = null;
    }
    setPlaybackState("idle");
    setActiveSlotKey(null);
  }, []);

  const play = useCallback(
    async (projectId: string, samplePath: string, slotKey: string) => {
      // If same slot is playing, stop it (toggle behavior)
      if (activeSlotKey === slotKey && playbackState === "playing") {
        stop();
        return;
      }

      // Stop any current playback first
      stop();

      setPlaybackState("loading");
      setActiveSlotKey(slotKey);

      try {
        const bytes = await getSampleAudioBytes(projectId, samplePath);
        const uint8 = new Uint8Array(bytes);

        // Determine MIME type from file extension
        const ext = samplePath.split(".").pop()?.toLowerCase();
        const mime =
          ext === "aif" || ext === "aiff" ? "audio/aiff" : "audio/wav";

        const blob = new Blob([uint8], { type: mime });
        const url = URL.createObjectURL(blob);
        blobUrlRef.current = url;

        const audio = new Audio(url);
        audioRef.current = audio;

        audio.onended = () => {
          stop();
        };

        audio.onerror = () => {
          stop();
        };

        await audio.play();
        setPlaybackState("playing");
      } catch {
        stop();
      }
    },
    [activeSlotKey, playbackState, stop]
  );

  return { play, stop, playbackState, activeSlotKey };
}
```

**Step 3: Update `src/components/project-detail/SlotRow.tsx`:**

Add a play button between the status icon column and the assign button column. The button shows a Play icon (triangle) when idle/not-this-slot, a spinner/loader when loading, and a Stop icon (square) when playing.

Changes needed:
1. Add `Play`, `Square`, `Loader2` to the lucide-react import (alongside existing icons)
2. Add new props to `SlotRowProps`:
   ```typescript
   onPlay?: (slotIndex: number, slotType: "flex" | "static") => void;
   playbackState?: "idle" | "loading" | "playing";
   isPlaying?: boolean; // true if THIS specific slot is the active one
   ```
3. Add a play button column (w-8) BEFORE the assign button column in the CollapsibleTrigger flex row:
   ```tsx
   {/* Play button — w-8 */}
   <span className="w-8 shrink-0 flex items-center justify-center">
     {slot.occupied && slot.full_path && (
       <button
         type="button"
         onClick={(e) => {
           e.stopPropagation();
           onPlay?.(slot.slot_index, slotType);
         }}
         className="h-8 w-8 flex items-center justify-center rounded hover:bg-[hsl(30,8%,20%)] text-muted-foreground hover:text-foreground"
         aria-label={
           isPlaying
             ? `Stop preview of ${slotType} slot ${slot.slot_index + 1}`
             : `Preview ${slotType} slot ${slot.slot_index + 1}`
         }
       >
         {isPlaying && playbackState === "loading" ? (
           <Loader2 size={14} className="animate-spin" />
         ) : isPlaying && playbackState === "playing" ? (
           <Square size={12} className="fill-current" />
         ) : (
           <Play size={14} className="fill-current" />
         )}
       </button>
     )}
   </span>
   ```
4. Also add `w-8` to `SlotTableHeader` in `SamplesTab.tsx` to keep columns aligned — add an empty `<span className="w-8 shrink-0" />` before the existing trailing assign column span.

**Step 4: Wire the hook in `src/components/project-detail/SamplesTab.tsx`:**

1. Import `useAudioPreview` from `@/hooks/useAudioPreview`
2. Call the hook at the top of `SamplesTab`:
   ```typescript
   const { play, stop, playbackState, activeSlotKey } = useAudioPreview();
   ```
3. Create a handler:
   ```typescript
   function handlePlay(slotIndex: number, slotType: "flex" | "static") {
     const slotKey = `${slotType}-${slotIndex}`;
     const slots = slotType === "flex" ? flexSlots : staticSlots;
     const slot = slots[slotIndex];
     if (!slot?.full_path) return;
     play(projectId, slot.full_path, slotKey);
   }
   ```
4. Pass `onPlay={handlePlay}` and derive `isPlaying` and `playbackState` in `SlotSection` → `SlotRow`:
   - Add `onPlay`, `activeSlotKey`, `playbackState` props to `SlotSectionProps`
   - In SlotSection, pass to each SlotRow:
     ```tsx
     onPlay={onPlay}
     playbackState={playbackState}
     isPlaying={activeSlotKey === `${slotType}-${slot.slot_index}`}
     ```
5. Stop playback when navigating away: add cleanup in a useEffect that calls `stop()` on unmount.
  </action>
  <verify>
    <automated>cd /Users/andrewlovettbarron/src/takoyaki && npx next build 2>&1 | tail -10</automated>
  </verify>
  <done>Play button visible on occupied slot rows. Clicking play fetches audio bytes from backend and plays via Web Audio. Clicking again (or playback ending) stops. Loading state shows spinner. Build passes without errors.</done>
</task>

</tasks>

<verification>
1. `cargo check -p takoyaki-app` — Rust compiles
2. `npx next build` — Frontend builds without type errors
3. Manual test: With OT card mounted, navigate to a project with samples, click play on an occupied slot — audio plays through speakers
4. Manual test: Click play again while playing — audio stops
5. Manual test: Click play on a slot with missing file — no crash, returns to idle (error handled gracefully)
</verification>

<success_criteria>
- Occupied sample slots show a play/stop button in the SlotRow
- Clicking play fetches audio data from the OT volume via Rust backend and plays it
- Playback can be toggled (play/stop) with the same button
- Only one sample plays at a time (starting a new one stops the previous)
- Empty slots do not show the play button
- Missing files are handled gracefully (no crash, button returns to idle)
- Both WAV and AIFF formats are supported (correct MIME type detection)
</success_criteria>

<output>
After completion, create `.planning/quick/260502-qkl-add-the-ability-to-preview-samples-and-i/260502-qkl-SUMMARY.md`
</output>
