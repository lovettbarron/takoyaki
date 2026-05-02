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
import type { SampleSlot, SampleSlotResponse } from "@/lib/types";

interface SlotPickerDialogProps {
  open: boolean;
  sampleFilename: string;
  slots: SampleSlotResponse | undefined;
  onConfirm: (slotType: "flex" | "static", slotIndex: number) => void;
  onCancel: () => void;
}

export function SlotPickerDialog({
  open,
  sampleFilename,
  slots,
  onConfirm,
  onCancel,
}: SlotPickerDialogProps) {
  const [slotTypeTab, setSlotTypeTab] = useState<"flex" | "static">("flex");
  const [selectedSlot, setSelectedSlot] = useState<number | null>(null);

  // Reset selection state when dialog opens (D-10)
  useEffect(() => {
    if (open) {
      setSlotTypeTab("flex");
      setSelectedSlot(null);
    }
  }, [open]);

  const currentSlots: SampleSlot[] = slotTypeTab === "flex"
    ? (slots?.flex ?? [])
    : (slots?.static_slots ?? []);

  const selectedSlotData = selectedSlot !== null
    ? currentSlots.find((s) => s.slot_index === selectedSlot)
    : null;

  return (
    <Dialog open={open} onOpenChange={(v) => { if (!v) onCancel(); }}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle className="font-mono text-base font-semibold">
            Assign to Slot
          </DialogTitle>
          <p className="font-mono text-xs text-muted-foreground">
            Pushing {sampleFilename} from Wallflower
          </p>
        </DialogHeader>

        {/* Flex/Static type toggle per D-10 */}
        <div className="flex gap-1 py-2">
          <Button
            variant={slotTypeTab === "flex" ? "default" : "ghost"}
            size="sm"
            className="font-mono text-xs"
            onClick={() => { setSlotTypeTab("flex"); setSelectedSlot(null); }}
          >
            FLEX
          </Button>
          <Button
            variant={slotTypeTab === "static" ? "default" : "ghost"}
            size="sm"
            className="font-mono text-xs"
            onClick={() => { setSlotTypeTab("static"); setSelectedSlot(null); }}
          >
            STATIC
          </Button>
        </div>

        {/* Slot list — scrollable, max-h-48 per UI-SPEC */}
        <ScrollArea className="max-h-48 border border-border rounded">
          {currentSlots.map((slot) => (
            <button
              key={slot.slot_index}
              type="button"
              onClick={() => setSelectedSlot(slot.slot_index)}
              className={[
                "flex h-8 w-full items-center gap-2 px-3 border-b border-[hsl(30,8%,26%)] hover:bg-[hsl(30,8%,20%)]",
                selectedSlot === slot.slot_index
                  ? "bg-[hsl(30,8%,20%)] border-l-2 border-l-[hsl(38,85%,55%)]"
                  : "",
              ].join(" ")}
            >
              {/* Slot number — #NNN format, w-12 */}
              <span className="w-12 shrink-0 font-mono text-xs tabular-nums">
                #{String(slot.slot_index + 1).padStart(3, "0")}
              </span>

              {/* Filename when occupied — flex-1 truncated */}
              <span className="flex-1 min-w-0 font-mono text-xs text-muted-foreground truncate">
                {slot.filename ?? ""}
              </span>

              {/* Status chip — amber for occupied, muted for empty (UI-SPEC) */}
              {slot.occupied ? (
                <span className="font-mono text-xs text-[hsl(38,85%,55%)] border border-[hsl(38,85%,55%)] rounded px-1 shrink-0">
                  occupied
                </span>
              ) : (
                <span className="font-mono text-xs text-muted-foreground border border-muted-foreground rounded px-1 shrink-0">
                  empty
                </span>
              )}
            </button>
          ))}
        </ScrollArea>

        {/* Occupied slot warning per UI-SPEC */}
        {selectedSlotData?.occupied && (
          <p className="font-mono text-xs text-[hsl(38,85%,55%)] py-1">
            Slot {String(selectedSlotData.slot_index + 1).padStart(3, "0")} is occupied. Existing sample will be replaced.
          </p>
        )}

        <DialogFooter className="flex justify-end gap-2 pt-4 border-t border-border">
          <Button
            variant="ghost"
            className="font-mono text-xs"
            onClick={onCancel}
          >
            Close Picker
          </Button>
          <Button
            variant="default"
            className="font-mono text-xs"
            disabled={selectedSlot === null}
            onClick={() => {
              if (selectedSlot !== null) {
                onConfirm(slotTypeTab, selectedSlot);
              }
            }}
          >
            Assign to Slot
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
