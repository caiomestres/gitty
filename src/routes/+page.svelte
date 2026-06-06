<script lang="ts">
  import { resolve } from "$app/paths";
  import { SvelteMap } from "svelte/reactivity";
  import { invoke } from "@tauri-apps/api/core";
  import type {
    BulkResultDto,
    RepoDto,
    RepoStatusDto,
    RepoWithStatus,
    TagDto,
  } from "$lib/types/workspace";
  import { handleError, success, type ActionFeedback } from "$lib/utils/error-handling";
  import FeedbackBanner from "$lib/components/FeedbackBanner.svelte";
  import PageError from "$lib/components/PageError.svelte";
  import Dialog from "$lib/components/Dialog.svelte";

  let repos = $state<RepoWithStatus[]>([]);
  let loading = $state(true);
  let pageError = $state<ActionFeedback | null>(null);
  let showScanDialog = $state(false);
  let scanPath = $state("");
  let scanning = $state(false);
  let fetchingAll = $state(false);
  let actionFeedback = $state<ActionFeedback | null>(null);

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
    const results = await Promise.allSettled(
      activeRepos.map(async (repo) => {
        const status = await invoke<RepoStatusDto>("get_repo_status", { repoId: repo.id });
        return { id: repo.id, status };
      }),
    );

    const statusMap = new SvelteMap<string, RepoStatusDto>();
    for (const result of results) {
      if (result.status === "fulfilled") {
        statusMap.set(result.value.id, result.value.status);
      }
    }

    repos = repos.map((r) => ({
      ...r,
      status: statusMap.get(r.id) ?? r.status,
      statusLoading: false,
    }));
  }

  async function handleScan() {
    if (!scanPath.trim()) return;
    scanning = true;
    actionFeedback = null;
    try {
      const result = await invoke<{ new: number; found: number }>("scan_directory", {
        path: scanPath.trim(),
      });
      actionFeedback = success(`Scan complete: ${result.found} found, ${result.new} new`);
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
      actionFeedback = success(
        `Fetch all: ${result.success_count} succeeded, ${result.failed_count} failed, ${result.skipped_count} skipped`,
      );
      await loadStatuses();
    } catch (e) {
      actionFeedback = handleError(e);
    } finally {
      fetchingAll = false;
    }
  }

  type RepoCommand = "fetch_repo" | "pull_repo";

  async function handleRepoOp(repoId: string, command: RepoCommand) {
    try {
      await invoke(command, { repoId });
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
  <header class="page-header">
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

  <FeedbackBanner feedback={actionFeedback} />

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
    <PageError error={pageError} />
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
                      onclick={() => handleRepoOp(repo.id, "fetch_repo")}
                    >
                      ↓
                    </button>
                    <button
                      class="btn-icon"
                      type="button"
                      title="Pull"
                      onclick={() => handleRepoOp(repo.id, "pull_repo")}
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
  <Dialog
    title="Scan Directory"
    description="Enter the path to a directory to scan for Git repositories."
    onClose={() => (showScanDialog = false)}
  >
    <input
      class="dialog-input mono"
      type="text"
      placeholder="C:\Users\you\projects"
      bind:value={scanPath}
      onkeydown={(e) => e.key === "Enter" && handleScan()}
    />
    {#snippet actions()}
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
    {/snippet}
  </Dialog>
{/if}

<style>
  .dashboard {
    padding: var(--space-xl);
    max-width: 1200px;
  }

  .header-actions {
    display: flex;
    gap: var(--space-sm);
    flex-shrink: 0;
  }

  .stats-bar {
    grid-template-columns: repeat(4, 1fr);
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
</style>
