"use client";

import { useState } from "react";
import { useBackupStore } from "@/lib/stores/backup";
import { Progress } from "@/components/ui/progress";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Archive, RotateCcw } from "lucide-react";
import { cancelBackup } from "@/lib/tauri";

export function BackupProgressView() {
  const { status, progress, activeProjectName, activeOperation } = useBackupStore();
  const [showCancelConfirm, setShowCancelConfirm] = useState(false);

  const isBackup = activeOperation === "backup";
  const operationLabel = isBackup ? "backup" : "restore";

  const filesCopied = progress?.filesCopied ?? 0;
  const totalFiles = progress?.totalFiles ?? 1;
  const progressPercent = totalFiles > 0 ? (filesCopied / totalFiles) * 100 : 0;

  const handleCancelClick = () => {
    setShowCancelConfirm(true);
  };

  const handleCancelConfirm = async () => {
    setShowCancelConfirm(false);
    try {
      await cancelBackup();
    } catch {
      // cancelBackup error is non-fatal — backend will handle cleanup
    }
  };

  const handleCancelDismiss = () => {
    setShowCancelConfirm(false);
  };

  return (
    <div className="flex flex-col items-center justify-center gap-4 p-6 h-full">
      {/* Heading */}
      <h2 className="font-mono text-lg font-semibold text-foreground flex items-center gap-2">
        {isBackup ? (
          <Archive className="h-4 w-4" />
        ) : (
          <RotateCcw className="h-4 w-4" />
        )}
        {isBackup
          ? `Backing up ${activeProjectName ?? "project"}...`
          : `Restoring ${activeProjectName ?? "project"}...`}
      </h2>

      {/* Separator */}
      <div className="border-t border-border w-full max-w-md" />

      {/* Progress bar row */}
      <div className="w-full max-w-md flex flex-col gap-1">
        <Progress value={progressPercent} className="h-2" />
        <span className="font-mono text-xs font-semibold text-muted-foreground text-right">
          {filesCopied} of {totalFiles} files
        </span>
      </div>

      {/* Current file */}
      <div className="font-mono text-xs text-muted-foreground pt-1 max-w-md text-ellipsis overflow-hidden whitespace-nowrap w-full">
        {progress?.currentFile ?? "Starting..."}
      </div>

      {/* Prose note */}
      <p className="text-sm text-muted-foreground pt-2 max-w-md" style={{ fontFamily: "var(--font-sans)" }}>
        {isBackup
          ? "This may take a moment for large sample collections."
          : "Do not disconnect your Octatrack until this completes."}
      </p>

      {/* Cancel button */}
      <Button
        variant="ghost"
        className="font-mono text-xs mt-4"
        onClick={handleCancelClick}
        disabled={status !== "in-progress"}
      >
        {isBackup ? "Cancel Backup" : "Cancel Restore"}
      </Button>

      {/* Cancel confirmation dialog */}
      <Dialog
        open={showCancelConfirm}
        onOpenChange={(isOpen) => { if (!isOpen) handleCancelDismiss(); }}
      >
        <DialogContent className="sm:max-w-sm p-6" showCloseButton={false}>
          <DialogHeader>
            <DialogTitle className="font-mono text-base font-semibold">
              {isBackup ? "Cancel the backup?" : "Cancel the restore?"}
            </DialogTitle>
          </DialogHeader>
          <div className="text-sm text-muted-foreground" style={{ fontFamily: "var(--font-sans)" }}>
            {isBackup
              ? "Cancel the backup? No files have been written yet."
              : "Cancel the restore? The project is unchanged."}
          </div>
          <DialogFooter className="flex justify-end gap-2 bg-transparent border-none -mx-0 -mb-0 rounded-none p-0">
            <Button
              variant="ghost"
              className="font-mono text-xs"
              onClick={handleCancelDismiss}
            >
              Keep Going
            </Button>
            <Button
              variant="destructive"
              className="font-mono text-xs"
              onClick={handleCancelConfirm}
            >
              {isBackup ? "Cancel Backup" : "Cancel Restore"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
