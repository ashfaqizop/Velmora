<script lang="ts">
  // The little marks after a name: verified, and Velmora Max.
  import { maxIds } from "./data.svelte";

  let { id, verified, size = 18 }: { id: string; verified: boolean; size?: number } = $props();
</script>

{#if verified}
  <span class="mark verified" style:--s="{size}px" title="Verified">
    <m3e-icon name="verified" filled aria-label="Verified"></m3e-icon>
  </span>
{/if}
{#if maxIds.has(id)}
  <span class="mark max" style:--s="{size}px" title="Velmora Max">
    <m3e-icon name="workspace_premium" filled aria-label="Velmora Max"></m3e-icon>
  </span>
{/if}

<style>
  .mark {
    display: inline-grid;
    flex: none;
    --m3e-icon-size: var(--s);
    line-height: 0;
  }
  .verified {
    color: var(--md-sys-color-primary);
  }
  .max {
    color: #f8d27a;
    filter: drop-shadow(0 0 6px color-mix(in srgb, #f8d27a 45%, transparent));
  }
</style>
