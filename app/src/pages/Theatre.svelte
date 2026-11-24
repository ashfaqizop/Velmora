<script lang="ts">
  // Theatre: movies everyone watches at the same moment. The lineup (picked
  // by Velmora every day), your tickets, and what's on now.
  import { fly } from "svelte/transition";
  import ScreeningCard from "../components/theatre/ScreeningCard.svelte";
  import { dayLabel } from "../components/theatre/time";
  import { account } from "../lib/account.svelte";
  import { theatre, type Screening } from "../lib/theatre.svelte";
  import { nav } from "../lib/state.svelte";

  let { tab = "lineup" }: { tab?: "lineup" | "tickets" } = $props();

  $effect(() => {
    theatre.load();
  });

  const featured = $derived(
    theatre.current.find((s) => ["live", "lobby"].includes(theatre.phase(s))) ?? theatre.current.find((s) => theatre.phase(s) === "upcoming") ?? null,
  );

  /** The rest, grouped by day in your time zone. */
  const days = $derived.by(() => {
    const out: { label: string; items: Screening[] }[] = [];
    for (const s of theatre.current) {
      if (s.id === featured?.id) continue;
      const label = dayLabel(s.starts_at, theatre.now);
      const group = out.find((g) => g.label === label);
      if (group) group.items.push(s);
      else out.push({ label, items: [s] });
    }
    return out;
  });

  const mine = $derived(theatre.lineup.filter((s) => theatre.holding(s)));

  const enter = (s: Screening) => nav.go({ name: "screening", id: s.id });

  const setTab = (t: "lineup" | "tickets") => nav.go({ name: "theatre", tab: t });
</script>

<div class="page">
  <header>
    <div>
      <m3e-heading variant="headline" size="large" level={1}>Theatre</m3e-heading>
      <p class="sub">Movies everyone on Velmora watches at the same moment. Each lobby and its chat open 30 minutes before; the chat is never saved.</p>
    </div>
  </header>

  <m3e-tabs class="tabs">
    <m3e-tab selected={tab === "lineup"} onclick={() => setTab("lineup")}>
      <m3e-icon slot="icon" name="theaters"></m3e-icon>Lineup
    </m3e-tab>
    <m3e-tab selected={tab === "tickets"} onclick={() => setTab("tickets")}>
      <m3e-icon slot="icon" name="confirmation_number"></m3e-icon>My tickets{mine.length ? ` · ${mine.length}` : ""}
    </m3e-tab>
  </m3e-tabs>

  {#if !theatre.loaded}
    <m3e-skeleton animation="wave" shape="rounded" class="ghost big"></m3e-skeleton>
    {#each Array(3) as _, i (i)}<m3e-skeleton animation="wave" shape="rounded" class="ghost"></m3e-skeleton>{/each}
  {:else if theatre.failed}
    <div class="empty">
      <m3e-icon name="cloud_off"></m3e-icon>
      <m3e-heading variant="title" size="large">The lineup didn't load</m3e-heading>
      <p>Theatre needs the internet. Check your connection.</p>
      <m3e-button variant="tonal" onclick={() => theatre.load()}>Try again</m3e-button>
    </div>
  {:else if tab === "tickets"}
    {#if !account.signedIn}
      <div class="empty"><p>Sign in to book screenings.</p></div>
    {:else if !mine.length}
      <div class="empty">
        <m3e-icon name="confirmation_number"></m3e-icon>
        <m3e-heading variant="title" size="large">No tickets yet</m3e-heading>
        <p>Book a screening from the lineup and it shows up here, with a reminder before it starts.</p>
        <m3e-button variant="tonal" onclick={() => setTab("lineup")}>See the lineup</m3e-button>
      </div>
    {:else}
      <div class="list">
        {#each mine as s, i (s.id)}
          <div in:fly={{ y: 10, duration: 260, delay: i * 40 }}><ScreeningCard {s} onenter={enter} /></div>
        {/each}
      </div>
    {/if}
  {:else if !featured}
    <div class="empty">
      <m3e-icon name="theaters"></m3e-icon>
      <m3e-heading variant="title" size="large">Nothing scheduled right now</m3e-heading>
      <p>Velmora puts together a new lineup every day. Check back soon.</p>
    </div>
  {:else}
    <section in:fly={{ y: 12, duration: 320 }}>
      <ScreeningCard s={featured} big onenter={enter} />
    </section>
    {#each days as d, gi (d.label)}
      <section class="day" in:fly={{ y: 12, duration: 320, delay: 80 + gi * 60 }}>
        <h2>{d.label}</h2>
        <div class="list">
          {#each d.items as s (s.id)}<ScreeningCard {s} onenter={enter} />{/each}
        </div>
      </section>
    {/each}
    <p class="note">Times are in your time zone. Velmora picks the lineup every day from what people watch, plus one hidden gem.</p>
  {/if}
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding-bottom: 32px;
  }
  .sub {
    margin: 4px 0 0;
    font-size: 14px;
    color: var(--md-sys-color-on-surface-variant);
    max-width: 70ch;
  }
  .tabs {
    --m3e-tabs-container-color: transparent;
  }
  .day {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .day h2 {
    margin: 0;
    font-size: 18px;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .note {
    margin: 0;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .ghost {
    width: 100%;
    height: 88px;
  }
  .ghost.big {
    height: 260px;
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 56px 16px;
    text-align: center;
    color: var(--md-sys-color-on-surface-variant);
    --m3e-icon-size: 40px;
  }
  .empty p {
    margin: 0;
    max-width: 48ch;
  }
</style>
