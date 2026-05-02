"use client";

import { useState, useEffect } from "react";
import { useQuery } from "@tanstack/react-query";
import { ChevronDown, ChevronUp } from "lucide-react";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import { searchWallflowerSamples } from "@/lib/tauri";
import { useSamplesStore } from "@/lib/stores/samples";
import { WallflowerSampleRow } from "./WallflowerSampleRow";
import type { WallflowerSample } from "@/lib/types";

interface WallflowerPanelProps {
  onPushToSlot: (sample: WallflowerSample) => void;
}

export function WallflowerPanel({ onPushToSlot }: WallflowerPanelProps) {
  const { wallflowerPanelExpanded, setWallflowerPanelExpanded } = useSamplesStore();
  const [query, setQuery] = useState("");
  const [debouncedQuery, setDebouncedQuery] = useState("");

  // 300ms debounce per UI-SPEC (D-12) — single-char queries valid for key searches (e.g. "C")
  useEffect(() => {
    const t = setTimeout(() => setDebouncedQuery(query), 300);
    return () => clearTimeout(t);
  }, [query]);

  const { data: samples, isLoading } = useQuery({
    queryKey: ["wallflower-search", debouncedQuery],
    queryFn: () => searchWallflowerSamples(debouncedQuery),
    enabled: wallflowerPanelExpanded,
  });

  return (
    <div className="mt-8">
      {/* Collapsible trigger — full-width h-10 bar per UI-SPEC */}
      <button
        type="button"
        onClick={() => setWallflowerPanelExpanded(!wallflowerPanelExpanded)}
        className="flex h-10 w-full items-center gap-2 px-4 bg-[hsl(30,8%,14%)] border-b border-[hsl(30,8%,26%)] hover:bg-[hsl(30,8%,18%)] font-mono text-sm font-semibold uppercase text-foreground"
      >
        {wallflowerPanelExpanded ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
        WALLFLOWER LIBRARY
      </button>

      {wallflowerPanelExpanded && (
        <div className="bg-[hsl(30,8%,16%)]">
          {/* Search/filter bar per UI-SPEC, D-12 */}
          <div className="px-4 py-2">
            <input
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search by name, key, BPM, or tag..."
              className="w-full h-8 px-3 bg-[hsl(30,8%,18%)] border border-border rounded-sm font-mono text-xs text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring"
            />
          </div>

          {/* Scrollable results area, max-h-96 per UI-SPEC */}
          <ScrollArea className="max-h-96">
            {isLoading && (
              <div className="space-y-0">
                {[1, 2, 3].map((i) => (
                  <Skeleton key={i} className="h-9 w-full rounded-none" />
                ))}
              </div>
            )}

            {!isLoading && samples && samples.length === 0 && (
              <div className="flex items-center justify-center py-8">
                <span className="font-mono text-xs text-muted-foreground">
                  No samples match your search.
                </span>
              </div>
            )}

            {!isLoading && samples && samples.map((sample) => (
              <WallflowerSampleRow
                key={sample.id}
                sample={sample}
                onPush={onPushToSlot}
              />
            ))}
          </ScrollArea>

          {/* Truncation indicator when results hit the 200-row limit (RESEARCH.md Pitfall 6) */}
          {samples && samples.length === 200 && (
            <div className="px-4 py-1 border-t border-[hsl(30,8%,26%)]">
              <span className="font-mono text-xs text-muted-foreground">
                Showing 200 results — refine your search
              </span>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
