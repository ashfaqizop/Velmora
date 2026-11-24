<script lang="ts">
  // A Theatre screening, from the lobby to the end. The movie loads paused
  // under the lobby (caching ahead) while the countdown runs; the last five
  // seconds count down big, then the lobby opens up into the movie, full
  // screen, on the shared clock. Joining late goes straight to the live point.
  import { onDestroy, onMount } from "svelte";
  import { fade, fly, scale } from "svelte/transition";
  import { backOut, cubicOut } from "svelte/easing";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Player, { type TheatreLive } from "./Player.svelte";
  import AdSlot from "../components/AdSlot.svelte";
  import { clock, runtime, span } from "../components/theatre/time";
  import { account } from "../lib/account.svelte";
  import * as api from "../lib/api";
  import { nav, settings } from "../lib/state.svelte";
  import { theatre, type Screening } from "../lib/theatre.svelte";
  import { room } from "../lib/room.svelte";
  import Chat from "../components/theatre/Chat.svelte";
  import Reactions from "../components/theatre/Reactions.svelte";
  import Watchers from "../components/theatre/Watchers.svelte";
  import RateCard from "../components/theatre/RateCard.svelte";
  import ScreeningCard from "../components/theatre/ScreeningCard.svelte";
  import type { TitleSummary } from "../lib/types";

  let { id }: { id: string } = $props();

  interface Entry {
    allowed: boolean;
    reason?: "not_yet" | "over" | "waitlist" | "no_ticket";
    server_now: string;
  }

  let entry = $state<Entry | null>(null);
  let screening = $state<Screening | null>(null);
  let title = $state<TitleSummary | null>(null);
  let problem = $state("");
  let now = $state(theatre.serverNow());
  let ended = $state(false);
  let joinedLate = $state(false);
  let moving = $state(false);
  let player = $state({ phase: "sources", status: "Finding a source", source: null as string | null, cachedSecs: 0, time: 0 });

  // Where the chat sits during the movie: over it, beside it, or hidden (C cycles).
  type Layout = "overlay" | "sidebar" | "hidden";
  const LAYOUT_KEY = "velmora.theatre.layout";
  let layout = $state<Layout>((() => {
    try {
      const v = localStorage.getItem(LAYOUT_KEY);
      return v === "sidebar" || v === "hidden" ? v : "overlay";
    } catch {
      return "overlay";
    }
  })());
  function setLayout(l: Layout) {
    layout = l;
    try {
      localStorage.setItem(LAYOUT_KEY, l);
    } catch {
      // Not remembered.
    }
  }
  const SIDEBAR = 360;
  let chat = $state<ReturnType<typeof Chat>>();
  const moviePos = () => (stage === "lobby" ? null : player.time);

  function keys(e: KeyboardEvent) {
    if ((e.target as HTMLElement)?.tagName === "INPUT" || e.ctrlKey || e.metaKey || e.altKey) return;
    if (e.key === "c" || e.key === "C") {
      if (stage !== "show") return;
      setLayout(layout === "overlay" ? "sidebar" : layout === "sidebar" ? "hidden" : "overlay");
    } else if (e.key === "Enter" && !room.quiet) {
      if (stage === "show" && layout === "hidden") setLayout("overlay");
      setTimeout(() => chat?.focus(), 30);
    }
  }

  const start = $derived(screening ? Date.parse(screening.starts_at) : 0);
  const end = $derived(screening ? Date.parse(screening.ends_at) : 0);
  /** lobby: before the start; show: the movie (credits included, sources run
   *  a little longer than TMDB says); after: it ended, or the chat closed. */
  const stage = $derived(!screening ? "loading" : now < start ? "lobby" : ended || now >= end + 30 * 60_000 ? "after" : "show");
  const left = $derived(Math.max(0, start - now));
  const finalCount = $derived(stage === "lobby" && left <= 5_000 ? Math.ceil(left / 1000) : 0);
  const ready = $derived(player.phase === "playing" && player.cachedSecs >= 20);
  const nextUp = $derived(theatre.current.find((s) => s.id !== id && ["upcoming", "lobby"].includes(theatre.phase(s))) ?? null);
  const others = $derived(theatre.current.filter((s) => s.id !== id && ["upcoming", "lobby"].includes(theatre.phase(s))));

  let ticker: ReturnType<typeof setInterval> | undefined;

  onMount(async () => {
    ticker = setInterval(() => (now = theatre.serverNow()), 100);
    window.addEventListener("keydown", keys);
    await enter();
  });

  onDestroy(() => {
    clearInterval(ticker);
    window.removeEventListener("keydown", keys);
    room.leave();
    getCurrentWindow().setFullscreen(false).catch(() => {});
  });

  async function enter() {
    problem = "";
    try {
      const [{ data, error }, row] = await Promise.all([
        account.client.rpc("theatre_enter", { sid: id }),
        account.client.from("theatre_screenings").select("*").eq("id", id).maybeSingle(),
      ]);
      if (error) throw error;
      entry = data as Entry;
      screening = (row.data as Screening) ?? null;
      // The server's clock, straight from this answer if we had none yet.
      if (!theatre.offset && entry.server_now) theatre.offset = Date.parse(entry.server_now) - Date.now();
      now = theatre.serverNow();
      if (!screening) throw new Error("not_found");
      joinedLate = now >= Date.parse(screening.starts_at) + 30_000;
      if (entry.allowed) {
        room.join(screening.id, screening.capacity);
        const [t] = await api.tmdbTitles([screening.k]);
        title = t ?? null;
        if (!title) problem = "Velmora couldn't load this movie's details.";
      }
    } catch (e) {
      const m = (e as { message?: string }).message ?? "";
      problem = m.includes("not_found") ? "This screening isn't on the lineup any more." : /fetch|network/i.test(m) ? "You seem to be offline." : "Couldn't open this screening.";
    }
  }

  function leave() {
    nav.back();
  }

  async function moveTo(s: Screening) {
    if (!screening) return;
    moving = false;
    await theatre.cancel(screening);
    await theatre.book(s);
    nav.replace({ name: "theatre", tab: "tickets" });
  }

  async function bookHere() {
    if (!screening) return;
    await theatre.book(screening);
    await enter();
  }

  // The live clock for the player. A fresh object each time `hold` changes is fine.
  const live = $derived<TheatreLive | null>(
    screening
      ? {
          startsAt: start,
          endsAt: end,
          now: () => theatre.serverNow(),
          hold: stage === "lobby",
          onstate: (s) => (player = s),
          onended: () => (ended = true),
          onleave: leave,
          insetRight: stage === "show" && layout === "sidebar" ? SIDEBAR : 0,
        }
      : null,
  );

  const hms = (ms: number) => {
    const t = Math.ceil(ms / 1000);
    const h = Math.floor(t / 3600);
    const m = Math.floor((t % 3600) / 60);
    const s = t % 60;
    const pad = (n: number) => String(n).padStart(2, "0");
    return h ? `${h}:${pad(m)}:${pad(s)}` : `${pad(m)}:${pad(s)}`;
  };

  /** The lobby lifts away from the middle into the movie. */
  function lift(_node: Element) {
    return {
      duration: 900,
      easing: cubicOut,
      css: (t: number) => `opacity: ${t}; transform: scale(${1 + (1 - t) * 0.08}); filter: blur(${(1 - t) * 12}px)`,
    };
  }

  const SHAPES = ["sunny", "soft-burst", "flower", "12-sided-cookie", "pill"];
</script>

<div class="screening">
  {#if title && entry?.allowed && live}
    <Player {title} target={{ type: "movie" }} mode={settings.audioMode} theatre={live} />
  {/if}

  {#if problem || (entry && !entry.allowed)}
    <section class="gate" in:fade={{ duration: 300 }}>
      <m3e-icon-button class="back" variant="tonal" aria-label="Back" onclick={leave}><m3e-icon name="arrow_back"></m3e-icon></m3e-icon-button>
      <div class="gate-card">
        <span class="gate-icon"><m3e-shape name="soft-burst"></m3e-shape><m3e-icon name={entry?.reason === "waitlist" ? "hourglass_top" : "theaters"}></m3e-icon></span>
        {#if problem}
          <h1>Can't open this screening</h1>
          <p>{problem}</p>
          <m3e-button variant="filled" onclick={enter}><m3e-icon slot="icon" name="refresh"></m3e-icon>Try again</m3e-button>
        {:else if entry?.reason === "not_yet" && screening}
          <h1>The lobby isn't open yet</h1>
          <p>It opens 30 minutes before {screening.name} starts, at {clock(start - 30 * 60_000)}.</p>
        {:else if entry?.reason === "waitlist"}
          <h1>You're on the waitlist</h1>
          <p>This screening is full. If a seat frees up, Velmora lets you know and you can come in.</p>
        {:else if entry?.reason === "no_ticket" && screening}
          <h1>This screening needs a ticket</h1>
          <p>{screening.name} is sold out or ticketed.{theatre.seatsLeft(screening) > 0 ? " There are seats left." : ""}</p>
          {#if theatre.seatsLeft(screening) > 0}
            <m3e-button variant="filled" onclick={bookHere}><m3e-icon slot="icon" name="confirmation_number"></m3e-icon>Book a seat</m3e-button>
          {/if}
        {:else}
          <h1>This screening is over</h1>
          <p>Check the lineup for the next one.</p>
        {/if}
      </div>
    </section>
  {:else if screening && stage === "lobby"}
    <section class="lobby" out:lift style:--bg={screening.backdrop ? `url(${screening.backdrop})` : "none"}>
      <span class="art" aria-hidden="true"></span>
      <header class="bar" data-tauri-drag-region>
        <m3e-icon-button variant="tonal" aria-label="Leave the lobby" onclick={leave}><m3e-icon name="arrow_back"></m3e-icon></m3e-icon-button>
        <span class="where" data-tauri-drag-region><m3e-icon name="theaters"></m3e-icon>Theatre · lobby</span>
        <span class="grow" data-tauri-drag-region></span>
        <Watchers />
      </header>

      <div class="body" in:fly={{ y: 16, duration: 500, easing: cubicOut }}>
        <div class="info">
          {#if screening.kind === "pick"}<span class="chip"><m3e-icon name="auto_awesome" filled></m3e-icon>Tonight's pick</span>{/if}
          <h1>{screening.name}</h1>
          <p class="meta">{screening.year ?? ""} · {runtime(screening.runtime)} · Starts {clock(start)}</p>

          <div class="count" aria-live="off">
            <span class="label">Starts in</span>
            <span class="digits">{hms(left)}</span>
          </div>

          <div class="ready" class:ok={ready} class:bad={player.phase === "error"}>
            {#if player.phase === "error"}
              <m3e-icon name="error"></m3e-icon>
              <span>Still looking for a source. Velmora keeps trying and joins as soon as it finds one.</span>
            {:else if ready}
              <m3e-icon name="check_circle" filled></m3e-icon>
              <span>Ready{player.source ? ` · ${player.source}` : ""} · {Math.floor(player.cachedSecs / 60) || "<1"} min loaded ahead</span>
            {:else}
              <m3e-loading-indicator class="spin"></m3e-loading-indicator>
              <span>{player.phase === "playing" ? `Loading ahead · ${Math.round(player.cachedSecs)} s ready` : player.status}</span>
            {/if}
          </div>

          {#if player.phase === "error" && others.length}
            <div class="move">
              <m3e-button variant="tonal" onclick={() => (moving = !moving)}><m3e-icon slot="icon" name="swap_horiz"></m3e-icon>Move my ticket</m3e-button>
              {#if moving}
                <div class="move-list" transition:fly={{ y: 6, duration: 200 }}>
                  {#each others.slice(0, 5) as s (s.id)}
                    <button type="button" onclick={() => moveTo(s)}><strong>{s.name}</strong><span>{clock(s.starts_at)}</span></button>
                  {/each}
                </div>
              {/if}
            </div>
          {/if}
        </div>

        <div class="lobby-side">
          {#if left <= 5 * 60_000 && left > 6_000}
            <div class="ad" transition:fade={{ duration: 300 }}><AdSlot id="lobby" /></div>
          {/if}
          <div class="lobby-chat"><Chat bind:this={chat} layout="sidebar" {moviePos} /></div>
        </div>
      </div>

      {#if finalCount > 0}
        <div class="final" transition:fade={{ duration: 250 }}>
          {#key finalCount}
            <span class="num" in:scale={{ start: 0.4, duration: 450, easing: backOut }} out:scale={{ start: 1.6, duration: 350, opacity: 0 }}>
              <m3e-shape name={SHAPES[finalCount % SHAPES.length]} class="num-shape"></m3e-shape>
              <span class="num-text">{finalCount}</span>
            </span>
          {/key}
        </div>
      {/if}
    </section>
  {:else if stage === "after" && screening}
    <section class="after" in:fade={{ duration: 600 }}>
      <div class="end">
        <div class="stamp" in:scale={{ start: 0.5, duration: 700, delay: 200, easing: backOut }}>
          <span class="disc">{#if screening.poster}<img src={screening.poster} alt="" />{/if}</span>
          <span class="date">{new Date(start).toLocaleDateString([], { day: "numeric", month: "short", year: "numeric" })}</span>
        </div>
        <h1 class="end-title">You were there</h1>
        <p class="end-sub">
          {screening.name}, watched together.{account.profile?.share_activity !== false ? " The stamp goes on your Velmora Social profile." : ""}
        </p>
        <div class="end-rate" in:fly={{ y: 12, duration: 400, delay: 500 }}><RateCard k={screening.k} name={screening.name} /></div>
        {#if nextUp}
          <div class="end-next" in:fly={{ y: 12, duration: 400, delay: 700 }}>
            <span class="next-label">Next in Theatre</span>
            <ScreeningCard s={nextUp} onenter={(s) => nav.replace({ name: "screening", id: s.id })} />
          </div>
        {/if}
        <m3e-button variant="text" onclick={() => nav.replace({ name: "theatre" })}><m3e-icon slot="icon" name="theaters"></m3e-icon>Back to Theatre</m3e-button>
      </div>
      {#if now < end + 30 * 60_000}
        <aside class="after-chat"><Chat bind:this={chat} layout="sidebar" {moviePos} closesIn={end + 30 * 60_000 - now} /></aside>
      {/if}
    </section>
  {:else if !screening && !problem}
    <section class="gate"><m3e-loading-indicator></m3e-loading-indicator></section>
  {/if}

  {#if stage === "show" && entry?.allowed}
    {#if layout === "sidebar"}
      <aside class="show-side" style:width="{SIDEBAR}px" transition:fly={{ x: SIDEBAR, duration: 360, easing: cubicOut }}>
        <Chat bind:this={chat} layout="sidebar" {moviePos} onlayout={setLayout} />
      </aside>
    {:else if layout === "overlay"}
      <Chat bind:this={chat} layout="overlay" {moviePos} onlayout={setLayout} />
    {:else}
      <button type="button" class="chat-fab" title="Show chat (C)" onclick={() => setLayout("overlay")} transition:fade={{ duration: 200 }}>
        <m3e-icon name="forum"></m3e-icon>{room.watchers.length}
      </button>
    {/if}
  {/if}
  {#if entry?.allowed && stage !== "loading"}<Reactions />{/if}

  {#if stage === "show" && joinedLate}
    <div class="toast" in:fly={{ y: -12, duration: 300 }} out:fade={{ duration: 300 }}
      {@attach () => {
        const t = setTimeout(() => (joinedLate = false), 4000);
        return () => clearTimeout(t);
      }}>
      <span class="live-dot"></span>Joined live, {span(now - start)} in
    </div>
  {/if}
</div>

<style>
  .screening {
    position: fixed;
    inset: 0;
    z-index: 60;
    color: #fff;
  }
  .gate,
  .after,
  .lobby {
    position: absolute;
    inset: 0;
    z-index: 70;
  }
  .gate,
  .after {
    display: grid;
    place-items: center;
    background: #070a14;
  }
  .after {
    background: rgb(7 10 20 / 88%);
    backdrop-filter: blur(18px);
  }
  .back {
    position: absolute;
    top: 16px;
    left: 20px;
  }
  .gate-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
    max-width: 480px;
    padding: 24px;
    text-align: center;
  }
  .gate-card h1 {
    margin: 0;
    font-size: 28px;
  }
  .gate-card p {
    margin: 0;
    color: rgb(255 255 255 / 75%);
    line-height: 1.5;
  }
  .gate-icon {
    display: inline-grid;
    place-items: center;
    width: 84px;
    height: 84px;
    --m3e-shape-size: 84px;
    --m3e-shape-container-color: var(--md-sys-color-primary-container);
    --m3e-icon-size: 36px;
    color: var(--md-sys-color-on-primary-container);
  }
  .gate-icon > * {
    grid-area: 1 / 1;
  }
  .gate-icon m3e-icon {
    position: relative;
    z-index: 1;
  }

  /* ── Lobby ── */
  .lobby {
    display: flex;
    flex-direction: column;
    background: #070a14;
    overflow: hidden;
    isolation: isolate;
  }
  .art {
    position: absolute;
    inset: -4%;
    z-index: -1;
    background:
      linear-gradient(90deg, rgb(7 10 20 / 95%) 25%, rgb(7 10 20 / 55%) 60%, rgb(7 10 20 / 30%)),
      linear-gradient(0deg, rgb(7 10 20 / 90%), transparent 50%),
      var(--bg) center / cover no-repeat;
    animation: drift 40s ease-in-out infinite alternate;
  }
  @keyframes drift {
    to {
      transform: scale(1.06) translate(-1%, -1%);
    }
  }
  .bar {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 16px 24px;
  }
  .where {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    font-weight: 600;
    opacity: 0.85;
    --m3e-icon-size: 18px;
  }
  .body {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 32px;
    padding: 0 clamp(24px, 6vw, 96px) 48px;
  }
  .info {
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-width: 640px;
  }
  .chip {
    align-self: flex-start;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 12px;
    border-radius: 999px;
    font-size: 13px;
    font-weight: 700;
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
    --m3e-icon-size: 16px;
  }
  h1 {
    margin: 0;
    font-family: "Instrument Serif", Georgia, serif;
    font-weight: 400;
    font-size: clamp(44px, 6vw, 80px);
    line-height: 1;
    letter-spacing: -0.01em;
  }
  .meta {
    margin: 0;
    font-size: 16px;
    opacity: 0.8;
  }
  .count {
    display: flex;
    flex-direction: column;
    margin-top: 12px;
  }
  .count .label {
    font-size: 13px;
    font-weight: 700;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    opacity: 0.7;
  }
  .digits {
    font-size: clamp(56px, 8vw, 104px);
    font-weight: 800;
    line-height: 1;
    font-variant-numeric: tabular-nums;
    letter-spacing: -0.02em;
  }
  .ready {
    display: flex;
    align-items: center;
    gap: 10px;
    align-self: flex-start;
    padding: 10px 16px;
    border-radius: 999px;
    font-size: 14px;
    background: rgb(255 255 255 / 8%);
    --m3e-icon-size: 20px;
    transition: background-color 300ms ease;
  }
  .ready.ok {
    background: rgb(62 207 142 / 18%);
    color: #8ff0c3;
  }
  .ready.bad {
    background: rgb(255 107 107 / 16%);
    color: #ffb4ab;
  }
  .spin {
    --m3e-loading-indicator-size: 22px;
  }
  .move {
    display: flex;
    flex-direction: column;
    gap: 8px;
    align-items: flex-start;
  }
  .move-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 320px;
    padding: 6px;
    border-radius: 18px;
    background: rgb(255 255 255 / 6%);
  }
  .move-list button {
    all: unset;
    display: flex;
    justify-content: space-between;
    gap: 16px;
    padding: 10px 14px;
    border-radius: 12px;
    cursor: pointer;
  }
  .move-list button:hover {
    background: rgb(255 255 255 / 8%);
  }
  .ad {
    flex: none;
    padding: 12px;
    border-radius: 20px;
    background: rgb(0 0 0 / 35%);
  }

  .grow {
    flex: 1;
  }
  .lobby-side {
    flex: none;
    align-self: stretch;
    width: 380px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-height: 0;
    max-height: calc(100vh - 120px);
  }
  .lobby-chat {
    flex: 1;
    min-height: 0;
    border-radius: 24px;
    overflow: hidden;
  }
  .ad {
    flex: none;
    align-self: center;
  }
  .after {
    grid-template-columns: 1fr auto;
    overflow-y: auto;
  }
  .end {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
    width: min(520px, 100% - 48px);
    margin: 40px auto;
    text-align: center;
  }
  .stamp {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }
  .disc {
    width: 132px;
    height: 132px;
    border-radius: 50%;
    overflow: hidden;
    rotate: -6deg;
    box-shadow:
      0 0 0 4px #070a14,
      0 0 0 8px var(--md-sys-color-primary),
      0 24px 50px -20px rgb(50 113 254 / 70%);
    background: rgb(255 255 255 / 8%);
  }
  .disc img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .date {
    padding: 3px 12px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 800;
    letter-spacing: 0.05em;
    background: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
  }
  .end-title {
    font-size: clamp(40px, 5vw, 60px);
  }
  .end-sub {
    margin: 0;
    opacity: 0.8;
    line-height: 1.5;
  }
  .end-rate,
  .end-next {
    width: 100%;
  }
  .end-next {
    display: flex;
    flex-direction: column;
    gap: 8px;
    text-align: left;
  }
  .next-label {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    opacity: 0.7;
  }
  .after-chat {
    width: 380px;
    height: 100vh;
  }
  .show-side {
    position: absolute;
    z-index: 72;
    top: 0;
    right: 0;
    bottom: 0;
  }
  .chat-fab {
    all: unset;
    position: absolute;
    z-index: 74;
    left: 24px;
    bottom: 118px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 8px 14px;
    border-radius: 999px;
    font-size: 13px;
    font-weight: 700;
    background: rgb(0 0 0 / 50%);
    backdrop-filter: blur(10px);
    cursor: pointer;
    opacity: 0.7;
    transition: opacity 150ms ease;
    --m3e-icon-size: 18px;
  }
  .chat-fab:hover,
  .chat-fab:focus-visible {
    opacity: 1;
  }

  /* ── The last five seconds ── */
  .final {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    background: rgb(7 10 20 / 55%);
    backdrop-filter: blur(6px);
  }
  .num {
    grid-area: 1 / 1;
    display: grid;
    place-items: center;
    width: 260px;
    height: 260px;
    --m3e-shape-size: 260px;
    --m3e-shape-container-color: var(--md-sys-color-primary);
  }
  .num > * {
    grid-area: 1 / 1;
  }
  .num-shape {
    animation: turn 5s linear infinite;
  }
  .num-text {
    position: relative;
    z-index: 1;
    font-size: 150px;
    font-weight: 800;
    line-height: 1;
    color: var(--md-sys-color-on-primary);
    font-variant-numeric: tabular-nums;
  }
  @keyframes turn {
    to {
      rotate: 1turn;
    }
  }

  .toast {
    position: absolute;
    z-index: 75;
    top: 20px;
    left: 0;
    right: 0;
    width: max-content;
    margin: 0 auto;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 18px;
    border-radius: 999px;
    font-size: 14px;
    font-weight: 600;
    background: rgb(0 0 0 / 70%);
    backdrop-filter: blur(10px);
  }
  .live-dot {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: #ff5449;
    animation: blink 1.4s ease-in-out infinite;
  }
  @keyframes blink {
    50% {
      opacity: 0.3;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .art,
    .num-shape,
    .live-dot {
      animation: none;
    }
  }
</style>
