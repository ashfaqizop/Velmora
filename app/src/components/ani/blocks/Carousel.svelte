<script lang="ts">
  import Poster from "../../Poster.svelte";
  import type { RefBook } from "../../../lib/ani/refs";
  import { personalize } from "../../../lib/ani/personalize.svelte";

  // render_carousel: a scrolling row of posters, like a Home rail.
  // Also render_collection, which wraps into a grid and ticks finished titles.
  let { args, refs, grid = false }: { args: Record<string, unknown>; refs: RefBook; grid?: boolean } = $props();

  const heading = $derived(args.heading as string | undefined);
  const titles = $derived(((args.refs as string[]) ?? []).flatMap((r) => refs.title(r) ?? []));

  let row: HTMLElement;
  const scroll = (dir: number) => row?.scrollBy({ left: dir * row.clientWidth * 0.8, behavior: "smooth" });
</script>

<div class="block">
  <header>
    {#if heading}<m3e-heading variant="title" size="medium" level={3}>{heading}</m3e-heading>{/if}
    {#if !grid && titles.length > 4}
      <div class="nav">
        <m3e-icon-button size="small" aria-label="Scroll left" onclick={() => scroll(-1)}><m3e-icon name="chevron_left"></m3e-icon></m3e-icon-button>
        <m3e-icon-button size="small" aria-label="Scroll right" onclick={() => scroll(1)}><m3e-icon name="chevron_right"></m3e-icon></m3e-icon-button>
      </div>
    {/if}
  </header>
  <div class="row" class:grid bind:this={row}>
    {#each titles as t (t.origin.provider + t.origin.id)}
      <div class="item">
        <Poster title={t} />
        {#if grid && personalize.finished(t)}
          <span class="seen" title="You've watched this"><m3e-icon name="check" filled></m3e-icon></span>
        {/if}
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
    justify-content: space-between;
    gap: 12px;
    min-height: 32px;
  }
  .nav {
    display: flex;
    gap: 4px;
    margin-left: auto;
  }
  .row {
    display: grid;
    grid-auto-flow: column;
    grid-auto-columns: 150px;
    gap: 14px;
    overflow-x: auto;
    scroll-snap-type: x mandatory;
    scrollbar-width: none;
    padding-bottom: 4px;
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
  }
  .seen {
    position: absolute;
    top: 8px;
    left: 8px;
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
    --m3e-icon-size: 18px;
    pointer-events: none;
  }
</style>
