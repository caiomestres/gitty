export interface RepoDto {
  id: string;
  path: string;
  name: string;
  state: string;
  group_id: string | null;
  tags: string[];
}

export interface RepoStatusDto {
  id: string;
  branch: string | null;
  detached: boolean;
  dirty: boolean;
  ahead: number;
  behind: number;
  head_summary: string | null;
  head_short_id: string | null;
  changed_files_count: number;
}

export interface ScanResultDto {
  found: number;
  new: number;
  relinked: number;
  existing: number;
  missing: number;
}

export interface OpResultDto {
  success: boolean;
  message: string;
}

export interface BulkResultDto {
  success_count: number;
  failed_count: number;
  skipped_count: number;
  details: RepoOpDto[];
}

export interface RepoOpDto {
  repo_name: string;
  repo_path: string;
  success: boolean;
  message: string;
}

export interface RepoWithStatus extends RepoDto {
  status?: RepoStatusDto;
  statusLoading?: boolean;
}

// --- M4 GUI DTOs ---

export interface ErrorDto {
  code: string;
  message: string;
}

export interface GroupDto {
  id: string;
  name: string;
  parent_id: string | null;
  repo_count: number;
}

export interface GroupTreeNodeDto {
  group: GroupDto;
  children: GroupTreeNodeDto[];
  repos: RepoDto[];
}

export interface TagDto {
  name: string;
  repo_count: number;
}

export interface MacroDto {
  id: string;
  name: string;
  steps: StepDto[];
  variables: Record<string, string>;
}

export interface StepDto {
  kind: StepKindDto;
  condition: string | null;
  rollback: StepDto | null;
  confirm: boolean;
}

export type StepKindDto =
  | { type: "git_op"; op: string; branch?: string }
  | { type: "shell"; command: string; label?: string };

export type SelectionDto =
  | { kind: "all" }
  | { kind: "group"; id: string }
  | { kind: "tag"; name: string }
  | { kind: "multiple"; ids: string[] };

export interface JobDto {
  id: string;
  repo_id: string;
  repo_name: string;
  status: string;
  error: string | null;
  step_results: StepResultDto[];
}

export interface StepResultDto {
  step_index: number;
  status: string;
  output: string | null;
}

export function errorMessage(err: unknown): string {
  if (typeof err === "string") return err;
  if (err && typeof err === "object" && "message" in err) return (err as ErrorDto).message;
  return String(err);
}
