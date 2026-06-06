<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { SvelteSet } from "svelte/reactivity";
  import type { GroupDto, GroupTreeNodeDto } from "$lib/types/workspace";
  import { handleError, type HandledError } from "$lib/utils/error-handling";
  import ErrorBanner from "$lib/components/ErrorBanner.svelte";

  let groups = $state<GroupDto[]>([]);
  let tree = $state<GroupTreeNodeDto[]>([]);
  let collapsed = new SvelteSet<string>();
  let loading = $state(true);
  let pageError = $state<HandledError | null>(null);
  let actionFeedback = $state<HandledError | null>(null);

  let showCreateDialog = $state(false);
  let createName = $state("");
  let createParentId = $state<string | null>(null);
  let creating = $state(false);

  let showRenameDialog = $state(false);
  let renameId = $state("");
  let renameName = $state("");

  let showDeleteDialog = $state(false);
  let deleteId = $state("");
  let deleteName = $state("");

  let showMoveDialog = $state(false);
  let moveId = $state("");
  let moveParentId = $state<string | null>(null);

  $effect(() => {
    loadGroups();
    let unlisten: (() => void) | undefined;
    listen("config-changed", () => loadGroups()).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  });

  async function loadGroups() {
    loading = true;
    pageError = null;
    try {
      [groups, tree] = await Promise.all([
        invoke<GroupDto[]>("list_groups"),
        invoke<GroupTreeNodeDto[]>("group_tree"),
      ]);
    } catch (e) {
      pageError = handleError(e);
    } finally {
      loading = false;
    }
  }

  function toggleNode(id: string) {
    if (collapsed.has(id)) {
      collapsed.delete(id);
    } else {
      collapsed.add(id);
    }
  }

  function repoCount(node: GroupTreeNodeDto): number {
    return node.repos.length + node.children.reduce((sum, child) => sum + repoCount(child), 0);
  }

  async function handleCreate() {
    if (!createName.trim()) return;
    creating = true;
    actionFeedback = null;
    try {
      await invoke("create_group", {
        name: createName.trim(),
        parentId: createParentId,
      });
      actionFeedback = { message: `Created group "${createName.trim()}"` };
      showCreateDialog = false;
      createName = "";
      createParentId = null;
      await loadGroups();
    } catch (e) {
      actionFeedback = handleError(e);
    } finally {
      creating = false;
    }
  }

  function openRename(group: GroupDto) {
    renameId = group.id;
    renameName = group.name;
    showRenameDialog = true;
  }

  async function handleRename() {
    if (!renameName.trim()) return;
    actionFeedback = null;
    try {
      await invoke("rename_group", { id: renameId, newName: renameName.trim() });
      actionFeedback = { message: `Renamed group to "${renameName.trim()}"` };
      showRenameDialog = false;
      await loadGroups();
    } catch (e) {
      actionFeedback = handleError(e);
    }
  }

  function openDelete(group: GroupDto) {
    deleteId = group.id;
    deleteName = group.name;
    showDeleteDialog = true;
  }

  async function handleDelete() {
    actionFeedback = null;
    try {
      await invoke("delete_group", { id: deleteId });
      actionFeedback = { message: `Deleted group "${deleteName}"` };
      showDeleteDialog = false;
      await loadGroups();
    } catch (e) {
      actionFeedback = handleError(e);
    }
  }

  function openMove(group: GroupDto) {
    moveId = group.id;
    moveParentId = group.parent_id;
    showMoveDialog = true;
  }

  async function handleMove() {
    actionFeedback = null;
    try {
      await invoke("move_group", { id: moveId, newParentId: moveParentId || null });
      actionFeedback = { message: "Group moved successfully" };
      showMoveDialog = false;
      await loadGroups();
    } catch (e) {
      actionFeedback = handleError(e);
    }
  }
</script>

{#snippet treeNode(node: GroupTreeNodeDto, depth: number)}
  <div class="tree-node" style="padding-left: {depth * 20}px">
    <div class="tree-row">
      {#if node.children.length > 0}
        <button
          class="tree-toggle"
          type="button"
          aria-label={collapsed.has(node.group.id) ? "Expand" : "Collapse"}
          onclick={() => toggleNode(node.group.id)}
        >
          {collapsed.has(node.group.id) ? "▸" : "▾"}
        </button>
      {:else}
        <span class="tree-spacer"></span>
      {/if}
      <span class="tree-name">{node.group.name}</span>
      {#if repoCount(node) === 0}
        <span class="tree-empty">(empty)</span>
      {:else}
        <span class="tree-count">{repoCount(node)}</span>
      {/if}
      <div class="tree-actions">
        <button class="btn-icon" type="button" title="Rename" onclick={() => openRename(node.group)}
          >✎</button
        >
        <button class="btn-icon" type="button" title="Move" onclick={() => openMove(node.group)}
          >↕</button
        >
        <button
          class="btn-icon btn-danger"
          type="button"
          title="Delete"
          onclick={() => openDelete(node.group)}>×</button
        >
      </div>
    </div>
    {#if !collapsed.has(node.group.id)}
      {#each node.children as child (child.group.id)}
        {@render treeNode(child, depth + 1)}
      {/each}
    {/if}
  </div>
{/snippet}

<div class="groups-page">
  <header class="page-header">
    <div>
      <h2 class="page-title">Groups</h2>
      <p class="page-subtitle">Organize repositories into hierarchical groups</p>
    </div>
    <button class="btn-primary" type="button" onclick={() => (showCreateDialog = true)}>
      Create Group
    </button>
  </header>

  <ErrorBanner error={actionFeedback} />

  {#if loading}
    <div class="empty-state">Loading groups…</div>
  {:else if pageError}
    <div class="empty-state error">
      {pageError.message}
      {#if pageError.hint}
        <p class="error-hint">{pageError.hint}</p>
      {/if}
    </div>
  {:else if tree.length === 0}
    <div class="empty-state">
      <p>No groups created yet.</p>
      <button class="btn-primary" type="button" onclick={() => (showCreateDialog = true)}>
        Create Your First Group
      </button>
    </div>
  {:else}
    <div class="group-tree-wrap">
      {#each tree as node (node.group.id)}
        {@render treeNode(node, 0)}
      {/each}
    </div>
  {/if}
</div>

<!-- Create Dialog -->
{#if showCreateDialog}
  <div
    class="dialog-backdrop"
    role="presentation"
    onclick={() => (showCreateDialog = false)}
    onkeydown={(e) => e.key === "Escape" && (showCreateDialog = false)}
  >
    <div
      class="dialog"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <h3 class="dialog-title">Create Group</h3>
      <label class="dialog-label">
        Name
        <input
          class="dialog-input"
          type="text"
          placeholder="Group name"
          bind:value={createName}
          onkeydown={(e) => e.key === "Enter" && handleCreate()}
        />
      </label>
      <label class="dialog-label">
        Parent (optional)
        <select class="dialog-select" bind:value={createParentId}>
          <option value={null}>— None (top-level) —</option>
          {#each groups as g (g.id)}
            <option value={g.id}>{g.name}</option>
          {/each}
        </select>
      </label>
      <div class="dialog-actions">
        <button class="btn-secondary" type="button" onclick={() => (showCreateDialog = false)}>
          Cancel
        </button>
        <button
          class="btn-primary"
          type="button"
          onclick={handleCreate}
          disabled={creating || !createName.trim()}
        >
          {creating ? "Creating…" : "Create"}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Rename Dialog -->
{#if showRenameDialog}
  <div
    class="dialog-backdrop"
    role="presentation"
    onclick={() => (showRenameDialog = false)}
    onkeydown={(e) => e.key === "Escape" && (showRenameDialog = false)}
  >
    <div
      class="dialog"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <h3 class="dialog-title">Rename Group</h3>
      <label class="dialog-label">
        New name
        <input
          class="dialog-input"
          type="text"
          bind:value={renameName}
          onkeydown={(e) => e.key === "Enter" && handleRename()}
        />
      </label>
      <div class="dialog-actions">
        <button class="btn-secondary" type="button" onclick={() => (showRenameDialog = false)}>
          Cancel
        </button>
        <button
          class="btn-primary"
          type="button"
          onclick={handleRename}
          disabled={!renameName.trim()}
        >
          Rename
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Delete Dialog -->
{#if showDeleteDialog}
  <div
    class="dialog-backdrop"
    role="presentation"
    onclick={() => (showDeleteDialog = false)}
    onkeydown={(e) => e.key === "Escape" && (showDeleteDialog = false)}
  >
    <div
      class="dialog"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <h3 class="dialog-title">Delete Group</h3>
      <p class="dialog-desc">
        Are you sure you want to delete <strong>{deleteName}</strong>? Repositories in this group
        will be moved to Ungrouped, and child groups will be re-parented.
      </p>
      <div class="dialog-actions">
        <button class="btn-secondary" type="button" onclick={() => (showDeleteDialog = false)}>
          Cancel
        </button>
        <button class="btn-danger-fill" type="button" onclick={handleDelete}>Delete</button>
      </div>
    </div>
  </div>
{/if}

<!-- Move Dialog -->
{#if showMoveDialog}
  <div
    class="dialog-backdrop"
    role="presentation"
    onclick={() => (showMoveDialog = false)}
    onkeydown={(e) => e.key === "Escape" && (showMoveDialog = false)}
  >
    <div
      class="dialog"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <h3 class="dialog-title">Move Group</h3>
      <label class="dialog-label">
        New parent
        <select class="dialog-select" bind:value={moveParentId}>
          <option value={null}>— None (top-level) —</option>
          {#each groups.filter((g) => g.id !== moveId) as g (g.id)}
            <option value={g.id}>{g.name}</option>
          {/each}
        </select>
      </label>
      <div class="dialog-actions">
        <button class="btn-secondary" type="button" onclick={() => (showMoveDialog = false)}>
          Cancel
        </button>
        <button class="btn-primary" type="button" onclick={handleMove}>Move</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .groups-page {
    padding: var(--space-xl);
    max-width: 960px;
  }

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: var(--space-lg);
    margin-bottom: var(--space-xl);
  }

  .group-tree-wrap {
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-lg);
    background: var(--color-surface-card);
    overflow: hidden;
    padding: var(--space-xs) 0;
  }

  .tree-node {
    font-size: var(--text-body);
  }

  .tree-row {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-xs) var(--space-base);
    border-bottom: 1px solid var(--color-hairline-soft);
  }

  .tree-node:last-child .tree-row {
    border-bottom: none;
  }

  .tree-toggle {
    width: 18px;
    height: 18px;
    padding: 0;
    border: none;
    background: none;
    color: var(--color-muted);
    font-size: var(--text-2xs);
    cursor: pointer;
    flex-shrink: 0;
  }

  .tree-spacer {
    width: 18px;
    flex-shrink: 0;
  }

  .tree-name {
    font-weight: 500;
    color: var(--color-ink);
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tree-empty {
    font-size: var(--text-sm);
    color: var(--color-muted-soft);
    flex-shrink: 0;
  }

  .tree-count {
    font-size: var(--text-sm);
    color: var(--color-muted);
    flex-shrink: 0;
  }

  .tree-actions {
    display: flex;
    gap: var(--space-xxs);
    flex-shrink: 0;
    margin-left: auto;
  }

  .btn-icon {
    margin-right: 0;
  }
</style>
