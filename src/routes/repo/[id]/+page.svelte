<script lang="ts">
  import { page } from "$app/stores";
  import { resolve } from "$app/paths";
  import { invoke } from "@tauri-apps/api/core";
  import type { RepoDto, RepoStatusDto, OpResultDto, GroupDto } from "$lib/types/workspace";
  import { handleError } from "$lib/types/workspace";
  import type { RepositoryHealthDto } from "$lib/types/health";
  import { getRepositoryHealth, refreshHealth } from "$lib/types/health";

  let repo = $state<RepoDto | null>(null);
  let status = $state<RepoStatusDto | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let errorHint = $state<string | undefined>(undefined);
  let actionMessage = $state<string | null>(null);
  let actionHint = $state<string | undefined>(undefined);
  let operating = $state(false);

  // Group assignment
  let allGroups = $state<GroupDto[]>([]);
  let selectedGroupId = $state("");
  let movingGroup = $state(false);

  // Tag editor
  let newTag = $state("");
  let tagError = $state<string | null>(null);
  let tagErrorHint = $state<string | undefined>(undefined);

  // Health
  let repoHealth = $state<RepositoryHealthDto | null>(null);
  let healthLoading = $state(false);
  let healthRefreshing = $state(false);

  // Changed files
  let changedFilesExpanded = $state(true);
  let showAllChangedFiles = $state(false);
  const CHANGED_FILES_LIMIT = 20;

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
    error = null;
    errorHint = undefined;
    repoHealth = null;
    try {
      const repos = await invoke<RepoDto[]>("list_repositories");
      repo = repos.find((r) => r.id === id) ?? null;
      if (!repo) {
        error = "Repository not found";
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
      const handled = handleError(e);
      if (!handled.isTransient) {
        error = handled.message;
        errorHint = handled.hint;
      }
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
      const handled = handleError(e);
      if (!handled.isTransient) {
        actionMessage = handled.message;
        actionHint = handled.hint;
      }
    } finally {
      healthRefreshing = false;
    }
  }

  function severityClass(severity: string): string {
    return `sev-${severity}`;
  }

  async function handleFetch() {
    if (!repo) return;
    operating = true;
    actionMessage = null;
    actionHint = undefined;
    try {
      const result = await invoke<OpResultDto>("fetch_repo", { repoId: repo.id });
      actionMessage = result.success ? "Fetch completed" : result.message;
      status = await invoke<RepoStatusDto>("get_repo_status", { repoId: repo.id });
    } catch (e) {
      const handled = handleError(e);
      if (!handled.isTransient) {
        actionMessage = handled.message;
        actionHint = handled.hint;
      }
    } finally {
      operating = false;
    }
  }

  async function handlePull() {
    if (!repo) return;
    operating = true;
    actionMessage = null;
    actionHint = undefined;
    try {
      const result = await invoke<OpResultDto>("pull_repo", { repoId: repo.id });
      actionMessage = result.success ? "Pull completed" : result.message;
      status = await invoke<RepoStatusDto>("get_repo_status", { repoId: repo.id });
    } catch (e) {
      const handled = handleError(e);
      if (!handled.isTransient) {
        actionMessage = handled.message;
        actionHint = handled.hint;
      }
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
    actionMessage = null;
    actionHint = undefined;
    try {
      await invoke("assign_repo_to_group", { repoId: repo.id, groupId: targetGroupId });
      actionMessage = `Moved from "${oldName}" to "${newName}"`;
      repo = { ...repo, group_id: targetGroupId };
      selectedGroupId = targetGroupId === ungroupedGroup?.id ? "" : targetGroupId;
    } catch (e) {
      const handled = handleError(e);
      if (!handled.isTransient) {
        actionMessage = handled.message;
        actionHint = handled.hint;
      }
      selectedGroupId = repo.group_id === ungroupedGroup?.id ? "" : (repo.group_id ?? "");
    } finally {
      movingGroup = false;
    }
  }

  async function handleAddTag() {
    if (!repo || !newTag.trim()) {
      tagError = "Tag name cannot be empty";
      return;
    }
    tagError = null;
    tagErrorHint = undefined;
    try {
      await invoke("add_tag", { repoId: repo.id, tag: newTag.trim() });
      repo = { ...repo, tags: [...repo.tags, newTag.trim()] };
      newTag = "";
    } catch (e) {
      const handled = handleError(e);
      if (!handled.isTransient) {
        tagError = handled.message;
        tagErrorHint = handled.hint;
      }
    }
  }

  async function handleRemoveTag(tag: string) {
    if (!repo) return;
    try {
      await invoke("remove_tag", { repoId: repo.id, tag });
      repo = { ...repo, tags: repo.tags.filter((t) => t !== tag) };
    } catch (e) {
      const handled = handleError(e);
      if (!handled.isTransient) {
        actionMessage = handled.message;
        actionHint = handled.hint;
      }
    }
  }
</script>

<div class="detail">
  {#if loading}
    <div class="loading-state">Loading repository…</div>
  {:else if error}
    <div class="error-state">
      {error}
      {#if errorHint}
        <p class="error-hint">{errorHint}</p>
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

    {#if actionMessage}
      <div class="action-banner" role="status">
        {actionMessage}
        {#if actionHint}
          <p class="error-hint">{actionHint}</p>
        {/if}
      </div>
    {/if}

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
        <section class="changed-files-section">
          <button
            class="changed-files-toggle"
            type="button"
            onclick={() => (changedFilesExpanded = !changedFilesExpanded)}
          >
            <span class="toggle-icon">{changedFilesExpanded ? "▾" : "▸"}</span>
            Changed files ({status.changed_files.length})
          </button>
          {#if changedFilesExpanded}
            <ul class="changed-files-list">
              {#each showAllChangedFiles ? status.changed_files : status.changed_files.slice(0, CHANGED_FILES_LIMIT) as file (file.path)}
                <li class="changed-file-item">
                  <span class="file-status status-{file.status}"
                    >{file.status[0]?.toUpperCase()}</span
                  >
                  <span class="file-path mono">{file.path}</span>
                </li>
              {/each}
            </ul>
            {#if status.changed_files.length > CHANGED_FILES_LIMIT}
              <button
                class="show-more-btn"
                type="button"
                onclick={() => (showAllChangedFiles = !showAllChangedFiles)}
              >
                {showAllChangedFiles
                  ? "Show fewer"
                  : `Show ${status.changed_files.length - CHANGED_FILES_LIMIT} more`}
              </button>
            {/if}
          {/if}
        </section>
      {/if}

      <section class="health-section">
        <div class="health-header">
          <h3 class="section-title">Health</h3>
          <button
            class="btn-secondary btn-sm"
            type="button"
            onclick={handleRefreshHealth}
            disabled={healthRefreshing || healthLoading}
          >
            {healthRefreshing ? "Refreshing…" : "Refresh"}
          </button>
        </div>
        {#if healthLoading}
          <div class="health-empty">Loading health data…</div>
        {:else if repoHealth && repoHealth.checks.length > 0}
          <div class="health-check-list">
            {#each repoHealth.checks as check (check.check_id)}
              <div class="health-check-item">
                <span class="sev-dot {severityClass(check.severity)}"></span>
                <span class="check-id">{check.check_id}</span>
                <span class="check-msg">{check.message}</span>
              </div>
            {/each}
          </div>
        {:else}
          <div class="health-empty">
            <span>No health data</span>
            <button
              class="btn-secondary btn-sm"
              type="button"
              onclick={handleRefreshHealth}
              disabled={healthRefreshing}
            >
              Refresh
            </button>
          </div>
        {/if}
      </section>

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
        <p class="tag-error">{tagError}</p>
        {#if tagErrorHint}
          <p class="error-hint">{tagErrorHint}</p>
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

  .changed-files-section {
    margin-bottom: var(--space-xl);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-lg);
    background: var(--color-surface-card);
    overflow: hidden;
  }

  .changed-files-toggle {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    width: 100%;
    padding: var(--space-sm) var(--space-base);
    border: none;
    background: none;
    font-size: var(--text-body);
    font-weight: 600;
    color: var(--color-ink);
    cursor: pointer;
    text-align: left;
  }

  .changed-files-toggle:hover {
    background: var(--color-hairline-soft);
  }

  .toggle-icon {
    font-size: var(--text-2xs);
    color: var(--color-muted);
  }

  .changed-files-list {
    list-style: none;
    margin: 0;
    padding: 0 var(--space-base) var(--space-sm);
  }

  .changed-file-item {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-xxs) 0;
    font-size: var(--text-caption);
  }

  .file-status {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border-radius: var(--radius-sm);
    font-size: var(--text-xs);
    font-weight: 600;
    flex-shrink: 0;
  }

  .file-status.status-added,
  .file-status.status-untracked {
    background: color-mix(in srgb, var(--color-success) 15%, transparent);
    color: var(--color-success);
  }

  .file-status.status-modified {
    background: color-mix(in srgb, var(--color-warning) 15%, transparent);
    color: var(--color-warning);
  }

  .file-status.status-deleted {
    background: color-mix(in srgb, var(--color-error) 15%, transparent);
    color: var(--color-error);
  }

  .file-status.status-renamed {
    background: color-mix(in srgb, var(--color-primary) 15%, transparent);
    color: var(--color-primary);
  }

  .file-path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--color-body);
  }

  .show-more-btn {
    display: block;
    width: 100%;
    padding: var(--space-xs) var(--space-base) var(--space-sm);
    border: none;
    background: none;
    font-size: var(--text-sm);
    color: var(--color-primary);
    cursor: pointer;
    text-align: left;
  }

  .show-more-btn:hover {
    text-decoration: underline;
  }

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
