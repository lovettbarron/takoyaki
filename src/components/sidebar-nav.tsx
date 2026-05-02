"use client";

import { FolderOpen, Music, Archive, Settings } from "lucide-react";
import type { LucideIcon } from "lucide-react";

interface NavSection {
  key: string;
  label: string;
  icon: LucideIcon;
  available: boolean;
}

const NAV_SECTIONS: NavSection[] = [
  { key: "projects", label: "Projects", icon: FolderOpen, available: true },
  { key: "samples", label: "Samples", icon: Music, available: false },
  { key: "backups", label: "Backups", icon: Archive, available: true },
  { key: "settings", label: "Settings", icon: Settings, available: true },
];

interface SidebarNavProps {
  activeSection: string;
  onSectionChange: (section: string) => void;
}

export function SidebarNav({ activeSection, onSectionChange }: SidebarNavProps) {
  return (
    <nav className="flex flex-col gap-1 p-3">
      {NAV_SECTIONS.map(({ key, label, icon: Icon, available }) => (
        <button
          key={key}
          onClick={() => available && onSectionChange(key)}
          disabled={!available}
          title={!available ? "Available in a future update" : undefined}
          className={`flex items-center gap-2 px-3 min-h-[44px] rounded text-xs font-semibold font-mono
            transition-colors relative
            ${activeSection === key
              ? "bg-accent/15 text-accent"
              : available
                ? "text-foreground/80 hover:bg-muted hover:text-foreground"
                : "text-muted-foreground/40 cursor-not-allowed opacity-50 pointer-events-none"
            }
          `}
        >
          {activeSection === key && (
            <span className="absolute left-0 top-1/2 -translate-y-1/2 w-[3px] h-5 bg-accent rounded-r" />
          )}
          <Icon size={14} strokeWidth={2} />
          {label}
        </button>
      ))}
    </nav>
  );
}
