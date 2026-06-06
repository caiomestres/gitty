<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { RepoDto } from "$lib/types/workspace";

  let repos = $state<RepoDto[]>([]);
  let loading = $state(true);

  const total = $derived(repos.length);
  const active = $derived(repos.filter((r) => r.state === "active").length);
  const missing = $derived(repos.filter((r) => r.state === "missing").length);

  $effect(() => {
    loadRepos();
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
</style>
