<script lang="ts">
  import type { StepDto } from "$lib/types/workspace";
  import StepKindEditor from "./StepKindEditor.svelte";

  interface Props {
    editingId: string | null;
    name: string;
    steps: StepDto[];
    variables: { key: string; value: string }[];
    saving: boolean;
    onSave: () => void;
    onCancel: () => void;
  }

  let {
    editingId,
    name = $bindable(),
    steps = $bindable(),
    variables = $bindable(),
    saving,
    onSave,
    onCancel,
  }: Props = $props();

  function newStep(): StepDto {
    return {
      kind: { type: "git_op", op: "fetch" },
      condition: null,
      rollback: null,
      confirm: false,
      retry: null,
    };
  }

  function addStep() {
    steps = [...steps, newStep()];
  }

  function removeStep(index: number) {
    steps = steps.filter((_, i) => i !== index);
  }

  function moveStep(index: number, dir: -1 | 1) {
    const target = index + dir;
    if (target < 0 || target >= steps.length) return;
    const copy = [...steps];
    [copy[index], copy[target]] = [copy[target], copy[index]];
    steps = copy;
  }

  function setStepType(index: number, type: "git_op" | "shell") {
    const step = steps[index];
    if (type === "git_op") {
      step.kind = { type: "git_op", op: "fetch" };
    } else {
      step.kind = { type: "shell", command: "", label: "" };
    }
    steps = [...steps];
  }

  function addVar() {
    variables = [...variables, { key: "", value: "" }];
  }

  function removeVar(index: number) {
    variables = variables.filter((_, i) => i !== index);
  }
</script>

<section class="builder">
  <h3 class="builder-title">{editingId ? "Edit Macro" : "New Macro"}</h3>

  <label class="field-label">
    Name
    <input class="field-input" type="text" bind:value={name} placeholder="Macro name" />
  </label>

  <div class="steps-section">
    <div class="steps-header">
      <span class="field-label">Steps</span>
      <button class="btn-secondary btn-sm" type="button" onclick={addStep}>+ Add Step</button>
    </div>

    {#each steps as step, i (i)}
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
              disabled={i === steps.length - 1}
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
          <StepKindEditor
            kind={step.kind}
            showRetry={step.kind.type === "git_op"}
            retry={step.retry}
            onKindChange={(k) => {
              step.kind = k;
              steps = [...steps];
            }}
            onRetryChange={(r) => {
              step.retry = r;
              steps = [...steps];
            }}
          />

          <label class="field-label field-inline">
            Condition (optional)
            <input
              class="field-input"
              type="text"
              placeholder="condition expression"
              value={step.condition ?? ""}
              oninput={(e) => {
                step.condition = (e.target as HTMLInputElement).value || null;
                steps = [...steps];
              }}
            />
          </label>

          <label class="field-checkbox">
            <input
              type="checkbox"
              checked={step.confirm}
              onchange={() => {
                step.confirm = !step.confirm;
                steps = [...steps];
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
                      steps = [...steps];
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
                        steps = [...steps];
                      }}
                    >
                      <option value="git_op">Git Operation</option>
                      <option value="shell">Shell Command</option>
                    </select>
                  </label>
                  <StepKindEditor
                    kind={step.rollback.kind}
                    onKindChange={(k) => {
                      if (!step.rollback) return;
                      step.rollback.kind = k;
                      steps = [...steps];
                    }}
                  />
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
                  steps = [...steps];
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
      <button class="btn-secondary btn-sm" type="button" onclick={addVar}>+ Add Variable</button>
    </div>
    {#each variables as v, i (i)}
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
    <button class="btn-secondary" type="button" onclick={onCancel}>Cancel</button>
    <button
      class="btn-primary"
      type="button"
      onclick={onSave}
      disabled={saving || !name.trim() || steps.length === 0}
    >
      {saving ? "Saving…" : "Save Macro"}
    </button>
  </div>
</section>

<style>
  .builder {
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-lg);
    background: var(--color-surface-card);
    padding: var(--space-lg);
  }

  .builder-title {
    font-size: var(--text-lg);
    font-weight: 600;
    margin-bottom: var(--space-base);
  }

  .field-label {
    display: block;
    font-size: var(--text-caption);
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
    font-size: var(--text-body);
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
    font-size: var(--text-caption);
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
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--color-muted);
    min-width: 24px;
  }

  .step-type-select {
    padding: var(--space-xxs) var(--space-xs);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-sm);
    background: var(--color-surface-card);
    font-size: var(--text-caption);
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
    font-size: var(--text-xs);
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

  .mono {
    font-family: var(--font-mono);
  }
</style>
