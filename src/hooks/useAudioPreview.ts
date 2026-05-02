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
