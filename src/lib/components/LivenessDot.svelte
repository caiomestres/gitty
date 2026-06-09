<script lang="ts">
  interface Props {
    status: "up" | "down" | "gray";
    label: string;
    responseTimeMs?: number | null;
  }

  let { status, label, responseTimeMs = null }: Props = $props();

  const title = $derived(
    `${label}: ${status}${responseTimeMs != null ? ` (${responseTimeMs}ms)` : ""}`,
  );
</script>

<span class="liveness-dot liveness-{status}" {title}></span>

<style>
  .liveness-dot {
    display: inline-block;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .liveness-up {
    background: var(--color-success);
    box-shadow: 0 0 4px color-mix(in srgb, var(--color-success) 40%, transparent);
  }

  .liveness-down {
    background: var(--color-error);
    box-shadow: 0 0 4px color-mix(in srgb, var(--color-error) 40%, transparent);
  }

  .liveness-gray {
    background: var(--color-muted-soft);
  }
</style>
