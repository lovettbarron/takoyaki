"use client";

import { useEffect } from "react";
import { useDeviceStore } from "@/lib/stores/device";
import { getDeviceStatus, confirmDevice } from "@/lib/tauri";
import { toast } from "sonner";

export function TauriEventListener() {
  const { setConnected, setConfirmed, reset } = useDeviceStore();

  useEffect(() => {
    const cleanupFns: (() => void)[] = [];

    async function autoConfirm(mountPoint: string) {
      setConnected(true, mountPoint);
      try {
        await confirmDevice(mountPoint);
      } catch {
        // fallback: just set local state
      }
      setConfirmed(true);
      const volumeName = mountPoint.split("/").pop() ?? mountPoint;
      toast("Octatrack connected at " + volumeName + ".");
    }

    async function setupListeners() {
      try {
        const status = await getDeviceStatus();
        if (status.connected && status.mountPoint) {
          await autoConfirm(status.mountPoint);
        }

        const { listen } = await import("@tauri-apps/api/event");

        const unlistenDevice = await listen<string | null>(
          "ot-device-changed",
          (event) => {
            if (event.payload) {
              autoConfirm(event.payload);
            } else {
              reset();
              toast("Octatrack disconnected.");
            }
          }
        );
        cleanupFns.push(unlistenDevice);
      } catch {
        // Not running in Tauri context (SSR / dev in browser)
      }
    }

    setupListeners();
    return () => {
      for (const cleanup of cleanupFns) cleanup();
    };
  }, [setConnected, setConfirmed, reset]);

  return null;
}
