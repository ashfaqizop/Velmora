<script lang="ts">
  import AdSlot from "../components/AdSlot.svelte";
  import TrackRow from "../components/music/TrackRow.svelte";
  import { musicSession } from "../lib/music/session.svelte";
  import { nav } from "../lib/state.svelte";
  import type { MusicItem } from "../lib/types";
  import { untrack } from "svelte";
  import { personalize } from "../lib/ani/personalize.svelte";
  import { fly, scale, slide } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import Poster from "../components/Poster.svelte";
  import PoweredBy from "../components/PoweredBy.svelte";
  import * as api from "../lib/api";
  import type { Narrowed, TitleSummary } from "../lib/types";
  import { keyOf } from "../lib/state.svelte";

  let { query }: { query: string } = $props();

  /** How long the "finished" card stays before it folds away. */
  const DONE_HOLD_MS = 1600;

  let results = $state<TitleSummary[]>([]);
  let answered = $state(0);
  let failed = $state(0);
  let running = $state(false);
  let showStatus = $state(false);
  let attempt = $state(0);
  let run = 0;

  // The engine looks the results over: if one of them is exactly what was
  // typed, only that shows, with a button for the full list.
  let narrowed = $state<Narrowed>({ exact: [], ranked: [] });
  let extensive = $state(false);
  let narrowSeq = 0;
  const exactOnly = $derived(narrowed.exact.length > 0 && !extensive);
  const shown = $derived(exactOnly ? narrowed.exact : narrowed.ranked.length ? narrowed.ranked : results);
  let hideTimer: ReturnType<typeof setTimeout> | undefined;

  /** Same title from different providers: same name, year and type. */
  const sameTitle = (t: TitleSummary) =>
    `${t.name.toLowerCase().replace(/[^\p{L}\p{N}]+/gu, "")}:${t.year ?? ""}:${t.kind === "movie" ? "m" : "s"}`;

  function merge(list: TitleSummary[], incoming: TitleSummary[]) {
    const out = [...list];
    for (const item of incoming) {
      const i = out.findIndex((x) => keyOf(x) === keyOf(item) || sameTitle(x) === sameTitle(item));
      if (i < 0) out.push(item);
      else if (!out[i].poster && item.poster) out[i] = item; // keep the copy with artwork
    }
    return out;
  }

  /** Asks the engine which results exactly match; the newest answer wins. */
  function narrow(q: string, mine: number) {
    const seq = ++narrowSeq;
    api
      .searchNarrow(q, $state.snapshot(results) as TitleSummary[])
      .then((n) => {
        if (mine === run && seq === narrowSeq) narrowed = n;
      })
      .catch((e) => console.warn("narrowing failed", e));
  }

  $effect(() => {
    const q = query;
    void attempt;
    const mine = ++run;
    clearTimeout(hideTimer);
    results = [];
    narrowed = { exact: [], ranked: [] };
    extensive = false;
    answered = 0;
    failed = 0;
    running = true;
    showStatus = true;
    untrack(() => personalize.search(q));
    api
      .searchStream(q, (batch) => {
        if (mine !== run) return; // a newer query took over
        answered += 1;
        if (batch.error) failed += 1;
        results = merge(results, batch.items);
        narrow(q, mine);
      })
      .catch((e) => console.warn("search failed", e))
      .finally(() => {
        if (mine !== run) return;
        running = false;
        hideTimer = setTimeout(() => mine === run && (showStatus = false), DONE_HOLD_MS);
      });
    return () => clearTimeout(hideTimer);
  });

  const allFailed = $derived(!running && answered > 0 && failed === answered);

  /** Folds the card's height away while fading it, so results move up smoothly. */
  function fold(node: Element) {
    const s = slide(node, { duration: 450, easing: cubicOut });
    return { ...s, css: (t: number, u: number) => `${s.css?.(t, u) ?? ""}opacity:${t};` };
  }

  // Signed in to YouTube Music: a few songs that match, under the titles.
  let songs = $state<MusicItem[]>([]);
  $effect(() => {
    const q = query.trim();
    songs = [];
    if (!musicSession.signedIn || q.length < 2) return;
    const timer = setTimeout(() => {
      api
        .musicSearch(q, "songs")
        .then((r) => (songs = r.shelves.flatMap((x) => x.items).filter((x) => x.kind === "song").slice(0, 4)))
        .catch(() => {});
    }, 500);
    return () => clearTimeout(timer);
  });
</script>

<div class="page">
  <m3e-heading variant="headline" size="large" level={1}>Results for “{query}”</m3e-heading>

  {#if showStatus}
    <div class="status-wrap" out:fold>
      <m3e-card variant="filled" class="status" in:fly={{ y: -8, duration: 300 }}>
        <div slot="content" class="status-row" aria-live="polite">
          <div class="status-icon">
            {#if running}
              <m3e-loading-indicator></m3e-loading-indicator>
            {:else}
              <span class="done" in:scale={{ start: 0.4, duration: 350, easing: cubicOut }}>
                <m3e-icon name="check_circle" filled></m3e-icon>
              </span>
            {/if}
          </div>
          <div class="status-text">
            <strong>{running ? "Searching through sources" : "Search finished"}</strong>
            <span>
              {answered}
              {answered === 1 ? "source" : "sources"} answered · {results.length}
              {results.length === 1 ? "result" : "results"}
            </span>
          </div>
          <div class="credit"><PoweredBy height={12} /></div>
        </div>
      </m3e-card>
    </div>
  {/if}

  {#if results.length}
    <div class="grid" class:exact={exactOnly}>
      {#each shown as t (keyOf(t))}
        <Poster title={t} />
      {/each}
    </div>
    {#if exactOnly}
      <m3e-card variant="filled" class="more" in:fly={{ y: 8, duration: 300 }}>
        <div slot="content" class="more-row">
          <m3e-icon name="travel_explore"></m3e-icon>
          <div class="more-text">
            <strong>Can't find the movie?</strong>
            <span>{results.length - narrowed.exact.length} more {results.length - narrowed.exact.length === 1 ? "result" : "results"} from the sources.</span>
          </div>
          <m3e-button variant="filled" onclick={() => (extensive = true)}>Click me for extensive search</m3e-button>
        </div>
      </m3e-card>
    {:else}
      <!-- Only under the full list, never next to an exact match. -->
      {#if shown.length >= 12}<AdSlot id="search" />{/if}
    {/if}
    {#if !exactOnly && extensive && narrowed.exact.length}
      <div class="back">
        <m3e-button variant="text" onclick={() => (extensive = false)}>
          <m3e-icon slot="icon" name="arrow_upward"></m3e-icon>Show only the exact match
        </m3e-button>
      </div>
    {/if}
  {:else if running}
    <div class="grid">
      {#each Array(12) as _, i (i)}
        <m3e-skeleton animation="wave" shape="rounded" class="ghost"></m3e-skeleton>
      {/each}
    </div>
  {:else if allFailed}
    <div class="empty">
      <m3e-icon name="cloud_off"></m3e-icon>
      <m3e-heading variant="title" size="large">The sources didn't answer</m3e-heading>
      <p>Check your connection, then try again.</p>
      <m3e-button variant="tonal" onclick={() => attempt++}>
        <m3e-icon slot="icon" name="refresh"></m3e-icon>Try again
      </m3e-button>
    </div>
  {:else}
    <div class="empty">
      <m3e-icon name="search_off"></m3e-icon>
      <m3e-heading variant="title" size="large">No matches</m3e-heading>
      <p>Try the original title, or fewer words.</p>
    </div>
  {/if}

  {#if songs.length}
    <section class="songs-row" in:fly={{ y: 10, duration: 300 }}>
      <header>
        <h2 class="display">Songs</h2>
        <m3e-button variant="text" size="small" onclick={() => nav.go({ name: "music-search", query })}>
          More in Music<m3e-icon slot="trailing-icon" name="arrow_forward"></m3e-icon>
        </m3e-button>
      </header>
      {#each songs as s, i (s.id)}
        <TrackRow item={s} index={i} list={songs} source={`Search: ${query}`} showAlbum={false} />
      {/each}
    </section>
  {/if}
</div>

<style>
  .songs-row {
    margin-top: 28px;
  }
  .songs-row header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .songs-row h2 {
    margin: 0 0 6px;
    font-size: 24px;
    font-weight: 400;
  }
  .page {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding-bottom: 32px;
  }
  .status-wrap {
    /* Keeps the page gap from jumping when the card folds away. */
    margin-bottom: -4px;
  }
  .status {
    --m3e-card-shape: var(--md-sys-shape-corner-large, 16px);
    --m3e-card-padding: 10px 16px;
  }
  .status-row {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .status-icon {
    flex: none;
    width: 40px;
    height: 40px;
    display: grid;
    place-items: center;
    --m3e-loading-indicator-container-size: 40px;
    --m3e-loading-indicator-active-indicator-size: 30px;
  }
  .done {
    display: grid;
    color: var(--md-sys-color-primary);
    --m3e-icon-size: 32px;
  }
  .status-text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .status-text strong {
    font-weight: 700;
  }
  .status-text span {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
    font-variant-numeric: tabular-nums;
  }
  .credit {
    flex: none;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 20px 16px;
  }
  /* A lone exact match shouldn't stretch across the whole row. */
  .grid.exact {
    grid-template-columns: repeat(auto-fill, minmax(150px, 190px));
  }
  .more {
    --m3e-card-shape: var(--md-sys-shape-corner-large, 16px);
    --m3e-card-padding: 12px 16px;
  }
  .more-row {
    display: flex;
    align-items: center;
    gap: 14px;
    flex-wrap: wrap;
    --m3e-icon-size: 28px;
  }
  .more-row > m3e-icon {
    flex: none;
    color: var(--md-sys-color-primary);
  }
  .more-text {
    flex: 1;
    min-width: 200px;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .more-text strong {
    font-weight: 700;
  }
  .more-text span {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .back {
    display: flex;
    justify-content: center;
  }
  .ghost {
    aspect-ratio: 2 / 3;
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 64px 0;
    color: var(--md-sys-color-on-surface-variant);
    --m3e-icon-size: 48px;
  }
  .empty p {
    margin: 0;
  }
</style>
