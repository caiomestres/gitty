<script lang="ts">
  import type { GroupedChangesDto, TimeWindow, Grouping } from "$lib/types/changes";
  import { getChanges } from "$lib/types/changes";
  import { handleError, type ActionFeedback } from "$lib/utils/error-handling";
  import PageError from "$lib/components/PageError.svelte";
  import Pagination from "$lib/components/Pagination.svelte";
  import { onMount } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import { invoke } from "@tauri-apps/api/core";

  let data = $state<GroupedChangesDto | null>(null);
  let loading = $state(true);
  let pageError = $state<ActionFeedback | null>(null);
  let window = $state<TimeWindow>("week");
  let grouping = $state<Grouping>("repository");
  let allBranchesRepos = new SvelteSet<string>();

  // Pagination
  let pageSize = $state(25);
  let currentPage = $state(1);

  interface FlatEntry {
    groupKey: string;
    entry: import("$lib/types/changes").ChangeEntryDto;
  }

  const flatEntries = $derived<FlatEntry[]>(
    data ? data.groups.flatMap((g) => g.entries.map((e) => ({ groupKey: g.key, entry: e }))) : [],
  );

  const pagedGroups = $derived.by(() => {
    const start = (currentPage - 1) * pageSize;
    const slice = flatEntries.slice(start, start + pageSize);
    const groups: { key: string; entries: import("$lib/types/changes").ChangeEntryDto[] }[] = [];
    for (const item of slice) {
      const last = groups[groups.length - 1];
      if (last && last.key === item.groupKey) {
        last.entries.push(item.entry);
      } else {
        groups.push({ key: item.groupKey, entries: [item.entry] });
      }
    }
    return groups;
  });

  onMount(() => {
    loadChanges();
  });

  async function loadChanges() {
    loading = true;
    pageError = null;
    try {
      const [changes, savedPageSize] = await Promise.all([
        getChanges(window, grouping, [...allBranchesRepos]),
        invoke<number>("get_page_size"),
      ]);
      data = changes;
      pageSize = savedPageSize;
      currentPage = 1;
    } catch (e) {
      pageError = handleError(e);
    } finally {
      loading = false;
    }
  }

  async function handlePageSizeChange(size: number) {
    pageSize = size;
    currentPage = 1;
    try {
      await invoke("set_page_size", { pageSize: size });
    } catch (e) {
      pageError = handleError(e);
    }
  }

  function handleWindowChange(newWindow: TimeWindow) {
    window = newWindow;
    loadChanges();
  }

  function handleGroupingChange(newGrouping: Grouping) {
    grouping = newGrouping;
    loadChanges();
  }

  function toggleAllBranches(repoId: string) {
    if (allBranchesRepos.has(repoId)) {
      allBranchesRepos.delete(repoId);
    } else {
      allBranchesRepos.add(repoId);
    }
    loadChanges();
  }

  function formatDate(iso: string): string {
    try {
      const d = new Date(iso);
      return d.toLocaleDateString(undefined, {
        month: "short",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      });
    } catch {
      return iso;
    }
  }

  const windowLabels: { value: TimeWindow; label: string }[] = [
    { value: "day", label: "24h" },
    { value: "week", label: "7d" },
    { value: "month", label: "30d" },
  ];

  const groupingLabels: { value: Grouping; label: string }[] = [
    { value: "repository", label: "Repository" },
    { value: "author", label: "Author" },
    { value: "branch", label: "Branch" },
  ];
</script>

<div class="changes-page">
  <header class="page-header">
    <div>
      <h2 class="page-title">Changes</h2>
      <p class="page-subtitle">
        {#if data}
          {data.total_commits} commits in the last {window === "day"
            ? "24 hours"
            : window === "week"
              ? "7 days"
              : "30 days"}
        {:else}
          Recent commit activity across your workspace
        {/if}
      </p>
    </div>
  </header>

  <div class="controls-bar">
    <div class="control-group">
      <span class="control-label">Time window</span>
      <div class="btn-group">
        {#each windowLabels as w (w.value)}
          <button
            class="btn-toggle"
            class:active={window === w.value}
            type="button"
            onclick={() => handleWindowChange(w.value)}
          >
            {w.label}
          </button>
        {/each}
      </div>
    </div>

    <div class="control-group">
      <span class="control-label">Group by</span>
      <div class="btn-group">
        {#each groupingLabels as g (g.value)}
          <button
            class="btn-toggle"
            class:active={grouping === g.value}
            type="button"
            onclick={() => handleGroupingChange(g.value)}
          >
            {g.label}
          </button>
        {/each}
      </div>
    </div>
  </div>

  {#if loading}
    <div class="empty-state">Scanning commits…</div>
  {:else if pageError}
    <PageError error={pageError} />
  {:else if !data || data.groups.length === 0}
    <div class="empty-state">
      <p>No commits found in the selected time window.</p>
    </div>
  {:else}
    <div class="groups-list">
      {#each pagedGroups as group, i (group.key + "-" + i)}
        <section class="change-group">
          <div class="group-header">
            <h3 class="group-key">{group.key}</h3>
            <div class="group-actions">
              {#if grouping === "repository" && group.entries.length > 0}
                {@const repoId = group.entries[0].repo_id}
                <button
                  class="btn-toggle-sm"
                  type="button"
                  title={allBranchesRepos.has(repoId) ? "Show HEAD only" : "Show all branches"}
                  onclick={() => toggleAllBranches(repoId)}
                >
                  {allBranchesRepos.has(repoId) ? "⊖ HEAD only" : "⊕ All branches"}
                </button>
              {/if}
              <span class="group-count">{group.entries.length} commits</span>
            </div>
          </div>
          <div class="commit-list">
            {#each group.entries as entry (entry.commit_hash)}
              <div class="commit-item">
                <span class="commit-hash mono">{entry.commit_hash.slice(0, 7)}</span>
                <span class="commit-subject">{entry.subject}</span>
                <span class="commit-meta">
                  {entry.author} · {formatDate(entry.date)}
                  {#if grouping !== "repository"}
                    · {entry.repo_name}
                  {/if}
                  {#if grouping !== "branch"}
                    · {entry.branch}
                  {/if}
                </span>
              </div>
            {/each}
          </div>
        </section>
      {/each}
    </div>
    <Pagination
      totalItems={flatEntries.length}
      {pageSize}
      {currentPage}
      onPageChange={(p) => (currentPage = p)}
      onPageSizeChange={handlePageSizeChange}
    />
  {/if}
</div>

<style>
  .changes-page {
    padding: var(--space-xl);
    max-width: 1200px;
  }

  .page-header {
    margin-bottom: var(--space-lg);
  }

  .controls-bar {
    display: flex;
    gap: var(--space-xl);
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

  .btn-group {
    display: flex;
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .btn-toggle {
    padding: var(--space-xs) var(--space-sm);
    border: none;
    background: var(--color-surface-card);
    color: var(--color-body);
    font-size: var(--text-caption);
    cursor: pointer;
    border-right: 1px solid var(--color-hairline);
    transition: background 0.15s ease;
  }

  .btn-toggle:last-child {
    border-right: none;
  }

  .btn-toggle:hover {
    background: var(--color-hairline-soft);
  }

  .btn-toggle.active {
    background: var(--color-primary);
    color: var(--color-on-primary);
  }

  .groups-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-lg);
  }

  .change-group {
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-lg);
    background: var(--color-surface-card);
    overflow: hidden;
  }

  .group-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-sm) var(--space-base);
    background: var(--color-canvas-soft);
    border-bottom: 1px solid var(--color-hairline);
  }

  .group-key {
    font-size: var(--text-body);
    font-weight: 600;
    color: var(--color-ink);
    margin: 0;
  }

  .group-count {
    font-size: var(--text-sm);
    color: var(--color-muted);
  }

  .group-actions {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .btn-toggle-sm {
    padding: 2px var(--space-sm);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-md);
    background: var(--color-surface-card);
    color: var(--color-muted);
    font-size: var(--text-xs);
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .btn-toggle-sm:hover {
    background: var(--color-hairline-soft);
    color: var(--color-ink);
  }

  .commit-list {
    display: flex;
    flex-direction: column;
  }

  .commit-item {
    display: grid;
    grid-template-columns: 70px 1fr;
    grid-template-rows: auto auto;
    gap: 0 var(--space-sm);
    padding: var(--space-xs) var(--space-base);
    border-bottom: 1px solid var(--color-hairline-soft);
    font-size: var(--text-caption);
  }

  .commit-item:last-child {
    border-bottom: none;
  }

  .commit-hash {
    grid-row: 1 / 3;
    color: var(--color-muted);
    font-size: var(--text-sm);
    padding-top: 2px;
  }

  .commit-subject {
    color: var(--color-ink);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .commit-meta {
    color: var(--color-muted-soft);
    font-size: var(--text-sm);
  }
</style>
