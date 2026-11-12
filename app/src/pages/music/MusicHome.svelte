<script lang="ts">
  // Music Home: a greeting, mood chips, your own rows (mixes, recently
  // played...), then YouTube Music's shelves, with more as you scroll.
  import { onMount } from "svelte";
  import { fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import * as api from "../../lib/api";
  import ShelfView from "../../components/music/ShelfView.svelte";
  import PersonalRows from "../../components/music/PersonalRows.svelte";
  import AdSlot from "../../components/AdSlot.svelte";
  import { greeting, openItem } from "../../lib/music/ui.svelte";
  import { musicHome } from "../../lib/music/feeds.svelte";
  import { musicSession } from "../../lib/music/session.svelte";
  import type { MusicItem } from "../../lib/types";

  let moods = $state<MusicItem[]>([]);
  let sentinel = $state<HTMLElement>();

  const first = $derived(musicSession.account?.name?.split(" ")[0]);

  onMount(() => {
    musicHome.load();
    api
      .musicFeed("FEmusic_moods_and_genres")
      .then((f) => (moods = f.shelves[0]?.items.slice(0, 12) ?? []))
      .catch(() => {});
    const io = new IntersectionObserver(([e]) => e.isIntersecting && musicHome.more(), { rootMargin: "600px" });
    if (sentinel) io.observe(sentinel);
    return () => io.disconnect();
  });
</script>

<div class="home">
  <header in:fly={{ y: 14, duration: 520, delay: 120, easing: cubicOut }}>
    <h1 class="display">{greeting()}{first ? `, ${first}` : ""}</h1>
    {#if moods.length}
      <div class="moods">
        {#each moods as m, i (m.id + (m.params ?? ""))}
          <button class="mood" style:--tint={m.tint ?? "var(--md-sys-color-primary)"} style:--i={i} onclick={() => openItem(m)}>
            {m.title}
          </button>
        {/each}
      </div>
    {/if}
  </header>

  {#if musicHome.loading && !musicHome.shelves.length}
    {#each Array(3) as _, i (i)}
      <div class="ghost">
        <m3e-skeleton animation="wave" shape="rounded" class="g-title"></m3e-skeleton>
        <div class="g-row">
          {#each Array(6) as __, j (j)}<m3e-skeleton animation="wave" shape="rounded" class="g-card"></m3e-skeleton>{/each}
        </div>
      </div>
    {/each}
  {:else if musicHome.error && !musicHome.shelves.length}
    <div class="empty">
      <m3e-icon name="cloud_off"></m3e-icon>
      <p>{musicHome.error}</p>
      <m3e-button variant="tonal" onclick={() => musicHome.load(true)}>Try again</m3e-button>
    </div>
  {:else}
    <!-- Ours first: built from this PC's own listening. -->
    <PersonalRows />
    {#each musicHome.shelves as shelf, i (shelf.id)}
      <ShelfView {shelf} />
      {#if i === 2}<AdSlot id="music" />{/if}
    {/each}
  {/if}
  <div bind:this={sentinel} class="sentinel">
    {#if musicHome.loadingMore}<m3e-loading-indicator></m3e-loading-indicator>{/if}
  </div>
</div>

<style>
  .home {
    display: flex;
    flex-direction: column;
    gap: 36px;
    padding-bottom: 120px;
  }
  header {
    display: flex;
    flex-direction: column;
    gap: 18px;
    padding-top: 8px;
  }
  h1 {
    margin: 0;
    font-size: clamp(40px, 4.6vw, 64px);
    font-weight: 400;
    line-height: 1;
  }
  .moods {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .mood {
    all: unset;
    cursor: pointer;
    padding: 9px 16px;
    border-radius: 12px;
    font-size: 14px;
    font-weight: 600;
    background: color-mix(in srgb, var(--tint) 22%, var(--md-sys-color-surface-container));
    border: 1px solid color-mix(in srgb, var(--tint) 40%, transparent);
    animation: pop 480ms cubic-bezier(0.2, 0, 0, 1) both;
    animation-delay: calc(var(--i) * 30ms + 200ms);
    transition:
      border-radius 260ms ease,
      background 200ms ease;
  }
  .mood:hover {
    border-radius: 20px;
    background: color-mix(in srgb, var(--tint) 34%, var(--md-sys-color-surface-container));
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: translateY(8px) scale(0.96);
    }
  }
  .ghost {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .g-title {
    width: 220px;
    height: 28px;
  }
  .g-row {
    display: flex;
    gap: 18px;
    overflow: hidden;
  }
  .g-card {
    width: 168px;
    height: 168px;
    flex: none;
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
