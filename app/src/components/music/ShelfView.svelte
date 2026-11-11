<script lang="ts">
  // One YouTube Music shelf: a carousel of cards, a block of song rows (laid
  // out in columns, like Quick picks), or a grid of mood tiles.
  import MusicCard from "./MusicCard.svelte";
  import Art from "./Art.svelte";
  import { artists, musicUi, openItem } from "../../lib/music/ui.svelte";
  import { musicPlayer } from "../../lib/music/player.svelte";
  import { nav } from "../../lib/state.svelte";
  import type { MusicShelf } from "../../lib/types";

  let { shelf, cardSize = 168 }: { shelf: MusicShelf; cardSize?: number } = $props();

  let track = $state<HTMLElement>();
  let atStart = $state(true);
  let atEnd = $state(false);

  function update() {
    if (!track) return;
    atStart = track.scrollLeft < 8;
    atEnd = track.scrollLeft + track.clientWidth >= track.scrollWidth - 8;
  }

  function page(dir: 1 | -1) {
    track?.scrollBy({ left: dir * track.clientWidth * 0.85, behavior: "smooth" });
  }

  function more() {
    const m = shelf.more;
    if (!m) return;
    if (m.pageType?.includes("PLAYLIST") && !m.browseId.startsWith("FE")) {
      nav.go({ name: "music-playlist", id: m.browseId });
    } else {
      nav.go({ name: "music-feed", browseId: m.browseId, params: m.params, title: shelf.title });
    }
  }

  $effect(() => {
    void shelf;
    queueMicrotask(update);
  });
</script>

<section class="shelf">
  <header>
    <h2 class="display">{shelf.title}</h2>
    <div class="tools">
      {#if shelf.more}
        <m3e-button variant="text" size="small" onclick={more}>More</m3e-button>
      {/if}
      {#if shelf.display !== "grid"}
        <m3e-icon-button variant="tonal" size="small" aria-label="Back" disabled={atStart} onclick={() => page(-1)}>
          <m3e-icon name="chevron_left"></m3e-icon>
        </m3e-icon-button>
        <m3e-icon-button variant="tonal" size="small" aria-label="Forward" disabled={atEnd} onclick={() => page(1)}>
          <m3e-icon name="chevron_right"></m3e-icon>
        </m3e-icon-button>
      {/if}
    </div>
  </header>

  {#if shelf.display === "grid"}
    <div class="tiles">
      {#each shelf.items as t (t.id + (t.params ?? ""))}
        <button class="tile" style:--tint={t.tint ?? "var(--md-sys-color-primary)"} onclick={() => openItem(t)}>
          {t.title}
        </button>
      {/each}
    </div>
  {:else if shelf.display === "list"}
    <div class="columns" bind:this={track} onscroll={update}>
      {#each shelf.items as s, i (s.id + i)}
        <div
          class="quick"
          class:current={musicPlayer.current?.id === s.id}
          role="button"
          tabindex="0"
          onclick={() => openItem(s, shelf.items, shelf.title)}
          onkeydown={(e) => e.key === "Enter" && openItem(s, shelf.items, shelf.title)}
          oncontextmenu={(e) => musicUi.openMenu(s, e)}
        >
          <Art thumbs={s.thumbnails} size={52} round={s.kind === "artist"} alt="" />
          <span class="text">
            <strong>{s.title}</strong>
            <span>{artists(s)}</span>
          </span>
        </div>
      {/each}
    </div>
  {:else}
    <div class="carousel" bind:this={track} onscroll={update}>
      {#each shelf.items as item, i (item.id + i)}
        <MusicCard {item} list={shelf.items} source={shelf.title} size={cardSize} />
      {/each}
    </div>
  {/if}
</section>

<style>
  .shelf {
    display: flex;
    flex-direction: column;
    gap: 14px;
    min-width: 0;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  h2 {
    margin: 0;
    font-size: 26px;
    font-weight: 400;
  }
  .tools {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .carousel {
    display: flex;
    gap: 18px;
    overflow-x: auto;
    scroll-snap-type: x proximity;
    scrollbar-width: none;
    padding: 4px 2px 6px;
  }
  .carousel > :global(*) {
    scroll-snap-align: start;
  }
  /* Song rows flow into columns of four, scrolling sideways. */
  .columns {
    display: grid;
    grid-auto-flow: column;
    grid-template-rows: repeat(4, auto);
    grid-auto-columns: minmax(300px, 32%);
    gap: 6px 16px;
    overflow-x: auto;
    scroll-snap-type: x proximity;
    scrollbar-width: none;
  }
  .quick {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px;
    border-radius: 14px;
    cursor: pointer;
    scroll-snap-align: start;
    min-width: 0;
    outline: none;
    transition: background 160ms ease;
  }
  .quick:hover,
  .quick:focus-visible {
    background: color-mix(in srgb, var(--md-sys-color-on-surface) 6%, transparent);
  }
  .quick.current strong {
    color: var(--md-sys-color-primary);
  }
  .text {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .text strong,
  .text span {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .text strong {
    font-size: 14px;
  }
  .text span {
    font-size: 12.5px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .tiles {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(170px, 1fr));
    gap: 12px;
  }
  .tile {
    all: unset;
    cursor: pointer;
    position: relative;
    overflow: hidden;
    padding: 18px 16px 18px 22px;
    border-radius: 16px;
    font-weight: 600;
    font-size: 14px;
    background: var(--md-sys-color-surface-container-high);
    transition:
      transform 220ms cubic-bezier(0.2, 0, 0, 1),
      border-radius 280ms ease;
  }
  .tile::before {
    content: "";
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 6px;
    background: var(--tint);
  }
  .tile:hover {
    transform: translateY(-2px);
    border-radius: 24px;
  }
</style>
