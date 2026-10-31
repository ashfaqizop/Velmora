<script lang="ts">
  import type { TitleSummary } from "../../../lib/types";

  // Poster or backdrop for Ani's cards, with initials when the art is missing.
  let { title, wide = false, onclick }: { title: TitleSummary; wide?: boolean; onclick?: () => void } = $props();

  let failed = $state(false);
  const src = $derived(wide ? (title.backdrop ?? title.poster) : (title.poster ?? title.backdrop));
  const initials = $derived(
    title.name
      .split(/\s+/)
      .slice(0, 2)
      .map((w) => w[0])
      .join("")
      .toUpperCase(),
  );
</script>

<button class="art" class:wide {onclick} aria-label="Open {title.name}" tabindex={onclick ? 0 : -1}>
  {#if src && !failed}
    <img {src} alt="" loading="lazy" decoding="async" onerror={() => (failed = true)} />
  {:else}
    <span class="fallback on-shape">
      <m3e-shape name="12-sided-cookie"></m3e-shape>
      <span>{initials}</span>
    </span>
  {/if}
</button>

<style>
  .art {
    all: unset;
    position: relative;
    display: grid;
    place-items: center;
    width: 100%;
    aspect-ratio: 2 / 3;
    border-radius: 16px;
    overflow: hidden;
    background: var(--md-sys-color-surface-container-highest);
    cursor: pointer;
    transition: transform 250ms var(--md-sys-motion-easing-emphasized, ease);
  }
  .art.wide {
    aspect-ratio: 16 / 9;
  }
  .art:hover {
    transform: scale(1.03);
  }
  .art:focus-visible {
    outline: 2px solid var(--md-sys-color-primary);
    outline-offset: 2px;
  }
  img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .fallback {
    --m3e-shape-size: 52px;
    --m3e-shape-container-color: var(--md-sys-color-primary-container);
    color: var(--md-sys-color-on-primary-container);
    font-weight: 700;
  }
  .fallback span {
    z-index: 1;
  }
</style>
