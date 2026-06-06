<script lang="ts">
  import type { StepKindDto } from "$lib/types/workspace";

  interface Props {
    kind: StepKindDto;
    showRetry?: boolean;
    retry?: { max_attempts: number; backoff_seconds: number } | null;
    onKindChange: (kind: StepKindDto) => void;
    onRetryChange?: (retry: { max_attempts: number; backoff_seconds: number } | null) => void;
  }

  let { kind, showRetry = false, retry = null, onKindChange, onRetryChange }: Props = $props();
</script>

{#if kind.type === "git_op"}
  <label class="field-label field-inline">
    Operation
    <select
      class="field-input"
      value={kind.op}
      onchange={(e) => {
        const op = (e.target as HTMLSelectElement)
          .value as import("$lib/types/workspace").GitOpName;
        onKindChange(
          op === "checkout" ? { type: "git_op", op, branch: "" } : { type: "git_op", op },
        );
      }}
    >
      <option value="fetch">Fetch</option>
      <option value="pull">Pull</option>
      <option value="checkout">Checkout</option>
    </select>
  </label>
  {#if kind.op === "checkout"}
    <label class="field-label field-inline">
      Branch
      <input
        class="field-input"
        type="text"
        placeholder="branch name"
        value={kind.branch ?? ""}
        oninput={(e) => {
          onKindChange({
            type: "git_op",
            op: "checkout",
            branch: (e.target as HTMLInputElement).value,
          });
        }}
      />
    </label>
  {/if}

  {#if showRetry && onRetryChange}
    <label class="field-checkbox">
      <input
        type="checkbox"
        checked={!!retry}
        onchange={() => {
          onRetryChange!(retry ? null : { max_attempts: 3, backoff_seconds: 2 });
        }}
      />
      Retry on network error
    </label>
    {#if retry}
      <label class="field-label field-inline">
        Max attempts
        <input
          class="field-input"
          type="number"
          min="1"
          max="10"
          value={retry.max_attempts}
          oninput={(e) => {
            onRetryChange!({
              ...retry!,
              max_attempts: parseInt((e.target as HTMLInputElement).value) || 3,
            });
          }}
        />
      </label>
    {/if}
  {/if}
{:else}
  <label class="field-label field-inline">
    Command
    <input
      class="field-input mono"
      type="text"
      placeholder="echo hello"
      value={kind.command}
      oninput={(e) => {
        if (kind.type === "shell") {
          onKindChange({ ...kind, command: (e.target as HTMLInputElement).value });
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
      value={kind.type === "shell" ? (kind.label ?? "") : ""}
      oninput={(e) => {
        if (kind.type === "shell") {
          onKindChange({ ...kind, label: (e.target as HTMLInputElement).value || undefined });
        }
      }}
    />
  </label>
{/if}

<style>
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

  .mono {
    font-family: var(--font-mono);
  }
</style>
