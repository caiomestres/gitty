<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import type { RepoDto } from "$lib/types/workspace";
  import { getTheme, setTheme, getThemeMeta, type ThemeId, THEME_IDS } from "$lib/utils/theme";

  let repos = $state<RepoDto[]>([]);
  let loading = $state(true);
  let currentTheme = $state<ThemeId>("default");

  const total = $derived(repos.length);
  const active = $derived(repos.filter((r) => r.state === "active").length);
  const missing = $derived(repos.filter((r) => r.state === "missing").length);

  onMount(() => {
    loadRepos();
    loadTheme();
  });

  async function loadRepos() {
    loading = true;
    try {
      repos = await invoke<RepoDto[]>("list_repositories");
    } catch {
      repos = [];
    } finally {
      loading = false;
    }
  }

  async function loadTheme() {
    try {
      currentTheme = await getTheme();
    } catch {
      currentTheme = "default";
    }
  }

  function cycleTheme() {
    const currentIndex = THEME_IDS.indexOf(currentTheme);
    const nextIndex = (currentIndex + 1) % THEME_IDS.length;
    const nextTheme = THEME_IDS[nextIndex];
    setTheme(nextTheme);
    currentTheme = nextTheme;
  }

  const themeMeta = $derived(getThemeMeta(currentTheme));
  const ThemeIcon = $derived(themeMeta.icon);
</script>

<footer class="bottom-bar">
  <div class="bottom-stats">
    {#if loading}
      <span class="stat">Loading workspace…</span>
    {:else}
      <span class="stat">
        <strong>{total}</strong>
        {total === 1 ? "repo" : "repos"}
      </span>
      <span class="divider" aria-hidden="true">·</span>
      <span class="stat stat-active">
        <strong>{active}</strong> active
      </span>
      {#if missing > 0}
        <span class="divider" aria-hidden="true">·</span>
        <span class="stat stat-missing">
          <strong>{missing}</strong> missing
        </span>
      {/if}
    {/if}
  </div>

  <div class="bottom-health">
    {#if !loading && total === 0}
      <span class="health-hint">Scan a directory to get started</span>
    {:else if !loading}
      <span class="health-ok">Workspace healthy</span>
    {/if}
  </div>

  <button
    class="theme-toggle"
    type="button"
    onclick={cycleTheme}
    title={`Theme: ${themeMeta.label} (click to cycle)`}
  >
    <ThemeIcon size={14} />
    <span class="theme-label">{themeMeta.label}</span>
  </button>
</footer>

<style>
  .bottom-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-xs) var(--space-lg);
    background: var(--color-canvas-soft);
    min-height: 36px;
    font-size: var(--text-caption);
    color: var(--color-muted);
  }

  .bottom-stats {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
  }

  .stat strong {
    color: var(--color-ink);
    font-weight: 600;
  }

  .stat-active strong {
    color: var(--color-success);
  }

  .stat-missing strong {
    color: var(--color-error);
  }

  .divider {
    color: var(--color-hairline-strong);
  }

  .health-ok {
    color: var(--color-success);
  }

  .health-hint {
    color: var(--color-muted-soft);
  }

  .theme-toggle {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-xs) var(--space-sm);
    background: transparent;
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-md);
    color: var(--color-muted);
    font-size: var(--text-sm);
    cursor: pointer;
    transition:
      background 0.15s ease,
      border-color 0.15s ease,
      color 0.15s ease;
  }

  .theme-toggle:hover {
    background: var(--color-hairline-soft);
    border-color: var(--color-hairline-strong);
    color: var(--color-ink);
  }

  .theme-label {
    font-size: var(--text-sm);
  }
</style>
