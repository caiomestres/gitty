<script lang="ts">
  import { page } from "$app/stores";
  import { resolve } from "$app/paths";
  import { invoke } from "@tauri-apps/api/core";
  import { SvelteSet } from "svelte/reactivity";
  import type { GroupTreeNodeDto } from "$lib/types/workspace";
  import { onConfigChanged } from "$lib/utils/config-events";
  import type { Component } from "svelte";
  import LayoutDashboard from "@lucide/svelte/icons/layout-dashboard";
  import HeartPulse from "@lucide/svelte/icons/heart-pulse";
  import History from "@lucide/svelte/icons/history";
  import ScrollText from "@lucide/svelte/icons/scroll-text";
  import FolderTree from "@lucide/svelte/icons/folder-tree";
  import Zap from "@lucide/svelte/icons/zap";
  import Settings from "@lucide/svelte/icons/settings";
  import ChevronRight from "@lucide/svelte/icons/chevron-right";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";

  const navItems: {
    href: "/" | "/groups" | "/macros" | "/health" | "/changes" | "/activity" | "/settings";
    label: string;
    icon: Component;
  }[] = [
    { href: "/", label: "Dashboard", icon: LayoutDashboard },
    { href: "/health", label: "Health", icon: HeartPulse },
    { href: "/changes", label: "Changes", icon: History },
    { href: "/activity", label: "Activity", icon: ScrollText },
    { href: "/groups", label: "Groups", icon: FolderTree },
    { href: "/macros", label: "Macros", icon: Zap },
    { href: "/settings", label: "Settings", icon: Settings },
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
        {#if collapsed.has(node.group.id)}
          <ChevronRight size={12} />
        {:else}
          <ChevronDown size={12} />
        {/if}
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
    <img
      class="brand-mark"
      src="/brand-mark.svg"
      alt=""
      width="28"
      height="28"
      aria-hidden="true"
    />
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
          <span class="nav-icon" aria-hidden="true"><item.icon size={16} /></span>
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
    width: 28px;
    height: 28px;
    border-radius: var(--radius-md);
    object-fit: contain;
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
    display: flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    color: var(--color-muted);
    transition: color 0.15s ease;
  }

  .nav-link:hover .nav-icon,
  .nav-link.active .nav-icon {
    color: var(--color-ink);
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
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-muted);
    width: 12px;
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
