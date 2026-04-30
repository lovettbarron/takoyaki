import { invoke, Channel } from "@tauri-apps/api/core";
import type {
  ProjectFilter,
  ProjectSummary,
  ProjectDetail,
  BankDetail,
  SampleSlotResponse,
  BackupSummary,
  FileChangeManifest,
  BackupEvent,
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

// Phase 3: Backup IPC wrappers

export async function listBackups(
  projectId?: string
): Promise<BackupSummary[]> {
  return invoke("list_backups", { projectId: projectId ?? null });
}

export async function computeDryRun(
  projectId: string,
  operation: string,
  backupId?: string
): Promise<FileChangeManifest> {
  return invoke("compute_dry_run", {
    projectId,
    operation,
    backupId: backupId ?? null,
  });
}

export async function backupProject(
  projectId: string,
  label: string,
  onEvent: Channel<BackupEvent>
): Promise<void> {
  return invoke("backup_project", { projectId, label, onEvent });
}

export async function restoreSnapshot(
  backupId: string,
  onEvent: Channel<BackupEvent>
): Promise<void> {
  return invoke("restore_snapshot", { backupId, onEvent });
}

export async function cancelBackup(): Promise<void> {
  return invoke("cancel_backup");
}
