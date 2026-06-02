<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import JobResults from "$lib/components/JobResults.svelte";
  import type {
    MacroDto,
    StepDto,
    GroupDto,
    TagDto,
    SelectionDto,
    JobDto,
    RepoDto,
  } from "$lib/types/workspace";
  import { errorMessage } from "$lib/types/workspace";

  let macros = $state<MacroDto[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let actionMessage = $state<string | null>(null);

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
  let selectionKind = $state<"all" | "group" | "tag" | "multiple">("all");
  let selGroupId = $state("");
  let selTagName = $state("");
  let selRepoIds = $state<string[]>([]);
  let allRepos = $state<{ id: string; name: string }[]>([]);
  let groups = $state<GroupDto[]>([]);
  let tags = $state<TagDto[]>([]);
  let running = $state(false);

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
    error = null;
    try {
      macros = await invoke<MacroDto[]>("list_macros");
    } catch (e) {
      error = errorMessage(e);
    } finally {
      loading = false;
    }
  }

  function newStep(): StepDto {
    return {
      kind: { type: "git_op", op: "fetch" },
      condition: null,
      rollback: null,
      confirm: false,
    };
  }

  function openNewBuilder() {
    editingId = null;
    builderName = "";
    builderSteps = [newStep()];
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

  function addStep() {
    builderSteps = [...builderSteps, newStep()];
  }

  function removeStep(index: number) {
    builderSteps = builderSteps.filter((_, i) => i !== index);
  }

  function moveStep(index: number, dir: -1 | 1) {
    const target = index + dir;
    if (target < 0 || target >= builderSteps.length) return;
    const copy = [...builderSteps];
    [copy[index], copy[target]] = [copy[target], copy[index]];
    builderSteps = copy;
  }

  function setStepType(index: number, type: "git_op" | "shell") {
    const step = builderSteps[index];
    if (type === "git_op") {
      step.kind = { type: "git_op", op: "fetch" };
    } else {
      step.kind = { type: "shell", command: "", label: "" };
    }
    builderSteps = [...builderSteps];
  }

  function addVar() {
    builderVars = [...builderVars, { key: "", value: "" }];
  }

  function removeVar(index: number) {
    builderVars = builderVars.filter((_, i) => i !== index);
  }

  async function handleSave() {
    if (!builderName.trim() || builderSteps.length === 0) return;
    saving = true;
    actionMessage = null;
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
      actionMessage = editingId ? "Macro updated" : "Macro created";
      showBuilder = false;
      await loadMacros();
    } catch (e) {
      actionMessage = errorMessage(e);
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
    actionMessage = null;
    try {
      await invoke("delete_macro", { id: deleteTarget.id });
      actionMessage = `Deleted macro "${deleteTarget.name}"`;
      showDeleteDialog = false;
      deleteTarget = null;
      await loadMacros();
    } catch (e) {
      actionMessage = errorMessage(e);
    }
  }

  async function openRun(m: MacroDto) {
    runTarget = m;
    selectionKind = "all";
    selGroupId = "";
    selTagName = "";
    selRepoIds = [];
    showRunDialog = true;
    try {
      const [g, t, repos] = await Promise.all([
        invoke<GroupDto[]>("list_groups"),
        invoke<TagDto[]>("list_tags"),
        invoke<RepoDto[]>("list_repositories"),
      ]);
      groups = g;
      tags = t;
      allRepos = repos.filter((r) => r.state === "active").map((r) => ({ id: r.id, name: r.name }));
    } catch {
      // non-critical
    }
  }

  async function handleRun() {
    if (!runTarget) return;
    running = true;
    actionMessage = null;
    try {
      let selection: SelectionDto;
      if (selectionKind === "all") {
        selection = { kind: "all" };
      } else if (selectionKind === "group") {
        selection = { kind: "group", id: selGroupId };
      } else if (selectionKind === "tag") {
        selection = { kind: "tag", name: selTagName };
      } else {
        selection = { kind: "multiple", ids: selRepoIds };
      }

      results = await invoke<JobDto[]>("run_macro", {
        nameOrId: runTarget.id,
        selection,
      });
      showRunDialog = false;
      showResults = true;
    } catch (e) {
      actionMessage = errorMessage(e);
    } finally {
      running = false;
    }
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

  {#if actionMessage}
    <div class="action-banner" role="status">{actionMessage}</div>
  {/if}

  {#if showResults && results.length > 0}
    <JobResults jobs={results} onDismiss={() => (showResults = false)} />
  {/if}

  {#if loading}
    <div class="empty-state">Loading macros…</div>
  {:else if error}
    <div class="empty-state error">{error}</div>
  {:else if showBuilder}
    <!-- Builder Form -->
    <section class="builder">
      <h3 class="builder-title">{editingId ? "Edit Macro" : "New Macro"}</h3>

      <label class="field-label">
        Name
        <input class="field-input" type="text" bind:value={builderName} placeholder="Macro name" />
      </label>

      <div class="steps-section">
        <div class="steps-header">
          <span class="field-label">Steps</span>
          <button class="btn-secondary btn-sm" type="button" onclick={addStep}>+ Add Step</button>
        </div>

        {#each builderSteps as step, i (i)}
          <div class="step-card">
            <div class="step-header">
              <span class="step-number">#{i + 1}</span>
              <select
                class="step-type-select"
                value={step.kind.type}
                onchange={(e) =>
                  setStepType(i, (e.target as HTMLSelectElement).value as "git_op" | "shell")}
              >
                <option value="git_op">Git Operation</option>
                <option value="shell">Shell Command</option>
              </select>
              <div class="step-actions">
                <button
                  class="btn-icon btn-sm"
                  type="button"
                  title="Move up"
                  disabled={i === 0}
                  onclick={() => moveStep(i, -1)}>↑</button
                >
                <button
                  class="btn-icon btn-sm"
                  type="button"
                  title="Move down"
                  disabled={i === builderSteps.length - 1}
                  onclick={() => moveStep(i, 1)}>↓</button
                >
                <button
                  class="btn-icon btn-sm btn-danger"
                  type="button"
                  title="Remove"
                  onclick={() => removeStep(i)}>×</button
                >
              </div>
            </div>

            <div class="step-body">
              {#if step.kind.type === "git_op"}
                <label class="field-label field-inline">
                  Operation
                  <select
                    class="field-input"
                    value={step.kind.op}
                    onchange={(e) => {
                      const op = (e.target as HTMLSelectElement).value;
                      step.kind =
                        op === "checkout"
                          ? { type: "git_op", op, branch: "" }
                          : { type: "git_op", op };
                      builderSteps = [...builderSteps];
                    }}
                  >
                    <option value="fetch">Fetch</option>
                    <option value="pull">Pull</option>
                    <option value="checkout">Checkout</option>
                  </select>
                </label>
                {#if step.kind.op === "checkout"}
                  <label class="field-label field-inline">
                    Branch
                    <input
                      class="field-input"
                      type="text"
                      placeholder="branch name"
                      value={step.kind.branch ?? ""}
                      oninput={(e) => {
                        if (step.kind.type === "git_op") {
                          step.kind = {
                            type: "git_op",
                            op: "checkout",
                            branch: (e.target as HTMLInputElement).value,
                          };
                          builderSteps = [...builderSteps];
                        }
                      }}
                    />
                  </label>
                {/if}
              {:else}
                <label class="field-label field-inline">
                  Command
                  <input
                    class="field-input mono"
                    type="text"
                    placeholder="echo hello"
                    value={step.kind.command}
                    oninput={(e) => {
                      if (step.kind.type === "shell") {
                        step.kind = { ...step.kind, command: (e.target as HTMLInputElement).value };
                        builderSteps = [...builderSteps];
                      }
                    }}
                  />
                </label>
                <label class="field-label field-inline">
                  Label (optional)
                  <input
                    class="field-input"
                    type="text"
                    placeholder="step label"
                    value={step.kind.type === "shell" ? (step.kind.label ?? "") : ""}
                    oninput={(e) => {
                      if (step.kind.type === "shell") {
                        step.kind = {
                          ...step.kind,
                          label: (e.target as HTMLInputElement).value || undefined,
                        };
                        builderSteps = [...builderSteps];
                      }
                    }}
                  />
                </label>
              {/if}

              <label class="field-label field-inline">
                Condition (optional)
                <input
                  class="field-input"
                  type="text"
                  placeholder="condition expression"
                  value={step.condition ?? ""}
                  oninput={(e) => {
                    step.condition = (e.target as HTMLInputElement).value || null;
                    builderSteps = [...builderSteps];
                  }}
                />
              </label>

              <label class="field-checkbox">
                <input
                  type="checkbox"
                  checked={step.confirm}
                  onchange={() => {
                    step.confirm = !step.confirm;
                    builderSteps = [...builderSteps];
                  }}
                />
                Require confirmation
              </label>

              <div class="rollback-section">
                {#if step.rollback}
                  <div class="rollback-card">
                    <div class="rollback-header">
                      <span class="rollback-label">Rollback step</span>
                      <button
                        class="btn-icon btn-sm btn-danger"
                        type="button"
                        title="Remove rollback"
                        onclick={() => {
                          step.rollback = null;
                          builderSteps = [...builderSteps];
                        }}>×</button
                      >
                    </div>
                    <div class="rollback-body">
                      <label class="field-label field-inline">
                        Type
                        <select
                          class="field-input"
                          value={step.rollback.kind.type}
                          onchange={(e) => {
                            if (!step.rollback) return;
                            const t = (e.target as HTMLSelectElement).value;
                            step.rollback.kind =
                              t === "git_op"
                                ? { type: "git_op", op: "fetch" }
                                : { type: "shell", command: "", label: "" };
                            builderSteps = [...builderSteps];
                          }}
                        >
                          <option value="git_op">Git Operation</option>
                          <option value="shell">Shell Command</option>
                        </select>
                      </label>
                      {#if step.rollback.kind.type === "git_op"}
                        <label class="field-label field-inline">
                          Operation
                          <select
                            class="field-input"
                            value={step.rollback.kind.op}
                            onchange={(e) => {
                              if (!step.rollback || step.rollback.kind.type !== "git_op") return;
                              const op = (e.target as HTMLSelectElement).value;
                              step.rollback.kind =
                                op === "checkout"
                                  ? { type: "git_op", op, branch: "" }
                                  : { type: "git_op", op };
                              builderSteps = [...builderSteps];
                            }}
                          >
                            <option value="fetch">Fetch</option>
                            <option value="pull">Pull</option>
                            <option value="checkout">Checkout</option>
                          </select>
                        </label>
                        {#if step.rollback.kind.op === "checkout"}
                          <label class="field-label field-inline">
                            Branch
                            <input
                              class="field-input"
                              type="text"
                              placeholder="branch name"
                              value={step.rollback.kind.branch ?? ""}
                              oninput={(e) => {
                                if (!step.rollback || step.rollback.kind.type !== "git_op") return;
                                step.rollback.kind = {
                                  type: "git_op",
                                  op: "checkout",
                                  branch: (e.target as HTMLInputElement).value,
                                };
                                builderSteps = [...builderSteps];
                              }}
                            />
                          </label>
                        {/if}
                      {:else}
                        <label class="field-label field-inline">
                          Command
                          <input
                            class="field-input mono"
                            type="text"
                            placeholder="rollback command"
                            value={step.rollback.kind.command}
                            oninput={(e) => {
                              if (!step.rollback || step.rollback.kind.type !== "shell") return;
                              step.rollback.kind = {
                                ...step.rollback.kind,
                                command: (e.target as HTMLInputElement).value,
                              };
                              builderSteps = [...builderSteps];
                            }}
                          />
                        </label>
                      {/if}
                    </div>
                  </div>
                {:else}
                  <button
                    class="btn-link"
                    type="button"
                    onclick={() => {
                      step.rollback = {
                        kind: { type: "shell", command: "" },
                        condition: null,
                        rollback: null,
                        confirm: false,
                      };
                      builderSteps = [...builderSteps];
                    }}>+ Add rollback</button
                  >
                {/if}
              </div>
            </div>
          </div>
        {/each}
      </div>

      <!-- Variables -->
      <div class="vars-section">
        <div class="steps-header">
          <span class="field-label">Variables</span>
          <button class="btn-secondary btn-sm" type="button" onclick={addVar}>+ Add Variable</button
          >
        </div>
        {#each builderVars as v, i (i)}
          <div class="var-row">
            <input class="field-input mono" type="text" placeholder="key" bind:value={v.key} />
            <span class="var-eq">=</span>
            <input class="field-input" type="text" placeholder="value" bind:value={v.value} />
            <button class="btn-icon btn-sm btn-danger" type="button" onclick={() => removeVar(i)}>
              ×
            </button>
          </div>
        {/each}
      </div>

      <div class="builder-actions">
        <button class="btn-secondary" type="button" onclick={() => (showBuilder = false)}>
          Cancel
        </button>
        <button
          class="btn-primary"
          type="button"
          onclick={handleSave}
          disabled={saving || !builderName.trim() || builderSteps.length === 0}
        >
          {saving ? "Saving…" : "Save Macro"}
        </button>
      </div>
    </section>
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
  <div
    class="dialog-backdrop"
    role="presentation"
    onclick={() => (showRunDialog = false)}
    onkeydown={(e) => e.key === "Escape" && (showRunDialog = false)}
  >
    <div
      class="dialog dialog-wide"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <h3 class="dialog-title">Run "{runTarget.name}"</h3>
      <p class="dialog-desc">Select which repositories to run this macro against.</p>

      <div class="selection-options">
        <label class="radio-label">
          <input type="radio" name="sel" value="all" bind:group={selectionKind} />
          All Repositories
        </label>
        <label class="radio-label">
          <input type="radio" name="sel" value="group" bind:group={selectionKind} />
          By Group
        </label>
        {#if selectionKind === "group"}
          <select class="field-input sel-picker" bind:value={selGroupId}>
            <option value="">— Select group —</option>
            {#each groups as g (g.id)}
              <option value={g.id}>{g.name}</option>
            {/each}
          </select>
        {/if}
        <label class="radio-label">
          <input type="radio" name="sel" value="tag" bind:group={selectionKind} />
          By Tag
        </label>
        {#if selectionKind === "tag"}
          <select class="field-input sel-picker" bind:value={selTagName}>
            <option value="">— Select tag —</option>
            {#each tags as t (t.name)}
              <option value={t.name}>{t.name} ({t.repo_count})</option>
            {/each}
          </select>
        {/if}
        <label class="radio-label">
          <input type="radio" name="sel" value="multiple" bind:group={selectionKind} />
          Individual Repositories
        </label>
        {#if selectionKind === "multiple"}
          <div class="repo-checklist">
            {#each allRepos as repo (repo.id)}
              <label class="check-label">
                <input
                  type="checkbox"
                  value={repo.id}
                  checked={selRepoIds.includes(repo.id)}
                  onchange={() => {
                    if (selRepoIds.includes(repo.id)) {
                      selRepoIds = selRepoIds.filter((id) => id !== repo.id);
                    } else {
                      selRepoIds = [...selRepoIds, repo.id];
                    }
                  }}
                />
                {repo.name}
              </label>
            {/each}
            {#if allRepos.length === 0}
              <span class="sel-empty">No active repositories found.</span>
            {/if}
          </div>
        {/if}
      </div>

      <div class="dialog-actions">
        <button class="btn-secondary" type="button" onclick={() => (showRunDialog = false)}>
          Cancel
        </button>
        <button
          class="btn-primary"
          type="button"
          onclick={handleRun}
          disabled={running ||
            (selectionKind === "group" && !selGroupId) ||
            (selectionKind === "tag" && !selTagName) ||
            (selectionKind === "multiple" && selRepoIds.length === 0)}
        >
          {running ? "Executing…" : "Execute"}
        </button>
      </div>
    </div>
  </div>
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

  /* Macro list */
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
    font-size: 15px;
  }

  .macro-meta {
    font-size: 13px;
    color: var(--color-muted);
    margin-left: var(--space-sm);
  }

  .macro-actions {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
  }

  /* Builder */
  .builder {
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-lg);
    background: var(--color-surface-card);
    padding: var(--space-lg);
  }

  .builder-title {
    font-size: 18px;
    font-weight: 600;
    margin-bottom: var(--space-base);
  }

  .field-label {
    display: block;
    font-size: 13px;
    font-weight: 500;
    color: var(--color-muted);
    margin-bottom: var(--space-sm);
  }

  .field-inline {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    margin-bottom: var(--space-xs);
  }

  .field-input {
    flex: 1;
    padding: var(--space-xs) var(--space-sm);
    border: 1px solid var(--color-hairline-strong);
    border-radius: var(--radius-md);
    background: var(--color-canvas-soft);
    color: var(--color-ink);
    font-size: 14px;
    box-sizing: border-box;
  }

  .field-input:focus {
    outline: 2px solid color-mix(in srgb, var(--color-primary) 40%, transparent);
    outline-offset: 1px;
  }

  .field-checkbox {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    font-size: 13px;
    color: var(--color-body);
    cursor: pointer;
  }

  .steps-section,
  .vars-section {
    margin-top: var(--space-base);
  }

  .steps-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-sm);
  }

  .step-card {
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-md);
    margin-bottom: var(--space-sm);
    overflow: hidden;
  }

  .step-header {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-xs) var(--space-sm);
    background: var(--color-canvas-soft);
    border-bottom: 1px solid var(--color-hairline);
  }

  .step-number {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-muted);
    min-width: 24px;
  }

  .step-type-select {
    padding: var(--space-xxs) var(--space-xs);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-sm);
    background: var(--color-surface-card);
    font-size: 13px;
    color: var(--color-ink);
  }

  .step-actions {
    margin-left: auto;
    display: flex;
    gap: var(--space-xxs);
  }

  .step-body {
    padding: var(--space-sm);
  }

  .rollback-section {
    margin-top: var(--space-xs);
    padding-top: var(--space-xs);
    border-top: 1px dashed var(--color-hairline);
  }

  .rollback-card {
    border: 1px solid color-mix(in srgb, var(--color-primary) 20%, transparent);
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--color-primary) 3%, transparent);
    overflow: hidden;
  }

  .rollback-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-xxs) var(--space-sm);
    background: color-mix(in srgb, var(--color-primary) 6%, transparent);
    border-bottom: 1px solid color-mix(in srgb, var(--color-primary) 15%, transparent);
  }

  .rollback-label {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--color-primary);
  }

  .rollback-body {
    padding: var(--space-xs) var(--space-sm);
  }

  .var-row {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    margin-bottom: var(--space-xs);
  }

  .var-eq {
    color: var(--color-muted);
    font-weight: 600;
  }

  .builder-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-sm);
    margin-top: var(--space-lg);
    padding-top: var(--space-base);
    border-top: 1px solid var(--color-hairline);
  }

  .selection-options {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .radio-label {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    font-size: 14px;
    color: var(--color-body);
    cursor: pointer;
  }

  .sel-picker {
    margin-left: var(--space-lg);
    max-width: 300px;
  }

  .repo-checklist {
    margin-left: var(--space-lg);
    max-height: 200px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: var(--space-xxs);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-md);
    padding: var(--space-sm);
  }

  .check-label {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    font-size: 13px;
    color: var(--color-body);
    cursor: pointer;
  }

  .sel-empty {
    font-size: 13px;
    color: var(--color-muted);
    font-style: italic;
  }

  .mono {
    font-family: var(--font-mono);
  }
</style>
