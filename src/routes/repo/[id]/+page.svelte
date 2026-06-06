<script lang="ts">
  import { page } from "$app/stores";
  import { resolve } from "$app/paths";
  import { invoke } from "@tauri-apps/api/core";
  import type { RepoDto, RepoStatusDto, OpResultDto, GroupDto } from "$lib/types/workspace";
  import { handleError, type HandledError } from "$lib/utils/error-handling";
  import ErrorBanner from "$lib/components/ErrorBanner.svelte";
  import type { RepositoryHealthDto } from "$lib/types/health";
  import { getRepositoryHealth, refreshHealth } from "$lib/types/health";
  import RepoHealthSection from "$lib/components/RepoHealthSection.svelte";
  import ChangedFilesList from "$lib/components/ChangedFilesList.svelte";

  let repo = $state<RepoDto | null>(null);
  let status = $state<RepoStatusDto | null>(null);
  let loading = $state(true);
  let pageError = $state<HandledError | null>(null);
  let actionFeedback = $state<HandledError | null>(null);
  let operating = $state(false);

  // Group assignment
  let allGroups = $state<GroupDto[]>([]);
  let selectedGroupId = $state("");
  let movingGroup = $state(false);

  // Tag editor
  let newTag = $state("");
  let tagError = $state<HandledError | null>(null);

  // Health
  let repoHealth = $state<RepositoryHealthDto | null>(null);
  let healthLoading = $state(false);
  let healthRefreshing = $state(false);

  const repoId = $derived($page.params.id ?? "");
  const branchLabel = $derived(() => {
    if (!status) return "—";
    if (status.detached) return "HEAD (detached)";
    return status.branch ?? "(unborn)";
  });

  const currentGroupName = $derived(() => {
    if (!repo?.group_id) return "Ungrouped";
    return allGroups.find((g) => g.id === repo?.group_id)?.name ?? "Ungrouped";
  });

  $effect(() => {
    if (repoId) loadRepo(repoId);
  });

  async function loadRepoHealth(id: string) {
    healthLoading = true;
    try {
      repoHealth = await getRepositoryHealth(id);
    } catch {
      repoHealth = null;
    } finally {
      healthLoading = false;
    }
  }

  async function loadRepo(id: string) {
    loading = true;
    pageError = null;
    repoHealth = null;
    try {
      const repos = await invoke<RepoDto[]>("list_repositories");
      repo = repos.find((r) => r.id === id) ?? null;
      if (!repo) {
        pageError = { message: "Repository not found" };
        return;
      }
      if (repo.state === "active") {
        status = await invoke<RepoStatusDto>("get_repo_status", { repoId: id });
        await loadRepoHealth(id);
      }
      allGroups = await invoke<GroupDto[]>("list_groups");
      const ungroupedGroup = allGroups.find((g) => g.name === "Ungrouped");
      selectedGroupId = repo.group_id === ungroupedGroup?.id ? "" : (repo.group_id ?? "");
    } catch (e) {
      pageError = handleError(e);
    } finally {
      loading = false;
    }
  }

  async function handleRefreshHealth() {
    healthRefreshing = true;
    try {
      await refreshHealth();
      if (repoId) await loadRepoHealth(repoId);
    } catch (e) {
      actionFeedback = handleError(e);
    } finally {
      healthRefreshing = false;
    }
  }

  async function handleFetch() {
    if (!repo) return;
    operating = true;
    actionFeedback = null;
    try {
      const result = await invoke<OpResultDto>("fetch_repo", { repoId: repo.id });
      actionFeedback = { message: result.success ? "Fetch completed" : result.message };
      status = await invoke<RepoStatusDto>("get_repo_status", { repoId: repo.id });
    } catch (e) {
      actionFeedback = handleError(e);
    } finally {
      operating = false;
    }
  }

  async function handlePull() {
    if (!repo) return;
    operating = true;
    actionFeedback = null;
    try {
      const result = await invoke<OpResultDto>("pull_repo", { repoId: repo.id });
      actionFeedback = { message: result.success ? "Pull completed" : result.message };
      status = await invoke<RepoStatusDto>("get_repo_status", { repoId: repo.id });
    } catch (e) {
      actionFeedback = handleError(e);
    } finally {
      operating = false;
    }
  }

  async function handleGroupChange(e: Event) {
    if (!repo) return;
    const newGroupId = (e.target as HTMLSelectElement).value;
    const ungroupedGroup = allGroups.find((g) => g.name === "Ungrouped");
    const targetGroupId = newGroupId === "" && ungroupedGroup ? ungroupedGroup.id : newGroupId;
    const currentGroupId = repo.group_id ?? ungroupedGroup?.id ?? "";
    if (targetGroupId === currentGroupId) return;

    const oldName = currentGroupName();
    const newName = allGroups.find((g) => g.id === targetGroupId)?.name ?? newGroupId;

    movingGroup = true;
    actionFeedback = null;
    try {
      await invoke("assign_repo_to_group", { repoId: repo.id, groupId: targetGroupId });
      actionFeedback = { message: `Moved from "${oldName}" to "${newName}"` };
      repo = { ...repo, group_id: targetGroupId };
      selectedGroupId = targetGroupId === ungroupedGroup?.id ? "" : targetGroupId;
    } catch (e) {
      actionFeedback = handleError(e);
      const ug = allGroups.find((g) => g.name === "Ungrouped");
      selectedGroupId = repo.group_id === ug?.id ? "" : (repo.group_id ?? "");
    } finally {
      movingGroup = false;
    }
  }

  async function handleAddTag() {
    if (!repo || !newTag.trim()) {
      tagError = { message: "Tag name cannot be empty" };
      return;
    }
    tagError = null;
    try {
      await invoke("add_tag", { repoId: repo.id, tag: newTag.trim() });
      repo = { ...repo, tags: [...repo.tags, newTag.trim()] };
      newTag = "";
    } catch (e) {
      tagError = handleError(e);
    }
  }

  async function handleRemoveTag(tag: string) {
    if (!repo) return;
    try {
      await invoke("remove_tag", { repoId: repo.id, tag });
      repo = { ...repo, tags: repo.tags.filter((t) => t !== tag) };
    } catch (e) {
      actionFeedback = handleError(e);
    }
  }
</script>

<div class="detail">
  {#if loading}
    <div class="loading-state">Loading repository…</div>
  {:else if pageError}
    <div class="error-state">
      {pageError.message}
      {#if pageError.hint}
        <p class="error-hint">{pageError.hint}</p>
      {/if}
    </div>
  {:else if repo}
    <header class="detail-header">
      <div>
        <a href={resolve("/")} class="back-link">← Workspace</a>
        <h2 class="repo-title">{repo.name}</h2>
        <p class="repo-path mono">{repo.path}</p>
      </div>
      <div class="header-actions">
        {#if repo.state === "active"}
          <button class="btn-secondary" type="button" onclick={handleFetch} disabled={operating}>
            Fetch
          </button>
          <button class="btn-primary" type="button" onclick={handlePull} disabled={operating}>
            Pull
          </button>
        {/if}
      </div>
    </header>

    <ErrorBanner error={actionFeedback} />

    {#if repo.state === "missing"}
      <div class="missing-banner">
        This repository's path no longer exists on disk. It will be retained for re-linking if the
        path reappears.
      </div>
    {:else if status}
      <div class="info-grid">
        <div class="info-card">
          <span class="info-label">Branch</span>
          <span class="info-value mono">{branchLabel()}</span>
        </div>
        <div class="info-card">
          <span class="info-label">Status</span>
          <span class="info-value">
            {#if status.dirty}
              <span class="badge badge-dirty">dirty</span>
            {:else}
              <span class="badge badge-clean">clean</span>
            {/if}
          </span>
        </div>
        <div class="info-card">
          <span class="info-label">Tracking</span>
          <span class="info-value">
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

      {#if status.dirty && status.changed_files.length > 0}
        <ChangedFilesList files={status.changed_files} />
      {/if}

      <RepoHealthSection
        health={repoHealth}
        loading={healthLoading}
        refreshing={healthRefreshing}
        onRefresh={handleRefreshHealth}
      />

      {#if status.head_short_id}
        <section class="commit-section">
          <h3 class="section-title">Last Commit</h3>
          <div class="commit-card">
            <span class="commit-oid mono">{status.head_short_id}</span>
            <span class="commit-msg">{status.head_summary ?? "—"}</span>
          </div>
        </section>
      {/if}
    {/if}

    <!-- Group Assignment -->
    <section class="group-section">
      <h3 class="section-title">Group</h3>
      <div class="group-select-wrap">
        <select
          class="group-select"
          value={selectedGroupId}
          onchange={handleGroupChange}
          disabled={movingGroup}
        >
          <option value="">Ungrouped</option>
          {#each allGroups as g (g.id)}
            <option value={g.id}>{g.name}</option>
          {/each}
        </select>
      </div>
    </section>

    <!-- Tag Editor -->
    <section class="tags-section">
      <h3 class="section-title">Tags</h3>
      <div class="tag-list">
        {#each repo.tags as tag (tag)}
          <span class="tag-pill">
            {tag}
            <button
              class="tag-remove"
              type="button"
              title="Remove tag"
              onclick={() => handleRemoveTag(tag)}>×</button
            >
          </span>
        {/each}
      </div>
      <div class="tag-add">
        <input
          class="tag-input"
          type="text"
          placeholder="Add tag…"
          bind:value={newTag}
          onkeydown={(e) => e.key === "Enter" && handleAddTag()}
        />
        <button
          class="btn-secondary btn-sm"
          type="button"
          onclick={handleAddTag}
          disabled={!newTag.trim()}>Add</button
        >
      </div>
      {#if tagError}
        <p class="tag-error">{tagError.message}</p>
        {#if tagError.hint}
          <p class="error-hint">{tagError.hint}</p>
        {/if}
      {/if}
    </section>

    <section class="meta-section">
      <h3 class="section-title">Metadata</h3>
      <dl class="meta-list">
        <div class="meta-row">
          <dt>ID</dt>
          <dd class="mono">{repo.id}</dd>
        </div>
        <div class="meta-row">
          <dt>State</dt>
          <dd>{repo.state}</dd>
        </div>
      </dl>
    </section>
  {/if}
</div>

<style>
  .detail {
    padding: var(--space-xl);
    max-width: 960px;
  }

  .loading-state,
  .error-state {
    padding: var(--space-xxl);
    text-align: center;
    color: var(--color-muted);
  }

  .error-state {
    color: var(--color-error);
  }

  .detail-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: var(--space-xl);
  }

  .back-link {
    display: inline-block;
    font-size: var(--text-caption);
    color: var(--color-muted);
    margin-bottom: var(--space-xs);
    text-decoration: none;
  }

  .back-link:hover {
    color: var(--color-ink);
  }

  .repo-title {
    font-size: var(--text-2xl);
    letter-spacing: -0.03em;
    margin-bottom: var(--space-xxs);
  }

  .repo-path {
    margin: 0;
    font-size: var(--text-caption);
    color: var(--color-muted);
  }

  .header-actions {
    display: flex;
    gap: var(--space-sm);
    flex-shrink: 0;
    padding-top: var(--space-lg);
  }

  .missing-banner {
    padding: var(--space-base);
    border: 1px solid color-mix(in srgb, var(--color-error) 30%, transparent);
    border-radius: var(--radius-lg);
    background: color-mix(in srgb, var(--color-error) 5%, transparent);
    font-size: var(--text-body);
    color: var(--color-body);
    margin-bottom: var(--space-xl);
  }

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

  .badge {
    font-size: var(--text-body);
  }

  .section-title {
    font-size: var(--text-body);
    font-weight: 600;
    color: var(--color-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    margin-bottom: var(--space-sm);
  }

  .commit-section {
    margin-bottom: var(--space-xl);
  }

  .commit-card {
    padding: var(--space-base);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-lg);
    background: var(--color-surface-card);
  }

  .commit-oid {
    display: inline-block;
    font-size: var(--text-caption);
    color: var(--color-muted);
    margin-right: var(--space-sm);
  }

  .commit-msg {
    color: var(--color-body);
  }

  /* Group assignment */
  .group-section {
    margin-bottom: var(--space-xl);
  }

  .group-select-wrap {
    max-width: 300px;
  }

  .group-select {
    width: 100%;
    padding: var(--space-sm) var(--space-base);
    border: 1px solid var(--color-hairline-strong);
    border-radius: var(--radius-md);
    background: var(--color-surface-card);
    color: var(--color-ink);
    font-size: var(--text-body);
  }

  .group-select:disabled {
    opacity: 0.6;
  }

  /* Tag editor */
  .tags-section {
    margin-bottom: var(--space-xl);
  }

  .tag-list {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-xs);
    margin-bottom: var(--space-sm);
  }

  .tag-pill {
    display: inline-flex;
    align-items: center;
    gap: var(--space-xxs);
    padding: var(--space-xxs) var(--space-sm);
    border-radius: var(--radius-pill);
    background: var(--color-surface-strong);
    font-size: var(--text-caption);
    color: var(--color-ink);
  }

  .tag-remove {
    background: none;
    border: none;
    color: var(--color-muted);
    cursor: pointer;
    font-size: var(--text-body);
    line-height: 1;
    padding: 0 2px;
  }

  .tag-remove:hover {
    color: var(--color-error);
  }

  .tag-add {
    display: flex;
    gap: var(--space-xs);
    max-width: 300px;
  }

  .tag-input {
    flex: 1;
    padding: var(--space-xs) var(--space-sm);
    border: 1px solid var(--color-hairline-strong);
    border-radius: var(--radius-md);
    background: var(--color-canvas-soft);
    color: var(--color-ink);
    font-size: var(--text-caption);
  }

  .tag-input:focus {
    outline: 2px solid color-mix(in srgb, var(--color-primary) 40%, transparent);
    outline-offset: 1px;
  }

  .tag-error {
    margin: var(--space-xxs) 0 0;
    font-size: var(--text-sm);
    color: var(--color-error);
  }

  /* Metadata */
  .meta-section {
    margin-bottom: var(--space-xl);
  }

  .meta-list {
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-lg);
    background: var(--color-surface-card);
    overflow: hidden;
  }

  .meta-row {
    display: flex;
    padding: var(--space-sm) var(--space-base);
    border-bottom: 1px solid var(--color-hairline-soft);
  }

  .meta-row:last-child {
    border-bottom: none;
  }

  .meta-row dt {
    width: 120px;
    flex-shrink: 0;
    font-size: var(--text-caption);
    color: var(--color-muted);
    font-weight: 500;
  }

  .meta-row dd {
    margin: 0;
    font-size: var(--text-caption);
    color: var(--color-body);
  }

  .mono {
    font-family: var(--font-mono);
  }
</style>
