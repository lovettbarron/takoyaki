"use client";

import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Toggle } from "@/components/ui/toggle";
import { getProjectSamples, getProjectDetail } from "@/lib/tauri";
import type { SampleSlot, BankDetail } from "@/lib/types";
import { SlotRow } from "./SlotRow";

interface SamplesTabProps {
  projectId: string;
}

/**
 * Build a map of slot_index => list of human-readable cross-reference strings
 * by walking the ProjectDetail banks/parts/tracks tree.
 */
function buildCrossRefMap(banks: BankDetail[]): Map<number, string[]> {
  const map = new Map<number, string[]>();

  for (const bank of banks) {
    for (const part of bank.parts) {
      for (const track of part.tracks) {
        if (track.sample_slot_index !== null) {
          const key = track.sample_slot_index;
          const label = `Bank ${String(bank.bank_index + 1).padStart(2, "0")} Part ${part.part_index + 1} Track ${track.track_index + 1}`;
          const existing = map.get(key) ?? [];
          existing.push(label);
          map.set(key, existing);
        }
      }
    }
  }

  return map;
}

function useCrossRefs(projectId: string): BankDetail[] {
  const { data: project } = useQuery({
    queryKey: ["project", projectId],
    queryFn: () => getProjectDetail(projectId),
    enabled: !!projectId,
  });
  return project?.banks ?? [];
}

/** Column header row matching SlotRow column layout */
function SlotTableHeader() {
  return (
    <div className="flex h-8 items-center border-b border-[hsl(30,8%,26%)]">
      <span className="w-12 shrink-0 px-3 font-mono text-xs font-semibold uppercase text-muted-foreground">
        #
      </span>
      <span className="min-w-0 flex-1 px-2 font-mono text-xs font-semibold uppercase text-muted-foreground">
        FILENAME
      </span>
      <span className="w-[72px] shrink-0 px-2 font-mono text-xs font-semibold uppercase text-muted-foreground">
        RATE
      </span>
      <span className="w-12 shrink-0 text-center font-mono text-xs font-semibold uppercase text-muted-foreground">
        STATUS
      </span>
    </div>
  );
}

interface SlotSectionProps {
  slots: SampleSlot[];
  showEmpty: boolean;
  crossRefMap: Map<number, string[]>;
  slotType: "flex" | "static";
}

function SlotSection({ slots, showEmpty, crossRefMap, slotType }: SlotSectionProps) {
  const filtered = slots.filter((s) => showEmpty || s.occupied);

  return (
    <div className="w-full">
      <SlotTableHeader />
      {filtered.map((slot) => (
        <SlotRow
          key={slot.slot_index}
          slot={slot}
          slotType={slotType}
          crossRefs={crossRefMap.get(slot.slot_index)}
        />
      ))}
    </div>
  );
}

export function SamplesTab({ projectId }: SamplesTabProps) {
  const [showEmpty, setShowEmpty] = useState(false);

  const { data: samples, isPending } = useQuery({
    queryKey: ["samples", projectId],
    queryFn: () => getProjectSamples(projectId),
    enabled: !!projectId,
  });

  const banks = useCrossRefs(projectId);
  const crossRefMap = buildCrossRefMap(banks);

  const flexSlots = samples?.flex ?? [];
  const staticSlots = samples?.static_slots ?? [];

  const hasAnyPopulated =
    flexSlots.some((s) => s.occupied) ||
    staticSlots.some((s) => s.occupied);

  if (!isPending && !hasAnyPopulated && !showEmpty) {
    return (
      <div className="flex flex-1 items-center justify-center p-8">
        <p className="text-sm text-muted-foreground">
          No samples assigned to any slot in this project.
        </p>
      </div>
    );
  }

  return (
    <div className="flex flex-col">
      {/* Top-right: Show all slots toggle */}
      <div className="flex justify-end px-4 pt-3 pb-1">
        <Toggle
          pressed={showEmpty}
          onPressedChange={setShowEmpty}
          className="text-xs font-mono"
          aria-label={showEmpty ? "Hide empty slots" : "Show all slots"}
        >
          {showEmpty ? "Hide empty slots" : "Show all slots"}
        </Toggle>
      </div>

      <div className="px-4">
        {/* FLEX SAMPLES section */}
        <h2 className="mb-2 font-mono text-lg font-semibold text-foreground">
          FLEX SAMPLES
        </h2>
        <SlotSection
          slots={flexSlots}
          showEmpty={showEmpty}
          crossRefMap={crossRefMap}
          slotType="flex"
        />

        {/* STATIC SAMPLES section */}
        <h2 className="mb-2 pt-6 font-mono text-lg font-semibold text-foreground">
          STATIC SAMPLES
        </h2>
        <SlotSection
          slots={staticSlots}
          showEmpty={showEmpty}
          crossRefMap={crossRefMap}
          slotType="static"
        />
      </div>
    </div>
  );
}
