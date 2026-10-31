<script lang="ts">
  import Poster from "../../Poster.svelte";
  import type { RefBook } from "../../../lib/ani/refs";
  import { personalize } from "../../../lib/ani/personalize.svelte";
  import { nav } from "../../../lib/state.svelte";
  import type { TitleSummary } from "../../../lib/types";

  // render_carousel: a scrolling row of posters, like a Home rail, with
  // optional tabs, rank numbers and "For you" chips.
  // Also render_collection, which wraps into a grid and ticks finished titles.
  let { args, refs, grid = false }: { args: Record<string, unknown>; refs: RefBook; grid?: boolean } = $props();

  type Group = { label: string; titles: TitleSummary[] };
  const resolve = (list: unknown) => ((list as string[]) ?? []).flatMap((r) => refs.title(r) ?? []);

  const heading = $derived(args.heading as string | undefined);
  const ranked = $derived(args.ranked === true);
  const forYou = $derived(new Set(resolve(args.for_you).map((t) => t.origin.provider + t.origin.id)));
  const groups = $derived.by((): Group[] => {
    const g = ((args.groups as { label: string; refs: string[] }[]) ?? []).map((x) => ({ label: x.label, titles: resolve(x.refs) }));
    return g.length ? g : [{ label: "", titles: resolve(args.refs) }];
  });
  let tab = $state(0);
  const current = $derived(groups[Math.min(tab, groups.length - 1)]);

  let row = $state<HTMLElement>();
  let atStart = $state(true);
  let atEnd = $state(false);

  function edges() {
    if (!row) return;
    atStart = row.scrollLeft < 8;
    atEnd = row.scrollLeft + row.clientWidth > row.scrollWidth - 8;
  }
  $effect(() => {
    void current;
    queueMicrotask(() => {
      row?.scrollTo({ left: 0 });
      edges();
    });
  });
  const scroll = (dir: number) => row?.scrollBy({ left: dir * row.clientWidth * 0.8, behavior: "smooth" });

  function seeAll() {
    const title = [heading, current.label].filter(Boolean).join(" · ") || "From Ani";
    nav.go({ name: "list", title, items: $state.snapshot(current.titles) as TitleSummary[] });
  }
</script>

<div class="block">
  <header>
    {#if heading}<m3e-heading variant="title" size="large" level={3} class="title">{heading}</m3e-heading>{/if}
    <span class="count">{current.titles.length} titles</span>
    <div class="nav">
      {#if !grid}
        <m3e-icon-button size="small" aria-label="Scroll left" disabled={atStart} onclick={() => scroll(-1)}>
          <m3e-icon name="chevron_left"></m3e-icon>
        </m3e-icon-button>
        <m3e-icon-button size="small" aria-label="Scroll right" disabled={atEnd} onclick={() => scroll(1)}>
          <m3e-icon name="chevron_right"></m3e-icon>
        </m3e-icon-button>
      {/if}
      <m3e-button variant="text" size="small" onclick={seeAll}>
        See all<m3e-icon slot="trailing-icon" name="arrow_forward"></m3e-icon>
      </m3e-button>
    </div>
  </header>

  {#if groups.length > 1}
    <m3e-tabs class="tabs">
      {#each groups as g, i (g.label + i)}
        <m3e-tab selected={i === tab} onclick={() => (tab = i)}>{g.label}</m3e-tab>
      {/each}
    </m3e-tabs>
  {/if}

  <div class="row" class:grid class:ranked class:fade-start={!grid && !atStart} class:fade-end={!grid && !atEnd} bind:this={row} onscroll={edges}>
    {#each current.titles as t, i (t.origin.provider + t.origin.id)}
      <div class="item">
        {#if ranked && !grid}<span class="rank" aria-hidden="true">{i + 1}</span>{/if}
        <div class="card">
          <Poster title={t} />
          {#if forYou.has(t.origin.provider + t.origin.id)}<span class="for-you">For you</span>{/if}
          {#if personalize.finished(t)}
            <span class="seen" title="You've watched this"><m3e-icon name="check"></m3e-icon></span>
          {/if}
        </div>
      </div>
    {/each}
  </div>
</div>

<style>
  .block {
    display: flex;
    flex-direction: column;
    gap: 10px;
    min-width: 0;
  }
  header {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 36px;
  }
  .count {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .nav {
    display: flex;
    align-items: center;
    gap: 2px;
    margin-left: auto;
  }
  .tabs {
    --m3e-tabs-container-color: transparent;
  }
  .row {
    display: grid;
    grid-auto-flow: column;
    grid-auto-columns: 150px;
    gap: 14px;
    overflow-x: auto;
    scroll-snap-type: x mandatory;
    scrollbar-width: none;
    padding: 4px 0 6px;
  }
  .row.ranked {
    grid-auto-columns: 196px;
  }
  /* Soft edges show there's more to scroll. */
  .row.fade-end {
    mask-image: linear-gradient(90deg, #000 85%, transparent);
  }
  .row.fade-start {
    mask-image: linear-gradient(90deg, transparent, #000 15%);
  }
  .row.fade-start.fade-end {
    mask-image: linear-gradient(90deg, transparent, #000 12%, #000 88%, transparent);
  }
  .row.grid {
    grid-auto-flow: row;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    overflow: visible;
  }
  .item {
    position: relative;
    scroll-snap-align: start;
    min-width: 0;
    display: flex;
    align-items: flex-end;
  }
  .rank {
    flex: none;
    width: 46px;
    margin-right: -14px;
    margin-bottom: 64px;
    font-family: "Instrument Serif", serif;
    font-size: 96px;
    line-height: 0.8;
    text-align: right;
    color: transparent;
    -webkit-text-stroke: 2px color-mix(in srgb, var(--md-sys-color-primary) 70%, transparent);
    z-index: 0;
    user-select: none;
  }
  .card {
    position: relative;
    flex: 1;
    min-width: 0;
    z-index: 1;
  }
  .for-you {
    position: absolute;
    left: 8px;
    bottom: 64px;
    padding: 2px 8px;
    border-radius: 999px;
    font-size: 11px;
    font-weight: 700;
    background: var(--md-sys-color-tertiary-container);
    color: var(--md-sys-color-on-tertiary-container);
    pointer-events: none;
  }
  .seen {
    position: absolute;
    right: 8px;
    bottom: 64px;
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    border-radius: 50%;
    background: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
    --m3e-icon-size: 16px;
    pointer-events: none;
  }
</style>
