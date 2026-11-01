<script lang="ts">
  import { latest } from "../lib/latest.svelte";
  import { formatCount } from "../lib/release";

  // Real installer downloads from GitHub, counting up once it loads.
  // Shows nothing until the number is known.
  let shown = $state(0);

  $effect(() => {
    const target = latest.downloads;
    if (target === null) return;
    const start = performance.now();
    const ms = 1200;
    let frame = requestAnimationFrame(function step(now) {
      const t = Math.min(1, (now - start) / ms);
      shown = Math.round(target * (1 - Math.pow(1 - t, 3)));
      if (t < 1) frame = requestAnimationFrame(step);
    });
    return () => cancelAnimationFrame(frame);
  });
</script>

{#if latest.downloads !== null}
  <span class="count" title="Installer downloads on GitHub, all versions">
    <m3e-icon name="download"></m3e-icon>
    <b>{formatCount(shown)}</b>
    {latest.downloads === 1 ? "download" : "downloads"}
  </span>
{/if}

<style>
  .count {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 5px 12px 5px 9px;
    border-radius: 999px;
    font-size: 13px;
    font-weight: 600;
    color: #e4dcff;
    background: rgb(123 108 255 / 12%);
    border: 1px solid rgb(123 108 255 / 28%);
    --m3e-icon-size: 16px;
  }
  b {
    font-variant-numeric: tabular-nums;
    color: #fff;
  }
</style>
