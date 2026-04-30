"use client";

import { SnapshotRow } from "./SnapshotRow";
import { SnapshotDetailPanel } from "./SnapshotDetailPanel";
import type { BackupSummary } from "@/lib/types";

interface BackupTimelineProps {
  projectName: string;
  backups: BackupSummary[];
  selectedId: string | null;
  onSelect: (id: string | null) => void;
  onRestore: (backup: BackupSummary) => void;
}

export function BackupTimeline({
  projectName,
  backups,
  selectedId,
  onSelect,
  onRestore,
}: BackupTimelineProps) {
  return (
    <div>
      {/* Project group header */}
      <div className="px-4 pt-6 pb-1 border-t border-border first:border-t-0">
        <p className="font-mono text-lg font-semibold text-foreground">
          {projectName}
        </p>
      </div>

      {/* Snapshot rows */}
      {backups.map((backup) => (
        <div key={backup.id}>
          <SnapshotRow
            backup={backup}
            selected={backup.id === selectedId}
            onSelect={onSelect}
            onRestore={onRestore}
          />
          {backup.id === selectedId && (
            <SnapshotDetailPanel backup={backup} />
          )}
        </div>
      ))}
    </div>
  );
}
