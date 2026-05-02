"use client";

import { useState, useEffect, useRef } from "react";
import { useDeviceStore } from "@/lib/stores/device";
import { useNavigationStore } from "@/lib/stores/navigation";
import { useBackupStore } from "@/lib/stores/backup";
import { useManagementStore } from "@/lib/stores/management";
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
import { BankCopyPickerDialog } from "@/components/management/BankCopyPickerDialog";
import { ConflictResolutionDialog } from "@/components/management/ConflictResolutionDialog";
import { WallflowerSettings } from "@/components/settings/WallflowerSettings";
import { Separator } from "@/components/ui/separator";
import {
  confirmDevice,
  dismissDevice,
  computeDryRun,
  backupProject,
  restoreSnapshot,
  computeManagementDryRun,
  duplicateProject,
  renameProject,
  exportProject,
  copyBank,
  listProjects,
} from "@/lib/tauri";
import { Channel } from "@tauri-apps/api/core";
import type { BackupEvent, ManagementEvent, ProjectSummary, ConflictResolution } from "@/lib/types";

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

  const {
    status: mgmtStatus,
    operation: mgmtOperation,
    dryRunManifest: mgmtDryRunManifest,
    successMessage: mgmtSuccessMessage,
    activeProjectId: mgmtActiveProjectId,
    activeProjectName: mgmtActiveProjectName,
    startOperation: mgmtStartOperation,
    setDryRunManifest: mgmtSetDryRunManifest,
    setProgress: mgmtSetProgress,
    setSuccessMessage: mgmtSetSuccessMessage,
    setStatus: mgmtSetStatus,
    reset: mgmtReset,
  } = useManagementStore();

  // Bank copy picker state
  const [bankCopyPickerOpen, setBankCopyPickerOpen] = useState(false);
  const [bankCopySourceIndex, setBankCopySourceIndex] = useState<number>(0);

  // Conflict resolution dialog state
  const [conflictDialogOpen, setConflictDialogOpen] = useState(false);
  const [pendingConflicts, setPendingConflicts] = useState<Array<{ filename: string; sourceHash: string; targetHash: string }>>([]);

  // Project list for BankCopyPickerDialog
  const [projectList, setProjectList] = useState<ProjectSummary[]>([]);

  // Refs to capture operation params for use in apply handler
  const pendingRenameRef = useRef<string | null>(null);
  const pendingBankCopyRef = useRef<{ targetProjectId: string; targetBankIndex: number } | null>(null);

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

  // Fetch project list when device is confirmed (for BankCopyPickerDialog)
  useEffect(() => {
    if (confirmed) {
      listProjects({}).then(setProjectList).catch(() => {});
    }
  }, [confirmed]);

  // Auto-dismiss management success message after 4 seconds
  useEffect(() => {
    if (mgmtSuccessMessage) {
      const timer = setTimeout(() => {
        mgmtReset();
      }, 4000);
      return () => clearTimeout(timer);
    }
  }, [mgmtSuccessMessage, mgmtReset]);

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

  // ---- Management operation handlers ----

  async function handleRename(newName: string) {
    if (!navProjectId) return;
    pendingRenameRef.current = newName;
    mgmtStartOperation(navProjectId, "", "rename");
    try {
      const manifest = await computeManagementDryRun(navProjectId, "rename", undefined, undefined, newName);
      mgmtSetDryRunManifest(manifest);
    } catch {
      mgmtReset();
    }
  }

  async function handleDuplicate() {
    if (!navProjectId) return;
    const projectName = mgmtActiveProjectName ?? "";
    // Default name: append _COPY (uppercase per OT convention), truncate to 16 chars
    let newName = `${projectName}_COPY`.substring(0, 16);
    if (newName.length > 16 || newName === projectName) {
      const prompted = window.prompt("Enter a name for the duplicate (max 16 chars, A-Z 0-9 _):", newName);
      if (!prompted) return;
      newName = prompted.toUpperCase().replace(/[^A-Z0-9_]/g, "").substring(0, 16);
      if (!newName) return;
    }
    pendingRenameRef.current = newName;
    mgmtStartOperation(navProjectId, projectName, "duplicate");
    try {
      const manifest = await computeManagementDryRun(navProjectId, "duplicate", undefined, undefined, newName);
      mgmtSetDryRunManifest(manifest);
    } catch {
      mgmtReset();
    }
  }

  async function handleExport() {
    if (!navProjectId) return;
    mgmtStartOperation(navProjectId, mgmtActiveProjectName ?? "", "export");
    try {
      const manifest = await computeManagementDryRun(navProjectId, "export");
      mgmtSetDryRunManifest(manifest);
    } catch {
      mgmtReset();
    }
  }

  function handleBankCopyTrigger(bankIndex: number) {
    setBankCopySourceIndex(bankIndex);
    setBankCopyPickerOpen(true);
  }

  async function handleBankCopyConfirm(targetProjectId: string, targetBankIndex: number) {
    setBankCopyPickerOpen(false);
    if (!navProjectId) return;
    pendingBankCopyRef.current = { targetProjectId, targetBankIndex };
    mgmtStartOperation(navProjectId, "", "bank-copy");
    try {
      const manifest = await computeManagementDryRun(
        navProjectId,
        "bank-copy",
        targetProjectId,
        bankCopySourceIndex,
      );
      mgmtSetDryRunManifest(manifest);
    } catch {
      mgmtReset();
    }
  }

  // Execute bank copy with given conflict resolutions (empty map = no conflicts or all defaulted)
  async function executeBankCopy(resolutions: Record<string, string>) {
    const projectId = mgmtActiveProjectId;
    if (!projectId || !pendingBankCopyRef.current) return;
    const { targetProjectId, targetBankIndex } = pendingBankCopyRef.current;
    mgmtSetStatus("in-progress");
    const channel = new Channel<ManagementEvent>();
    channel.onmessage = (event) => {
      if (event.event === "progress") {
        mgmtSetProgress({
          filesProcessed: event.data.filesProcessed,
          totalFiles: event.data.totalFiles,
          currentFile: event.data.currentFile,
        });
      } else if (event.event === "complete") {
        mgmtSetSuccessMessage(`Copied bank to project`);
      } else if (event.event === "failed") {
        mgmtSetStatus("failed");
      }
    };
    try {
      await copyBank(projectId, bankCopySourceIndex, targetProjectId, targetBankIndex, resolutions, channel);
    } catch {
      mgmtSetStatus("failed");
    }
  }

  async function handleMgmtDryRunApply() {
    const projectId = mgmtActiveProjectId;
    const projectName = mgmtActiveProjectName ?? "";

    // Bank-copy with conflicts: show resolution dialog before executing
    if (mgmtOperation === "bank-copy" && mgmtDryRunManifest && mgmtDryRunManifest.conflictDetails.length > 0) {
      setPendingConflicts(mgmtDryRunManifest.conflictDetails);
      mgmtSetDryRunManifest(null);
      setConflictDialogOpen(true);
      return;
    }

    mgmtSetDryRunManifest(null);
    mgmtSetStatus("in-progress");

    try {
      switch (mgmtOperation) {
        case "rename": {
          const newName = pendingRenameRef.current;
          if (!projectId || !newName) break;
          await renameProject(projectId, newName);
          mgmtSetSuccessMessage(`Renamed ${projectName} -> ${newName}`);
          break;
        }
        case "duplicate": {
          const newName = pendingRenameRef.current;
          if (!projectId || !newName) break;
          const channel = new Channel<ManagementEvent>();
          channel.onmessage = (event) => {
            if (event.event === "progress") {
              mgmtSetProgress({
                filesProcessed: event.data.filesProcessed,
                totalFiles: event.data.totalFiles,
                currentFile: event.data.currentFile,
              });
            } else if (event.event === "complete") {
              mgmtSetSuccessMessage(`Duplicated ${projectName} -> ${newName}`);
            } else if (event.event === "failed") {
              mgmtSetStatus("failed");
            }
          };
          await duplicateProject(projectId, newName, channel);
          break;
        }
        case "export": {
          if (!projectId) break;
          const channel = new Channel<ManagementEvent>();
          let exportedFiles = 0;
          channel.onmessage = (event) => {
            if (event.event === "progress") {
              exportedFiles = event.data.filesProcessed;
              mgmtSetProgress({
                filesProcessed: event.data.filesProcessed,
                totalFiles: event.data.totalFiles,
                currentFile: event.data.currentFile,
              });
            } else if (event.event === "complete") {
              mgmtSetSuccessMessage(`Exported ${projectName} -- ${exportedFiles} files`);
            } else if (event.event === "failed") {
              mgmtSetStatus("failed");
            }
          };
          await exportProject(projectId, channel);
          break;
        }
        case "bank-copy": {
          // No conflicts — proceed immediately with empty resolutions map
          await executeBankCopy({});
          break;
        }
      }
    } catch {
      mgmtSetStatus("failed");
    }
  }

  function handleMgmtDryRunCancel() {
    mgmtReset();
  }

  function handleConflictResolve(resolutions: Record<string, ConflictResolution>) {
    setConflictDialogOpen(false);
    executeBankCopy(resolutions);
  }

  function handleConflictCancel() {
    setConflictDialogOpen(false);
    mgmtReset();
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
                onRename={handleRename}
                onDuplicate={handleDuplicate}
                onExport={handleExport}
                onCopyBankToProject={handleBankCopyTrigger}
              />
            )}
            {view === "backups" && <BackupsView />}
            {activeSection === "settings" && (
              <div className="flex flex-col p-8 max-w-sm">
                <h2 className="font-mono text-base font-semibold text-foreground mb-6">
                  SETTINGS
                </h2>
                <WallflowerSettings />
              </div>
            )}
          </>
        )}
      </main>

      {/* Backup success banner overlay */}
      {successBanner && (
        <InlineSuccessBanner banner={successBanner} onDismiss={handleBannerDismiss} />
      )}

      {/* Management success message banner */}
      {mgmtSuccessMessage && (
        <div className="fixed top-0 inset-x-0 z-50 bg-[hsl(140,30%,14%)] border-b border-[hsl(140,40%,28%)] px-4 py-2">
          <div className="flex items-center gap-2 max-w-2xl mx-auto">
            <span className="font-mono text-xs text-[hsl(140,60%,72%)]">
              {mgmtSuccessMessage}
            </span>
            <button
              type="button"
              className="ml-auto font-mono text-xs text-[hsl(140,60%,72%)] hover:text-[hsl(140,60%,82%)]"
              onClick={() => mgmtReset()}
            >
              Dismiss
            </button>
          </div>
        </div>
      )}

      {/* Backup dry-run modal */}
      <DryRunModal
        open={dryRunManifest !== null}
        manifest={dryRunManifest}
        onApply={handleDryRunApply}
        onCancel={handleDryRunCancel}
      />

      {/* Management dry-run modal */}
      <DryRunModal
        open={mgmtDryRunManifest !== null && mgmtStatus === "dry-running"}
        manifest={mgmtDryRunManifest}
        onApply={handleMgmtDryRunApply}
        onCancel={handleMgmtDryRunCancel}
      />

      {/* Bank copy picker dialog */}
      <BankCopyPickerDialog
        open={bankCopyPickerOpen}
        sourceBankIndex={bankCopySourceIndex}
        sourceProjectName={mgmtActiveProjectName ?? ""}
        sourceProjectId={navProjectId ?? ""}
        projects={projectList}
        onConfirm={handleBankCopyConfirm}
        onCancel={() => setBankCopyPickerOpen(false)}
      />

      {/* Conflict resolution dialog — shown after dry-run Apply when conflicts exist */}
      <ConflictResolutionDialog
        open={conflictDialogOpen}
        conflicts={pendingConflicts}
        onResolve={handleConflictResolve}
        onCancel={handleConflictCancel}
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
