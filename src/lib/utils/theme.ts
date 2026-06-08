import { invoke } from "@tauri-apps/api/core";

export async function getTheme(): Promise<string> {
  return invoke<string>("get_theme");
}

export async function setTheme(theme: string): Promise<void> {
  await invoke("set_theme", { theme });
  applyTheme(theme);
}

export function applyTheme(theme: string): void {
  document.documentElement.dataset.theme = theme;
}
