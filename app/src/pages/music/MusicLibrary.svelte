<script lang="ts">
  // Library: Liked songs, playlists, albums, artists and downloaded songs.
  import { untrack } from "svelte";
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import * as api from "../../lib/api";
  import { musicExports } from "../../lib/music/exports.svelte";
  import { cached } from "../../lib/music/cache";
  import MusicCard from "../../components/music/MusicCard.svelte";
  import TrackRow from "../../components/music/TrackRow.svelte";
  import { musicPlayer } from "../../lib/music/player.svelte";
  import { likes } from "../../lib/music/ui.svelte";
  import { nav } from "../../lib/state.svelte";
  import type { MusicItem, MusicLibraryKind, OfflineSong } from "../../lib/types";

  type Tab = "liked" | "playlists" | "albums" | "artists" | "downloaded";
  let { tab = "liked" }: { tab?: Tab } = $props();

  const TABS: [Tab, string, string][] = [
    ["liked", "Liked songs", "favorite"],
    ["playlists", "Playlists", "queue_music"],
    ["albums", "Albums", "album"],
    ["artists", "Artists", "person"],
    ["downloaded", "Downloaded", "download_done"],
  ];

  let items = $state<MusicItem[]>([]);
  let offline = $state<OfflineSong[]>([]);

  // A song that finishes downloading shows up without leaving the tab.
  $effect(() => {
    const n = musicExports.landed;
    if (!n || untrack(() => tab) !== "downloaded") return;
    api
      .musicDownloads()
      .then((o) => {
        offline = o;
        items = o.map((x) => ({ ...x.item, localPath: x.path }));
      })
      .catch(() => {});
  });

  const STAGE: Record<string, string> = { waiting: "Waiting", downloading: "Downloading", converting: "Tagging" };
  let loading = $state(true);
  let error = $state<string | null>(null);

  $effect(() => {
    const t = tab;
    loading = true;
    error = null;
    items = [];
    (async () => {
      try {
        if (t === "liked") {
          await cached("library:liked", () => api.musicLiked(), (p, fresh) => {
            if (tab !== t) return;
            items = p.tracks;
            loading = false;
            if (fresh) likes.seed(p.tracks.map((i) => i.id));
          });
        } else if (t === "downloaded") {
          offline = await api.musicDownloads();
          items = offline.map((o) => ({ ...o.item, localPath: o.path }));
        } else {
          await cached(`library:${t}`, () => api.musicLibrary(t as MusicLibraryKind), (list) => {
            if (tab !== t) return;
            items = list;
            loading = false;
          });
        }
      } catch (e) {
        error = typeof e === "string" ? e : "Your library couldn't be loaded.";
      } finally {
        loading = false;
      }
    })();
  });

  async function remove(song: MusicItem) {
    if (!song.localPath) return;
    try {
      await api.musicDownloadDelete(song.localPath);
      items = items.filter((i) => i !== song);
      M3eSnackbar.open(`Deleted ${song.title} from this PC`);
    } catch (e) {
      M3eSnackbar.open(String(e));
    }
  }
</script>

<div class="library">
  <h1 class="display">Library</h1>
  <m3e-segmented-button class="tabs">
    {#each TABS as [id, label, icon] (id)}
      <m3e-button-segment checked={tab === id} onclick={() => nav.replace({ name: "music-library", tab: id })}>
        <m3e-icon slot="icon" name={icon}></m3e-icon>{label}
      </m3e-button-segment>
    {/each}
  </m3e-segmented-button>

  {#if tab === "downloaded" && musicExports.active.length}
    <div class="jobs" aria-live="polite">
      <strong class="jobs-head">Downloading {musicExports.active.length} song{musicExports.active.length === 1 ? "" : "s"}</strong>
      {#each musicExports.active as j (j.videoId)}
        <div class="job">
          <span class="jt"><strong>{j.title}</strong><span>{j.artist}</span></span>
          <span class="js">{STAGE[j.stage] ?? ""} · {j.format.toUpperCase()}</span>
          <span class="jbar"><span style:width="{Math.round((j.stage === "converting" ? 1 : j.progress) * 100)}%"></span></span>
        </div>
      {/each}
    </div>
  {/if}

  {#if loading}
    <m3e-loading-indicator class="spin"></m3e-loading-indicator>
  {:else if error}
    <p class="note">{error}</p>
  {:else if !items.length}
    <div class="empty">
      <m3e-icon name={TABS.find((t) => t[0] === tab)?.[2] ?? "library_music"}></m3e-icon>
      <p>
        {#if tab === "downloaded"}Songs you download play here without a connection. Download from any song's menu.
        {:else if tab === "liked"}Songs you like show up here.
        {:else}Nothing saved here yet.{/if}
      </p>
    </div>
  {:else if tab === "liked" || tab === "downloaded"}
    <div class="bar">
      <m3e-button variant="filled" onclick={() => musicPlayer.playList(items, 0, tab === "liked" ? "Liked songs" : "Downloaded")}>
        <m3e-icon slot="icon" name="play_arrow" filled></m3e-icon>Play all
      </m3e-button>
      <span class="count">{items.length} songs</span>
    </div>
    <section class="tracks">
      {#each items as t, i (t.id + i)}
        <div class="row-wrap">
          <TrackRow item={t} index={i} list={items} source={tab === "liked" ? "Liked songs" : "Downloaded"} numbered />
          {#if t.localPath}
            <m3e-icon-button class="del" aria-label="Delete from this PC" onclick={() => remove(t)}>
              <m3e-icon name="delete"></m3e-icon>
            </m3e-icon-button>
          {/if}
        </div>
      {/each}
    </section>
  {:else}
    <section class="grid">
      {#each items as item, i (item.id + i)}
        <MusicCard {item} size={176} />
      {/each}
    </section>
  {/if}
</div>

<style>
  .jobs {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 16px 18px;
    border-radius: 24px;
    background: var(--md-sys-color-surface-container);
  }
  .jobs-head {
    font-size: 14px;
  }
  .job {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
    gap: 4px 16px;
  }
  .jt {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .jt strong,
  .jt span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .jt span,
  .js {
    font-size: 12.5px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .jbar {
    grid-column: 1 / -1;
    height: 4px;
    border-radius: 2px;
    overflow: hidden;
    background: var(--md-sys-color-surface-container-highest);
  }
  .jbar span {
    display: block;
    height: 100%;
    border-radius: 2px;
    background: var(--md-sys-color-primary);
    transition: width 300ms ease;
  }
  .library {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding-bottom: 120px;
  }
  h1 {
    margin: 8px 0 0;
    font-size: clamp(40px, 4.6vw, 64px);
    font-weight: 400;
    line-height: 1;
  }
  .tabs {
    align-self: flex-start;
  }
  .bar {
    display: flex;
    align-items: center;
    gap: 16px;
  }
  .count {
    color: var(--md-sys-color-on-surface-variant);
    font-size: 14px;
  }
  .tracks {
    display: flex;
    flex-direction: column;
  }
  .row-wrap {
    display: flex;
    align-items: center;
  }
  .row-wrap > :global(.row) {
    flex: 1;
    min-width: 0;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(176px, 1fr));
    gap: 26px 18px;
  }
  .spin {
    align-self: center;
    margin: 60px 0;
  }
  .note {
    color: var(--md-sys-color-on-surface-variant);
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 60px 0;
    color: var(--md-sys-color-on-surface-variant);
    --m3e-icon-size: 40px;
  }
</style>
