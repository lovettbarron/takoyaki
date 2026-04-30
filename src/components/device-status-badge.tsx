"use client";

import { useDeviceStore } from "@/lib/stores/device";

export function DeviceStatusBadge() {
  const { connected, mountPoint } = useDeviceStore();

  return (
    <div className="flex items-center gap-2 px-3 py-1 rounded font-mono text-xs">
      <span
        className={`inline-block w-2 h-2 rounded-full ${
          connected ? "bg-[hsl(140_60%_42%)]" : "bg-[hsl(30_8%_38%)]"
        }`}
      />
      <span className={connected ? "text-foreground" : "text-muted-foreground"}>
        {connected ? mountPoint?.split("/").pop() || "Connected" : "No Device"}
      </span>
    </div>
  );
}
