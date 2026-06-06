export type SchedulerMode = "simple" | "advanced";

export interface TimeOfDay {
  hour: number;
  minute: number;
}

export interface SchedulerConfigDto {
  enabled: boolean;
  macro_id: string | null;
  trigger:
    | { mode: "simple"; interval_minutes: number }
    | {
        mode: "advanced";
        interval_minutes: number;
        window_start: TimeOfDay;
        window_end: TimeOfDay;
        days: string[];
      };
  power: { pause_on_battery: boolean; battery_threshold: number };
}

export const DAY_OPTIONS = [
  { key: "mon", label: "Mon" },
  { key: "tue", label: "Tue" },
  { key: "wed", label: "Wed" },
  { key: "thu", label: "Thu" },
  { key: "fri", label: "Fri" },
  { key: "sat", label: "Sat" },
  { key: "sun", label: "Sun" },
] as const;

export function parseTime(value: string): TimeOfDay | null {
  const match = /^(\d{1,2}):(\d{2})$/.exec(value.trim());
  if (!match) return null;
  const hour = Number(match[1]);
  const minute = Number(match[2]);
  if (hour > 23 || minute > 59) return null;
  return { hour, minute };
}

export function formatTime(t: TimeOfDay): string {
  return `${String(t.hour).padStart(2, "0")}:${String(t.minute).padStart(2, "0")}`;
}
