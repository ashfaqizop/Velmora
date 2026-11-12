<script lang="ts">
  // Music Home's own rows, from what this PC has played (musicdb.rs): mixes
  // from your top artists and this time of day, then Recently played, On
  // repeat and Forgotten favorites. A row needs a few songs before it shows.
  import { onMount } from "svelte";
  import * as api from "../../lib/api";
  import ShelfView from "./ShelfView.svelte";
  import Art from "./Art.svelte";
  import { musicPlayer } from "../../lib/music/player.svelte";
  import type { MusicItem, MusicShelf } from "../../lib/types";

  let data = $state<api.MusicPersonal | null>(null);

  onMount(() => {
    api
      .musicPersonal()
      .then((d) => (data = d))
      .catch(() => {});
  });

  // A song played from Downloads remembers its file; these rows stream.
  const clean = (list: MusicItem[]) => list.map(({ localPath: _, ...i }) => i as MusicItem);

  const shelves = $derived.by((): MusicShelf[] => {
    if (!data) return [];
    const rows: [string, string, MusicItem[]][] = [
      ["mine-recent", "Recently played", data.recent],
      ["mine-repeat", "On repeat", data.onRepeat],
      ["mine-forgotten", "Forgotten favorites", data.forgotten],
    ];
    return rows.filter(([, , items]) => items.length >= 3).map(([id, title, items]) => ({ id, title, items: clean(items), display: "card", more: null }));
  });

  /** A mix is YouTube Music's radio for its seed song. */
  function play(m: api.MusicPersonal["mixes"][number]) {
    musicPlayer.playTarget({ videoId: m.seed.id, playlistId: `RDAMVM${m.seed.id}`, params: null }, m.title);
  }
</script>

{#if data?.mixes.length}
  <section class="mixes">
    <h2>Mixes for you</h2>
    <div class="row">
      {#each data.mixes as m (m.title)}
        <button class="mix" onclick={() => play(m)} aria-label="Play {m.title}">
          <span class="art">
            <Art thumbs={m.seed.thumbnails} size={196} alt="" />
            <span class="shade"></span>
            <span class="tag"><m3e-icon name="all_inclusive"></m3e-icon>Mix</span>
            <span class="go"><m3e-icon name="play_arrow" filled></m3e-icon></span>
          </span>
          <strong>{m.title}</strong>
          <span class="sub">{m.subtitle}</span>
        </button>
      {/each}
    </div>
  </section>
{/if}

{#each shelves as s (s.id)}
  <ShelfView shelf={s} />
{/each}

<style>
  .mixes {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  h2 {
    margin: 0;
    font-size: 26px;
    font-weight: 400;
  }
  .row {
    display: flex;
    gap: 18px;
    overflow-x: auto;
    scrollbar-width: none;
    padding: 4px 2px 6px;
  }
  .mix {
    all: unset;
    cursor: pointer;
    flex: none;
    width: 196px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .mix:focus-visible .art {
    outline: 2px solid var(--md-sys-color-primary);
    outline-offset: 3px;
  }
  .art {
    position: relative;
    display: block;
    width: 196px;
    height: 196px;
    margin-bottom: 6px;
    border-radius: 20px;
    overflow: hidden;
  }
  .art :global(img) {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 400ms ease;
  }
  .mix:hover .art :global(img) {
    transform: scale(1.05);
  }
  /* The brand colours over the cover say "mix", not "this one song". */
  .shade {
    position: absolute;
    inset: 0;
    background: linear-gradient(160deg, color-mix(in srgb, var(--brand-indigo) 55%, transparent), color-mix(in srgb, var(--brand-coral) 45%, transparent));
    mix-blend-mode: multiply;
  }
  .tag {
    position: absolute;
    left: 12px;
    top: 12px;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 10px 4px 8px;
    border-radius: 12px;
    font-size: 12px;
    font-weight: 800;
    color: #fff;
    background: rgb(0 0 0 / 35%);
    backdrop-filter: blur(8px);
    --m3e-icon-size: 16px;
  }
  .go {
    position: absolute;
    right: 12px;
    bottom: 12px;
    display: grid;
    place-items: center;
    width: 44px;
    height: 44px;
    border-radius: 50%;
    background: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
    opacity: 0;
    transform: translateY(6px);
    transition:
      opacity 200ms,
      transform 200ms;
  }
  .mix:hover .go,
  .mix:focus-visible .go {
    opacity: 1;
    transform: none;
  }
  strong {
    font-weight: 700;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .sub {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
