<script lang="ts">
  import type { NotificationDto } from "$lib/types/notifications";
  import { getNotifications, markNotificationRead } from "$lib/types/notifications";

  let notifications = $state<NotificationDto[]>([]);
  let open = $state(false);

  const unreadCount = $derived(notifications.filter((n) => !n.read).length);

  $effect(() => {
    loadNotifications();
    const interval = setInterval(loadNotifications, 60_000);
    return () => clearInterval(interval);
  });

  async function loadNotifications() {
    try {
      notifications = await getNotifications();
    } catch {
      notifications = [];
    }
  }

  function toggle() {
    open = !open;
    if (open) loadNotifications();
  }

  async function handleMarkRead(id: string) {
    await markNotificationRead(id);
    notifications = notifications.map((n) => (n.id === id ? { ...n, read: true } : n));
  }

  function formatTime(iso: string): string {
    try {
      const d = new Date(iso);
      return d.toLocaleDateString(undefined, {
        month: "short",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      });
    } catch {
      return iso;
    }
  }

  function close() {
    open = false;
  }
</script>

<div class="notification-wrapper">
  <button class="bell-btn" type="button" onclick={toggle} aria-label="Notifications">
    <span class="bell-icon" aria-hidden="true">🔔</span>
    {#if unreadCount > 0}
      <span class="badge">{unreadCount}</span>
    {/if}
  </button>

  {#if open}
    <div class="panel-backdrop" role="presentation" onclick={close}></div>
    <div class="panel" role="dialog" aria-label="Notifications">
      <div class="panel-header">
        <h3 class="panel-title">Notifications</h3>
      </div>
      <div class="panel-body">
        {#if notifications.length === 0}
          <div class="panel-empty">No notifications</div>
        {:else}
          {#each notifications as notif (notif.id)}
            <button
              class="notif-item"
              class:unread={!notif.read}
              type="button"
              onclick={() => handleMarkRead(notif.id)}
            >
              <span class="notif-dot sev-{notif.severity}"></span>
              <div class="notif-content">
                <span class="notif-title">{notif.title}</span>
                <span class="notif-body">{notif.body}</span>
                <span class="notif-time">{formatTime(notif.timestamp)}</span>
              </div>
            </button>
          {/each}
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .notification-wrapper {
    position: relative;
  }

  .bell-btn {
    position: relative;
    background: none;
    border: none;
    padding: var(--space-xs);
    cursor: pointer;
    font-size: 16px;
    line-height: 1;
    border-radius: var(--radius-md);
    transition: background 0.15s ease;
  }

  .bell-btn:hover {
    background: var(--color-hairline-soft);
  }

  .badge {
    position: absolute;
    top: -2px;
    right: -4px;
    min-width: 16px;
    height: 16px;
    border-radius: 999px;
    background: var(--color-error);
    color: white;
    font-size: 10px;
    font-weight: 700;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0 4px;
    line-height: 1;
  }

  .panel-backdrop {
    position: fixed;
    inset: 0;
    z-index: 99;
  }

  .panel {
    position: absolute;
    top: calc(100% + var(--space-xs));
    right: 0;
    width: 340px;
    max-height: 400px;
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-lg);
    background: var(--color-surface-card);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.08);
    z-index: 100;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .panel-header {
    padding: var(--space-sm) var(--space-base);
    border-bottom: 1px solid var(--color-hairline);
  }

  .panel-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-ink);
    margin: 0;
  }

  .panel-body {
    overflow-y: auto;
    flex: 1;
  }

  .panel-empty {
    padding: var(--space-xl) var(--space-base);
    text-align: center;
    font-size: 13px;
    color: var(--color-muted);
  }

  .notif-item {
    display: flex;
    align-items: flex-start;
    gap: var(--space-sm);
    padding: var(--space-sm) var(--space-base);
    border: none;
    background: none;
    width: 100%;
    text-align: left;
    cursor: pointer;
    border-bottom: 1px solid var(--color-hairline-soft);
    transition: background 0.1s ease;
  }

  .notif-item:last-child {
    border-bottom: none;
  }

  .notif-item:hover {
    background: var(--color-hairline-soft);
  }

  .notif-item.unread {
    background: var(--color-canvas-soft);
  }

  .notif-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
    margin-top: 5px;
  }

  .notif-dot.sev-info {
    background: var(--color-primary);
  }
  .notif-dot.sev-warning {
    background: var(--color-warning, #d97706);
  }
  .notif-dot.sev-critical {
    background: var(--color-error);
  }

  .notif-content {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .notif-title {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-ink);
  }

  .notif-body {
    font-size: 12px;
    color: var(--color-body);
  }

  .notif-time {
    font-size: 11px;
    color: var(--color-muted-soft);
  }
</style>
