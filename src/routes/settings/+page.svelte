<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { handleError, success, type ActionFeedback } from "$lib/utils/error-handling";
  import { onConfigChanged } from "$lib/utils/config-events";
  import FeedbackBanner from "$lib/components/FeedbackBanner.svelte";
  import PageError from "$lib/components/PageError.svelte";
  import SchedulerSettings from "$lib/components/SchedulerSettings.svelte";
  import NotificationSettings from "$lib/components/NotificationSettings.svelte";
  import Dialog from "$lib/components/Dialog.svelte";

  let scanRoots = $state<string[]>([]);
  let loading = $state(true);
  let pageError = $state<ActionFeedback | null>(null);
  let showAddDialog = $state(false);
  let newPath = $state("");
  let scanning = $state(false);
  let actionFeedback = $state<ActionFeedback | null>(null);

  let schedulerRef: SchedulerSettings | undefined = $state();
  let notifRef: NotificationSettings | undefined = $state();

  onMount(() => {
    loadScanRoots();
    return onConfigChanged(() => {
      loadScanRoots();
      schedulerRef?.load();
      notifRef?.load();
    });
  });

  async function loadScanRoots() {
    loading = true;
    pageError = null;
    try {
      scanRoots = await invoke<string[]>("list_scan_roots");
    } catch (e) {
      pageError = handleError(e);
    } finally {
      loading = false;
    }
  }

  async function handleAdd() {
    if (!newPath.trim()) return;
    scanning = true;
    actionFeedback = null;
    try {
      const result = await invoke<{ found: number; new: number }>("scan_directory", {
        path: newPath.trim(),
      });
      actionFeedback = success(`Scan complete: ${result.found} found, ${result.new} new`);
      showAddDialog = false;
      newPath = "";
      await loadScanRoots();
    } catch (e) {
      actionFeedback = handleError(e);
    } finally {
      scanning = false;
    }
  }

  async function handleRemove(path: string) {
    actionFeedback = null;
    try {
      await invoke("remove_scan_root", { path });
      actionFeedback = success(`Removed scan root: ${path}`);
      await loadScanRoots();
    } catch (e) {
      actionFeedback = handleError(e);
    }
  }

  async function handleRescan(path: string) {
    actionFeedback = null;
    try {
      const result = await invoke<{ found: number; new: number }>("scan_directory", { path });
      actionFeedback = success(`Rescanned ${path}: ${result.found} found, ${result.new} new`);
    } catch (e) {
      actionFeedback = handleError(e);
    }
  }

  function handleFeedback(fb: ActionFeedback | null) {
    actionFeedback = fb;
  }
</script>

<div class="settings">
  <header class="settings-header">
    <div>
      <h2 class="page-title">Settings</h2>
      <p class="page-subtitle">Manage Scan Roots and workspace configuration</p>
    </div>
  </header>

  <FeedbackBanner feedback={actionFeedback} />

  <section class="settings-section">
    <div class="section-header">
      <h3 class="section-title">Scan Roots</h3>
      <button class="btn-secondary" type="button" onclick={() => (showAddDialog = true)}>
        Add Scan Root
      </button>
    </div>

    <p class="section-desc">
      Directories scanned recursively for Git repositories. Adding a path triggers a scan that
      discovers and registers all repositories beneath it.
    </p>

    {#if loading}
      <div class="empty-state">Loading…</div>
    {:else if pageError}
      <PageError error={pageError} />
    {:else if scanRoots.length === 0}
      <div class="empty-state">
        <p>No Scan Roots configured.</p>
        <button class="btn-primary" type="button" onclick={() => (showAddDialog = true)}>
          Add Your First Scan Root
        </button>
      </div>
    {:else}
      <div class="scan-root-list">
        {#each scanRoots as root (root)}
          <div class="scan-root-item">
            <span class="scan-root-path mono">{root}</span>
            <div class="scan-root-actions">
              <button
                class="btn-icon"
                type="button"
                title="Rescan"
                onclick={() => handleRescan(root)}
              >
                ↻
              </button>
              <button
                class="btn-icon btn-danger"
                type="button"
                title="Remove"
                onclick={() => handleRemove(root)}
              >
                ×
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </section>

  <SchedulerSettings bind:this={schedulerRef} onFeedback={handleFeedback} />
  <NotificationSettings bind:this={notifRef} onFeedback={handleFeedback} />
</div>

{#if showAddDialog}
  <Dialog
    title="Add Scan Root"
    description="Enter the path to a directory. Gitty will recursively discover all Git repositories inside it."
    onClose={() => (showAddDialog = false)}
  >
    <input
      class="dialog-input mono"
      type="text"
      placeholder="C:\Users\you\projects"
      bind:value={newPath}
      onkeydown={(e) => e.key === "Enter" && handleAdd()}
    />
    {#snippet actions()}
      <button class="btn-secondary" type="button" onclick={() => (showAddDialog = false)}>
        Cancel
      </button>
      <button
        class="btn-primary"
        type="button"
        onclick={handleAdd}
        disabled={scanning || !newPath.trim()}
      >
        {scanning ? "Scanning…" : "Scan & Add"}
      </button>
    {/snippet}
  </Dialog>
{/if}

<style>
  .settings {
    padding: var(--space-xl);
    max-width: 960px;
  }

  .settings-header {
    margin-bottom: var(--space-xl);
  }

  .settings-section {
    margin-bottom: var(--space-xxl);
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-sm);
  }

  .section-title {
    font-size: var(--text-lg);
    font-weight: 600;
    color: var(--color-ink);
  }

  .section-desc {
    margin: 0 0 var(--space-base);
    font-size: var(--text-body);
    color: var(--color-muted);
  }

  .scan-root-list {
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-lg);
    background: var(--color-surface-card);
    overflow: hidden;
  }

  .scan-root-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-sm) var(--space-base);
    border-bottom: 1px solid var(--color-hairline-soft);
  }

  .scan-root-item:last-child {
    border-bottom: none;
  }

  .scan-root-path {
    font-size: var(--text-body);
    color: var(--color-body);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  .scan-root-actions {
    display: flex;
    gap: var(--space-xxs);
    flex-shrink: 0;
  }
</style>
