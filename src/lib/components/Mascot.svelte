<script lang="ts">
  interface Props {
    size?: number;
    class?: string;
  }

  let { size = 32, class: className = "" }: Props = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 32 32"
  fill="none"
  xmlns="http://www.w3.org/2000/svg"
  class={className}
  aria-label="Gitty logo"
>
  <!-- Folder -->
  <path
    d="M27.5,5.5H18.2L16.1,9.7H4.4V26.5H29.6V5.5Zm0,4.2H19.3l1.1-2.1h7.1Z"
    class="logo-folder"
  />
  <!-- Git diamond badge -->
  <path
    d="M30.6,19.565,21.435,10.4a1.352,1.352,0,0,0-1.913,0l-1.9,1.9,2.415,2.415a1.606,1.606,0,0,1,2.034,2.047L24.4,19.091a1.607,1.607,0,1,1-.964.907L21.26,17.827V23.54a1.606,1.606,0,1,1-1.323-.047V17.727a1.61,1.61,0,0,1-.873-2.11l-2.381-2.381L10.4,19.522a1.353,1.353,0,0,0,0,1.913L19.565,30.6a1.353,1.353,0,0,0,1.913,0L30.6,21.478a1.353,1.353,0,0,0,0-1.913"
    class="logo-badge"
  />
  <!-- Git branch symbol -->
  <path
    d="M17.619,12.3l2.415,2.415a1.606,1.606,0,0,1,2.034,2.047L24.4,19.091a1.607,1.607,0,1,1-.964.907L21.26,17.827V23.54a1.606,1.606,0,1,1-1.323-.047V17.727a1.61,1.61,0,0,1-.873-2.11l-2.381-2.381"
    class="logo-symbol"
  />
</svg>

<style>
  svg {
    flex-shrink: 0;
  }

  .logo-folder {
    fill: var(--logo-folder, #bf5e55);
  }

  .logo-badge {
    fill: var(--logo-badge, #dd4c35);
  }

  .logo-symbol {
    fill: var(--logo-symbol, #ffffff);
  }
</style>
