<script lang="ts">
  import { onMount } from "svelte";
  import AppShell from "$lib/components/AppShell.svelte";
  import { getTheme, applyTheme } from "$lib/utils/theme";
  import "$lib/styles/theme-default.css";
  import "$lib/styles/global.css";

  let { children } = $props();

  onMount(() => {
    getTheme().then(applyTheme);
  });
</script>

<AppShell>
  {@render children()}
</AppShell>
