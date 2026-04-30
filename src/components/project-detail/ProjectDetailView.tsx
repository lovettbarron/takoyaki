"use client";

import { useEffect } from "react";
import { useQuery } from "@tanstack/react-query";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Skeleton } from "@/components/ui/skeleton";
import { Badge } from "@/components/ui/badge";
import { useNavigationStore } from "@/lib/stores/navigation";
import { getProjectDetail, runHealthCheck } from "@/lib/tauri";
import type { HealthCheckComplete } from "@/lib/types";
import { MetadataHeader } from "./MetadataHeader";
import { BanksTab } from "./BanksTab";
import { SamplesTab } from "./SamplesTab";
import { HealthTab } from "./HealthTab";

interface ProjectDetailViewProps {
  onBackUp?: () => void;
}

export function ProjectDetailView({ onBackUp }: ProjectDetailViewProps = {}) {
  const {
    selectedProjectId,
    selectedBankIndex,
    activeTab,
    setActiveTab,
    navigateToList,
  } = useNavigationStore();

  const { data: project, isPending } = useQuery({
    queryKey: ["project", selectedProjectId],
    queryFn: () => getProjectDetail(selectedProjectId!),
    enabled: selectedProjectId !== null,
  });

  // Read health data from react-query cache (populated by HealthEventListener)
  const { data: healthData } = useQuery<HealthCheckComplete | null>({
    queryKey: ["health", selectedProjectId],
    queryFn: () => Promise.resolve(null),
    enabled: false, // never fetches — only reads what HealthEventListener sets via setQueryData
  });

  // Trigger health check on project open (D-11)
  useEffect(() => {
    if (selectedProjectId) {
      runHealthCheck(selectedProjectId).catch(() => {});
    }
  }, [selectedProjectId]);

  const issueCount = healthData?.issues?.length ?? 0;

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      {/* Breadcrumb */}
      <div className="px-4 py-2">
        <Breadcrumb>
          <BreadcrumbList>
            <BreadcrumbItem>
              <BreadcrumbLink
                className="cursor-pointer text-[hsl(38,85%,55%)] hover:text-[hsl(38,85%,65%)]"
                onClick={() => navigateToList()}
              >
                Projects
              </BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
              {selectedBankIndex !== null ? (
                <BreadcrumbLink className="text-muted-foreground">
                  {project?.project_name ?? "Loading…"}
                </BreadcrumbLink>
              ) : (
                <BreadcrumbPage className="text-muted-foreground">
                  {project?.project_name ?? "Loading…"}
                </BreadcrumbPage>
              )}
            </BreadcrumbItem>
            {selectedBankIndex !== null && (
              <>
                <BreadcrumbSeparator />
                <BreadcrumbItem>
                  <BreadcrumbPage className="text-muted-foreground">
                    Bank {selectedBankIndex + 1}
                  </BreadcrumbPage>
                </BreadcrumbItem>
              </>
            )}
          </BreadcrumbList>
        </Breadcrumb>
      </div>

      {/* Metadata Header */}
      {isPending ? (
        <div className="flex h-12 items-center justify-between border-b border-[hsl(30,8%,26%)] px-4 py-2">
          <Skeleton className="h-6 w-48" />
          <Skeleton className="h-4 w-64" />
        </div>
      ) : project ? (
        <MetadataHeader project={project} onBackUp={onBackUp} />
      ) : null}

      {/* Tabs */}
      <Tabs
        value={activeTab}
        onValueChange={(v) =>
          setActiveTab(v as "banks" | "samples" | "health")
        }
        className="flex flex-1 flex-col overflow-hidden"
      >
        <TabsList className="h-10 justify-start rounded-none border-b border-[hsl(30,8%,26%)] bg-transparent px-4">
          <TabsTrigger
            value="banks"
            className="data-[state=active]:border-b-2 data-[state=active]:border-[hsl(38,85%,55%)] data-[state=active]:text-foreground rounded-none bg-transparent text-muted-foreground"
          >
            Banks
          </TabsTrigger>
          <TabsTrigger
            value="samples"
            className="data-[state=active]:border-b-2 data-[state=active]:border-[hsl(38,85%,55%)] data-[state=active]:text-foreground rounded-none bg-transparent text-muted-foreground"
          >
            Samples
          </TabsTrigger>
          <TabsTrigger
            value="health"
            className="data-[state=active]:border-b-2 data-[state=active]:border-[hsl(38,85%,55%)] data-[state=active]:text-foreground rounded-none bg-transparent text-muted-foreground"
          >
            Health
            {issueCount > 0 && (
              <Badge variant="secondary" className="ml-1.5 text-xs">
                {issueCount}
              </Badge>
            )}
          </TabsTrigger>
        </TabsList>

        {/* Banks Tab Content */}
        <TabsContent
          value="banks"
          className="flex-1 overflow-auto p-4 mt-0"
        >
          {isPending ? (
            <div className="space-y-3">
              <Skeleton className="h-48 w-48" />
              <Skeleton className="h-4 w-32" />
            </div>
          ) : project ? (
            <BanksTab project={project} />
          ) : null}
        </TabsContent>

        {/* Samples Tab Content */}
        <TabsContent
          value="samples"
          className="flex-1 overflow-auto mt-0"
        >
          {isPending ? (
            <div className="space-y-3 p-4">
              <Skeleton className="h-8 w-full" />
              <Skeleton className="h-8 w-full" />
              <Skeleton className="h-8 w-full" />
            </div>
          ) : selectedProjectId ? (
            <SamplesTab projectId={selectedProjectId} />
          ) : null}
        </TabsContent>

        {/* Health Tab Content */}
        <TabsContent
          value="health"
          className="flex-1 overflow-auto p-4 mt-0"
        >
          {selectedProjectId && (
            <HealthTab projectId={selectedProjectId} />
          )}
        </TabsContent>
      </Tabs>
    </div>
  );
}
