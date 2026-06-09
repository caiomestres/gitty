<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { invoke } from "@tauri-apps/api/core";
  import FolderSearch from "@lucide/svelte/icons/folder-search";
  import BookOpen from "@lucide/svelte/icons/book-open";
  import ExternalLink from "@lucide/svelte/icons/external-link";
  import Mascot from "./Mascot.svelte";

  interface Props {
    onScanComplete: () => void;
  }

  let { onScanComplete }: Props = $props();

  let scanning = $state(false);

  async function openFolderPicker() {
    scanning = true;
    try {
      const selected = await open({ directory: true, multiple: false, title: "Select Scan Root" });
      if (selected) {
        await invoke("scan_directory", { path: selected });
        onScanComplete();
      }
    } finally {
      scanning = false;
    }
  }
</script>

<div class="onboarding-card">
  <div class="onboarding-hero">
    <Mascot size={64} />
    <h2 class="onboarding-title">Welcome to Gitty</h2>
    <p class="onboarding-subtitle">Your personal workspace manager for tracking Git repositories</p>
  </div>

  <div class="onboarding-content">
    <div class="onboarding-section">
      <h3 class="section-title">What are Scan Roots?</h3>
      <p class="section-text">
        Scan Roots are directories where Gitty looks for Git repositories. Add a folder containing
        your projects, and Gitty will automatically discover and track all repositories within it.
      </p>
    </div>

    <div class="onboarding-actions">
      <button
        class="btn-primary btn-large"
        type="button"
        onclick={openFolderPicker}
        disabled={scanning}
      >
        <FolderSearch size={20} />
        {scanning ? "Scanning…" : "Add Your First Scan Root"}
      </button>
    </div>

    <div class="onboarding-links">
      <a
        href="https://github.com/caiomestres/gitty#readme"
        target="_blank"
        rel="noopener"
        class="link-item"
      >
        <BookOpen size={16} />
        <span>Documentation</span>
      </a>
      <a
        href="https://github.com/caiomestres/gitty"
        target="_blank"
        rel="noopener"
        class="link-item"
      >
        <ExternalLink size={16} />
        <span>GitHub</span>
      </a>
    </div>
  </div>
</div>

<style>
  .onboarding-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    max-width: 560px;
    margin: var(--space-xxl) auto;
    padding: var(--space-xxl);
    background: var(--color-surface-card);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-xl);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
  }

  .onboarding-hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    margin-bottom: var(--space-xl);
  }

  :global(.onboarding-hero svg) {
    margin-bottom: var(--space-base);
  }

  .onboarding-title {
    font-size: var(--text-2xl);
    font-weight: 600;
    color: var(--color-ink);
    margin: 0 0 var(--space-xs);
    letter-spacing: -0.02em;
  }

  .onboarding-subtitle {
    font-size: var(--text-body);
    color: var(--color-muted);
    margin: 0;
    max-width: 400px;
  }

  .onboarding-content {
    width: 100%;
  }

  .onboarding-section {
    margin-bottom: var(--space-xl);
    text-align: center;
  }

  .section-title {
    font-size: var(--text-lg);
    font-weight: 600;
    color: var(--color-ink);
    margin: 0 0 var(--space-sm);
  }

  .section-text {
    font-size: var(--text-body);
    color: var(--color-body);
    line-height: 1.6;
    margin: 0;
  }

  .onboarding-actions {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
    margin-bottom: var(--space-xl);
  }

  .btn-large {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-sm);
    padding: var(--space-sm) var(--space-lg);
    font-size: var(--text-base);
    font-weight: 500;
  }

  .onboarding-links {
    display: flex;
    justify-content: center;
    gap: var(--space-lg);
  }

  .link-item {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    font-size: var(--text-caption);
    color: var(--color-muted);
    text-decoration: none;
    transition: color 0.15s ease;
  }

  .link-item:hover {
    color: var(--color-primary);
  }

  .link-item :global(svg) {
    color: var(--color-muted-soft);
  }

  .link-item:hover :global(svg) {
    color: var(--color-primary);
  }
</style>
