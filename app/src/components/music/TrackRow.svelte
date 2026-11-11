<script lang="ts">
  // One song in a list: number (or a small equalizer while it plays), cover,
  // title, artists, album, plays, like, length and the song menu.
  import Art from "./Art.svelte";
  import { clock, likes, musicUi, openItem, type MenuContext } from "../../lib/music/ui.svelte";
  import { musicPlayer } from "../../lib/music/player.svelte";
  import { nav } from "../../lib/state.svelte";
  import type { MusicItem } from "../../lib/types";

  let {
    item,
    index,
    list,
    source,
    ctx = {},
    showArt = true,
    showAlbum = true,
    numbered = false,
  }: {
    item: MusicItem;
    index: number;
    list: MusicItem[];
    source?: string;
    ctx?: MenuContext;
    showArt?: boolean;
    showAlbum?: boolean;
    numbered?: boolean;
  } = $props();

  const current = $derived(musicPlayer.current?.id === item.id);
  const liked = $derived(likes.liked(item.id));

  function goArtist(e: MouseEvent, id: string | null) {
    e.stopPropagation();
    if (id) nav.go({ name: "music-artist", id });
  }
</script>

<div
  class="row"
  class:current
  role="button"
  tabindex="0"
  onclick={() => openItem(item, list, source)}
  onkeydown={(e) => e.key === "Enter" && openItem(item, list, source)}
  oncontextmenu={(e) => musicUi.openMenu(item, e, ctx)}
>
  <span class="lead">
    {#if current && musicPlayer.playing}
      <span class="eq" aria-label="Playing"><i></i><i></i><i></i></span>
    {:else if numbered}
      <span class="num">{index + 1}</span>
    {/if}
    <m3e-icon class="hover-play" name="play_arrow" filled></m3e-icon>
  </span>
  {#if showArt}<Art thumbs={item.thumbnails} size={44} alt="" />{/if}
  <span class="main">
    <strong class="title">{item.title}{#if item.explicit}<span class="e">E</span>{/if}</strong>
    <span class="by">
      {#each item.artists as a, k (k)}
        {#if k > 0},&nbsp;{/if}
        {#if a.id}
          <button class="link" onclick={(e) => goArtist(e, a.id)}>{a.name}</button>
        {:else}<span>{a.name}</span>{/if}
      {:else}
        {item.subtitle ?? ""}
      {/each}
    </span>
  </span>
  {#if showAlbum}
    <span class="album">
      {#if item.album && item.albumId}
        <button
          class="link"
          onclick={(e) => {
            e.stopPropagation();
            nav.go({ name: "music-album", id: item.albumId! });
          }}>{item.album}</button
        >
      {:else}{item.album ?? ""}{/if}
    </span>
  {/if}
  <span class="plays">{item.playCount ?? ""}</span>
  <button
    class="icon like"
    class:on={liked}
    aria-label={liked ? "Unlike" : "Like"}
    onclick={(e) => {
      e.stopPropagation();
      likes.toggle(item);
    }}
  >
    <m3e-icon name="favorite" filled={liked}></m3e-icon>
  </button>
  <span class="len">{item.duration ? clock(item.duration) : ""}</span>
  <button class="icon more" aria-label="More" onclick={(e) => musicUi.openMenu(item, e, ctx)}>
    <m3e-icon name="more_vert"></m3e-icon>
  </button>
</div>

<style>
  .row {
    display: grid;
    grid-template-columns: 32px auto minmax(0, 3fr) minmax(0, 2fr) 90px 36px 52px 36px;
    align-items: center;
    gap: 12px;
    padding: 6px 10px;
    border-radius: 14px;
    cursor: pointer;
    outline: none;
    transition: background 160ms ease;
  }
  .row:not(:has(.album)) {
    grid-template-columns: 32px auto minmax(0, 1fr) 90px 36px 52px 36px;
  }
  .row:hover,
  .row:focus-visible {
    background: color-mix(in srgb, var(--md-sys-color-on-surface) 6%, transparent);
  }
  .row.current {
    background: color-mix(in srgb, var(--md-sys-color-primary) 12%, transparent);
  }
  .row.current .title {
    color: var(--md-sys-color-primary);
  }
  .lead {
    position: relative;
    display: grid;
    place-items: center;
    height: 32px;
    color: var(--md-sys-color-on-surface-variant);
    font-size: 13px;
    font-variant-numeric: tabular-nums;
  }
  .hover-play {
    position: absolute;
    opacity: 0;
    color: var(--md-sys-color-on-surface);
  }
  .row:hover .hover-play {
    opacity: 1;
  }
  .row:hover .lead > :not(.hover-play) {
    opacity: 0;
  }
  .main {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .title {
    font-size: 14px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .by,
  .album,
  .plays,
  .len {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .plays,
  .len {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  .link {
    all: unset;
    cursor: pointer;
  }
  .link:hover {
    text-decoration: underline;
    color: var(--md-sys-color-on-surface);
  }
  .icon {
    all: unset;
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    cursor: pointer;
    color: var(--md-sys-color-on-surface-variant);
    opacity: 0;
    transition: opacity 150ms ease;
    --m3e-icon-size: 20px;
  }
  .icon:hover {
    background: color-mix(in srgb, var(--md-sys-color-on-surface) 10%, transparent);
  }
  .row:hover .icon,
  .icon.on {
    opacity: 1;
  }
  .like.on {
    color: var(--brand-coral, #ff5a5a);
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
  .eq {
    display: flex;
    align-items: flex-end;
    gap: 2px;
    height: 14px;
  }
  .eq i {
    width: 3px;
    background: var(--md-sys-color-primary);
    border-radius: 2px;
    animation: eq 900ms ease-in-out infinite;
  }
  .eq i:nth-child(2) {
    animation-delay: -300ms;
  }
  .eq i:nth-child(3) {
    animation-delay: -600ms;
  }
  @keyframes eq {
    0%,
    100% {
      height: 30%;
    }
    50% {
      height: 100%;
    }
  }
  @media (max-width: 1100px) {
    .row {
      grid-template-columns: 32px auto minmax(0, 1fr) 36px 52px 36px;
    }
    .album,
    .plays {
      display: none;
    }
  }
</style>
