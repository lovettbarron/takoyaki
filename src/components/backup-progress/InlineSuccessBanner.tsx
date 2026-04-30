"use client";

import { useEffect } from "react";
import { CircleCheck, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { SuccessBanner } from "@/lib/stores/backup";

function formatFileSize(bytes: number): string {
  if (bytes >= 1_000_000) {
    return `${Math.round(bytes / 1_000_000)} MB`;
  }
  if (bytes >= 1_000) {
    return `${Math.round(bytes / 1_000)} KB`;
  }
  return `${bytes} B`;
}

interface InlineSuccessBannerProps {
  banner: SuccessBanner;
  onDismiss: () => void;
}

export function InlineSuccessBanner({ banner, onDismiss }: InlineSuccessBannerProps) {
  useEffect(() => {
    const timer = setTimeout(() => onDismiss(), 4000);
    return () => clearTimeout(timer);
  }, [onDismiss]);

  const isBackup = banner.operation === "backup";
  const message = isBackup
    ? `Backed up ${banner.projectName} -- ${banner.fileCount} files . ${formatFileSize(banner.totalBytes)}`
    : `Restored ${banner.projectName} -- ${banner.fileCount} files . ${formatFileSize(banner.totalBytes)}`;

  return (
    <div className="fixed top-0 inset-x-0 z-50 bg-[hsl(140,30%,14%)] border-b border-[hsl(140,40%,28%)] px-4 py-2">
      <div className="flex flex-col gap-0.5 max-w-2xl mx-auto">
        {/* Line 1: icon + message + dismiss */}
        <div className="flex items-center gap-2">
          <CircleCheck className="h-4 w-4 text-[hsl(140,60%,72%)]" />
          <span className="font-mono text-xs text-[hsl(140,60%,72%)]">
            {message}
          </span>
          <Button
            variant="ghost"
            size="icon"
            className="h-5 w-5 ml-auto text-[hsl(140,60%,72%)] hover:text-[hsl(140,60%,82%)] hover:bg-transparent"
            onClick={onDismiss}
          >
            <X className="h-[14px] w-[14px]" />
            <span className="sr-only">Dismiss</span>
          </Button>
        </div>

        {/* Line 2: destination path */}
        <div className="font-mono text-xs text-[hsl(140,60%,72%)] opacity-70">
          {banner.destination}
        </div>

        {/* Line 3: checksum result */}
        {banner.checksumOk ? (
          <div className="font-mono text-xs text-[hsl(140,60%,42%)]">
            ✓ Verified
          </div>
        ) : (
          <div className="font-mono text-xs text-[hsl(0,68%,48%)]">
            Verification failed -- backup may be incomplete
          </div>
        )}
      </div>
    </div>
  );
}
