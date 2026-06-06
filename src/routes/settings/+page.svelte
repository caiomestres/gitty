<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import type { NotificationConfigDto } from "$lib/types/notifications";
  import { getNotificationConfig, setNotificationConfig } from "$lib/types/notifications";
  import type { MacroDto } from "$lib/types/workspace";
  import { handleError } from "$lib/types/workspace";

  type SchedulerMode = "simple" | "advanced";

  interface TimeOfDay {
    hour: number;
    minute: number;
  }

  interface SchedulerConfigDto {
    enabled: boolean;
    macro_id: string | null;
    trigger:
      | { mode: "simple"; interval_minutes: number }
      | {
          mode: "advanced";
          interval_minutes: number;
          window_start: TimeOfDay;
          window_end: TimeOfDay;
          days: string[];
        };
    power: { pause_on_battery: boolean; battery_threshold: number };
  }

  const DAY_OPTIONS = [
    { key: "mon", label: "Mon" },
    { key: "tue", label: "Tue" },
    { key: "wed", label: "Wed" },
    { key: "thu", label: "Thu" },
    { key: "fri", label: "Fri" },
    { key: "sat", label: "Sat" },
    { key: "sun", label: "Sun" },
  ] as const;

  let scanRoots = $state<string[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let errorHint = $state<string | undefined>(undefined);
  let showAddDialog = $state(false);
  let newPath = $state("");
  let scanning = $state(false);
  let actionMessage = $state<string | null>(null);
  let actionHint = $state<string | undefined>(undefined);

  // Scheduler state
  let schedulerEnabled = $state(false);
  let schedulerInterval = $state(30);
  let schedulerMode = $state<SchedulerMode>("simple");
  let pauseOnBattery = $state(true);
  let batteryThreshold = $state(20);
  let macroId = $state<string | null>(null);
  let macros = $state<MacroDto[]>([]);
  let windowStart = $state("09:00");
  let windowEnd = $state("17:00");
  let selectedDays = $state<string[]>(["mon", "tue", "wed", "thu", "fri"]);
  let schedulerLoading = $state(true);

  const windowWrapsMidnight = $derived(() => {
    const start = parseTime(windowStart);
    const end = parseTime(windowEnd);
    if (!start || !end) return false;
    return start.hour * 60 + start.minute > end.hour * 60 + end.minute;
  });

  // Notification state
  let notifTrigger = $state<NotificationConfigDto["trigger"]>("on_critical");
  let notifPollingMinutes = $state<number | null>(5);
  let notifLoading = $state(true);

  function parseTime(value: string): TimeOfDay | null {
    const match = /^(\d{1,2}):(\d{2})$/.exec(value.trim());
    if (!match) return null;
    const hour = Number(match[1]);
    const minute = Number(match[2]);
    if (hour > 23 || minute > 59) return null;
    return { hour, minute };
  }

  function formatTime(t: TimeOfDay): string {
    return `${String(t.hour).padStart(2, "0")}:${String(t.minute).padStart(2, "0")}`;
  }

  function applySchedulerConfig(config: SchedulerConfigDto) {
    schedulerEnabled = config.enabled;
    macroId = config.macro_id;
    pauseOnBattery = config.power.pause_on_battery;
    batteryThreshold = config.power.battery_threshold;

    if (config.trigger.mode === "advanced") {
      schedulerMode = "advanced";
      schedulerInterval = config.trigger.interval_minutes;
      windowStart = formatTime(config.trigger.window_start);
      windowEnd = formatTime(config.trigger.window_end);
      selectedDays = [...config.trigger.days];
    } else {
      schedulerMode = "simple";
      schedulerInterval = config.trigger.interval_minutes;
    }
  }

  function buildTrigger():
    | { mode: "simple"; interval_minutes: number }
    | {
        mode: "advanced";
        interval_minutes: number;
        window_start: TimeOfDay;
        window_end: TimeOfDay;
        days: string[];
      } {
    if (schedulerMode === "simple") {
      return { mode: "simple", interval_minutes: schedulerInterval };
    }

    const start = parseTime(windowStart) ?? { hour: 9, minute: 0 };
    const end = parseTime(windowEnd) ?? { hour: 17, minute: 0 };
    return {
      mode: "advanced",
      interval_minutes: schedulerInterval,
      window_start: start,
      window_end: end,
      days: selectedDays,
    };
  }

  async function loadSchedulerConfig() {
    schedulerLoading = true;
    try {
      const config = await invoke<SchedulerConfigDto>("get_scheduler_config");
      applySchedulerConfig(config);
    } catch {
      /* ignore */
    } finally {
      schedulerLoading = false;
    }
  }

  async function loadMacros() {
    try {
      macros = await invoke<MacroDto[]>("list_macros");
    } catch {
      macros = [];
    }
  }

  async function saveSchedulerConfig() {
    try {
      await invoke("set_scheduler_config", {
        config: {
          enabled: schedulerEnabled,
          trigger: buildTrigger(),
          power: {
            pause_on_battery: pauseOnBattery,
            battery_threshold: batteryThreshold,
          },
          macro_id: macroId,
        },
      });
      actionMessage = "Scheduler settings saved";
    } catch (e) {
      const handled = handleError(e);
      if (!handled.isTransient) {
        actionMessage = handled.message;
        actionHint = handled.hint;
      }
    }
  }

  function toggleDay(day: string) {
    if (selectedDays.includes(day)) {
      selectedDays = selectedDays.filter((d) => d !== day);
    } else {
      selectedDays = [...selectedDays, day];
    }
    saveSchedulerConfig();
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
      const handled = handleError(e);
      if (!handled.isTransient) {
        actionMessage = handled.message;
        actionHint = handled.hint;
      }
    }
  }

  onMount(() => {
    loadScanRoots();
    loadSchedulerConfig();
    loadMacros();
    loadNotifConfig();

    let unlisten: (() => void) | undefined;
    listen("config-changed", () => {
      loadScanRoots();
      loadSchedulerConfig();
      loadMacros();
      loadNotifConfig();
    }).then((fn) => {
      unlisten = fn;
    });

    return () => unlisten?.();
  });

  async function loadScanRoots() {
    loading = true;
    error = null;
    errorHint = undefined;
    try {
      scanRoots = await invoke<string[]>("list_scan_roots");
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

  async function handleAdd() {
    if (!newPath.trim()) return;
    scanning = true;
    actionMessage = null;
    actionHint = undefined;
    try {
      const result = await invoke<{ found: number; new: number }>("scan_directory", {
        path: newPath.trim(),
      });
      actionMessage = `Scan complete: ${result.found} found, ${result.new} new`;
      showAddDialog = false;
      newPath = "";
      await loadScanRoots();
    } catch (e) {
      const handled = handleError(e);
      if (!handled.isTransient) {
        actionMessage = handled.message;
        actionHint = handled.hint;
      }
    } finally {
      scanning = false;
    }
  }

  async function handleRemove(path: string) {
    actionMessage = null;
    actionHint = undefined;
    try {
      await invoke("remove_scan_root", { path });
      actionMessage = `Removed scan root: ${path}`;
      await loadScanRoots();
    } catch (e) {
      const handled = handleError(e);
      if (!handled.isTransient) {
        actionMessage = handled.message;
        actionHint = handled.hint;
      }
    }
  }

  async function handleRescan(path: string) {
    actionMessage = null;
    actionHint = undefined;
    try {
      const result = await invoke<{ found: number; new: number }>("scan_directory", { path });
      actionMessage = `Rescanned ${path}: ${result.found} found, ${result.new} new`;
    } catch (e) {
      const handled = handleError(e);
      if (!handled.isTransient) {
        actionMessage = handled.message;
        actionHint = handled.hint;
      }
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
    <div class="action-banner" role="status">
      {actionMessage}
      {#if actionHint}
        <p class="error-hint">{actionHint}</p>
      {/if}
    </div>
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
      <div class="empty-state error">
        {error}
        {#if errorHint}
          <p class="error-hint">{errorHint}</p>
        {/if}
      </div>
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
          Mode
          <select
            class="setting-select"
            bind:value={schedulerMode}
            disabled={!schedulerEnabled}
            onchange={saveSchedulerConfig}
          >
            <option value="simple">Simple</option>
            <option value="advanced">Advanced</option>
          </select>
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
      {#if schedulerMode === "advanced"}
        <div class="setting-row">
          <label class="setting-label">
            Window start
            <input
              type="time"
              class="setting-input setting-input-time"
              bind:value={windowStart}
              disabled={!schedulerEnabled}
              onchange={saveSchedulerConfig}
            />
          </label>
        </div>
        <div class="setting-row">
          <label class="setting-label">
            Window end
            <input
              type="time"
              class="setting-input setting-input-time"
              bind:value={windowEnd}
              disabled={!schedulerEnabled}
              onchange={saveSchedulerConfig}
            />
          </label>
        </div>
        {#if windowWrapsMidnight()}
          <p class="setting-note">Window wraps past midnight</p>
        {/if}
        <div class="setting-row">
          <span class="setting-label">Days</span>
          <div class="day-checkboxes">
            {#each DAY_OPTIONS as day (day.key)}
              <label class="day-checkbox">
                <input
                  type="checkbox"
                  checked={selectedDays.includes(day.key)}
                  disabled={!schedulerEnabled}
                  onchange={() => toggleDay(day.key)}
                />
                {day.label}
              </label>
            {/each}
          </div>
        </div>
      {/if}
      <div class="setting-row">
        <label class="setting-label">
          <input
            type="checkbox"
            bind:checked={pauseOnBattery}
            onchange={saveSchedulerConfig}
            disabled={!schedulerEnabled}
          />
          Pause on battery
        </label>
      </div>
      <div class="setting-row">
        <label class="setting-label">
          Battery threshold (%)
          <input
            type="number"
            class="setting-input"
            min="5"
            max="100"
            bind:value={batteryThreshold}
            disabled={!schedulerEnabled || !pauseOnBattery}
            onchange={saveSchedulerConfig}
          />
        </label>
      </div>
      <div class="setting-row">
        <label class="setting-label">
          Macro
          <select
            class="setting-select"
            bind:value={macroId}
            disabled={!schedulerEnabled}
            onchange={saveSchedulerConfig}
          >
            <option value={null}>Default (fetch all)</option>
            {#each macros as m (m.id)}
              <option value={m.id}>{m.name}</option>
            {/each}
          </select>
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
    font-size: var(--text-body);
    color: var(--color-body);
  }

  .setting-input {
    width: 80px;
    padding: var(--space-xs) var(--space-sm);
    border: 1px solid var(--color-hairline-strong);
    border-radius: var(--radius-md);
    background: var(--color-surface-card);
    color: var(--color-ink);
    font-size: var(--text-body);
  }

  .setting-select {
    padding: var(--space-xs) var(--space-sm);
    border: 1px solid var(--color-hairline-strong);
    border-radius: var(--radius-md);
    background: var(--color-surface-card);
    color: var(--color-ink);
    font-size: var(--text-body);
  }

  .setting-input-time {
    width: 120px;
  }

  .setting-note {
    margin: 0 0 var(--space-sm);
    font-size: var(--text-caption);
    color: var(--color-muted);
  }

  .day-checkboxes {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-sm);
  }

  .day-checkbox {
    display: flex;
    align-items: center;
    gap: var(--space-xxs);
    font-size: var(--text-caption);
    color: var(--color-body);
  }
</style>
