"use client";

import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { listBackups, computeDryRun } from "@/lib/tauri";
import { BackupTimeline } from "./BackupTimeline";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import { useBackupStore } from "@/lib/stores/backup";
import type { BackupSummary } from "@/lib/types";

export function BackupsView() {
  const [selectedSnapshotId, setSelectedSnapshotId] = useState<string | null>(null);

  const { data, isPending } = useQuery({
    queryKey: ["backups"],
    queryFn: () => listBackups(),
  });

  // Group backups by project_name, preserving DESC sort from API
  const projectGroups: Map<string, BackupSummary[]> = new Map();
  if (data) {
    for (const backup of data) {
      const existing = projectGroups.get(backup.project_name);
      if (existing) {
        existing.push(backup);
      } else {
        projectGroups.set(backup.project_name, [backup]);
      }
    }
  }

  async function handleRestoreShortcut(backup: BackupSummary) {
    const { startOperation, setDryRunManifest, reset } = useBackupStore.getState();
    startOperation(backup.project_id, backup.project_name, "restore", backup.id);
    try {
      const manifest = await computeDryRun(backup.project_id, "restore", backup.id);
      setDryRunManifest(manifest);
    } catch {
      reset();
    }
  }

  return (
    <div className="flex flex-col h-full">
      {/* Section heading */}
      <div className="px-4 pt-6 pb-2">
        <p className="font-mono text-xs font-semibold text-muted-foreground tracking-wider uppercase">
          BACKUPS
        </p>
      </div>
      <div className="border-t border-border" />

      <ScrollArea className="flex-1">
        {isPending ? (
          /* Loading skeletons */
          <div className="px-4 py-2 space-y-2">
            <Skeleton className="h-12 w-full" />
            <Skeleton className="h-12 w-full" />
            <Skeleton className="h-12 w-full" />
          </div>
        ) : projectGroups.size === 0 ? (
          /* Empty state */
          <div className="flex flex-col items-center justify-center h-64 gap-2">
            <p className="font-mono text-sm text-foreground">No backups yet</p>
            <p className="font-mono text-xs text-muted-foreground">
              Back up a project from the Projects view to see it here.
            </p>
          </div>
        ) : (
          /* Project groups */
          Array.from(projectGroups.entries()).map(([projectName, backups]) => (
            <BackupTimeline
              key={projectName}
              projectName={projectName}
              backups={backups}
              selectedId={selectedSnapshotId}
              onSelect={setSelectedSnapshotId}
              onRestore={handleRestoreShortcut}
            />
          ))
        )}
      </ScrollArea>
    </div>
  );
}
