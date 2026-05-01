"use client";

import { useState, useEffect } from "react";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { ConflictResolution } from "@/lib/types";

interface ConflictItem {
  filename: string;
  sourceHash: string;
  targetHash: string;
}

interface ConflictResolutionDialogProps {
  open: boolean;
  conflicts: ConflictItem[];
  onResolve: (resolutions: Record<string, ConflictResolution>) => void;
  onCancel: () => void;
}

const RESOLUTION_OPTIONS: { value: ConflictResolution; label: string }[] = [
  { value: "keep-target", label: "Keep Target" },
  { value: "use-source", label: "Use Source" },
  { value: "rename-incoming", label: "Rename Incoming" },
];

function ResolutionButton({
  value,
  label,
  selected,
  onClick,
}: {
  value: ConflictResolution;
  label: string;
  selected: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={[
        "font-mono text-xs px-2 py-1 rounded border transition-colors",
        selected
          ? "bg-[hsl(30,8%,20%)] border-[hsl(38,85%,55%)] text-[hsl(38,85%,55%)]"
          : "border-border hover:bg-[hsl(30,8%,16%)] text-muted-foreground hover:text-foreground",
      ].join(" ")}
    >
      {label}
    </button>
  );
}

export function ConflictResolutionDialog({
  open,
  conflicts,
  onResolve,
  onCancel,
}: ConflictResolutionDialogProps) {
  const [resolutions, setResolutions] = useState<Record<string, ConflictResolution | null>>({});

  // Reset local state when dialog opens
  useEffect(() => {
    if (open) {
      const initial: Record<string, ConflictResolution | null> = {};
      for (const c of conflicts) {
        initial[c.filename] = null;
      }
      setResolutions(initial);
    }
  }, [open, conflicts]);

  function setOne(filename: string, value: ConflictResolution) {
    setResolutions((prev) => ({ ...prev, [filename]: value }));
  }

  function setAll(value: ConflictResolution) {
    const next: Record<string, ConflictResolution | null> = {};
    for (const c of conflicts) {
      next[c.filename] = value;
    }
    setResolutions(next);
  }

  const allResolved =
    conflicts.length > 0 &&
    conflicts.every((c) => resolutions[c.filename] !== null && resolutions[c.filename] !== undefined);

  function handleApply() {
    if (!allResolved) return;
    const result: Record<string, ConflictResolution> = {};
    for (const c of conflicts) {
      result[c.filename] = resolutions[c.filename] as ConflictResolution;
    }
    onResolve(result);
  }

  return (
    <Dialog open={open} onOpenChange={(isOpen) => { if (!isOpen) onCancel(); }}>
      <DialogContent className="sm:max-w-[600px] p-6" showCloseButton={false}>
        <DialogHeader>
          <DialogTitle className="font-mono text-base font-semibold">
            Resolve Sample Conflicts
          </DialogTitle>
        </DialogHeader>

        <p className="font-mono text-xs text-muted-foreground">
          {conflicts.length} sample{conflicts.length !== 1 ? "s" : ""} exist in both projects with different content.
          Choose how to handle each conflict before copying.
        </p>

        {/* Bulk resolution row */}
        <div className="flex items-center gap-3 py-2 border-y border-border">
          <span className="font-mono text-xs text-muted-foreground">Apply to all:</span>
          <div className="flex gap-2">
            {RESOLUTION_OPTIONS.map((opt) => (
              <ResolutionButton
                key={opt.value}
                value={opt.value}
                label={opt.label}
                selected={conflicts.every((c) => resolutions[c.filename] === opt.value)}
                onClick={() => setAll(opt.value)}
              />
            ))}
          </div>
        </div>

        {/* Per-conflict rows */}
        <ScrollArea className="max-h-72">
          <div className="divide-y divide-border">
            {conflicts.map((conflict) => {
              const current = resolutions[conflict.filename] ?? null;
              return (
                <div key={conflict.filename} className="py-3 px-1 flex flex-col gap-2">
                  {/* Filename */}
                  <span className="font-mono text-xs text-[hsl(280,60%,55%)] truncate">
                    {conflict.filename}
                  </span>
                  {/* Hash snippets */}
                  <div className="flex gap-4 font-mono text-[10px] text-muted-foreground">
                    <span>src: {conflict.sourceHash.slice(0, 8)}</span>
                    <span>tgt: {conflict.targetHash.slice(0, 8)}</span>
                  </div>
                  {/* Resolution buttons */}
                  <div className="flex gap-2">
                    {RESOLUTION_OPTIONS.map((opt) => (
                      <ResolutionButton
                        key={opt.value}
                        value={opt.value}
                        label={opt.label}
                        selected={current === opt.value}
                        onClick={() => setOne(conflict.filename, opt.value)}
                      />
                    ))}
                  </div>
                </div>
              );
            })}
          </div>
        </ScrollArea>

        {/* Footer */}
        <DialogFooter className="flex justify-end gap-2 pt-4 border-t border-border">
          <Button variant="ghost" className="font-mono text-xs" onClick={onCancel}>
            Cancel
          </Button>
          <Button
            variant="default"
            className="font-mono text-xs"
            onClick={handleApply}
            disabled={!allResolved}
          >
            Apply with Resolutions
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
