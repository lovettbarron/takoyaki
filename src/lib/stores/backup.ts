import { create } from "zustand";
import type { FileChangeManifest } from "@/lib/types";

export type BackupStatus =
  | "idle"
  | "dry-running"
  | "in-progress"
  | "complete"
  | "failed"
  | "cancelled";

export interface BackupProgress {
  filesCopied: number;
  totalFiles: number;
  currentFile: string;
}

export interface SuccessBanner {
  message: string;
  destination: string;
  checksumOk: boolean;
  projectName: string;
  fileCount: number;
  totalBytes: number;
  operation: "backup" | "restore";
}

interface BackupState {
  status: BackupStatus;
  progress: BackupProgress | null;
  successBanner: SuccessBanner | null;
  dryRunManifest: FileChangeManifest | null;
  activeProjectId: string | null;
  activeProjectName: string | null;
  activeBackupId: string | null;
  activeOperation: "backup" | "restore" | null;

  setStatus: (status: BackupStatus) => void;
  setProgress: (progress: BackupProgress) => void;
  setSuccessBanner: (banner: SuccessBanner | null) => void;
  setDryRunManifest: (manifest: FileChangeManifest | null) => void;
  startOperation: (projectId: string, projectName: string, operation: "backup" | "restore", backupId?: string) => void;
  reset: () => void;
}

export const useBackupStore = create<BackupState>((set) => ({
  status: "idle",
  progress: null,
  successBanner: null,
  dryRunManifest: null,
  activeProjectId: null,
  activeProjectName: null,
  activeBackupId: null,
  activeOperation: null,

  setStatus: (status) => set({ status }),
  setProgress: (progress) => set({ progress }),
  setSuccessBanner: (banner) => set({ successBanner: banner }),
  setDryRunManifest: (manifest) => set({ dryRunManifest: manifest }),
  startOperation: (projectId, projectName, operation, backupId) =>
    set({
      activeProjectId: projectId,
      activeProjectName: projectName,
      activeOperation: operation,
      activeBackupId: backupId ?? null,
      status: "dry-running",
      progress: null,
      successBanner: null,
    }),
  reset: () =>
    set({
      status: "idle",
      progress: null,
      successBanner: null,
      dryRunManifest: null,
      activeProjectId: null,
      activeProjectName: null,
      activeBackupId: null,
      activeOperation: null,
    }),
}));
