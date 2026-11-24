<script lang="ts">
  // A title's poster linking to its page here, with its name under it.
  import { titles } from "./meta.svelte";
  import { titleHref } from "./nav.svelte";

  let { k, rank = 0, note = "", i = 0 }: { k: string; rank?: number; note?: string; i?: number } = $props();

  const meta = $derived(titles.get(k));
</script>

{#if meta !== null}
  <a class="tile" href={titleHref(k)} style:--i={i} title={meta?.name}>
    <span class="poster">
      {#if meta?.poster}<img src={meta.poster} alt="" loading="lazy" decoding="async" />{:else if meta === undefined}<span class="shimmer"></span>{:else}<m3e-icon name="movie"></m3e-icon>{/if}
      {#if rank}<span class="rank">{rank}</span>{/if}
    </span>
    <span class="name">{meta?.name ?? " "}</span>
    {#if note || meta?.year}<span class="note">{note || meta?.year}</span>{/if}
  </a>
{/if}

<style>
  .tile {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
    color: inherit;
    text-decoration: none;
    animation: rise 460ms cubic-bezier(0.2, 0.8, 0.2, 1) both;
    animation-delay: calc(min(var(--i), 16) * 35ms);
  }
  .poster {
    position: relative;
    display: grid;
    place-items: center;
    aspect-ratio: 2 / 3;
    border-radius: 18px;
    overflow: hidden;
    background: var(--md-sys-color-surface-container-highest);
    box-shadow: 0 14px 30px -18px rgb(0 0 0 / 80%);
    transition:
      transform 320ms cubic-bezier(0.2, 0.8, 0.2, 1),
      border-radius 320ms ease;
    color: var(--md-sys-color-on-surface-variant);
    --m3e-icon-size: 36px;
  }
  .poster img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .shimmer {
    position: absolute;
    inset: 0;
    background: linear-gradient(100deg, transparent 30%, color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent) 50%, transparent 70%);
    background-size: 200% 100%;
    animation: shimmer 1.4s linear infinite;
  }
  .rank {
    position: absolute;
    left: 8px;
    bottom: 4px;
    font-family: var(--display);
    font-size: 52px;
    line-height: 1;
    color: #fff;
    text-shadow: 0 4px 18px rgb(0 0 0 / 80%);
  }
  .tile:hover .poster,
  .tile:focus-visible .poster {
    transform: translateY(-4px);
    border-radius: 26px;
  }
  .name {
    font-weight: 600;
    font-size: 14px;
    line-height: 1.3;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .note {
    margin-top: -4px;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(12px);
    }
  }
  @keyframes shimmer {
    to {
      background-position: -200% 0;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .tile,
    .shimmer {
      animation: none;
    }
  }
</style>
