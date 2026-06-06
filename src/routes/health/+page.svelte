<script lang="ts">
  import type { WorkspaceHealthDto, RepositoryHealthDto } from "$lib/types/health";
  import { getWorkspaceHealth, getRepositoryHealth, refreshHealth } from "$lib/types/health";
  import { onMount } from "svelte";
  import { resolve } from "$app/paths";
  import { handleError, type HandledError } from "$lib/utils/error-handling";

  let health = $state<WorkspaceHealthDto | null>(null);
  let loading = $state(true);
  let refreshing = $state(false);
  let pageError = $state<HandledError | null>(null);
  let expandedRepo = $state<string | null>(null);
  let repoDetail = $state<RepositoryHealthDto | null>(null);
  let detailLoading = $state(false);

  onMount(() => {
    loadHealth();
  });

  async function loadHealth() {
    loading = true;
    pageError = null;
    try {
      health = await getWorkspaceHealth();
    } catch (e) {
      pageError = handleError(e);
    } finally {
      loading = false;
    }
  }

  async function handleRefresh() {
    refreshing = true;
    pageError = null;
    try {
      health = await refreshHealth();
    } catch (e) {
      pageError = handleError(e);
    } finally {
      refreshing = false;
    }
  }

  async function toggleRepo(repoId: string) {
    if (expandedRepo === repoId) {
      expandedRepo = null;
      repoDetail = null;
      return;
    }
    expandedRepo = repoId;
    detailLoading = true;
    try {
      repoDetail = await getRepositoryHealth(repoId);
    } catch {
      repoDetail = null;
    } finally {
      detailLoading = false;
    }
  }

  function scoreDisplay(score: number): string {
    if (score < 0) return "N/A";
    return `${Math.round(score)}%`;
  }

  function severityClass(severity: string): string {
    return `sev-${severity}`;
  }
</script>

<div class="health-page">
  <header class="page-header">
    <div>
      <h2 class="page-title">Workspace Health</h2>
      {#if health?.last_evaluated}
        <p class="page-subtitle">Last evaluated: {health.last_evaluated}</p>
      {:else}
        <p class="page-subtitle">Health status of your repositories</p>
      {/if}
    </div>
    <button class="btn-primary" type="button" onclick={handleRefresh} disabled={refreshing}>
      {refreshing ? "Evaluating…" : "Refresh Health"}
    </button>
  </header>

  {#if loading}
    <div class="empty-state">Evaluating workspace health…</div>
  {:else if pageError}
    <div class="empty-state error">
      {pageError.message}
      {#if pageError.hint}
        <p class="error-hint">{pageError.hint}</p>
      {/if}
    </div>
  {:else if health}
    <section class="stats-bar" aria-label="Health statistics">
      <div class="stat-card stat-score">
        <span class="stat-value">{scoreDisplay(health.score)}</span>
        <span class="stat-label">Health Score</span>
      </div>
      <div class="stat-card">
        <span class="stat-value">{health.total_repos}</span>
        <span class="stat-label">Total</span>
      </div>
      <div class="stat-card">
        <span class="stat-value stat-healthy">{health.healthy_count}</span>
        <span class="stat-label">Healthy</span>
      </div>
      <div class="stat-card">
        <span class="stat-value stat-warning">{health.warning_count}</span>
        <span class="stat-label">Warning</span>
      </div>
      <div class="stat-card">
        <span class="stat-value stat-critical">{health.critical_count}</span>
        <span class="stat-label">Critical</span>
      </div>
    </section>

    {#if health.repositories.length === 0}
      <div class="empty-state">No active repositories to evaluate.</div>
    {:else}
      <div class="repo-table-wrap">
        <table class="repo-table">
          <thead>
            <tr>
              <th>Status</th>
              <th>Repository</th>
              <th>Severity</th>
              <th>Checks</th>
            </tr>
          </thead>
          <tbody>
            {#each health.repositories as repo (repo.repo_id)}
              <tr
                class="repo-row"
                class:expanded={expandedRepo === repo.repo_id}
                onclick={() => toggleRepo(repo.repo_id)}
              >
                <td class="col-status">
                  <span
                    class="sev-dot {severityClass(repo.worst_severity)}"
                    aria-label={repo.worst_severity}
                  ></span>
                </td>
                <td class="col-name">
                  <a
                    href={resolve(`/repo/${repo.repo_id}`)}
                    class="repo-link"
                    onclick={(e) => e.stopPropagation()}
                  >
                    {repo.repo_name}
                  </a>
                </td>
                <td class="col-severity">{repo.worst_severity}</td>
                <td class="col-checks">{repo.checks.length} checks</td>
              </tr>
              {#if expandedRepo === repo.repo_id}
                <tr class="detail-row">
                  <td colspan="4">
                    {#if detailLoading}
                      <div class="detail-loading">Loading details…</div>
                    {:else if repoDetail}
                      <div class="check-list">
                        {#each repoDetail.checks as check (check.check_id)}
                          <div class="check-item">
                            <span class="sev-dot {severityClass(check.severity)}"></span>
                            <span class="check-id">{check.check_id}</span>
                            <span class="check-msg">{check.message}</span>
                          </div>
                        {/each}
                      </div>
                    {/if}
                  </td>
                </tr>
              {/if}
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  {/if}
</div>

<style>
  .health-page {
    padding: var(--space-xl);
    max-width: 1200px;
  }

  .page-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-lg);
    margin-bottom: var(--space-xl);
  }

  .stats-bar {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: var(--space-base);
    margin-bottom: var(--space-xl);
  }

  .stat-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-xxs);
    padding: var(--space-base) var(--space-md);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-lg);
    background: var(--color-surface-card);
  }

  .stat-score {
    border-color: var(--color-primary);
  }

  .stat-value {
    font-size: var(--text-3xl);
    font-weight: 400;
    color: var(--color-ink);
    letter-spacing: -0.03em;
    line-height: 1;
  }

  .stat-value.stat-healthy {
    color: var(--color-success);
  }
  .stat-value.stat-warning {
    color: var(--color-warning);
  }
  .stat-value.stat-critical {
    color: var(--color-error);
  }

  .stat-label {
    font-size: var(--text-caption);
    color: var(--color-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .repo-table-wrap {
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-lg);
    background: var(--color-surface-card);
    overflow: hidden;
  }

  .repo-table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--text-body);
  }

  .repo-table th {
    text-align: left;
    padding: var(--space-sm) var(--space-base);
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--color-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    border-bottom: 1px solid var(--color-hairline);
    background: var(--color-canvas-soft);
  }

  .repo-table td {
    padding: var(--space-sm) var(--space-base);
    border-bottom: 1px solid var(--color-hairline-soft);
    vertical-align: middle;
  }

  .repo-row {
    cursor: pointer;
    transition: background 0.1s ease;
  }

  .repo-row:hover {
    background: var(--color-hairline-soft);
  }

  .repo-row.expanded {
    background: var(--color-canvas-soft);
  }

  .col-status {
    width: 40px;
    text-align: center;
  }

  .sev-dot {
    display: inline-block;
    width: 10px;
    height: 10px;
    border-radius: 50%;
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

  .col-severity {
    text-transform: capitalize;
    font-size: var(--text-caption);
    color: var(--color-muted);
  }

  .col-checks {
    font-size: var(--text-caption);
    color: var(--color-muted);
  }

  .repo-link {
    color: var(--color-ink);
    font-weight: 500;
    text-decoration: none;
  }

  .repo-link:hover {
    color: var(--color-primary);
  }

  .detail-row td {
    padding: 0 var(--space-base) var(--space-base);
    background: var(--color-canvas-soft);
  }

  .check-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
    padding: var(--space-sm) 0;
  }

  .check-item {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-xs) var(--space-sm);
    border-radius: var(--radius-md);
    background: var(--color-surface-card);
    font-size: var(--text-caption);
  }

  .check-id {
    font-weight: 500;
    color: var(--color-ink);
    min-width: 80px;
  }

  .check-msg {
    color: var(--color-body);
  }

  .detail-loading {
    padding: var(--space-sm);
    font-size: var(--text-caption);
    color: var(--color-muted);
  }
</style>
