<script lang="ts">
  import type { JobDto } from "$lib/types/workspace";

  interface Props {
    jobs: JobDto[];
    onDismiss: () => void;
  }

  let { jobs, onDismiss }: Props = $props();

  const successCount = $derived(jobs.filter((r) => r.status === "success").length);
  const failedCount = $derived(jobs.filter((r) => r.status === "failed").length);
  const skippedCount = $derived(jobs.filter((r) => r.status === "skipped").length);
</script>

<section class="results-panel">
  <div class="results-header">
    <h3 class="results-title">Execution Results</h3>
    <button class="btn-icon" type="button" onclick={onDismiss}>×</button>
  </div>
  <div class="results-summary">
    <span class="result-stat result-success">{successCount} succeeded</span>
    <span class="result-stat result-failed">{failedCount} failed</span>
    <span class="result-stat result-skipped">{skippedCount} skipped</span>
  </div>
  <div class="results-list">
    {#each jobs as job (job.id)}
      <div class="result-row">
        <span
          class="result-indicator"
          class:success={job.status === "success"}
          class:failed={job.status === "failed"}
          class:skipped={job.status === "skipped"}
        ></span>
        <span class="result-repo">{job.repo_name}</span>
        {#if job.error}
          <span class="result-error">{job.error}</span>
        {/if}
      </div>
    {/each}
  </div>
</section>

<style>
  .results-panel {
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-lg);
    background: var(--color-surface-card);
    margin-bottom: var(--space-lg);
    overflow: hidden;
  }

  .results-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-sm) var(--space-base);
    background: var(--color-canvas-soft);
    border-bottom: 1px solid var(--color-hairline);
  }

  .results-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-ink);
  }

  .results-summary {
    display: flex;
    gap: var(--space-base);
    padding: var(--space-sm) var(--space-base);
    border-bottom: 1px solid var(--color-hairline-soft);
    font-size: 13px;
  }

  .result-stat {
    font-weight: 500;
  }
  .result-success {
    color: var(--color-success);
  }
  .result-failed {
    color: var(--color-error);
  }
  .result-skipped {
    color: var(--color-muted);
  }

  .results-list {
    max-height: 300px;
    overflow-y: auto;
  }

  .result-row {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-xs) var(--space-base);
    border-bottom: 1px solid var(--color-hairline-soft);
    font-size: 13px;
  }

  .result-row:last-child {
    border-bottom: none;
  }

  .result-indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
    background: var(--color-muted);
  }

  .result-indicator.success {
    background: var(--color-success);
  }
  .result-indicator.failed {
    background: var(--color-error);
  }
  .result-indicator.skipped {
    background: var(--color-muted-soft);
  }

  .result-repo {
    font-weight: 500;
    color: var(--color-ink);
  }

  .result-error {
    color: var(--color-error);
    font-size: 12px;
    margin-left: auto;
    max-width: 300px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
