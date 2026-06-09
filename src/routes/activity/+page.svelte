<script lang="ts">
  import {
    getActivityLog,
    clearActivityLog,
    OPERATION_LABELS,
    ALL_OPERATIONS,
    type ActivityEntryDto,
    type OperationType,
  } from "$lib/types/activity";
  import { handleError, type ActionFeedback } from "$lib/utils/error-handling";
  import PageError from "$lib/components/PageError.svelte";
  import { onMount } from "svelte";

  let entries = $state<ActivityEntryDto[]>([]);
  let loading = $state(true);
  let pageError = $state<ActionFeedback | null>(null);

  let filterOp = $state<OperationType | "">("");
  let filterTarget = $state("");
  let filterDateFrom = $state("");
  let filterDateTo = $state("");

  onMount(() => {
    loadLog();
  });

  async function loadLog() {
    loading = true;
    pageError = null;
    try {
      entries = await getActivityLog();
    } catch (e) {
      pageError = handleError(e);
    } finally {
      loading = false;
    }
  }

  async function handleClear() {
    try {
      await clearActivityLog();
      entries = [];
    } catch (e) {
      pageError = handleError(e);
    }
  }

  function formatDate(iso: string): string {
    try {
      const d = new Date(iso);
      return d.toLocaleDateString(undefined, {
        month: "short",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
      });
    } catch {
      return iso;
    }
  }

  function formatDuration(ms: number | undefined): string {
    if (ms === undefined) return "—";
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(1)}s`;
  }

  const filtered = $derived.by(() => {
    let result = [...entries].reverse();

    if (filterOp) {
      result = result.filter((e) => e.operation === filterOp);
    }

    if (filterTarget.trim()) {
      const q = filterTarget.trim().toLowerCase();
      result = result.filter((e) => e.target?.toLowerCase().includes(q));
    }

    if (filterDateFrom) {
      const from = new Date(filterDateFrom).getTime();
      result = result.filter((e) => new Date(e.timestamp).getTime() >= from);
    }

    if (filterDateTo) {
      const toTime = new Date(filterDateTo).getTime() + 24 * 60 * 60 * 1000;
      result = result.filter((e) => new Date(e.timestamp).getTime() < toTime);
    }

    return result;
  });
</script>

<div class="activity-page">
  <header class="page-header">
    <div>
      <h2 class="page-title">Activity</h2>
      <p class="page-subtitle">
        {#if entries.length > 0}
          {entries.length} operations recorded
        {:else}
          Operation history across your workspace
        {/if}
      </p>
    </div>
    {#if entries.length > 0}
      <button class="btn-clear" type="button" onclick={handleClear}>Clear log</button>
    {/if}
  </header>

  <div class="controls-bar">
    <div class="control-group">
      <label class="control-label" for="filter-op">Type</label>
      <select id="filter-op" class="control-select" bind:value={filterOp}>
        <option value="">All</option>
        {#each ALL_OPERATIONS as op (op)}
          <option value={op}>{OPERATION_LABELS[op]}</option>
        {/each}
      </select>
    </div>

    <div class="control-group">
      <label class="control-label" for="filter-target">Repository</label>
      <input
        id="filter-target"
        class="control-input"
        type="text"
        placeholder="Filter by name…"
        bind:value={filterTarget}
      />
    </div>

    <div class="control-group">
      <label class="control-label" for="filter-from">From</label>
      <input id="filter-from" class="control-input" type="date" bind:value={filterDateFrom} />
    </div>

    <div class="control-group">
      <label class="control-label" for="filter-to">To</label>
      <input id="filter-to" class="control-input" type="date" bind:value={filterDateTo} />
    </div>
  </div>

  {#if loading}
    <div class="empty-state">Loading activity log…</div>
  {:else if pageError}
    <PageError error={pageError} />
  {:else if entries.length === 0}
    <div class="empty-state">
      <p>No activity recorded yet.</p>
      <p class="empty-hint">Operations like scans, fetches, and macro runs will appear here.</p>
    </div>
  {:else if filtered.length === 0}
    <div class="empty-state">
      <p>No entries match the current filters.</p>
    </div>
  {:else}
    <div class="log-table-wrap">
      <table class="log-table">
        <thead>
          <tr>
            <th>Time</th>
            <th>Operation</th>
            <th>Target</th>
            <th>Details</th>
            <th>Duration</th>
            <th>Status</th>
          </tr>
        </thead>
        <tbody>
          {#each filtered as entry, i (i)}
            <tr class:has-error={!!entry.error}>
              <td class="cell-time mono">{formatDate(entry.timestamp)}</td>
              <td class="cell-op">
                <span class="op-badge">{OPERATION_LABELS[entry.operation]}</span>
              </td>
              <td class="cell-target">{entry.target ?? "—"}</td>
              <td class="cell-details">{entry.details ?? "—"}</td>
              <td class="cell-duration mono">{formatDuration(entry.duration_ms)}</td>
              <td class="cell-status">
                {#if entry.error}
                  <span class="status-error" title={entry.error}>Error</span>
                {:else}
                  <span class="status-ok">OK</span>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<style>
  .activity-page {
    padding: var(--space-xl);
    max-width: 1200px;
  }

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: var(--space-lg);
  }

  .btn-clear {
    padding: var(--space-xs) var(--space-sm);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-md);
    background: var(--color-surface-card);
    color: var(--color-muted);
    font-size: var(--text-caption);
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .btn-clear:hover {
    background: var(--color-hairline-soft);
    color: var(--color-ink);
  }

  .controls-bar {
    display: flex;
    gap: var(--space-lg);
    margin-bottom: var(--space-xl);
    flex-wrap: wrap;
  }

  .control-group {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .control-label {
    font-size: var(--text-caption);
    color: var(--color-muted);
    white-space: nowrap;
  }

  .control-select,
  .control-input {
    padding: var(--space-xs) var(--space-sm);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-md);
    background: var(--color-surface-card);
    color: var(--color-ink);
    font-size: var(--text-caption);
    font-family: inherit;
  }

  .control-input {
    width: 140px;
  }

  .control-select:focus,
  .control-input:focus {
    outline: 2px solid var(--color-primary);
    outline-offset: -1px;
  }

  .log-table-wrap {
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-lg);
    background: var(--color-surface-card);
    overflow: hidden;
  }

  .log-table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--text-caption);
  }

  .log-table th {
    text-align: left;
    padding: var(--space-sm) var(--space-base);
    background: var(--color-canvas-soft);
    border-bottom: 1px solid var(--color-hairline);
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--color-muted);
    white-space: nowrap;
  }

  .log-table td {
    padding: var(--space-xs) var(--space-base);
    border-bottom: 1px solid var(--color-hairline-soft);
    color: var(--color-body);
    vertical-align: top;
  }

  .log-table tr:last-child td {
    border-bottom: none;
  }

  .log-table tr.has-error td {
    background: color-mix(in srgb, var(--color-critical) 5%, transparent);
  }

  .cell-time {
    white-space: nowrap;
    color: var(--color-muted);
    font-size: var(--text-sm);
  }

  .cell-target,
  .cell-details {
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .cell-duration {
    white-space: nowrap;
    color: var(--color-muted);
    font-size: var(--text-sm);
  }

  .op-badge {
    display: inline-block;
    padding: 1px var(--space-xs);
    border-radius: var(--radius-sm);
    background: var(--color-hairline-soft);
    color: var(--color-ink);
    font-size: var(--text-sm);
    white-space: nowrap;
  }

  .status-ok {
    color: var(--color-ok);
    font-size: var(--text-sm);
    font-weight: 500;
  }

  .status-error {
    color: var(--color-critical);
    font-size: var(--text-sm);
    font-weight: 500;
    cursor: help;
  }

  .empty-hint {
    font-size: var(--text-sm);
    color: var(--color-muted-soft);
    margin-top: var(--space-xs);
  }
</style>
