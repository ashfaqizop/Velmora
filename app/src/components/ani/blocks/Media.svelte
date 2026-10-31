<script lang="ts">
  import { detailOf } from "../../../lib/ani/tools/shape";
  import type { RefBook } from "../../../lib/ani/refs";
  import type { TitleDetail } from "../../../lib/types";

  // render_trailer (thumbnail, then the player on click) and render_gallery
  // (a strip of stills that opens one large).
  let { args, refs, mode }: { args: Record<string, unknown>; refs: RefBook; mode: "trailer" | "gallery" } = $props();

  const t = $derived(refs.title(args.ref as string));
  let detail = $state<TitleDetail | null>(null);
  let playing = $state(false);
  let big = $state<string | null>(null);
  $effect(() => {
    if (t) detailOf(t).then((d) => (detail = d)).catch(() => {});
  });

  const videoId = $derived(detail?.trailer?.match(/(?:embed\/|v=|youtu\.be\/)([\w-]{6,})/)?.[1]);
  const embed = $derived(videoId ? `https://www.youtube-nocookie.com/embed/${videoId}?autoplay=1&rel=0` : null);
  const images = $derived([...(detail?.summary.backdrop ? [detail.summary.backdrop] : []), ...(detail?.screenshots ?? [])].slice(0, 16));
</script>

{#if t}
  {#if mode === "trailer"}
    <div class="trailer">
      {#if playing && embed}
        <iframe src={embed} title="{t.name} trailer" allow="autoplay; encrypted-media; fullscreen" allowfullscreen></iframe>
      {:else if videoId}
        <button class="thumb" onclick={() => (playing = true)} aria-label="Play the trailer">
          <img src={`https://i.ytimg.com/vi/${videoId}/hqdefault.jpg`} alt="" />
          <span class="on-shape play"><m3e-shape name="circle"></m3e-shape><m3e-icon name="play_arrow" filled></m3e-icon></span>
          <span class="cap">{t.name} · Trailer</span>
        </button>
      {:else if detail}
        <p class="none">No trailer for {t.name}.</p>
      {:else}
        <m3e-skeleton animation="wave" shape="rounded" class="ghost"></m3e-skeleton>
      {/if}
    </div>
  {:else}
    <div class="gallery">
      {#if big}
        <button class="big" onclick={() => (big = null)} aria-label="Close"><img src={big} alt="" /></button>
      {/if}
      <div class="strip">
        {#each images as src (src)}
          <button class="shot" class:on={big === src} onclick={() => (big = big === src ? null : src)}>
            <img {src} alt="" loading="lazy" />
          </button>
        {:else}
          {#if detail}<p class="none">No stills for {t.name}.</p>{:else}<m3e-skeleton animation="wave" shape="rounded" class="ghost"></m3e-skeleton>{/if}
        {/each}
      </div>
    </div>
  {/if}
{/if}

<style>
  .trailer {
    border-radius: 24px;
    overflow: hidden;
    aspect-ratio: 16 / 9;
    background: #000;
  }
  iframe {
    width: 100%;
    height: 100%;
    border: 0;
  }
  .thumb,
  .shot,
  .big {
    all: unset;
    cursor: pointer;
    display: block;
    position: relative;
  }
  .thumb {
    width: 100%;
    height: 100%;
  }
  .thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    filter: brightness(0.75);
    transition: filter 200ms ease;
  }
  .thumb:hover img {
    filter: brightness(0.9);
  }
  .play {
    position: absolute;
    inset: 0;
    margin: auto;
    width: 72px;
    height: 72px;
    --m3e-shape-size: 72px;
    --m3e-shape-container-color: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
    --m3e-icon-size: 36px;
  }
  .play m3e-icon {
    z-index: 1;
  }
  .cap {
    position: absolute;
    left: 18px;
    bottom: 14px;
    color: #fff;
    font-weight: 700;
    text-shadow: 0 1px 8px rgb(0 0 0 / 60%);
  }
  .gallery {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .big img {
    width: 100%;
    border-radius: 20px;
    display: block;
  }
  .strip {
    display: grid;
    grid-auto-flow: column;
    grid-auto-columns: 200px;
    gap: 10px;
    overflow-x: auto;
    scrollbar-width: thin;
    padding-bottom: 4px;
  }
  .shot {
    aspect-ratio: 16 / 9;
    border-radius: 14px;
    overflow: hidden;
    outline: 2px solid transparent;
    transition: outline-color 150ms ease;
  }
  .shot.on,
  .shot:focus-visible {
    outline-color: var(--md-sys-color-primary);
  }
  .shot img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .ghost {
    width: 100%;
    height: 100%;
    min-height: 120px;
  }
  .none {
    margin: 0;
    padding: 16px;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
</style>
