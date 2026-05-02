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
  ManagementEvent,
  SampleDryRunResult,
  AssignSampleResult,
  WallflowerStatus,
  WallflowerSample,
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

export async function indexOtProjects(): Promise<number> {
  return invoke("index_ot_projects");
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

// Phase 4: Management IPC wrappers
export async function computeManagementDryRun(
  projectId: string,
  operation: string,
  targetProjectId?: string,
  bankIndex?: number,
  newName?: string,
): Promise<FileChangeManifest> {
  return invoke("compute_management_dry_run", {
    projectId,
    operation,
    targetProjectId: targetProjectId ?? null,
    bankIndex: bankIndex ?? null,
    newName: newName ?? null,
  });
}

export async function duplicateProject(
  projectId: string,
  newName: string,
  onEvent: Channel<ManagementEvent>,
): Promise<void> {
  return invoke("duplicate_project", { projectId, newName, onEvent });
}

export async function renameProject(
  projectId: string,
  newName: string,
): Promise<void> {
  return invoke("rename_project", { projectId, newName });
}

export async function exportProject(
  projectId: string,
  onEvent: Channel<ManagementEvent>,
): Promise<void> {
  return invoke("export_project", { projectId, onEvent });
}

export async function copyBank(
  sourceProjectId: string,
  sourceBankIndex: number,
  targetProjectId: string,
  targetBankIndex: number,
  conflictResolutions: Record<string, string>,
  onEvent: Channel<ManagementEvent>,
): Promise<void> {
  return invoke("copy_bank", {
    sourceProjectId,
    sourceBankIndex,
    targetProjectId,
    targetBankIndex,
    conflictResolutions,
    onEvent,
  });
}

// ── Phase 5: Sample Assignment IPC ──────────────────────────────────────

export async function computeSampleDryRun(
  projectId: string,
  slotType: "flex" | "static",
  slotIndex: number,
  filePath: string,
): Promise<SampleDryRunResult> {
  return invoke("compute_sample_dry_run", { projectId, slotType, slotIndex, filePath });
}

export async function assignSample(
  projectId: string,
  slotType: "flex" | "static",
  slotIndex: number,
  filePath: string,
  fromWallflower: boolean,
): Promise<AssignSampleResult> {
  return invoke("assign_sample", { projectId, slotType, slotIndex, filePath, fromWallflower });
}

// ── Phase 5: Wallflower IPC ─────────────────────────────────────────────

export async function getWallflowerStatus(): Promise<WallflowerStatus> {
  return invoke("get_wallflower_status");
}

export async function searchWallflowerSamples(query: string): Promise<WallflowerSample[]> {
  return invoke("search_wallflower_samples", { query });
}

export async function setWallflowerDbPath(path: string): Promise<WallflowerStatus> {
  return invoke("set_wallflower_db_path", { path });
}

// ── Audio Preview IPC ─────────────────────────────────────────────────────

export async function getSampleAudioBytes(
  projectId: string,
  samplePath: string
): Promise<number[]> {
  return invoke("get_sample_audio_bytes", { projectId, samplePath });
}
