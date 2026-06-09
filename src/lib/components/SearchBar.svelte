<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import type { SearchResultDto } from "$lib/types/workspace";
  import Search from "@lucide/svelte/icons/search";

  let searchQuery = $state("");
  let searchResults = $state<SearchResultDto[]>([]);
  let showResults = $state(false);
  let containerRef = $state<HTMLDivElement | null>(null);

  $effect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (containerRef && !containerRef.contains(event.target as Node)) {
        showResults = false;
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  });

  $effect(() => {
    if (searchQuery.trim().length >= 2) {
      performSearch(searchQuery);
    } else {
      searchResults = [];
    }
  });

  async function performSearch(query: string) {
    try {
      const results = await invoke<SearchResultDto[]>("search_repositories", { query });
      searchResults = results.slice(0, 10);
      showResults = true;
    } catch {
      searchResults = [];
    }
  }

  function navigateToRepo(repoId: string) {
    showResults = false;
    searchQuery = "";
    goto(resolve(`/repo/${repoId}`));
  }
</script>

<div class="search-container" bind:this={containerRef}>
  <div class="search-input-wrapper">
    <Search size={16} class="search-icon" />
    <input
      type="text"
      class="search-input"
      placeholder="Search repositories..."
      bind:value={searchQuery}
      onfocus={() => {
        if (searchQuery.trim().length >= 2) showResults = true;
      }}
    />
  </div>

  {#if showResults && searchResults.length > 0}
    <div class="search-results">
      {#each searchResults as result (result.id)}
        <button class="search-result-item" type="button" onclick={() => navigateToRepo(result.id)}>
          <span class="result-name">{result.name}</span>
          <span class="result-path mono">{result.path}</span>
        </button>
      {/each}
    </div>
  {:else if showResults && searchQuery.trim().length >= 2 && searchResults.length === 0}
    <div class="search-results">
      <div class="search-no-results">No repositories found</div>
    </div>
  {/if}
</div>

<style>
  .search-container {
    position: relative;
    width: 100%;
  }

  .search-input-wrapper {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-xs) var(--space-sm);
    background: var(--color-surface-card);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-md);
    transition:
      border-color 0.15s ease,
      box-shadow 0.15s ease;
  }

  .search-input-wrapper:focus-within {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--color-primary) 15%, transparent);
  }

  :global(.search-icon) {
    color: var(--color-muted);
    flex-shrink: 0;
  }

  .search-input {
    flex: 1;
    border: none;
    background: transparent;
    font-size: var(--text-caption);
    color: var(--color-ink);
    outline: none;
    min-width: 0;
  }

  .search-input::placeholder {
    color: var(--color-muted-soft);
  }

  .search-results {
    position: absolute;
    top: calc(100% + var(--space-xs));
    left: 0;
    right: 0;
    background: var(--color-surface-card);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-md);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
    z-index: 100;
    max-height: 300px;
    overflow-y: auto;
  }

  .search-result-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: var(--space-sm) var(--space-base);
    border: none;
    background: none;
    width: 100%;
    text-align: left;
    cursor: pointer;
    border-bottom: 1px solid var(--color-hairline-soft);
    transition: background 0.1s ease;
  }

  .search-result-item:last-child {
    border-bottom: none;
  }

  .search-result-item:hover {
    background: var(--color-hairline-soft);
  }

  .result-name {
    font-size: var(--text-caption);
    font-weight: 500;
    color: var(--color-ink);
  }

  .result-path {
    font-size: var(--text-sm);
    color: var(--color-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .search-no-results {
    padding: var(--space-base);
    text-align: center;
    font-size: var(--text-caption);
    color: var(--color-muted);
  }
</style>
