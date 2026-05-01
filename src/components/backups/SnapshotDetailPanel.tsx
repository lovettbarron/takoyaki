"use client";

import { useQuery } from "@tanstack/react-query";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import { Button } from "@/components/ui/button";
import type { BackupSummary, ChangeType } from "@/lib/types";
import { useBackupStore } from "@/lib/stores/backup";
import { useDeviceStore } from "@/lib/stores/device";
import { computeDryRun } from "@/lib/tauri";

interface SnapshotDetailPanelProps {
  backup: BackupSummary;
}

function formatFileSize(bytes: number): string {
  if (bytes >= 1_000_000) return `${Math.round(bytes / 1_000_000)} MB`;
  if (bytes >= 1_000) return `${Math.round(bytes / 1_000)} KB`;
  return `${bytes} B`;
}

function changeTypeColor(ct: ChangeType): string {
  switch (ct) {
    case "Added":
      return "text-[hsl(140,60%,42%)]";
    case "Modified":
      return "text-[hsl(38,85%,55%)]";
    case "Removed":
      return "text-[hsl(0,68%,48%)]";
    case "Unchanged":
      return "text-muted-foreground";
    case "Conflict":
      return "text-[hsl(280,60%,55%)]";
  }
}

export function SnapshotDetailPanel({ backup }: SnapshotDetailPanelProps) {
  const { connected } = useDeviceStore();
  const isConnected = connected;

  const { data: manifest, isPending } = useQuery({
    queryKey: ["restore-manifest", backup.id],
    queryFn: () => computeDryRun(backup.project_id, "restore", backup.id),
    enabled: isConnected,
  });

  async function handleRestoreClick() {
    const { startOperation, setDryRunManifest, reset } = useBackupStore.getState();
    startOperation(backup.project_id, backup.project_name, "restore", backup.id);
    try {
      const m = await computeDryRun(backup.project_id, "restore", backup.id);
      setDryRunManifest(m);
    } catch {
      reset();
    }
  }

  return (
    <div className="border-t border-border bg-secondary px-4 py-3">
      {/* Detail header */}
      <p className="font-mono text-lg font-semibold text-foreground pb-1">
        {backup.project_name} -- {backup.operation.replace(/-/g, " ")} -- {backup.created_at}
      </p>

      {/* Destination path */}
      <p className="font-mono text-xs text-muted-foreground text-ellipsis overflow-hidden whitespace-nowrap max-w-full pb-2">
        <span title={backup.dest_path}>{backup.dest_path}</span>
      </p>

      {isConnected ? (
        isPending ? (
          /* Loading skeleton */
          <div className="space-y-1 py-2">
            <Skeleton className="h-8 w-full" />
            <Skeleton className="h-8 w-full" />
            <Skeleton className="h-8 w-full" />
          </div>
        ) : manifest ? (
          <>
            {/* Change summary strip */}
            <div className="flex gap-4 py-2 border-b border-border">
              <span className="font-mono text-xs font-semibold text-[hsl(140,60%,42%)]">
                Added ({manifest.totalAdded})
              </span>
              <span className="font-mono text-xs font-semibold text-[hsl(38,85%,55%)]">
                Modified ({manifest.totalModified})
              </span>
              <span className="font-mono text-xs font-semibold text-[hsl(0,68%,48%)]">
                Removed ({manifest.totalRemoved})
              </span>
              <span className="font-mono text-xs font-semibold text-muted-foreground">
                Unchanged ({manifest.totalUnchanged})
              </span>
            </div>

            {/* File change list */}
            <ScrollArea className="max-h-60">
              <div className="divide-y divide-border">
                {manifest.entries.map((entry, i) => (
                  <div key={i} className="h-8 flex items-center px-4 gap-2">
                    <span
                      className={`font-mono text-xs font-semibold w-20 ${changeTypeColor(entry.changeType)}`}
                    >
                      {entry.changeType}
                    </span>
                    <span className="font-mono text-xs text-foreground flex-1 text-ellipsis overflow-hidden whitespace-nowrap">
                      {entry.path}
                    </span>
                    <span className="font-mono text-xs text-muted-foreground w-[72px] text-right tabular-nums">
                      {formatFileSize(entry.sizeBytes)}
                    </span>
                  </div>
                ))}
              </div>
            </ScrollArea>
          </>
        ) : null
      ) : (
        /* Disconnected summary */
        <div className="py-4">
          <p className="font-mono text-xs text-muted-foreground">
            {backup.file_count} files . {formatFileSize(backup.total_bytes)}
          </p>
          <p className="text-xs text-muted-foreground italic pt-1">
            Connect your Octatrack to see changes from current state.
          </p>
        </div>
      )}

      {/* Snapshot guarantee note */}
      <p className="text-xs text-muted-foreground italic py-2 border-t border-border">
        A snapshot of the current state will be created before restoring.
      </p>

      {/* Button row */}
      <div className="flex justify-end gap-2 pt-2">
        {isConnected ? (
          <Button
            variant="default"
            className="font-mono text-xs"
            onClick={handleRestoreClick}
          >
            Restore This Snapshot
          </Button>
        ) : (
          <>
            <Button variant="default" className="font-mono text-xs" disabled>
              Restore This Snapshot
            </Button>
            <span className="font-mono text-xs text-muted-foreground self-center">
              Connect your Octatrack to restore this snapshot.
            </span>
          </>
        )}
      </div>
    </div>
  );
}
