<script lang="ts">
  import { onMount } from "svelte";
  import MatchCard from "../components/MatchCard.svelte";
  import AdSlot from "../components/AdSlot.svelte";
  import { sports } from "../lib/live.svelte";
  import { groupByLeague } from "../lib/sports";

  // Every match, grouped by league, with the same three tabs as the Home rail.
  type Tab = "live" | "upcoming" | "results";
  let picked = $state<Tab | null>(null);

  const tab = $derived<Tab>(picked ?? (sports.live.length ? "live" : "upcoming"));
  const list = $derived(tab === "live" ? sports.live : tab === "upcoming" ? sports.upcoming : sports.results);
  const groups = $derived(groupByLeague(list));
  const TABS: { id: Tab; label: string }[] = [
    { id: "live", label: "Live" },
    { id: "upcoming", label: "Upcoming" },
    { id: "results", label: "Results" },
  ];
  const count = (id: Tab) => (id === "live" ? sports.live.length : id === "upcoming" ? sports.upcoming.length : sports.results.length);
  const EMPTY: Record<Tab, string> = {
    live: "Nothing is in play right now.",
    upcoming: "No matches coming up in the next two days.",
    results: "No recent results.",
  };

  onMount(() => {
    sports.load();
  });
</script>

<div class="page">
  <header>
    <div>
      <m3e-heading variant="headline" size="large" level={1}>Sports</m3e-heading>
      <p class="sub">Live scores, fixtures and results. Free sources run a few minutes behind.</p>
    </div>
    <m3e-icon-button aria-label="Refresh" disabled={sports.loading} onclick={() => sports.load(true)}>
      <m3e-icon name="refresh"></m3e-icon>
    </m3e-icon-button>
  </header>

  <m3e-tabs class="tabs">
    {#each TABS as t (t.id)}
      <m3e-tab selected={tab === t.id} onclick={() => (picked = t.id)}>{t.label}{count(t.id) ? ` · ${count(t.id)}` : ""}</m3e-tab>
    {/each}
  </m3e-tabs>

  {#if !sports.loaded}
    <div class="grid">
      {#each Array(8) as _, i (i)}
        <m3e-skeleton animation="wave" shape="rounded" class="ghost"></m3e-skeleton>
      {/each}
    </div>
  {:else if sports.failed}
    <div class="empty">
      <m3e-icon name="cloud_off"></m3e-icon>
      <m3e-heading variant="title" size="large">Scores didn't load</m3e-heading>
      <p>Check your connection, then try again.</p>
      <m3e-button variant="tonal" onclick={() => sports.load(true)}>
        <m3e-icon slot="icon" name="refresh"></m3e-icon>Try again
      </m3e-button>
    </div>
  {:else if groups.length}
    {#each groups as g, i (g.id)}
      <section>
        <h2>
          {#if g.badge}<img src={g.badge} alt="" loading="lazy" />{/if}
          <span>{g.name}</span>
          <small>{g.matches.length}</small>
        </h2>
        <div class="grid">
          {#each g.matches as m (m.id)}
            <MatchCard match={m} />
          {/each}
        </div>
      </section>
      <!-- Between leagues, never on a match page near "where to watch". -->
      {#if i === 1 && groups.length > 2}<AdSlot id="sports" />{/if}
    {/each}
  {:else}
    <div class="empty">
      <m3e-icon name="sports_score"></m3e-icon>
      <p>{EMPTY[tab]}</p>
    </div>
  {/if}
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 18px;
    padding-bottom: 32px;
  }
  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
  }
  .sub {
    margin: 4px 0 0;
    font-size: 14px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .tabs {
    --m3e-tabs-container-color: transparent;
    align-self: flex-start;
  }
  section {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  h2 {
    display: flex;
    align-items: center;
    gap: 10px;
    margin: 6px 0 0;
    font-family: var(--app-display-font);
    font-weight: 400;
    font-size: 26px;
  }
  h2 img {
    width: 26px;
    height: 26px;
    object-fit: contain;
  }
  h2 small {
    font-family: var(--app-font);
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 12px;
  }
  .ghost {
    height: 132px;
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 56px 0;
    text-align: center;
    color: var(--md-sys-color-on-surface-variant);
    --m3e-icon-size: 48px;
  }
  .empty p {
    margin: 0;
  }
</style>
