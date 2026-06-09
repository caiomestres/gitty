<script lang="ts">
  import type { Snippet } from "svelte";
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import type { WorkspaceHealthDto } from "$lib/types/health";
  import { getWorkspaceHealth } from "$lib/types/health";
  import Mascot from "./Mascot.svelte";
  import SearchBar from "./SearchBar.svelte";

  interface Props {
    notification?: Snippet;
  }

  let { notification }: Props = $props();

  let health = $state<WorkspaceHealthDto | null>(null);
  let healthLoading = $state(true);

  const healthScore = $derived(health?.score ?? null);
  const healthSeverity = $derived.by(() => {
    if (!health) return "unknown";
    if (health.critical_count > 0) return "critical";
    if (health.warning_count > 0) return "warning";
    return "healthy";
  });

  onMount(() => {
    loadHealth();
    const interval = setInterval(loadHealth, 30000);
    return () => clearInterval(interval);
  });

  async function loadHealth() {
    try {
      health = await getWorkspaceHealth();
    } catch {
      health = null;
    } finally {
      healthLoading = false;
    }
  }

  function navigateToHealth() {
    goto(resolve("/health"));
  }
</script>

<header class="status-bar">
  <div class="status-left">
    <a href={resolve("/")} class="logo-link" title="Go to Dashboard">
      <Mascot size={20} />
      <h1 class="app-title">Gitty</h1>
    </a>

    <button
      type="button"
      class="health-indicator"
      class:health-unknown={healthLoading || !health}
      class:health-healthy={!healthLoading && healthSeverity === "healthy"}
      class:health-warning={!healthLoading && healthSeverity === "warning"}
      class:health-critical={!healthLoading && healthSeverity === "critical"}
      onclick={navigateToHealth}
      title={health
        ? `Workspace health: ${Math.round(healthScore ?? 0)}% — Click for details`
        : "Loading health status..."}
    >
      <span class="health-dot"></span>
      {#if healthLoading}
        <span class="health-score">—</span>
      {:else if healthScore !== null}
        <span class="health-score">{Math.round(healthScore)}%</span>
      {:else}
        <span class="health-score">—</span>
      {/if}
    </button>
  </div>

  <div class="status-center">
    <SearchBar />
  </div>

  <div class="status-right">
    {#if notification}
      {@render notification()}
    {/if}
  </div>
</header>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-sm) var(--space-lg);
    background: var(--color-canvas-soft);
    min-height: 48px;
    gap: var(--space-lg);
  }

  .status-left {
    display: flex;
    align-items: center;
    gap: var(--space-base);
    flex-shrink: 0;
  }

  .logo-link {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    text-decoration: none;
    color: inherit;
  }

  .app-title {
    font-size: var(--text-lg);
    font-weight: 600;
    letter-spacing: -0.02em;
    margin: 0;
  }

  .health-indicator {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-xs) var(--space-sm);
    background: var(--color-surface-card);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition:
      background 0.15s ease,
      border-color 0.15s ease;
  }

  .health-indicator:hover {
    background: var(--color-hairline-soft);
    border-color: var(--color-hairline-strong);
  }

  .health-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-muted-soft);
  }

  .health-healthy .health-dot {
    background: var(--color-success);
    box-shadow: 0 0 4px color-mix(in srgb, var(--color-success) 40%, transparent);
  }

  .health-warning .health-dot {
    background: var(--color-warning);
    box-shadow: 0 0 4px color-mix(in srgb, var(--color-warning) 40%, transparent);
  }

  .health-critical .health-dot {
    background: var(--color-error);
    box-shadow: 0 0 4px color-mix(in srgb, var(--color-error) 40%, transparent);
  }

  .health-score {
    font-size: var(--text-caption);
    font-weight: 600;
    color: var(--color-ink);
    min-width: 2em;
  }

  .status-center {
    flex: 1;
    display: flex;
    justify-content: center;
    max-width: 400px;
  }

  .status-right {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    flex-shrink: 0;
  }
</style>
