<script lang="ts">
  import type { Snippet } from "svelte";
  import Poster from "../components/Poster.svelte";
  import AdSlot from "../components/AdSlot.svelte";
  import type { TitleSummary } from "../lib/types";
  import { keyOf } from "../lib/state.svelte";

  let {
    title,
    subtitle,
    items,
    loading = false,
    empty,
  }: { title: string; subtitle?: string; items: TitleSummary[]; loading?: boolean; empty?: Snippet } = $props();

  // Long lists get an ad between blocks of posters, at most two per page.
  const BLOCK = 30;
  const blocks = $derived(
    items.length > BLOCK
      ? [items.slice(0, BLOCK), items.slice(BLOCK, BLOCK * 2), items.slice(BLOCK * 2)].filter((b) => b.length)
      : [items],
  );
</script>

<div class="page">
  <header>
    <m3e-heading variant="headline" size="large" level={1}>{title}</m3e-heading>
    {#if subtitle}<p>{subtitle}</p>{/if}
  </header>

  {#if loading}
    <div class="grid">
      {#each Array(12) as _, i (i)}
        <m3e-skeleton animation="wave" shape="rounded" class="ghost"></m3e-skeleton>
      {/each}
    </div>
  {:else if items.length}
    {#each blocks as block, b (b)}
      <div class="grid">
        {#each block as t (keyOf(t))}
          <Poster title={t} />
        {/each}
      </div>
      {#if b < blocks.length - 1}<AdSlot id="grid" />{/if}
    {/each}
  {:else}
    <div class="empty">{@render empty?.()}</div>
  {/if}
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 24px;
    padding-bottom: 32px;
  }
  header p {
    margin: 6px 0 0;
    color: var(--md-sys-color-on-surface-variant);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 20px 16px;
  }
  .ghost {
    aspect-ratio: 2 / 3;
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 64px 0;
    text-align: center;
    color: var(--md-sys-color-on-surface-variant);
    --m3e-icon-size: 48px;
  }
</style>
