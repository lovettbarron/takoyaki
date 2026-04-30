"use client";

interface BankGridCellProps {
  bankIndex: number;
  populated: boolean;
  selected: boolean;
  onClick?: () => void;
}

export function BankGridCell({
  bankIndex,
  populated,
  selected,
  onClick,
}: BankGridCellProps) {
  return (
    <button
      type="button"
      onClick={populated ? onClick : undefined}
      disabled={!populated}
      className={[
        "flex w-12 h-12 flex-col items-center justify-center rounded border font-mono text-xs",
        populated
          ? "cursor-pointer hover:bg-[hsl(30,8%,20%)]"
          : "cursor-default opacity-40",
        selected
          ? "border-[hsl(38,85%,55%)] bg-[hsl(30,8%,20%)]/30"
          : "border-border",
      ].join(" ")}
      aria-label={`Bank ${bankIndex + 1}${populated ? "" : " (empty)"}`}
      aria-pressed={selected}
    >
      {/* Dot indicator */}
      <span
        className={[
          "h-2 w-2 rounded-full",
          populated ? "bg-foreground" : "border border-muted-foreground",
        ].join(" ")}
      />
      {/* Bank number */}
      <span className="mt-1 tabular-nums">
        {String(bankIndex + 1).padStart(2, "0")}
      </span>
    </button>
  );
}
