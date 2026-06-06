<script lang="ts">
  import type { ActionFeedback } from "$lib/utils/error-handling";

  interface Props {
    feedback: ActionFeedback | null;
    role?: string;
    class?: string;
  }

  let { feedback, role = "status", class: className = "" }: Props = $props();
</script>

{#if feedback}
  <div class={`feedback-banner feedback-${feedback.severity} ${className}`} {role}>
    {feedback.message}
    {#if feedback.hint}
      <p class="feedback-hint">{feedback.hint}</p>
    {/if}
  </div>
{/if}

<style>
  .feedback-banner {
    padding: var(--space-sm) var(--space-base);
    border-radius: var(--radius-md);
    font-size: var(--text-body);
    margin-bottom: var(--space-base);
  }

  .feedback-success {
    background: color-mix(in srgb, var(--color-success) 10%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-success) 30%, transparent);
    color: var(--color-success);
  }

  .feedback-error {
    background: color-mix(in srgb, var(--color-error) 10%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-error) 30%, transparent);
    color: var(--color-error);
  }

  .feedback-hint {
    margin: var(--space-xxs) 0 0;
    font-size: var(--text-caption);
    opacity: 0.85;
  }
</style>
