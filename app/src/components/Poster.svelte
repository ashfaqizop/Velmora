<script lang="ts">
  import type { TitleSummary } from "../lib/types";
  import { library, meta, nav } from "../lib/state.svelte";

  // `width` fixes the card size in rails; grids leave it unset and stretch.
  let { title, width }: { title: TitleSummary; width?: number } = $props();

  let failed = $state(false);
  const initials = $derived(
    title.name
      .split(/\s+/)
      .slice(0, 2)
      .map((w) => w[0])
      .join("")
      .toUpperCase(),
  );
  const saved = $derived(library.has(title));
  const open = () => nav.go({ name: "title", title });
</script>

<m3e-card actionable variant="filled" class="poster" style:width={width ? `${width}px` : undefined} onclick={open} title={title.name}>
  <div class="art">
    {#if title.poster && !failed}
      <img src={title.poster} alt="" loading="lazy" decoding="async" onerror={() => (failed = true)} />
    {:else}
      <div class="fallback">
        <span class="on-shape initials">
          <m3e-shape name="12-sided-cookie"></m3e-shape>
          <span>{initials}</span>
        </span>
      </div>
    {/if}
    {#if title.rating}
      <span class="rating"><m3e-icon name="star" filled></m3e-icon>{title.rating.toFixed(1)}</span>
    {/if}
    {#if saved}
      <span class="saved" aria-label="In My List"><m3e-icon name="bookmark" filled></m3e-icon></span>
    {/if}
  </div>
  <div class="info">
    <span class="name">{title.name}</span>
    <span class="meta">{meta(title)}</span>
  </div>
</m3e-card>

<style>
  .poster {
    --m3e-card-padding: 0;
    --m3e-card-shape: var(--md-sys-shape-corner-large, 16px);
    flex: none;
    background: transparent;
    overflow: hidden;
    border-radius: var(--md-sys-shape-corner-large, 16px);
  }
  .art {
    position: relative;
    aspect-ratio: 2 / 3;
    border-radius: var(--md-sys-shape-corner-large, 16px);
    overflow: hidden;
    background: var(--md-sys-color-surface-container-high);
  }
  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
    transition: transform var(--md-sys-motion-spring-default-spatial, 500ms ease);
  }
  .poster:hover img {
    transform: scale(1.05);
  }
  .fallback {
    height: 100%;
    display: grid;
    place-items: center;
    background: linear-gradient(160deg, var(--md-sys-color-primary-container), var(--md-sys-color-surface-container-high));
  }
  .initials {
    --m3e-shape-size: 72px;
    --m3e-shape-container-color: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
    font-weight: 700;
    font-size: 20px;
  }
  .rating {
    position: absolute;
    top: 8px;
    left: 8px;
    display: inline-flex;
    align-items: center;
    gap: 2px;
    padding: 2px 8px 2px 6px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 600;
    color: #fff;
    background: rgb(0 0 0 / 55%);
    backdrop-filter: blur(8px);
    --m3e-icon-size: 14px;
  }
  .rating m3e-icon {
    color: #ffc94d;
  }
  .saved {
    position: absolute;
    top: 6px;
    right: 6px;
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
    --m3e-icon-size: 16px;
  }
  .info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 10px 4px 4px;
  }
  .name {
    font-size: 14px;
    font-weight: 500;
    color: var(--md-sys-color-on-surface);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .meta {
    font-size: 12px;
    color: var(--md-sys-color-on-surface-variant);
  }
</style>
