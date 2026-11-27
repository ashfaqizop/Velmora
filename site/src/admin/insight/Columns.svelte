<script lang="ts">
  // A day-by-day column chart: one bar per point, the value on hover, the
  // first and last labels underneath.
  let { points, unit = "", height = 120 }: { points: { label: string; value: number }[]; unit?: string; height?: number } = $props();
  const peak = $derived(Math.max(1, ...points.map((p) => p.value)));
  const total = $derived(points.reduce((a, p) => a + p.value, 0));
</script>

<div class="cols" style:height="{height}px" style:--n={points.length} role="img" aria-label="{total.toLocaleString()} {unit} over {points.length} days">
  {#each points as p (p.label)}
    <div class="col" title="{p.label}: {p.value.toLocaleString()} {unit}">
      <span style:height="{p.value ? Math.max(3, (p.value / peak) * 100) : 0}%"></span>
    </div>
  {/each}
</div>
{#if points.length}
  <div class="axis muted"><span>{points[0].label}</span><span>peak {peak.toLocaleString()}</span><span>{points[points.length - 1].label}</span></div>
{/if}

<style>
  .cols {
    display: grid;
    grid-template-columns: repeat(var(--n), minmax(0, 1fr));
    align-items: end;
    gap: 3px;
  }
  .col {
    height: 100%;
    display: flex;
    align-items: flex-end;
  }
  .col span {
    display: block;
    width: 100%;
    border-radius: 4px 4px 2px 2px;
    background: linear-gradient(to top, color-mix(in srgb, var(--md-sys-color-primary) 45%, transparent), var(--md-sys-color-primary));
    transition: height 500ms cubic-bezier(0.2, 0, 0, 1);
  }
  .col:hover span {
    background: var(--md-sys-color-tertiary, var(--md-sys-color-primary));
  }
  .axis {
    display: flex;
    justify-content: space-between;
    margin-top: 6px;
    font-size: 12px;
  }
</style>
