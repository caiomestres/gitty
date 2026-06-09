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
  aria-label="Gitty mascot"
>
  <!-- Body/Circle background -->
  <circle
    cx="16"
    cy="16"
    r="15"
    class="mascot-bg"
    stroke="currentColor"
    stroke-width="1.5"
    stroke-opacity="0.2"
  />

  <!-- Face - simplified lion tamarin silhouette -->
  <ellipse cx="16" cy="17" rx="7" ry="8" class="mascot-face-shadow" />

  <!-- Ears (the mane) - golden tufts (themeable) -->
  <circle cx="9" cy="12" r="4" class="mascot-mane-secondary" />
  <circle cx="23" cy="12" r="4" class="mascot-mane-secondary" />
  <circle cx="10" cy="9" r="3" class="mascot-mane-primary" />
  <circle cx="22" cy="9" r="3" class="mascot-mane-primary" />

  <!-- Face -->
  <circle cx="16" cy="17" r="5" class="mascot-face" />
  <circle cx="14" cy="16" r="1" class="mascot-eye" />
  <circle cx="18" cy="16" r="1" class="mascot-eye" />
  <ellipse cx="16" cy="19" rx="2" ry="1.5" class="mascot-muzzle" />
</svg>

<style>
  svg {
    flex-shrink: 0;
  }

  .mascot-bg {
    fill: var(--mascot-bg, #f5f0e8);
  }

  .mascot-mane-primary {
    fill: var(--mascot-mane-primary, #e8c9a0);
  }

  .mascot-mane-secondary {
    fill: var(--mascot-mane-secondary, #d4a574);
  }

  .mascot-face {
    fill: var(--mascot-face, #8b7355);
  }

  .mascot-face-shadow {
    fill: var(--mascot-face-shadow, rgba(26, 26, 26, 0.15));
  }

  .mascot-eye {
    fill: var(--mascot-eye, #1a1a1a);
  }

  .mascot-muzzle {
    fill: var(--mascot-muzzle, rgba(26, 26, 26, 0.3));
  }
</style>
