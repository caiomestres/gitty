<script lang="ts">
  import type { Snippet } from "svelte";
  import Sidebar from "./Sidebar.svelte";
  import StatusBar from "./StatusBar.svelte";
  import BottomBar from "./BottomBar.svelte";
  import NotificationPanel from "./NotificationPanel.svelte";

  interface Props {
    children: Snippet;
  }

  let { children }: Props = $props();
</script>

<div class="app-shell">
  <div class="top-bar">
    <StatusBar />
    <div class="top-bar-actions">
      <NotificationPanel />
    </div>
  </div>
  <div class="app-body">
    <Sidebar />
    <main class="main-content">
      {@render children()}
    </main>
  </div>
  <BottomBar />
</div>

<style>
  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
    background: var(--color-canvas);
  }

  .top-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .top-bar-actions {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    padding-right: var(--space-base);
  }

  .app-body {
    display: flex;
    flex: 1;
    min-height: 0;
    border-top: 1px solid var(--color-hairline);
    border-bottom: 1px solid var(--color-hairline);
  }

  .main-content {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
    background: var(--color-canvas);
  }
</style>
