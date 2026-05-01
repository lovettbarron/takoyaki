import { create } from "zustand";
import type { FileChangeManifest, ManagementOperation, ConflictEntry } from "@/lib/types";

export type ManagementStatus =
  | "idle"
  | "dry-running"
  | "in-progress"
  | "complete"
  | "failed";

interface ManagementState {
  status: ManagementStatus;
  operation: ManagementOperation | null;
  activeProjectId: string | null;
  activeProjectName: string | null;
  dryRunManifest: FileChangeManifest | null;
  conflicts: ConflictEntry[];
  progress: { filesProcessed: number; totalFiles: number; currentFile: string } | null;
  successMessage: string | null;

  setStatus: (status: ManagementStatus) => void;
  startOperation: (projectId: string, projectName: string, operation: ManagementOperation) => void;
  setDryRunManifest: (manifest: FileChangeManifest | null) => void;
  setConflicts: (conflicts: ConflictEntry[]) => void;
  setProgress: (progress: { filesProcessed: number; totalFiles: number; currentFile: string }) => void;
  setSuccessMessage: (message: string) => void;
  reset: () => void;
}

export const useManagementStore = create<ManagementState>((set) => ({
  status: "idle",
  operation: null,
  activeProjectId: null,
  activeProjectName: null,
  dryRunManifest: null,
  conflicts: [],
  progress: null,
  successMessage: null,

  setStatus: (status) => set({ status }),
  startOperation: (projectId, projectName, operation) =>
    set({
      activeProjectId: projectId,
      activeProjectName: projectName,
      operation,
      status: "dry-running",
      dryRunManifest: null,
      conflicts: [],
      progress: null,
      successMessage: null,
    }),
  setDryRunManifest: (manifest) => set({ dryRunManifest: manifest }),
  setConflicts: (conflicts) => set({ conflicts }),
  setProgress: (progress) => set({ progress }),
  setSuccessMessage: (message) => set({ successMessage: message, status: "complete" }),
  reset: () =>
    set({
      status: "idle",
      operation: null,
      activeProjectId: null,
      activeProjectName: null,
      dryRunManifest: null,
      conflicts: [],
      progress: null,
      successMessage: null,
    }),
}));
