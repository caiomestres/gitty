export interface EnvironmentDto {
  name: string;
  url: string;
  health_path: string;
  enabled: boolean;
  interval_seconds: number;
}

export type LivenessStatusName = "up" | "down";

export interface LivenessResultDto {
  environment_name: string;
  status: LivenessStatusName;
  checked_at: string;
  response_time_ms: number | null;
  error: string | null;
}

export interface RepoLivenessDto {
  repo_id: string;
  results: LivenessResultDto[];
}

export interface DashboardLivenessDot {
  name: string;
  /** "up" | "down" | "gray" */
  status: "up" | "down" | "gray";
  response_time_ms: number | null;
}

export interface RepoDashboardLiveness {
  repo_id: string;
  dots: DashboardLivenessDot[];
}

export interface EndpointSuggestionDto {
  name: string;
  url: string;
  health_path: string;
  source_file: string;
  description: string;
}

import { invoke } from "@tauri-apps/api/core";

export async function discoverEndpoints(repoId: string): Promise<EndpointSuggestionDto[]> {
  return invoke<EndpointSuggestionDto[]>("discover_endpoints", { repoId });
}
