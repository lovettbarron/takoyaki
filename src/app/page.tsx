"use client";

import { useState, useEffect } from "react";
import { useDeviceStore } from "@/lib/stores/device";
import { useNavigationStore } from "@/lib/stores/navigation";
import { SidebarNav } from "@/components/sidebar-nav";
import { DeviceStatusBadge } from "@/components/device-status-badge";
import { VolumeConfirmDialog } from "@/components/volume-confirm-dialog";
import { ProjectTable } from "@/components/projects/ProjectTable";
import { ProjectDetailView } from "@/components/project-detail/ProjectDetailView";
import { Separator } from "@/components/ui/separator";
import { confirmDevice, dismissDevice } from "@/lib/tauri";

type ActiveSection = "projects" | "samples" | "backups" | "settings";

export default function Home() {
  const [activeSection, setActiveSection] = useState<ActiveSection>("projects");
  const { connected, mountPoint, confirmed, setConfirmed } = useDeviceStore();
  const [showConfirmDialog, setShowConfirmDialog] = useState(false);
  const { view } = useNavigationStore();

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
          onSectionChange={(s) => setActiveSection(s as ActiveSection)}
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
        ) : (
          <>
            {/* HealthEventListener will be mounted here by Plan 05 */}
            {view === "project-list" && <ProjectTable />}
            {view === "project-detail" && <ProjectDetailView />}
          </>
        )}
      </main>

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
