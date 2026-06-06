<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type {
    MacroDto,
    GroupDto,
    TagDto,
    SelectionDto,
    JobDto,
    RepoDto,
  } from "$lib/types/workspace";
  import { handleError } from "$lib/types/workspace";

  interface Props {
    macro: MacroDto;
    onComplete: (jobs: JobDto[]) => void;
    onCancel: () => void;
    onError: (msg: string) => void;
  }

  let { macro: runTarget, onComplete, onCancel, onError }: Props = $props();

  let selectionKind = $state<"all" | "group" | "tag" | "multiple">("all");
  let selGroupId = $state("");
  let selTagName = $state("");
  let selRepoIds = $state<string[]>([]);
  let allRepos = $state<{ id: string; name: string }[]>([]);
  let groups = $state<GroupDto[]>([]);
  let tags = $state<TagDto[]>([]);
  let running = $state(false);

  $effect(() => {
    loadSelectionData();
  });

  async function loadSelectionData() {
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
    running = true;
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

      const results = await invoke<JobDto[]>("run_macro", {
        nameOrId: runTarget.id,
        selection,
      });
      onComplete(results);
    } catch (e) {
      const handled = handleError(e);
      if (!handled.isTransient) {
        onError(handled.message);
      }
    } finally {
      running = false;
    }
  }

  let runDisabled = $derived(
    running ||
      (selectionKind === "group" && !selGroupId) ||
      (selectionKind === "tag" && !selTagName) ||
      (selectionKind === "multiple" && selRepoIds.length === 0),
  );
</script>

<div
  class="dialog-backdrop"
  role="presentation"
  onclick={onCancel}
  onkeydown={(e) => e.key === "Escape" && onCancel()}
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
      <button class="btn-secondary" type="button" onclick={onCancel}>Cancel</button>
      <button class="btn-primary" type="button" onclick={handleRun} disabled={runDisabled}>
        {running ? "Executing…" : "Execute"}
      </button>
    </div>
  </div>
</div>

<style>
  .selection-options {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .radio-label {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    font-size: var(--text-body);
    color: var(--color-body);
    cursor: pointer;
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
    font-size: var(--text-caption);
    color: var(--color-body);
    cursor: pointer;
  }

  .sel-empty {
    font-size: var(--text-caption);
    color: var(--color-muted);
    font-style: italic;
  }
</style>
