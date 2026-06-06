<script lang="ts">
  import { page } from "$app/stores";
  import { resolve } from "$app/paths";
  import { invoke } from "@tauri-apps/api/core";
  import type { RepoDto, RepoStatusDto, OpResultDto, GroupDto } from "$lib/types/workspace";
  import { handleError, success, type ActionFeedback } from "$lib/utils/error-handling";
  import FeedbackBanner from "$lib/components/FeedbackBanner.svelte";
  import PageError from "$lib/components/PageError.svelte";
  import type { RepositoryHealthDto } from "$lib/types/health";
  import { getRepositoryHealth, refreshHealth } from "$lib/types/health";
  import RepoHealthSection from "$lib/components/RepoHealthSection.svelte";
  import RepoStatusGrid from "$lib/components/RepoStatusGrid.svelte";
  import RepoGroupSelect from "$lib/components/RepoGroupSelect.svelte";
  import RepoTagEditor from "$lib/components/RepoTagEditor.svelte";
  import ChangedFilesList from "$lib/components/ChangedFilesList.svelte";

  let repo = $state<RepoDto | null>(null);
  let status = $state<RepoStatusDto | null>(null);
  let loading = $state(true);
  let pageError = $state<ActionFeedback | null>(null);
  let actionFeedback = $state<ActionFeedback | null>(null);
  let operating = $state(false);

  let allGroups = $state<GroupDto[]>([]);
  let selectedGroupId = $state("");
  let movingGroup = $state(false);

  let tagError = $state<ActionFeedback | null>(null);

  let repoHealth = $state<RepositoryHealthDto | null>(null);
  let healthLoading = $state(false);
  let healthRefreshing = $state(false);

  const repoId = $derived($page.params.id ?? "");
  const branchLabel = $derived.by(() => {
    if (!status) return "—";
    if (status.detached) return "HEAD (detached)";
    return status.branch ?? "(unborn)";
  });

  const currentGroupName = $derived.by(() => {
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
        pageError = { message: "Repository not found", severity: "error" };
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

  type RepoOp = { command: "fetch_repo"; label: "Fetch" } | { command: "pull_repo"; label: "Pull" };

  async function handleRepoOp(command: RepoOp["command"], label: RepoOp["label"]) {
    if (!repo) return;
    operating = true;
    actionFeedback = null;
    try {
      const result = await invoke<OpResultDto>(command, { repoId: repo.id });
      actionFeedback = result.success
        ? success(`${label} completed`)
        : { message: result.message, severity: "error" };
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

    const oldName = currentGroupName;
    const newName = allGroups.find((g) => g.id === targetGroupId)?.name ?? newGroupId;

    movingGroup = true;
    actionFeedback = null;
    try {
      await invoke("assign_repo_to_group", { repoId: repo.id, groupId: targetGroupId });
      actionFeedback = success(`Moved from "${oldName}" to "${newName}"`);
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

  async function handleAddTag(tag: string) {
    if (!repo) return;
    tagError = null;
    try {
      await invoke("add_tag", { repoId: repo.id, tag });
      repo = { ...repo, tags: [...repo.tags, tag] };
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
    <PageError error={pageError} />
  {:else if repo}
    <header class="detail-header">
      <div>
        <a href={resolve("/")} class="back-link">← Workspace</a>
        <h2 class="repo-title">{repo.name}</h2>
        <p class="repo-path mono">{repo.path}</p>
      </div>
      <div class="header-actions">
        {#if repo.state === "active"}
          <button
            class="btn-secondary"
            type="button"
            onclick={() => handleRepoOp("fetch_repo", "Fetch")}
            disabled={operating}
          >
            Fetch
          </button>
          <button
            class="btn-primary"
            type="button"
            onclick={() => handleRepoOp("pull_repo", "Pull")}
            disabled={operating}
          >
            Pull
          </button>
        {/if}
      </div>
    </header>

    <FeedbackBanner feedback={actionFeedback} />

    {#if repo.state === "missing"}
      <div class="missing-banner">
        This repository's path no longer exists on disk. It will be retained for re-linking if the
        path reappears.
      </div>
    {:else if status}
      <RepoStatusGrid {status} {branchLabel} />

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

    <RepoGroupSelect
      groups={allGroups}
      {selectedGroupId}
      disabled={movingGroup}
      onchange={handleGroupChange}
    />

    <RepoTagEditor
      tags={repo.tags}
      onAdd={handleAddTag}
      onRemove={handleRemoveTag}
      error={tagError}
    />

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

  .loading-state {
    padding: var(--space-xxl);
    text-align: center;
    color: var(--color-muted);
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
</style>
