<script lang="ts">
  // A screening's live chat on Velmora Social, read-only: messages, movie-time
  // stamps, @mentions and reactions as they happen. Chatting is in the app.
  import { tick } from "svelte";
  import { fade, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import Avatar from "../auth/Avatar.svelte";
  import { liveRoom } from "./liveroom.svelte";
  import { profileHref } from "./nav.svelte";
  import { openInApp, span, theatre, type Screening } from "./theatre.svelte";

  let { s, onclose }: { s: Screening; onclose: () => void } = $props();

  let list = $state<HTMLDivElement>();
  let stuck = $state(true);
  let floats = $state<{ id: number; e: string; x: number; drift: number; size: number }[]>([]);
  let seq = 0;
  const recent: { e: string; t: number }[] = [];

  $effect(() => {
    const id = s.id;
    liveRoom.open(id);
    liveRoom.onReact = (e) => {
      const now = Date.now();
      recent.push({ e, t: now });
      while (recent.length && now - recent[0].t > 1500) recent.shift();
      const same = recent.filter((r) => r.e === e).length;
      const f = { id: ++seq, e, x: Math.random() * 70, drift: (Math.random() - 0.5) * 60, size: Math.min(1.8, 1 + (same - 1) * 0.12) };
      floats = [...floats.slice(-40), f];
      setTimeout(() => (floats = floats.filter((q) => q.id !== f.id)), 2700);
    };
    return () => {
      liveRoom.onReact = null;
      liveRoom.close();
    };
  });

  $effect(() => {
    void liveRoom.messages.length;
    if (stuck && list) tick().then(() => list && (list.scrollTop = list.scrollHeight));
  });

  function onScroll() {
    if (list) stuck = list.scrollHeight - list.scrollTop - list.clientHeight < 40;
  }

  const phase = $derived(theatre.phase(s));
  const start = $derived(Date.parse(s.starts_at));
  const end = $derived(Date.parse(s.ends_at));
  const stamp = (at: number | null) => {
    if (at === null || at < 0) return "lobby";
    const t = Math.floor(at / 1000);
    const h = Math.floor(t / 3600);
    const m = Math.floor((t % 3600) / 60);
    const sec = String(t % 60).padStart(2, "0");
    return h ? `${h}:${String(m).padStart(2, "0")}:${sec}` : `${m}:${sec}`;
  };
  const parts = (body: string) => body.split(/(@[A-Za-z0-9_]{3,20})/g).filter(Boolean);

  function key(e: KeyboardEvent) {
    if (e.key === "Escape") onclose();
  }
</script>

<svelte:window onkeydown={key} />

<button type="button" class="scrim" aria-label="Close live chat" onclick={onclose} transition:fade={{ duration: 200 }}></button>
<aside class="drawer" aria-label="Live chat for {s.name}" transition:fly={{ x: 420, duration: 380, easing: cubicOut }}>
  <header>
    <div class="top">
      {#if phase === "live"}<span class="live"><i></i>Live</span>{:else if phase === "lobby"}<span class="lobby">Lobby · starts in {span(start - theatre.now)}</span>{:else}<span class="lobby">Just ended · chat closes in {span(end + 30 * 60_000 - theatre.now)}</span>{/if}
      <m3e-icon-button aria-label="Close" onclick={onclose}><m3e-icon name="close"></m3e-icon></m3e-icon-button>
    </div>
    <strong class="name">{s.name}</strong>
    <div class="watch">
      <span class="faces">{#each liveRoom.watchers.slice(0, 5) as w (w.uid)}<a href={profileHref(w.username)} title={w.display_name}><Avatar value={w.avatar} size={26} /></a>{/each}</span>
      <span><b>{liveRoom.watchers.length}</b> watching in Velmora</span>
    </div>
  </header>

  <div class="messages" bind:this={list} onscroll={onScroll}>
    {#if liveRoom.status === "denied"}
      <p class="note">This screening is ticketed, so only people with a seat can follow its chat.</p>
    {:else if liveRoom.status === "connecting"}
      <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
    {:else if !liveRoom.messages.length}
      <p class="note">It's quiet so far. Messages show up here as people send them from the app; nothing is saved.</p>
    {/if}
    {#each liveRoom.messages as m (m.id)}
      <div class="msg" class:mention={m.mention} in:fly={{ y: 8, duration: 220, easing: cubicOut }}>
        <a href={profileHref(m.username)} class="pic"><Avatar value={m.avatar} size={32} /></a>
        <div class="text">
          <span class="line">
            <a href={profileHref(m.username)} class="who">{m.display_name}</a>
            {#if m.verified}<m3e-icon name="verified" filled class="tick"></m3e-icon>{/if}
            <span class="at">{stamp(m.at)}</span>
          </span>
          <span class="body">{#each parts(m.body) as p, i (i)}{#if p.startsWith("@")}<b class="tag">{p}</b>{:else}{p}{/if}{/each}</span>
        </div>
      </div>
    {/each}
  </div>

  <div class="floats" aria-hidden="true">
    {#each floats as f (f.id)}<span style:left="{f.x}%" style:--drift="{f.drift}px" style:--size={f.size}>{f.e}</span>{/each}
  </div>

  <footer>
    <span class="hint"><m3e-icon name="visibility"></m3e-icon>You're following along. Chat and react from the Velmora app.</span>
    <m3e-button variant="filled" href={openInApp(s.id)}><m3e-icon slot="icon" name="play_arrow" filled></m3e-icon>{phase === "lobby" ? "Join the lobby" : "Watch in Velmora"}</m3e-button>
  </footer>
</aside>

<style>
  .scrim {
    all: unset;
    position: fixed;
    inset: 0;
    z-index: 40;
    background: rgb(0 0 0 / 45%);
    backdrop-filter: blur(2px);
  }
  .drawer {
    position: fixed;
    z-index: 41;
    top: 0;
    right: 0;
    bottom: 0;
    width: min(420px, 100%);
    display: flex;
    flex-direction: column;
    background: color-mix(in srgb, #0d1324 97%, transparent);
    border-left: 1px solid rgb(255 255 255 / 8%);
    box-shadow: -30px 0 80px -30px rgb(0 0 0 / 80%);
  }
  header {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 14px 16px 14px 20px;
    border-bottom: 1px solid rgb(255 255 255 / 7%);
  }
  .top {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .live,
  .lobby {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 3px 12px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 800;
  }
  .live {
    background: var(--md-sys-color-error);
    color: var(--md-sys-color-on-error);
  }
  .live i {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: currentColor;
    animation: pulse 1.4s ease-in-out infinite;
  }
  .lobby {
    background: var(--md-sys-color-tertiary-container);
    color: var(--md-sys-color-on-tertiary-container);
  }
  .name {
    font-family: var(--display);
    font-weight: 400;
    font-size: 26px;
    line-height: 1.1;
  }
  .watch {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .watch b {
    color: var(--md-sys-color-on-surface);
  }
  .faces {
    display: flex;
  }
  .faces a {
    display: inline-grid;
    margin-left: -6px;
    border-radius: 50%;
    box-shadow: 0 0 0 2px #0d1324;
  }
  .faces a:first-child {
    margin-left: 0;
  }
  .messages {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 10px 10px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .note {
    margin: auto 20px;
    text-align: center;
    font-size: 14px;
    line-height: 1.5;
    color: var(--md-sys-color-on-surface-variant);
  }
  .center {
    margin: auto;
  }
  .msg {
    display: flex;
    gap: 10px;
    padding: 7px 10px;
    border-radius: 14px;
  }
  .msg:hover {
    background: rgb(255 255 255 / 4%);
  }
  .msg.mention {
    background: rgb(50 113 254 / 18%);
    box-shadow: inset 3px 0 0 var(--md-sys-color-primary);
  }
  .pic {
    display: inline-grid;
    flex: none;
    align-self: flex-start;
  }
  .text {
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .line {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
  }
  .who {
    color: inherit;
    font-weight: 700;
    text-decoration: none;
  }
  .who:hover {
    text-decoration: underline;
  }
  .tick {
    --m3e-icon-size: 13px;
    color: var(--md-sys-color-primary);
  }
  .at {
    font-size: 11px;
    opacity: 0.5;
    font-variant-numeric: tabular-nums;
  }
  .body {
    font-size: 14px;
    line-height: 1.45;
    overflow-wrap: anywhere;
  }
  .tag {
    color: #9db8ff;
  }
  .floats {
    position: absolute;
    right: 18px;
    bottom: 110px;
    width: 110px;
    height: 55%;
    pointer-events: none;
  }
  .floats span {
    position: absolute;
    bottom: 0;
    font-size: calc(28px * var(--size));
    filter: drop-shadow(0 4px 10px rgb(0 0 0 / 50%));
    animation: rise 2.6s cubic-bezier(0.2, 0.7, 0.3, 1) forwards;
  }
  @keyframes rise {
    0% {
      transform: translate(0, 0) scale(0.4);
      opacity: 0;
    }
    12% {
      transform: translate(calc(var(--drift) * 0.15), -20px) scale(1.1);
      opacity: 1;
    }
    100% {
      transform: translate(var(--drift), -380px) scale(0.9);
      opacity: 0;
    }
  }
  footer {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 14px 18px 18px;
    border-top: 1px solid rgb(255 255 255 / 7%);
  }
  .hint {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
    --m3e-icon-size: 18px;
  }
  @keyframes pulse {
    50% {
      opacity: 0.35;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .live i {
      animation: none;
    }
    .floats span {
      animation-duration: 1ms;
    }
  }
</style>
