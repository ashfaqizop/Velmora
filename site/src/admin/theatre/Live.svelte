<script lang="ts">
  // What is on right now or about to be: lobby, live and after-chat. Numbers
  // come from tickets and who came in, refreshed while the page is open.
  import { onMount } from "svelte";
  import { call, sayError } from "../api";
  import { count } from "../format";
  import RoomPanel from "./RoomPanel.svelte";
  import { clockOf, PHASE_LABEL, phaseOf, type Screening } from "./types";

  let rows = $state<Screening[]>([]);
  let loading = $state(true);
  let room = $state<string | null>(null);
  let now = $state(Date.now());

  async function load() {
    try {
      const r = (await call<{ rows: Screening[] }>("theatreLive")) as unknown as { rows: Screening[] };
      rows = r.rows;
      now = Date.now();
    } catch (e) {
      sayError(e);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    void load();
    const t = setInterval(load, 20_000);
    return () => clearInterval(t);
  });
</script>

{#if loading}
  <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
{:else if !rows.length}
  <div class="card empty">Nothing is in its lobby, playing or in after-chat, and nothing starts in the next three hours.</div>
{:else}
  <div class="grid cols-2">
    {#each rows as s (s.id)}
      {@const phase = phaseOf(s, now)}
      <button class="card live" class:hot={phase === "live"} onclick={() => (room = s.id)}>
        <div class="head">
          <span class="chip {phase === 'live' ? 'bad' : phase === 'upcoming' ? '' : 'warn'}">{PHASE_LABEL[phase]}</span>
          {#if s.special}<span class="chip info">Special</span>{/if}
          {#if s.chat_mode !== "normal"}<span class="chip warn">Chat: {s.chat_mode === "closed" ? "reactions only" : "slow"}</span>{/if}
          <span class="muted when">{clockOf(s.starts_at)} to {clockOf(s.ends_at)}</span>
        </div>
        <strong class="name">{s.name}</strong>
        <div class="nums">
          <span><b>{count(s.entered ?? 0)}</b> came in</span>
          <span><b>{count(s.booked)}</b>{s.mode === "fixed" ? ` of ${count(s.capacity)}` : ""} booked</span>
          {#if s.waitlisted}<span><b>{count(s.waitlisted)}</b> waiting</span>{/if}
        </div>
        <span class="muted open">Open the room<m3e-icon name="arrow_forward"></m3e-icon></span>
      </button>
    {/each}
  </div>
  <p class="muted foot">"Came in" counts people who have entered the lobby or show. It doesn't drop when they leave.</p>
{/if}

{#if room}<RoomPanel id={room} onclose={() => { room = null; void load(); }} />{/if}

<style>
  .live {
    display: flex;
    flex-direction: column;
    gap: 10px;
    text-align: left;
    color: inherit;
    font: inherit;
    cursor: pointer;
    transition:
      background-color 160ms ease,
      transform 160ms ease;
  }
  .live:hover {
    background: var(--md-sys-color-surface-container-high);
    transform: translateY(-2px);
  }
  .live.hot {
    border-color: color-mix(in srgb, #ff5449 60%, transparent);
  }
  .head {
    display: flex;
    gap: 6px;
    align-items: center;
    flex-wrap: wrap;
  }
  .when {
    margin-left: auto;
    font-size: 12.5px;
  }
  .name {
    font-size: 18px;
  }
  .nums {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
    font-size: 13.5px;
  }
  .nums b {
    font-family: var(--display);
    font-weight: 400;
    font-size: 24px;
    margin-right: 2px;
  }
  .open {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 12.5px;
    --m3e-icon-size: 16px;
  }
  .foot {
    margin: 0;
    font-size: 12.5px;
  }
</style>
