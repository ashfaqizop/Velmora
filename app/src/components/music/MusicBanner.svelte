<script lang="ts">
  // The immersive header on album, artist and playlist pages: the artwork
  // fills a wide band that fades into the page, with the name in big serif.
  import type { Snippet } from "svelte";
  import { fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import Art from "./Art.svelte";
  import { art } from "../../lib/music/ui.svelte";
  import type { MusicThumb } from "../../lib/types";

  let {
    title,
    thumbs,
    kicker,
    meta,
    description,
    round = false,
    actions,
  }: {
    title: string;
    thumbs: MusicThumb[];
    kicker?: string;
    meta?: string;
    description?: string | null;
    round?: boolean;
    actions?: Snippet;
  } = $props();

  const bg = $derived(art(thumbs, 600));
  let open = $state(false);
</script>

<header class="banner" class:round>
  {#if bg}<div class="glow" style:background-image={`url("${bg}")`}></div>{/if}
  <div class="fade"></div>
  <div class="inner" in:fly={{ y: 16, duration: 480, easing: cubicOut }}>
    <div class="cover">
      <Art {thumbs} size={round ? 200 : 220} {round} alt={title} />
    </div>
    <div class="text">
      {#if kicker}<span class="kicker">{kicker}</span>{/if}
      <h1 class="display">{title}</h1>
      {#if meta}<span class="meta">{meta}</span>{/if}
      {#if description}
        <button class="desc" class:open onclick={() => (open = !open)}>{description}</button>
      {/if}
      {#if actions}<div class="actions">{@render actions()}</div>{/if}
    </div>
  </div>
</header>

<style>
  .banner {
    position: relative;
    margin: -4px 0 8px;
    border-radius: 32px;
    overflow: hidden;
    isolation: isolate;
  }
  .glow {
    position: absolute;
    inset: -60px;
    background-size: cover;
    background-position: center;
    filter: blur(60px) saturate(1.4);
    opacity: 0.55;
    z-index: -2;
    animation: drift 18s ease-in-out infinite alternate;
  }
  @keyframes drift {
    to {
      transform: scale(1.12) translate(2%, -2%);
    }
  }
  .fade {
    position: absolute;
    inset: 0;
    z-index: -1;
    background: linear-gradient(180deg, transparent 0%, color-mix(in srgb, var(--md-sys-color-background) 85%, transparent) 100%);
  }
  .inner {
    display: flex;
    align-items: flex-end;
    gap: 32px;
    padding: 40px 36px 32px;
  }
  .cover :global(.art) {
    box-shadow: 0 24px 60px rgb(0 0 0 / 45%);
    border-radius: 24px;
  }
  .round .cover :global(.art) {
    border-radius: 50%;
  }
  .text {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 0;
  }
  .kicker {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--md-sys-color-primary);
  }
  h1 {
    margin: 0;
    font-size: clamp(40px, 5vw, 72px);
    font-weight: 400;
    line-height: 1;
    text-wrap: balance;
  }
  .meta {
    color: var(--md-sys-color-on-surface-variant);
    font-size: 14px;
  }
  .desc {
    all: unset;
    cursor: pointer;
    max-width: 760px;
    font-size: 13.5px;
    line-height: 1.55;
    color: var(--md-sys-color-on-surface-variant);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .desc.open {
    -webkit-line-clamp: unset;
    line-clamp: unset;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    margin-top: 8px;
  }
</style>
