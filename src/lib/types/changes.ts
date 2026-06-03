import { invoke } from "@tauri-apps/api/core";

export interface ChangeEntryDto {
  commit_hash: string;
  author: string;
  date: string;
  subject: string;
  branch: string;
  repo_id: string;
  repo_name: string;
}

export interface ChangeGroupDto {
  key: string;
  entries: ChangeEntryDto[];
}

export interface GroupedChangesDto {
  groups: ChangeGroupDto[];
  total_commits: number;
}

export type TimeWindow = "day" | "week" | "month";
export type Grouping = "author" | "repository" | "branch";

export async function getChanges(
  window: TimeWindow,
  grouping: Grouping,
  allBranchesRepos: string[] = [],
): Promise<GroupedChangesDto> {
  return invoke<GroupedChangesDto>("get_changes", {
    window,
    grouping,
    allBranchesRepos,
  });
}
