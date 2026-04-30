import { invoke } from "@tauri-apps/api/core";
import type {
  ProjectFilter,
  ProjectSummary,
  ProjectDetail,
  BankDetail,
  SampleSlotResponse,
} from "./types";

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

// Phase 2: Project browsing IPC wrappers

export async function listProjects(
  filter: ProjectFilter
): Promise<ProjectSummary[]> {
  return invoke("list_projects", { filter });
}

export async function getProjectDetail(
  projectId: string
): Promise<ProjectDetail> {
  return invoke("get_project_detail", { projectId });
}

export async function getProjectBanks(
  projectId: string
): Promise<BankDetail[]> {
  return invoke("get_project_banks", { projectId });
}

export async function getProjectSamples(
  projectId: string
): Promise<SampleSlotResponse> {
  return invoke("get_project_samples", { projectId });
}

export async function runHealthCheck(projectId: string): Promise<void> {
  // Results arrive via "health-complete" event, not return value
  return invoke("run_health_check", { projectId });
}
