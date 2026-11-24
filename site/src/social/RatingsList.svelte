<script lang="ts">
  // Someone's ratings and reviews, newest first.
  import { fly } from "svelte/transition";
  import Stars from "./Stars.svelte";
  import { ago } from "./format";
  import { titles } from "./meta.svelte";
  import { titleHref } from "./nav.svelte";
  import { userRatings, type Rating } from "./activity.svelte";

  let { userId, empty }: { userId: string; empty: string } = $props();

  let items = $state<Rating[] | null>(null);
  let more = $state(false);
  let revealed = $state(new Set<string>());

  $effect(() => {
    const id = userId;
    items = null;
    userRatings(id, 0)
      .then((r) => {
        items = r.items;
        more = r.more;
      })
      .catch(() => (items = []));
  });

  async function next() {
    const r = await userRatings(userId, items!.length);
    items = [...items!, ...r.items];
    more = r.more;
  }
</script>

{#if items === null}
  <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
{:else if !items.length}
  <p class="empty">{empty}</p>
{:else}
  <div class="list">
    {#each items as r, i (r.k)}
      {@const meta = titles.get(r.k)}
      <a class="row" href={titleHref(r.k)} in:fly={{ y: 8, duration: 240, delay: Math.min(i, 10) * 30 }}>
        <span class="thumb">{#if meta?.poster}<img src={meta.poster} alt="" loading="lazy" />{/if}</span>
        <span class="body">
          <strong>{meta?.name ?? "…"}{#if meta?.year}<span class="yr"> · {meta.year}</span>{/if}</strong>
          <span class="stars"><Stars value={r.score} size={15} /><span>{ago(r.updated_at)}</span></span>
          {#if r.review}
            {#if r.spoiler && !revealed.has(r.k)}
              <button type="button" class="spoiler" onclick={(e) => (e.preventDefault(), (revealed = new Set([...revealed, r.k])))}>Contains spoilers. Show the review</button>
            {:else}
              <span class="review">{r.review}</span>
            {/if}
          {/if}
        </span>
      </a>
    {/each}
  </div>
  {#if more}<div class="more"><m3e-button variant="tonal" onclick={next}>Show more</m3e-button></div>{/if}
{/if}

<style>
  .list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .row {
    display: flex;
    gap: 14px;
    padding: 10px 12px;
    border-radius: 20px;
    color: inherit;
    text-decoration: none;
    transition: background-color 160ms ease;
  }
  .row:hover {
    background: color-mix(in srgb, var(--md-sys-color-on-surface) 4%, transparent);
  }
  .thumb {
    flex: none;
    width: 52px;
    height: 78px;
    border-radius: 10px;
    overflow: hidden;
    background: var(--md-sys-color-surface-container-highest);
  }
  .thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .body {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .yr,
  .stars span {
    font-weight: 400;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .stars {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .review {
    line-height: 1.55;
    white-space: pre-line;
    overflow-wrap: anywhere;
  }
  .spoiler {
    align-self: flex-start;
    padding: 6px 12px;
    border: 1px dashed var(--md-sys-color-outline);
    border-radius: 12px;
    background: none;
    color: var(--md-sys-color-on-surface-variant);
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }
  .center {
    display: grid;
    place-items: center;
    padding: 32px 0;
  }
  .empty {
    margin: 0;
    padding: 28px 12px;
    text-align: center;
    color: var(--md-sys-color-on-surface-variant);
  }
  .more {
    display: flex;
    justify-content: center;
    padding-top: 10px;
  }
</style>
