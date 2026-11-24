<script lang="ts">
  // Theatre's live chat. Two looks:
  //  - sidebar: a panel on the right, full height, the video beside it;
  //  - overlay: messages float over the bottom-left of the movie on a clear
  //    background and fade after a few seconds; hovering or typing brings
  //    them back.
  // Messages carry the movie time they were sent at, @mentions light up,
  // and reactions float over the video. Nothing is saved.
  import { tick, untrack } from "svelte";
  import { fade, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import Avatar from "../Avatar.svelte";
  import Who from "./Who.svelte";
  import Watchers from "./Watchers.svelte";
  import { account } from "../../lib/account.svelte";
  import { room, REACTIONS, type ChatMessage } from "../../lib/room.svelte";
  import { formatTime } from "../../lib/state.svelte";

  let {
    layout,
    moviePos,
    onlayout,
    closesIn = null,
  }: {
    layout: "sidebar" | "overlay";
    /** The movie time to stamp on a message (seconds), or null in the lobby. */
    moviePos: () => number | null;
    onlayout?: (l: "sidebar" | "overlay" | "hidden") => void;
    /** After the movie: how long until the chat closes (ms). */
    closesIn?: number | null;
  } = $props();

  let text = $state("");
  let input = $state<HTMLInputElement>();
  let list = $state<HTMLDivElement>();
  let stuck = $state(true);
  let unseen = $state(0);
  let focused = $state(false);
  let hovered = $state(false);

  const MAX = 200;

  // Follow new messages unless you scrolled up to read.
  // Only a new message re-runs this; reading `stuck` / `unseen` here is untracked
  // (otherwise `unseen += 1` would re-trigger itself forever).
  $effect(() => {
    const n = room.messages.length;
    void n;
    if (!list) return;
    untrack(() => {
      if (stuck) tick().then(() => list && (list.scrollTop = list.scrollHeight));
      else unseen += 1;
    });
  });

  function onScroll() {
    if (!list) return;
    stuck = list.scrollHeight - list.scrollTop - list.clientHeight < 40;
    if (stuck) unseen = 0;
  }

  function jump() {
    stuck = true;
    unseen = 0;
    list?.scrollTo({ top: list.scrollHeight, behavior: "smooth" });
  }

  async function send(e?: Event) {
    e?.preventDefault();
    const pos = moviePos();
    if (await room.say(text, pos === null ? null : pos * 1000)) {
      text = "";
      jump();
    }
  }

  /** Focus the box (Enter from anywhere in the screening). */
  export function focus() {
    input?.focus();
  }

  // @mention suggestions from the people in the room.
  const mentionQuery = $derived(/(^|\s)@([A-Za-z0-9_]{0,20})$/.exec(text)?.[2] ?? null);
  const suggestions = $derived(
    mentionQuery === null
      ? []
      : room.watchers
          .filter((w) => w.uid !== account.profile?.id && w.username.toLowerCase().startsWith(mentionQuery.toLowerCase()))
          .slice(0, 5),
  );
  function complete(username: string) {
    text = text.replace(/@([A-Za-z0-9_]{0,20})$/, `@${username} `);
    input?.focus();
  }

  /** Split a message so @mentions can be highlighted (Svelte escapes the text). */
  function parts(body: string) {
    return body.split(/(@[A-Za-z0-9_]{3,20})/g).filter(Boolean);
  }

  const stamp = (m: ChatMessage) => (m.at === null || m.at < 0 ? "lobby" : formatTime(m.at / 1000, m.at >= 3_600_000));
  const shown = $derived(layout === "overlay" ? room.messages.slice(-8) : room.messages);
  const awake = $derived(focused || hovered || text.length > 0);
</script>

<section
  class="chat {layout}"
  class:awake
  aria-label="Theatre chat"
  onpointerenter={() => (hovered = true)}
  onpointerleave={() => (hovered = false)}
  in:fly={{ x: layout === "sidebar" ? 40 : -20, duration: 380, easing: cubicOut }}
>
  {#if layout === "sidebar"}
    <header>
      <strong>Live chat</strong>
      <Watchers />
      <span class="grow"></span>
      <m3e-icon-button size="small" aria-label="Show chat over the video" title="Over the video (C)" onclick={() => onlayout?.("overlay")}><m3e-icon name="picture_in_picture"></m3e-icon></m3e-icon-button>
      <m3e-icon-button size="small" aria-label="Hide chat" title="Hide chat (C)" onclick={() => onlayout?.("hidden")}><m3e-icon name="right_panel_close"></m3e-icon></m3e-icon-button>
    </header>
    {#if closesIn !== null}<p class="closing"><m3e-icon name="schedule"></m3e-icon>Chat closes in {Math.ceil(closesIn / 60_000)} min. Nothing here is saved.</p>{/if}
  {/if}

  {#if room.quiet}
    <div class="quiet">
      <m3e-icon name="forum"></m3e-icon>
      <strong>Chat is full</strong>
      <span>You're watching in sync with everyone. Chat turns on as soon as there's room.</span>
    </div>
  {:else}
    <div class="messages" bind:this={list} onscroll={onScroll}>
      {#if layout === "sidebar" && !room.messages.length}
        <p class="empty">{room.failed ? "The chat couldn't connect. It will keep trying." : "Say hi. Messages aren't saved anywhere and vanish when the chat closes."}</p>
      {/if}
      {#each shown as m (m.id)}
        <div class="msg" class:mention={m.mention} class:me={m.uid === account.profile?.id} in:fly={{ y: 8, duration: 220, easing: cubicOut }}>
          <Who uid={m.uid} username={m.username}><Avatar value={m.avatar} size={layout === "overlay" ? 26 : 30} /></Who>
          <div class="text">
            <span class="line">
              <Who uid={m.uid} username={m.username}><span class="name">{m.display_name}</span></Who>
              {#if m.verified}<m3e-icon name="verified" filled class="tick"></m3e-icon>{/if}
              <span class="at">{stamp(m)}</span>
            </span>
            <span class="body">{#each parts(m.body) as p, i (i)}{#if p.startsWith("@")}<b class="mention-tag">{p}</b>{:else}{p}{/if}{/each}</span>
          </div>
        </div>
      {/each}
    </div>
    {#if unseen > 0 && !stuck && layout === "sidebar"}
      <button type="button" class="new" onclick={jump} transition:fade={{ duration: 150 }}><m3e-icon name="arrow_downward"></m3e-icon>{unseen} new</button>
    {/if}

    <div class="compose">
      <div class="reacts" role="group" aria-label="React">
        {#each REACTIONS as r (r)}<button type="button" class="react" aria-label="React {r}" onclick={() => room.react(r)}>{r}</button>{/each}
        {#if layout === "overlay"}
          <span class="grow"></span>
          <Watchers />
          <m3e-icon-button size="small" aria-label="Chat beside the video" title="Beside the video (C)" onclick={() => onlayout?.("sidebar")}><m3e-icon name="right_panel_open"></m3e-icon></m3e-icon-button>
        {/if}
      </div>
      {#if suggestions.length}
        <div class="suggest" transition:fly={{ y: 4, duration: 150 }}>
          {#each suggestions as w (w.uid)}
            <button type="button" onclick={() => complete(w.username)}><Avatar value={w.avatar} size={20} />{w.display_name}<span>@{w.username}</span></button>
          {/each}
        </div>
      {/if}
      <form onsubmit={send}>
        <input
          bind:this={input}
          bind:value={text}
          maxlength={MAX}
          placeholder={room.connected ? "Say something…" : "Connecting…"}
          aria-label="Chat message"
          disabled={!room.connected}
          onfocus={() => (focused = true)}
          onblur={() => (focused = false)}
          onkeydown={(e) => {
            if (e.key === "Tab" && suggestions.length) {
              e.preventDefault();
              complete(suggestions[0].username);
            } else if (e.key === "Escape") input?.blur();
            e.stopPropagation();
          }}
        />
        <m3e-icon-button variant="filled" size="small" aria-label="Send" disabled={!text.trim() || room.sending || !room.connected} onclick={send}>
          <m3e-icon name="send"></m3e-icon>
        </m3e-icon-button>
      </form>
      {#if room.notice}<p class="notice" transition:fade={{ duration: 150 }}>{room.notice}</p>{/if}
    </div>
  {/if}
</section>

<style>
  .chat {
    color: #fff;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  /* ── Sidebar ── */
  .sidebar {
    height: 100%;
    background: color-mix(in srgb, #0d1324 94%, transparent);
    border-left: 1px solid rgb(255 255 255 / 8%);
    backdrop-filter: blur(20px);
  }
  header {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 14px 10px 10px 16px;
  }
  header strong {
    font-size: 15px;
  }
  .grow {
    flex: 1;
  }
  .closing {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 0 16px 8px;
    font-size: 12px;
    opacity: 0.75;
    --m3e-icon-size: 14px;
  }
  .messages {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 4px 8px 8px;
    scrollbar-width: thin;
  }
  .empty {
    margin: auto 12px;
    text-align: center;
    font-size: 13px;
    opacity: 0.6;
    line-height: 1.5;
  }
  .msg {
    display: flex;
    gap: 10px;
    padding: 6px 8px;
    border-radius: 14px;
  }
  .sidebar .msg:hover {
    background: rgb(255 255 255 / 5%);
  }
  .msg.mention {
    background: rgb(50 113 254 / 18%);
    box-shadow: inset 3px 0 0 var(--md-sys-color-primary);
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
  .name {
    font-weight: 700;
    opacity: 0.95;
  }
  .me .name {
    color: #b9ccff;
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
  .mention-tag {
    color: #9db8ff;
  }
  .new {
    all: unset;
    align-self: center;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    margin-top: -40px;
    margin-bottom: 8px;
    padding: 6px 12px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 700;
    background: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
    cursor: pointer;
    --m3e-icon-size: 16px;
    position: relative;
    z-index: 1;
  }
  .compose {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 8px 12px 14px;
  }
  .reacts {
    display: flex;
    align-items: center;
    gap: 2px;
  }
  .react {
    all: unset;
    display: grid;
    place-items: center;
    width: 34px;
    height: 34px;
    border-radius: 50%;
    font-size: 19px;
    cursor: pointer;
    transition: transform 180ms cubic-bezier(0.2, 0.8, 0.2, 1.6), background-color 150ms ease;
  }
  .react:hover {
    transform: scale(1.25);
    background: rgb(255 255 255 / 8%);
  }
  .react:active {
    transform: scale(0.9);
  }
  .react:focus-visible {
    outline: 2px solid var(--md-sys-color-primary);
  }
  form {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  input {
    flex: 1;
    min-width: 0;
    height: 42px;
    padding: 0 16px;
    border: 1px solid rgb(255 255 255 / 12%);
    border-radius: 999px;
    background: rgb(255 255 255 / 6%);
    color: #fff;
    font: inherit;
    font-size: 14px;
    outline: none;
    transition: border-color 150ms ease, background-color 150ms ease;
  }
  input:focus {
    border-color: var(--md-sys-color-primary);
    background: rgb(255 255 255 / 9%);
  }
  .suggest {
    position: absolute;
    left: 12px;
    right: 12px;
    bottom: calc(100% - 8px);
    display: flex;
    flex-direction: column;
    padding: 6px;
    border-radius: 16px;
    background: #141c33;
    border: 1px solid rgb(255 255 255 / 10%);
    box-shadow: 0 16px 40px -16px rgb(0 0 0 / 80%);
  }
  .suggest button {
    all: unset;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 7px 10px;
    border-radius: 10px;
    font-size: 13px;
    cursor: pointer;
  }
  .suggest button:hover,
  .suggest button:focus-visible {
    background: rgb(255 255 255 / 8%);
  }
  .suggest span {
    opacity: 0.6;
  }
  .notice {
    margin: 0;
    font-size: 12px;
    color: #ffb4ab;
  }
  .quiet {
    margin: auto 16px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    text-align: center;
    font-size: 13px;
    opacity: 0.85;
    --m3e-icon-size: 30px;
  }

  /* ── Overlay: over the movie, clear background ── */
  .overlay {
    position: absolute;
    z-index: 74;
    left: 24px;
    bottom: 118px;
    width: min(380px, 40vw);
    max-height: 52vh;
  }
  .overlay .messages {
    justify-content: flex-end;
    overflow: hidden;
    padding: 0;
    mask-image: linear-gradient(0deg, #000 70%, transparent);
  }
  .overlay .msg {
    padding: 4px 6px;
    text-shadow: 0 1px 3px rgb(0 0 0 / 90%);
    animation: settle 10s ease forwards;
  }
  .overlay.awake .msg {
    animation: none;
    opacity: 1;
    background: rgb(0 0 0 / 35%);
  }
  @keyframes settle {
    0%,
    75% {
      opacity: 1;
    }
    100% {
      opacity: 0;
    }
  }
  .overlay .compose {
    padding: 8px 0 0;
    opacity: 0.35;
    transition: opacity 200ms ease;
  }
  .overlay.awake .compose {
    opacity: 1;
  }
  .overlay input {
    background: rgb(0 0 0 / 45%);
    backdrop-filter: blur(10px);
  }
  @media (prefers-reduced-motion: reduce) {
    .overlay .msg {
      animation: none;
    }
  }
</style>
