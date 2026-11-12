<script lang="ts">
  import { onMount } from "svelte";
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import * as api from "../../lib/api";
  import { cached } from "../../lib/music/cache";
  import MusicBanner from "../../components/music/MusicBanner.svelte";
  import TrackRow from "../../components/music/TrackRow.svelte";
  import AdSlot from "../../components/AdSlot.svelte";
  import { musicPlayer } from "../../lib/music/player.svelte";
  import { likes, musicUi, shareLink } from "../../lib/music/ui.svelte";
  import { downloads } from "../../lib/state.svelte";
  import type { MusicPlaylist } from "../../lib/types";

  /** `id` "LM" is Liked songs. */
  let { id }: { id: string } = $props();

  let list = $state<MusicPlaylist | null>(null);
  let error = $state<string | null>(null);
  let more = $state(false);
  let sentinel = $state<HTMLElement>();

  async function load() {
    try {
      await cached(
        `playlist:${id}`,
        () => (id === "LM" ? api.musicLiked() : api.musicPlaylist(id)),
        (l, fresh) => {
          list = l;
          if (id === "LM" && fresh) likes.seed(l.tracks.map((t) => t.id));
        },
      );
    } catch (e) {
      error = typeof e === "string" ? e : "This playlist couldn't be loaded.";
    }
  }

  async function loadMore() {
    if (!list?.next || more) return;
    more = true;
    try {
      const page = await api.musicPlaylistMore(list.next);
      const have = new Set(list.tracks.map((t) => t.id));
      list.tracks = [...list.tracks, ...page.tracks.filter((t) => !have.has(t.id))];
      list.next = page.next;
    } catch {
      list.next = null;
    } finally {
      more = false;
    }
  }

  onMount(() => {
    load();
    const io = new IntersectionObserver(([e]) => e.isIntersecting && loadMore(), { rootMargin: "800px" });
    if (sentinel) io.observe(sentinel);
    return () => io.disconnect();
  });

  const meta = $derived(
    list ? [list.subtitle, list.trackCount ? `${list.trackCount} songs` : `${list.tracks.length} songs`].filter(Boolean).join(" · ") : "",
  );

  async function downloadAll() {
    if (!list) return;
    for (const t of list.tracks) await api.musicDownload($state.snapshot(t)).catch(() => {});
    downloads.refresh();
    M3eSnackbar.open(`Downloading ${list.tracks.length} songs.`);
  }
</script>

{#if list}
  <div class="page">
    <MusicBanner
      title={id === "LM" ? "Liked songs" : list.title}
      thumbs={list.thumbnails}
      kicker={id === "LM" ? "Your likes" : list.editable ? "Your playlist" : "Playlist"}
      {meta}
      description={list.description}
    >
      {#snippet actions()}
        <m3e-button variant="filled" disabled={!list!.tracks.length} onclick={() => musicPlayer.playList(list!.tracks, 0, list!.title)}>
          <m3e-icon slot="icon" name="play_arrow" filled></m3e-icon>Play
        </m3e-button>
        <m3e-button
          variant="tonal"
          disabled={!list!.tracks.length}
          onclick={() => {
            if (list!.shuffle) return musicPlayer.playTarget(list!.shuffle, list!.title);
            if (!musicPlayer.shuffle) musicPlayer.toggleShuffle();
            musicPlayer.playList(list!.tracks, Math.floor(Math.random() * list!.tracks.length), list!.title);
          }}
        >
          <m3e-icon slot="icon" name="shuffle"></m3e-icon>Shuffle
        </m3e-button>
        {#if list!.tracks[0]}
          <m3e-button variant="tonal" onclick={() => musicPlayer.playSong(list!.tracks[0], `Radio from ${list!.title}`)}>
            <m3e-icon slot="icon" name="radio"></m3e-icon>Radio
          </m3e-button>
        {/if}
        <m3e-icon-button variant="tonal" aria-label="Download all" onclick={downloadAll}>
          <m3e-icon name="download"></m3e-icon>
        </m3e-icon-button>
        {#if id !== "LM"}
          <m3e-icon-button
            variant="tonal"
            aria-label="Copy link"
            onclick={async () => {
              await navigator.clipboard.writeText(shareLink({ kind: "playlist", id } as never)).catch(() => {});
              M3eSnackbar.open("Link copied");
            }}
          >
            <m3e-icon name="link"></m3e-icon>
          </m3e-icon-button>
        {/if}
      {/snippet}
    </MusicBanner>
    <section class="tracks">
      {#each list.tracks as t, i (t.id + i)}
        <TrackRow
          item={t}
          index={i}
          list={list.tracks}
          source={list.title}
          numbered
          ctx={list.editable ? { playlistId: list.id, onRemoved: () => (list!.tracks = list!.tracks.filter((x) => x !== t)) } : {}}
        />
      {:else}
        <p class="note">No songs here yet. Add some from any song's menu.</p>
      {/each}
    </section>
    <div bind:this={sentinel} class="sentinel">{#if more}<m3e-loading-indicator></m3e-loading-indicator>{/if}</div>
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
  .sentinel {
    display: grid;
    place-items: center;
    min-height: 24px;
  }
  .spin {
    display: block;
    margin: 80px auto;
  }
  .note {
    color: var(--md-sys-color-on-surface-variant);
  }
</style>
