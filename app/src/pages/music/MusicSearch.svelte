<script lang="ts">
  // Music search: the top result as a big card, then songs, artists, albums,
  // videos and playlists. Filter chips narrow it to one kind.
  import * as api from "../../lib/api";
  import Art from "../../components/music/Art.svelte";
  import ShelfView from "../../components/music/ShelfView.svelte";
  import TrackRow from "../../components/music/TrackRow.svelte";
  import MusicCard from "../../components/music/MusicCard.svelte";
  import { artists, openItem, playCollection, playable } from "../../lib/music/ui.svelte";
  import { musicPlayer } from "../../lib/music/player.svelte";
  import type { MusicSearchFilter, MusicSearchResults } from "../../lib/types";

  let { query }: { query: string } = $props();

  const FILTERS: [MusicSearchFilter, string][] = [
    ["all", "All"],
    ["songs", "Songs"],
    ["videos", "Videos"],
    ["albums", "Albums"],
    ["artists", "Artists"],
    ["playlists", "Playlists"],
  ];

  let filter = $state<MusicSearchFilter>("all");
  let results = $state<MusicSearchResults | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let seq = 0;

  $effect(() => {
    const q = query.trim();
    const f = filter;
    if (!q) return;
    const mine = ++seq;
    loading = true;
    error = null;
    const timer = setTimeout(async () => {
      try {
        const r = await api.musicSearch(q, f);
        if (mine === seq) results = r;
      } catch (e) {
        if (mine === seq) error = typeof e === "string" ? e : "Search didn't work.";
      } finally {
        if (mine === seq) loading = false;
      }
    }, 250);
    return () => clearTimeout(timer);
  });

  const top = $derived(results?.top ?? null);
  const songShelf = $derived(results?.shelves.find((s) => s.display === "list" && s.items.some(playable)));
  const otherShelves = $derived(results?.shelves.filter((s) => s !== songShelf) ?? []);

  const listed = $derived(filter === "all" ? otherShelves : (results?.shelves ?? []));

  function playTopClick(e: MouseEvent) {
    e.stopPropagation();
    playTop();
  }

  function playTop() {
    if (!top) return;
    if (results?.topAction && !playable(top)) return musicPlayer.playTarget(results.topAction, top.title);
    if (playable(top)) return openItem(top);
    playCollection(top);
  }
</script>

<div class="search">
  <h1 class="display">“{query}”</h1>
  <m3e-chip-set>
    {#each FILTERS as [id, label] (id)}
      <m3e-filter-chip selected={filter === id} onclick={() => (filter = id)}>{label}</m3e-filter-chip>
    {/each}
  </m3e-chip-set>

  {#if loading && !results}
    <m3e-loading-indicator class="spin"></m3e-loading-indicator>
  {:else if error}
    <p class="note">{error}</p>
  {:else if results && !results.shelves.length && !top}
    <p class="note">Nothing found for “{query}”.</p>
  {:else if results}
    <div class="results" class:dim={loading}>
      {#if filter === "all" && (top || songShelf)}
        <div class="top-row">
          {#if top}
            <div
              class="top"
              role="button"
              tabindex="0"
              onclick={() => openItem(top)}
              onkeydown={(e) => e.key === "Enter" && openItem(top)}
            >
              <Art thumbs={top.thumbnails} size={140} round={top.kind === "artist"} alt={top.title} />
              <span class="kind">{top.kind === "artist" ? "Artist" : top.kind === "album" ? "Album" : top.kind === "playlist" ? "Playlist" : top.kind === "video" ? "Video" : "Song"}</span>
              <strong class="display">{top.title}</strong>
              <span class="by">{artists(top)}</span>
              <m3e-button variant="filled" onclick={playTopClick}>
                <m3e-icon slot="icon" name="play_arrow" filled></m3e-icon>Play
              </m3e-button>
            </div>
          {/if}
          {#if songShelf}
            <div class="songs">
              <h2 class="display">Songs</h2>
              {#each songShelf.items.slice(0, 5) as s, i (s.id)}
                <TrackRow item={s} index={i} list={songShelf.items} source={`Search: ${query}`} showAlbum={false} />
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      {#each listed as shelf (shelf.id)}
        {#if filter !== "all" && shelf.display === "list" && shelf.items.some(playable)}
          <section class="list">
            {#each shelf.items as s, i (s.id + i)}
              <TrackRow item={s} index={i} list={shelf.items} source={`Search: ${query}`} />
            {/each}
          </section>
        {:else if filter !== "all"}
          <section class="grid">
            {#each shelf.items as item, i (item.id + i)}
              <MusicCard {item} list={shelf.items} size={176} />
            {/each}
          </section>
        {:else}
          <ShelfView {shelf} />
        {/if}
      {/each}
    </div>
  {/if}
</div>

<style>
  .search {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding-bottom: 120px;
  }
  h1 {
    margin: 8px 0 0;
    font-size: clamp(32px, 3.6vw, 52px);
    font-weight: 400;
  }
  h2 {
    margin: 0 0 8px;
    font-size: 26px;
    font-weight: 400;
  }
  .spin {
    align-self: center;
    margin: 60px 0;
  }
  .note {
    color: var(--md-sys-color-on-surface-variant);
  }
  .results {
    display: flex;
    flex-direction: column;
    gap: 34px;
    transition: opacity 200ms ease;
  }
  .results.dim {
    opacity: 0.55;
  }
  .top-row {
    display: grid;
    grid-template-columns: minmax(260px, 340px) minmax(0, 1fr);
    gap: 24px;
  }
  .top {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
    padding: 24px;
    border-radius: 28px;
    background: var(--md-sys-color-surface-container);
    cursor: pointer;
    transition: background 200ms ease;
  }
  .top:hover {
    background: var(--md-sys-color-surface-container-high);
  }
  .kind {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--md-sys-color-primary);
    margin-top: 6px;
  }
  .top strong {
    font-size: 30px;
    font-weight: 400;
    line-height: 1.1;
  }
  .by {
    color: var(--md-sys-color-on-surface-variant);
    font-size: 14px;
    margin-bottom: 6px;
  }
  .songs {
    min-width: 0;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(176px, 1fr));
    gap: 24px 18px;
  }
  @media (max-width: 1100px) {
    .top-row {
      grid-template-columns: 1fr;
    }
  }
</style>
