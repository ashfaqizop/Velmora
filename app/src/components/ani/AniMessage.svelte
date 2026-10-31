<script lang="ts">
  import { fade } from "svelte/transition";
  import AniSteps from "./AniSteps.svelte";
  import AniBlock from "./AniBlock.svelte";
  import { chats, type Turn } from "../../lib/ani/chat.svelte";
  import { personalize } from "../../lib/ani/personalize.svelte";
  import { renderMarkdown } from "../../lib/ani/markdown";

  // One reply from Ani: research steps, then text and UI blocks in order.
  let { turn, last = false }: { turn: Extract<Turn, { role: "ani" }>; last?: boolean } = $props();

  const working = $derived(last && chats.busy);
  const researching = $derived(working && turn.steps.some((s) => s.state === "running"));
  const empty = $derived(!turn.blocks.some((b) => b.kind !== "text" || b.text.trim()));

  function undoNote(id: string) {
    personalize.forgetNote(id);
    const b = turn.blocks.find((x) => x.kind === "note" && x.id === id);
    if (b && b.kind === "note") b.removed = true;
  }
</script>

<div class="msg">
  <span class="on-shape avatar" aria-hidden="true">
    <m3e-shape name="very-sunny" class:spin={working}></m3e-shape>
    <m3e-icon name="wand_stars" filled></m3e-icon>
  </span>
  <div class="body">
    <AniSteps steps={turn.steps} working={researching} />

    {#each turn.blocks as b, i (i)}
      {#if b.kind === "text"}
        {#if b.text.trim()}
          <div class="md">{@html renderMarkdown(b.text)}</div>
        {/if}
      {:else if b.kind === "ui"}
        <AniBlock name={b.name} args={b.args} refs={chats.book} />
      {:else if b.kind === "note"}
        <div class="note" class:removed={b.removed} in:fade={{ duration: 200 }}>
          <m3e-icon name={b.removed ? "bookmark_remove" : "bookmark_added"} filled></m3e-icon>
          <span>{b.removed ? "Forgot" : "Noted"}: {b.text}</span>
          {#if !b.removed && personalize.notes.some((n) => n.id === b.id)}
            <m3e-button variant="text" size="extra-small" onclick={() => undoNote(b.id)}>Undo</m3e-button>
          {/if}
        </div>
      {/if}
    {/each}

    {#if working && empty && !researching}
      <div class="typing" aria-label="Ani is writing"><span></span><span></span><span></span></div>
    {/if}

    {#if turn.error}
      <m3e-callout variant="error" class="error">
        <m3e-icon slot="icon" name="error"></m3e-icon>
        {turn.error}
        {#if last}
          <m3e-button variant="text" size="small" onclick={() => chats.retry()}>Try again</m3e-button>
        {/if}
      </m3e-callout>
    {:else if turn.stopped}
      <span class="stopped">Stopped.</span>
    {/if}
  </div>
</div>

<style>
  .msg {
    display: grid;
    grid-template-columns: 36px minmax(0, 1fr);
    gap: 14px;
    align-items: start;
  }
  .avatar {
    --m3e-shape-size: 36px;
    --m3e-shape-container-color: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
    --m3e-icon-size: 18px;
  }
  .avatar m3e-shape.spin {
    animation: spin 2.4s linear infinite;
  }
  @keyframes spin {
    to {
      rotate: 360deg;
    }
  }
  .body {
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-width: 0;
    padding-top: 6px;
  }
  .md {
    font-size: 15px;
    line-height: 1.6;
  }
  .md :global(p) {
    margin: 0 0 10px;
  }
  .md :global(p:last-child) {
    margin-bottom: 0;
  }
  .md :global(ul),
  .md :global(ol) {
    margin: 0 0 10px;
    padding-left: 22px;
  }
  .md :global(li) {
    margin: 3px 0;
  }
  .md :global(h4) {
    margin: 14px 0 6px;
    font-size: 15px;
    font-weight: 700;
  }
  .md :global(code) {
    font-family: ui-monospace, "Cascadia Mono", Consolas, monospace;
    font-size: 13px;
    padding: 1px 6px;
    border-radius: 6px;
    background: var(--md-sys-color-surface-container-high);
  }
  .note {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    align-self: flex-start;
    padding: 4px 6px 4px 12px;
    border-radius: 999px;
    font-size: 13px;
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
    --m3e-icon-size: 16px;
  }
  .note.removed {
    background: var(--md-sys-color-surface-container-high);
    color: var(--md-sys-color-on-surface-variant);
    padding-right: 12px;
  }
  .typing {
    display: inline-flex;
    gap: 5px;
    padding: 8px 0;
  }
  .typing span {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--md-sys-color-primary);
    animation: bounce 1.2s ease-in-out infinite;
  }
  .typing span:nth-child(2) {
    animation-delay: 0.15s;
  }
  .typing span:nth-child(3) {
    animation-delay: 0.3s;
  }
  @keyframes bounce {
    0%,
    80%,
    100% {
      opacity: 0.3;
      translate: 0 0;
    }
    40% {
      opacity: 1;
      translate: 0 -4px;
    }
  }
  .error {
    display: block;
  }
  .stopped {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
</style>
