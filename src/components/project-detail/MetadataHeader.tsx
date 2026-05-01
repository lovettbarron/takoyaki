"use client";

import { useState } from "react";
import type { ProjectDetail } from "@/lib/types";
import { Button } from "@/components/ui/button";
import { Archive, Pencil, Copy, PackageOpen } from "lucide-react";

interface MetadataHeaderProps {
  project: ProjectDetail;
  onBackUp?: () => void;
  onRename?: (newName: string) => void;
  onDuplicate?: () => void;
  onExport?: () => void;
}

export function MetadataHeader({ project, onBackUp, onRename, onDuplicate, onExport }: MetadataHeaderProps) {
  const [isRenaming, setIsRenaming] = useState(false);
  const [renameValue, setRenameValue] = useState(project.project_name);

  const tempoDisplay =
    project.tempo_bpm !== null
      ? `${project.tempo_bpm.toFixed(1)} BPM`
      : "-- BPM";

  const bankCountDisplay = `${project.bank_count ?? 0} banks`;

  let modifiedDisplay = "--";
  if (project.last_modified) {
    // Parse ISO date and format as YYYY-MM-DD
    try {
      const date = new Date(project.last_modified);
      modifiedDisplay = date.toISOString().split("T")[0];
    } catch {
      modifiedDisplay = project.last_modified;
    }
  }

  function handleRenameChange(e: React.ChangeEvent<HTMLInputElement>) {
    const upper = e.target.value.toUpperCase().replace(/[^A-Z0-9_]/g, "");
    if (upper.length <= 16) {
      setRenameValue(upper);
    }
  }

  function handleRenameKeyDown(e: React.KeyboardEvent<HTMLInputElement>) {
    if (e.key === "Enter" && renameValue.length > 0 && renameValue !== project.project_name) {
      setIsRenaming(false);
      onRename?.(renameValue);
    }
    if (e.key === "Escape") {
      setIsRenaming(false);
      setRenameValue(project.project_name);
    }
  }

  function handleRenameClick() {
    setRenameValue(project.project_name);
    setIsRenaming(true);
  }

  return (
    <div className="flex h-12 items-center justify-between border-b border-[hsl(30,8%,26%)] px-4 py-2">
      {/* Left: project name — Display typography or inline rename input */}
      {isRenaming ? (
        <input
          className="font-mono text-2xl font-semibold text-foreground bg-transparent border-b border-[hsl(38,85%,55%)] outline-none min-w-32 max-w-xs"
          value={renameValue}
          onChange={handleRenameChange}
          onKeyDown={handleRenameKeyDown}
          onBlur={() => { setIsRenaming(false); setRenameValue(project.project_name); }}
          maxLength={16}
          autoFocus
          placeholder="Project name..."
        />
      ) : (
        <span className="font-mono text-2xl font-semibold text-foreground">
          {project.project_name}
        </span>
      )}

      {/* Right: metadata items + management buttons + Back Up button */}
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-1 font-mono text-sm text-muted-foreground">
          <span>{tempoDisplay}</span>
          <span className="opacity-40">·</span>
          <span>{bankCountDisplay}</span>
          <span className="opacity-40">·</span>
          <span>Modified {modifiedDisplay}</span>
        </div>
        <Button
          variant="ghost"
          size="sm"
          className="font-mono text-xs h-8 gap-1"
          onClick={handleRenameClick}
          aria-label="Rename project"
        >
          <Pencil className="h-3.5 w-3.5" />
          Rename
        </Button>
        <Button
          variant="ghost"
          size="sm"
          className="font-mono text-xs h-8 gap-1"
          onClick={onDuplicate}
          aria-label="Duplicate project"
        >
          <Copy className="h-3.5 w-3.5" />
          Duplicate
        </Button>
        <Button
          variant="ghost"
          size="sm"
          className="font-mono text-xs h-8 gap-1"
          onClick={onExport}
          aria-label="Export project"
        >
          <PackageOpen className="h-3.5 w-3.5" />
          Export
        </Button>
        {onBackUp && (
          <Button
            variant="default"
            size="sm"
            className="font-mono text-xs h-8 gap-1"
            onClick={onBackUp}
          >
            <Archive className="h-3.5 w-3.5" />
            Back Up
          </Button>
        )}
      </div>
    </div>
  );
}
