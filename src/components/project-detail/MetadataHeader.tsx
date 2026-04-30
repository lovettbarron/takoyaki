"use client";

import type { ProjectDetail } from "@/lib/types";

interface MetadataHeaderProps {
  project: ProjectDetail;
}

export function MetadataHeader({ project }: MetadataHeaderProps) {
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

  return (
    <div className="flex h-12 items-center justify-between border-b border-[hsl(30,8%,26%)] px-4 py-2">
      {/* Left: project name — Display typography */}
      <span className="font-mono text-2xl font-semibold text-foreground">
        {project.project_name}
      </span>

      {/* Right: metadata items separated by middle dot */}
      <div className="flex items-center gap-1 font-mono text-sm text-muted-foreground">
        <span>{tempoDisplay}</span>
        <span className="opacity-40">·</span>
        <span>{bankCountDisplay}</span>
        <span className="opacity-40">·</span>
        <span>Modified {modifiedDisplay}</span>
      </div>
    </div>
  );
}
