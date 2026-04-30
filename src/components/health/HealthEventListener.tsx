"use client";

import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import type { HealthCheckComplete } from "@/lib/types";

export function HealthEventListener() {
  const queryClient = useQueryClient();

  useEffect(() => {
    const cleanupFns: (() => void)[] = [];

    async function setupListeners() {
      try {
        const { listen } = await import("@tauri-apps/api/event");

        const unlisten = await listen<HealthCheckComplete>(
          "health-complete",
          (event) => {
            const { project_id } = event.payload;
            queryClient.setQueryData(["health", project_id], event.payload);
          }
        );
        cleanupFns.push(unlisten);
      } catch {
        // Not in Tauri context (SSR, browser dev)
      }
    }

    setupListeners();
    return () => cleanupFns.forEach((fn) => fn());
  }, [queryClient]);

  return null;
}
