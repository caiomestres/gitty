<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import type { GroupDto } from "$lib/types/workspace";
  import { errorMessage } from "$lib/types/workspace";

  let groups = $state<GroupDto[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let actionMessage = $state<string | null>(null);

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
    error = null;
    try {
      groups = await invoke<GroupDto[]>("list_groups");
    } catch (e) {
      error = errorMessage(e);
    } finally {
      loading = false;
    }
  }

  function parentName(parentId: string | null): string {
    if (!parentId) return "—";
    return groups.find((g) => g.id === parentId)?.name ?? parentId;
  }

  async function handleCreate() {
    if (!createName.trim()) return;
    creating = true;
    actionMessage = null;
    try {
      await invoke("create_group", {
        name: createName.trim(),
        parentId: createParentId,
      });
      actionMessage = `Created group "${createName.trim()}"`;
      showCreateDialog = false;
      createName = "";
      createParentId = null;
      await loadGroups();
    } catch (e) {
      actionMessage = errorMessage(e);
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
    actionMessage = null;
    try {
      await invoke("rename_group", { id: renameId, newName: renameName.trim() });
      actionMessage = `Renamed group to "${renameName.trim()}"`;
      showRenameDialog = false;
      await loadGroups();
    } catch (e) {
      actionMessage = errorMessage(e);
    }
  }

  function openDelete(group: GroupDto) {
    deleteId = group.id;
    deleteName = group.name;
    showDeleteDialog = true;
  }

  async function handleDelete() {
    actionMessage = null;
    try {
      await invoke("delete_group", { id: deleteId });
      actionMessage = `Deleted group "${deleteName}"`;
      showDeleteDialog = false;
      await loadGroups();
    } catch (e) {
      actionMessage = errorMessage(e);
    }
  }

  function openMove(group: GroupDto) {
    moveId = group.id;
    moveParentId = group.parent_id;
    showMoveDialog = true;
  }

  async function handleMove() {
    actionMessage = null;
    try {
      await invoke("move_group", { id: moveId, newParentId: moveParentId || null });
      actionMessage = "Group moved successfully";
      showMoveDialog = false;
      await loadGroups();
    } catch (e) {
      actionMessage = errorMessage(e);
    }
  }
</script>

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

  {#if actionMessage}
    <div class="action-banner" role="status">{actionMessage}</div>
  {/if}

  {#if loading}
    <div class="empty-state">Loading groups…</div>
  {:else if error}
    <div class="empty-state error">{error}</div>
  {:else if groups.length === 0}
    <div class="empty-state">
      <p>No groups created yet.</p>
      <button class="btn-primary" type="button" onclick={() => (showCreateDialog = true)}>
        Create Your First Group
      </button>
    </div>
  {:else}
    <div class="group-table-wrap">
      <table class="group-table">
        <thead>
          <tr>
            <th>Name</th>
            <th>Parent</th>
            <th>Repos</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {#each groups as group (group.id)}
            <tr>
              <td class="col-name">{group.name}</td>
              <td class="col-parent">{parentName(group.parent_id)}</td>
              <td class="col-count">{group.repo_count}</td>
              <td class="col-actions">
                <button
                  class="btn-icon"
                  type="button"
                  title="Rename"
                  onclick={() => openRename(group)}>✎</button
                >
                <button class="btn-icon" type="button" title="Move" onclick={() => openMove(group)}
                  >↕</button
                >
                <button
                  class="btn-icon btn-danger"
                  type="button"
                  title="Delete"
                  onclick={() => openDelete(group)}>×</button
                >
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
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

  .group-table-wrap {
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-lg);
    background: var(--color-surface-card);
    overflow: hidden;
  }

  .group-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 14px;
  }

  .group-table th {
    text-align: left;
    padding: var(--space-sm) var(--space-base);
    font-size: 12px;
    font-weight: 600;
    color: var(--color-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    border-bottom: 1px solid var(--color-hairline);
    background: var(--color-canvas-soft);
  }

  .group-table td {
    padding: var(--space-sm) var(--space-base);
    border-bottom: 1px solid var(--color-hairline-soft);
    vertical-align: middle;
  }

  .group-table tr:last-child td {
    border-bottom: none;
  }

  .col-name {
    font-weight: 500;
    color: var(--color-ink);
  }
  .col-parent {
    color: var(--color-muted);
  }
  .col-count {
    color: var(--color-body);
  }
  .col-actions {
    white-space: nowrap;
  }

  .btn-icon {
    margin-right: var(--space-xxs);
  }
</style>
