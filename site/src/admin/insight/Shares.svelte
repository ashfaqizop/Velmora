<script lang="ts">
  // Labelled horizontal bars for a breakdown (sources, versions, players).
  let { rows, names = {} }: { rows: { key: string; count: number; share: number }[]; names?: Record<string, string> } = $props();
</script>

{#if !rows.length}
  <p class="muted none">No numbers yet.</p>
{:else}
  <ul class="shares">
    {#each rows as r (r.key)}
      <li>
        <span class="name">{names[r.key] ?? r.key}</span>
        <span class="track"><span style:width="{Math.max(1, r.share * 100)}%"></span></span>
        <span class="num">{Math.round(r.share * 100)}%</span>
        <span class="muted count">{r.count.toLocaleString()}</span>
      </li>
    {/each}
  </ul>
{/if}

<style>
  .shares {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  li {
    display: grid;
    grid-template-columns: minmax(80px, 140px) minmax(0, 1fr) 44px 64px;
    align-items: center;
    gap: 12px;
    font-size: 14px;
  }
  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .track {
    height: 10px;
    border-radius: 999px;
    background: var(--md-sys-color-surface-container-highest);
    overflow: hidden;
  }
  .track span {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: var(--md-sys-color-primary);
    transition: width 500ms cubic-bezier(0.2, 0, 0, 1);
  }
  .num {
    font-weight: 700;
    text-align: right;
  }
  .count {
    text-align: right;
    font-size: 12.5px;
  }
  .none {
    margin: 0;
  }
</style>
