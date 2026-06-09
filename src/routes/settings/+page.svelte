<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getVersion } from "@tauri-apps/api/app";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { open } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";
  import { handleError, success, type ActionFeedback } from "$lib/utils/error-handling";
  import { addToast } from "$lib/stores/toast.svelte";
  import { onConfigChanged } from "$lib/utils/config-events";
  import FeedbackBanner from "$lib/components/FeedbackBanner.svelte";
  import PageError from "$lib/components/PageError.svelte";
  import SchedulerSettings from "$lib/components/SchedulerSettings.svelte";
  import NotificationSettings from "$lib/components/NotificationSettings.svelte";
  import ThemeSettings from "$lib/components/ThemeSettings.svelte";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Trash2 from "@lucide/svelte/icons/trash-2";

  let scanRoots = $state<string[]>([]);
  let loading = $state(true);
  let appVersion = $state("0.1.0");
  let pageError = $state<ActionFeedback | null>(null);
  let showManualInput = $state(false);
  let newPath = $state("");
  let scanning = $state(false);
  let actionFeedback = $state<ActionFeedback | null>(null);
  let dragOver = $state(false);

  let schedulerRef: SchedulerSettings | undefined = $state();
  let notifRef: NotificationSettings | undefined = $state();

  onMount(() => {
    loadScanRoots();
    getVersion()
      .then((v) => (appVersion = v))
      .catch(() => {});

    const webview = getCurrentWebviewWindow();
    const unlisten = webview.onDragDropEvent((event) => {
      if (event.payload.type === "over") {
        dragOver = true;
      } else if (event.payload.type === "drop") {
        dragOver = false;
        for (const path of event.payload.paths) {
          addScanRoot(path);
        }
      } else {
        dragOver = false;
      }
    });

    const unlistenConfig = onConfigChanged(() => {
      loadScanRoots();
      schedulerRef?.load();
      notifRef?.load();
    });

    return () => {
      unlisten.then((fn) => fn());
      unlistenConfig();
    };
  });

  async function loadScanRoots() {
    loading = true;
    pageError = null;
    try {
      scanRoots = await invoke<string[]>("list_scan_roots");
    } catch (e) {
      pageError = handleError(e);
    } finally {
      loading = false;
    }
  }

  async function openFolderPicker() {
    const selected = await open({ directory: true, multiple: false, title: "Select Scan Root" });
    if (selected) {
      await addScanRoot(selected);
    }
  }

  async function addScanRoot(path: string) {
    scanning = true;
    actionFeedback = null;
    try {
      const result = await invoke<{ found: number; new: number }>("scan_directory", { path });
      actionFeedback = success(`Scan complete: ${result.found} found, ${result.new} new`);
      await loadScanRoots();
    } catch (e) {
      const feedback = handleError(e);
      if (feedback) {
        addToast({
          message: feedback.message,
          hint: feedback.hint,
          severity: "error",
          dismissAfterMs: 5000,
        });
      }
    } finally {
      scanning = false;
    }
  }

  async function handleManualAdd() {
    if (!newPath.trim()) return;
    await addScanRoot(newPath.trim());
    showManualInput = false;
    newPath = "";
  }

  async function handleRemove(path: string) {
    actionFeedback = null;
    try {
      await invoke("remove_scan_root", { path });
      actionFeedback = success(`Removed scan root: ${path}`);
      await loadScanRoots();
    } catch (e) {
      actionFeedback = handleError(e);
    }
  }

  async function handleRescan(path: string) {
    actionFeedback = null;
    try {
      const result = await invoke<{ found: number; new: number }>("scan_directory", { path });
      actionFeedback = success(`Rescanned ${path}: ${result.found} found, ${result.new} new`);
    } catch (e) {
      actionFeedback = handleError(e);
    }
  }

  function handleFeedback(fb: ActionFeedback | null) {
    actionFeedback = fb;
  }
</script>

<div class="settings">
  <header class="settings-header">
    <div>
      <h2 class="page-title">Settings</h2>
      <p class="page-subtitle">Manage Scan Roots and workspace configuration</p>
    </div>
  </header>

  <FeedbackBanner feedback={actionFeedback} />

  <section class="settings-section">
    <div class="section-header">
      <h3 class="section-title">Scan Roots</h3>
      <button class="btn-secondary" type="button" onclick={openFolderPicker} disabled={scanning}>
        {scanning ? "Scanning…" : "Add Scan Root"}
      </button>
    </div>

    <p class="section-desc">
      Directories scanned recursively for Git repositories. Adding a path triggers a scan that
      discovers and registers all repositories beneath it.
    </p>

    <div class="drop-zone" class:drop-zone-active={dragOver}>
      <p class="drop-zone-label">
        {dragOver ? "Drop folder to add as Scan Root" : "Drag & drop a folder here to add it"}
      </p>
    </div>

    {#if loading}
      <div class="empty-state">Loading…</div>
    {:else if pageError}
      <PageError error={pageError} />
    {:else if scanRoots.length === 0}
      <div class="empty-state">
        <p>No Scan Roots configured.</p>
        <button class="btn-primary" type="button" onclick={openFolderPicker}>
          Add Your First Scan Root
        </button>
      </div>
    {:else}
      <div class="scan-root-list">
        {#each scanRoots as root (root)}
          <div class="scan-root-item">
            <span class="scan-root-path mono">{root}</span>
            <div class="scan-root-actions">
              <button
                class="btn-icon"
                type="button"
                title="Rescan"
                onclick={() => handleRescan(root)}
              >
                <RefreshCw size={14} />
              </button>
              <button
                class="btn-icon btn-danger"
                type="button"
                title="Remove"
                onclick={() => handleRemove(root)}
              >
                <Trash2 size={14} />
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}

    <button
      class="btn-link manual-toggle"
      type="button"
      onclick={() => (showManualInput = !showManualInput)}
    >
      {showManualInput ? "Hide manual input" : "Enter path manually"}
    </button>

    {#if showManualInput}
      <div class="manual-input-row">
        <input
          class="dialog-input mono"
          type="text"
          placeholder="C:\Users\you\projects"
          bind:value={newPath}
          onkeydown={(e) => e.key === "Enter" && handleManualAdd()}
        />
        <button
          class="btn-primary"
          type="button"
          onclick={handleManualAdd}
          disabled={scanning || !newPath.trim()}
        >
          {scanning ? "Scanning…" : "Scan & Add"}
        </button>
      </div>
    {/if}
  </section>

  <ThemeSettings />
  <SchedulerSettings bind:this={schedulerRef} onFeedback={handleFeedback} />
  <NotificationSettings bind:this={notifRef} onFeedback={handleFeedback} />

  <section class="settings-section about-section">
    <div class="section-header">
      <h3 class="section-title">About Gitty</h3>
    </div>
    <p class="about-version">Version {appVersion}</p>
    <p class="about-privacy">
      Gitty is fully offline. Your data never leaves your machine. There are no accounts, no
      telemetry, no analytics, no cloud sync. Network calls are only made to your own Git remotes
      and to liveness endpoints you explicitly configure.
    </p>
    <a class="about-link" href="https://github.com/caiomestres/gitty" target="_blank" rel="noopener"
      >github.com/caiomestres/gitty</a
    >
  </section>
</div>

<style>
  .settings {
    padding: var(--space-xl);
    max-width: 960px;
  }

  .settings-header {
    margin-bottom: var(--space-xl);
  }

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
  }

  .section-desc {
    margin: 0 0 var(--space-base);
    font-size: var(--text-body);
    color: var(--color-muted);
  }

  .scan-root-list {
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-lg);
    background: var(--color-surface-card);
    overflow: hidden;
  }

  .scan-root-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-sm) var(--space-base);
    border-bottom: 1px solid var(--color-hairline-soft);
  }

  .scan-root-item:last-child {
    border-bottom: none;
  }

  .scan-root-path {
    font-size: var(--text-body);
    color: var(--color-body);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  .scan-root-actions {
    display: flex;
    gap: var(--space-xxs);
    flex-shrink: 0;
  }

  .about-version {
    margin: 0 0 var(--space-sm);
    font-size: var(--text-body);
    font-weight: 600;
    color: var(--color-ink);
  }

  .about-privacy {
    margin: 0 0 var(--space-base);
    font-size: var(--text-body);
    color: var(--color-body);
    line-height: 1.6;
  }

  .about-link {
    font-size: var(--text-body);
    color: var(--color-accent);
    text-decoration: none;
  }

  .about-link:hover {
    text-decoration: underline;
  }

  .drop-zone {
    border: 2px dashed var(--color-hairline);
    border-radius: var(--radius-lg);
    padding: var(--space-base);
    text-align: center;
    margin-bottom: var(--space-base);
    transition:
      border-color 0.15s ease,
      background-color 0.15s ease;
  }

  .drop-zone-active {
    border-color: var(--color-accent);
    background-color: color-mix(in srgb, var(--color-accent) 8%, transparent);
  }

  .drop-zone-label {
    margin: 0;
    font-size: var(--text-body);
    color: var(--color-muted);
  }

  .drop-zone-active .drop-zone-label {
    color: var(--color-accent);
    font-weight: 500;
  }

  .manual-toggle {
    margin-top: var(--space-sm);
  }

  .btn-link {
    background: none;
    border: none;
    color: var(--color-accent);
    font-size: var(--text-body);
    cursor: pointer;
    padding: 0;
    text-decoration: underline;
  }

  .btn-link:hover {
    color: var(--color-ink);
  }

  .manual-input-row {
    display: flex;
    gap: var(--space-sm);
    margin-top: var(--space-sm);
    align-items: center;
  }

  .manual-input-row .dialog-input {
    flex: 1;
  }
</style>
