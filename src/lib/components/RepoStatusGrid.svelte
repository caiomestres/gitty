<script lang="ts">
  import type { RepoStatusDto } from "$lib/types/workspace";

  interface Props {
    status: RepoStatusDto;
    branchLabel: string;
  }

  let { status, branchLabel }: Props = $props();

  function trackingTooltip(): string {
    if (status.ahead === 0 && status.behind === 0) return "Local branch is in sync with upstream";
    const parts: string[] = [];
    if (status.ahead > 0) parts.push(`${status.ahead} ahead`);
    if (status.behind > 0) parts.push(`${status.behind} behind`);
    return `Local branch is ${parts.join(", ")} of upstream`;
  }
</script>

<div class="info-grid">
  <div class="info-card">
    <span class="info-label">Branch</span>
    <span class="info-value mono">{branchLabel}</span>
  </div>
  <div class="info-card">
    <span class="info-label">Status</span>
    <span class="info-value">
      {#if status.dirty}
        <span
          class="badge badge-dirty"
          title="This repository has uncommitted changes in the working tree">dirty</span
        >
      {:else}
        <span class="badge badge-clean" title="Working tree is clean — no uncommitted changes"
          >clean</span
        >
      {/if}
    </span>
  </div>
  <div class="info-card">
    <span class="info-label">Tracking</span>
    <span class="info-value" title={trackingTooltip()}>
      {#if status.ahead === 0 && status.behind === 0}
        Up to date
      {:else}
        {#if status.ahead > 0}↑{status.ahead}{/if}
        {#if status.behind > 0}↓{status.behind}{/if}
      {/if}
    </span>
  </div>
  <div class="info-card">
    <span class="info-label">Changed Files</span>
    <span class="info-value">{status.changed_files_count}</span>
  </div>
</div>

<style>
  .info-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: var(--space-base);
    margin-bottom: var(--space-xl);
  }

  .info-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-xxs);
    padding: var(--space-base);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-lg);
    background: var(--color-surface-card);
  }

  .info-label {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--color-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .info-value {
    font-size: var(--text-lg);
    color: var(--color-ink);
  }
</style>
