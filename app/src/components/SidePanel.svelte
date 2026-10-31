<script lang="ts">
  import type { TitleSummary } from "../lib/types";
  import { feed, keyOf, library, meta, nav } from "../lib/state.svelte";

  // One pick per day, stable across launches.
  const pick = $derived.by(() => {
    const pool = feed.rails.flatMap((r) => r.items).filter((i) => i.backdrop && (i.rating ?? 0) >= 7);
    if (!pool.length) return null;
    const day = Math.floor(Date.now() / 86_400_000);
    return pool[day % pool.length];
  });

  const fresh = $derived((feed.rail("hdhub_latest") ?? feed.rail("airing_today"))?.items.slice(0, 5) ?? []);
  const freshTitle = $derived(feed.rail("hdhub_latest") ? "Just added" : "Airing today");
  const saved = $derived(library.watchlist.slice(0, 5));

  const open = (title: TitleSummary) => nav.go({ name: "title", title });
</script>

<aside class="panel">
  {#if pick}
    <m3e-card variant="filled" class="pick" style:--art={`url("${pick.backdrop}")`}>
      <div class="pick-body">
        <span class="overline"><m3e-icon name="auto_awesome" filled></m3e-icon>Tonight's pick</span>
        <strong class="display">{pick.name}</strong>
        <span class="meta">{meta(pick)}</span>
        <m3e-button variant="filled" size="small" onclick={() => open(pick)}>
          <m3e-icon slot="icon" name="play_arrow" filled></m3e-icon>
          Watch
        </m3e-button>
      </div>
    </m3e-card>
  {:else if feed.loading}
    <m3e-skeleton animation="wave" shape="rounded" class="pick-ghost"></m3e-skeleton>
  {/if}

  <section>
    <header>
      <m3e-icon name="bookmarks" filled></m3e-icon>
      <m3e-heading variant="title" size="medium" level={2}>My List</m3e-heading>
      {#if library.watchlist.length}
        <m3e-button variant="text" size="extra-small" onclick={() => nav.go({ name: "mylist" })}>See all</m3e-button>
      {/if}
    </header>
    {#if saved.length}
      <m3e-list>
        {#each saved as t (keyOf(t))}
          {@render row(t, true)}
        {/each}
      </m3e-list>
    {:else}
      <p class="empty">Titles you save show up here.</p>
    {/if}
  </section>

  {#if fresh.length}
    <section>
      <header>
        <m3e-icon name="new_releases" filled></m3e-icon>
        <m3e-heading variant="title" size="medium" level={2}>{freshTitle}</m3e-heading>
        <m3e-button
          variant="text"
          size="extra-small"
          onclick={() => nav.go({ name: "collection", id: feed.rail("hdhub_latest") ? "hdhub_latest" : "airing_today" })}
        >
          See all
        </m3e-button>
      </header>
      <m3e-list>
        {#each fresh as t (keyOf(t))}
          {@render row(t, false)}
        {/each}
      </m3e-list>
    </section>
  {/if}
</aside>

{#snippet row(t: TitleSummary, inList: boolean)}
  <m3e-list-action onclick={() => open(t)}>
    <div slot="leading" class="thumb" style:background-image={t.poster ? `url("${t.poster}")` : undefined}>
      {#if !t.poster}<m3e-icon name="movie"></m3e-icon>{/if}
    </div>
    <span class="row-name">{t.name}</span>
    <span slot="supporting-text">{meta(t)}</span>
    <m3e-icon-button
      slot="trailing"
      size="small"
      aria-label={inList ? "Remove from My List" : library.has(t) ? "Saved" : "Add to My List"}
      onclick={(e: Event) => {
        e.stopPropagation();
        library.toggle(t);
      }}
    >
      <m3e-icon name={inList || library.has(t) ? "bookmark_remove" : "bookmark_add"}></m3e-icon>
    </m3e-icon-button>
  </m3e-list-action>
{/snippet}

<style>
  .panel {
    height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding: 4px 20px 20px 4px;
    box-sizing: border-box;
  }
  .pick {
    --m3e-card-padding: 0;
    --m3e-card-shape: var(--md-sys-shape-corner-extra-large, 28px);
    flex: none;
    overflow: hidden;
  }
  /* The art lives inside the card body; the card's own container colour
     would otherwise paint over anything set on the host. */
  .pick-body {
    border-radius: var(--md-sys-shape-corner-extra-large, 28px);
    background:
      linear-gradient(180deg, rgb(12 10 20 / 10%) 0%, rgb(12 10 20 / 55%) 45%, rgb(12 10 20 / 92%) 100%),
      var(--art) center 30% / cover no-repeat,
      var(--md-sys-color-surface-container-high);
    min-height: 220px;
    padding: 20px;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    align-items: flex-start;
    gap: 6px;
    color: #fff;
  }
  .overline {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--md-sys-color-primary);
    --m3e-icon-size: 16px;
  }
  .pick strong {
    font-size: 26px;
    line-height: 1.15;
  }
  .pick .meta {
    font-size: 13px;
    opacity: 0.8;
    margin-bottom: 8px;
  }
  .pick-ghost {
    height: 200px;
    flex: none;
  }
  section {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  header {
    display: flex;
    align-items: center;
    gap: 8px;
    --m3e-icon-size: 20px;
  }
  header m3e-icon {
    color: var(--md-sys-color-primary);
  }
  header m3e-button {
    margin-left: auto;
  }
  m3e-list {
    --m3e-list-item-leading-space: 0;
    --m3e-list-item-trailing-space: 0;
    min-width: 0;
  }
  m3e-list-action {
    min-width: 0;
  }
  .thumb {
    width: 44px;
    height: 60px;
    border-radius: 10px;
    background: var(--md-sys-color-surface-container-highest) center / cover;
    display: grid;
    place-items: center;
    color: var(--md-sys-color-on-surface-variant);
  }
  /* Long titles wrap to two lines instead of widening the row. */
  .row-name {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    overflow-wrap: anywhere;
  }
  .empty {
    margin: 4px 0 0;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
</style>
