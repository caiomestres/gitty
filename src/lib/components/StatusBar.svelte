<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let fetching = $state(false);

  async function handleFetchAll() {
    fetching = true;
    try {
      await invoke("fetch_all");
    } finally {
      fetching = false;
    }
  }
</script>

<header class="status-bar">
  <div class="status-left">
    <h1 class="app-title">Gitty</h1>
    <span class="app-subtitle">Workspace Manager</span>
  </div>

  <div class="status-actions">
    <button class="btn-secondary" onclick={handleFetchAll} disabled={fetching} type="button">
      {fetching ? "Fetching…" : "Fetch All"}
    </button>
  </div>
</header>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-sm) var(--space-lg);
    background: var(--color-canvas-soft);
    min-height: 48px;
  }

  .status-left {
    display: flex;
    align-items: baseline;
    gap: var(--space-sm);
  }

  .app-title {
    font-size: var(--text-lg);
    font-weight: 600;
    letter-spacing: -0.02em;
  }

  .app-subtitle {
    font-size: var(--text-caption);
    color: var(--color-muted);
  }

  .status-actions {
    display: flex;
    gap: var(--space-xs);
  }

  .btn-secondary {
    padding: var(--space-xs) var(--space-base);
    border: 1px solid var(--color-hairline-strong);
    border-radius: var(--radius-md);
    background: var(--color-surface-card);
    color: var(--color-ink);
    font-size: var(--text-caption);
    font-weight: 500;
    transition: background 0.15s ease;
  }

  .btn-secondary:hover:not(:disabled) {
    background: var(--color-hairline-soft);
  }

  .btn-secondary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
