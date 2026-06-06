<script lang="ts">
  import { page } from "$app/stores";
  import { resolve } from "$app/paths";
  import { invoke } from "@tauri-apps/api/core";
  import { SvelteSet } from "svelte/reactivity";
  import type { GroupTreeNodeDto } from "$lib/types/workspace";
  import { onConfigChanged } from "$lib/utils/config-events";

  const navItems: {
    href: "/" | "/groups" | "/macros" | "/health" | "/changes" | "/settings";
    label: string;
    icon: string;
  }[] = [
    { href: "/", label: "Dashboard", icon: "◫" },
    { href: "/health", label: "Health", icon: "♥" },
    { href: "/changes", label: "Changes", icon: "⟳" },
    { href: "/groups", label: "Groups", icon: "⊞" },
    { href: "/macros", label: "Macros", icon: "⚡" },
    { href: "/settings", label: "Settings", icon: "⚙" },
  ];

  let tree = $state<GroupTreeNodeDto[]>([]);
  let collapsed = new SvelteSet<string>();

  $effect(() => {
    loadTree();
    return onConfigChanged(() => loadTree());
  });

  async function loadTree() {
    try {
      tree = await invoke<GroupTreeNodeDto[]>("group_tree");
    } catch {
      tree = [];
    }
  }

  function toggleNode(id: string) {
    if (collapsed.has(id)) {
      collapsed.delete(id);
    } else {
      collapsed.add(id);
    }
  }

  function hasAnyRepos(nodes: GroupTreeNodeDto[]): boolean {
    return nodes.some(
      (n) => n.repos.length > 0 || (n.children.length > 0 && hasAnyRepos(n.children)),
    );
  }

  const anyRepos = $derived(hasAnyRepos(tree));
</script>

{#snippet treeNode(node: GroupTreeNodeDto, depth: number)}
  <div class="tree-group">
    <button
      class="tree-toggle"
      style="padding-left: {12 + depth * 14}px"
      onclick={() => toggleNode(node.group.id)}
    >
      <span class="toggle-icon" aria-hidden="true">
        {collapsed.has(node.group.id) ? "▸" : "▾"}
      </span>
      <span class="group-name">{node.group.name}</span>
      {#if node.repos.length > 0}
        <span class="group-count">{node.repos.length}</span>
      {:else if node.repos.length === 0 && !hasAnyRepos(node.children)}
        <span class="group-empty">(empty)</span>
      {/if}
    </button>

    {#if !collapsed.has(node.group.id)}
      {#each node.repos as repo (repo.id)}
        <a
          href={resolve(`/repo/${repo.id}`)}
          class="tree-repo"
          class:active={$page.url.pathname === `/repo/${repo.id}`}
          class:missing={repo.state === "missing"}
          style="padding-left: {12 + (depth + 1) * 14}px"
        >
          {repo.name}
        </a>
      {/each}

      {#each node.children as child (child.group.id)}
        {@render treeNode(child, depth + 1)}
      {/each}
    {/if}
  </div>
{/snippet}

<nav class="sidebar" aria-label="Main navigation">
  <div class="sidebar-brand">
    <span class="brand-mark" aria-hidden="true">G</span>
    <span class="brand-name">Gitty</span>
  </div>

  <ul class="nav-list">
    {#each navItems as item (item.href)}
      <li>
        <a
          href={resolve(item.href)}
          class="nav-link"
          class:active={$page.url.pathname === item.href}
          aria-current={$page.url.pathname === item.href ? "page" : undefined}
        >
          <span class="nav-icon" aria-hidden="true">{item.icon}</span>
          {item.label}
        </a>
      </li>
    {/each}
  </ul>

  <div class="tree-section">
    <span class="tree-heading">Explorer</span>
    {#if tree.length === 0 || !anyRepos}
      <div class="tree-empty-state">
        <p>No repositories. Scan a directory to get started.</p>
        <a href={resolve("/settings")} class="tree-empty-link">Go to Settings</a>
      </div>
    {/if}
    {#if tree.length > 0}
      <div class="tree-root">
        {#each tree as node (node.group.id)}
          {@render treeNode(node, 0)}
        {/each}
      </div>
    {/if}
  </div>
</nav>

<style>
  .sidebar {
    width: 220px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    background: var(--color-canvas-soft);
    border-right: 1px solid var(--color-hairline);
    overflow-y: auto;
  }

  .sidebar-brand {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-base);
  }

  .brand-mark {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-md);
    background: var(--color-primary);
    color: var(--color-on-primary);
    font-size: var(--text-body);
    font-weight: 600;
  }

  .brand-name {
    font-size: var(--text-base);
    font-weight: 600;
    color: var(--color-ink);
    letter-spacing: -0.02em;
  }

  .nav-list {
    list-style: none;
    margin: 0;
    padding: 0 var(--space-xs);
    display: flex;
    flex-direction: column;
    gap: var(--space-xxs);
  }

  .nav-link {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding: var(--space-sm) var(--space-base);
    border-radius: var(--radius-md);
    color: var(--color-body);
    font-size: var(--text-body);
    transition: background 0.15s ease;
    text-decoration: none;
  }

  .nav-link:hover {
    background: var(--color-hairline-soft);
    color: var(--color-ink);
  }

  .nav-link.active {
    background: var(--color-surface-card);
    color: var(--color-ink);
    border: 1px solid var(--color-hairline);
  }

  .nav-icon {
    font-size: var(--text-body);
    opacity: 0.7;
    width: 18px;
    text-align: center;
  }

  .tree-section {
    margin-top: var(--space-base);
    padding-top: var(--space-sm);
    border-top: 1px solid var(--color-hairline);
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  .tree-heading {
    display: block;
    font-size: var(--text-xs);
    font-weight: 600;
    color: var(--color-muted);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    padding: var(--space-xxs) var(--space-base);
    margin-bottom: var(--space-xxs);
  }

  .tree-toggle {
    display: flex;
    align-items: center;
    gap: var(--space-xxs);
    width: 100%;
    padding: 3px var(--space-xs);
    border: none;
    background: none;
    color: var(--color-ink);
    font-size: var(--text-caption);
    font-weight: 500;
    cursor: pointer;
    border-radius: var(--radius-sm);
    text-align: left;
  }

  .tree-toggle:hover {
    background: var(--color-hairline-soft);
  }

  .toggle-icon {
    font-size: var(--text-2xs);
    color: var(--color-muted);
    width: 12px;
    text-align: center;
    flex-shrink: 0;
  }

  .group-name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .group-count {
    font-size: var(--text-xs);
    color: var(--color-muted);
    flex-shrink: 0;
  }

  .group-empty {
    font-size: var(--text-xs);
    color: var(--color-muted-soft);
    flex-shrink: 0;
  }

  .tree-empty-state {
    padding: var(--space-sm) var(--space-base);
    font-size: var(--text-sm);
    color: var(--color-muted);
    line-height: 1.5;
  }

  .tree-empty-state p {
    margin: 0 0 var(--space-xs);
  }

  .tree-empty-link {
    color: var(--color-primary);
    text-decoration: none;
    font-size: var(--text-sm);
  }

  .tree-empty-link:hover {
    text-decoration: underline;
  }

  .tree-repo {
    display: block;
    padding: 2px var(--space-xs);
    font-size: var(--text-caption);
    color: var(--color-body);
    text-decoration: none;
    border-radius: var(--radius-sm);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tree-repo:hover {
    background: var(--color-hairline-soft);
    color: var(--color-ink);
  }

  .tree-repo.active {
    background: var(--color-surface-card);
    color: var(--color-ink);
  }

  .tree-repo.missing {
    opacity: 0.5;
    font-style: italic;
  }
</style>
