"use client";

import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { FileChangeManifest, FileChangeEntry, ChangeType } from "@/lib/types";

interface DryRunModalProps {
  open: boolean;
  manifest: FileChangeManifest | null;
  onApply: () => void;
  onCancel: () => void;
}

function formatFileSize(bytes: number): string {
  if (bytes >= 1_000_000) {
    return `${Math.round(bytes / 1_000_000)} MB`;
  }
  if (bytes >= 1_000) {
    return `${Math.round(bytes / 1_000)} KB`;
  }
  return `${bytes} B`;
}

function changeTypeColor(type: ChangeType): string {
  switch (type) {
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

function FileChangeRow({ entry }: { entry: FileChangeEntry }) {
  const colorClass = changeTypeColor(entry.changeType);
  return (
    <div className="h-8 flex items-center px-4 gap-2">
      <span className={`font-mono text-xs font-semibold w-20 ${colorClass}`}>
        {entry.changeType}
      </span>
      <span className="font-mono text-xs text-foreground flex-1 text-ellipsis overflow-hidden whitespace-nowrap">
        {entry.path}
      </span>
      <span className="font-mono text-xs text-muted-foreground w-[72px] text-right tabular-nums">
        {formatFileSize(entry.sizeBytes)}
      </span>
    </div>
  );
}

export function DryRunModal({ open, manifest, onApply, onCancel }: DryRunModalProps) {
  if (!manifest) return null;

  const isBackup = manifest.operationLabel.startsWith("Back Up");
  const applyCount = manifest.totalAdded + manifest.totalModified;

  return (
    <Dialog open={open} onOpenChange={(isOpen) => { if (!isOpen) onCancel(); }}>
      <DialogContent className="sm:max-w-[560px] p-6" showCloseButton={false}>
        <DialogHeader>
          <DialogTitle className="font-mono text-base font-semibold">
            {manifest.operationLabel}
          </DialogTitle>
        </DialogHeader>

        {/* Operation summary */}
        <div className="text-sm leading-relaxed" style={{ fontFamily: "var(--font-sans)" }}>
          {isBackup
            ? `${manifest.totalAdded + manifest.totalModified + manifest.totalUnchanged + manifest.totalRemoved} files will be copied to:`
            : `${manifest.totalAdded + manifest.totalModified + manifest.totalUnchanged + manifest.totalRemoved} files will be restored from:`}
        </div>

        {/* Destination path */}
        <div className="font-mono text-xs text-muted-foreground text-ellipsis overflow-hidden whitespace-nowrap max-w-[480px]">
          {manifest.destinationPath}
        </div>

        {/* Snapshot guarantee line (D-10 — exact text verbatim) */}
        <div className="text-xs text-muted-foreground italic pb-2 border-b border-border">
          A snapshot of the current state will be created before applying.
        </div>

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
        <ScrollArea className="max-h-72">
          <div className="divide-y divide-border">
            {manifest.entries.map((entry, i) => (
              <FileChangeRow key={`${entry.path}-${i}`} entry={entry} />
            ))}
          </div>
        </ScrollArea>

        {/* Footer */}
        <DialogFooter className="flex justify-end gap-2 pt-4 border-t border-border bg-transparent border-none -mx-0 -mb-0 rounded-none p-0">
          <Button
            variant="ghost"
            className="font-mono text-xs"
            onClick={onCancel}
          >
            Don&apos;t Apply
          </Button>
          {isBackup ? (
            <Button
              variant="default"
              className="font-mono text-xs"
              onClick={onApply}
            >
              Back Up {applyCount} files
            </Button>
          ) : (
            <Button
              variant="destructive"
              className="font-mono text-xs"
              onClick={onApply}
            >
              Restore Snapshot
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
