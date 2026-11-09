<script lang="ts">
  import { nav } from "../lib/state.svelte";
  import { hasScore, statusText, whenText } from "../lib/sports";
  import type { SportsMatch } from "../lib/types";

  // One match: league, status, both teams and the score.
  let { match, width }: { match: SportsMatch; width?: number } = $props();

  const isLive = $derived(match.status === "live");
  const score = $derived(hasScore(match));
  const lead = $derived.by(() => {
    if (!score) return null;
    const [h, a] = [match.home_score ?? 0, match.away_score ?? 0];
    return h === a ? null : h > a ? "home" : "away";
  });
</script>

{#snippet team(name: string, badge: string | undefined, goals: number | undefined, leading: boolean)}
  <div class="team" class:leading>
    <span class="badge">
      <span class="initial">{name.trim().charAt(0).toUpperCase()}</span>
      {#if badge}
        <img src={badge} alt="" loading="lazy" onerror={(e) => ((e.currentTarget as HTMLImageElement).style.display = "none")} />
      {/if}
    </span>
    <span class="name">{name}</span>
    {#if score}<b class="goals">{goals ?? 0}</b>{/if}
  </div>
{/snippet}

<button class="match" style:width={width ? `${width}px` : undefined} onclick={() => nav.go({ name: "match", match })}>
  <header>
    <span class="league">
      {#if match.league.badge}<img src={match.league.badge} alt="" loading="lazy" />{/if}
      <span>{match.league.name || match.sport}</span>
    </span>
    <span class="status" class:live={isLive}>
      {#if isLive}<i></i>{/if}{statusText(match)}
    </span>
  </header>

  {#if match.home && match.away}
    <div class="teams">
      {@render team(match.home.name, match.home.badge, match.home_score, lead === "home")}
      {@render team(match.away.name, match.away.badge, match.away_score, lead === "away")}
    </div>
  {:else}
    <div class="event">{match.title}</div>
  {/if}

  <footer>{match.status === "upcoming" || match.status === "postponed" ? whenText(match) : (match.round ?? match.sport)}</footer>
</button>

<style>
  .match {
    all: unset;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 14px 16px;
    min-width: 0;
    border-radius: 22px;
    cursor: pointer;
    background: var(--md-sys-color-surface-container);
    transition:
      background 0.2s,
      transform 0.25s cubic-bezier(0.2, 0.7, 0.2, 1);
  }
  .match:hover,
  .match:focus-visible {
    background: var(--md-sys-color-surface-container-high);
    transform: translateY(-2px);
  }
  .match:focus-visible {
    outline: 2px solid var(--md-sys-color-primary);
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    font-size: 12px;
  }
  .league {
    min-width: 0;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .league span {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .league img {
    width: 16px;
    height: 16px;
    object-fit: contain;
  }
  .status {
    flex: none;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 2px 9px;
    border-radius: 999px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    background: var(--md-sys-color-surface-container-highest);
  }
  .status.live {
    color: #ff8a92;
    background: rgb(255 59 71 / 18%);
    letter-spacing: 0.05em;
  }
  .status i {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: #ff3b47;
    box-shadow: 0 0 0 0 rgb(255 59 71 / 60%);
    animation: pulse 1.6s infinite;
  }
  @keyframes pulse {
    to {
      box-shadow: 0 0 0 7px rgb(255 59 71 / 0%);
    }
  }
  .teams {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .team {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
    color: var(--md-sys-color-on-surface-variant);
  }
  .team.leading {
    color: var(--md-sys-color-on-surface);
  }
  .badge {
    position: relative;
    flex: none;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    overflow: hidden;
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
    font-size: 13px;
    font-weight: 700;
  }
  .badge img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: contain;
    padding: 2px;
    background: var(--md-sys-color-surface-container);
  }
  .name {
    flex: 1;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-weight: 600;
  }
  .goals {
    font-size: 18px;
    font-variant-numeric: tabular-nums;
  }
  .event {
    font-family: var(--app-display-font);
    font-size: 22px;
    line-height: 1.15;
  }
  footer {
    font-size: 12px;
    color: var(--md-sys-color-on-surface-variant);
  }
</style>
