// Phase 2 TypeScript types — manually maintained to match Rust specta-generated types.
// When tauri-specta auto-generates bindings.ts, these can be replaced by the auto-generated ones.

// Project browsing types
export interface ProjectFilter {
  name?: string;
  bpm_min?: number;
  bpm_max?: number;
  modified_since?: string;
}

export interface ProjectSummary {
  id: string;
  set_name: string;
  project_name: string;
  card_path: string;
  tempo_bpm: number | null;
  bank_count: number | null;
  last_modified: string | null;
}

export interface ProjectDetail {
  project_name: string;
  tempo_bpm: number | null;
  bank_count: number | null;
  last_modified: string | null;
  banks: BankDetail[];
}

export interface BankDetail {
  bank_index: number;
  populated: boolean;
  bank_name: string | null;
  parts: PartDetail[];
}

export interface PartDetail {
  part_index: number;
  part_name: string | null;
  tracks: TrackDetail[];
}

export interface TrackDetail {
  track_index: number;
  machine_type: string;
  sample_slot_index: number | null;
  sample_filename: string | null;
}

export interface SampleSlotResponse {
  flex: SampleSlot[];
  static_slots: SampleSlot[];
}

export interface SampleSlot {
  slot_index: number;
  occupied: boolean;
  filename: string | null;
  full_path: string | null;
  sample_rate: number | null;
  status: string;
}

export interface HealthIssue {
  severity: "error" | "warning" | "info";
  slot_type: string;
  slot_index: number;
  path?: string;
  filename?: string;
  detail: string;
}

export interface HealthCheckComplete {
  project_id: string;
  issues: HealthIssue[];
  scanned_at: string;
}

// Phase 3: Backup types (SAFE-01, SAFE-02, SAFE-05, SAFE-06, SAFE-07)

export interface BackupSummary {
  id: string;
  project_id: string;
  project_name: string;
  dest_path: string;
  created_at: string;
  operation: string;       // "manual-backup" | "pre-restore"
  file_count: number;
  total_bytes: number;
  checksum_ok: boolean;
  status: string;          // "in-progress" | "complete"
}

export interface BackupFileRecord {
  id: string;
  backup_id: string;
  relative_path: string;
  stored_path: string;
  file_hash: string;
  size_bytes: number;
  change_type: string;
}

export type ChangeType = "Added" | "Modified" | "Removed" | "Unchanged";

export interface FileChangeEntry {
  path: string;
  changeType: ChangeType;
  sizeBytes: number;
}

export interface FileChangeManifest {
  entries: FileChangeEntry[];
  totalAdded: number;
  totalModified: number;
  totalRemoved: number;
  totalUnchanged: number;
  totalBytes: number;
  destinationPath: string;
  operationLabel: string;
  projectName: string;
}

export type BackupEventType = "started" | "progress" | "complete" | "failed";

export type BackupEvent =
  | { event: "started"; data: { totalFiles: number; destination: string } }
  | { event: "progress"; data: { filesCopied: number; totalFiles: number; currentFile: string } }
  | { event: "complete"; data: { filesCopied: number; totalBytes: number; destination: string; checksumOk: boolean } }
  | { event: "failed"; data: { reason: string } };
