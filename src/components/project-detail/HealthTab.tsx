"use client";

import { useQuery } from "@tanstack/react-query";
import { CircleCheck } from "lucide-react";
import { Progress } from "@/components/ui/progress";
import { HealthSeverityGroup } from "@/components/health/HealthSeverityGroup";
import type { HealthCheckComplete } from "@/lib/types";

interface HealthTabProps {
  projectId: string;
}

function formatScanTimestamp(scannedAt: string): string {
  try {
    const date = new Date(scannedAt);
    return date.toLocaleString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return scannedAt;
  }
}

export function HealthTab({ projectId }: HealthTabProps) {
  const { data: healthData } = useQuery<HealthCheckComplete | null>({
    queryKey: ["health", projectId],
    queryFn: () => Promise.resolve(null as any),
    enabled: false, // never fetches — only reads what HealthEventListener populates
  });

  // Scanning state — healthData not yet available
  if (!healthData) {
    return (
      <div className="flex flex-col gap-3 pt-8 px-2 max-w-md">
        <Progress value={null} className="h-1.5" />
        <span className="font-mono text-sm text-muted-foreground">
          Scanning project...
        </span>
      </div>
    );
  }

  const errors = healthData.issues.filter((i) => i.severity === "error");
  const warnings = healthData.issues.filter((i) => i.severity === "warning");
  const infos = healthData.issues.filter((i) => i.severity === "info");

  // All clear state
  if (healthData.issues.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-16 gap-3">
        <CircleCheck
          size={48}
          className="text-[hsl(140,60%,42%)]"
          aria-hidden="true"
        />
        <h2 className="font-mono text-2xl font-semibold text-[hsl(140,60%,42%)]">
          All clear
        </h2>
        <p className="text-sm text-muted-foreground">
          No issues found. Last scanned{" "}
          {formatScanTimestamp(healthData.scanned_at)}.
        </p>
      </div>
    );
  }

  // Issues found — group by severity: Errors first, then Warnings, then Info
  return (
    <div className="px-2">
      <HealthSeverityGroup severity="error" issues={errors} />
      <HealthSeverityGroup severity="warning" issues={warnings} />
      <HealthSeverityGroup severity="info" issues={infos} />
    </div>
  );
}
