<script lang="ts">
  import type { RepositoryHealthDto } from "$lib/types/health";

  interface Props {
    health: RepositoryHealthDto | null;
    loading: boolean;
    refreshing: boolean;
    onRefresh: () => void;
  }

  let { health, loading, refreshing, onRefresh }: Props = $props();
</script>

<section class="health-section">
  <div class="health-header">
    <h3 class="section-title">Health</h3>
    <button
      class="btn-secondary btn-sm"
      type="button"
      onclick={onRefresh}
      disabled={refreshing || loading}
    >
      {refreshing ? "Refreshing…" : "Refresh"}
    </button>
  </div>
  {#if loading}
    <div class="health-empty">Loading health data…</div>
  {:else if health && health.checks.length > 0}
    <div class="health-check-list">
      {#each health.checks as check (check.check_id)}
        <div class="health-check-item">
          <span class="sev-dot sev-{check.severity}"></span>
          <span class="check-id">{check.check_id}</span>
          <span class="check-msg">{check.message}</span>
        </div>
      {/each}
    </div>
  {:else}
    <div class="health-empty">
      <span>No health data</span>
      <button class="btn-secondary btn-sm" type="button" onclick={onRefresh} disabled={refreshing}>
        Refresh
      </button>
    </div>
  {/if}
</section>

<style>
  .health-section {
    margin-bottom: var(--space-xl);
  }

  .health-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-sm);
    margin-bottom: var(--space-sm);
  }

  .health-header .section-title {
    margin-bottom: 0;
  }

  .section-title {
    font-size: var(--text-body);
    font-weight: 600;
    color: var(--color-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    margin-bottom: var(--space-sm);
  }

  .health-empty {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-base);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-lg);
    background: var(--color-surface-card);
    font-size: var(--text-caption);
    color: var(--color-muted);
  }

  .health-check-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-lg);
    background: var(--color-surface-card);
    padding: var(--space-sm);
  }

  .health-check-item {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-xs) var(--space-sm);
    font-size: var(--text-caption);
  }

  .sev-dot {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .sev-dot.sev-healthy {
    background: var(--color-success);
  }

  .sev-dot.sev-warning {
    background: var(--color-warning);
  }

  .sev-dot.sev-critical {
    background: var(--color-error);
  }

  .check-id {
    font-weight: 500;
    color: var(--color-ink);
    min-width: 80px;
  }

  .check-msg {
    color: var(--color-body);
  }
</style>
