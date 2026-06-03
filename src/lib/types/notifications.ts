import { invoke } from "@tauri-apps/api/core";

export interface NotificationDto {
  id: string;
  timestamp: string;
  severity: "info" | "warning" | "critical";
  title: string;
  body: string;
  read: boolean;
}

export interface NotificationConfigDto {
  trigger: "on_critical" | "on_any_change" | "on_scheduler_complete" | "disabled";
  polling_interval_minutes: number | null;
}

export async function getNotifications(): Promise<NotificationDto[]> {
  return invoke<NotificationDto[]>("get_notifications");
}

export async function markNotificationRead(id: string): Promise<void> {
  return invoke("mark_notification_read", { id });
}

export async function getNotificationConfig(): Promise<NotificationConfigDto> {
  return invoke<NotificationConfigDto>("get_notification_config");
}

export async function setNotificationConfig(config: NotificationConfigDto): Promise<void> {
  return invoke("set_notification_config", { config });
}
