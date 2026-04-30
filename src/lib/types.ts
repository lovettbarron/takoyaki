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
