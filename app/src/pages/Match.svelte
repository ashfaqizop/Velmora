<script lang="ts">
  import { onMount } from "svelte";
  import * as api from "../lib/api";
  import WatchList from "../components/WatchList.svelte";
  import { hasScore, statusText, whenText } from "../lib/sports";
  import type { SportsMatch } from "../lib/types";

  let { match: initial }: { match: SportsMatch } = $props();

  // Start from the card's data, then refresh it from the source.
  let fresh = $state<SportsMatch | null>(null);
  const m = $derived(fresh ?? initial);
  const score = $derived(hasScore(m));
  const isLive = $derived(m.status === "live");

  onMount(() => {
    api
      .sportsLookup(initial.id)
      .then((x) => (fresh = x))
      .catch(() => {});
  });
</script>

{#snippet side(name: string, badge: string | undefined, goals: number | undefined)}
  <div class="side">
    <span class="badge">
      <span class="initial">{name.trim().charAt(0).toUpperCase()}</span>
      {#if badge}<img src={badge} alt="" onerror={(e) => ((e.currentTarget as HTMLImageElement).style.display = "none")} />{/if}
    </span>
    <strong>{name}</strong>
    {#if score}<span class="goals">{goals ?? 0}</span>{/if}
  </div>
{/snippet}

<div class="page">
  <section class="score">
    <div class="league">
      {#if m.league.badge}<img src={m.league.badge} alt="" />{/if}
      <span>{m.league.name || m.sport}</span>
      {#if m.round}<span class="round">· {m.round}</span>{/if}
    </div>

    {#if m.home && m.away}
      <div class="board">
        {@render side(m.home.name, m.home.badge, m.home_score)}
        <div class="mid">
          <span class="status" class:live={isLive}>{#if isLive}<i></i>{/if}{statusText(m)}</span>
          {#if m.status === "upcoming"}<span class="when">{whenText(m)}</span>{/if}
        </div>
        {@render side(m.away.name, m.away.badge, m.away_score)}
      </div>
    {:else}
      <m3e-heading variant="headline" size="large" level={1} class="event">{m.title}</m3e-heading>
      <div class="mid"><span class="status" class:live={isLive}>{statusText(m)}</span><span class="when">{whenText(m)}</span></div>
    {/if}

    <div class="meta">
      {#if m.venue}<span><m3e-icon name="location_on"></m3e-icon>{m.venue}</span>{/if}
      {#if m.status !== "upcoming" && m.kickoff}<span><m3e-icon name="schedule"></m3e-icon>{whenText(m)}</span>{/if}
      <span><m3e-icon name="sports"></m3e-icon>{m.sport}</span>
    </div>
    <p class="fine">Scores can run a few minutes behind.</p>
  </section>

  <section class="watch">
    <WatchList match={initial} />
  </section>
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 22px;
    padding-bottom: 32px;
    max-width: 880px;
  }
  .score {
    display: flex;
    flex-direction: column;
    gap: 18px;
    padding: 26px;
    border-radius: 30px;
    background:
      radial-gradient(80% 120% at 0% 0%, color-mix(in srgb, var(--md-sys-color-primary) 22%, transparent), transparent 60%),
      var(--md-sys-color-surface-container);
  }
  .league {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .league img {
    width: 22px;
    height: 22px;
    object-fit: contain;
  }
  .board {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
    align-items: center;
    gap: 12px;
  }
  .side {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    text-align: center;
  }
  .side strong {
    font-family: var(--app-display-font);
    font-weight: 400;
    font-size: 26px;
    line-height: 1.1;
  }
  .badge {
    position: relative;
    display: grid;
    place-items: center;
    overflow: hidden;
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
  }
  .badge {
    width: 84px;
    height: 84px;
    border-radius: 50%;
    font-size: 36px;
    font-weight: 700;
  }
  .badge img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: contain;
    padding: 8px;
    background: var(--md-sys-color-surface-container);
  }
  .goals {
    font-family: var(--app-display-font);
    font-size: 64px;
    line-height: 1;
    font-variant-numeric: tabular-nums;
  }
  .mid {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
  }
  .status {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 4px 12px;
    border-radius: 999px;
    font-size: 13px;
    font-weight: 700;
    background: var(--md-sys-color-surface-container-highest);
  }
  .status.live {
    color: #ff8a92;
    background: rgb(255 59 71 / 18%);
  }
  .status i {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #ff3b47;
    box-shadow: 0 0 0 0 rgb(255 59 71 / 60%);
    animation: pulse 1.6s infinite;
  }
  @keyframes pulse {
    to {
      box-shadow: 0 0 0 8px rgb(255 59 71 / 0%);
    }
  }
  .when {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .meta {
    display: flex;
    flex-wrap: wrap;
    gap: 8px 20px;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .meta span {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    --m3e-icon-size: 18px;
  }
  .fine {
    margin: 0;
    font-size: 12px;
    color: var(--md-sys-color-on-surface-variant);
    opacity: 0.8;
  }
  @media (max-width: 700px) {
    .badge {
      width: 60px;
      height: 60px;
      font-size: 26px;
    }
    .side strong {
      font-size: 20px;
    }
    .goals {
      font-size: 44px;
    }
  }
</style>
