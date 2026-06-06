<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { SvelteSet } from "svelte/reactivity";
  import type { GroupDto, GroupTreeNodeDto } from "$lib/types/workspace";
  import { handleError, success, type ActionFeedback } from "$lib/utils/error-handling";
  import { onConfigChanged } from "$lib/utils/config-events";
  import FeedbackBanner from "$lib/components/FeedbackBanner.svelte";
  import PageError from "$lib/components/PageError.svelte";
  import Dialog from "$lib/components/Dialog.svelte";

  let groups = $state<GroupDto[]>([]);
  let tree = $state<GroupTreeNodeDto[]>([]);
  let collapsed = new SvelteSet<string>();
  let loading = $state(true);
  let pageError = $state<ActionFeedback | null>(null);
  let actionFeedback = $state<ActionFeedback | null>(null);

  type DialogState =
    | { kind: "none" }
    | { kind: "create"; name: string; parentId: string | null; saving: boolean }
    | { kind: "rename"; id: string; name: string }
    | { kind: "delete"; id: string; name: string }
    | { kind: "move"; id: string; parentId: string | null };

  let dialog = $state<DialogState>({ kind: "none" });

  const createDialog = $derived(dialog.kind === "create" ? dialog : null);
  const renameDialog = $derived(dialog.kind === "rename" ? dialog : null);
  const deleteDialog = $derived(dialog.kind === "delete" ? dialog : null);
  const moveDialog = $derived(dialog.kind === "move" ? dialog : null);

  $effect(() => {
    loadGroups();
    return onConfigChanged(() => loadGroups());
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
    if (dialog.kind !== "create" || !dialog.name.trim()) return;
    dialog = { ...dialog, saving: true };
    actionFeedback = null;
    try {
      await invoke("create_group", {
        name: dialog.name.trim(),
        parentId: dialog.parentId,
      });
      actionFeedback = success(`Created group "${dialog.name.trim()}"`);
      dialog = { kind: "none" };
      await loadGroups();
    } catch (e) {
      actionFeedback = handleError(e);
    } finally {
      if (dialog.kind === "create") dialog = { ...dialog, saving: false };
    }
  }

  function openRename(group: GroupDto) {
    dialog = { kind: "rename", id: group.id, name: group.name };
  }

  async function handleRename() {
    if (dialog.kind !== "rename" || !dialog.name.trim()) return;
    actionFeedback = null;
    try {
      await invoke("rename_group", { id: dialog.id, newName: dialog.name.trim() });
      actionFeedback = success(`Renamed group to "${dialog.name.trim()}"`);
      dialog = { kind: "none" };
      await loadGroups();
    } catch (e) {
      actionFeedback = handleError(e);
    }
  }

  function openDelete(group: GroupDto) {
    dialog = { kind: "delete", id: group.id, name: group.name };
  }

  async function handleDelete() {
    if (dialog.kind !== "delete") return;
    actionFeedback = null;
    try {
      await invoke("delete_group", { id: dialog.id });
      actionFeedback = success(`Deleted group "${dialog.name}"`);
      dialog = { kind: "none" };
      await loadGroups();
    } catch (e) {
      actionFeedback = handleError(e);
    }
  }

  function openMove(group: GroupDto) {
    dialog = { kind: "move", id: group.id, parentId: group.parent_id };
  }

  async function handleMove() {
    if (dialog.kind !== "move") return;
    actionFeedback = null;
    try {
      await invoke("move_group", { id: dialog.id, newParentId: dialog.parentId || null });
      actionFeedback = success("Group moved successfully");
      dialog = { kind: "none" };
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
    <button
      class="btn-primary"
      type="button"
      onclick={() => (dialog = { kind: "create", name: "", parentId: null, saving: false })}
    >
      Create Group
    </button>
  </header>

  <FeedbackBanner feedback={actionFeedback} />

  {#if loading}
    <div class="empty-state">Loading groups…</div>
  {:else if pageError}
    <PageError error={pageError} />
  {:else if tree.length === 0}
    <div class="empty-state">
      <p>No groups created yet.</p>
      <button
        class="btn-primary"
        type="button"
        onclick={() => (dialog = { kind: "create", name: "", parentId: null, saving: false })}
      >
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

{#if createDialog}
  <Dialog title="Create Group" onClose={() => (dialog = { kind: "none" })}>
    <label class="dialog-label">
      Name
      <input
        class="dialog-input"
        type="text"
        placeholder="Group name"
        bind:value={createDialog.name}
        onkeydown={(e) => e.key === "Enter" && handleCreate()}
      />
    </label>
    <label class="dialog-label">
      Parent (optional)
      <select class="dialog-select" bind:value={createDialog.parentId}>
        <option value={null}>— None (top-level) —</option>
        {#each groups as g (g.id)}
          <option value={g.id}>{g.name}</option>
        {/each}
      </select>
    </label>
    {#snippet actions()}
      <button class="btn-secondary" type="button" onclick={() => (dialog = { kind: "none" })}>
        Cancel
      </button>
      <button
        class="btn-primary"
        type="button"
        onclick={handleCreate}
        disabled={createDialog.saving || !createDialog.name.trim()}
      >
        {createDialog.saving ? "Creating…" : "Create"}
      </button>
    {/snippet}
  </Dialog>
{/if}

{#if renameDialog}
  <Dialog title="Rename Group" onClose={() => (dialog = { kind: "none" })}>
    <label class="dialog-label">
      New name
      <input
        class="dialog-input"
        type="text"
        bind:value={renameDialog.name}
        onkeydown={(e) => e.key === "Enter" && handleRename()}
      />
    </label>
    {#snippet actions()}
      <button class="btn-secondary" type="button" onclick={() => (dialog = { kind: "none" })}>
        Cancel
      </button>
      <button
        class="btn-primary"
        type="button"
        onclick={handleRename}
        disabled={!renameDialog.name.trim()}
      >
        Rename
      </button>
    {/snippet}
  </Dialog>
{/if}

{#if deleteDialog}
  <Dialog title="Delete Group" onClose={() => (dialog = { kind: "none" })}>
    <p class="dialog-desc">
      Are you sure you want to delete <strong>{deleteDialog.name}</strong>? Repositories in this
      group will be moved to Ungrouped, and child groups will be re-parented.
    </p>
    {#snippet actions()}
      <button class="btn-secondary" type="button" onclick={() => (dialog = { kind: "none" })}>
        Cancel
      </button>
      <button class="btn-danger-fill" type="button" onclick={handleDelete}>Delete</button>
    {/snippet}
  </Dialog>
{/if}

{#if moveDialog}
  <Dialog title="Move Group" onClose={() => (dialog = { kind: "none" })}>
    <label class="dialog-label">
      New parent
      <select class="dialog-select" bind:value={moveDialog.parentId}>
        <option value={null}>— None (top-level) —</option>
        {#each groups.filter((g) => g.id !== moveDialog.id) as g (g.id)}
          <option value={g.id}>{g.name}</option>
        {/each}
      </select>
    </label>
    {#snippet actions()}
      <button class="btn-secondary" type="button" onclick={() => (dialog = { kind: "none" })}>
        Cancel
      </button>
      <button class="btn-primary" type="button" onclick={handleMove}>Move</button>
    {/snippet}
  </Dialog>
{/if}

<style>
  .groups-page {
    padding: var(--space-xl);
    max-width: 960px;
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
