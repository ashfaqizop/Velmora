<script lang="ts">
  // Find people by username or display name.
  import { untrack } from "svelte";
  import { fly } from "svelte/transition";
  import PersonRow from "./PersonRow.svelte";
  import { searchPeople, type Person } from "./data.svelte";
  import { go } from "./nav.svelte";

  let { q }: { q: string } = $props();

  let text = $state(untrack(() => q));
  let results = $state<Person[]>([]);
  let phase = $state<"idle" | "searching" | "done" | "error">("idle");
  let problem = $state("");
  let input = $state<HTMLInputElement>();
  let run = 0;

  $effect(() => {
    input?.focus();
  });

  $effect(() => {
    const term = text.trim();
    const mine = ++run;
    if (!term) {
      results = [];
      phase = "idle";
      return;
    }
    phase = "searching";
    const timer = setTimeout(async () => {
      go(`/search?q=${encodeURIComponent(term)}`, { replace: true, keepScroll: true });
      try {
        const found = await searchPeople(term);
        if (mine !== run) return;
        results = found;
        phase = "done";
      } catch (e) {
        if (mine !== run) return;
        problem = (e as Error).message;
        phase = "error";
      }
    }, 280);
    return () => clearTimeout(timer);
  });
</script>

<section class="search">
  <m3e-heading variant="display" size="small" level={1}>Find people</m3e-heading>
  <m3e-search-bar class="bar" clearable onclear={() => (text = "")}>
    <m3e-icon slot="leading" name="search"></m3e-icon>
    <input slot="input" bind:this={input} bind:value={text} type="search" placeholder="Search by name or @username" aria-label="Search people" autocomplete="off" spellcheck="false" />
    <span slot="trailing" class="spin">{#if phase === "searching"}<m3e-loading-indicator></m3e-loading-indicator>{/if}</span>
  </m3e-search-bar>

  <div class="results">
    {#if phase === "idle"}
      <p class="hint">Type a name or a username. Velmora shows exact matches first, then the most followed.</p>
    {:else if phase === "error"}
      <p class="hint" role="alert">{problem}</p>
    {:else if phase === "done" && !results.length}
      <p class="hint">No one matches “{text.trim()}”.</p>
    {:else}
      {#each results as p, i (p.id)}
        <div in:fly={{ y: 8, duration: 220, delay: Math.min(i, 10) * 25 }}><PersonRow person={p} /></div>
      {/each}
    {/if}
  </div>
</section>

<style>
  .search {
    display: flex;
    flex-direction: column;
    gap: 22px;
  }
  .bar {
    --m3e-search-bar-container-height: 60px;
  }
  .bar input {
    font-size: 17px;
  }
  .spin {
    display: inline-grid;
    width: 32px;
    --m3e-loading-indicator-size: 28px;
  }
  .results {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .hint {
    margin: 0;
    padding: 24px 12px;
    text-align: center;
    color: var(--md-sys-color-on-surface-variant);
  }
</style>
