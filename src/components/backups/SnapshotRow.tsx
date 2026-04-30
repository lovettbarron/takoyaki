"use client";

import { Button } from "@/components/ui/button";
import type { BackupSummary } from "@/lib/types";

interface SnapshotRowProps {
  backup: BackupSummary;
  selected: boolean;
  onSelect: (id: string | null) => void;
  onRestore: (backup: BackupSummary) => void;
}

function formatFileSize(bytes: number): string {
  if (bytes >= 1_000_000) return `${Math.round(bytes / 1_000_000)} MB`;
  if (bytes >= 1_000) return `${Math.round(bytes / 1_000)} KB`;
  return `${bytes} B`;
}

export function SnapshotRow({ backup, selected, onSelect, onRestore }: SnapshotRowProps) {
  const operationLabel = backup.operation.replace(/-/g, " ");

  return (
    <div
      className={[
        "h-12 px-4 flex items-center cursor-pointer",
        selected
          ? "bg-secondary border-l-4 border-l-[hsl(38,85%,55%)]"
          : "hover:bg-[hsl(30,8%,20%)]",
      ].join(" ")}
      onClick={() => onSelect(selected ? null : backup.id)}
    >
      {/* Timestamp */}
      <span className="font-mono text-xs text-foreground tabular-nums w-[140px] shrink-0">
        {backup.created_at}
      </span>

      {/* Operation label */}
      <span className="font-mono text-xs font-semibold text-muted-foreground flex-1">
        {operationLabel}
      </span>

      {/* File count */}
      <span className="font-mono text-xs text-muted-foreground w-[72px] text-right tabular-nums">
        {backup.file_count} files
      </span>

      {/* Size */}
      <span className="font-mono text-xs text-muted-foreground w-[72px] text-right tabular-nums">
        {formatFileSize(backup.total_bytes)}
      </span>

      {/* Restore shortcut button */}
      <Button
        variant="ghost"
        size="sm"
        className="font-mono text-xs h-7 ml-2"
        onClick={(e) => {
          e.stopPropagation();
          onRestore(backup);
        }}
      >
        Restore
      </Button>
    </div>
  );
}
