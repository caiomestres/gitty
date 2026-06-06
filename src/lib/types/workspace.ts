import { addToast } from "$lib/stores/toast.svelte";

export interface RepoDto {
  id: string;
  path: string;
  name: string;
  state: string;
  group_id: string | null;
  tags: string[];
}

export interface ChangedFileDto {
  path: string;
  status: string;
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
  changed_files: ChangedFileDto[];
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
  hint?: string;
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
  retry?: { max_attempts: number; backoff_seconds: number } | null;
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

export interface HandledError {
  message: string;
  hint?: string;
  isTransient: boolean;
}

export function handleError(err: unknown): HandledError {
  let message: string;
  let hint: string | undefined;
  let code = "unknown";

  if (typeof err === "string") {
    message = err;
  } else if (err && typeof err === "object") {
    const e = err as Record<string, unknown>;
    message = (e.message as string) ?? String(err);
    hint = e.hint as string | undefined;
    code = (e.code as string) ?? "unknown";
  } else {
    message = String(err);
  }

  const isTransient =
    code === "lock_contention" ||
    (code === "git_error" && message.toLowerCase().includes("network"));

  if (isTransient) {
    addToast({ message, hint, severity: "error", dismissAfterMs: 5000 });
  }

  return { message, hint, isTransient };
}
