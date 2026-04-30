"use client";

import { CircleX, CircleAlert, Info } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import type { HealthIssue } from "@/lib/types";

interface HealthSeverityGroupProps {
  severity: "error" | "warning" | "info";
  issues: HealthIssue[];
}

function SeverityHeader({
  severity,
  count,
}: {
  severity: "error" | "warning" | "info";
  count: number;
}) {
  if (severity === "error") {
    return (
      <div className="flex items-center gap-2">
        <CircleX
          size={16}
          className="text-[hsl(0,68%,48%)] shrink-0"
          aria-hidden="true"
        />
        <span className="font-mono text-sm font-semibold text-[hsl(0,68%,48%)]">
          Missing files
        </span>
        <Badge variant="secondary" className="text-xs">
          {count}
        </Badge>
      </div>
    );
  }

  if (severity === "warning") {
    return (
      <div className="flex items-center gap-2">
        <CircleAlert
          size={16}
          className="text-[hsl(38,85%,55%)] shrink-0"
          aria-hidden="true"
        />
        <span className="font-mono text-sm font-semibold text-[hsl(38,85%,55%)]">
          Format issues
        </span>
        <Badge variant="secondary" className="text-xs">
          {count}
        </Badge>
      </div>
    );
  }

  // info
  return (
    <div className="flex items-center gap-2">
      <Info
        size={16}
        className="text-[hsl(210,40%,52%)] shrink-0"
        aria-hidden="true"
      />
      <span className="font-mono text-sm font-semibold text-[hsl(210,40%,52%)]">
        Unused samples
      </span>
      <Badge variant="secondary" className="text-xs">
        {count}
      </Badge>
    </div>
  );
}

function IssueRow({
  issue,
  severity,
}: {
  issue: HealthIssue;
  severity: "error" | "warning" | "info";
}) {
  const borderColor =
    severity === "error"
      ? "border-[hsl(0,68%,48%)]"
      : severity === "warning"
        ? "border-[hsl(38,85%,55%)]"
        : "border-[hsl(210,40%,52%)]";

  let content: React.ReactNode;

  if (severity === "error") {
    content = (
      <span
        className="text-sm text-foreground"
        style={{ fontFamily: "var(--font-sans)", lineHeight: "1.6" }}
      >
        Missing:{" "}
        <code className="font-mono text-sm text-muted-foreground">
          {issue.path ?? issue.detail}
        </code>
      </span>
    );
  } else if (severity === "warning") {
    content = (
      <span
        className="text-sm text-foreground"
        style={{ fontFamily: "var(--font-sans)", lineHeight: "1.6" }}
      >
        <code className="font-mono text-sm text-muted-foreground">
          {issue.filename}
        </code>
        {" — "}
        {issue.detail}
      </span>
    );
  } else {
    content = (
      <span
        className="text-sm text-foreground"
        style={{ fontFamily: "var(--font-sans)", lineHeight: "1.6" }}
      >
        <code className="font-mono text-sm text-muted-foreground">
          {issue.filename}
        </code>
        {" (slot #"}
        {String(issue.slot_index + 1).padStart(3, "0")}
        {") — assigned but not referenced by any track."}
      </span>
    );
  }

  return (
    <div className={`border-l-2 pl-2 py-1 ${borderColor}`}>{content}</div>
  );
}

export function HealthSeverityGroup({
  severity,
  issues,
}: HealthSeverityGroupProps) {
  if (issues.length === 0) return null;

  return (
    <div className="mt-6">
      <SeverityHeader severity={severity} count={issues.length} />
      <div className="mt-2 space-y-2">
        {issues.map((issue, i) => (
          <IssueRow key={i} issue={issue} severity={severity} />
        ))}
      </div>
    </div>
  );
}
