<script lang="ts">
  interface Props {
    totalItems: number;
    pageSize: number;
    currentPage: number;
    onPageChange: (page: number) => void;
    onPageSizeChange: (size: number) => void;
  }

  const PAGE_SIZE_OPTIONS = [10, 25, 50, 100];

  let { totalItems, pageSize, currentPage, onPageChange, onPageSizeChange }: Props = $props();

  const totalPages = $derived(Math.max(1, Math.ceil(totalItems / pageSize)));
  const startItem = $derived((currentPage - 1) * pageSize + 1);
  const endItem = $derived(Math.min(currentPage * pageSize, totalItems));
  const visible = $derived(totalItems > pageSize);
</script>

{#if visible}
  <div class="pagination" aria-label="Pagination">
    <span class="pagination-info">
      Showing {startItem}–{endItem} of {totalItems}
    </span>

    <div class="pagination-controls">
      <button
        class="btn-page"
        type="button"
        disabled={currentPage <= 1}
        onclick={() => onPageChange(currentPage - 1)}
      >
        Prev
      </button>
      <span class="page-indicator">
        {currentPage} / {totalPages}
      </span>
      <button
        class="btn-page"
        type="button"
        disabled={currentPage >= totalPages}
        onclick={() => onPageChange(currentPage + 1)}
      >
        Next
      </button>
    </div>

    <label class="page-size-label">
      Rows
      <select
        class="page-size-select"
        value={pageSize}
        onchange={(e) => onPageSizeChange(Number(e.currentTarget.value))}
      >
        {#each PAGE_SIZE_OPTIONS as opt (opt)}
          <option value={opt}>{opt}</option>
        {/each}
      </select>
    </label>
  </div>
{/if}

<style>
  .pagination {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-base);
    padding: var(--space-sm) 0;
    margin-top: var(--space-sm);
    font-size: var(--text-caption);
    color: var(--color-muted);
  }

  .pagination-info {
    white-space: nowrap;
  }

  .pagination-controls {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
  }

  .btn-page {
    padding: var(--space-xs) var(--space-sm);
    border: 1px solid var(--color-hairline-strong);
    border-radius: var(--radius-md);
    background: var(--color-surface-card);
    color: var(--color-body);
    font-size: var(--text-caption);
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .btn-page:hover:not(:disabled) {
    background: var(--color-hairline-soft);
  }

  .btn-page:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .page-indicator {
    min-width: 3em;
    text-align: center;
    white-space: nowrap;
  }

  .page-size-label {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    white-space: nowrap;
  }

  .page-size-select {
    padding: var(--space-xs) var(--space-sm);
    border: 1px solid var(--color-hairline-strong);
    border-radius: var(--radius-md);
    background: var(--color-surface-card);
    color: var(--color-ink);
    font-size: var(--text-caption);
  }
</style>
