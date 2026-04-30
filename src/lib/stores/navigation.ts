import { create } from "zustand";
import type { ProjectFilter } from "@/lib/types";

// Navigation store — manages view state and breadcrumb context

type View = "project-list" | "project-detail" | "backups";

interface NavigationState {
  view: View;
  selectedProjectId: string | null;
  selectedBankIndex: number | null;
  activeTab: "banks" | "samples" | "health";

  navigateToProject: (projectId: string) => void;
  navigateToList: () => void;
  navigateToBackups: () => void;
  selectBank: (bankIndex: number | null) => void;
  setActiveTab: (tab: "banks" | "samples" | "health") => void;
}

export const useNavigationStore = create<NavigationState>((set) => ({
  view: "project-list",
  selectedProjectId: null,
  selectedBankIndex: null,
  activeTab: "banks",

  navigateToProject: (projectId) =>
    set({
      view: "project-detail",
      selectedProjectId: projectId,
      selectedBankIndex: null,
      activeTab: "banks",
    }),
  navigateToList: () =>
    set({ view: "project-list", selectedProjectId: null, selectedBankIndex: null }),
  navigateToBackups: () =>
    set({ view: "backups", selectedProjectId: null, selectedBankIndex: null, activeTab: "banks" }),
  selectBank: (bankIndex) => set({ selectedBankIndex: bankIndex }),
  setActiveTab: (tab) => set({ activeTab: tab }),
}));

// Filter store — manages ProjectFilter state for the project list search bar

interface FilterState {
  filter: ProjectFilter;
  hasActiveFilters: boolean;
  setFilter: (partial: Partial<ProjectFilter>) => void;
  clearFilter: () => void;
}

export const useFilterStore = create<FilterState>((set) => ({
  filter: {},
  hasActiveFilters: false,
  setFilter: (partial) =>
    set((state) => {
      const newFilter = { ...state.filter, ...partial };
      // Remove keys explicitly set to undefined so they don't pollute the filter object
      Object.keys(newFilter).forEach((key) => {
        const k = key as keyof ProjectFilter;
        if (newFilter[k] === undefined || newFilter[k] === null || newFilter[k] === "") {
          delete newFilter[k];
        }
      });
      return {
        filter: newFilter,
        hasActiveFilters: Object.keys(newFilter).length > 0,
      };
    }),
  clearFilter: () => set({ filter: {}, hasActiveFilters: false }),
}));
