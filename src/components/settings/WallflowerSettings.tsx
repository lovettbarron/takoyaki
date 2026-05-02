"use client";

import { useState, useEffect } from "react";
import { CircleCheck, CircleMinus } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import { getWallflowerStatus, setWallflowerDbPath } from "@/lib/tauri";
import { useSamplesStore } from "@/lib/stores/samples";

export function WallflowerSettings() {
  const { wallflowerConnected, wallflowerDbPath, setWallflowerConnected } = useSamplesStore();
  const [sampleCount, setSampleCount] = useState<number | null>(null);

  // Fetch current Wallflower connection status on mount
  useEffect(() => {
    getWallflowerStatus()
      .then((status) => {
        setWallflowerConnected(status.connected, status.db_path);
        setSampleCount(status.sample_count);
      })
      .catch(() => {
        setWallflowerConnected(false);
      });
  }, [setWallflowerConnected]);

  async function handleChangePath() {
    let selected: string | string[] | null = null;
    try {
      selected = await open({
        multiple: false,
        filters: [{ name: "SQLite Database", extensions: ["db"] }],
      });
    } catch {
      return;
    }
    if (!selected) return;
    const path = typeof selected === "string" ? selected : selected[0];
    try {
      const status = await setWallflowerDbPath(path);
      setWallflowerConnected(status.connected, status.db_path);
      setSampleCount(status.sample_count);
    } catch {
      // Path was invalid — keep current state
    }
  }

  return (
    <div className="space-y-2">
      <h3 className="font-mono text-xs font-semibold uppercase text-muted-foreground">
        WALLFLOWER
      </h3>

      {/* Connection status row — D-08 */}
      <div className="flex items-center gap-2 h-8">
        {wallflowerConnected ? (
          <CircleCheck size={14} className="text-[hsl(140,60%,42%)] shrink-0" />
        ) : (
          <CircleMinus size={14} className="text-muted-foreground shrink-0" />
        )}
        <span className="font-mono text-xs">
          {wallflowerConnected ? "Connected — wallflower.db" : "Not connected"}
        </span>
        {wallflowerConnected && sampleCount !== null && (
          <span className="font-mono text-xs text-muted-foreground ml-auto">
            {sampleCount} samples
          </span>
        )}
      </div>

      {/* DB path row (when connected) */}
      {wallflowerConnected && wallflowerDbPath && (
        <p
          className="font-mono text-xs text-muted-foreground truncate"
          title={wallflowerDbPath}
        >
          {wallflowerDbPath}
        </p>
      )}

      {/* Change path button */}
      <Button
        variant="ghost"
        size="sm"
        className="font-mono text-xs h-8"
        onClick={handleChangePath}
      >
        Change...
      </Button>
    </div>
  );
}
