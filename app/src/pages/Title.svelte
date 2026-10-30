<script lang="ts">
  import { fade } from "svelte/transition";
  import * as api from "../lib/api";
  import type { TitleDetail, TitleSummary } from "../lib/types";
  import { kindLabel, library } from "../lib/state.svelte";

  let { title }: { title: TitleSummary } = $props();

  let detail = $state<TitleDetail | null>(null);
  let error = $state<string | null>(null);
  let season = $state(1);

  $effect(() => {
    const t = title;
    detail = null;
    error = null;
    api
      .detail(t.origin)
      .then((d) => {
        if (t !== title) return;
        detail = d;
        season = d.seasons[0]?.number ?? 1;
      })
      .catch((e) => (error = String(e)));
  });

  const s = $derived(detail?.summary ?? title);
  const saved = $derived(library.has(title));
  const episodes = $derived(detail?.seasons.find((x) => x.number === season)?.episodes ?? []);
  const runtime = $derived.by(() => {
    const m = detail?.runtime_minutes;
    if (!m) return null;
    return m >= 60 ? `${Math.floor(m / 60)}h ${m % 60}m` : `${m}m`;
  });
</script>

<article class="title-page">
  <div class="banner" style:background-image={s.backdrop ? `url("${s.backdrop}")` : undefined}>
    <div class="scrim"></div>
  </div>

  <div class="head">
    <div class="poster" style:background-image={s.poster ? `url("${s.poster}")` : undefined}></div>
    <div class="info">
      <h1 class="display">{s.name}</h1>
      <div class="meta">
        {#if s.rating}<span class="star"><m3e-icon name="star" filled></m3e-icon>{s.rating.toFixed(1)}</span>{/if}
        {#if s.year}<span>{s.year}</span>{/if}
        <span>{kindLabel[s.kind]}</span>
        {#if runtime}<span>{runtime}</span>{/if}
      </div>
      {#if detail?.genres.length}
        <m3e-chip-set aria-label="Genres">
          {#each detail.genres as g (g)}<m3e-chip>{g}</m3e-chip>{/each}
        </m3e-chip-set>
      {/if}
      <div class="actions">
        <m3e-button id="btn-play" variant="filled" size="medium" disabled-interactive>
          <m3e-icon slot="icon" name="play_arrow" filled></m3e-icon>
          Play
        </m3e-button>
        <m3e-tooltip for="btn-play">The player arrives in the next build</m3e-tooltip>
        <m3e-button variant="tonal" size="medium" toggle selected={saved} onclick={() => library.toggle(title)}>
          <m3e-icon slot="icon" name="add"></m3e-icon>
          <m3e-icon slot="selected-icon" name="check"></m3e-icon>
          My List
        </m3e-button>
      </div>
    </div>
  </div>

  <div class="body">
    {#if s.overview}<p class="overview">{s.overview}</p>{/if}

    {#if error}
      <p class="error">Couldn't load details: {error}</p>
    {:else if !detail}
      <m3e-skeleton animation="wave" shape="rounded" class="ghost"></m3e-skeleton>
    {:else}
      <div in:fade={{ duration: 250 }} class="facts">
        {#if detail.directors.length}
          <div><span class="label">Director</span>{detail.directors.join(", ")}</div>
        {/if}
        {#if detail.cast.length}
          <div><span class="label">Cast</span>{detail.cast.slice(0, 8).join(", ")}</div>
        {/if}
      </div>

      {#if detail.seasons.length}
        <section class="episodes" in:fade={{ duration: 250 }}>
          {#if detail.seasons.length > 1}
            <m3e-tabs>
              {#each detail.seasons as x (x.number)}
                <m3e-tab selected={x.number === season} onclick={() => (season = x.number)}>
                  {x.name ?? `Season ${x.number}`}
                </m3e-tab>
              {/each}
            </m3e-tabs>
          {:else}
            <m3e-heading variant="title" size="large" level={2}>Episodes</m3e-heading>
          {/if}
          <m3e-list variant="segmented">
            {#each episodes as ep (ep.number)}
              <m3e-list-item>
                <div slot="leading" class="ep-thumb" style:background-image={ep.thumbnail ? `url("${ep.thumbnail}")` : undefined}>
                  <span>{ep.number}</span>
                </div>
                {ep.title ?? `Episode ${ep.number}`}
                {#if ep.overview || ep.air_date}
                  <span slot="supporting-text">{ep.overview ?? ep.air_date}</span>
                {/if}
                <m3e-icon slot="trailing" name="play_circle"></m3e-icon>
              </m3e-list-item>
            {/each}
          </m3e-list>
        </section>
      {/if}
    {/if}
  </div>
</article>

<style>
  .title-page {
    position: relative;
    padding-bottom: 32px;
  }
  .banner {
    position: relative;
    height: 300px;
    border-radius: var(--md-sys-shape-corner-extra-large, 28px);
    overflow: hidden;
    background: var(--md-sys-color-surface-container-high) center 25% / cover;
  }
  .scrim {
    position: absolute;
    inset: 0;
    background: linear-gradient(0deg, var(--md-sys-color-background) 0%, transparent 70%);
  }
  .head {
    position: relative;
    display: flex;
    gap: 28px;
    margin: -150px 32px 0;
    align-items: flex-end;
  }
  .poster {
    flex: none;
    width: 180px;
    aspect-ratio: 2 / 3;
    border-radius: var(--md-sys-shape-corner-large, 16px);
    background: var(--md-sys-color-surface-container-highest) center / cover;
    box-shadow: 0 18px 40px rgb(0 0 0 / 45%);
  }
  .info {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding-bottom: 6px;
    min-width: 0;
  }
  h1 {
    margin: 0;
    font-size: clamp(30px, 3.4vw, 46px);
    line-height: 1.05;
    font-weight: 800;
  }
  .meta {
    display: flex;
    gap: 14px;
    align-items: center;
    font-size: 14px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .star {
    display: inline-flex;
    gap: 3px;
    align-items: center;
    color: var(--md-sys-color-on-surface);
    font-weight: 600;
    --m3e-icon-size: 16px;
  }
  .star m3e-icon {
    color: #ffc94d;
  }
  .actions {
    display: flex;
    gap: 10px;
  }
  .body {
    display: flex;
    flex-direction: column;
    gap: 20px;
    margin: 28px 32px 0;
  }
  .overview {
    margin: 0;
    max-width: 760px;
    font-size: 15px;
    line-height: 1.6;
  }
  .facts {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 14px;
  }
  .label {
    display: inline-block;
    width: 80px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .error {
    color: var(--md-sys-color-error);
  }
  .ghost {
    height: 160px;
  }
  .episodes {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .ep-thumb {
    width: 112px;
    aspect-ratio: 16 / 9;
    border-radius: 10px;
    background: var(--md-sys-color-surface-container-highest) center / cover;
    display: grid;
    place-items: center;
    font-weight: 700;
    color: var(--md-sys-color-on-surface-variant);
  }
</style>
