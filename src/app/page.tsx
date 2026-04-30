"use client";

import { useState, useEffect } from "react";
import { useDeviceStore } from "@/lib/stores/device";
import { useNavigationStore } from "@/lib/stores/navigation";
import { useBackupStore } from "@/lib/stores/backup";
import { SidebarNav } from "@/components/sidebar-nav";
import { DeviceStatusBadge } from "@/components/device-status-badge";
import { VolumeConfirmDialog } from "@/components/volume-confirm-dialog";
import { ProjectTable } from "@/components/projects/ProjectTable";
import { ProjectDetailView } from "@/components/project-detail/ProjectDetailView";
import { HealthEventListener } from "@/components/health/HealthEventListener";
import { DryRunModal } from "@/components/backups/DryRunModal";
import { BackupsView } from "@/components/backups/BackupsView";
import { BackupProgressView } from "@/components/backup-progress/BackupProgressView";
import { InlineSuccessBanner } from "@/components/backup-progress/InlineSuccessBanner";
import { Separator } from "@/components/ui/separator";
import { confirmDevice, dismissDevice, computeDryRun, backupProject, restoreSnapshot } from "@/lib/tauri";
import { Channel } from "@tauri-apps/api/core";
import type { BackupEvent } from "@/lib/types";

type ActiveSection = "projects" | "samples" | "backups" | "settings";

export default function Home() {
  const [activeSection, setActiveSection] = useState<ActiveSection>("projects");
  const { connected, mountPoint, confirmed, setConfirmed } = useDeviceStore();
  const [showConfirmDialog, setShowConfirmDialog] = useState(false);
  const { view, selectedProjectId: navProjectId, navigateToBackups, navigateToList } = useNavigationStore();
  const {
    status,
    dryRunManifest,
    successBanner,
    activeProjectId,
    activeProjectName,
    activeOperation,
    activeBackupId,
    setStatus,
    setProgress,
    setSuccessBanner,
    setDryRunManifest,
    startOperation,
    reset,
  } = useBackupStore();

  // Show confirmation dialog when device is detected but not yet confirmed (D-14)
  // Debounce: 500ms delay per UI-SPEC.md Interaction Contract
  useEffect(() => {
    if (connected && !confirmed && mountPoint) {
      const timer = setTimeout(() => {
        setShowConfirmDialog(true);
      }, 500);
      return () => clearTimeout(timer);
    } else if (!connected) {
      setShowConfirmDialog(false);
    }
  }, [connected, confirmed, mountPoint]);

  // Auto-navigate to Projects on confirm (D-11)
  useEffect(() => {
    if (confirmed) {
      setActiveSection("projects");
    }
  }, [confirmed]);

  const handleConfirm = async () => {
    if (mountPoint) {
      try {
        await confirmDevice(mountPoint);
        setConfirmed(true);
      } catch {
        // If invoke fails (not in Tauri), just set local state
        setConfirmed(true);
      }
    }
    setShowConfirmDialog(false);
  };

  const handleDismiss = async () => {
    try {
      await dismissDevice();
    } catch {
      // Not in Tauri context
    }
    setShowConfirmDialog(false);
  };

  // Backup trigger — called from MetadataHeader onBackUp prop
  async function handleBackUpClick(projectId: string, projectName: string) {
    startOperation(projectId, projectName, "backup");
    try {
      const manifest = await computeDryRun(projectId, "backup");
      setDryRunManifest(manifest);
    } catch {
      reset();
    }
  }

  // Dry-run apply — start actual backup/restore operation
  async function handleDryRunApply() {
    setDryRunManifest(null);
    setStatus("in-progress");
    const channel = new Channel<BackupEvent>();
    channel.onmessage = (event) => {
      switch (event.event) {
        case "started":
          // already in progress state
          break;
        case "progress":
          setProgress({
            filesCopied: event.data.filesCopied,
            totalFiles: event.data.totalFiles,
            currentFile: event.data.currentFile,
          });
          break;
        case "complete":
          setStatus("complete");
          setSuccessBanner({
            message:
              activeOperation === "backup"
                ? `Backed up ${activeProjectName}`
                : `Restored ${activeProjectName}`,
            destination: event.data.destination,
            checksumOk: event.data.checksumOk,
            projectName: activeProjectName ?? "",
            fileCount: event.data.filesCopied,
            totalBytes: event.data.totalBytes,
            operation: activeOperation ?? "backup",
          });
          break;
        case "failed":
          setStatus("failed");
          break;
      }
    };
    try {
      if (activeOperation === "backup" && activeProjectId) {
        await backupProject(activeProjectId, "backup", channel);
      } else if (activeOperation === "restore" && activeBackupId) {
        await restoreSnapshot(activeBackupId, channel);
      }
    } catch {
      setStatus("failed");
    }
  }

  // Dry-run cancel — user clicked "Don't Apply"
  function handleDryRunCancel() {
    setDryRunManifest(null);
    reset();
  }

  // Success banner dismiss
  function handleBannerDismiss() {
    setSuccessBanner(null);
    reset();
  }

  return (
    <div className="flex h-screen bg-background">
      {/* Sidebar */}
      <aside className="flex flex-col w-[220px] border-r border-border bg-sidebar shrink-0">
        {/* App title + device status */}
        <div className="flex flex-col px-4 pt-8 pb-2">
          <h1 className="text-sm font-semibold font-mono text-foreground tracking-wide">
            Takoyaki
          </h1>
          <DeviceStatusBadge />
        </div>
        <Separator className="mx-3" />
        {/* Navigation */}
        <SidebarNav
          activeSection={activeSection}
          onSectionChange={(s) => {
            setActiveSection(s as ActiveSection);
            if (s === "backups") {
              navigateToBackups();
            } else if (s === "projects") {
              navigateToList();
            }
          }}
        />
        {/* Spacer pushes version to bottom */}
        <div className="flex-1" />
        <div className="px-4 pb-4">
          <span className="text-[10px] text-muted-foreground/40 font-mono">v0.1.0</span>
        </div>
      </aside>

      {/* Content area */}
      <main className="flex-1 flex flex-col overflow-hidden">
        {!connected || !confirmed ? (
          <div className="flex flex-1 items-center justify-center p-6">
            <div className="text-center max-w-sm">
              <h2 className="text-lg font-semibold font-mono text-foreground mb-2">
                No Device Connected
              </h2>
              <p className="text-sm text-muted-foreground leading-relaxed" style={{ fontFamily: "var(--font-sans)" }}>
                Connect your Octatrack in USB disk mode to get started.
                The app will detect it automatically.
              </p>
            </div>
          </div>
        ) : status === "in-progress" ? (
          <>
            <HealthEventListener />
            <BackupProgressView />
          </>
        ) : (
          <>
            <HealthEventListener />
            {view === "project-list" && <ProjectTable />}
            {view === "project-detail" && (
              <ProjectDetailView
                onBackUp={
                  navProjectId && status === "idle"
                    ? () => handleBackUpClick(navProjectId, "")
                    : undefined
                }
              />
            )}
            {view === "backups" && <BackupsView />}
          </>
        )}
      </main>

      {/* Success banner overlay */}
      {successBanner && (
        <InlineSuccessBanner banner={successBanner} onDismiss={handleBannerDismiss} />
      )}

      {/* Dry-run modal */}
      <DryRunModal
        open={dryRunManifest !== null}
        manifest={dryRunManifest}
        onApply={handleDryRunApply}
        onCancel={handleDryRunCancel}
      />

      {/* Volume confirmation dialog */}
      <VolumeConfirmDialog
        open={showConfirmDialog}
        mountPoint={mountPoint || ""}
        onConfirm={handleConfirm}
        onDismiss={handleDismiss}
      />
    </div>
  );
}
