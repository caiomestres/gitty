import { invoke } from "@tauri-apps/api/core";
import type { Component } from "svelte";
import Sun from "@lucide/svelte/icons/sun";
import Moon from "@lucide/svelte/icons/moon";
import Trophy from "@lucide/svelte/icons/trophy";

export type ThemeId = "default" | "dark" | "world-cup-brasil";

export const THEME_IDS: ThemeId[] = ["default", "dark", "world-cup-brasil"];

export interface ThemeMeta {
  id: ThemeId;
  label: string;
  description: string;
  icon: Component;
}

export const THEMES: ThemeMeta[] = [
  {
    id: "default",
    label: "Light",
    description: "Warm cream canvas with editorial typography",
    icon: Sun,
  },
  {
    id: "dark",
    label: "Dark",
    description: "Deep navy canvas with high-contrast accents",
    icon: Moon,
  },
  {
    id: "world-cup-brasil",
    label: "Brasil",
    description: "Celebration of green, yellow, and football",
    icon: Trophy,
  },
];

const THEME_MAP = new Map<ThemeId, ThemeMeta>(THEMES.map((t) => [t.id, t]));

export function getThemeMeta(id: ThemeId): ThemeMeta {
  return THEME_MAP.get(id) ?? THEMES[0];
}

export async function getTheme(): Promise<ThemeId> {
  const theme = await invoke<string>("get_theme");
  return isValidTheme(theme) ? theme : "default";
}

export async function setTheme(theme: ThemeId): Promise<void> {
  await invoke("set_theme", { theme });
  applyTheme(theme);
}

export function applyTheme(theme: string): void {
  const validTheme = isValidTheme(theme) ? theme : "default";
  document.documentElement.dataset.theme = validTheme;
}

function isValidTheme(theme: string): theme is ThemeId {
  return THEME_IDS.includes(theme as ThemeId);
}
