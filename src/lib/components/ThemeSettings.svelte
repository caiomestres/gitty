<script lang="ts">
  import { onMount } from "svelte";
  import { getTheme, setTheme, THEMES, type ThemeId } from "$lib/utils/theme";
  import Check from "@lucide/svelte/icons/check";

  let currentTheme = $state<ThemeId>("default");
  let loading = $state(true);

  onMount(() => {
    loadTheme();
  });

  async function loadTheme() {
    try {
      currentTheme = await getTheme();
    } catch {
      currentTheme = "default";
    } finally {
      loading = false;
    }
  }

  async function selectTheme(themeId: ThemeId) {
    if (themeId === currentTheme) return;
    currentTheme = themeId;
    await setTheme(themeId);
  }
</script>

<section class="settings-section">
  <div class="section-header">
    <h3 class="section-title">Theme</h3>
  </div>

  <p class="section-desc">Choose your preferred visual style for the app.</p>

  <div class="theme-grid">
    {#each THEMES as theme (theme.id)}
      <button
        class="theme-card"
        class:active={currentTheme === theme.id}
        type="button"
        onclick={() => selectTheme(theme.id)}
        disabled={loading}
      >
        <!-- Preview miniature -->
        <div class="theme-preview" data-preview-theme={theme.id}>
          <div class="preview-canvas">
            <div class="preview-card">
              <div class="preview-line short"></div>
              <div class="preview-line"></div>
              <div class="preview-dot"></div>
            </div>
          </div>
        </div>

        <div class="theme-info">
          <div class="theme-header">
            <span class="theme-icon">
              <theme.icon size={16} />
            </span>
            <span class="theme-name">{theme.label}</span>
            {#if currentTheme === theme.id}
              <span class="theme-check">
                <Check size={14} />
              </span>
            {/if}
          </div>
          <p class="theme-description">{theme.description}</p>
        </div>
      </button>
    {/each}
  </div>
</section>

<style>
  .settings-section {
    margin-bottom: var(--space-xxl);
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-sm);
  }

  .section-title {
    font-size: var(--text-lg);
    font-weight: 600;
    color: var(--color-ink);
    margin: 0;
  }

  .section-desc {
    margin: 0 0 var(--space-lg);
    font-size: var(--text-body);
    color: var(--color-muted);
  }

  .theme-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: var(--space-base);
  }

  .theme-card {
    display: flex;
    flex-direction: column;
    padding: 0;
    background: var(--color-surface-card);
    border: 2px solid var(--color-hairline);
    border-radius: var(--radius-lg);
    cursor: pointer;
    transition:
      border-color 0.15s ease,
      box-shadow 0.15s ease,
      transform 0.1s ease;
    overflow: hidden;
  }

  .theme-card:hover {
    border-color: var(--color-hairline-strong);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
    transform: translateY(-1px);
  }

  .theme-card.active {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-primary) 15%, transparent);
  }

  .theme-card:active:not(:disabled) {
    transform: translateY(0);
  }

  .theme-card:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  /* Theme Preview Miniatures */
  .theme-preview {
    height: 80px;
    position: relative;
    overflow: hidden;
  }

  /* Default theme preview */
  .theme-preview[data-preview-theme="default"] .preview-canvas {
    background: #f7f7f4;
  }
  .theme-preview[data-preview-theme="default"] .preview-card {
    background: #ffffff;
    border-color: #e6e5e0;
  }
  .theme-preview[data-preview-theme="default"] .preview-line {
    background: #5a5852;
  }
  .theme-preview[data-preview-theme="default"] .preview-dot {
    background: #f54e00;
  }

  /* Dark theme preview */
  .theme-preview[data-preview-theme="dark"] .preview-canvas {
    background: #1a1a2e;
  }
  .theme-preview[data-preview-theme="dark"] .preview-card {
    background: #252536;
    border-color: #3a3a4a;
  }
  .theme-preview[data-preview-theme="dark"] .preview-line {
    background: #b8b8b8;
  }
  .theme-preview[data-preview-theme="dark"] .preview-dot {
    background: #ff6b35;
  }

  /* Brasil theme preview */
  .theme-preview[data-preview-theme="world-cup-brasil"] .preview-canvas {
    background: #fffdf5;
  }
  .theme-preview[data-preview-theme="world-cup-brasil"] .preview-card {
    background: #ffffff;
    border-color: #e8d5a3;
  }
  .theme-preview[data-preview-theme="world-cup-brasil"] .preview-line {
    background: #4a4a4a;
  }
  .theme-preview[data-preview-theme="world-cup-brasil"] .preview-dot {
    background: #009c3b;
  }

  .preview-canvas {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-sm);
  }

  .preview-card {
    width: 100%;
    max-width: 100px;
    padding: var(--space-xs);
    border: 1px solid;
    border-radius: var(--radius-sm);
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }

  .preview-line {
    height: 4px;
    border-radius: 2px;
    opacity: 0.6;
  }

  .preview-line.short {
    width: 60%;
  }

  .preview-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    margin-top: var(--space-xs);
  }

  .theme-info {
    padding: var(--space-sm) var(--space-base);
    border-top: 1px solid var(--color-hairline-soft);
    text-align: left;
  }

  .theme-header {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    margin-bottom: var(--space-xs);
  }

  .theme-icon {
    display: flex;
    align-items: center;
    color: var(--color-muted);
  }

  .theme-card.active .theme-icon {
    color: var(--color-primary);
  }

  .theme-name {
    font-size: var(--text-caption);
    font-weight: 600;
    color: var(--color-ink);
    flex: 1;
  }

  .theme-check {
    display: flex;
    align-items: center;
    color: var(--color-success);
  }

  .theme-description {
    margin: 0;
    font-size: var(--text-sm);
    color: var(--color-muted);
    line-height: 1.4;
  }
</style>
