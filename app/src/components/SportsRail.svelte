<script lang="ts">
  import MatchCard from "./MatchCard.svelte";
  import { sports } from "../lib/live.svelte";
  import { nav } from "../lib/state.svelte";

  // The Home rail: live now, coming up, and recent results in one place.
  type Tab = "live" | "upcoming" | "results";
  let picked = $state<Tab | null>(null);

  const tab = $derived<Tab>(picked ?? (sports.live.length ? "live" : "upcoming"));
  const list = $derived(tab === "live" ? sports.live : tab === "upcoming" ? sports.upcoming : sports.results);
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
</script>

<section class="rail">
  <header>
    <m3e-icon name="sports_soccer" class="icon"></m3e-icon>
    <m3e-heading variant="title" size="large" level={2}>Sports</m3e-heading>
    <m3e-button variant="text" size="small" class="see-all" onclick={() => nav.go({ name: "sports" })}>
      See all
      <m3e-icon slot="trailing-icon" name="chevron_right"></m3e-icon>
    </m3e-button>
  </header>

  <m3e-tabs class="tabs">
    {#each TABS as t (t.id)}
      <m3e-tab selected={tab === t.id} onclick={() => (picked = t.id)}>
        {#if t.id === "live" && sports.live.length}<span class="dot"></span>{/if}
        {t.label}{count(t.id) ? ` · ${count(t.id)}` : ""}
      </m3e-tab>
    {/each}
  </m3e-tabs>

  {#if !sports.loaded}
    <div class="row">
      {#each Array(4) as _, i (i)}
        <m3e-skeleton animation="wave" shape="rounded" class="ghost"></m3e-skeleton>
      {/each}
    </div>
  {:else if sports.failed}
    <div class="note">
      <span>Scores didn't load.</span>
      <m3e-button variant="text" size="small" onclick={() => sports.load(true)}>Try again</m3e-button>
    </div>
  {:else if list.length}
    <m3e-slide-group threshold="48">
      <div class="row">
        {#each list.slice(0, 24) as m (m.id)}
          <MatchCard match={m} width={280} />
        {/each}
      </div>
    </m3e-slide-group>
    <p class="fine">Scores can run a few minutes behind.</p>
  {:else}
    <p class="note">{EMPTY[tab]}</p>
  {/if}
</section>

<style>
  .rail {
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-width: 0;
  }
  header {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .icon {
    color: var(--md-sys-color-primary);
  }
  .see-all {
    margin-left: auto;
  }
  .tabs {
    --m3e-tabs-container-color: transparent;
    align-self: flex-start;
  }
  .dot {
    display: inline-block;
    width: 8px;
    height: 8px;
    margin-right: 6px;
    border-radius: 50%;
    background: #ff3b47;
    animation: pulse 1.6s infinite;
  }
  @keyframes pulse {
    to {
      box-shadow: 0 0 0 7px rgb(255 59 71 / 0%);
    }
  }
  m3e-slide-group {
    min-width: 0;
  }
  .row {
    display: flex;
    gap: 12px;
    padding: 2px 0 6px;
  }
  .ghost {
    flex: none;
    width: 280px;
    height: 132px;
  }
  .note {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0;
    font-size: 14px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .fine {
    margin: 0;
    font-size: 12px;
    color: var(--md-sys-color-on-surface-variant);
    opacity: 0.8;
  }
</style>
