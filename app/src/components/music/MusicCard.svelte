<script lang="ts">
  // A card in a carousel or grid: album, single, playlist, artist, song or video.
  import Art from "./Art.svelte";
  import { musicUi, openItem, playCollection, playable } from "../../lib/music/ui.svelte";
  import { musicPlayer } from "../../lib/music/player.svelte";
  import type { MusicItem } from "../../lib/types";

  let {
    item,
    list,
    source,
    size = 168,
  }: { item: MusicItem; list?: MusicItem[]; source?: string; size?: number } = $props();

  const round = $derived(item.kind === "artist");
  const wide = $derived(item.kind === "video");
  const current = $derived(musicPlayer.current?.id === item.id);

  function play(e: MouseEvent) {
    e.stopPropagation();
    if (playable(item)) openItem(item, list, source);
    else playCollection(item);
  }
</script>

<div
  class="card"
  class:round
  role="button"
  tabindex="0"
  style:--w={`${wide ? Math.round(size * 1.6) : size}px`}
  onclick={() => openItem(item, list, source)}
  onkeydown={(e) => e.key === "Enter" && openItem(item, list, source)}
  oncontextmenu={(e) => playable(item) && musicUi.openMenu(item, e)}
>
  <div class="cover">
    <Art thumbs={item.thumbnails} size={wide ? Math.round(size * 1.6) : size} {round} {wide} alt={item.title} />
    {#if item.kind !== "category"}
      <button class="play" class:on={current && musicPlayer.playing} aria-label={`Play ${item.title}`} onclick={play}>
        <m3e-icon name={current && musicPlayer.playing ? "graphic_eq" : "play_arrow"} filled></m3e-icon>
      </button>
    {/if}
  </div>
  <strong class="title" title={item.title}>
    {item.title}{#if item.explicit}<span class="e" title="Explicit">E</span>{/if}
  </strong>
  {#if item.subtitle}<span class="sub">{item.subtitle}</span>{/if}
</div>

<style>
  .card {
    width: var(--w);
    display: flex;
    flex-direction: column;
    gap: 6px;
    cursor: pointer;
    flex: none;
    outline: none;
  }
  .card.round {
    text-align: center;
    align-items: center;
  }
  .cover {
    position: relative;
    transition: transform 260ms cubic-bezier(0.2, 0, 0, 1);
  }
  .card:hover .cover,
  .card:focus-visible .cover {
    transform: translateY(-3px);
  }
  .play {
    position: absolute;
    right: 10px;
    bottom: 10px;
    width: 46px;
    height: 46px;
    border-radius: 16px;
    border: 0;
    display: grid;
    place-items: center;
    cursor: pointer;
    background: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
    box-shadow: 0 6px 18px rgb(0 0 0 / 35%);
    opacity: 0;
    transform: translateY(8px) scale(0.9);
    transition:
      opacity 200ms ease,
      transform 260ms cubic-bezier(0.2, 0, 0, 1),
      border-radius 300ms ease;
  }
  .round .play {
    right: 50%;
    margin-right: -23px;
  }
  .card:hover .play,
  .card:focus-visible .play,
  .play.on {
    opacity: 1;
    transform: none;
  }
  .play:hover {
    border-radius: 50%;
  }
  .title {
    font-size: 14px;
    font-weight: 600;
    line-height: 1.3;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .sub {
    font-size: 12.5px;
    color: var(--md-sys-color-on-surface-variant);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .e {
    display: inline-grid;
    place-items: center;
    margin-left: 6px;
    width: 15px;
    height: 15px;
    border-radius: 3px;
    font-size: 9.5px;
    font-weight: 800;
    vertical-align: 1px;
    background: var(--md-sys-color-on-surface-variant);
    color: var(--md-sys-color-surface);
  }
</style>
