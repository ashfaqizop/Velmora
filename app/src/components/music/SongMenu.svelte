<script lang="ts">
  // The song menu, opened by right-click or ⋮ on any song. One instance for
  // the whole app (mounted in App.svelte), placed where the click was.
  import { fade, scale } from "svelte/transition";
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import * as api from "../../lib/api";
  import { musicExports } from "../../lib/music/exports.svelte";
  import { likes, musicUi, shareLink, artists } from "../../lib/music/ui.svelte";
  import { musicPlayer } from "../../lib/music/player.svelte";
  import { ani, downloads, nav } from "../../lib/state.svelte";

  let box = $state<HTMLElement>();
  let pos = $state({ x: 0, y: 0 });

  const m = $derived(musicUi.menu);
  const it = $derived(m?.item);
  const firstArtist = $derived(it?.artists.find((a) => a.id));

  // Keep the menu inside the window.
  $effect(() => {
    if (!m || !box) return;
    const r = box.getBoundingClientRect();
    pos = {
      x: Math.min(m.x, innerWidth - r.width - 8),
      y: Math.min(m.y, innerHeight - r.height - 8),
    };
  });

  function run(fn: () => unknown) {
    musicUi.closeMenu();
    fn();
  }

  async function download() {
    if (!it) return;
    try {
      await musicExports.download([it]);
      M3eSnackbar.open(`Downloading ${it.title}. Find it in Library, under Downloaded.`);
    } catch (e) {
      M3eSnackbar.open(`Couldn't download: ${e}`);
    }
  }

  async function copy() {
    if (!it) return;
    await navigator.clipboard.writeText(shareLink(it)).catch(() => {});
    M3eSnackbar.open("Link copied");
  }

  async function remove() {
    const ctx = m?.ctx;
    if (!it || !ctx?.playlistId || !it.setVideoId) return;
    try {
      await api.musicPlaylistRemove(ctx.playlistId, it.id, it.setVideoId);
      ctx.onRemoved?.();
      M3eSnackbar.open("Removed from the playlist");
    } catch (e) {
      M3eSnackbar.open(String(e));
    }
  }
</script>

<svelte:window
  onkeydown={(e) => e.key === "Escape" && musicUi.closeMenu()}
  onresize={() => musicUi.closeMenu()}
/>

{#if m && it}
  <div class="scrim" role="presentation" onclick={() => musicUi.closeMenu()} oncontextmenu={(e) => (e.preventDefault(), musicUi.closeMenu())} transition:fade={{ duration: 120 }}></div>
  <div
    class="menu"
    role="menu"
    tabindex="-1"
    bind:this={box}
    style:left={`${pos.x || m.x}px`}
    style:top={`${pos.y || m.y}px`}
    transition:scale={{ start: 0.92, duration: 160 }}
  >
    <div class="head">
      <strong>{it.title}</strong>
      <span>{artists(it)}</span>
    </div>
    <button role="menuitem" onclick={() => run(() => musicPlayer.playNext(it))}><m3e-icon name="playlist_play"></m3e-icon>Play next</button>
    <button role="menuitem" onclick={() => run(() => musicPlayer.addToQueue(it))}><m3e-icon name="queue_music"></m3e-icon>Add to queue</button>
    <button role="menuitem" onclick={() => run(() => musicPlayer.playSong(it, "Radio"))}><m3e-icon name="radio"></m3e-icon>Start radio</button>
    <hr />
    <button role="menuitem" onclick={() => run(() => likes.toggle(it))}>
      <m3e-icon name="favorite" filled={likes.liked(it.id)}></m3e-icon>{likes.liked(it.id) ? "Remove from Liked songs" : "Like"}
    </button>
    <button role="menuitem" onclick={() => run(() => musicUi.addToPlaylist([it]))}><m3e-icon name="playlist_add"></m3e-icon>Add to playlist</button>
    {#if m.ctx.playlistId && it.setVideoId}
      <button role="menuitem" onclick={() => run(remove)}><m3e-icon name="playlist_remove"></m3e-icon>Remove from this playlist</button>
    {/if}
    <hr />
    {#if it.albumId}
      <button role="menuitem" onclick={() => run(() => nav.go({ name: "music-album", id: it.albumId! }))}><m3e-icon name="album"></m3e-icon>Go to album</button>
    {/if}
    {#if firstArtist?.id}
      <button role="menuitem" onclick={() => run(() => nav.go({ name: "music-artist", id: firstArtist.id! }))}><m3e-icon name="person"></m3e-icon>Go to artist</button>
    {/if}
    {#if !it.localPath}
      <button role="menuitem" onclick={() => run(download)}><m3e-icon name="download"></m3e-icon>Download</button>
    {/if}
    <button role="menuitem" onclick={() => run(copy)}><m3e-icon name="link"></m3e-icon>Copy link</button>
    <button role="menuitem" onclick={() => run(() => ani.ask(`Tell me about the song "${it.title}" by ${artists(it)}, and suggest songs like it.`))}>
      <m3e-icon name="wand_stars"></m3e-icon>Ask Ani
    </button>
  </div>
{/if}

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 900;
  }
  .menu {
    position: fixed;
    z-index: 901;
    min-width: 240px;
    max-width: 320px;
    padding: 8px;
    border-radius: 20px;
    background: var(--md-sys-color-surface-container-high);
    box-shadow: 0 12px 40px rgb(0 0 0 / 45%);
    display: flex;
    flex-direction: column;
    transform-origin: top left;
    outline: none;
  }
  .head {
    display: flex;
    flex-direction: column;
    padding: 8px 12px 10px;
    min-width: 0;
  }
  .head strong,
  .head span {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .head span {
    font-size: 12.5px;
    color: var(--md-sys-color-on-surface-variant);
  }
  button {
    all: unset;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 9px 12px;
    border-radius: 12px;
    font-size: 14px;
    cursor: pointer;
    --m3e-icon-size: 20px;
  }
  button:hover,
  button:focus-visible {
    background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
  }
  button m3e-icon {
    color: var(--md-sys-color-on-surface-variant);
  }
  hr {
    border: 0;
    height: 1px;
    margin: 6px 4px;
    background: var(--md-sys-color-outline-variant);
  }
</style>
