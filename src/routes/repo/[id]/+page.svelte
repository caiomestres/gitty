<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { invoke } from "@tauri-apps/api/core";
  import type { RepoDto, RepoStatusDto, OpResultDto, GroupDto } from "$lib/types/workspace";
  import type { EnvironmentDto, LivenessResultDto } from "$lib/types/liveness";
  import { handleError, success, type ActionFeedback } from "$lib/utils/error-handling";
  import { addToast } from "$lib/stores/toast.svelte";
  import FeedbackBanner from "$lib/components/FeedbackBanner.svelte";
  import PageError from "$lib/components/PageError.svelte";
  import Dialog from "$lib/components/Dialog.svelte";
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

  let showUnregisterDialog = $state(false);
  let unregistering = $state(false);

  let repoHealth = $state<RepositoryHealthDto | null>(null);
  let healthLoading = $state(false);
  let healthRefreshing = $state(false);

  // Environments
  let environments = $state<EnvironmentDto[]>([]);
  let livenessResults = $state<LivenessResultDto[]>([]);
  let showEnvForm = $state(false);
  let editingEnv = $state<string | null>(null);
  let envForm = $state<EnvironmentDto>({
    name: "",
    url: "",
    health_path: "/health",
    enabled: true,
    interval_seconds: 300,
  });
  let envSaving = $state(false);
  let envError = $state<ActionFeedback | null>(null);
  let showRemoveEnvDialog = $state(false);
  let removingEnvName = $state("");
  let removingEnv = $state(false);
  let probingEnv = $state<string | null>(null);

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

  async function loadEnvironments(id: string) {
    try {
      environments = await invoke<EnvironmentDto[]>("list_environments", { repoId: id });
      livenessResults = await invoke<LivenessResultDto[]>("get_liveness_results", { repoId: id });
    } catch {
      environments = [];
      livenessResults = [];
    }
  }

  function getLivenessStatus(envName: string): "up" | "down" | "gray" {
    const result = livenessResults.find((r) => r.environment_name === envName);
    if (!result) return "gray";
    return result.status;
  }

  function getLivenessResult(envName: string): LivenessResultDto | undefined {
    return livenessResults.find((r) => r.environment_name === envName);
  }

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
      await loadEnvironments(id);
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

  function resetEnvForm() {
    envForm = { name: "", url: "", health_path: "/health", enabled: true, interval_seconds: 300 };
    editingEnv = null;
    showEnvForm = false;
    envError = null;
  }

  function startEditEnv(env: EnvironmentDto) {
    envForm = { ...env };
    editingEnv = env.name;
    showEnvForm = true;
    envError = null;
  }

  async function handleSaveEnv() {
    if (!repo) return;
    envSaving = true;
    envError = null;
    try {
      if (editingEnv) {
        await invoke("update_environment", { repoId: repo.id, envName: editingEnv, env: envForm });
        actionFeedback = success(`Environment "${envForm.name}" updated`);
      } else {
        await invoke("add_environment", { repoId: repo.id, env: envForm });
        actionFeedback = success(`Environment "${envForm.name}" added`);
      }
      resetEnvForm();
      await loadEnvironments(repo.id);
    } catch (e) {
      envError = handleError(e);
    } finally {
      envSaving = false;
    }
  }

  function confirmRemoveEnv(name: string) {
    removingEnvName = name;
    showRemoveEnvDialog = true;
  }

  async function handleRemoveEnv() {
    if (!repo) return;
    removingEnv = true;
    try {
      await invoke("remove_environment", { repoId: repo.id, envName: removingEnvName });
      actionFeedback = success(`Environment "${removingEnvName}" removed`);
      showRemoveEnvDialog = false;
      removingEnvName = "";
      await loadEnvironments(repo.id);
    } catch (e) {
      actionFeedback = handleError(e);
      showRemoveEnvDialog = false;
    } finally {
      removingEnv = false;
    }
  }

  async function handleProbeEnv(envName: string) {
    if (!repo) return;
    probingEnv = envName;
    try {
      await invoke("probe_environment_cmd", { repoId: repo.id, envName });
      await loadEnvironments(repo.id);
    } catch (e) {
      actionFeedback = handleError(e);
    } finally {
      probingEnv = null;
    }
  }

  async function handleUnregister() {
    if (!repo) return;
    unregistering = true;
    try {
      await invoke("unregister_repository", { repoId: repo.id });
      addToast({ message: `"${repo.name}" unregistered`, severity: "info", dismissAfterMs: 4000 });
      await goto(resolve("/"));
    } catch (e) {
      actionFeedback = handleError(e);
      showUnregisterDialog = false;
    } finally {
      unregistering = false;
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
            title="Fetch latest changes from remote"
            onclick={() => handleRepoOp("fetch_repo", "Fetch")}
            disabled={operating}
          >
            Fetch
          </button>
          <button
            class="btn-primary"
            type="button"
            title="Pull and merge remote changes"
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

    <section class="env-section">
      <div class="env-header">
        <h3 class="section-title">Environments</h3>
        {#if !showEnvForm}
          <button
            class="btn-secondary btn-sm"
            type="button"
            onclick={() => {
              resetEnvForm();
              showEnvForm = true;
            }}
          >
            Add Environment
          </button>
        {/if}
      </div>

      {#if showEnvForm}
        <div class="env-form-card">
          <div class="env-form-row">
            <label class="env-label">
              Name
              <input
                class="env-input"
                type="text"
                bind:value={envForm.name}
                placeholder="staging"
                disabled={!!editingEnv}
              />
            </label>
            <label class="env-label">
              URL
              <input
                class="env-input"
                type="text"
                bind:value={envForm.url}
                placeholder="https://staging.example.com"
              />
            </label>
          </div>
          <div class="env-form-row">
            <label class="env-label">
              Health Path
              <input
                class="env-input"
                type="text"
                bind:value={envForm.health_path}
                placeholder="/health"
              />
            </label>
            <label class="env-label">
              Interval (seconds)
              <input
                class="env-input"
                type="number"
                bind:value={envForm.interval_seconds}
                min="30"
              />
            </label>
            <label class="env-label env-label-inline">
              <input type="checkbox" bind:checked={envForm.enabled} />
              Enabled
            </label>
          </div>
          {#if envError}
            <div class="env-form-error">{envError.message}</div>
          {/if}
          <div class="env-form-actions">
            <button class="btn-secondary btn-sm" type="button" onclick={resetEnvForm}>Cancel</button
            >
            <button
              class="btn-primary btn-sm"
              type="button"
              onclick={handleSaveEnv}
              disabled={envSaving || !envForm.name.trim() || !envForm.url.trim()}
            >
              {envSaving ? "Saving…" : editingEnv ? "Update" : "Add"}
            </button>
          </div>
        </div>
      {/if}

      {#if environments.length === 0 && !showEnvForm}
        <div class="env-empty">No environments configured. Add one to monitor service health.</div>
      {:else}
        <div class="env-list">
          {#each environments as env (env.name)}
            {@const status = getLivenessStatus(env.name)}
            {@const result = getLivenessResult(env.name)}
            <div class="env-card">
              <div class="env-card-main">
                <span
                  class="liveness-dot liveness-{status}"
                  title="{env.name}: {status}{result?.response_time_ms != null
                    ? ` (${result.response_time_ms}ms)`
                    : ''}"
                ></span>
                <div class="env-card-info">
                  <span class="env-name">{env.name}</span>
                  <span class="env-url mono">{env.url}{env.health_path}</span>
                </div>
                {#if result?.response_time_ms != null}
                  <span class="env-latency mono">{result.response_time_ms}ms</span>
                {/if}
                {#if !env.enabled}
                  <span class="badge badge-muted">disabled</span>
                {/if}
              </div>
              <div class="env-card-actions">
                <button
                  class="btn-icon"
                  type="button"
                  title="Probe now"
                  disabled={probingEnv === env.name}
                  onclick={() => handleProbeEnv(env.name)}
                >
                  {probingEnv === env.name ? "…" : "⟳"}
                </button>
                <button
                  class="btn-icon"
                  type="button"
                  title="Edit"
                  onclick={() => startEditEnv(env)}>✎</button
                >
                <button
                  class="btn-icon btn-icon-danger"
                  type="button"
                  title="Remove"
                  onclick={() => confirmRemoveEnv(env.name)}>✕</button
                >
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </section>

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

    <section class="danger-section">
      <h3 class="section-title">Danger Zone</h3>
      <div class="danger-card">
        <div class="danger-info">
          <strong>Unregister repository</strong>
          <p>Remove this repository from Gitty. The git repository on disk will not be affected.</p>
        </div>
        <button class="btn-danger" type="button" onclick={() => (showUnregisterDialog = true)}>
          Unregister
        </button>
      </div>
    </section>
  {/if}
</div>

{#if showRemoveEnvDialog}
  <Dialog title="Remove Environment" onClose={() => (showRemoveEnvDialog = false)}>
    <p class="dialog-body">
      Are you sure you want to remove environment <strong>{removingEnvName}</strong>? Liveness data
      for this environment will be lost.
    </p>
    {#snippet actions()}
      <button class="btn-secondary" type="button" onclick={() => (showRemoveEnvDialog = false)}
        >Cancel</button
      >
      <button class="btn-danger" type="button" onclick={handleRemoveEnv} disabled={removingEnv}>
        {removingEnv ? "Removing…" : "Remove"}
      </button>
    {/snippet}
  </Dialog>
{/if}

{#if showUnregisterDialog}
  <Dialog title="Unregister Repository" onClose={() => (showUnregisterDialog = false)}>
    <p class="dialog-body">
      Are you sure you want to unregister <strong>{repo?.name}</strong>? Group assignments, tags,
      and liveness configuration will be lost. The repository on disk will not be affected.
    </p>
    {#snippet actions()}
      <button class="btn-secondary" type="button" onclick={() => (showUnregisterDialog = false)}>
        Cancel
      </button>
      <button class="btn-danger" type="button" onclick={handleUnregister} disabled={unregistering}>
        {unregistering ? "Unregistering…" : "Unregister"}
      </button>
    {/snippet}
  </Dialog>
{/if}

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

  .danger-section {
    margin-bottom: var(--space-xl);
  }

  .danger-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-base);
    padding: var(--space-base);
    border: 1px solid color-mix(in srgb, var(--color-error) 30%, transparent);
    border-radius: var(--radius-lg);
    background: color-mix(in srgb, var(--color-error) 5%, transparent);
  }

  .danger-info p {
    margin: var(--space-xxs) 0 0;
    font-size: var(--text-caption);
    color: var(--color-muted);
  }

  .dialog-body {
    font-size: var(--text-body);
    color: var(--color-body);
    line-height: 1.5;
    margin: 0;
  }

  /* Environments */
  .env-section {
    margin-bottom: var(--space-xl);
  }

  .env-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-sm);
  }

  .env-header .section-title {
    margin-bottom: 0;
  }

  .env-empty {
    font-size: var(--text-caption);
    color: var(--color-muted);
    padding: var(--space-base);
    border: 1px dashed var(--color-hairline);
    border-radius: var(--radius-lg);
    text-align: center;
  }

  .env-form-card {
    padding: var(--space-base);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-lg);
    background: var(--color-surface-card);
    margin-bottom: var(--space-base);
  }

  .env-form-row {
    display: flex;
    gap: var(--space-sm);
    margin-bottom: var(--space-sm);
    flex-wrap: wrap;
  }

  .env-label {
    display: flex;
    flex-direction: column;
    gap: var(--space-xxs);
    font-size: var(--text-caption);
    color: var(--color-muted);
    flex: 1;
    min-width: 140px;
  }

  .env-label-inline {
    flex-direction: row;
    align-items: center;
    gap: var(--space-xs);
    min-width: auto;
    flex: 0;
    white-space: nowrap;
    padding-top: var(--space-base);
  }

  .env-input {
    padding: var(--space-xs) var(--space-sm);
    border: 1px solid var(--color-hairline-strong);
    border-radius: var(--radius-md);
    background: var(--color-canvas);
    color: var(--color-ink);
    font-size: var(--text-caption);
  }

  .env-form-error {
    color: var(--color-error);
    font-size: var(--text-caption);
    margin-bottom: var(--space-sm);
  }

  .env-form-actions {
    display: flex;
    gap: var(--space-sm);
    justify-content: flex-end;
  }

  .btn-sm {
    padding: var(--space-xs) var(--space-sm);
    font-size: var(--text-caption);
  }

  .env-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }

  .env-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-sm) var(--space-base);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-lg);
    background: var(--color-surface-card);
  }

  .env-card-main {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    flex: 1;
    min-width: 0;
  }

  .env-card-info {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .env-name {
    font-weight: 500;
    font-size: var(--text-body);
    color: var(--color-ink);
  }

  .env-url {
    font-size: var(--text-sm);
    color: var(--color-muted-soft);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .env-latency {
    font-size: var(--text-sm);
    color: var(--color-muted);
    flex-shrink: 0;
  }

  .env-card-actions {
    display: flex;
    gap: var(--space-xxs);
    flex-shrink: 0;
  }

  .btn-icon-danger:hover {
    color: var(--color-error);
  }

  .badge-muted {
    background: var(--color-muted-soft);
    color: var(--color-body);
    font-size: var(--text-xs);
    padding: 1px 6px;
    border-radius: var(--radius-sm);
  }

  .liveness-dot {
    display: inline-block;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .liveness-up {
    background: var(--color-success);
    box-shadow: 0 0 4px color-mix(in srgb, var(--color-success) 40%, transparent);
  }

  .liveness-down {
    background: var(--color-error);
    box-shadow: 0 0 4px color-mix(in srgb, var(--color-error) 40%, transparent);
  }

  .liveness-gray {
    background: var(--color-muted-soft);
  }
</style>
