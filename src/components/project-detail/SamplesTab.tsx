"use client";

import { useState, useEffect } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { CircleCheck, X } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { Toggle } from "@/components/ui/toggle";
import { getProjectSamples, getProjectDetail, computeSampleDryRun, assignSample, getWallflowerStatus } from "@/lib/tauri";
import { useSamplesStore } from "@/lib/stores/samples";
import { useDeviceStore } from "@/lib/stores/device";
import { DryRunModal } from "@/components/backups/DryRunModal";
import type { SampleSlot, BankDetail, HealthCheckComplete, HealthIssue, SampleDryRunResult, WallflowerSample } from "@/lib/types";
import { useAudioPreview } from "@/hooks/useAudioPreview";
import { SlotRow } from "./SlotRow";
import { WallflowerPanel } from "./WallflowerPanel";
import { SlotPickerDialog } from "./SlotPickerDialog";

interface SamplesTabProps {
  projectId: string;
}

/**
 * Build a map of slot_index => list of human-readable cross-reference strings
 * by walking the ProjectDetail banks/parts/tracks tree.
 */
function buildCrossRefMap(banks: BankDetail[]): Map<number, string[]> {
  const map = new Map<number, string[]>();

  for (const bank of banks) {
    for (const part of bank.parts) {
      for (const track of part.tracks) {
        if (track.sample_slot_index !== null) {
          const key = track.sample_slot_index;
          const label = `Bank ${String(bank.bank_index + 1).padStart(2, "0")} Part ${part.part_index + 1} Track ${track.track_index + 1}`;
          const existing = map.get(key) ?? [];
          existing.push(label);
          map.set(key, existing);
        }
      }
    }
  }

  return map;
}

function useCrossRefs(projectId: string): BankDetail[] {
  const { data: project } = useQuery({
    queryKey: ["project", projectId],
    queryFn: () => getProjectDetail(projectId),
    enabled: !!projectId,
  });
  return project?.banks ?? [];
}

/** Column header row matching SlotRow column layout (with trailing assign column) */
function SlotTableHeader() {
  return (
    <div className="flex h-8 items-center border-b border-[hsl(30,8%,26%)]">
      <span className="w-12 shrink-0 px-3 font-mono text-xs font-semibold uppercase text-muted-foreground">
        #
      </span>
      <span className="min-w-0 flex-1 px-2 font-mono text-xs font-semibold uppercase text-muted-foreground">
        FILENAME
      </span>
      <span className="w-[72px] shrink-0 px-2 font-mono text-xs font-semibold uppercase text-muted-foreground">
        RATE
      </span>
      <span className="w-12 shrink-0 text-center font-mono text-xs font-semibold uppercase text-muted-foreground">
        STATUS
      </span>
      {/* Trailing play column — no label (icon-only column) */}
      <span className="w-8 shrink-0" />
      {/* Trailing assign column — no label (icon-only column) */}
      <span className="w-8 shrink-0" />
    </div>
  );
}

interface SlotSectionProps {
  slots: SampleSlot[];
  showEmpty: boolean;
  crossRefMap: Map<number, string[]>;
  slotType: "flex" | "static";
  healthIssues?: HealthIssue[];
  onAssign?: (slotIndex: number, slotType: "flex" | "static") => void;
  onPlay?: (slotIndex: number, slotType: "flex" | "static") => void;
  activeSlotKey?: string | null;
  playbackState?: "idle" | "loading" | "playing" | "error";
  slotError: { slotIndex: number; slotType: "flex" | "static"; message: string } | null;
  slotErrorRedirect: { label: string; targetSlotType: "flex" | "static"; targetSlotIndex: number } | null;
  onSlotRedirect: () => void;
  onDismissError: () => void;
}

function SlotSection({
  slots,
  showEmpty,
  crossRefMap,
  slotType,
  healthIssues,
  onAssign,
  onPlay,
  activeSlotKey,
  playbackState,
  slotError,
  slotErrorRedirect,
  onSlotRedirect,
  onDismissError,
}: SlotSectionProps) {
  const filtered = slots.filter((s) => showEmpty || s.occupied);

  return (
    <div className="w-full">
      <SlotTableHeader />
      {filtered.map((slot) => {
        const isErrorSlot =
          slotError?.slotIndex === slot.slot_index &&
          slotError?.slotType === slotType;
        return (
          <SlotRow
            key={slot.slot_index}
            slot={slot}
            slotType={slotType}
            crossRefs={crossRefMap.get(slot.slot_index)}
            healthIssues={healthIssues}
            onAssign={onAssign}
            onPlay={onPlay}
            playbackState={playbackState}
            isPlaying={activeSlotKey === `${slotType}-${slot.slot_index}`}
            assignError={isErrorSlot ? slotError!.message : null}
            assignErrorRedirect={
              isErrorSlot && slotErrorRedirect
                ? { label: slotErrorRedirect.label, onRedirect: onSlotRedirect }
                : isErrorSlot
                ? { label: "Dismiss", onRedirect: onDismissError }
                : null
            }
            onDismiss={isErrorSlot ? onDismissError : undefined}
          />
        );
      })}
    </div>
  );
}

/** Simple inline success banner for sample assignment (separate from backup InlineSuccessBanner) */
function AssignSuccessBanner({ message, onDismiss }: { message: string; onDismiss: () => void }) {
  return (
    <div className="flex items-center gap-2 px-4 py-2 bg-[hsl(140,30%,14%)] border-b border-[hsl(140,40%,28%)]">
      <CircleCheck className="h-4 w-4 shrink-0 text-[hsl(140,60%,72%)]" />
      <span className="font-mono text-xs text-[hsl(140,60%,72%)] flex-1">{message}</span>
      <button
        type="button"
        onClick={onDismiss}
        className="h-5 w-5 flex items-center justify-center text-[hsl(140,60%,72%)] hover:text-[hsl(140,60%,82%)]"
        aria-label="Dismiss"
      >
        <X className="h-[14px] w-[14px]" />
      </button>
    </div>
  );
}

export function SamplesTab({ projectId }: SamplesTabProps) {
  const [showEmpty, setShowEmpty] = useState(false);
  const [dryRunOpen, setDryRunOpen] = useState(false);
  const [isApplying, setIsApplying] = useState(false);
  const [pendingApplyLabel, setPendingApplyLabel] = useState<string>("Assign Sample");

  const queryClient = useQueryClient();
  const deviceConnected = useDeviceStore((s) => s.connected);

  const {
    dryRunManifest,
    softWarnings,
    successMessage,
    pendingSlotType,
    pendingSlotIndex,
    pendingFilePath,
    pendingFromWallflower,
    slotError,
    slotErrorRedirect,
    wallflowerConnected,
    slotPickerOpen,
    slotPickerSampleFilename,
    slotPickerSampleFilePath,
    setAssignStatus,
    setDryRunResult,
    setPendingAssign,
    setSlotError,
    clearSlotError,
    setSuccessMessage,
    setWallflowerConnected,
    openSlotPicker,
    closeSlotPicker,
    reset,
  } = useSamplesStore();

  // Audio preview hook
  const { play: playAudio, stop: stopAudio, playbackState: audioPlaybackState, activeSlotKey: audioActiveSlotKey } = useAudioPreview();

  // Stop audio playback on unmount (navigating away)
  useEffect(() => {
    return () => { stopAudio(); };
  }, [stopAudio]);

  // Check Wallflower connection status on mount (D-07: panel hidden when unavailable)
  useEffect(() => {
    getWallflowerStatus()
      .then((status) => {
        setWallflowerConnected(status.connected, status.db_path);
      })
      .catch(() => {
        setWallflowerConnected(false);
      });
  }, [setWallflowerConnected]);

  const { data: samples, isPending } = useQuery({
    queryKey: ["samples", projectId],
    queryFn: () => getProjectSamples(projectId),
    enabled: !!projectId,
  });

  // Read health data from react-query cache (populated by HealthEventListener)
  const { data: healthData } = useQuery<HealthCheckComplete | null>({
    queryKey: ["health", projectId],
    queryFn: () => Promise.resolve(null as any),
    enabled: false,
  });

  const banks = useCrossRefs(projectId);
  const crossRefMap = buildCrossRefMap(banks);

  const flexSlots = samples?.flex ?? [];
  const staticSlots = samples?.static_slots ?? [];

  // ── Audio preview handler ──
  function handlePlay(slotIndex: number, slotType: "flex" | "static") {
    const slotKey = `${slotType}-${slotIndex}`;
    const slots = slotType === "flex" ? flexSlots : staticSlots;
    const slot = slots[slotIndex];
    if (!slot?.full_path) return;
    playAudio(projectId, slot.full_path, slotKey);
  }

  // ── Assign flow handler (triggered by SlotRow assign button) ──
  // T-05-11: Guard against concurrent assigns — return early when not idle
  async function handleAssign(slotIndex: number, slotType: "flex" | "static") {
    const currentStatus = useSamplesStore.getState().assignStatus;
    if (currentStatus !== "idle") return;

    clearSlotError();
    setAssignStatus("picking-file");

    // Open native macOS file picker per D-01
    let filePath: string | null = null;
    try {
      const result = await open({
        multiple: false,
        filters: [{ name: "Audio", extensions: ["wav", "aif", "aiff"] }],
      });
      filePath = typeof result === "string" ? result : null;
    } catch {
      setAssignStatus("idle");
      return;
    }

    if (!filePath) {
      // User cancelled the file picker
      setAssignStatus("idle");
      return;
    }

    setAssignStatus("dry-running");

    try {
      const result: SampleDryRunResult = await computeSampleDryRun(
        projectId,
        slotType,
        slotIndex,
        filePath,
      );

      // D-14: Hard block — show inline error below the slot row, do NOT open dry-run modal
      if (result.hard_block) {
        setAssignStatus("idle");

        // D-13: Slot type mismatch — offer redirect to equivalent slot in correct type
        if (result.hard_block.toLowerCase().includes("flex")) {
          setSlotError(slotIndex, slotType, result.hard_block, {
            label: `Assign to Static #${String(slotIndex + 1).padStart(3, "0")}`,
            targetSlotType: "static",
            targetSlotIndex: slotIndex,
          });
        } else {
          // Format error — no redirect per UI-SPEC
          setSlotError(slotIndex, slotType, result.hard_block);
        }
        return;
      }

      // Determine apply button label based on whether slot is occupied
      const isOccupied =
        slotType === "flex"
          ? (flexSlots[slotIndex]?.occupied ?? false)
          : (staticSlots[slotIndex]?.occupied ?? false);
      setPendingApplyLabel(isOccupied ? "Replace Sample" : "Assign Sample");

      // Store dry-run result and open preview modal per D-03
      setDryRunResult(result.manifest, null, result.soft_warnings ?? []);
      setPendingAssign(slotType, slotIndex, filePath, false);
      setDryRunOpen(true);
    } catch (err) {
      setAssignStatus("failed");
      setSlotError(slotIndex, slotType, `Assignment failed: ${String(err)}`);
    }
  }

  // ── Apply handler (DryRunModal "Assign Sample" / "Replace Sample" button) ──
  async function handleApplyAssign() {
    if (!pendingSlotType || pendingSlotIndex === null || !pendingFilePath) return;

    setIsApplying(true);
    setAssignStatus("assigning");

    try {
      const result = await assignSample(
        projectId,
        pendingSlotType,
        pendingSlotIndex,
        pendingFilePath,
        pendingFromWallflower, // true for Wallflower push (copies file to /AUDIO/), false for desktop
      );

      setDryRunOpen(false);
      setIsApplying(false);

      // D-05: Success banner — distinct message for Wallflower push vs desktop assign
      const slotLabel = `${pendingSlotType === "flex" ? "Flex" : "Static"} #${String(pendingSlotIndex + 1).padStart(3, "0")}`;
      const bannerMessage = pendingFromWallflower
        ? `Pushed ${result.filename} to ${slotLabel} — ${result.files_written} files updated · copied to /AUDIO/`
        : `Assigned ${result.filename} to ${slotLabel} — ${result.files_written} files updated`;
      setSuccessMessage(bannerMessage);

      // Invalidate samples cache so the slot list refreshes
      queryClient.invalidateQueries({ queryKey: ["samples", projectId] });

      // Auto-dismiss success after 4 seconds
      setTimeout(() => reset(), 4000);
    } catch (err) {
      setIsApplying(false);
      setDryRunOpen(false);
      setAssignStatus("failed");
      if (pendingSlotIndex !== null && pendingSlotType) {
        setSlotError(
          pendingSlotIndex,
          pendingSlotType,
          `Assignment failed: ${String(err)}`,
        );
      }
    }
  }

  // ── Push-to-slot flow (Wallflower sample -> slot picker -> dry-run -> assign) ──
  function handlePushToSlot(sample: WallflowerSample) {
    openSlotPicker(sample.filename, sample.file_path);
  }

  async function handleSlotPickerConfirm(slotType: "flex" | "static", slotIndex: number) {
    if (!slotPickerSampleFilePath) return;
    closeSlotPicker();

    // Run through the same dry-run flow as desktop assignment, but with fromWallflower=true
    clearSlotError();
    setAssignStatus("dry-running");

    try {
      const result: SampleDryRunResult = await computeSampleDryRun(
        projectId,
        slotType,
        slotIndex,
        slotPickerSampleFilePath,
      );

      if (result.hard_block) {
        setAssignStatus("idle");
        setSlotError(slotIndex, slotType, result.hard_block);
        return;
      }

      // Determine apply button label based on whether slot is occupied
      const isOccupied =
        slotType === "flex"
          ? (flexSlots[slotIndex]?.occupied ?? false)
          : (staticSlots[slotIndex]?.occupied ?? false);
      setPendingApplyLabel(isOccupied ? "Replace Sample" : "Assign Sample");

      setDryRunResult(result.manifest, null, result.soft_warnings ?? []);
      setPendingAssign(slotType, slotIndex, slotPickerSampleFilePath, true); // fromWallflower = true
      setDryRunOpen(true);
    } catch (err) {
      setAssignStatus("failed");
    }
  }

  // ── Cancel handler ──
  function handleCancelAssign() {
    setDryRunOpen(false);
    setIsApplying(false);
    reset();
  }

  // ── Redirect handler for D-13 slot type mismatch ──
  function handleSlotRedirect() {
    if (!slotErrorRedirect) return;
    clearSlotError();
    handleAssign(slotErrorRedirect.targetSlotIndex, slotErrorRedirect.targetSlotType);
  }

  const hasAnyPopulated =
    flexSlots.some((s) => s.occupied) ||
    staticSlots.some((s) => s.occupied);

  if (!isPending && !hasAnyPopulated && !showEmpty) {
    return (
      <div className="flex flex-1 items-center justify-center p-8">
        <p className="text-sm text-muted-foreground">
          No samples assigned to any slot in this project.
        </p>
      </div>
    );
  }

  // onAssign only provided when device is connected (disables buttons otherwise)
  const assignHandler = deviceConnected ? handleAssign : undefined;

  return (
    <div className="flex flex-col">
      {/* D-05: Success banner — auto-dismissing after 4 seconds */}
      {successMessage && (
        <AssignSuccessBanner
          message={successMessage}
          onDismiss={() => reset()}
        />
      )}

      {/* Top-right: Show all slots toggle */}
      <div className="flex justify-end px-4 pt-3 pb-1">
        <Toggle
          pressed={showEmpty}
          onPressedChange={setShowEmpty}
          className="text-xs font-mono"
          aria-label={showEmpty ? "Hide empty slots" : "Show all slots"}
        >
          {showEmpty ? "Hide empty slots" : "Show all slots"}
        </Toggle>
      </div>

      <div className="px-4">
        {/* FLEX SAMPLES section */}
        <h2 className="mb-2 font-mono text-lg font-semibold text-foreground">
          FLEX SAMPLES
        </h2>
        <SlotSection
          slots={flexSlots}
          showEmpty={showEmpty}
          crossRefMap={crossRefMap}
          slotType="flex"
          healthIssues={healthData?.issues}
          onAssign={assignHandler}
          onPlay={handlePlay}
          activeSlotKey={audioActiveSlotKey}
          playbackState={audioPlaybackState}
          slotError={slotError}
          slotErrorRedirect={slotErrorRedirect}
          onSlotRedirect={handleSlotRedirect}
          onDismissError={clearSlotError}
        />

        {/* STATIC SAMPLES section */}
        <h2 className="mb-2 pt-6 font-mono text-lg font-semibold text-foreground">
          STATIC SAMPLES
        </h2>
        <SlotSection
          slots={staticSlots}
          showEmpty={showEmpty}
          crossRefMap={crossRefMap}
          slotType="static"
          healthIssues={healthData?.issues}
          onAssign={assignHandler}
          onPlay={handlePlay}
          activeSlotKey={audioActiveSlotKey}
          playbackState={audioPlaybackState}
          slotError={slotError}
          slotErrorRedirect={slotErrorRedirect}
          onSlotRedirect={handleSlotRedirect}
          onDismissError={clearSlotError}
        />
      </div>

      {/* D-07: Wallflower panel — hidden entirely when DB unavailable */}
      {wallflowerConnected && (
        <div className="px-4">
          <WallflowerPanel onPushToSlot={handlePushToSlot} />
        </div>
      )}

      {/* Dry-run preview modal per D-03, D-04 */}
      <DryRunModal
        open={dryRunOpen}
        manifest={dryRunManifest}
        onApply={handleApplyAssign}
        onCancel={handleCancelAssign}
        applyLabel={pendingApplyLabel}
        isApplying={isApplying}
        softWarnings={softWarnings}
      />

      {/* Slot Picker Dialog for push-to-slot flow per D-10 */}
      <SlotPickerDialog
        open={slotPickerOpen}
        sampleFilename={slotPickerSampleFilename ?? ""}
        slots={samples}
        onConfirm={handleSlotPickerConfirm}
        onCancel={closeSlotPicker}
      />
    </div>
  );
}
