<script lang="ts">
  import { resolve } from "$app/paths";
  import { invoke } from "@tauri-apps/api/core";
  import type {
    BulkResultDto,
    RepoDto,
    RepoStatusDto,
    RepoWithStatus,
    TagDto,
  } from "$lib/types/workspace";
  import { handleError, type HandledError } from "$lib/utils/error-handling";
  import ErrorBanner from "$lib/components/ErrorBanner.svelte";

  let repos = $state<RepoWithStatus[]>([]);
  let loading = $state(true);
  let pageError = $state<HandledError | null>(null);
  let showScanDialog = $state(false);
  let scanPath = $state("");
  let scanning = $state(false);
  let fetchingAll = $state(false);
  let actionFeedback = $state<HandledError | null>(null);

  // Tag filter
  let allTags = $state<TagDto[]>([]);
  let selectedTag = $state("");

  const filteredRepos = $derived(
    selectedTag ? repos.filter((r) => r.tags.includes(selectedTag)) : repos,
  );
  const total = $derived(repos.length);
  const activeCount = $derived(repos.filter((r) => r.state === "active").length);
  const missingCount = $derived(repos.filter((r) => r.state === "missing").length);
  const dirtyCount = $derived(repos.filter((r) => r.status?.dirty).length);

  $effect(() => {
    loadWorkspace();
  });

  async function loadWorkspace() {
    loading = true;
    pageError = null;
    try {
      const list = await invoke<RepoDto[]>("list_repositories");
      repos = list.map((r) => ({ ...r, statusLoading: r.state === "active" }));
      allTags = await invoke<TagDto[]>("list_tags");
      await loadStatuses();
    } catch (e) {
      pageError = handleError(e);
    } finally {
      loading = false;
    }
  }

  async function loadStatuses() {
    const activeRepos = repos.filter((r) => r.state === "active");
    await Promise.all(
      activeRepos.map(async (repo) => {
        try {
          const status = await invoke<RepoStatusDto>("get_repo_status", { repoId: repo.id });
          repos = repos.map((r) => (r.id === repo.id ? { ...r, status, statusLoading: false } : r));
        } catch {
          repos = repos.map((r) => (r.id === repo.id ? { ...r, statusLoading: false } : r));
        }
      }),
    );
  }

  async function handleScan() {
    if (!scanPath.trim()) return;
    scanning = true;
    actionFeedback = null;
    try {
      const result = await invoke<{ new: number; found: number }>("scan_directory", {
        path: scanPath.trim(),
      });
      actionFeedback = { message: `Scan complete: ${result.found} found, ${result.new} new` };
      showScanDialog = false;
      scanPath = "";
      await loadWorkspace();
    } catch (e) {
      actionFeedback = handleError(e);
    } finally {
      scanning = false;
    }
  }

  async function handleFetchAll() {
    fetchingAll = true;
    actionFeedback = null;
    try {
      const result = await invoke<BulkResultDto>("fetch_all");
      actionFeedback = {
        message: `Fetch all: ${result.success_count} succeeded, ${result.failed_count} failed, ${result.skipped_count} skipped`,
      };
      await loadStatuses();
    } catch (e) {
      actionFeedback = handleError(e);
    } finally {
      fetchingAll = false;
    }
  }

  async function handleFetch(repoId: string) {
    try {
      await invoke("fetch_repo", { repoId });
      const status = await invoke<RepoStatusDto>("get_repo_status", { repoId });
      repos = repos.map((r) => (r.id === repoId ? { ...r, status } : r));
    } catch (e) {
      actionFeedback = handleError(e);
    }
  }

  async function handlePull(repoId: string) {
    try {
      await invoke("pull_repo", { repoId });
      const status = await invoke<RepoStatusDto>("get_repo_status", { repoId });
      repos = repos.map((r) => (r.id === repoId ? { ...r, status } : r));
    } catch (e) {
      actionFeedback = handleError(e);
    }
  }

  function trackingLabel(status: RepoStatusDto | undefined): string {
    if (!status) return "—";
    if (status.ahead === 0 && status.behind === 0) return "Up to date";
    const parts: string[] = [];
    if (status.ahead > 0) parts.push(`↑${status.ahead}`);
    if (status.behind > 0) parts.push(`↓${status.behind}`);
    return parts.join(" ");
  }

  function branchLabel(repo: RepoWithStatus): string {
    if (repo.state === "missing") return "—";
    if (repo.statusLoading) return "…";
    if (repo.status?.detached) return "detached";
    return repo.status?.branch ?? "—";
  }
</script>

<div class="dashboard">
  <header class="dashboard-header">
    <div>
      <h2 class="page-title">Workspace</h2>
      <p class="page-subtitle">Overview of registered repositories</p>
    </div>
    <div class="header-actions">
      <button class="btn-secondary" type="button" onclick={() => (showScanDialog = true)}>
        Scan Directory
      </button>
      <button
        class="btn-primary"
        type="button"
        onclick={handleFetchAll}
        disabled={fetchingAll || activeCount === 0}
      >
        {fetchingAll ? "Fetching…" : "Fetch All"}
      </button>
    </div>
  </header>

  <ErrorBanner error={actionFeedback} />

  <section class="stats-bar" aria-label="Workspace statistics">
    <div class="stat-card">
      <span class="stat-value">{total}</span>
      <span class="stat-label">Total</span>
    </div>
    <div class="stat-card">
      <span class="stat-value stat-active">{activeCount}</span>
      <span class="stat-label">Active</span>
    </div>
    <div class="stat-card">
      <span class="stat-value stat-missing">{missingCount}</span>
      <span class="stat-label">Missing</span>
    </div>
    <div class="stat-card">
      <span class="stat-value stat-dirty">{dirtyCount}</span>
      <span class="stat-label">Dirty</span>
    </div>
  </section>

  {#if loading}
    <div class="empty-state">Loading repositories…</div>
  {:else if pageError}
    <div class="empty-state error">
      {pageError.message}
      {#if pageError.hint}
        <p class="error-hint">{pageError.hint}</p>
      {/if}
    </div>
  {:else if repos.length === 0}
    <div class="empty-state">
      <p>No repositories registered yet.</p>
      <button class="btn-primary" type="button" onclick={() => (showScanDialog = true)}>
        Scan a Directory
      </button>
    </div>
  {:else}
    <!-- Tag filter -->
    {#if allTags.length > 0}
      <div class="filter-bar">
        <label class="filter-label">
          Filter by tag:
          <select class="filter-select" bind:value={selectedTag}>
            <option value="">All</option>
            {#each allTags as tag (tag.name)}
              <option value={tag.name}>{tag.name} ({tag.repo_count})</option>
            {/each}
          </select>
        </label>
        {#if selectedTag}
          <button class="btn-link" type="button" onclick={() => (selectedTag = "")}>Clear</button>
        {/if}
      </div>
    {/if}

    {#if filteredRepos.length === 0}
      <div class="empty-state">
        No repositories match the selected tag "{selectedTag}".
      </div>
    {:else}
      <div class="repo-table-wrap">
        <table class="repo-table">
          <thead>
            <tr>
              <th>Name</th>
              <th>Branch</th>
              <th>Status</th>
              <th>Tracking</th>
              <th>Last Commit</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each filteredRepos as repo (repo.id)}
              <tr class:missing={repo.state === "missing"}>
                <td class="col-name">
                  <a href={resolve(`/repo/${repo.id}`)} class="repo-link">
                    <span class="repo-name">{repo.name}</span>
                    <span class="repo-path mono">{repo.path}</span>
                  </a>
                </td>
                <td class="mono">{branchLabel(repo)}</td>
                <td>
                  {#if repo.state === "missing"}
                    <span class="badge badge-missing">missing</span>
                  {:else if repo.statusLoading}
                    <span class="badge badge-loading">…</span>
                  {:else if repo.status?.dirty}
                    <span class="badge badge-dirty">dirty</span>
                  {:else}
                    <span class="badge badge-clean">clean</span>
                  {/if}
                </td>
                <td class="tracking">{trackingLabel(repo.status)}</td>
                <td class="col-commit">
                  {#if repo.status?.head_short_id}
                    <span class="mono commit-id">{repo.status.head_short_id}</span>
                  {/if}
                  <span class="commit-summary">{repo.status?.head_summary ?? "—"}</span>
                </td>
                <td class="col-actions">
                  {#if repo.state === "active"}
                    <button
                      class="btn-icon"
                      type="button"
                      title="Fetch"
                      onclick={() => handleFetch(repo.id)}
                    >
                      ↓
                    </button>
                    <button
                      class="btn-icon"
                      type="button"
                      title="Pull"
                      onclick={() => handlePull(repo.id)}
                    >
                      ⟳
                    </button>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  {/if}
</div>

{#if showScanDialog}
  <div
    class="dialog-backdrop"
    role="presentation"
    onclick={() => (showScanDialog = false)}
    onkeydown={(e) => e.key === "Escape" && (showScanDialog = false)}
  >
    <div
      class="dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="scan-title"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <h3 id="scan-title" class="dialog-title">Scan Directory</h3>
      <p class="dialog-desc">Enter the path to a directory to scan for Git repositories.</p>
      <input
        class="dialog-input mono"
        type="text"
        placeholder="C:\Users\you\projects"
        bind:value={scanPath}
        onkeydown={(e) => e.key === "Enter" && handleScan()}
      />
      <div class="dialog-actions">
        <button class="btn-secondary" type="button" onclick={() => (showScanDialog = false)}>
          Cancel
        </button>
        <button
          class="btn-primary"
          type="button"
          onclick={handleScan}
          disabled={scanning || !scanPath.trim()}
        >
          {scanning ? "Scanning…" : "Scan"}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .dashboard {
    padding: var(--space-xl);
    max-width: 1200px;
  }

  .dashboard-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-lg);
    margin-bottom: var(--space-xl);
  }

  .header-actions {
    display: flex;
    gap: var(--space-sm);
    flex-shrink: 0;
  }

  .stats-bar {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
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

  .stat-value {
    font-size: var(--text-3xl);
    font-weight: 400;
    color: var(--color-ink);
    letter-spacing: -0.03em;
    line-height: 1;
  }

  .stat-value.stat-active {
    color: var(--color-success);
  }
  .stat-value.stat-missing {
    color: var(--color-error);
  }
  .stat-value.stat-dirty {
    color: var(--color-primary);
  }

  .stat-label {
    font-size: var(--text-caption);
    color: var(--color-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .filter-bar {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    margin-bottom: var(--space-base);
  }

  .filter-label {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    font-size: var(--text-caption);
    color: var(--color-muted);
  }

  .filter-select {
    padding: var(--space-xs) var(--space-sm);
    border: 1px solid var(--color-hairline-strong);
    border-radius: var(--radius-md);
    background: var(--color-surface-card);
    color: var(--color-ink);
    font-size: var(--text-caption);
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

  .repo-table tr:last-child td {
    border-bottom: none;
  }
  .repo-table tr.missing {
    opacity: 0.6;
  }

  .col-name {
    min-width: 160px;
  }

  .repo-link {
    text-decoration: none;
    color: inherit;
    display: block;
  }

  .repo-link:hover .repo-name {
    color: var(--color-primary);
  }

  .repo-name {
    display: block;
    font-weight: 500;
    color: var(--color-ink);
  }

  .repo-path {
    display: block;
    font-size: var(--text-sm);
    color: var(--color-muted-soft);
    margin-top: 2px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 280px;
  }

  .col-commit {
    max-width: 320px;
  }

  .commit-id {
    display: inline-block;
    font-size: var(--text-sm);
    color: var(--color-muted);
    margin-right: var(--space-xs);
  }

  .commit-summary {
    color: var(--color-body);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    display: inline-block;
    max-width: 240px;
    vertical-align: middle;
  }

  .tracking {
    font-size: var(--text-caption);
    color: var(--color-muted);
    white-space: nowrap;
  }

  .col-actions {
    white-space: nowrap;
  }

  .btn-icon {
    margin-right: var(--space-xxs);
  }

  .mono {
    font-family: var(--font-mono);
  }
</style>
