<script lang="ts">
  import { onMount } from "svelte";
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import * as api from "../../lib/api";
  import { musicExports } from "../../lib/music/exports.svelte";
  import { cached } from "../../lib/music/cache";
  import MusicBanner from "../../components/music/MusicBanner.svelte";
  import TrackRow from "../../components/music/TrackRow.svelte";
  import AdSlot from "../../components/AdSlot.svelte";
  import { musicPlayer } from "../../lib/music/player.svelte";
  import { musicUi, shareLink } from "../../lib/music/ui.svelte";
  import { downloads, nav } from "../../lib/state.svelte";
  import type { MusicAlbum } from "../../lib/types";

  let { id }: { id: string } = $props();

  let album = $state<MusicAlbum | null>(null);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      await cached(`album:${id}`, () => api.musicAlbum(id), (a) => (album = a));
    } catch (e) {
      error = typeof e === "string" ? e : "This album couldn't be loaded.";
    }
  });

  const meta = $derived(
    album
      ? [album.artists.map((a) => a.name).join(", "), album.year, album.trackCount ? `${album.trackCount} songs` : null, album.length]
          .filter(Boolean)
          .join(" · ")
      : "",
  );

  async function downloadAll() {
    if (!album) return;
    try {
      const n = await musicExports.download(album.tracks, { title: album.title, thumbnails: album.thumbnails });
      M3eSnackbar.open(`Downloading ${n} song${n === 1 ? "" : "s"} from ${album.title}.`);
    } catch (e) {
      M3eSnackbar.open(`Couldn't download: ${e}`);
    }
  }

  async function copy() {
    await navigator.clipboard.writeText(shareLink({ kind: "album", id } as never)).catch(() => {});
    M3eSnackbar.open("Link copied");
  }
</script>

{#if album}
  <div class="page">
    <MusicBanner title={album.title} thumbs={album.thumbnails} kicker="Album" {meta} description={album.description}>
      {#snippet actions()}
        <m3e-button variant="filled" onclick={() => musicPlayer.playList(album!.tracks, 0, album!.title)}>
          <m3e-icon slot="icon" name="play_arrow" filled></m3e-icon>Play
        </m3e-button>
        <m3e-button
          variant="tonal"
          onclick={() => {
            if (!musicPlayer.shuffle) musicPlayer.toggleShuffle();
            musicPlayer.playList(album!.tracks, Math.floor(Math.random() * album!.tracks.length), album!.title);
          }}
        >
          <m3e-icon slot="icon" name="shuffle"></m3e-icon>Shuffle
        </m3e-button>
        {#if album!.artists[0]?.id}
          <m3e-button variant="tonal" onclick={() => nav.go({ name: "music-artist", id: album!.artists[0].id! })}>
            <m3e-icon slot="icon" name="person"></m3e-icon>Artist
          </m3e-button>
        {/if}
        <m3e-icon-button variant="tonal" aria-label="Add to playlist" onclick={() => musicUi.addToPlaylist(album!.tracks)}>
          <m3e-icon name="playlist_add"></m3e-icon>
        </m3e-icon-button>
        <m3e-icon-button variant="tonal" aria-label="Download album" onclick={downloadAll}>
          <m3e-icon name="download"></m3e-icon>
        </m3e-icon-button>
        <m3e-icon-button variant="tonal" aria-label="Copy link" onclick={copy}>
          <m3e-icon name="link"></m3e-icon>
        </m3e-icon-button>
      {/snippet}
    </MusicBanner>
    <section class="tracks">
      {#each album.tracks as t, i (t.id)}
        <TrackRow item={t} index={i} list={album.tracks} source={album.title} showArt={false} showAlbum={false} numbered />
      {/each}
    </section>
    <AdSlot id="music" />
  </div>
{:else if error}
  <p class="note">{error}</p>
{:else}
  <m3e-loading-indicator class="spin"></m3e-loading-indicator>
{/if}

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding-bottom: 120px;
  }
  .tracks {
    display: flex;
    flex-direction: column;
  }
  .spin {
    display: block;
    margin: 80px auto;
  }
  .note {
    color: var(--md-sys-color-on-surface-variant);
  }
</style>
