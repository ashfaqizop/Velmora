<script lang="ts">
  import { tick } from "svelte";
  import { fade, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import AniKey from "../components/AniKey.svelte";
  import AniMessage from "../components/ani/AniMessage.svelte";
  import { chats } from "../lib/ani/chat.svelte";
  import { personalize } from "../lib/ani/personalize.svelte";
  import { ani, library } from "../lib/state.svelte";

  ani.loadKey().catch((e) => console.warn("ani key status", e));

  let draft = $state("");
  let input: HTMLTextAreaElement;
  let thread: HTMLElement;

  const chat = $derived(chats.current);
  const ready = $derived(!!ani.key?.set);

  // Starters that use what Velmora already knows about the user.
  const suggestions = $derived.by(() => {
    const out: string[] = [];
    const watched = personalize.recentTitles(["finish", "stream"], 1)[0] ?? library.history[0]?.title;
    if (watched) out.push(`More like ${watched.name}`);
    out.push("Something short and funny for tonight");
    out.push("What anime is airing this week?");
    const cont = library.continueWatching[0];
    if (cont) out.push(`Is ${cont.title.name} worth finishing?`);
    else out.push("Underrated sci-fi from the last ten years");
    out.push("Best 1080p download of a movie I've been meaning to see");
    return out.slice(0, 4);
  });

  async function send(text = draft) {
    if (!text.trim() || chats.busy || !ready) return;
    draft = "";
    const sending = chats.send(text);
    await tick();
    scrollDown(true);
    await sending;
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey && !e.isComposing) {
      e.preventDefault();
      send();
    }
  }

  /** Follow the reply while it streams, unless the user scrolled up to read. */
  function scrollDown(force = false) {
    const scroller = thread?.closest("main");
    if (!scroller) return;
    const near = scroller.scrollHeight - scroller.scrollTop - scroller.clientHeight < 160;
    if (force || near) scroller.scrollTo({ top: scroller.scrollHeight, behavior: force ? "smooth" : "auto" });
  }

  $effect(() => {
    // Re-run whenever the open chat's content changes.
    void JSON.stringify(chats.current?.turns.at(-1));
    tick().then(() => scrollDown());
  });

  function openChat(id: string) {
    chats.open(id);
    tick().then(() => scrollDown(true));
  }

  function when(at: number) {
    const d = Math.floor((Date.now() - at) / 86_400_000);
    if (d < 1) return "Today";
    if (d < 2) return "Yesterday";
    if (d < 7) return `${d} days ago`;
    return new Date(at).toLocaleDateString(undefined, { day: "numeric", month: "short" });
  }
</script>

<div class="ani-page">
  <aside class="history">
    <m3e-button variant="tonal" class="new" disabled={!chat && !chats.busy} onclick={() => chats.newChat()}>
      <m3e-icon slot="icon" name="edit_square"></m3e-icon>New chat
    </m3e-button>
    {#if chats.list.length}
      <span class="label">Recent</span>
      <m3e-action-list class="list">
        {#each chats.list as c (c.id)}
          <m3e-list-action class:current={chat?.id === c.id} onclick={() => openChat(c.id)}>
            {c.title}
            <span slot="supporting-text">{when(c.updated)}</span>
            <m3e-icon-button
              slot="trailing"
              size="small"
              aria-label="Delete chat"
              onclick={(e: Event) => {
                e.stopPropagation();
                chats.remove(c.id);
              }}
            >
              <m3e-icon name="delete"></m3e-icon>
            </m3e-icon-button>
          </m3e-list-action>
        {/each}
      </m3e-action-list>
    {/if}
  </aside>

  <section class="chat" bind:this={thread}>
    {#if !chat?.turns.length}
      <header class="intro" in:fade={{ duration: 300 }}>
        <span class="on-shape mark" aria-hidden="true">
          <m3e-shape name="very-sunny"></m3e-shape>
          <m3e-icon name="wand_stars" filled></m3e-icon>
        </span>
        <h1 class="display">Ask Ani</h1>
        <p>Recommendations from what you watch, answers about any title, and links found by UniverseAPI.</p>
      </header>

      {#if ani.key && !ready}
        <m3e-card variant="filled" class="setup">
          <div slot="content" class="setup-body">
            <m3e-heading variant="title" size="medium" level={2}>Add your Gemini key</m3e-heading>
            <p>Ani runs on Google's Gemini with a key you bring. Getting one is free and takes a minute.</p>
            <AniKey />
          </div>
        </m3e-card>
      {:else if ready}
        <div class="starters">
          {#each suggestions as s, i (s)}
            <button class="starter" in:fly={{ y: 10, duration: 360, delay: 80 * i, easing: cubicOut }} onclick={() => send(s)}>
              <m3e-icon name="arrow_outward"></m3e-icon>{s}
            </button>
          {/each}
        </div>
      {/if}
    {:else}
      <div class="thread">
        {#each chat.turns as t, i (t.id)}
          {#if t.role === "user"}
            <div class="you" in:fly={{ y: 8, duration: 260, easing: cubicOut }}>{t.text}</div>
          {:else}
            <AniMessage turn={t} last={i === chat.turns.length - 1} />
          {/if}
        {/each}
      </div>
    {/if}

    <div class="composer">
      <div class="box" class:disabled={!ready}>
        <textarea
          bind:this={input}
          bind:value={draft}
          rows="1"
          placeholder={ready ? "Ask Ani about movies, series or anime" : "Add a Gemini key to talk to Ani"}
          disabled={!ready}
          onkeydown={onKey}
          aria-label="Message Ani"
        ></textarea>
        {#if chats.busy}
          <m3e-icon-button variant="filled" aria-label="Stop" onclick={() => chats.stop()}>
            <m3e-icon name="stop" filled></m3e-icon>
          </m3e-icon-button>
        {:else}
          <m3e-icon-button variant="filled" aria-label="Send" disabled={!draft.trim() || !ready} onclick={() => send()}>
            <m3e-icon name="arrow_upward"></m3e-icon>
          </m3e-icon-button>
        {/if}
      </div>
      <span class="fine">Ani runs on Gemini with your key and can get things wrong. Links are found by UniverseAPI.</span>
    </div>
  </section>
</div>

<style>
  .ani-page {
    display: grid;
    grid-template-columns: 240px minmax(0, 1fr);
    gap: 28px;
    min-height: calc(100vh - var(--app-titlebar-height) - 60px);
  }
  .history {
    position: sticky;
    top: 0;
    align-self: start;
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-height: calc(100vh - var(--app-titlebar-height) - 60px);
    overflow-y: auto;
    scrollbar-width: none;
  }
  .new {
    align-self: flex-start;
  }
  .label {
    margin: 12px 0 0 12px;
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--md-sys-color-on-surface-variant);
  }
  .list m3e-list-action {
    border-radius: 16px;
    --m3e-list-item-label-text-size: 14px;
  }
  .list m3e-list-action.current {
    background: var(--md-sys-color-secondary-container);
  }
  .chat {
    display: flex;
    flex-direction: column;
    gap: 28px;
    max-width: 820px;
    width: 100%;
    margin: 0 auto;
    min-height: 100%;
  }
  .intro {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 14px;
    margin-top: 8vh;
  }
  .mark {
    --m3e-shape-size: 72px;
    --m3e-shape-container-color: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
    --m3e-icon-size: 34px;
  }
  .mark m3e-shape {
    animation: spin 30s linear infinite;
  }
  @keyframes spin {
    to {
      rotate: 360deg;
    }
  }
  h1 {
    margin: 0;
    font-size: 52px;
    font-weight: 400;
    line-height: 1;
  }
  p {
    margin: 0;
    line-height: 1.55;
    color: var(--md-sys-color-on-surface-variant);
  }
  .setup-body {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 8px;
  }
  .starters {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
  }
  .starter {
    all: unset;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 14px 16px;
    border-radius: 20px;
    font-size: 14px;
    background: var(--md-sys-color-surface-container);
    transition:
      background 200ms ease,
      transform 200ms ease;
    --m3e-icon-size: 18px;
  }
  .starter m3e-icon {
    color: var(--md-sys-color-primary);
  }
  .starter:hover {
    background: var(--md-sys-color-surface-container-high);
    transform: translateY(-1px);
  }
  .starter:focus-visible {
    outline: 2px solid var(--md-sys-color-primary);
  }
  .thread {
    display: flex;
    flex-direction: column;
    gap: 26px;
    padding-top: 8px;
  }
  .you {
    align-self: flex-end;
    max-width: 75%;
    padding: 10px 16px;
    border-radius: 22px 22px 6px 22px;
    background: var(--md-sys-color-primary-container);
    color: var(--md-sys-color-on-primary-container);
    font-size: 15px;
    line-height: 1.5;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .composer {
    position: sticky;
    bottom: 0;
    margin-top: auto;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 16px 0 14px;
    background: linear-gradient(transparent, var(--md-sys-color-background) 30%);
  }
  .box {
    width: 100%;
    display: flex;
    align-items: flex-end;
    gap: 8px;
    padding: 8px 8px 8px 20px;
    border-radius: 28px;
    background: var(--md-sys-color-surface-container-high);
    box-sizing: border-box;
    transition: box-shadow 200ms ease;
  }
  .box:focus-within {
    box-shadow: 0 0 0 2px var(--md-sys-color-primary);
  }
  .box.disabled {
    opacity: 0.6;
  }
  textarea {
    flex: 1;
    min-height: 24px;
    max-height: 180px;
    field-sizing: content;
    resize: none;
    border: none;
    outline: none;
    background: none;
    color: var(--md-sys-color-on-surface);
    font: inherit;
    font-size: 15px;
    line-height: 1.5;
    padding: 8px 0;
  }
  textarea::placeholder {
    color: var(--md-sys-color-on-surface-variant);
  }
  .fine {
    font-size: 12px;
    color: var(--md-sys-color-on-surface-variant);
  }
  @media (max-width: 1100px) {
    .ani-page {
      grid-template-columns: minmax(0, 1fr);
    }
    .history {
      display: none;
    }
  }
</style>
