"use client";

import { useState, useRef, useCallback, useEffect } from "react";
import { playSample, stopSample } from "@/lib/tauri";

export type PlaybackState = "idle" | "loading" | "playing" | "error";

export function useAudioPreview() {
  const [playbackState, setPlaybackState] = useState<PlaybackState>("idle");
  const [activeSlotKey, setActiveSlotKey] = useState<string | null>(null);
  const [lastError, setLastError] = useState<string | null>(null);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const stop = useCallback(async () => {
    if (timerRef.current) {
      clearInterval(timerRef.current);
      timerRef.current = null;
    }
    try {
      await stopSample();
    } catch {
      // best-effort
    }
    setPlaybackState("idle");
    setActiveSlotKey(null);
  }, []);

  useEffect(() => {
    return () => { stopSample().catch(() => {}); };
  }, []);

  const play = useCallback(
    async (projectId: string, samplePath: string, slotKey: string) => {
      if (activeSlotKey === slotKey && playbackState === "playing") {
        await stop();
        return;
      }

      await stop();

      setPlaybackState("loading");
      setActiveSlotKey(slotKey);
      setLastError(null);

      try {
        await playSample(projectId, samplePath);
        setPlaybackState("playing");
      } catch (err) {
        const msg = typeof err === "object" && err !== null
          ? (err as Record<string, unknown>).Io ?? (err as Record<string, unknown>).Device ?? String(err)
          : String(err);
        console.error("[audio-preview] Playback failed:", msg);
        setLastError(String(msg));
        setPlaybackState("error");
        setActiveSlotKey(null);
      }
    },
    [activeSlotKey, playbackState, stop]
  );

  return { play, stop, playbackState, activeSlotKey, lastError };
}
