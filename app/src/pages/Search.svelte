<script lang="ts">
  import { personalize } from "../lib/ani/personalize.svelte";
  import { fly, scale, slide } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import Poster from "../components/Poster.svelte";
  import PoweredBy from "../components/PoweredBy.svelte";
  import * as api from "../lib/api";
  import type { TitleSummary } from "../lib/types";
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

  $effect(() => {
    const q = query;
    void attempt;
    const mine = ++run;
    clearTimeout(hideTimer);
    results = [];
    answered = 0;
    failed = 0;
    running = true;
    showStatus = true;
    personalize.search(q);
    api
      .searchStream(q, (batch) => {
        if (mine !== run) return; // a newer query took over
        answered += 1;
        if (batch.error) failed += 1;
        results = merge(results, batch.items);
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
    <div class="grid">
      {#each results as t (keyOf(t))}
        <Poster title={t} />
      {/each}
    </div>
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
</div>

<style>
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
