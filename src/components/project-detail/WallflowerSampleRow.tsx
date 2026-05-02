"use client";

import { Upload } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import type { WallflowerSample } from "@/lib/types";

interface WallflowerSampleRowProps {
  sample: WallflowerSample;
  onPush: (sample: WallflowerSample) => void;
}

export function WallflowerSampleRow({ sample, onPush }: WallflowerSampleRowProps) {
  return (
    <div className="flex h-9 w-full items-center gap-2 px-3 border-b border-[hsl(30,8%,26%)] hover:bg-[hsl(30,8%,20%)]">
      {/* Filename — flex-1 truncated */}
      <span className="min-w-0 flex-1 font-mono text-xs text-foreground truncate">
        {sample.filename}
      </span>

      {/* Key badge — w-8, empty if null */}
      <span className="w-8 shrink-0 font-mono text-xs text-muted-foreground tabular-nums text-center">
        {sample.key_name ?? ""}
      </span>

      {/* BPM badge — w-10, rounded integer or empty */}
      <span className="w-10 shrink-0 font-mono text-xs text-muted-foreground tabular-nums text-center">
        {sample.bpm ? Math.round(sample.bpm) : ""}
      </span>

      {/* Tags — up to 3 badges + "+N" overflow count (D-11) */}
      <div className="flex gap-1 shrink-0 max-w-[120px] overflow-hidden items-center">
        {sample.tags.slice(0, 3).map((tag) => (
          <Badge
            key={tag}
            variant="outline"
            className="font-mono text-xs h-4 px-1 py-0 whitespace-nowrap"
          >
            {tag}
          </Badge>
        ))}
        {sample.tags.length > 3 && (
          <span className="font-mono text-xs text-muted-foreground">
            +{sample.tags.length - 3}
          </span>
        )}
      </div>

      {/* Push button — h-6 w-6, Upload icon 12px, ghost per UI-SPEC */}
      <button
        type="button"
        onClick={() => onPush(sample)}
        className="h-6 w-6 shrink-0 flex items-center justify-center rounded hover:bg-[hsl(30,8%,26%)] text-muted-foreground hover:text-foreground"
        aria-label={`Push ${sample.filename} to slot`}
      >
        <Upload size={12} />
      </button>
    </div>
  );
}
