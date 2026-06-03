<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import type { NotificationConfigDto } from "$lib/types/notifications";
  import { getNotificationConfig, setNotificationConfig } from "$lib/types/notifications";

  let scanRoots = $state<string[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let showAddDialog = $state(false);
  let newPath = $state("");
  let scanning = $state(false);
  let actionMessage = $state<string | null>(null);

  // Scheduler state
  let schedulerEnabled = $state(false);
  let schedulerInterval = $state(30);
  let schedulerLoading = $state(true);

  // Notification state
  let notifTrigger = $state<NotificationConfigDto["trigger"]>("on_critical");
  let notifPollingMinutes = $state<number | null>(5);
  let notifLoading = $state(true);

  async function loadSchedulerConfig() {
    schedulerLoading = true;
    try {
      const status = await invoke<{
        enabled: boolean;
        last_run: string | null;
        next_run: string | null;
      }>("get_scheduler_status");
      schedulerEnabled = status.enabled;
    } catch {
      /* ignore */
    } finally {
      schedulerLoading = false;
    }
  }

  async function saveSchedulerConfig() {
    try {
      await invoke("set_scheduler_config", {
        config: {
          enabled: schedulerEnabled,
          trigger: { mode: "simple", interval_minutes: schedulerInterval },
          power: { pause_on_battery: true, battery_threshold: 20 },
          macro_id: null,
        },
      });
      actionMessage = "Scheduler settings saved";
    } catch (e) {
      actionMessage = `Error: ${String(e)}`;
    }
  }

  async function loadNotifConfig() {
    notifLoading = true;
    try {
      const cfg = await getNotificationConfig();
      notifTrigger = cfg.trigger;
      notifPollingMinutes = cfg.polling_interval_minutes;
    } catch {
      /* ignore */
    } finally {
      notifLoading = false;
    }
  }

  async function saveNotifConfig() {
    try {
      await setNotificationConfig({
        trigger: notifTrigger,
        polling_interval_minutes: notifPollingMinutes,
      });
      actionMessage = "Notification settings saved";
    } catch (e) {
      actionMessage = `Error: ${String(e)}`;
    }
  }

  onMount(() => {
    loadScanRoots();
    loadSchedulerConfig();
    loadNotifConfig();
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

  <section class="settings-section">
    <div class="section-header">
      <h3 class="section-title">Scheduler</h3>
    </div>
    <p class="section-desc">
      Configure automatic background operations. When enabled, the scheduler periodically fetches
      all repositories to keep them fresh.
    </p>

    {#if schedulerLoading}
      <div class="empty-state">Loading scheduler settings…</div>
    {:else}
      <div class="setting-row">
        <label class="setting-label">
          <input type="checkbox" bind:checked={schedulerEnabled} onchange={saveSchedulerConfig} />
          Enable scheduler
        </label>
      </div>
      <div class="setting-row">
        <label class="setting-label">
          Interval (minutes)
          <input
            type="number"
            class="setting-input"
            min="1"
            max="1440"
            bind:value={schedulerInterval}
            disabled={!schedulerEnabled}
            onchange={saveSchedulerConfig}
          />
        </label>
      </div>
    {/if}
  </section>

  <section class="settings-section">
    <div class="section-header">
      <h3 class="section-title">Notifications</h3>
    </div>
    <p class="section-desc">Configure when health notifications are generated.</p>

    {#if notifLoading}
      <div class="empty-state">Loading notification settings…</div>
    {:else}
      <div class="setting-row">
        <label class="setting-label">
          Trigger
          <select class="setting-select" bind:value={notifTrigger} onchange={saveNotifConfig}>
            <option value="on_critical">On critical only</option>
            <option value="on_any_change">On any change</option>
            <option value="on_scheduler_complete">On scheduler complete</option>
            <option value="disabled">Disabled</option>
          </select>
        </label>
      </div>
      <div class="setting-row">
        <label class="setting-label">
          Polling interval (minutes)
          <input
            type="number"
            class="setting-input"
            min="1"
            max="60"
            bind:value={notifPollingMinutes}
            onchange={saveNotifConfig}
          />
        </label>
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

  .setting-row {
    margin-bottom: var(--space-sm);
  }

  .setting-label {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    font-size: 14px;
    color: var(--color-body);
  }

  .setting-input {
    width: 80px;
    padding: var(--space-xs) var(--space-sm);
    border: 1px solid var(--color-hairline-strong);
    border-radius: var(--radius-md);
    background: var(--color-surface-card);
    color: var(--color-ink);
    font-size: 14px;
  }

  .setting-select {
    padding: var(--space-xs) var(--space-sm);
    border: 1px solid var(--color-hairline-strong);
    border-radius: var(--radius-md);
    background: var(--color-surface-card);
    color: var(--color-ink);
    font-size: 14px;
  }
</style>
