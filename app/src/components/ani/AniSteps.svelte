<script lang="ts">
  import { slide } from "svelte/transition";
  import type { Step } from "../../lib/ani/agent";

  // What the research stage looked up. Open while it runs, then folds into
  // a one-line summary that can be expanded.
  let { steps, working }: { steps: Step[]; working: boolean } = $props();

  let expanded = $state(false);
  const open = $derived(working || expanded);
  const done = $derived(steps.filter((s) => s.state === "done").length);
</script>

{#if steps.length}
  <div class="steps">
    <button class="head" onclick={() => (expanded = !expanded)} disabled={working} aria-expanded={open}>
      {#if working}
        <m3e-loading-indicator class="spinner"></m3e-loading-indicator>
        <span>Looking things up</span>
      {:else}
        <m3e-icon name="travel_explore"></m3e-icon>
        <span>Looked up {done} thing{done === 1 ? "" : "s"}</span>
        <m3e-icon class="chev" class:up={expanded} name="expand_more"></m3e-icon>
      {/if}
    </button>
    {#if open}
      <ul transition:slide={{ duration: 220 }}>
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

<style>
  .steps {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
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
  .head:hover:not(:disabled) {
    background: var(--md-sys-color-surface-container-high);
  }
  .head:disabled {
    cursor: default;
  }
  .spinner {
    --m3e-loading-indicator-size: 24px;
  }
  .chev {
    transition: rotate 200ms ease;
  }
  .chev.up {
    rotate: 180deg;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0 0 0 10px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    border-left: 2px solid var(--md-sys-color-outline-variant);
  }
  li {
    display: flex;
    align-items: center;
    gap: 8px;
    padding-left: 8px;
    --m3e-icon-size: 16px;
  }
  li.running m3e-icon {
    animation: spin 1s linear infinite;
  }
  li.done m3e-icon {
    color: var(--md-sys-color-primary);
  }
  li.failed {
    opacity: 0.7;
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
