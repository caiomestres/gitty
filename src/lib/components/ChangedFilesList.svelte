<script lang="ts">
  import type { ChangedFileDto } from "$lib/types/workspace";

  interface Props {
    files: ChangedFileDto[];
    limit?: number;
  }

  let { files, limit = 20 }: Props = $props();

  let expanded = $state(true);
  let showAll = $state(false);
</script>

<section class="changed-files-section">
  <button class="changed-files-toggle" type="button" onclick={() => (expanded = !expanded)}>
    <span class="toggle-icon">{expanded ? "▾" : "▸"}</span>
    Changed files ({files.length})
  </button>
  {#if expanded}
    <ul class="changed-files-list">
      {#each showAll ? files : files.slice(0, limit) as file (file.path)}
        <li class="changed-file-item">
          <span class="file-status status-{file.status}">{file.status[0]?.toUpperCase()}</span>
          <span class="file-path mono">{file.path}</span>
        </li>
      {/each}
    </ul>
    {#if files.length > limit}
      <button class="show-more-btn" type="button" onclick={() => (showAll = !showAll)}>
        {showAll ? "Show fewer" : `Show ${files.length - limit} more`}
      </button>
    {/if}
  {/if}
</section>

<style>
  .changed-files-section {
    margin-bottom: var(--space-xl);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-lg);
    background: var(--color-surface-card);
    overflow: hidden;
  }

  .changed-files-toggle {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    width: 100%;
    padding: var(--space-sm) var(--space-base);
    border: none;
    background: none;
    font-size: var(--text-body);
    font-weight: 600;
    color: var(--color-ink);
    cursor: pointer;
    text-align: left;
  }

  .changed-files-toggle:hover {
    background: var(--color-hairline-soft);
  }

  .toggle-icon {
    font-size: var(--text-2xs);
    color: var(--color-muted);
  }

  .changed-files-list {
    list-style: none;
    margin: 0;
    padding: 0 var(--space-base) var(--space-sm);
  }

  .changed-file-item {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-xxs) 0;
    font-size: var(--text-caption);
  }

  .file-status {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border-radius: var(--radius-sm);
    font-size: var(--text-xs);
    font-weight: 600;
    flex-shrink: 0;
  }

  .file-status.status-added,
  .file-status.status-untracked {
    background: color-mix(in srgb, var(--color-success) 15%, transparent);
    color: var(--color-success);
  }

  .file-status.status-modified {
    background: color-mix(in srgb, var(--color-warning) 15%, transparent);
    color: var(--color-warning);
  }

  .file-status.status-deleted {
    background: color-mix(in srgb, var(--color-error) 15%, transparent);
    color: var(--color-error);
  }

  .file-status.status-renamed {
    background: color-mix(in srgb, var(--color-primary) 15%, transparent);
    color: var(--color-primary);
  }

  .file-path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--color-body);
  }

  .show-more-btn {
    display: block;
    width: 100%;
    padding: var(--space-xs) var(--space-base) var(--space-sm);
    border: none;
    background: none;
    font-size: var(--text-sm);
    color: var(--color-primary);
    cursor: pointer;
    text-align: left;
  }

  .show-more-btn:hover {
    text-decoration: underline;
  }
</style>
