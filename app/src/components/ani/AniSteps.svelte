<script lang="ts">
  import { slide } from "svelte/transition";
  import type { Phase, Step } from "../../lib/ani/agent";
  import { renderMarkdown } from "../../lib/ani/markdown";

  // What Ani is doing, live: a status line, its thinking as Gemini streams it,
  // and the lookups. Once the reply is done it folds into one summary line.
  let {
    steps,
    thoughts = "",
    phase,
    thinkingMs,
    working,
  }: { steps: Step[]; thoughts?: string; phase?: Phase; thinkingMs?: number; working: boolean } = $props();

  let expanded = $state(false);
  let box = $state<HTMLElement>();

  const LABEL: Record<Phase, string> = {
    thinking: "Thinking",
    researching: "Looking things up",
    planning: "Planning the answer",
    writing: "Writing",
  };

  const running = $derived(steps.find((s) => s.state === "running"));
  const done = $derived(steps.filter((s) => s.state === "done").length);
  const status = $derived(phase === "researching" && running ? running.label : phase ? LABEL[phase] : "");
  // Live while thinking or looking things up; the answer takes over once it starts.
  const live = $derived(working && phase !== "writing");
  const open = $derived(live || expanded);
  const hasDetail = $derived(!!thoughts.trim() || steps.length > 0);

  const summary = $derived.by(() => {
    const parts: string[] = [];
    if (thinkingMs !== undefined && thoughts.trim()) parts.push(`Thought for ${Math.max(1, Math.round(thinkingMs / 1000))}s`);
    if (steps.length) parts.push(`Looked up ${done} thing${done === 1 ? "" : "s"}`);
    return parts.join(" · ");
  });

  // Keep the newest thinking in view while it streams.
  $effect(() => {
    void thoughts;
    if (live && box) box.scrollTop = box.scrollHeight;
  });
</script>

{#if working || summary}
  <div class="activity">
    {#if working && phase}
      <div class="status" aria-live="polite">
        <m3e-loading-indicator class="spinner"></m3e-loading-indicator>
        {#key status}<span class="shimmer">{status}</span>{/key}
      </div>
    {:else if summary && hasDetail}
      <button class="head" onclick={() => (expanded = !expanded)} aria-expanded={expanded}>
        <m3e-icon name={thoughts.trim() ? "psychology" : "travel_explore"}></m3e-icon>
        <span>{summary}</span>
        <m3e-icon class="chev" class:up={expanded} name="expand_more"></m3e-icon>
      </button>
    {/if}

    {#if open && hasDetail}
      <div class="detail" transition:slide={{ duration: 220 }}>
        {#if thoughts.trim()}
          <div class="thoughts" class:live bind:this={box}>{@html renderMarkdown(thoughts)}</div>
        {/if}
        {#if steps.length}
          <ul>
            {#each steps as s, i (i)}
              <li class={s.state}>
                <m3e-icon name={s.state === "running" ? "progress_activity" : s.state === "done" ? "check" : "close"}></m3e-icon>
                {s.label}
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    {/if}
  </div>
{/if}

<style>
  .activity {
    display: flex;
    flex-direction: column;
    gap: 8px;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .status {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-height: 28px;
  }
  .spinner {
    --m3e-loading-indicator-size: 24px;
  }
  /* A light sweep across the status text while Ani works. */
  .shimmer {
    font-weight: 600;
    background: linear-gradient(
      90deg,
      var(--md-sys-color-on-surface-variant) 0%,
      var(--md-sys-color-on-surface) 45%,
      var(--md-sys-color-on-surface-variant) 90%
    );
    background-size: 200% 100%;
    background-clip: text;
    -webkit-background-clip: text;
    color: transparent;
    animation: sweep 1.8s linear infinite;
  }
  @keyframes sweep {
    from {
      background-position: 100% 0;
    }
    to {
      background-position: -100% 0;
    }
  }
  .head {
    all: unset;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    align-self: flex-start;
    padding: 4px 10px 4px 6px;
    border-radius: 999px;
    --m3e-icon-size: 18px;
  }
  .head:hover {
    background: var(--md-sys-color-surface-container-high);
  }
  .head:focus-visible {
    outline: 2px solid var(--md-sys-color-primary);
  }
  .chev {
    transition: rotate 200ms ease;
  }
  .chev.up {
    rotate: 180deg;
  }
  .detail {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding-left: 12px;
    border-left: 2px solid var(--md-sys-color-outline-variant);
  }
  .thoughts {
    font-size: 13px;
    line-height: 1.55;
    max-height: 280px;
    overflow-y: auto;
    scrollbar-width: thin;
  }
  .thoughts.live {
    max-height: 132px;
    mask-image: linear-gradient(transparent, #000 28px);
  }
  .thoughts :global(p) {
    margin: 0 0 8px;
  }
  .thoughts :global(strong) {
    color: var(--md-sys-color-on-surface);
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  li {
    display: flex;
    align-items: center;
    gap: 8px;
    --m3e-icon-size: 16px;
  }
  li.running m3e-icon {
    animation: spin 1s linear infinite;
  }
  li.done m3e-icon {
    color: var(--md-sys-color-primary);
  }
  li.failed {
    opacity: 0.75;
  }
  li.failed m3e-icon {
    color: var(--md-sys-color-error);
  }
  @keyframes spin {
    to {
      rotate: 360deg;
    }
  }
</style>
