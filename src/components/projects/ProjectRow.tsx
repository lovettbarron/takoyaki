"use client";

import { TableRow, TableCell } from "@/components/ui/table";
import { useNavigationStore } from "@/lib/stores/navigation";
import type { ProjectSummary } from "@/lib/types";

interface ProjectRowProps {
  project: ProjectSummary;
}

function formatModifiedDate(dateStr: string | null): string {
  if (!dateStr) return "--";
  // Accept ISO date strings; return just the date portion
  return dateStr.slice(0, 10);
}

export function ProjectRow({ project }: ProjectRowProps) {
  const { navigateToProject } = useNavigationStore();

  const handleSelect = () => {
    navigateToProject(project.id);
  };

  return (
    <TableRow
      className="cursor-pointer h-9 hover:bg-[hsl(30,8%,20%)] transition-colors"
      onClick={handleSelect}
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          handleSelect();
        }
      }}
      aria-selected={false}
    >
      {/* NAME */}
      <TableCell className="min-w-[120px] py-0">
        <span className="block truncate font-mono text-sm text-foreground">
          {project.project_name}
        </span>
      </TableCell>

      {/* BPM */}
      <TableCell className="w-20 py-0">
        <span className="font-mono text-sm tabular-nums text-muted-foreground">
          {project.tempo_bpm !== null ? project.tempo_bpm.toFixed(1) : "--"}
        </span>
      </TableCell>

      {/* BANKS */}
      <TableCell className="w-20 py-0">
        <span className="font-mono text-sm tabular-nums text-muted-foreground">
          {project.bank_count ?? 0}/16
        </span>
      </TableCell>

      {/* MODIFIED */}
      <TableCell className="w-[140px] py-0">
        <span className="text-sm text-muted-foreground tabular-nums">
          {formatModifiedDate(project.last_modified)}
        </span>
      </TableCell>
    </TableRow>
  );
}
