<script lang="ts">
  // 24 cells, one per UTC hour, brighter where more happens. Shows local time on hover.
  let { values, unit = "plays" }: { values: number[]; unit?: string } = $props();
  const peak = $derived(Math.max(1, ...values));
  const offset = -new Date().getTimezoneOffset() / 60;
  const local = (h: number) => {
    const l = (((h + offset) % 24) + 24) % 24;
    return `${String(Math.floor(l)).padStart(2, "0")}:${l % 1 ? "30" : "00"}`;
  };
</script>

<div class="hours" role="img" aria-label="Activity by hour">
  {#each values as v, h (h)}
    <div class="cell" style:--a={v / peak} title="{String(h).padStart(2, '0')}:00 UTC ({local(h)} your time): {v.toLocaleString()} {unit}">
      <span>{String(h).padStart(2, "0")}</span>
    </div>
  {/each}
</div>
<p class="muted note">UTC hours. Hover for your local time.</p>

<style>
  .hours {
    display: grid;
    grid-template-columns: repeat(24, minmax(0, 1fr));
    gap: 3px;
  }
  .cell {
    aspect-ratio: 1 / 1.6;
    border-radius: 8px;
    display: flex;
    align-items: flex-end;
    justify-content: center;
    padding-bottom: 4px;
    background: color-mix(in srgb, var(--md-sys-color-primary) calc(var(--a) * 85% + 6%), var(--md-sys-color-surface-container-highest));
    font-size: 10px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .note {
    margin: 8px 0 0;
    font-size: 12px;
  }
</style>
