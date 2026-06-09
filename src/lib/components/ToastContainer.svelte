<script lang="ts">
  import { getToasts, dismissToast } from "$lib/stores/toast.svelte";
  import X from "@lucide/svelte/icons/x";

  const toasts = $derived(getToasts());
</script>

{#if toasts.length > 0}
  <div class="toast-container" aria-live="polite">
    {#each toasts as toast (toast.id)}
      <div class="toast toast-{toast.severity}" role="alert">
        <div class="toast-content">
          <span class="toast-dot sev-{toast.severity}"></span>
          <div class="toast-text">
            <span class="toast-message">{toast.message}</span>
            {#if toast.hint}
              <span class="toast-hint">{toast.hint}</span>
            {/if}
          </div>
        </div>
        <button
          class="toast-dismiss"
          type="button"
          onclick={() => dismissToast(toast.id)}
          aria-label="Dismiss"
        >
          <X size={14} />
        </button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-container {
    position: fixed;
    top: var(--space-base);
    right: var(--space-base);
    z-index: 200;
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
    max-width: 360px;
  }

  .toast {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-sm);
    padding: var(--space-sm) var(--space-base);
    background: var(--color-surface-card);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-lg);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.1);
    animation: slide-in 0.2s ease-out;
  }

  @keyframes slide-in {
    from {
      opacity: 0;
      transform: translateX(20px);
    }
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }

  .toast-content {
    display: flex;
    align-items: flex-start;
    gap: var(--space-sm);
    min-width: 0;
  }

  .toast-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
    margin-top: 5px;
  }

  .toast-dot.sev-error {
    background: var(--color-error);
  }
  .toast-dot.sev-warning {
    background: var(--color-warning);
  }
  .toast-dot.sev-info {
    background: var(--color-primary);
  }

  .toast-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .toast-message {
    font-size: var(--text-caption);
    color: var(--color-body);
  }

  .toast-hint {
    font-size: var(--text-sm);
    color: var(--color-muted);
  }

  .toast-dismiss {
    background: none;
    border: none;
    color: var(--color-muted);
    font-size: var(--text-base);
    cursor: pointer;
    padding: 0 var(--space-xxs);
    line-height: 1;
    flex-shrink: 0;
  }

  .toast-dismiss:hover {
    color: var(--color-ink);
  }
</style>
