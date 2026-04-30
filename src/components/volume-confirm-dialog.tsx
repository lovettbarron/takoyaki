"use client";

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";

interface VolumeConfirmDialogProps {
  open: boolean;
  mountPoint: string;
  onConfirm: () => void;
  onDismiss: () => void;
}

export function VolumeConfirmDialog({
  open,
  mountPoint,
  onConfirm,
  onDismiss,
}: VolumeConfirmDialogProps) {
  return (
    <Dialog open={open} onOpenChange={(isOpen: boolean) => { if (!isOpen) onDismiss(); }}>
      <DialogContent className="sm:max-w-md" showCloseButton={false}>
        <DialogHeader>
          <DialogTitle className="font-mono text-base font-semibold">
            Octatrack Found
          </DialogTitle>
          <DialogDescription className="text-sm leading-relaxed">
            Found a volume that looks like an Octatrack at:{" "}
            <code className="font-mono text-foreground bg-muted px-1 py-0.5 rounded text-xs">
              {mountPoint}
            </code>
            <br />
            <br />
            Would you like to use this device?
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="ghost" onClick={onDismiss} className="font-mono text-xs">
            Not Now
          </Button>
          <Button
            onClick={onConfirm}
            className="font-mono text-xs bg-accent text-accent-foreground hover:bg-accent/90"
          >
            Use this device
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
