import { invoke } from "@tauri-apps/api/core";

export interface DeviceStatus {
  connected: boolean;
  mountPoint: string | null;
  confirmed: boolean;
}

export async function getDeviceStatus(): Promise<DeviceStatus> {
  return invoke("get_device_status");
}

export async function confirmDevice(mountPoint: string): Promise<void> {
  return invoke("confirm_device", { mountPoint });
}

export async function dismissDevice(): Promise<void> {
  return invoke("dismiss_device");
}
