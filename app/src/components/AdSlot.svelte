<script lang="ts">
  // One ad slot. The ad is a page on the Velmora site, shown in a sandboxed
  // iframe: it can run its own scripts and open links (which go to the
  // default browser, see create_main_window in lib.rs), but it can't touch
  // the app or navigate it. The page reports back whether it filled; an empty
  // slot stays collapsed. All the rules live in lib/ads.svelte.ts.
  import { onMount, untrack } from "svelte";
  import { fade } from "svelte/transition";
  import { ads, unitSize, type SlotId } from "../lib/ads.svelte";

  let {
    id,
    label = true,
    shown = $bindable(false),
  }: {
    id: SlotId;
    label?: boolean;
    /** True while an ad is actually on screen, for parents that make room for it. */
    shown?: boolean;
  } = $props();

  // A slot never changes its id, so these are fixed for its lifetime.
  const fixedId = untrack(() => id);
  // Page ads take turns; the sidebar and the player's cards don't.
  const shared = fixedId !== "sidebar" && fixedId !== "pause" && fixedId !== "end";
  const instance = `${fixedId}-${Math.random().toString(36).slice(2)}`;

  let box = $state<HTMLElement>();
  let frame = $state<HTMLIFrameElement>();
  let width = $state(0);
  let visible = $state(false);
  /** Set the first time this slot may load; it then keeps its ad while on the page. */
  let started = $state(false);
  let round = $state(0);
  let filled = $state(false);
  let counted = false;
  let height = $state(0);

  const unit = $derived(ads.unit(id));
  const size = $derived(unitSize(unit));
  // Banners keep their size and shrink to fit; native ads take the width.
  const scale = $derived(size && width ? Math.min(1, width / size.width) : 1);
  const show = $derived(started && ads.canShow(id) && (!shared || ads.holder === instance));

  $effect(() => {
    shown = show && filled;
  });

  // Start when it's on screen and the rules allow it.
  $effect(() => {
    if (started || !visible || !ads.allowed(id)) return;
    if (shared && !ads.claim(instance)) return;
    started = true;
  });

  // Give up the turn when out of view, so the next page ad can show.
  $effect(() => {
    if (shared && started && !visible) ads.release(instance);
    if (shared && started && visible && ads.holder === null) ads.claim(instance);
  });

  function onMessage(e: MessageEvent) {
    if (!frame || e.source !== frame.contentWindow) return;
    const d = e.data;
    if (!d || d.source !== "velmora-ad") return;
    if (d.state === "filled") {
      filled = true;
      if (typeof d.height === "number") height = Math.min(600, Math.max(0, d.height));
      if (!counted) {
        counted = true;
        ads.counted(id);
      }
    } else if (d.state === "empty") {
      filled = false;
      if (shared) ads.release(instance);
    }
  }

  onMount(() => {
    const io = new IntersectionObserver(([e]) => (visible = e.isIntersecting), { threshold: 0.35 });
    if (box) io.observe(box);
    const ro = new ResizeObserver(([e]) => (width = e.contentRect.width));
    if (box) ro.observe(box);

    // A fresh ad now and then, only while it's seen and the window has focus.
    const timer = setInterval(() => {
      if (!show || !filled || !visible || document.hidden || !document.hasFocus()) return;
      if (!ads.allowed(id)) return;
      counted = false;
      round += 1;
    }, ads.rules.refreshSeconds * 1000);

    addEventListener("message", onMessage);
    return () => {
      io.disconnect();
      ro.disconnect();
      clearInterval(timer);
      removeEventListener("message", onMessage);
      if (shared) ads.release(instance);
    };
  });
</script>

<div class="slot {id}" class:filled={show && filled} bind:this={box} aria-hidden={!(show && filled)}>
  {#if show}
    {#key round}
      <div
        class="ad"
        class:native={!size}
        style:height={size ? `${size.height * scale}px` : height ? `${height}px` : undefined}
        in:fade={{ duration: 300 }}
      >
        {#if label}<span class="tag">Ad</span>{/if}
        <iframe
          bind:this={frame}
          title="Advertisement"
          src={`${ads.src(id)}?r=${round}`}
          sandbox="allow-scripts allow-same-origin allow-popups allow-popups-to-escape-sandbox"
          referrerpolicy="strict-origin-when-cross-origin"
          scrolling="no"
          width={size?.width}
          height={size ? size.height : height || 120}
          style:transform={scale < 1 ? `scale(${scale})` : undefined}
        ></iframe>
      </div>
    {/key}
  {/if}
</div>

<style>
  /* Collapsed (zero height, no margin) until the ad actually shows. */
  .slot {
    display: grid;
    justify-items: center;
    width: 100%;
    max-height: 0;
    overflow: hidden;
    opacity: 0;
    transition:
      max-height 400ms ease,
      opacity 400ms ease;
  }
  .slot.filled {
    max-height: 640px;
    opacity: 1;
  }
  .ad {
    position: relative;
    display: grid;
    justify-items: center;
    align-items: start;
    width: 100%;
  }
  .ad.native {
    padding: 12px 14px;
    border-radius: 20px;
    background: color-mix(in srgb, var(--m3e-color-surface-container, #1d1b20) 70%, transparent);
    box-sizing: border-box;
  }
  iframe {
    border: 0;
    display: block;
    transform-origin: top center;
    background: transparent;
    color-scheme: dark;
  }
  .ad.native iframe {
    width: 100%;
  }
  .tag {
    position: absolute;
    top: 6px;
    right: 8px;
    z-index: 1;
    padding: 1px 6px;
    border-radius: 6px;
    font-size: 10px;
    letter-spacing: 0.06em;
    color: rgb(255 255 255 / 55%);
    background: rgb(0 0 0 / 35%);
    pointer-events: none;
  }
</style>
