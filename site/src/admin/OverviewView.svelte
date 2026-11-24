<script lang="ts">
  // The first page: how full the free tiers are, whether things are healthy,
  // and how big Velmora Social has grown.
  import { onMount } from "svelte";
  import { call, sayError } from "./api";
  import { ago, bytes, count } from "./format";
  import { go } from "./route.svelte";

  interface Meter { id: string; label: string; used: number; limit: number; unit: "bytes" | "count"; note?: string }
  interface Run { name: string; branch: string; status: string; conclusion: string | null; at: string; url: string }
  interface Data {
    stats: Record<string, number>;
    meters: Meter[];
    redis: { ok: boolean; keys?: number; message?: string };
    ci: { ok: boolean; message?: string; runs: Run[]; release: { tag: string; at: string; url: string } | null };
    fallback: { mpv: number; builtin: number; rate: number | null };
    versions: { key: string; count: number }[];
    tonight: { ok: boolean; rows: { id: string; name: string; starts_at: string; status: string }[] };
  }

  let data = $state<Data | null>(null);
  let loading = $state(true);

  async function load() {
    loading = true;
    try {
      data = (await call<Data>("overview")) as Data;
    } catch (e) {
      sayError(e);
    } finally {
      loading = false;
    }
  }
  onMount(load);

  const show = (m: Meter) => (m.unit === "bytes" ? `${bytes(m.used)} of ${bytes(m.limit)}` : `${count(m.used)} of ${count(m.limit)}`);
  const level = (m: Meter) => {
    const f = m.used / m.limit;
    return f >= 0.9 ? "bad" : f >= 0.7 ? "warn" : "ok";
  };
  const runState = (r: Run) => (r.status !== "completed" ? "info" : r.conclusion === "success" ? "ok" : r.conclusion === "cancelled" ? "warn" : "bad");
  const runLabel = (r: Run) => (r.status !== "completed" ? "running" : (r.conclusion ?? "done"));
  const time = (iso: string) => new Date(iso).toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" });
</script>

<div class="adm-page">
  <div class="adm-head">
    <div class="grow">
      <h1>Overview</h1>
      <p>How full the free tiers are, whether the build is green, and how Velmora Social is growing.</p>
    </div>
    <m3e-button variant="tonal" disabled={loading} onclick={load}><m3e-icon slot="icon" name="refresh"></m3e-icon>Refresh</m3e-button>
  </div>

  {#if !data}
    <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
  {:else}
    {@const s = data.stats}
    <div class="grid cols-4">
      <div class="card tile"><span class="n">{count(s.users)}</span><span class="l">Accounts</span><span class="muted sub">{count(s.signups_7d)} joined this week</span></div>
      <div class="card tile"><span class="n">{count(s.active_30d)}</span><span class="l">Active, 30 days</span><span class="muted sub">{count(s.signups_today)} joined today</span></div>
      <button class="card tile click" class:hot={s.reports_open > 0} onclick={() => go("reports")}>
        <span class="n">{count(s.reports_open)}</span><span class="l">Open reports</span><span class="muted sub">{s.reports_open ? "Waiting for you" : "All clear"}</span>
      </button>
      <button class="card tile click" onclick={() => go("users")}>
        <span class="n">{count(s.hidden + s.banned)}</span><span class="l">Hidden or banned</span><span class="muted sub">{count(s.verified)} verified</span>
      </button>
    </div>

    <div class="grid cols-2">
      <section class="card">
        <h2><m3e-icon name="storage"></m3e-icon>Free-tier meters</h2>
        <div class="meters">
          {#each data.meters as m (m.id)}
            <div class="meter">
              <div class="top"><span>{m.label}</span><span class="muted">{show(m)}</span></div>
              <div class="bar" role="meter" aria-valuenow={m.used} aria-valuemin={0} aria-valuemax={m.limit} aria-label={m.label}>
                <span class="fill {level(m)}" style:width="{Math.max(1.5, Math.min(100, (m.used / m.limit) * 100))}%"></span>
              </div>
              {#if m.note}<span class="muted note">{m.note}</span>{/if}
            </div>
          {/each}
          {#if data.redis.ok}<span class="muted note">Upstash holds {count(data.redis.keys ?? 0)} keys.</span>{:else}<span class="muted note">Upstash didn't answer: {data.redis.message}</span>{/if}
        </div>
      </section>

      <section class="card">
        <h2><m3e-icon name="monitoring"></m3e-icon>Build and releases</h2>
        {#if !data.ci.ok}
          <p class="muted">GitHub didn't answer ({data.ci.message}). Try again in a few minutes.</p>
        {:else}
          {#if data.ci.release}
            <p class="release"><span class="chip info">Latest release</span> <a href={data.ci.release.url} target="_blank" rel="noopener">{data.ci.release.tag}</a> <span class="muted">{ago(data.ci.release.at)}</span></p>
          {/if}
          <ul class="runs">
            {#each data.ci.runs as r (r.url)}
              <li>
                <span class="chip {runState(r)}">{runLabel(r)}</span>
                <a href={r.url} target="_blank" rel="noopener">{r.name}</a>
                <span class="muted">{r.branch} · {ago(r.at)}</span>
              </li>
            {/each}
          </ul>
        {/if}
      </section>
    </div>

    <div class="grid cols-2">
      <section class="card">
        <h2><m3e-icon name="health_and_safety"></m3e-icon>Playback health (7 days)</h2>
        {#if data.fallback.rate === null}
          <p class="muted">No playback numbers yet.</p>
        {:else}
          <p class="big">{(data.fallback.rate * 100).toFixed(1)}%<span class="muted"> of plays used the built-in player</span></p>
          <p class="muted">That's the fallback when mpv can't start. {count(data.fallback.mpv)} plays on mpv, {count(data.fallback.builtin)} on the built-in player.</p>
        {/if}
        {#if data.versions.length}
          <div class="vers">
            {#each data.versions as v (v.key)}<span class="chip">{v.key} · {count(v.count)}</span>{/each}
          </div>
        {/if}
      </section>

      <section class="card">
        <h2><m3e-icon name="theaters"></m3e-icon>Theatre today</h2>
        {#if !data.tonight.ok}
          <p class="muted">Couldn't read the lineup.</p>
        {:else if !data.tonight.rows.length}
          <p class="muted">Nothing is scheduled today.</p>
        {:else}
          <ul class="runs">
            {#each data.tonight.rows as t (t.id)}
              <li><span class="mono muted">{time(t.starts_at)}</span> <span>{t.name}</span> <span class="chip">{t.status}</span></li>
            {/each}
          </ul>
        {/if}
      </section>
    </div>

    <section class="card">
      <h2><m3e-icon name="insights"></m3e-icon>Velmora Social</h2>
      <div class="grid cols-4 mini">
        <div><span class="n sm">{count(s.profiles)}</span><span class="l">Profiles</span></div>
        <div><span class="n sm">{count(s.follows)}</span><span class="l">Follows</span></div>
        <div><span class="n sm">{count(s.ratings)}</span><span class="l">Ratings ({count(s.reviews)} with a review)</span></div>
        <div><span class="n sm">{count(s.lists)}</span><span class="l">Lists</span></div>
      </div>
    </section>
  {/if}
</div>

<style>
  .tile {
    display: flex;
    flex-direction: column;
    gap: 2px;
    text-align: left;
    color: inherit;
    font: inherit;
  }
  .tile.click {
    cursor: pointer;
    transition:
      background-color 160ms ease,
      transform 160ms ease;
  }
  .tile.click:hover {
    background: var(--md-sys-color-surface-container-high);
    transform: translateY(-2px);
  }
  .tile.hot {
    border-color: color-mix(in srgb, var(--amber) 60%, transparent);
  }
  .n {
    font-family: var(--display);
    font-size: 44px;
    line-height: 1;
  }
  .n.sm {
    display: block;
    font-size: 32px;
  }
  .l {
    font-size: 13px;
    font-weight: 700;
  }
  .sub {
    font-size: 12.5px;
  }
  .meters {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .meter {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .top {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    font-size: 14px;
    font-weight: 600;
  }
  .top .muted {
    font-weight: 500;
    font-size: 13px;
  }
  .bar {
    height: 10px;
    border-radius: 999px;
    background: var(--md-sys-color-surface-container-highest);
    overflow: hidden;
  }
  .fill {
    display: block;
    height: 100%;
    border-radius: 999px;
    transition: width 500ms cubic-bezier(0.2, 0, 0, 1);
  }
  .fill.ok {
    background: #4fd18b;
  }
  .fill.warn {
    background: var(--amber);
  }
  .fill.bad {
    background: #ff5449;
  }
  .note {
    font-size: 12.5px;
  }
  .runs {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
    font-size: 14px;
  }
  .runs li {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }
  .runs a,
  .release a {
    font-weight: 700;
    text-decoration: none;
  }
  .runs a:hover,
  .release a:hover {
    text-decoration: underline;
  }
  .release {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0 0 14px;
    flex-wrap: wrap;
  }
  .big {
    margin: 0 0 6px;
    font-family: var(--display);
    font-size: 40px;
    line-height: 1.1;
  }
  .big .muted {
    font-family: var(--font);
    font-size: 14px;
  }
  p {
    margin: 0 0 8px;
    font-size: 14px;
  }
  .vers {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    margin-top: 12px;
  }
  .mini > div {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
</style>
