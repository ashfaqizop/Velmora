<script lang="ts">
  // Velmora by the numbers. Everything here is a total from the app's daily
  // anonymous report (see /privacy/): no IDs, no accounts, no titles by person.
  import { onMount } from "svelte";
  import Backdrop from "./components/Backdrop.svelte";
  import Nav from "./components/Nav.svelte";
  import Footer from "./components/Footer.svelte";

  interface Row {
    key: string;
    count: number;
  }
  interface Summary {
    days: number;
    sources: Row[];
    engines: Row[];
    quality: Row[];
    versions: Row[];
    daily: { day: string; count: number }[];
  }

  let data = $state<Summary | null>(null);
  let failed = $state(false);

  onMount(async () => {
    try {
      const r = await fetch("/api/stats/summary?days=14");
      if (!r.ok) throw new Error(String(r.status));
      data = await r.json();
    } catch {
      failed = true;
    }
  });

  const NAMES: Record<string, string> = {
    hdhub: "HDHub",
    four_k_hd_hub: "4KHDHub",
    movie_box: "MovieBox",
    french_stream: "FrenchStream",
    zoko_anime: "ZokoAnime",
    hi_anime: "HiAnime",
    anime_sama: "AnimeSama",
    yts: "YTS",
    eztv: "EZTV",
    nyaa: "Nyaa",
    coflix: "Coflix",
    dramachi: "Dramachi",
  };
  const name = (id: string) => NAMES[id] ?? id.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
  const ENGINE: Record<string, string> = { mpv: "mpv", builtin: "Built-in player" };
  const QUALITY: Record<string, string> = { "480": "480p", "720": "720p", "1080": "1080p", "1440": "1440p", "2160": "4K", other: "Other" };

  const total = (rows: Row[]) => rows.reduce((n, r) => n + r.count, 0) || 1;
  const pct = (rows: Row[], r: Row) => Math.round((r.count / total(rows)) * 100);
  const peak = $derived(Math.max(1, ...(data?.daily.map((d) => d.count) ?? [1])));
  const reported = $derived(data?.daily.reduce((n, d) => n + d.count, 0) ?? 0);
  const weekday = (day: string) => new Date(`${day}T12:00:00Z`).toLocaleDateString(undefined, { weekday: "short" });
</script>

<Backdrop />
<Nav />
<main id="top">
  <article class="wrap narrow">
    <span class="eyebrow">Anonymous totals</span>
    <h1 class="title">Velmora by the numbers</h1>
    <p class="lead">
      Once a day each Velmora install sends plain totals: which sites it played from, which player ran and the video quality. No IDs, no accounts. Here is what
      adds up. You can turn this off in the app, under Settings, Privacy; <a href="/privacy/">the privacy policy</a> has the details.
    </p>

    {#if failed}
      <p class="note">The numbers couldn't be loaded right now. Try again in a minute.</p>
    {:else if !data}
      <p class="note">Loading…</p>
    {:else if !reported}
      <p class="note">Nothing has been reported yet. Check back soon.</p>
    {:else}
      <section>
        <h2>Installs reporting in</h2>
        <p class="sub">One bar per day. Each install counts once a day.</p>
        <div class="days" role="img" aria-label="Installs reporting each day">
          {#each data.daily as d (d.day)}
            <div class="day" title="{d.day}: {d.count}">
              <span class="n">{d.count}</span>
              <span class="bar" style:height="{Math.max(4, (d.count / peak) * 100)}%"></span>
              <span class="w">{weekday(d.day)}</span>
            </div>
          {/each}
        </div>
      </section>

      {#snippet board(rows: Row[], label: (k: string) => string)}
        <ol class="board">
          {#each rows as r (r.key)}
            <li>
              <span class="k">{label(r.key)}</span>
              <span class="meter"><i style:width="{Math.max(2, pct(rows, r))}%"></i></span>
              <span class="v">{pct(rows, r)}%</span>
            </li>
          {/each}
        </ol>
      {/snippet}

      <section>
        <h2>Most used sites</h2>
        <p class="sub">Share of plays over the last {data.days} days.</p>
        {@render board(data.sources, name)}
      </section>

      <section class="two">
        <div>
          <h2>Which player</h2>
          <p class="sub">mpv, or the built-in one that takes over when mpv can't start.</p>
          {@render board(data.engines, (k) => ENGINE[k] ?? k)}
        </div>
        <div>
          <h2>Video quality</h2>
          <p class="sub">Of what actually played.</p>
          {@render board(data.quality, (k) => QUALITY[k] ?? k)}
        </div>
      </section>

      <section>
        <h2>Versions in use</h2>
        <p class="sub">Installs reporting each version.</p>
        {@render board(data.versions, (k) => `Velmora ${k}`)}
      </section>
    {/if}
  </article>
</main>
<Footer />

<style>
  article {
    padding-top: 140px;
    padding-bottom: 60px;
  }
  .narrow {
    width: min(820px, 100% - 48px);
  }
  h1 {
    margin-top: 12px;
  }
  h2 {
    margin: 0 0 4px;
    font-size: 22px;
  }
  section {
    padding-top: 44px;
  }
  .lead,
  .sub,
  .note {
    margin: 0;
    line-height: 1.7;
    color: var(--muted);
  }
  .lead {
    margin-top: 20px;
  }
  .lead a {
    color: #c9c1ff;
  }
  .sub {
    margin-bottom: 18px;
    font-size: 14px;
  }
  .note {
    margin-top: 40px;
  }
  .days {
    display: grid;
    grid-template-columns: repeat(14, minmax(0, 1fr));
    gap: 6px;
    align-items: end;
    height: 190px;
  }
  .day {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: flex-end;
    gap: 4px;
    height: 100%;
    min-width: 0;
  }
  .day .bar {
    width: 100%;
    border-radius: 8px 8px 3px 3px;
    background: linear-gradient(180deg, var(--violet), var(--blue));
    transition: height 600ms cubic-bezier(0.2, 0.8, 0.2, 1);
  }
  .day .n,
  .day .w {
    font-size: 11px;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }
  .board {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .board li {
    display: grid;
    grid-template-columns: minmax(90px, 160px) 1fr 44px;
    align-items: center;
    gap: 12px;
    font-size: 15px;
  }
  .k {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .meter {
    height: 10px;
    border-radius: 999px;
    background: var(--glass);
    overflow: hidden;
  }
  .meter i {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: linear-gradient(90deg, var(--blue), var(--violet));
  }
  .v {
    text-align: right;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }
  .two {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 36px;
  }
  @media (max-width: 600px) {
    article {
      padding-top: 110px;
    }
    .narrow {
      width: calc(100% - 32px);
    }
    .day .n {
      display: none;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .day .bar {
      transition: none;
    }
  }
</style>
