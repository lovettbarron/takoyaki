"use client";

import { useEffect } from "react";
import { useDeviceStore } from "@/lib/stores/device";

export function TauriEventListener() {
  const { setConnected, reset } = useDeviceStore();

  useEffect(() => {
    const cleanupFns: (() => void)[] = [];

    async function setupListeners() {
      try {
        const { listen } = await import("@tauri-apps/api/event");

        const unlistenDevice = await listen<string | null>(
          "ot-device-changed",
          (event) => {
            if (event.payload) {
              setConnected(true, event.payload);
            } else {
              reset();
            }
          }
        );
        cleanupFns.push(unlistenDevice);
      } catch {
        // Not running in Tauri context (SSR / dev in browser)
        // This is expected during next dev — Tauri API is only available
        // when running inside the Tauri webview
      }
    }

    setupListeners();
    return () => {
      for (const cleanup of cleanupFns) cleanup();
    };
  }, [setConnected, reset]);

  return null;
}
