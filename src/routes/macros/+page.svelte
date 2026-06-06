<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import JobResults from "$lib/components/JobResults.svelte";
  import MacroEditor from "$lib/components/MacroEditor.svelte";
  import MacroRunner from "$lib/components/MacroRunner.svelte";
  import type { MacroDto, StepDto, JobDto } from "$lib/types/workspace";
  import { handleError, type HandledError } from "$lib/utils/error-handling";
  import ErrorBanner from "$lib/components/ErrorBanner.svelte";

  let macros = $state<MacroDto[]>([]);
  let loading = $state(true);
  let pageError = $state<HandledError | null>(null);
  let actionFeedback = $state<HandledError | null>(null);

  // Builder state
  let showBuilder = $state(false);
  let editingId = $state<string | null>(null);
  let builderName = $state("");
  let builderSteps = $state<StepDto[]>([]);
  let builderVars = $state<{ key: string; value: string }[]>([]);
  let saving = $state(false);

  // Delete state
  let showDeleteDialog = $state(false);
  let deleteTarget = $state<MacroDto | null>(null);

  // Run state
  let showRunDialog = $state(false);
  let runTarget = $state<MacroDto | null>(null);

  // Results
  let showResults = $state(false);
  let results = $state<JobDto[]>([]);

  $effect(() => {
    loadMacros();
    let unlisten: (() => void) | undefined;
    listen("config-changed", () => loadMacros()).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  });

  async function loadMacros() {
    loading = true;
    pageError = null;
    try {
      macros = await invoke<MacroDto[]>("list_macros");
    } catch (e) {
      pageError = handleError(e);
    } finally {
      loading = false;
    }
  }

  function openNewBuilder() {
    editingId = null;
    builderName = "";
    builderSteps = [
      { kind: { type: "git_op", op: "fetch" }, condition: null, rollback: null, confirm: false },
    ];
    builderVars = [];
    showBuilder = true;
  }

  function openEditBuilder(m: MacroDto) {
    editingId = m.id;
    builderName = m.name;
    builderSteps = structuredClone(m.steps);
    builderVars = Object.entries(m.variables).map(([key, value]) => ({ key, value }));
    showBuilder = true;
  }

  async function handleSave() {
    if (!builderName.trim() || builderSteps.length === 0) return;
    saving = true;
    actionFeedback = null;
    try {
      if (editingId) {
        await invoke("delete_macro", { id: editingId });
      }
      const variables: Record<string, string> = {};
      for (const v of builderVars) {
        if (v.key.trim()) variables[v.key.trim()] = v.value;
      }
      await invoke("define_macro", {
        name: builderName.trim(),
        steps: builderSteps,
        variables,
      });
      actionFeedback = { message: editingId ? "Macro updated" : "Macro created" };
      showBuilder = false;
      await loadMacros();
    } catch (e) {
      actionFeedback = handleError(e);
    } finally {
      saving = false;
    }
  }

  function openDelete(m: MacroDto) {
    deleteTarget = m;
    showDeleteDialog = true;
  }

  async function handleDelete() {
    if (!deleteTarget) return;
    actionFeedback = null;
    try {
      await invoke("delete_macro", { id: deleteTarget.id });
      actionFeedback = { message: `Deleted macro "${deleteTarget.name}"` };
      showDeleteDialog = false;
      deleteTarget = null;
      await loadMacros();
    } catch (e) {
      actionFeedback = handleError(e);
    }
  }

  function openRun(m: MacroDto) {
    runTarget = m;
    showRunDialog = true;
  }
</script>

<div class="macros-page">
  <header class="page-header">
    <div>
      <h2 class="page-title">Macros</h2>
      <p class="page-subtitle">Multi-step workflows for bulk repository operations</p>
    </div>
    <button class="btn-primary" type="button" onclick={openNewBuilder}>New Macro</button>
  </header>

  <ErrorBanner error={actionFeedback} />

  {#if showResults && results.length > 0}
    <JobResults jobs={results} onDismiss={() => (showResults = false)} />
  {/if}

  {#if loading}
    <div class="empty-state">Loading macros…</div>
  {:else if pageError}
    <div class="empty-state error">{pageError.message}</div>
  {:else if showBuilder}
    <MacroEditor
      {editingId}
      bind:name={builderName}
      bind:steps={builderSteps}
      bind:variables={builderVars}
      {saving}
      onSave={handleSave}
      onCancel={() => (showBuilder = false)}
    />
  {:else if macros.length === 0}
    <div class="empty-state">
      <p>No macros defined yet.</p>
      <button class="btn-primary" type="button" onclick={openNewBuilder}>
        Create Your First Macro
      </button>
    </div>
  {:else}
    <div class="macro-list">
      {#each macros as m (m.id)}
        <div class="macro-card">
          <div class="macro-info">
            <span class="macro-name">{m.name}</span>
            <span class="macro-meta">{m.steps.length} step{m.steps.length !== 1 ? "s" : ""}</span>
          </div>
          <div class="macro-actions">
            <button class="btn-secondary btn-sm" type="button" onclick={() => openRun(m)}>
              Run
            </button>
            <button class="btn-icon" type="button" title="Edit" onclick={() => openEditBuilder(m)}
              >✎</button
            >
            <button
              class="btn-icon btn-danger"
              type="button"
              title="Delete"
              onclick={() => openDelete(m)}>×</button
            >
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<!-- Delete Confirmation -->
{#if showDeleteDialog && deleteTarget}
  <div
    class="dialog-backdrop"
    role="presentation"
    onclick={() => (showDeleteDialog = false)}
    onkeydown={(e) => e.key === "Escape" && (showDeleteDialog = false)}
  >
    <div
      class="dialog"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <h3 class="dialog-title">Delete Macro</h3>
      <p class="dialog-desc">
        Are you sure you want to delete <strong>{deleteTarget.name}</strong>?
      </p>
      <div class="dialog-actions">
        <button class="btn-secondary" type="button" onclick={() => (showDeleteDialog = false)}>
          Cancel
        </button>
        <button class="btn-danger-fill" type="button" onclick={handleDelete}>Delete</button>
      </div>
    </div>
  </div>
{/if}

<!-- Run Selection Dialog -->
{#if showRunDialog && runTarget}
  <MacroRunner
    macro={runTarget}
    onComplete={(jobs) => {
      showRunDialog = false;
      results = jobs;
      showResults = true;
    }}
    onCancel={() => (showRunDialog = false)}
    onError={(msg) => {
      actionFeedback = { message: msg };
    }}
  />
{/if}

<style>
  .macros-page {
    padding: var(--space-xl);
    max-width: 960px;
  }

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: var(--space-lg);
    margin-bottom: var(--space-xl);
  }

  .macro-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .macro-card {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-base);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-lg);
    background: var(--color-surface-card);
  }

  .macro-name {
    font-weight: 500;
    color: var(--color-ink);
    font-size: var(--text-body);
  }

  .macro-meta {
    font-size: var(--text-caption);
    color: var(--color-muted);
    margin-left: var(--space-sm);
  }

  .macro-actions {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
  }
</style>
