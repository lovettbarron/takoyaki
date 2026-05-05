"use client";

import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { ChevronUp, ChevronDown } from "lucide-react";
import {
  Table,
  TableHeader,
  TableBody,
  TableHead,
  TableRow,
} from "@/components/ui/table";
import { Skeleton } from "@/components/ui/skeleton";
import { ProjectRow } from "./ProjectRow";
import { ProjectSearchBar } from "./ProjectSearchBar";
import { indexOtProjects, listProjects } from "@/lib/tauri";
import { useFilterStore } from "@/lib/stores/navigation";
import type { ProjectSummary } from "@/lib/types";

type SortColumn = "project_name" | "tempo_bpm" | "bank_count" | "last_modified";
type SortDirection = "asc" | "desc";

const COLUMNS: {
  key: SortColumn;
  label: string;
  className: string;
}[] = [
  { key: "project_name", label: "NAME", className: "min-w-[120px]" },
  { key: "tempo_bpm", label: "BPM", className: "w-20" },
  { key: "bank_count", label: "BANKS", className: "w-20" },
  { key: "last_modified", label: "MODIFIED", className: "w-[140px]" },
];

function sortProjects(
  projects: ProjectSummary[],
  column: SortColumn,
  direction: SortDirection
): ProjectSummary[] {
  return [...projects].sort((a, b) => {
    let aVal: string | number | null;
    let bVal: string | number | null;

    switch (column) {
      case "project_name":
        aVal = a.project_name.toLowerCase();
        bVal = b.project_name.toLowerCase();
        break;
      case "tempo_bpm":
        aVal = a.tempo_bpm;
        bVal = b.tempo_bpm;
        break;
      case "bank_count":
        aVal = a.bank_count;
        bVal = b.bank_count;
        break;
      case "last_modified":
        aVal = a.last_modified;
        bVal = b.last_modified;
        break;
    }

    // Nulls always sort last
    if (aVal === null && bVal === null) return 0;
    if (aVal === null) return 1;
    if (bVal === null) return -1;

    const cmp = aVal < bVal ? -1 : aVal > bVal ? 1 : 0;
    return direction === "asc" ? cmp : -cmp;
  });
}

export function ProjectTable() {
  const { filter, hasActiveFilters } = useFilterStore();
  const [sortColumn, setSortColumn] = useState<SortColumn>("last_modified");
  const [sortDirection, setSortDirection] = useState<SortDirection>("desc");

  const { data, isPending } = useQuery({
    queryKey: ["projects", filter],
    queryFn: async () => {
      await indexOtProjects();
      return listProjects(filter);
    },
  });

  const sortedData = useMemo(() => {
    if (!data) return [];
    return sortProjects(data, sortColumn, sortDirection);
  }, [data, sortColumn, sortDirection]);

  const handleSort = (column: SortColumn) => {
    if (sortColumn === column) {
      setSortDirection((d) => (d === "asc" ? "desc" : "asc"));
    } else {
      setSortColumn(column);
      setSortDirection("asc");
    }
  };

  return (
    <div className="flex flex-col h-full w-full">
      <ProjectSearchBar resultCount={data?.length} />

      <div className="flex-1 overflow-y-auto">
        <Table>
          <TableHeader>
            <TableRow className="hover:bg-transparent">
              {COLUMNS.map((col) => (
                <TableHead
                  key={col.key}
                  className={`${col.className} cursor-pointer select-none`}
                  onClick={() => handleSort(col.key)}
                >
                  <span className="flex items-center gap-1 font-mono text-xs font-semibold uppercase text-muted-foreground">
                    {col.label}
                    {sortColumn === col.key ? (
                      sortDirection === "asc" ? (
                        <ChevronUp size={12} />
                      ) : (
                        <ChevronDown size={12} />
                      )
                    ) : null}
                  </span>
                </TableHead>
              ))}
            </TableRow>
          </TableHeader>
          <TableBody>
            {isPending ? (
              // Skeleton rows while loading
              Array.from({ length: 5 }).map((_, i) => (
                <TableRow key={i} className="h-9">
                  <td className="min-w-[120px] py-0 px-4">
                    <Skeleton className="h-4 w-[140px]" />
                  </td>
                  <td className="w-20 py-0 px-4">
                    <Skeleton className="h-4 w-12" />
                  </td>
                  <td className="w-20 py-0 px-4">
                    <Skeleton className="h-4 w-12" />
                  </td>
                  <td className="w-[140px] py-0 px-4">
                    <Skeleton className="h-4 w-[100px]" />
                  </td>
                </TableRow>
              ))
            ) : sortedData.length === 0 ? (
              // Empty states per UI-SPEC Copywriting Contract
              <TableRow className="hover:bg-transparent">
                <td colSpan={4} className="py-12 text-center">
                  {hasActiveFilters ? (
                    <div className="space-y-2">
                      <p className="font-mono text-sm font-semibold text-foreground">
                        No matching projects
                      </p>
                      <p className="text-sm text-muted-foreground">
                        No projects match &ldquo;{filter.name}&rdquo;. Clear the search to see all
                        projects.
                      </p>
                    </div>
                  ) : (
                    <div className="space-y-2">
                      <p className="font-mono text-sm font-semibold text-foreground">
                        No projects found
                      </p>
                      <p className="text-sm text-muted-foreground">
                        The Octatrack card appears to be empty or uses an unsupported folder
                        structure.
                      </p>
                    </div>
                  )}
                </td>
              </TableRow>
            ) : (
              sortedData.map((project) => (
                <ProjectRow key={project.id} project={project} />
              ))
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  );
}
