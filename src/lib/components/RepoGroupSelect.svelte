<script lang="ts">
  import type { GroupDto } from "$lib/types/workspace";

  interface Props {
    groups: GroupDto[];
    selectedGroupId: string;
    disabled: boolean;
    onchange: (e: Event) => void;
  }

  let { groups, selectedGroupId, disabled, onchange }: Props = $props();
</script>

<section class="group-section">
  <h3 class="section-title">Group</h3>
  <div class="group-select-wrap">
    <select class="group-select" value={selectedGroupId} {onchange} {disabled}>
      <option value="">Ungrouped</option>
      {#each groups as g (g.id)}
        <option value={g.id}>{g.name}</option>
      {/each}
    </select>
  </div>
</section>

<style>
  .group-section {
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

  .group-select-wrap {
    max-width: 300px;
  }

  .group-select {
    width: 100%;
    padding: var(--space-sm) var(--space-base);
    border: 1px solid var(--color-hairline-strong);
    border-radius: var(--radius-md);
    background: var(--color-surface-card);
    color: var(--color-ink);
    font-size: var(--text-body);
  }

  .group-select:disabled {
    opacity: 0.6;
  }
</style>
