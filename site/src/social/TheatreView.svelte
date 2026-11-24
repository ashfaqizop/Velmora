<script lang="ts">
  // Theatre on Velmora Social: today's and tomorrow's screenings, booking,
  // and your tickets. Watching opens the Velmora app.
  import { fly } from "svelte/transition";
  import TheatreCard from "./TheatreCard.svelte";
  import LiveChat from "./LiveChat.svelte";
  import { auth, loadMax } from "./data.svelte";
  import { beginSignIn } from "../lib/session";
  import { MAIN_SITE } from "../auth/config";
  import { dayLabel, theatre, type Screening } from "./theatre.svelte";

  let tab = $state<"lineup" | "tickets">("lineup");
  let message = $state("");
  let chatting = $state<Screening | null>(null);
  const openChat = (s: Screening) => (chatting = s);
  let msgTimer: ReturnType<typeof setTimeout> | undefined;

  $effect(() => {
    document.title = "Theatre · Velmora Social";
    theatre.start();
    theatre.load();
    if (auth.me) loadMax([auth.me.id]);
  });

  function say(m: string) {
    message = m;
    clearTimeout(msgTimer);
    msgTimer = setTimeout(() => (message = ""), 5000);
  }

  const featured = $derived(
    theatre.current.find((s) => ["live", "lobby"].includes(theatre.phase(s))) ?? theatre.current.find((s) => theatre.phase(s) === "upcoming") ?? null,
  );
  const days = $derived.by(() => {
    const out: { label: string; items: Screening[] }[] = [];
    for (const s of theatre.current) {
      if (s.id === featured?.id) continue;
      const label = dayLabel(s.starts_at, theatre.now);
      const g = out.find((x) => x.label === label);
      if (g) g.items.push(s);
      else out.push({ label, items: [s] });
    }
    return out;
  });
  const mine = $derived(theatre.current.filter((s) => ["booked", "waitlist"].includes(theatre.tickets[s.id] ?? "")));
</script>

<section class="head">
  <span class="eyebrow">Velmora Theatre</span>
  <h1 class="display">Watch together, <em>live.</em></h1>
  <p class="lead">Every day Velmora puts on movies everyone watches at the same moment, with a live chat that's never saved. Book here; watching happens in the Velmora app.</p>
</section>

<m3e-tabs class="tabs">
  <m3e-tab selected={tab === "lineup"} onclick={() => (tab = "lineup")}><m3e-icon slot="icon" name="theaters"></m3e-icon>Lineup</m3e-tab>
  <m3e-tab selected={tab === "tickets"} onclick={() => (tab = "tickets")}><m3e-icon slot="icon" name="confirmation_number"></m3e-icon>My tickets{mine.length ? ` · ${mine.length}` : ""}</m3e-tab>
</m3e-tabs>

{#if message}<p class="message" role="status" in:fly={{ y: -6, duration: 220 }}>{message}</p>{/if}

{#if !theatre.loaded}
  <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
{:else if theatre.failed}
  <p class="empty">The lineup didn't load. Check your connection and try again.</p>
{:else if tab === "tickets"}
  {#if !auth.me}
    <div class="empty">
      <p>Sign in to book screenings and see your tickets.</p>
      <m3e-button variant="filled" onclick={() => beginSignIn()}><m3e-icon slot="icon" name="login"></m3e-icon>Sign in</m3e-button>
    </div>
  {:else if !mine.length}
    <p class="empty">No tickets yet. Book a screening from the lineup.</p>
  {:else}
    <div class="list">{#each mine as s (s.id)}<TheatreCard {s} onmessage={say} onchat={openChat} />{/each}</div>
  {/if}
{:else if !featured}
  <p class="empty">Nothing is scheduled right now. Velmora puts together a new lineup every day.</p>
{:else}
  <div in:fly={{ y: 12, duration: 320 }}><TheatreCard s={featured} big onmessage={say} onchat={openChat} /></div>
  {#each days as d, i (d.label)}
    <section class="day" in:fly={{ y: 12, duration: 320, delay: 80 + i * 60 }}>
      <h2>{d.label}</h2>
      <div class="list">{#each d.items as s (s.id)}<TheatreCard {s} onmessage={say} onchat={openChat} />{/each}</div>
    </section>
  {/each}
  <p class="note">
    Times are in your time zone. Open screenings need no ticket; busy ones are ticketed and can sell out, with a waitlist.
    Don't have the app? <a href={MAIN_SITE}>Get Velmora</a>.
  </p>
{/if}

{#if chatting}<LiveChat s={chatting} onclose={() => (chatting = null)} />{/if}

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
    font-size: clamp(40px, 6vw, 68px);
    line-height: 1;
  }
  .display em {
    color: #f8e8c8;
  }
  .lead {
    margin: 0;
    max-width: 66ch;
    color: var(--md-sys-color-on-surface-variant);
    font-size: 16px;
    line-height: 1.6;
  }
  .tabs {
    --m3e-tabs-container-color: transparent;
  }
  .message {
    margin: 0;
    padding: 12px 16px;
    border-radius: 16px;
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
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
  .center {
    display: grid;
    place-items: center;
    padding: 40px 0;
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    margin: 0;
    padding: 40px 12px;
    text-align: center;
    color: var(--md-sys-color-on-surface-variant);
  }
  .empty p {
    margin: 0;
  }
  .note {
    margin: 0;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .note a {
    color: var(--md-sys-color-primary);
  }
</style>
