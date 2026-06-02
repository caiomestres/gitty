<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let scanRoots = $state<string[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let showAddDialog = $state(false);
  let newPath = $state("");
  let scanning = $state(false);
  let actionMessage = $state<string | null>(null);

  $effect(() => {
    loadScanRoots();
  });

  async function loadScanRoots() {
    loading = true;
    error = null;
    try {
      scanRoots = await invoke<string[]>("list_scan_roots");
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function handleAdd() {
    if (!newPath.trim()) return;
    scanning = true;
    actionMessage = null;
    try {
      const result = await invoke<{ found: number; new: number }>("scan_directory", {
        path: newPath.trim(),
      });
      actionMessage = `Scan complete: ${result.found} found, ${result.new} new`;
      showAddDialog = false;
      newPath = "";
      await loadScanRoots();
    } catch (e) {
      actionMessage = `Error: ${String(e)}`;
    } finally {
      scanning = false;
    }
  }

  async function handleRemove(path: string) {
    actionMessage = null;
    try {
      await invoke("remove_scan_root", { path });
      actionMessage = `Removed scan root: ${path}`;
      await loadScanRoots();
    } catch (e) {
      actionMessage = `Error: ${String(e)}`;
    }
  }

  async function handleRescan(path: string) {
    actionMessage = null;
    try {
      const result = await invoke<{ found: number; new: number }>("scan_directory", { path });
      actionMessage = `Rescanned ${path}: ${result.found} found, ${result.new} new`;
    } catch (e) {
      actionMessage = `Error: ${String(e)}`;
    }
  }
</script>

<div class="settings">
  <header class="settings-header">
    <div>
      <h2 class="page-title">Settings</h2>
      <p class="page-subtitle">Manage Scan Roots and workspace configuration</p>
    </div>
  </header>

  {#if actionMessage}
    <div class="action-banner" role="status">{actionMessage}</div>
  {/if}

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
    {:else if error}
      <div class="empty-state error">{error}</div>
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
</div>

{#if showAddDialog}
  <div
    class="dialog-backdrop"
    role="presentation"
    onclick={() => (showAddDialog = false)}
    onkeydown={(e) => e.key === "Escape" && (showAddDialog = false)}
  >
    <div
      class="dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="add-scan-title"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <h3 id="add-scan-title" class="dialog-title">Add Scan Root</h3>
      <p class="dialog-desc">
        Enter the path to a directory. Gitty will recursively discover all Git repositories inside
        it.
      </p>
      <input
        class="dialog-input mono"
        type="text"
        placeholder="C:\Users\you\projects"
        bind:value={newPath}
        onkeydown={(e) => e.key === "Enter" && handleAdd()}
      />
      <div class="dialog-actions">
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
      </div>
    </div>
  </div>
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
    font-size: 18px;
    font-weight: 600;
    color: var(--color-ink);
  }

  .section-desc {
    margin: 0 0 var(--space-base);
    font-size: 14px;
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
    font-size: 14px;
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

  .mono {
    font-family: var(--font-mono);
  }
</style>
