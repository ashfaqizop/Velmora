<script lang="ts">
  // Platform analytics: the anonymous app stats (installs, versions, players,
  // sources, what people watch) next to account growth from Supabase.
  import { onMount } from "svelte";
  import { call, sayError } from "../api";
  import { count } from "../format";
  import Columns from "./Columns.svelte";
  import Hours from "./Hours.svelte";
  import Shares from "./Shares.svelte";

  interface Row {
    key: string;
    count: number;
    share: number;
  }
  interface Data {
    days: number;
    installs: { day: string; count: number }[];
    versions: Row[];
    engines: Row[];
    quality: Row[];
    sources: Row[];
    hours: number[];
    titles: { key: string; count: number; name: string; year: number | null; poster: string | null }[];
    music: { key: string; count: number }[];
    growth: { day: string; signups: number; follows: number; ratings: number; reviews: number; activity: number }[];
    ads: { day: string; count: number }[];
    note: string;
  }

  const SOURCE_NAMES: Record<string, string> = {
    hd_hub: "HDHub", hi_anime: "HiAnime", zoko_anime: "ZokoAnime", movie_box: "MovieBox", four_k_hd_hub: "4KHDHub",
    dramachi: "Dramachi", anime_sama: "Anime-Sama", french_stream: "French Stream", coflix: "Coflix", yts: "YTS",
    eztv: "EZTV", nyaa: "Nyaa", iptv_org: "Live TV", youtube_music: "YouTube Music",
  };
  const ENGINE_NAMES: Record<string, string> = { mpv: "mpv", builtin: "Built-in (fallback)" };
  const QUALITY_NAMES: Record<string, string> = { "2160": "4K", "1440": "1440p", "1080": "1080p", "720": "720p", "480": "480p", other: "Other" };

  let days = $state(14);
  let data = $state<Data | null>(null);
  let loading = $state(true);

  async function load() {
    loading = true;
    try {
      data = await call<Data>("analytics", { days });
    } catch (e) {
      sayError(e);
    } finally {
      loading = false;
    }
  }
  onMount(load);

  const short = (d: string) => new Date(`${d}T12:00:00Z`).toLocaleDateString(undefined, { month: "short", day: "numeric" });
  const series = (rows: { day: string }[], f: (r: never) => number) => rows.map((r) => ({ label: short(r.day), value: f(r as never) }));
  const sum = (rows: { day: string }[] | undefined, f: (r: never) => number) => (rows ?? []).reduce((a, r) => a + f(r as never), 0);
  const lastWeek = $derived(data ? Math.round(sum(data.installs.slice(-7), (r: { count: number }) => r.count) / 7) : 0);
</script>

<div class="adm-page">
  <div class="adm-head">
    <div class="grow">
      <h1>Analytics</h1>
      <p>Anonymous counts the apps send once a day, next to account growth. Nothing here says who.</p>
    </div>
    <m3e-filter-chip-set aria-label="Period">
      {#each [7, 14, 30] as d (d)}
        <m3e-filter-chip selected={days === d} onclick={() => { days = d; void load(); }}>{d} days</m3e-filter-chip>
      {/each}
    </m3e-filter-chip-set>
  </div>

  {#if !data}
    <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
  {:else}
    <div class="grid cols-4" class:busy={loading}>
      <div class="card stat"><span class="muted">Installs a day (7-day average)</span><strong>{count(lastWeek)}</strong></div>
      <div class="card stat"><span class="muted">Sign-ups, 30 days</span><strong>{count(sum(data.growth, (r: { signups: number }) => r.signups))}</strong></div>
      <div class="card stat"><span class="muted">Ratings, 30 days</span><strong>{count(sum(data.growth, (r: { ratings: number }) => r.ratings))}</strong></div>
      <div class="card stat"><span class="muted">App ads shown, 30 days</span><strong>{count(sum(data.ads, (r: { count: number }) => r.count))}</strong></div>
    </div>

    <div class="grid cols-2">
      <section class="card">
        <h2><m3e-icon name="devices_fold"></m3e-icon>Installs reporting each day</h2>
        <Columns points={series(data.installs, (r: { count: number }) => r.count)} unit="installs" />
      </section>
      <section class="card">
        <h2><m3e-icon name="person_add"></m3e-icon>Sign-ups each day</h2>
        <Columns points={series(data.growth, (r: { signups: number }) => r.signups)} unit="sign-ups" />
      </section>
      <section class="card">
        <h2><m3e-icon name="star"></m3e-icon>Ratings and reviews</h2>
        <Columns points={series(data.growth, (r: { ratings: number }) => r.ratings)} unit="ratings" />
        <p class="muted small">{count(sum(data.growth, (r: { reviews: number }) => r.reviews))} with a written review. Follows: {count(sum(data.growth, (r: { follows: number }) => r.follows))}. Feed activity: {count(sum(data.growth, (r: { activity: number }) => r.activity))}.</p>
      </section>
      <section class="card">
        <h2><m3e-icon name="ads_click"></m3e-icon>App ads shown</h2>
        <Columns points={series(data.ads, (r: { count: number }) => r.count)} unit="ads" />
      </section>
    </div>

    <section class="card">
      <h2><m3e-icon name="schedule"></m3e-icon>When people press play</h2>
      <Hours values={data.hours} />
    </section>

    <div class="grid cols-2">
      <section class="card">
        <h2><m3e-icon name="hub"></m3e-icon>Sources that played</h2>
        <Shares rows={data.sources} names={SOURCE_NAMES} />
      </section>
      <section class="card">
        <h2><m3e-icon name="history"></m3e-icon>App versions</h2>
        <Shares rows={data.versions} />
        <h2 class="sub"><m3e-icon name="play_circle"></m3e-icon>Player</h2>
        <Shares rows={data.engines} names={ENGINE_NAMES} />
        <h2 class="sub"><m3e-icon name="tv"></m3e-icon>Quality</h2>
        <Shares rows={data.quality} names={QUALITY_NAMES} />
      </section>
    </div>

    <div class="grid cols-2">
      <section class="card">
        <h2><m3e-icon name="local_fire_department"></m3e-icon>Most watched</h2>
        {#if !data.titles.length}
          <p class="muted small">Nothing has passed the privacy threshold yet.</p>
        {:else}
          <ol class="titles">
            {#each data.titles as t, i (t.key)}
              <li>
                <span class="rank">{i + 1}</span>
                {#if t.poster}<img src={t.poster} alt="" width="34" height="51" loading="lazy" />{:else}<span class="ph"></span>{/if}
                <span class="tname">{t.name}{t.year ? ` (${t.year})` : ""}<span class="muted mono">{t.key}</span></span>
                <span class="num">{count(t.count)}</span>
              </li>
            {/each}
          </ol>
        {/if}
      </section>
      <section class="card">
        <h2><m3e-icon name="music_note"></m3e-icon>Most played songs</h2>
        {#if !data.music.length}
          <p class="muted small">Nothing has passed the privacy threshold yet.</p>
        {:else}
          <ol class="titles">
            {#each data.music as m, i (m.key)}
              <li>
                <span class="rank">{i + 1}</span>
                <img src="https://i.ytimg.com/vi/{m.key}/default.jpg" alt="" width="51" height="38" loading="lazy" />
                <a class="tname" href="https://music.youtube.com/watch?v={m.key}" target="_blank" rel="noreferrer">{m.key}</a>
                <span class="num">{count(m.count)}</span>
              </li>
            {/each}
          </ol>
        {/if}
      </section>
    </div>

    <p class="muted small">{data.note}</p>
  {/if}
</div>

<style>
  .busy {
    opacity: 0.6;
  }
  .stat {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .stat span {
    font-size: 13px;
  }
  .stat strong {
    font-family: var(--display);
    font-weight: 400;
    font-size: 40px;
    line-height: 1.05;
  }
  .small {
    font-size: 13px;
    line-height: 1.5;
    margin: 10px 0 0;
  }
  .sub {
    margin-top: 18px !important;
  }
  .titles {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .titles li {
    display: grid;
    grid-template-columns: 22px auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 12px;
    font-size: 14px;
  }
  .rank {
    font-weight: 800;
    color: var(--md-sys-color-primary);
    text-align: right;
  }
  .titles img,
  .ph {
    border-radius: 6px;
    object-fit: cover;
    background: var(--md-sys-color-surface-container-highest);
  }
  .ph {
    width: 34px;
    height: 51px;
  }
  .tname {
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow: hidden;
    color: inherit;
  }
  .tname .mono {
    font-size: 11.5px;
  }
  .num {
    font-weight: 700;
  }
</style>
