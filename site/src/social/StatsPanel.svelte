<script lang="ts">
  // A profile's stats, from what they share: time watched, titles, ratings and top genres.
  import { profileStats, type Stats } from "./activity.svelte";
  import { titles } from "./meta.svelte";
  import { count } from "./format";

  let { id, mine, name }: { id: string; mine: boolean; name: string } = $props();

  let stats = $state<Stats | null | undefined>(undefined);
  let problem = $state("");

  $effect(() => {
    const who = id;
    stats = undefined;
    profileStats(who)
      .then((s) => (stats = s))
      .catch((e) => {
        problem = (e as Error).message;
        stats = null;
      });
  });

  // Genres weighted by how high each title sits in their most-watched list.
  const genres = $derived.by(() => {
    if (!stats) return [];
    const score = new Map<string, number>();
    stats.top.forEach((k, i) => {
      for (const g of titles.get(k)?.genres ?? []) score.set(g, (score.get(g) ?? 0) + (stats!.top.length - i));
    });
    const total = [...score.values()].reduce((a, b) => a + b, 0) || 1;
    return [...score.entries()]
      .sort((a, b) => b[1] - a[1])
      .slice(0, 6)
      .map(([g, n]) => ({ g, pct: Math.round((n / total) * 100) }));
  });

  const hours = (m: number) => (m >= 600 ? count(Math.round(m / 60)) : (Math.round(m / 6) / 10).toString());
</script>

{#if stats === undefined}
  <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
{:else if stats === null}
  <p class="empty">{problem || `${name} keeps what they watch private.`}</p>
{:else}
  <div class="tiles">
    <div class="tile big"><strong>{hours(stats.minutes)}</strong><span>hours watched</span></div>
    <div class="tile"><strong>{hours(stats.minutes_month)}</strong><span>hours this month</span></div>
    <div class="tile"><strong>{count(stats.movies)}</strong><span>movies</span></div>
    <div class="tile"><strong>{count(stats.shows)}</strong><span>shows</span></div>
    <div class="tile"><strong>{count(stats.episodes)}</strong><span>episodes</span></div>
    <div class="tile"><strong>{count(stats.ratings)}</strong><span>ratings{stats.average != null ? ` · avg ★ ${(stats.average / 2).toFixed(1)}` : ""}</span></div>
    <div class="tile"><strong>{count(stats.reviews)}</strong><span>reviews</span></div>
  </div>
  {#if genres.length}
    <div class="genres">
      <h3>Top genres</h3>
      {#each genres as g, i (g.g)}
        <div class="bar" style:--w="{Math.max(g.pct, 4)}%" style:--i={i}>
          <span class="label">{g.g}</span><span class="pct">{g.pct}%</span>
        </div>
      {/each}
    </div>
  {/if}
  {#if mine}<p class="hint">Stats come from what Velmora shares from your app. Turn sharing on or off in Edit profile or in the app's Privacy settings.</p>{/if}
{/if}

<style>
  .tiles {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
    gap: 10px;
  }
  .tile {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 16px 18px;
    border-radius: 22px;
    background: var(--md-sys-color-surface-container-high);
  }
  .tile.big {
    grid-column: span 2;
    background: linear-gradient(135deg, var(--md-sys-color-primary-container), var(--md-sys-color-tertiary-container));
    color: var(--md-sys-color-on-primary-container);
  }
  .tile strong {
    font-size: 28px;
    font-variant-numeric: tabular-nums;
  }
  .tile.big strong {
    font-size: 44px;
    font-family: var(--display);
    font-weight: 400;
  }
  .tile span {
    font-size: 13px;
    opacity: 0.8;
  }
  .genres {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 20px;
  }
  .genres h3 {
    margin: 0 0 4px;
    font-size: 16px;
  }
  .bar {
    position: relative;
    display: flex;
    justify-content: space-between;
    padding: 10px 14px;
    border-radius: 14px;
    overflow: hidden;
    background: var(--md-sys-color-surface-container-high);
    font-size: 14px;
  }
  .bar::before {
    content: "";
    position: absolute;
    inset: 0 auto 0 0;
    width: var(--w);
    background: color-mix(in srgb, var(--md-sys-color-primary) 30%, transparent);
    border-radius: 14px;
    animation: grow 700ms cubic-bezier(0.2, 0.8, 0.2, 1) both;
    animation-delay: calc(var(--i) * 70ms);
  }
  .label,
  .pct {
    position: relative;
  }
  .pct {
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .hint,
  .empty {
    margin: 16px 0 0;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .empty {
    padding: 24px 12px;
    text-align: center;
    font-size: 15px;
  }
  .center {
    display: grid;
    place-items: center;
    padding: 32px 0;
  }
  @keyframes grow {
    from {
      width: 0;
    }
  }
  @media (max-width: 400px) {
    .tile.big {
      grid-column: auto;
    }
  }
</style>
