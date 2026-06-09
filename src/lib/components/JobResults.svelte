<script lang="ts">
  import type { JobDto, StepResultDto } from "$lib/types/workspace";
  import { SvelteSet } from "svelte/reactivity";
  import X from "@lucide/svelte/icons/x";

  interface Props {
    jobs: JobDto[];
    onDismiss: () => void;
  }

  let { jobs, onDismiss }: Props = $props();

  let collapsedSteps = new SvelteSet<string>();

  const successCount = $derived(jobs.filter((r) => r.status === "success").length);
  const failedCount = $derived(jobs.filter((r) => r.status === "failed").length);
  const skippedCount = $derived(jobs.filter((r) => r.status === "skipped").length);

  function stepKey(jobId: string, stepIndex: number): string {
    return `${jobId}:${stepIndex}`;
  }

  function toggleStep(jobId: string, stepIndex: number) {
    const key = stepKey(jobId, stepIndex);
    if (collapsedSteps.has(key)) {
      collapsedSteps.delete(key);
    } else {
      collapsedSteps.add(key);
    }
  }

  function isStepCollapsed(jobId: string, stepIndex: number): boolean {
    return collapsedSteps.has(stepKey(jobId, stepIndex));
  }

  function stepIcon(status: string): string {
    if (status === "success") return "✓";
    if (status === "failed") return "✗";
    return "⊘";
  }
</script>

{#snippet stepRow(step: StepResultDto, jobId: string)}
  <div class="step-row" class:step-failed={step.status === "failed"}>
    <button
      class="step-header"
      type="button"
      onclick={() => toggleStep(jobId, step.step_index)}
      aria-expanded={!isStepCollapsed(jobId, step.step_index)}
    >
      <span class="step-index">{step.step_index + 1}</span>
      <span
        class="step-icon"
        class:success={step.status === "success"}
        class:failed={step.status === "failed"}
      >
        {stepIcon(step.status)}
      </span>
      <span class="step-collapse-hint">
        {isStepCollapsed(jobId, step.step_index) ? "▸" : "▾"}
      </span>
    </button>
    {#if !isStepCollapsed(jobId, step.step_index)}
      <div class="step-body">
        {#if step.output}
          <pre class="step-output">{step.output}</pre>
        {:else}
          <span class="step-no-output">No output</span>
        {/if}
      </div>
    {/if}
  </div>
{/snippet}

<section class="results-panel">
  <div class="results-header">
    <h3 class="results-title">Execution Results</h3>
    <button class="btn-icon" type="button" onclick={onDismiss}><X size={16} /></button>
  </div>
  <div class="results-summary">
    <span class="result-stat result-success">{successCount} succeeded</span>
    <span class="result-stat result-failed">{failedCount} failed</span>
    <span class="result-stat result-skipped">{skippedCount} skipped</span>
  </div>
  <div class="results-list">
    {#each jobs as job (job.id)}
      <div class="result-entry">
        <div class="result-row" class:result-failed={job.status === "failed"}>
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
          {#if job.step_results.length > 1}
            <span class="step-count">{job.step_results.length} steps</span>
          {/if}
        </div>

        <div class="step-results">
          {#each job.step_results as step (step.step_index)}
            {@render stepRow(step, job.id)}
          {/each}
        </div>
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
    font-size: var(--text-body);
    font-weight: 600;
    color: var(--color-ink);
  }

  .results-summary {
    display: flex;
    gap: var(--space-base);
    padding: var(--space-sm) var(--space-base);
    border-bottom: 1px solid var(--color-hairline-soft);
    font-size: var(--text-caption);
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
    max-height: 400px;
    overflow-y: auto;
  }

  .result-entry {
    border-bottom: 1px solid var(--color-hairline-soft);
  }

  .result-entry:last-child {
    border-bottom: none;
  }

  .result-row {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    width: 100%;
    padding: var(--space-xs) var(--space-base);
    font-size: var(--text-caption);
    text-align: left;
  }

  .result-row.result-failed {
    background: color-mix(in srgb, var(--color-error) 6%, transparent);
    border-left: 3px solid var(--color-error);
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
    font-size: var(--text-sm);
    margin-left: auto;
    max-width: 300px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .step-count {
    margin-left: auto;
    font-size: var(--text-2xs);
    color: var(--color-muted);
    flex-shrink: 0;
  }

  .step-results {
    padding: var(--space-xxs) var(--space-base) var(--space-sm);
    padding-left: calc(var(--space-base) + 16px);
    background: var(--color-canvas-soft);
  }

  .step-row {
    border-radius: var(--radius-sm);
    margin-bottom: var(--space-xxs);
    font-size: var(--text-sm);
  }

  .step-row.step-failed {
    background: color-mix(in srgb, var(--color-error) 8%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-error) 25%, transparent);
  }

  .step-header {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-xxs) var(--space-xs);
    border: none;
    background: none;
    cursor: pointer;
    width: 100%;
    text-align: left;
    border-radius: var(--radius-sm);
    transition: background 0.1s ease;
  }

  .step-header:hover {
    background: color-mix(in srgb, var(--color-ink) 4%, transparent);
  }

  .step-index {
    min-width: 16px;
    color: var(--color-muted);
    font-family: var(--font-mono);
  }

  .step-icon {
    flex-shrink: 0;
    width: 14px;
    text-align: center;
    font-weight: 600;
  }

  .step-icon.success {
    color: var(--color-success);
  }

  .step-icon.failed {
    color: var(--color-error);
  }

  .step-collapse-hint {
    margin-left: auto;
    font-size: var(--text-2xs);
    color: var(--color-muted);
  }

  .step-body {
    padding: var(--space-xxs) var(--space-xs) var(--space-xs);
    padding-left: calc(var(--space-xs) + 16px + var(--space-sm));
  }

  .step-output {
    margin: 0;
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    white-space: pre-wrap;
    word-break: break-word;
    color: var(--color-body);
  }

  .step-no-output {
    color: var(--color-muted-soft);
    font-style: italic;
  }
</style>
