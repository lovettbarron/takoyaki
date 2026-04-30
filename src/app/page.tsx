"use client";

import { useState } from "react";
import { useDeviceStore } from "@/lib/stores/device";
import { SidebarNav } from "@/components/sidebar-nav";
import { DeviceStatusBadge } from "@/components/device-status-badge";
import { Separator } from "@/components/ui/separator";

type ActiveSection = "projects" | "samples" | "backups" | "settings";

export default function Home() {
  const [activeSection, setActiveSection] = useState<ActiveSection>("projects");
  const { connected } = useDeviceStore();

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
      <main className="flex-1 flex items-center justify-center overflow-y-auto p-6">
        {!connected ? (
          <div className="text-center max-w-sm">
            <h2 className="text-lg font-semibold font-mono text-foreground mb-2">
              No Device Connected
            </h2>
            <p className="text-sm text-muted-foreground leading-relaxed" style={{ fontFamily: "var(--font-sans)" }}>
              Connect your Octatrack in USB disk mode to get started.
              The app will detect it automatically.
            </p>
          </div>
        ) : (
          <div className="text-center max-w-sm">
            <h2 className="text-lg font-semibold font-mono text-foreground mb-2">
              Projects
            </h2>
            <p className="text-sm text-muted-foreground leading-relaxed" style={{ fontFamily: "var(--font-sans)" }}>
              Project browser will be available in Phase 2.
            </p>
          </div>
        )}
      </main>
    </div>
  );
}
