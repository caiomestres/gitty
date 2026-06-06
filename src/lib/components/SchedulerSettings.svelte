<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { MacroDto } from "$lib/types/workspace";
  import { handleError, success, type ActionFeedback } from "$lib/utils/error-handling";
  import type { SchedulerConfigDto, SchedulerMode } from "$lib/types/scheduler";
  import { DAY_OPTIONS, parseTime, formatTime } from "$lib/types/scheduler";

  interface Props {
    onFeedback: (fb: ActionFeedback | null) => void;
  }

  let { onFeedback }: Props = $props();

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
  let loading = $state(true);

  const windowWrapsMidnight = $derived.by(() => {
    const start = parseTime(windowStart);
    const end = parseTime(windowEnd);
    if (!start || !end) return false;
    return start.hour * 60 + start.minute > end.hour * 60 + end.minute;
  });

  function applyConfig(config: SchedulerConfigDto) {
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

  function buildTrigger(): SchedulerConfigDto["trigger"] {
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

  export async function load() {
    loading = true;
    try {
      const [config, m] = await Promise.all([
        invoke<SchedulerConfigDto>("get_scheduler_config"),
        invoke<MacroDto[]>("list_macros"),
      ]);
      applyConfig(config);
      macros = m;
    } catch {
      /* ignore */
    } finally {
      loading = false;
    }
  }

  async function save() {
    try {
      await invoke("set_scheduler_config", {
        config: {
          enabled: schedulerEnabled,
          trigger: buildTrigger(),
          power: { pause_on_battery: pauseOnBattery, battery_threshold: batteryThreshold },
          macro_id: macroId,
        },
      });
      onFeedback(success("Scheduler settings saved"));
    } catch (e) {
      onFeedback(handleError(e));
    }
  }

  function toggleDay(day: string) {
    if (selectedDays.includes(day)) {
      selectedDays = selectedDays.filter((d) => d !== day);
    } else {
      selectedDays = [...selectedDays, day];
    }
    save();
  }

  $effect(() => {
    load();
  });
</script>

<section class="settings-section">
  <div class="section-header">
    <h3 class="section-title">Scheduler</h3>
  </div>
  <p class="section-desc">
    Configure automatic background operations. When enabled, the scheduler periodically fetches all
    repositories to keep them fresh.
  </p>

  {#if loading}
    <div class="empty-state">Loading scheduler settings…</div>
  {:else}
    <div class="setting-row">
      <label class="setting-label">
        <input type="checkbox" bind:checked={schedulerEnabled} onchange={save} />
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
          onchange={save}
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
          onchange={save}
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
            onchange={save}
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
            onchange={save}
          />
        </label>
      </div>
      {#if windowWrapsMidnight}
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
          onchange={save}
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
          onchange={save}
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
          onchange={save}
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

<style>
  /* Shared .settings-*, .section-*, .setting-* live in global.css */
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
