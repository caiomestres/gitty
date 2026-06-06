<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    title: string;
    description?: string;
    wide?: boolean;
    onClose: () => void;
    actions: Snippet;
    children: Snippet;
  }

  let { title, description, wide = false, onClose, actions, children }: Props = $props();
</script>

<div
  class="dialog-backdrop"
  role="presentation"
  onclick={onClose}
  onkeydown={(e) => e.key === "Escape" && onClose()}
>
  <div
    class={`dialog ${wide ? "dialog-wide" : ""}`}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
  >
    <h3 class="dialog-title">{title}</h3>
    {#if description}
      <p class="dialog-desc">{description}</p>
    {/if}
    {@render children()}
    <div class="dialog-actions">
      {@render actions()}
    </div>
  </div>
</div>
