<script lang="ts">
  import type { HandledError } from "$lib/utils/error-handling";

  interface Props {
    error: HandledError | null;
    role?: string;
    class?: string;
  }

  let { error, role = "status", class: className = "" }: Props = $props();
</script>

{#if error}
  <div class={`action-banner ${className}`} {role}>
    {error.message}
    {#if error.hint}
      <p class="error-hint">{error.hint}</p>
    {/if}
  </div>
{/if}
