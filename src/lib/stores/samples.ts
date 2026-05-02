import { create } from "zustand";
import type { FileChangeManifest } from "@/lib/types";

export type AssignStatus =
  | "idle"
  | "picking-file"
  | "dry-running"
  | "confirming"
  | "assigning"
  | "complete"
  | "failed";

interface SamplesState {
  // Assignment flow state
  assignStatus: AssignStatus;
  dryRunManifest: FileChangeManifest | null;
  hardBlock: string | null;
  softWarnings: string[];
  successMessage: string | null;
  pendingSlotType: "flex" | "static" | null;
  pendingSlotIndex: number | null;
  pendingFilePath: string | null;
  pendingFromWallflower: boolean;

  // Per-slot error state (for inline errors per D-13)
  slotError: { slotIndex: number; slotType: "flex" | "static"; message: string } | null;
  slotErrorRedirect: { label: string; targetSlotType: "flex" | "static"; targetSlotIndex: number } | null;

  // Wallflower state
  wallflowerConnected: boolean;
  wallflowerDbPath: string | null;
  wallflowerPanelExpanded: boolean;

  // Slot picker dialog state (for push-to-slot flow)
  slotPickerOpen: boolean;
  slotPickerSampleFilename: string | null;
  slotPickerSampleFilePath: string | null;

  // Actions
  setAssignStatus: (status: AssignStatus) => void;
  setDryRunResult: (manifest: FileChangeManifest | null, hardBlock: string | null, softWarnings: string[]) => void;
  setPendingAssign: (slotType: "flex" | "static", slotIndex: number, filePath: string, fromWallflower: boolean) => void;
  setSlotError: (slotIndex: number, slotType: "flex" | "static", message: string, redirect?: { label: string; targetSlotType: "flex" | "static"; targetSlotIndex: number }) => void;
  clearSlotError: () => void;
  setSuccessMessage: (message: string) => void;
  setWallflowerConnected: (connected: boolean, dbPath?: string | null) => void;
  setWallflowerPanelExpanded: (expanded: boolean) => void;
  openSlotPicker: (filename: string, filePath: string) => void;
  closeSlotPicker: () => void;
  reset: () => void;
}

export const useSamplesStore = create<SamplesState>((set) => ({
  assignStatus: "idle",
  dryRunManifest: null,
  hardBlock: null,
  softWarnings: [],
  successMessage: null,
  pendingSlotType: null,
  pendingSlotIndex: null,
  pendingFilePath: null,
  pendingFromWallflower: false,
  slotError: null,
  slotErrorRedirect: null,
  wallflowerConnected: false,
  wallflowerDbPath: null,
  wallflowerPanelExpanded: true, // Default expanded per D-09 / UI-SPEC
  slotPickerOpen: false,
  slotPickerSampleFilename: null,
  slotPickerSampleFilePath: null,

  setAssignStatus: (status) => set({ assignStatus: status }),

  setDryRunResult: (manifest, hardBlock, softWarnings) =>
    set({ dryRunManifest: manifest, hardBlock: hardBlock, softWarnings }),

  setPendingAssign: (slotType, slotIndex, filePath, fromWallflower) =>
    set({
      pendingSlotType: slotType,
      pendingSlotIndex: slotIndex,
      pendingFilePath: filePath,
      pendingFromWallflower: fromWallflower,
      assignStatus: "confirming",
    }),

  setSlotError: (slotIndex, slotType, message, redirect) =>
    set({
      slotError: { slotIndex, slotType, message },
      slotErrorRedirect: redirect ?? null,
    }),

  clearSlotError: () => set({ slotError: null, slotErrorRedirect: null }),

  setSuccessMessage: (message) =>
    set({ successMessage: message, assignStatus: "complete" }),

  setWallflowerConnected: (connected, dbPath) =>
    set({ wallflowerConnected: connected, wallflowerDbPath: dbPath ?? null }),

  setWallflowerPanelExpanded: (expanded) =>
    set({ wallflowerPanelExpanded: expanded }),

  openSlotPicker: (filename, filePath) =>
    set({
      slotPickerOpen: true,
      slotPickerSampleFilename: filename,
      slotPickerSampleFilePath: filePath,
    }),

  closeSlotPicker: () =>
    set({
      slotPickerOpen: false,
      slotPickerSampleFilename: null,
      slotPickerSampleFilePath: null,
    }),

  reset: () =>
    set({
      assignStatus: "idle",
      dryRunManifest: null,
      hardBlock: null,
      softWarnings: [],
      successMessage: null,
      pendingSlotType: null,
      pendingSlotIndex: null,
      pendingFilePath: null,
      pendingFromWallflower: false,
      slotError: null,
      slotErrorRedirect: null,
      slotPickerOpen: false,
      slotPickerSampleFilename: null,
      slotPickerSampleFilePath: null,
    }),
}));
