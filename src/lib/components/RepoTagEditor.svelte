<script lang="ts">
  import type { ActionFeedback } from "$lib/utils/error-handling";

  interface Props {
    tags: string[];
    onAdd: (tag: string) => void;
    onRemove: (tag: string) => void;
    error: ActionFeedback | null;
  }

  let { tags, onAdd, onRemove, error }: Props = $props();

  let newTag = $state("");

  function handleAdd() {
    if (!newTag.trim()) return;
    onAdd(newTag.trim());
    newTag = "";
  }
</script>

<section class="tags-section">
  <h3 class="section-title">Tags</h3>
  <div class="tag-list">
    {#each tags as tag (tag)}
      <span class="tag-pill">
        {tag}
        <button class="tag-remove" type="button" title="Remove tag" onclick={() => onRemove(tag)}
          >×</button
        >
      </span>
    {/each}
  </div>
  <div class="tag-add">
    <input
      class="tag-input"
      type="text"
      placeholder="Add tag…"
      bind:value={newTag}
      onkeydown={(e) => e.key === "Enter" && handleAdd()}
    />
    <button class="btn-secondary btn-sm" type="button" onclick={handleAdd} disabled={!newTag.trim()}
      >Add</button
    >
  </div>
  {#if error}
    <p class="tag-error">{error.message}</p>
    {#if error.hint}
      <p class="error-hint">{error.hint}</p>
    {/if}
  {/if}
</section>

<style>
  .tags-section {
    margin-bottom: var(--space-xl);
  }

  .section-title {
    font-size: var(--text-body);
    font-weight: 600;
    color: var(--color-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    margin-bottom: var(--space-sm);
  }

  .tag-list {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-xs);
    margin-bottom: var(--space-sm);
  }

  .tag-pill {
    display: inline-flex;
    align-items: center;
    gap: var(--space-xxs);
    padding: var(--space-xxs) var(--space-sm);
    border-radius: var(--radius-pill);
    background: var(--color-surface-strong);
    font-size: var(--text-caption);
    color: var(--color-ink);
  }

  .tag-remove {
    background: none;
    border: none;
    color: var(--color-muted);
    cursor: pointer;
    font-size: var(--text-body);
    line-height: 1;
    padding: 0 2px;
  }

  .tag-remove:hover {
    color: var(--color-error);
  }

  .tag-add {
    display: flex;
    gap: var(--space-xs);
    max-width: 300px;
  }

  .tag-input {
    flex: 1;
    padding: var(--space-xs) var(--space-sm);
    border: 1px solid var(--color-hairline-strong);
    border-radius: var(--radius-md);
    background: var(--color-canvas-soft);
    color: var(--color-ink);
    font-size: var(--text-caption);
  }

  .tag-input:focus {
    outline: 2px solid color-mix(in srgb, var(--color-primary) 40%, transparent);
    outline-offset: 1px;
  }

  .tag-error {
    margin: var(--space-xxs) 0 0;
    font-size: var(--text-sm);
    color: var(--color-error);
  }
</style>
