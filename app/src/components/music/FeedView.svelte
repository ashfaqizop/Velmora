<script lang="ts">
  // A page of shelves from YouTube Music: charts, new releases, a mood, or a
  // shelf's "More". Loads more as you scroll.
  import { onMount } from "svelte";
  import * as api from "../../lib/api";
  import ShelfView from "./ShelfView.svelte";
  import AdSlot from "../AdSlot.svelte";
  import type { MusicShelf } from "../../lib/types";

  let { browseId, params = null, title = null }: { browseId: string; params?: string | null; title?: string | null } = $props();

  let shelves = $state<MusicShelf[]>([]);
  let heading = $state<string | null>(null);
  let next: string | null = null;
  let loading = $state(true);
  let more = $state(false);
  let error = $state<string | null>(null);
  let sentinel = $state<HTMLElement>();

  async function load() {
    loading = true;
    error = null;
    try {
      const page = await api.musicFeed(browseId, params);
      shelves = page.shelves;
      heading = page.title;
      next = page.next;
    } catch (e) {
      error = typeof e === "string" ? e : "YouTube Music didn't answer.";
    } finally {
      loading = false;
    }
  }

  async function loadMore() {
    if (!next || more || loading) return;
    more = true;
    try {
      const page = await api.musicFeed(browseId, params, next);
      shelves = [...shelves, ...page.shelves];
      next = page.next;
    } catch {
      next = null;
    } finally {
      more = false;
    }
  }

  onMount(() => {
    load();
    const io = new IntersectionObserver(([e]) => e.isIntersecting && loadMore(), { rootMargin: "600px" });
    if (sentinel) io.observe(sentinel);
    return () => io.disconnect();
  });
</script>

<div class="feed">
  {#if title || heading}<h1 class="display">{title ?? heading}</h1>{/if}
  {#if loading}
    <m3e-loading-indicator class="spin"></m3e-loading-indicator>
  {:else if error}
    <div class="empty">
      <m3e-icon name="cloud_off"></m3e-icon>
      <p>{error}</p>
      <m3e-button variant="tonal" onclick={load}>Try again</m3e-button>
    </div>
  {:else if !shelves.length}
    <div class="empty"><p>Nothing here right now.</p></div>
  {:else}
    {#each shelves as shelf, i (shelf.id + i)}
      <ShelfView {shelf} />
      {#if i === 1}<AdSlot id="music" />{/if}
    {/each}
  {/if}
  <div bind:this={sentinel} class="sentinel">{#if more}<m3e-loading-indicator></m3e-loading-indicator>{/if}</div>
</div>

<style>
  .feed {
    display: flex;
    flex-direction: column;
    gap: 34px;
    padding-bottom: 120px;
  }
  h1 {
    margin: 8px 0 0;
    font-size: clamp(36px, 4vw, 56px);
    font-weight: 400;
  }
  .spin {
    align-self: center;
    margin: 60px 0;
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 60px 0;
    color: var(--md-sys-color-on-surface-variant);
  }
  .sentinel {
    display: grid;
    place-items: center;
    min-height: 40px;
  }
</style>
