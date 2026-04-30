"use client";

import { useRef } from "react";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useFilterStore } from "@/lib/stores/navigation";

interface ProjectSearchBarProps {
  resultCount?: number;
}

// BPM range option → { bpm_min, bpm_max }
const BPM_OPTIONS: {
  value: string;
  label: string;
  bpm_min?: number;
  bpm_max?: number;
}[] = [
  { value: "any", label: "Any BPM" },
  { value: "60-90", label: "60 – 90", bpm_min: 60, bpm_max: 90 },
  { value: "90-120", label: "90 – 120", bpm_min: 90, bpm_max: 120 },
  { value: "120-140", label: "120 – 140", bpm_min: 120, bpm_max: 140 },
  { value: "140+", label: "140+", bpm_min: 140 },
];

// Date range option → modified_since ISO date string
function getModifiedSince(days: number): string {
  const d = new Date();
  d.setDate(d.getDate() - days);
  return d.toISOString().slice(0, 10); // "YYYY-MM-DD"
}

const DATE_OPTIONS: { value: string; label: string; days?: number }[] = [
  { value: "any", label: "Any time" },
  { value: "7", label: "Last 7 days", days: 7 },
  { value: "30", label: "Last 30 days", days: 30 },
  { value: "90", label: "Last 90 days", days: 90 },
];

export function ProjectSearchBar({ resultCount }: ProjectSearchBarProps) {
  const { filter, setFilter, clearFilter, hasActiveFilters } = useFilterStore();
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleNameChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => {
      setFilter({ name: value || undefined });
    }, 150);
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Escape") {
      if (debounceRef.current) clearTimeout(debounceRef.current);
      (e.target as HTMLInputElement).value = "";
      setFilter({ name: undefined });
    }
  };

  const handleBpmChange = (value: string | null) => {
    if (!value || value === "any") {
      setFilter({ bpm_min: undefined, bpm_max: undefined });
      return;
    }
    const option = BPM_OPTIONS.find((o) => o.value === value);
    if (!option) {
      setFilter({ bpm_min: undefined, bpm_max: undefined });
    } else {
      setFilter({ bpm_min: option.bpm_min, bpm_max: option.bpm_max });
    }
  };

  const handleDateChange = (value: string | null) => {
    if (!value || value === "any") {
      setFilter({ modified_since: undefined });
      return;
    }
    const option = DATE_OPTIONS.find((o) => o.value === value);
    if (!option || !option.days) {
      setFilter({ modified_since: undefined });
    } else {
      setFilter({ modified_since: getModifiedSince(option.days) });
    }
  };

  // Derive the current BPM select value from filter state
  const currentBpmValue =
    BPM_OPTIONS.find(
      (o) => o.bpm_min === filter.bpm_min && o.bpm_max === filter.bpm_max
    )?.value ?? "any";

  return (
    <div
      role="search"
      aria-label="Filter projects"
      className="flex items-center gap-2 px-4 pb-2 pt-3"
    >
      <Input
        type="search"
        placeholder="Search projects..."
        defaultValue={filter.name ?? ""}
        onChange={handleNameChange}
        onKeyDown={handleKeyDown}
        className="h-8 w-[200px] font-mono text-sm"
        aria-label="Search projects by name"
      />

      <Select value={currentBpmValue} onValueChange={handleBpmChange}>
        <SelectTrigger className="h-8 w-[130px] font-mono text-xs">
          <SelectValue placeholder="Any BPM" />
        </SelectTrigger>
        <SelectContent>
          {BPM_OPTIONS.map((opt) => (
            <SelectItem key={opt.value} value={opt.value} className="font-mono text-xs">
              {opt.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>

      <Select value="any" onValueChange={handleDateChange}>
        <SelectTrigger className="h-8 w-[140px] font-mono text-xs">
          <SelectValue placeholder="Any time" />
        </SelectTrigger>
        <SelectContent>
          {DATE_OPTIONS.map((opt) => (
            <SelectItem key={opt.value} value={opt.value} className="font-mono text-xs">
              {opt.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>

      {hasActiveFilters && (
        <>
          {resultCount !== undefined && (
            <span className="text-xs text-muted-foreground tabular-nums">
              {resultCount} {resultCount === 1 ? "result" : "results"}
            </span>
          )}
          <button
            type="button"
            onClick={clearFilter}
            className="text-xs text-muted-foreground hover:text-foreground transition-colors"
          >
            Clear
          </button>
        </>
      )}

      {/* Screen reader live region */}
      <div aria-live="polite" className="sr-only">
        {resultCount !== undefined
          ? `${resultCount} project${resultCount === 1 ? "" : "s"} matching filters`
          : ""}
      </div>
    </div>
  );
}
