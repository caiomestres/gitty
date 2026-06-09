import { invoke } from "@tauri-apps/api/core";

export type OperationType =
  | "scan"
  | "fetch"
  | "pull"
  | "checkout"
  | "unregister"
  | "macro_run"
  | "scheduler_run"
  | "liveness_check"
  | "health_evaluation"
  | "config_change";

export interface ActivityEntryDto {
  timestamp: string;
  operation: OperationType;
  target?: string;
  details?: string;
  duration_ms?: number;
  error?: string;
}

export const OPERATION_LABELS: Record<OperationType, string> = {
  scan: "Scan",
  fetch: "Fetch",
  pull: "Pull",
  checkout: "Checkout",
  unregister: "Unregister",
  macro_run: "Macro Run",
  scheduler_run: "Scheduler Run",
  liveness_check: "Liveness Check",
  health_evaluation: "Health Evaluation",
  config_change: "Config Change",
};

export const ALL_OPERATIONS: OperationType[] = [
  "scan",
  "fetch",
  "pull",
  "checkout",
  "unregister",
  "macro_run",
  "scheduler_run",
  "liveness_check",
  "health_evaluation",
  "config_change",
];

export async function getActivityLog(): Promise<ActivityEntryDto[]> {
  return invoke<ActivityEntryDto[]>("get_activity_log");
}

export async function clearActivityLog(): Promise<void> {
  return invoke("clear_activity_log");
}
