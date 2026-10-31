<script lang="ts">
  import { fade } from "svelte/transition";
  import Poster from "../components/Poster.svelte";
  import Rail from "../components/Rail.svelte";
  import * as api from "../lib/api";
  import { providerLabel } from "../lib/options";
  import { openDownload } from "../lib/overlays";
  import type { Person, Target, TitleDetail, TitleSummary } from "../lib/types";
  import { formatLeft, keyOf, kindLabel, library, nav, nextEpisode, settings, targetLabel } from "../lib/state.svelte";

  let { title }: { title: TitleSummary } = $props();

  let detail = $state<TitleDetail | null>(null);
  let error = $state<string | null>(null);
  let season = $state(1);
  let attempt = $state(0);

  let trailerDialog: HTMLElement & { show(): Promise<void> };
  let trailerOpen = $state(false);
  let galleryDialog: HTMLElement & { show(): Promise<void> };
  let galleryIndex = $state(0);

  $effect(() => {
    const t = title;
    void attempt;
    detail = null;
    error = null;
    api
      .detail(t.origin)
      .then((d) => {
        if (t !== title) return;
        detail = d;
        const last = library.last(t);
        season = last?.season ?? d.seasons[0]?.number ?? 1;
      })
      .catch((e) => (error = String(e)));
  });

  const s = $derived(detail?.summary ?? title);
  const x = $derived(detail?.extra);
  const saved = $derived(library.has(title));
  const episodes = $derived(detail?.seasons.find((v) => v.number === season)?.episodes ?? []);
  const episodic = $derived(s.kind !== "movie");
  const last = $derived(library.last(title));

  /** What Play starts: the resume point, the episode after a finished one, or the very first. */
  const playTarget = $derived.by((): Target | null => {
    if (!episodic) return { type: "movie" };
    if (!detail) return null;
    if (last?.episode !== undefined) {
      const at: Target = { type: "episode", season: last.season ?? 1, episode: last.episode };
      const done = last.duration > 0 && last.position / last.duration >= 0.95;
      return done ? (nextEpisode(detail, at) ?? at) : at;
    }
    const first = [...detail.seasons].sort((a, b) => a.number - b.number)[0];
    const ep = first?.episodes.slice().sort((a, b) => a.number - b.number)[0];
    return first && ep ? { type: "episode", season: first.number, episode: ep.number } : null;
  });
  const resuming = $derived(playTarget ? library.resumeAt(title, playTarget) > 0 : false);
  const playLabel = $derived.by(() => {
    if (!playTarget) return "Play";
    const ep = playTarget.type === "episode" ? ` ${targetLabel(playTarget)}` : "";
    return `${resuming ? "Resume" : "Play"}${ep}`;
  });

  function play(target: Target | null) {
    if (!target) return;
    nav.go({ name: "player", title, target, mode: settings.audioMode, detail: detail ?? undefined });
  }

  /** Progress through an episode, for the bar under its row. */
  function watched(n: number) {
    if (!last || last.season !== season || last.episode !== n || !last.duration) return 0;
    return (last.position / last.duration) * 100;
  }

  /** Site categories that some providers list as genres ("300MB Movies", "BollyWood"). */
  const NOT_GENRE = /(\d+\s*mb\b|\bmovies?\b|\bhd\b|\d{3,4}p\b|web-?dl|blu-?ray|dual audio|\b(bolly|holly|tolly|kolly)wood\b)/i;
  const genres = $derived((detail?.genres ?? []).filter((g) => !NOT_GENRE.test(g)));

  const runtime = $derived.by(() => {
    const m = detail?.runtime_minutes;
    if (!m) return null;
    return m >= 60 ? `${Math.floor(m / 60)}h ${m % 60}m` : `${m}m`;
  });

  // ── Formatting ───────────────────────────────────────────────────

  const languageNames = new Intl.DisplayNames(["en"], { type: "language" });
  const language = (code?: string) => {
    if (!code) return null;
    try {
      return languageNames.of(code) ?? code;
    } catch {
      return code;
    }
  };
  const money = (n?: number) =>
    n ? new Intl.NumberFormat("en-US", { style: "currency", currency: "USD", notation: "compact", maximumFractionDigits: 1 }).format(n) : null;
  const count = (n?: number) => (n ? new Intl.NumberFormat("en-US", { notation: "compact", maximumFractionDigits: 1 }).format(n) : null);
  function longDate(iso?: string) {
    if (!iso) return null;
    const [y, m, d] = iso.split("-").map(Number);
    if (!m) return String(y);
    const date = new Date(Date.UTC(y, m - 1, d || 1));
    return date.toLocaleDateString("en-GB", { day: d ? "numeric" : undefined, month: "long", year: "numeric", timeZone: "UTC" });
  }

  /** Cast with photos and roles; falls back to plain names from older sources. */
  const cast = $derived<Person[]>(x?.cast.length ? x.cast : (detail?.cast ?? []).map((name) => ({ name })));
  const crew = $derived<Person[]>(
    x?.crew.length ? x.crew : (detail?.directors ?? []).map((name) => ({ name, role: "Director" })),
  );

  /** Label/value rows for the details card; empty ones are left out. */
  const facts = $derived.by(() => {
    if (!detail) return [];
    const rows: [string, string | null | undefined][] = [
      ["Released", longDate(x?.release_date)],
      ["Status", x?.status],
      ["Rated", x?.certification],
      ["Format", x?.format],
      ["Season", x?.season_label],
      ["Based on", x?.source_material],
      ["Seasons", x?.season_count ? String(x.season_count) : null],
      ["Episodes", x?.episode_count ? String(x.episode_count) : null],
      ["Runtime", runtime ? (episodic ? `${runtime} per episode` : runtime) : null],
      ["Original language", language(x?.original_language)],
      ["Spoken languages", x?.languages.join(", ")],
      ["Country", x?.countries.join(", ")],
      [s.kind === "anime" ? "Studio" : episodic ? "Network" : "Studios", x?.studios.slice(0, 4).join(", ")],
      ["Budget", money(x?.budget)],
      ["Box office", money(x?.revenue)],
      ["Awards", x?.awards],
      ["Original title", s.original_name],
    ];
    return rows.filter((r): r is [string, string] => !!r[1]);
  });

  /** Every provider known to carry this title. */
  const availableOn = $derived([...new Set([s.origin.provider, ...(s.also_on ?? []).map((r) => r.provider)])]);

  const links = $derived.by(() => {
    const ids = s.ids ?? {};
    const out: { label: string; url: string }[] = [];
    if (ids.imdb) out.push({ label: "IMDb", url: `https://www.imdb.com/title/${ids.imdb}/` });
    if (ids.tmdb) {
      const kind = s.origin.provider === "tmdb" ? s.origin.id.split("/")[0] : episodic ? "tv" : "movie";
      out.push({ label: "TMDB", url: `https://www.themoviedb.org/${kind}/${ids.tmdb}` });
    }
    if (ids.anilist) out.push({ label: "AniList", url: `https://anilist.co/anime/${ids.anilist}` });
    if (ids.mal) out.push({ label: "MyAnimeList", url: `https://myanimelist.net/anime/${ids.mal}` });
    if (x?.homepage) out.push({ label: "Official site", url: x.homepage });
    return out;
  });

  /** YouTube embeds go through the privacy-enhanced domain and start playing on open. */
  const trailerSrc = $derived(
    detail?.trailer ? `${detail.trailer.replace("www.youtube.com", "www.youtube-nocookie.com")}?autoplay=1&rel=0` : null,
  );

  function openTrailer() {
    trailerOpen = true;
    trailerDialog.show();
  }

  function openGallery(i: number) {
    galleryIndex = i;
    galleryDialog.show();
  }

  const initials = (name: string) =>
    name
      .split(/\s+/)
      .slice(0, 2)
      .map((w) => w[0])
      .join("")
      .toUpperCase();
</script>

<article class="title-page">
  <div class="banner">
    <!-- No backdrop (some providers only have posters): use a soft, blurred poster instead. -->
    {#if s.backdrop || s.poster}
      <div class="art" class:soft={!s.backdrop} style:background-image={`url("${s.backdrop ?? s.poster}")`}></div>
    {/if}
    <div class="scrim"></div>
  </div>

  <div class="head">
    <div class="poster" style:background-image={s.poster ? `url("${s.poster}")` : undefined}></div>
    <div class="info">
      <h1 class="display">{s.name}</h1>
      {#if x?.tagline}<p class="tagline display">{x.tagline}</p>{/if}
      <div class="meta">
        {#if s.rating}
          <span class="star">
            <m3e-icon name="star" filled></m3e-icon>{s.rating.toFixed(1)}
            {#if x?.rating_source || x?.vote_count}
              <span class="votes">
                {[x?.rating_source, x?.vote_count ? `${count(x.vote_count)} votes` : null].filter(Boolean).join(" · ")}
              </span>
            {/if}
          </span>
        {/if}
        {#if x?.certification}<span class="cert">{x.certification}</span>{/if}
        {#if s.year}<span>{s.year}</span>{/if}
        <span>{kindLabel[s.kind]}</span>
        {#if x?.season_count}<span>{x.season_count} {x.season_count === 1 ? "season" : "seasons"}</span>{/if}
        {#if runtime}<span>{runtime}</span>{/if}
      </div>
      {#if genres.length || s.badges?.length}
        <m3e-chip-set aria-label="Genres">
          {#each genres as g (g)}<m3e-chip>{g}</m3e-chip>{/each}
          {#each s.badges ?? [] as b (b)}<m3e-chip class="badge">{b}</m3e-chip>{/each}
        </m3e-chip-set>
      {/if}
      <div class="actions">
        <m3e-button variant="filled" size="medium" disabled={!playTarget} onclick={() => play(playTarget)}>
          <m3e-icon slot="icon" name="play_arrow" filled></m3e-icon>
          {playLabel}
        </m3e-button>
        <m3e-button variant="tonal" size="medium" toggle selected={saved} onclick={() => library.toggle(title)}>
          <m3e-icon slot="icon" name="add"></m3e-icon>
          <m3e-icon slot="selected-icon" name="check"></m3e-icon>
          My List
        </m3e-button>
        {#if trailerSrc}
          <m3e-button variant="tonal" size="medium" onclick={openTrailer}>
            <m3e-icon slot="icon" name="smart_display"></m3e-icon>
            Trailer
          </m3e-button>
        {/if}
        {#if !episodic}
          <m3e-icon-button id="btn-download" variant="tonal" size="medium" aria-label="Download" onclick={() => openDownload(title, { type: "movie" })}>
            <m3e-icon name="download"></m3e-icon>
          </m3e-icon-button>
          <m3e-tooltip for="btn-download">Download</m3e-tooltip>
        {/if}
        {#if s.kind === "anime"}
          <m3e-segmented-button aria-label="Audio">
            <m3e-button-segment checked={settings.audioMode === "sub"} onclick={() => settings.setAudioMode("sub")}>Sub</m3e-button-segment>
            <m3e-button-segment checked={settings.audioMode === "dub"} onclick={() => settings.setAudioMode("dub")}>Dub</m3e-button-segment>
          </m3e-segmented-button>
        {/if}
      </div>
      {#if resuming && last}
        <span class="resume-hint">{formatLeft(last.duration - last.position)}</span>
      {/if}
    </div>
  </div>

  {#if error}
    <div class="error">
      <p>Couldn't load the details: {error}</p>
      <m3e-button variant="tonal" onclick={() => attempt++}>
        <m3e-icon slot="icon" name="refresh"></m3e-icon>Try again
      </m3e-button>
    </div>
  {/if}

  <div class="body">
    <div class="main">
      {#if s.overview}
        <section>
          <m3e-heading variant="title" size="large" level={2}>Story</m3e-heading>
          <p class="overview">{s.overview}</p>
        </section>
      {/if}

      {#if !detail && !error}
        <m3e-skeleton animation="wave" shape="rounded" class="ghost"></m3e-skeleton>
      {:else if detail}
        {#if detail.seasons.length}
          <section class="episodes" in:fade={{ duration: 250 }}>
            {#if detail.seasons.length > 1}
              <m3e-tabs>
                {#each detail.seasons as v (v.number)}
                  <m3e-tab selected={v.number === season} onclick={() => (season = v.number)}>
                    {v.name ?? `Season ${v.number}`}
                  </m3e-tab>
                {/each}
              </m3e-tabs>
            {:else}
              <m3e-heading variant="title" size="large" level={2}>Episodes</m3e-heading>
            {/if}
            <m3e-list variant="segmented">
              {#each episodes as ep (ep.number)}
                <m3e-list-action onclick={() => play({ type: "episode", season, episode: ep.number })}>
                  <div slot="leading" class="ep-thumb" style:background-image={ep.thumbnail ? `url("${ep.thumbnail}")` : undefined}>
                    <span>{ep.number}</span>
                    {#if watched(ep.number) > 0}
                      <m3e-linear-progress-indicator value={watched(ep.number)}></m3e-linear-progress-indicator>
                    {/if}
                  </div>
                  {ep.title ?? `Episode ${ep.number}`}
                  {#if ep.overview || ep.air_date}
                    <span slot="supporting-text">{ep.overview ?? longDate(ep.air_date)}</span>
                  {/if}
                  <m3e-icon-button
                    slot="trailing"
                    aria-label="Download episode {ep.number}"
                    onclick={(e: Event) => {
                      e.stopPropagation();
                      openDownload(title, { type: "episode", season, episode: ep.number }, detail ?? undefined);
                    }}
                  >
                    <m3e-icon name="download"></m3e-icon>
                  </m3e-icon-button>
                </m3e-list-action>
              {/each}
            </m3e-list>
          </section>
        {/if}

        {#if cast.length}
          <section in:fade={{ duration: 250 }}>
            <m3e-heading variant="title" size="large" level={2}>Cast</m3e-heading>
            <m3e-slide-group threshold="48">
              <div class="people">
                {#each cast as p, i (p.name + i)}
                  {@render person(p)}
                {/each}
              </div>
            </m3e-slide-group>
          </section>
        {/if}

        {#if crew.length}
          <section in:fade={{ duration: 250 }}>
            <m3e-heading variant="title" size="large" level={2}>{s.kind === "anime" ? "Staff" : "Crew"}</m3e-heading>
            <m3e-slide-group threshold="48">
              <div class="people">
                {#each crew as p, i (p.name + i)}
                  {@render person(p)}
                {/each}
              </div>
            </m3e-slide-group>
          </section>
        {/if}

        {#if detail.screenshots.length}
          <section in:fade={{ duration: 250 }}>
            <m3e-heading variant="title" size="large" level={2}>Gallery</m3e-heading>
            <m3e-slide-group threshold="48">
              <div class="gallery">
                {#each detail.screenshots as src, i (src)}
                  <button class="shot" aria-label="Open image {i + 1}" onclick={() => openGallery(i)}>
                    <img {src} alt="" loading="lazy" decoding="async" />
                  </button>
                {/each}
              </div>
            </m3e-slide-group>
          </section>
        {/if}

        {#if x?.keywords.length}
          <section in:fade={{ duration: 250 }}>
            <m3e-heading variant="title" size="large" level={2}>{s.kind === "anime" ? "Tags" : "Keywords"}</m3e-heading>
            <m3e-chip-set aria-label="Keywords">
              {#each x.keywords as k (k)}<m3e-chip class="keyword">{k}</m3e-chip>{/each}
            </m3e-chip-set>
          </section>
        {/if}
      {/if}
    </div>

    {#if detail && (facts.length || availableOn.length || links.length)}
      <aside class="side" in:fade={{ duration: 250 }}>
        <m3e-card variant="filled" class="details">
          <div slot="content">
            <m3e-heading variant="title" size="medium" level={2}>Details</m3e-heading>
            <dl>
              {#each facts as [label, value] (label)}
                <dt>{label}</dt>
                <dd>{value}</dd>
              {/each}
              {#if availableOn.length}
                <dt>Available on</dt>
                <dd>{availableOn.map(providerLabel).join(", ")}</dd>
              {/if}
            </dl>
            {#if links.length}
              <m3e-divider></m3e-divider>
              <m3e-chip-set aria-label="Links" class="links">
                {#each links as l (l.url)}
                  <m3e-assist-chip onclick={() => api.openExternal(l.url)}>
                    <m3e-icon slot="icon" name="open_in_new"></m3e-icon>{l.label}
                  </m3e-assist-chip>
                {/each}
              </m3e-chip-set>
            {/if}
          </div>
        </m3e-card>
      </aside>
    {/if}
  </div>

  {#if x?.recommendations.length}
    <div class="more" in:fade={{ duration: 250 }}>
      <Rail title="More like this" icon="recommend">
        {#each x.recommendations as t (keyOf(t))}
          <Poster title={t} width={150} />
        {/each}
      </Rail>
    </div>
  {/if}
</article>

{#snippet person(p: Person)}
  <div class="person">
    <m3e-avatar class="face">
      {#if p.photo}<img src={p.photo} alt="" loading="lazy" />{:else}{initials(p.name)}{/if}
    </m3e-avatar>
    <span class="person-name">{p.name}</span>
    {#if p.role}<span class="person-role">{p.role}</span>{/if}
  </div>
{/snippet}

<m3e-dialog bind:this={trailerDialog} dismissible onclosed={() => (trailerOpen = false)} class="trailer">
  <span slot="header">{s.name}: trailer</span>
  <div class="video">
    <!-- Only mounted while open, so closing the dialog stops playback. -->
    {#if trailerOpen && trailerSrc}
      <iframe src={trailerSrc} title="Trailer" allow="autoplay; encrypted-media; fullscreen" allowfullscreen></iframe>
    {/if}
  </div>
</m3e-dialog>

<m3e-dialog bind:this={galleryDialog} dismissible class="lightbox">
  <span slot="header">{s.name}</span>
  {#if detail?.screenshots.length}
    <div class="lightbox-body">
      <img src={detail.screenshots[galleryIndex]} alt="" />
      <div class="lightbox-nav">
        <m3e-icon-button
          variant="tonal"
          aria-label="Previous image"
          disabled={galleryIndex === 0}
          onclick={() => (galleryIndex = Math.max(0, galleryIndex - 1))}
        >
          <m3e-icon name="chevron_left"></m3e-icon>
        </m3e-icon-button>
        <span>{galleryIndex + 1} / {detail.screenshots.length}</span>
        <m3e-icon-button
          variant="tonal"
          aria-label="Next image"
          disabled={galleryIndex === detail.screenshots.length - 1}
          onclick={() => detail && (galleryIndex = Math.min(detail.screenshots.length - 1, galleryIndex + 1))}
        >
          <m3e-icon name="chevron_right"></m3e-icon>
        </m3e-icon-button>
      </div>
    </div>
  {/if}
</m3e-dialog>

<style>
  .title-page {
    position: relative;
    padding-bottom: 40px;
  }
  .banner {
    position: relative;
    height: 320px;
    border-radius: var(--md-sys-shape-corner-extra-large, 28px);
    overflow: hidden;
    background: var(--md-sys-color-surface-container-high);
  }
  .art {
    position: absolute;
    inset: 0;
    background: center 25% / cover no-repeat;
  }
  .art.soft {
    filter: blur(32px) saturate(1.3);
    transform: scale(1.25);
    opacity: 0.75;
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
    margin: -160px 32px 0;
    align-items: flex-end;
  }
  .poster {
    flex: none;
    width: 190px;
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
    font-size: clamp(36px, 4vw, 56px);
    line-height: 1.02;
    font-weight: 400;
  }
  .tagline {
    margin: -4px 0 0;
    font-size: 20px;
    font-style: italic;
    color: var(--md-sys-color-on-surface-variant);
  }
  .meta {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 14px;
    align-items: center;
    font-size: 14px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .star {
    display: inline-flex;
    gap: 4px;
    align-items: center;
    color: var(--md-sys-color-on-surface);
    font-weight: 600;
    --m3e-icon-size: 16px;
  }
  .star m3e-icon {
    color: #ffc94d;
  }
  .votes {
    font-weight: 400;
    color: var(--md-sys-color-on-surface-variant);
  }
  .cert {
    padding: 1px 8px;
    border-radius: 6px;
    border: 1px solid var(--md-sys-color-outline);
    font-size: 12px;
    font-weight: 600;
    color: var(--md-sys-color-on-surface);
  }
  .badge {
    --m3e-chip-container-color: var(--md-sys-color-secondary-container);
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
  }
  .resume-hint {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .error {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
    margin: 24px 32px 0;
  }
  .error p {
    margin: 0;
    color: var(--md-sys-color-error);
  }

  /* Main column and a details card beside it; stacks on narrow windows. */
  .body {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 320px;
    gap: 32px;
    margin: 32px 32px 0;
    align-items: start;
  }
  @media (max-width: 1180px) {
    .body {
      grid-template-columns: minmax(0, 1fr);
    }
  }
  .main {
    display: flex;
    flex-direction: column;
    gap: 28px;
    min-width: 0;
  }
  section {
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-width: 0;
  }
  .overview {
    margin: 0;
    max-width: 780px;
    font-size: 15px;
    line-height: 1.65;
  }
  .ghost {
    height: 200px;
  }
  .ep-thumb {
    position: relative;
    overflow: hidden;
    width: 112px;
    aspect-ratio: 16 / 9;
    border-radius: 10px;
    background: var(--md-sys-color-surface-container-highest) center / cover;
    display: grid;
    place-items: center;
    font-weight: 700;
    color: var(--md-sys-color-on-surface-variant);
  }
  .ep-thumb m3e-linear-progress-indicator {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    --m3e-linear-progress-indicator-thickness: 3px;
  }

  .people {
    display: flex;
    gap: 18px;
    padding-block: 4px;
  }
  .person {
    flex: none;
    width: 104px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    text-align: center;
  }
  .face {
    --m3e-avatar-size: 84px;
    --m3e-avatar-font-size: 24px;
  }
  .face img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .person-name {
    font-size: 13px;
    font-weight: 600;
    line-height: 1.25;
  }
  .person-role {
    font-size: 12px;
    line-height: 1.25;
    color: var(--md-sys-color-on-surface-variant);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .gallery {
    display: flex;
    gap: 12px;
  }
  .shot {
    all: unset;
    cursor: zoom-in;
    flex: none;
    width: 280px;
    aspect-ratio: 16 / 9;
    border-radius: var(--md-sys-shape-corner-large, 16px);
    overflow: hidden;
    background: var(--md-sys-color-surface-container-high);
  }
  .shot img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
    transition: transform var(--md-sys-motion-spring-default-spatial, 500ms ease);
  }
  .shot:hover img {
    transform: scale(1.05);
  }
  .shot:focus-visible {
    outline: 2px solid var(--md-sys-color-primary);
    outline-offset: 2px;
  }
  .keyword {
    --m3e-chip-label-text-color: var(--md-sys-color-on-surface-variant);
  }

  .side {
    position: sticky;
    top: 8px;
  }
  .details {
    --m3e-card-shape: var(--md-sys-shape-corner-extra-large, 28px);
    --m3e-card-padding: 20px;
  }
  dl {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: 10px 16px;
    margin: 14px 0 6px;
    font-size: 14px;
  }
  dt {
    color: var(--md-sys-color-on-surface-variant);
    white-space: nowrap;
  }
  dd {
    margin: 0;
    overflow-wrap: anywhere;
  }
  .links {
    margin-top: 12px;
  }

  .more {
    margin: 36px 32px 0;
  }

  .trailer {
    --m3e-dialog-max-width: min(960px, 92vw);
    --m3e-dialog-min-width: min(960px, 92vw);
  }
  .video {
    aspect-ratio: 16 / 9;
    border-radius: var(--md-sys-shape-corner-large, 16px);
    overflow: hidden;
    background: #000;
  }
  .video iframe {
    width: 100%;
    height: 100%;
    border: 0;
    display: block;
  }
  .lightbox {
    --m3e-dialog-max-width: min(1200px, 94vw);
    --m3e-dialog-min-width: min(1200px, 94vw);
  }
  .lightbox-body {
    display: flex;
    flex-direction: column;
    gap: 12px;
    align-items: center;
  }
  .lightbox-body img {
    width: 100%;
    max-height: 72vh;
    object-fit: contain;
    border-radius: var(--md-sys-shape-corner-large, 16px);
  }
  .lightbox-nav {
    display: flex;
    align-items: center;
    gap: 16px;
    font-size: 14px;
    font-variant-numeric: tabular-nums;
  }
</style>
