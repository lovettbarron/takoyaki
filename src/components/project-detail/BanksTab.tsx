"use client";

import type { ProjectDetail, BankDetail } from "@/lib/types";
import { useNavigationStore } from "@/lib/stores/navigation";
import { BankGridCell } from "./BankGridCell";

interface BanksTabProps {
  project: ProjectDetail;
}

export function BanksTab({ project }: BanksTabProps) {
  const { selectedBankIndex, selectBank } = useNavigationStore();

  const banks = project.banks;
  const populatedCount = banks.filter((b) => b.populated).length;

  // Pad banks array to always have 16 entries for the 4x4 grid
  const paddedBanks: (BankDetail | null)[] = Array.from(
    { length: 16 },
    (_, i) => banks[i] ?? null
  );

  const selectedBank =
    selectedBankIndex !== null ? banks[selectedBankIndex] ?? null : null;

  return (
    <div className="space-y-4">
      {/* 4x4 Bank grid */}
      <div>
        <div className="grid grid-cols-4 gap-2">
          {paddedBanks.map((bank, i) => {
            const isPopulated = bank?.populated ?? false;
            const isSelected = selectedBankIndex === i && isPopulated;
            return (
              <BankGridCell
                key={i}
                bankIndex={i}
                populated={isPopulated}
                selected={isSelected}
                onClick={
                  isPopulated
                    ? () =>
                        selectedBankIndex === i
                          ? selectBank(null)
                          : selectBank(i)
                    : undefined
                }
              />
            );
          })}
        </div>
        {/* Bank count summary */}
        <p className="mt-2 text-xs text-muted-foreground">
          {populatedCount} of 16 banks used
        </p>
      </div>

      {/* Drill-down panel or hint */}
      {selectedBankIndex === null || selectedBank === null ? (
        <p className="text-sm text-muted-foreground">
          Select a bank to see its parts and tracks.
        </p>
      ) : (
        <div className="space-y-3">
          {/* Bank heading */}
          <h2 className="font-mono text-lg font-semibold text-foreground">
            Bank {String(selectedBankIndex + 1).padStart(2, "0")}
            {selectedBank.bank_name ? (
              <span className="ml-2 font-normal text-muted-foreground">
                — {selectedBank.bank_name}
              </span>
            ) : null}
          </h2>

          {/* Parts: 4-column layout */}
          {selectedBank.parts.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              No patterns in this bank.
            </p>
          ) : (
            <div className="grid grid-cols-4 gap-4">
              {selectedBank.parts.map((part) => (
                <div key={part.part_index} className="space-y-1">
                  {/* Part heading */}
                  <p className="font-mono text-xs font-semibold text-muted-foreground uppercase">
                    Part {part.part_index + 1}
                  </p>
                  {/* Track list */}
                  <div className="space-y-0.5">
                    {part.tracks.map((track) => (
                      <div
                        key={track.track_index}
                        className="flex items-center gap-1.5"
                      >
                        {/* Track index — Label, muted, tabular-nums */}
                        <span className="w-4 shrink-0 font-mono text-xs tabular-nums text-muted-foreground">
                          {track.track_index + 1}
                        </span>
                        {/* Machine type — Label, muted, font-mono */}
                        <span className="w-12 shrink-0 truncate font-mono text-xs text-muted-foreground">
                          {track.machine_type}
                        </span>
                        {/* Sample filename — Body, truncated */}
                        <span className="min-w-0 truncate font-mono text-xs text-foreground">
                          {track.sample_filename ?? "--"}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
