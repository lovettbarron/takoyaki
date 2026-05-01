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
import { getProjectBanks } from "@/lib/tauri";
import type { ProjectSummary, BankDetail } from "@/lib/types";

interface BankCopyPickerDialogProps {
  open: boolean;
  sourceBankIndex: number;
  sourceProjectName: string;
  sourceProjectId: string;
  projects: ProjectSummary[];
  onConfirm: (targetProjectId: string, targetBankIndex: number) => void;
  onCancel: () => void;
}

export function BankCopyPickerDialog({
  open,
  sourceBankIndex,
  sourceProjectName,
  sourceProjectId,
  projects,
  onConfirm,
  onCancel,
}: BankCopyPickerDialogProps) {
  const [step, setStep] = useState<1 | 2>(1);
  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(null);
  const [selectedBankSlot, setSelectedBankSlot] = useState<number | null>(null);
  const [targetBanks, setTargetBanks] = useState<BankDetail[]>([]);

  // Reset state when dialog opens
  useEffect(() => {
    if (open) {
      setStep(1);
      setSelectedProjectId(null);
      setSelectedBankSlot(null);
      setTargetBanks([]);
    }
  }, [open]);

  // Fetch target project banks when advancing to step 2
  async function handleNext() {
    if (!selectedProjectId) return;
    try {
      const banks = await getProjectBanks(selectedProjectId);
      setTargetBanks(banks);
    } catch {
      setTargetBanks([]);
    }
    setSelectedBankSlot(null);
    setStep(2);
  }

  function handleBack() {
    setStep(1);
    setSelectedBankSlot(null);
  }

  function handleConfirm() {
    if (selectedProjectId !== null && selectedBankSlot !== null) {
      onConfirm(selectedProjectId, selectedBankSlot);
    }
  }

  // Filter out source project — user shouldn't copy to same project
  const otherProjects = projects.filter((p) => p.id !== sourceProjectId);

  const selectedProjectName =
    otherProjects.find((p) => p.id === selectedProjectId)?.project_name ?? "";

  // Build 4x4 padded bank grid for target project
  const paddedTargetBanks: (BankDetail | null)[] = Array.from(
    { length: 16 },
    (_, i) => targetBanks[i] ?? null
  );

  const selectedSlotPopulated =
    selectedBankSlot !== null
      ? (paddedTargetBanks[selectedBankSlot]?.populated ?? false)
      : false;

  return (
    <Dialog open={open} onOpenChange={(isOpen) => { if (!isOpen) onCancel(); }}>
      <DialogContent className="sm:max-w-[480px] p-6" showCloseButton={false}>
        {step === 1 ? (
          <>
            <DialogHeader>
              <DialogTitle className="font-mono text-base font-semibold">
                Copy Bank to Project
              </DialogTitle>
            </DialogHeader>

            <p className="font-mono text-xs text-muted-foreground">
              Copying Bank {String(sourceBankIndex + 1).padStart(2, "0")} from{" "}
              <span className="text-foreground">{sourceProjectName}</span>
            </p>

            <ScrollArea className="max-h-64">
              {otherProjects.length === 0 ? (
                <div className="px-4 py-6 text-center">
                  <span className="font-mono text-xs text-muted-foreground">
                    No other projects on this card.
                  </span>
                </div>
              ) : (
                <div className="divide-y divide-border">
                  {otherProjects.map((project) => (
                    <button
                      key={project.id}
                      type="button"
                      className={[
                        "w-full text-left font-mono text-xs px-4 py-2 transition-colors",
                        selectedProjectId === project.id
                          ? "bg-[hsl(30,8%,20%)] border-l-2 border-[hsl(38,85%,55%)]"
                          : "hover:bg-[hsl(30,8%,16%)]",
                      ].join(" ")}
                      onClick={() => setSelectedProjectId(project.id)}
                    >
                      {project.project_name}
                    </button>
                  ))}
                </div>
              )}
            </ScrollArea>

            <DialogFooter className="flex justify-end gap-2 pt-4 border-t border-border">
              <Button variant="ghost" className="font-mono text-xs" onClick={onCancel}>
                Cancel
              </Button>
              <Button
                variant="default"
                className="font-mono text-xs"
                onClick={handleNext}
                disabled={selectedProjectId === null}
              >
                Next
              </Button>
            </DialogFooter>
          </>
        ) : (
          <>
            <DialogHeader>
              <DialogTitle className="font-mono text-base font-semibold">
                Select Target Slot in {selectedProjectName}
              </DialogTitle>
            </DialogHeader>

            <p className="font-mono text-xs text-muted-foreground">
              Copying Bank {String(sourceBankIndex + 1).padStart(2, "0")} from{" "}
              <span className="text-foreground">{sourceProjectName}</span>
            </p>

            {/* 4x4 bank grid */}
            <div className="grid grid-cols-4 gap-2">
              {paddedTargetBanks.map((bank, i) => {
                const isPopulated = bank?.populated ?? false;
                const isSelected = selectedBankSlot === i;
                return (
                  <button
                    key={i}
                    type="button"
                    onClick={() => setSelectedBankSlot(i)}
                    className={[
                      "flex w-12 h-12 flex-col items-center justify-center rounded border font-mono text-xs transition-colors",
                      isSelected
                        ? "border-[hsl(38,85%,55%)] bg-[hsl(30,8%,20%)]/30"
                        : "border-border hover:bg-[hsl(30,8%,20%)]",
                    ].join(" ")}
                    aria-label={`Bank ${i + 1}${isPopulated ? " (occupied)" : " (empty)"}`}
                  >
                    <span
                      className={[
                        "h-2 w-2 rounded-full",
                        isPopulated ? "bg-foreground" : "border border-muted-foreground",
                      ].join(" ")}
                    />
                    <span className="mt-1 tabular-nums">
                      {String(i + 1).padStart(2, "0")}
                    </span>
                  </button>
                );
              })}
            </div>

            {/* Overwrite warning */}
            {selectedBankSlot !== null && selectedSlotPopulated && (
              <p className="text-xs text-[hsl(38,85%,55%)]">
                Bank {String(selectedBankSlot + 1).padStart(2, "0")} is occupied. Existing content will be overwritten.
              </p>
            )}

            <DialogFooter className="flex justify-end gap-2 pt-4 border-t border-border">
              <Button variant="ghost" className="font-mono text-xs" onClick={handleBack}>
                Back
              </Button>
              <Button
                variant="default"
                className="font-mono text-xs"
                onClick={handleConfirm}
                disabled={selectedBankSlot === null}
              >
                Copy Bank
              </Button>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}
