<script lang="ts">
  import type { ActionFeedback } from "$lib/utils/error-handling";

  interface Props {
    error: ActionFeedback;
  }

  let { error }: Props = $props();
</script>

<div class="empty-state error">
  {error.message}
  {#if error.hint}
    <p class="error-hint">{error.hint}</p>
  {/if}
</div>
