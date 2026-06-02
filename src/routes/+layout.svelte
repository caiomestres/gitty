<script lang="ts">
  import AppShell from "$lib/components/AppShell.svelte";
  import "$lib/styles/tokens.css";
  import "$lib/styles/global.css";

  let { children } = $props();
</script>

<AppShell>
  {@render children()}
</AppShell>
