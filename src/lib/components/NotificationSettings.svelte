<script lang="ts">
  import { handleError, success, type ActionFeedback } from "$lib/utils/error-handling";
  import type { NotificationConfigDto } from "$lib/types/notifications";
  import { getNotificationConfig, setNotificationConfig } from "$lib/types/notifications";

  interface Props {
    onFeedback: (fb: ActionFeedback | null) => void;
  }

  let { onFeedback }: Props = $props();

  let trigger = $state<NotificationConfigDto["trigger"]>("on_critical");
  let pollingMinutes = $state<number | null>(5);
  let loading = $state(true);

  export async function load() {
    loading = true;
    try {
      const cfg = await getNotificationConfig();
      trigger = cfg.trigger;
      pollingMinutes = cfg.polling_interval_minutes;
    } catch {
      /* ignore */
    } finally {
      loading = false;
    }
  }

  async function save() {
    try {
      await setNotificationConfig({
        trigger,
        polling_interval_minutes: pollingMinutes,
      });
      onFeedback(success("Notification settings saved"));
    } catch (e) {
      onFeedback(handleError(e));
    }
  }

  $effect(() => {
    load();
  });
</script>

<section class="settings-section">
  <div class="section-header">
    <h3 class="section-title">Notifications</h3>
  </div>
  <p class="section-desc">Configure when health notifications are generated.</p>

  {#if loading}
    <div class="empty-state">Loading notification settings…</div>
  {:else}
    <div class="setting-row">
      <label class="setting-label">
        Trigger
        <select class="setting-select" bind:value={trigger} onchange={save}>
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
          bind:value={pollingMinutes}
          onchange={save}
        />
      </label>
    </div>
  {/if}
</section>

<style>
  /* Component-specific styles only; shared .settings-*, .section-*, .setting-* live in global.css */
</style>
