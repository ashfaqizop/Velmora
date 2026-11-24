<script lang="ts">
  // What's popular on Velmora this week (from the anonymous play counts), and
  // the best rated titles (from ratings).
  import { fly } from "svelte/transition";
  import Poster from "./Poster.svelte";
  import { trendingSongs, trendingTitles, topRated, type Trend } from "./activity.svelte";
  import { openInApp, songThumb, songs } from "./meta.svelte";
  import { go, href, type TrendTab } from "./nav.svelte";

  let { tab }: { tab: TrendTab } = $props();

  const TABS: { id: TrendTab; label: string; icon: string }[] = [
    { id: "movies", label: "Movies", icon: "movie" },
    { id: "shows", label: "Shows & anime", icon: "tv" },
    { id: "music", label: "Music", icon: "music_note" },
    { id: "rated", label: "Top rated", icon: "star" },
  ];

  let list = $state<(Trend & { average?: number })[] | null>(null);

  $effect(() => {
    const t = tab;
    document.title = "Trending · Velmora Social";
    list = null;
    let alive = true;
    const job =
      t === "movies"
        ? trendingTitles("movie")
        : t === "shows"
          ? trendingTitles("tv")
          : t === "music"
            ? trendingSongs()
            : topRated().then((l) => l.map((r) => ({ k: r.k, people: r.ratings, average: r.average })));
    job.then((l) => alive && (list = l)).catch(() => alive && (list = []));
    return () => (alive = false);
  });
</script>

<section class="head">
  <span class="eyebrow">This week on Velmora</span>
  <h1 class="display">Trending</h1>
  <p class="lead">What people play most, counted anonymously, and the titles Velmora rates highest.</p>
</section>

<m3e-tabs class="tabs">
  {#each TABS as t (t.id)}
    <m3e-tab selected={tab === t.id} onclick={() => go(t.id === "movies" ? "/trending" : `/trending/${t.id}`, { replace: true, keepScroll: true })}>
      <m3e-icon slot="icon" name={t.icon}></m3e-icon>{t.label}
    </m3e-tab>
  {/each}
</m3e-tabs>

{#if list === null}
  <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
{:else if !list.length}
  <p class="empty">
    {tab === "rated" ? "Nothing has three ratings yet. Rate what you watch and it shows up here." : "Not enough plays this week yet. Check back soon."}
  </p>
{:else if tab === "music"}
  <ol class="songs">
    {#each list as s, i (s.k)}
      {@const meta = songs.get(s.k)}
      {#if meta !== null}
        <li in:fly={{ y: 8, duration: 240, delay: Math.min(i, 14) * 25 }}>
          <span class="n">{i + 1}</span>
          <img src={songThumb(s.k)} alt="" loading="lazy" />
          <span class="song">
            <strong>{meta?.title ?? "…"}</strong>
            <span>{meta?.artist ?? ""}</span>
          </span>
          <span class="people">{s.people} listening</span>
          <m3e-icon-button variant="filled" aria-label="Play in Velmora" href={openInApp(`music/${s.k}`)} target="_blank"><m3e-icon name="play_arrow" filled></m3e-icon></m3e-icon-button>
        </li>
      {/if}
    {/each}
  </ol>
{:else}
  <div class="grid">
    {#each list as t, i (t.k)}
      <Poster k={t.k} rank={i + 1} {i} note={tab === "rated" ? `★ ${((t.average ?? 0) / 2).toFixed(1)} · ${t.people} ratings` : `${t.people} watching`} />
    {/each}
  </div>
{/if}

<p class="foot">
  Counts come from people who leave anonymous statistics on. <a href={href("/leaderboards")}>See the leaderboards</a>.
</p>

<style>
  .head {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .eyebrow {
    font-size: 13px;
    font-weight: 700;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--md-sys-color-primary);
  }
  .display {
    margin: 0;
    font-family: var(--display);
    font-weight: 400;
    font-size: clamp(44px, 7vw, 72px);
    line-height: 1;
  }
  .lead {
    margin: 0;
    color: var(--md-sys-color-on-surface-variant);
    font-size: 17px;
  }
  .tabs {
    --m3e-tabs-container-color: transparent;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 22px 18px;
  }
  .songs {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .songs li {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 8px 12px;
    border-radius: 20px;
    transition: background-color 160ms ease;
  }
  .songs li:hover {
    background: color-mix(in srgb, var(--md-sys-color-on-surface) 5%, transparent);
  }
  .n {
    width: 28px;
    text-align: right;
    font-family: var(--display);
    font-size: 26px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .songs img {
    width: 56px;
    height: 56px;
    border-radius: 14px;
    object-fit: cover;
  }
  .song {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .song strong,
  .song span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .song span,
  .people {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .center {
    display: grid;
    place-items: center;
    padding: 48px 0;
  }
  .empty,
  .foot {
    margin: 0;
    text-align: center;
    color: var(--md-sys-color-on-surface-variant);
  }
  .empty {
    padding: 40px 12px;
  }
  .foot {
    font-size: 13px;
  }
  .foot a {
    color: var(--md-sys-color-primary);
  }
  @media (max-width: 560px) {
    .grid {
      grid-template-columns: repeat(auto-fill, minmax(110px, 1fr));
      gap: 16px 12px;
    }
    .people {
      display: none;
    }
  }
</style>
