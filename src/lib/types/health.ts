import { invoke } from "@tauri-apps/api/core";

export interface WorkspaceHealthDto {
  score: number | null;
  total_repos: number;
  critical_count: number;
  warning_count: number;
  healthy_count: number;
  repositories: RepositoryHealthDto[];
  last_evaluated: string | null;
}

export interface RepositoryHealthDto {
  repo_id: string;
  repo_name: string;
  checks: CheckResultDto[];
  worst_severity: "healthy" | "warning" | "critical";
}

export interface CheckResultDto {
  check_id: string;
  severity: "healthy" | "warning" | "critical";
  message: string;
}

export async function getWorkspaceHealth(): Promise<WorkspaceHealthDto> {
  return invoke<WorkspaceHealthDto>("get_workspace_health");
}

export async function getRepositoryHealth(repoId: string): Promise<RepositoryHealthDto> {
  return invoke<RepositoryHealthDto>("get_repository_health", { repoId });
}

export async function refreshHealth(): Promise<WorkspaceHealthDto> {
  return invoke<WorkspaceHealthDto>("refresh_health");
}
