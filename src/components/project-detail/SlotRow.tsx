"use client";

import { useState } from "react";
import { CircleCheck, CircleX, CircleAlert, Upload, Play, Square, Loader2 } from "lucide-react";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import type { SampleSlot, HealthIssue } from "@/lib/types";

interface SlotRowProps {
  slot: SampleSlot;
  slotType: "flex" | "static";
  /** Bank cross-reference data from ProjectDetail */
  crossRefs?: string[];
  /** Health issues from the background health check */
  healthIssues?: HealthIssue[];
  /** Called when the assign button is clicked — opens the file picker flow in SamplesTab */
  onAssign?: (slotIndex: number, slotType: "flex" | "static") => void;
  /** Called when the play button is clicked — triggers audio preview */
  onPlay?: (slotIndex: number, slotType: "flex" | "static") => void;
  /** Current playback state from useAudioPreview */
  playbackState?: "idle" | "loading" | "playing" | "error";
  /** Whether THIS specific slot is the currently active/playing one */
  isPlaying?: boolean;
  /** Inline error message for this slot (hard block or assignment failure) */
  assignError?: string | null;
  /** One-click redirect for slot-type mismatch errors (D-13) */
  assignErrorRedirect?: { label: string; onRedirect: () => void } | null;
}

function getSlotHealth(
  slotType: "flex" | "static",
  slotIndex: number,
  healthIssues?: HealthIssue[]
): { status: "ok" | "error" | "warning" | "info" | "unknown"; tooltip: string } {
  if (!healthIssues) return { status: "unknown", tooltip: "Health check in progress..." };

  const issue = healthIssues.find(
    (i) => i.slot_type === slotType && i.slot_index === slotIndex
  );

  if (!issue) return { status: "ok", tooltip: "Sample OK" };

  switch (issue.severity) {
    case "error":
      return { status: "error", tooltip: `File not found: ${issue.path ?? issue.detail}` };
    case "warning":
      return { status: "warning", tooltip: issue.detail };
    case "info":
      return { status: "info", tooltip: issue.detail };
  }
}

function formatSampleRate(rate: number | null): string {
  if (rate === null) return "--";
  if (rate === 44100) return "44.1k";
  if (rate === 48000) return "48k";
  return String(rate);
}

function StatusIcon({
  status,
  tooltip,
}: {
  status: "ok" | "error" | "warning" | "info" | "unknown";
  tooltip: string;
}) {
  if (status === "ok") {
    return (
      <span title={tooltip}>
        <CircleCheck size={16} className="text-[hsl(140,60%,42%)]" aria-label={tooltip} />
      </span>
    );
  }

  if (status === "error") {
    return (
      <span title={tooltip}>
        <CircleX size={16} className="text-[hsl(0,68%,48%)]" aria-label={tooltip} />
      </span>
    );
  }

  if (status === "warning") {
    return (
      <span title={tooltip}>
        <CircleAlert size={16} className="text-[hsl(38,85%,55%)]" aria-label={tooltip} />
      </span>
    );
  }

  if (status === "info") {
    return (
      <span title={tooltip}>
        <CircleAlert size={16} className="text-[hsl(210,40%,52%)]" aria-label={tooltip} />
      </span>
    );
  }

  return null;
}

export function SlotRow({ slot, slotType, crossRefs, healthIssues, onAssign, onPlay, playbackState, isPlaying, assignError, assignErrorRedirect }: SlotRowProps) {
  const [isOpen, setIsOpen] = useState(false);

  const slotNumber = String(slot.slot_index + 1).padStart(3, "0");
  const isMuted = !slot.occupied;

  // Derive health status from the background health check results
  const slotHealth = slot.occupied
    ? getSlotHealth(slotType, slot.slot_index, healthIssues)
    : null;

  return (
    <Collapsible open={isOpen} onOpenChange={setIsOpen}>
      {/* Main slot row — trigger wraps the entire row */}
      <CollapsibleTrigger
        className={[
          "flex h-9 w-full items-center gap-0 cursor-pointer border-b border-[hsl(30,8%,26%)] hover:bg-[hsl(30,8%,20%)] text-left",
          isMuted ? "text-muted-foreground" : "",
        ].join(" ")}
        onKeyDown={(e: React.KeyboardEvent) => {
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            setIsOpen((prev) => !prev);
          }
        }}
      >
        {/* Slot number — w-12 */}
        <span className="w-12 shrink-0 px-3 font-mono text-xs tabular-nums text-muted-foreground">
          #{slotNumber}
        </span>

        {/* Filename — flex-1 */}
        <span className="min-w-0 flex-1 px-2 font-mono text-sm">
          {slot.filename && slot.full_path ? (
            <span className="block truncate max-w-xs" title={slot.full_path}>
              {slot.filename}
            </span>
          ) : (
            <span className="text-muted-foreground">--</span>
          )}
        </span>

        {/* Sample rate — w-[72px] */}
        <span className="w-[72px] shrink-0 px-2 font-mono text-sm">
          {formatSampleRate(slot.sample_rate)}
        </span>

        {/* Status icon — w-12 */}
        <span className="w-12 shrink-0 flex items-center justify-center">
          {slotHealth && (
            <StatusIcon status={slotHealth.status} tooltip={slotHealth.tooltip} />
          )}
        </span>

        {/* Play button — w-8 */}
        <span className="w-8 shrink-0 flex items-center justify-center">
          {slot.occupied && slot.full_path && (
            <span
              role="button"
              tabIndex={-1}
              onClick={(e) => {
                e.stopPropagation();
                e.preventDefault();
                onPlay?.(slot.slot_index, slotType);
              }}
              onKeyDown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.stopPropagation();
                  e.preventDefault();
                  onPlay?.(slot.slot_index, slotType);
                }
              }}
              className="h-8 w-8 flex items-center justify-center rounded hover:bg-[hsl(30,8%,20%)] text-muted-foreground hover:text-foreground cursor-pointer"
              aria-label={
                isPlaying
                  ? `Stop preview of ${slotType} slot ${slot.slot_index + 1}`
                  : `Preview ${slotType} slot ${slot.slot_index + 1}`
              }
            >
              {isPlaying && playbackState === "loading" ? (
                <Loader2 size={14} className="animate-spin" />
              ) : isPlaying && playbackState === "playing" ? (
                <Square size={12} className="fill-current" />
              ) : playbackState === "error" && isPlaying ? (
                <Play size={14} className="fill-current text-destructive" />
              ) : (
                <Play size={14} className="fill-current" />
              )}
            </span>
          )}
        </span>

        {/* Assign button — w-8 trailing column */}
        <span className="w-8 shrink-0 flex items-center justify-center">
          <span
            role="button"
            tabIndex={-1}
            onClick={(e) => {
              e.stopPropagation();
              e.preventDefault();
              onAssign?.(slot.slot_index, slotType);
            }}
            className={[
              "h-8 w-8 flex items-center justify-center rounded hover:bg-[hsl(30,8%,20%)] text-muted-foreground hover:text-foreground cursor-pointer",
              !onAssign ? "opacity-40 pointer-events-none" : "",
            ].join(" ")}
            aria-label={`Assign sample to ${slotType === "flex" ? "Flex" : "Static"} slot ${String(slot.slot_index + 1).padStart(3, "0")}`}
          >
            <Upload size={14} />
          </span>
        </span>
      </CollapsibleTrigger>

      {/* Expanded cross-reference detail */}
      <CollapsibleContent>
        <div className="bg-[hsl(30,8%,16%)] px-4 py-2 text-sm">
          {crossRefs && crossRefs.length > 0 ? (
            <span className="text-muted-foreground">
              <span className="font-semibold text-foreground">
                Referenced by:
              </span>{" "}
              {crossRefs.join(", ")}
            </span>
          ) : (
            <span className="text-muted-foreground">
              Not referenced by any bank, part, or track.
            </span>
          )}
        </div>
      </CollapsibleContent>

      {/* Inline error display — hard block for format errors and slot type mismatches (D-13, D-14) */}
      {assignError && (
        <div className="flex items-start gap-2 px-3 py-2 bg-[hsl(0,68%,12%)] border border-destructive rounded mx-4 mb-2">
          <span className="font-mono text-xs text-destructive flex-1">{assignError}</span>
          {assignErrorRedirect && (
            <button
              type="button"
              className="font-mono text-xs text-[hsl(38,85%,55%)] underline whitespace-nowrap shrink-0"
              onClick={assignErrorRedirect.onRedirect}
            >
              {assignErrorRedirect.label}
            </button>
          )}
          <button
            type="button"
            className="font-mono text-xs text-muted-foreground whitespace-nowrap shrink-0"
            onClick={(e) => {
              e.stopPropagation();
              // Parent clears via clearSlotError — this button is visual-only; parent must wire dismiss
            }}
          >
            Dismiss
          </button>
        </div>
      )}
    </Collapsible>
  );
}
