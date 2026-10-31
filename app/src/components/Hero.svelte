<script lang="ts">
  import { onMount } from "svelte";
  import { fade, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import type { TitleSummary } from "../lib/types";
  import { kindLabel, library, nav } from "../lib/state.svelte";

  let { items, loading = false }: { items: TitleSummary[]; loading?: boolean } = $props();

  let index = $state(0);
  let paused = $state(false);
  const current = $derived(items[index % Math.max(items.length, 1)]);
  const saved = $derived(current ? library.has(current) : false);

  const go = (delta: number) => (index = (index + delta + items.length) % items.length);

  onMount(() => {
    const timer = setInterval(() => {
      if (!paused && items.length > 1 && !document.hidden) go(1);
    }, 8000);
    return () => clearInterval(timer);
  });
</script>

{#if loading || !current}
  <m3e-skeleton animation="wave" shape="rounded" class="hero ghost"></m3e-skeleton>
{:else}
  <section
    class="hero"
    aria-roledescription="carousel"
    aria-label="Trending now"
    onmouseenter={() => (paused = true)}
    onmouseleave={() => (paused = false)}
  >
    {#key current}
      <div
        class="backdrop"
        style:background-image={`url("${current.backdrop ?? current.poster}")`}
        in:fade={{ duration: 700 }}
        out:fade={{ duration: 700 }}
      ></div>
    {/key}
    <div class="scrim"></div>

    {#key current}
      <div class="content" in:fly={{ x: -24, duration: 650, delay: 150, easing: cubicOut }} out:fade={{ duration: 200 }}>
        <m3e-chip class="badge"><m3e-icon slot="icon" name="local_fire_department" filled></m3e-icon>Trending now</m3e-chip>
        <h1 class="display title">{current.name}</h1>
        <div class="meta">
          {#if current.rating}<span class="star"><m3e-icon name="star" filled></m3e-icon>{current.rating.toFixed(1)}</span>{/if}
          {#if current.year}<span>{current.year}</span>{/if}
          <span>{kindLabel[current.kind]}</span>
          {#each current.badges ?? [] as b (b)}<span class="tag">{b}</span>{/each}
        </div>
        {#if current.overview}<p class="overview">{current.overview}</p>{/if}
        <div class="actions">
          <m3e-button variant="filled" size="medium" onclick={() => nav.go({ name: "title", title: current })}>
            <m3e-icon slot="icon" name="play_arrow" filled></m3e-icon>
            Watch now
          </m3e-button>
          <m3e-button variant="tonal" size="medium" toggle selected={saved} onclick={() => library.toggle(current)}>
            <m3e-icon slot="icon" name="add"></m3e-icon>
            <m3e-icon slot="selected-icon" name="check"></m3e-icon>
            My List
          </m3e-button>
          <m3e-icon-button variant="tonal" size="medium" aria-label="Details" onclick={() => nav.go({ name: "title", title: current })}>
            <m3e-icon name="info"></m3e-icon>
          </m3e-icon-button>
        </div>
      </div>
    {/key}

    {#if items.length > 1}
      <div class="pager">
        <div class="dots" role="tablist" aria-label="Choose slide">
          {#each items as item, i (item.origin.id + i)}
            <button
              class="dot"
              class:active={i === index}
              role="tab"
              aria-selected={i === index}
              aria-label={item.name}
              onclick={() => (index = i)}
            ></button>
          {/each}
        </div>
        <m3e-icon-button variant="tonal" aria-label="Previous" onclick={() => go(-1)}>
          <m3e-icon name="chevron_left"></m3e-icon>
        </m3e-icon-button>
        <m3e-icon-button variant="tonal" aria-label="Next" onclick={() => go(1)}>
          <m3e-icon name="chevron_right"></m3e-icon>
        </m3e-icon-button>
      </div>
    {/if}
  </section>
{/if}

<style>
  .hero {
    position: relative;
    display: block;
    height: clamp(320px, 42vh, 440px);
    border-radius: var(--md-sys-shape-corner-extra-large, 28px);
    overflow: hidden;
    isolation: isolate;
    background: var(--md-sys-color-surface-container);
  }
  .backdrop {
    position: absolute;
    inset: 0;
    background: center 25% / cover no-repeat;
    z-index: -2;
    animation: drift 18s ease-in-out infinite alternate;
  }
  @keyframes drift {
    from {
      transform: scale(1.02);
    }
    to {
      transform: scale(1.08) translateX(-1%);
    }
  }
  .scrim {
    position: absolute;
    inset: 0;
    z-index: -1;
    background:
      linear-gradient(90deg, rgb(10 8 18 / 92%) 0%, rgb(10 8 18 / 62%) 38%, transparent 72%),
      linear-gradient(0deg, rgb(10 8 18 / 70%), transparent 45%);
  }
  .content {
    position: absolute;
    left: 40px;
    bottom: 36px;
    width: min(520px, 60%);
    display: flex;
    flex-direction: column;
    gap: 14px;
    color: #fff;
  }
  .badge {
    align-self: flex-start;
    --m3e-icon-size: 18px;
  }
  .badge m3e-icon {
    color: var(--brand-coral);
  }
  .title {
    margin: 0;
    font-size: clamp(40px, 4.6vw, 64px);
    line-height: 1;
    font-weight: 400;
    text-wrap: balance;
  }
  .meta {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px 14px;
    font-size: 14px;
    color: rgb(255 255 255 / 80%);
  }
  .star {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    color: #fff;
    font-weight: 600;
    --m3e-icon-size: 16px;
  }
  .star m3e-icon {
    color: #ffc94d;
  }
  .tag {
    padding: 1px 8px;
    border-radius: 6px;
    font-size: 12px;
    border: 1px solid rgb(255 255 255 / 35%);
  }
  .overview {
    margin: 0;
    font-size: 15px;
    line-height: 1.5;
    color: rgb(255 255 255 / 85%);
    display: -webkit-box;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 6px;
  }
  .pager {
    position: absolute;
    right: 24px;
    bottom: 24px;
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .dots {
    display: flex;
    gap: 6px;
    margin-right: 8px;
  }
  .dot {
    all: unset;
    cursor: pointer;
    width: 8px;
    height: 8px;
    border-radius: 999px;
    background: rgb(255 255 255 / 40%);
    transition:
      width var(--md-sys-motion-spring-fast-spatial, 350ms ease),
      background-color 200ms ease;
  }
  .dot.active {
    width: 24px;
    background: var(--md-sys-color-primary);
  }
  .dot:focus-visible {
    outline: 2px solid var(--md-sys-color-primary);
    outline-offset: 2px;
  }
  .ghost {
    width: 100%;
  }
</style>
