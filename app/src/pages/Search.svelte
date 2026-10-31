<script lang="ts">
  import Poster from "../components/Poster.svelte";
  import * as api from "../lib/api";
  import type { TitleSummary } from "../lib/types";
  import { keyOf } from "../lib/state.svelte";
  import { providerLabel } from "../lib/options";

  let { query }: { query: string } = $props();

  type ProviderState = { id: string; count: number; error?: string };

  let results = $state<TitleSummary[]>([]);
  let providers = $state<ProviderState[]>([]);
  let running = $state(false);
  let run = 0;
  let attempt = $state(0);

  $effect(() => {
    const q = query;
    void attempt;
    const mine = ++run;
    results = [];
    providers = [];
    running = true;
    api
      .searchStream(q, (batch) => {
        if (mine !== run) return; // a newer query took over
        providers = [...providers, { id: batch.provider, count: batch.items.length, error: batch.error }];
        const seen = new Set(results.map(keyOf));
        results = [...results, ...batch.items.filter((i) => !seen.has(keyOf(i)))];
      })
      .catch((e) => console.warn("search failed", e))
      .finally(() => {
        if (mine === run) running = false;
      });
  });

  const label = providerLabel;
</script>

<div class="page">
  <header>
    <div class="title">
      <m3e-heading variant="headline" size="large" level={1}>Results for “{query}”</m3e-heading>
      {#if running}<m3e-loading-indicator></m3e-loading-indicator>{/if}
    </div>
    <m3e-chip-set aria-label="Sources">
      {#each providers as p (p.id)}
        <m3e-chip class:error={!!p.error} title={p.error ?? `${p.count} results`}>
          <m3e-icon slot="icon" name={p.error ? "error" : p.count ? "check_circle" : "remove_circle_outline"}></m3e-icon>
          {label(p.id)}{p.count ? ` · ${p.count}` : ""}
        </m3e-chip>
      {/each}
    </m3e-chip-set>
  </header>

  {#if results.length}
    <div class="grid">
      {#each results as t (keyOf(t))}
        <Poster title={t} />
      {/each}
    </div>
  {:else if running}
    <div class="grid">
      {#each Array(12) as _, i (i)}
        <m3e-skeleton animation="wave" shape="rounded" class="ghost"></m3e-skeleton>
      {/each}
    </div>
  {:else if providers.length && providers.every((p) => p.error)}
    <div class="empty">
      <m3e-icon name="cloud_off"></m3e-icon>
      <m3e-heading variant="title" size="large">The sources didn't answer</m3e-heading>
      <p>Check your connection, then try again.</p>
      <m3e-button variant="tonal" onclick={() => attempt++}>
        <m3e-icon slot="icon" name="refresh"></m3e-icon>Try again
      </m3e-button>
    </div>
  {:else}
    <div class="empty">
      <m3e-icon name="search_off"></m3e-icon>
      <m3e-heading variant="title" size="large">No matches</m3e-heading>
      <p>Try the original title, or fewer words.</p>
    </div>
  {/if}
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding-bottom: 32px;
  }
  header {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .title {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .title m3e-loading-indicator {
    --m3e-loading-indicator-container-size: 36px;
    --m3e-loading-indicator-active-indicator-size: 28px;
  }
  m3e-chip {
    --m3e-icon-size: 18px;
  }
  m3e-chip.error {
    color: var(--md-sys-color-error);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 20px 16px;
  }
  .ghost {
    aspect-ratio: 2 / 3;
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 64px 0;
    color: var(--md-sys-color-on-surface-variant);
    --m3e-icon-size: 48px;
  }
  .empty p {
    margin: 0;
  }
</style>
