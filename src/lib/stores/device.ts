import { create } from "zustand";

export interface DeviceState {
  connected: boolean;
  mountPoint: string | null;
  confirmed: boolean;
  setConnected: (connected: boolean, mountPoint: string | null) => void;
  setConfirmed: (confirmed: boolean) => void;
  reset: () => void;
}

export const useDeviceStore = create<DeviceState>((set) => ({
  connected: false,
  mountPoint: null,
  confirmed: false,
  setConnected: (connected, mountPoint) => set({ connected, mountPoint }),
  setConfirmed: (confirmed) => set({ confirmed }),
  reset: () => set({ connected: false, mountPoint: null, confirmed: false }),
}));
