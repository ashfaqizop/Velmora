<script lang="ts">
  import PoweredBy from "./PoweredBy.svelte";
  import AdSlot from "./AdSlot.svelte";
  import { ani, downloads, library, nav, type Route } from "../lib/state.svelte";

  const main: { label: string; icon: string; route: Route }[] = [
    { label: "Home", icon: "home", route: { name: "home" } },
    { label: "Movies", icon: "movie", route: { name: "browse", kind: "movie" } },
    { label: "Series", icon: "tv", route: { name: "browse", kind: "series" } },
    { label: "Anime", icon: "animation", route: { name: "browse", kind: "anime" } },
    { label: "Live TV", icon: "live_tv", route: { name: "live" } },
    { label: "Sports", icon: "sports_soccer", route: { name: "sports" } },
    { label: "My List", icon: "bookmarks", route: { name: "mylist" } },
    { label: "Downloads", icon: "download", route: { name: "downloads" } },
    { label: "Settings", icon: "settings", route: { name: "settings" } },
  ];

  // Ani sits after the browse pages, and only when the user turned it on.
  const ANI = { label: "Ani", icon: "wand_stars", route: { name: "ani" } as Route };
  const items = $derived(ani.enabled ? [...main.slice(0, 4), ANI, ...main.slice(4)] : main);

  function isActive(r: Route) {
    const cur = nav.route;
    // A match belongs under Sports.
    if (r.name === "sports" && cur.name === "match") return true;
    if (cur.name !== r.name) return false;
    if (cur.name === "browse" && r.name === "browse") return cur.kind === r.kind;
    return true;
  }

  // The sidebar ad needs a taller card, so it only runs on tall windows.
  let windowHeight = $state(innerHeight);
  let adShown = $state(false);
</script>

<aside class="sidebar">
  <m3e-nav-menu class="menu">
    {#each items as item (item.label)}
      <m3e-nav-menu-item selected={isActive(item.route)} onclick={() => nav.go(item.route)}>
        <m3e-icon slot="icon" name={item.icon}></m3e-icon>
        <m3e-icon slot="selected-icon" name={item.icon} filled></m3e-icon>
        <span slot="label">{item.label}</span>
        {#if item.route.name === "mylist" && library.watchlist.length}
          <span slot="badge">{library.watchlist.length}</span>
        {:else if item.route.name === "downloads" && downloads.active}
          <span slot="badge">{downloads.active}</span>
        {/if}
      </m3e-nav-menu-item>
    {/each}

  </m3e-nav-menu>

  <div class="promo" class:with-ad={adShown}>
    <m3e-shape name="flower" class="s1"></m3e-shape>
    <m3e-shape name="soft-burst" class="s2"></m3e-shape>
    <m3e-shape name="circle" class="s3"></m3e-shape>
    {#if windowHeight >= 920}
      <div class="sidebar-ad"><AdSlot id="sidebar" bind:shown={adShown} /></div>
    {/if}
    <div class="credit"><PoweredBy height={10} /></div>
  </div>
</aside>

<svelte:window bind:innerHeight={windowHeight} />

<style>
  .sidebar {
    height: 100%;
    display: flex;
    flex-direction: column;
    min-height: 0;
    padding: 4px 12px 16px;
    box-sizing: border-box;
  }
  .menu {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    overflow-x: hidden;
    /* Scrolls only on very short windows, without a visible bar. */
    scrollbar-width: none;
    --m3e-nav-menu-item-height: 48px;
    --m3e-nav-menu-item-shape: 999px;
    --m3e-nav-menu-item-selected-container-color: var(--md-sys-color-primary);
    --m3e-nav-menu-item-selected-label-color: var(--md-sys-color-on-primary);
    --m3e-nav-menu-padding-left: 0;
    --m3e-nav-menu-padding-right: 0;
  }
  /* Deep gradient with white text so it reads with any accent colour. */
  .promo {
    position: relative;
    flex: none;
    margin-top: 12px;
    height: 96px;
    border-radius: var(--md-sys-shape-corner-extra-large, 28px);
    overflow: hidden;
    padding: 20px;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    gap: 6px;
    background: linear-gradient(
      150deg,
      color-mix(in srgb, var(--md-sys-color-primary) 58%, #120b24) 0%,
      color-mix(in srgb, var(--brand-coral) 42%, #1a0b1c) 100%
    );
    color: #fff;
    /* PoweredBy reads these. */
    --md-sys-color-on-surface: #fff;
    --md-sys-color-on-surface-variant: rgb(255 255 255 / 78%);
  }
  /* A soft shade along the bottom keeps the credit readable over the shapes. */
  .promo::after {
    content: "";
    position: absolute;
    inset: 40% 0 0;
    background: linear-gradient(0deg, rgb(10 6 20 / 55%), transparent);
    pointer-events: none;
  }
  .promo m3e-shape {
    position: absolute;
    opacity: 0.5;
  }
  .s1 {
    --m3e-shape-size: 72px;
    --m3e-shape-container-color: var(--md-sys-color-primary-container);
    top: 14px;
    left: 16px;
    animation:
      spin 26s linear infinite,
      drift 9s ease-in-out infinite alternate;
  }
  .s2 {
    --m3e-shape-size: 120px;
    --m3e-shape-container-color: var(--brand-coral);
    top: -36px;
    right: -30px;
    opacity: 0.45;
    animation: spin 40s linear infinite reverse;
  }
  .s3 {
    --m3e-shape-size: 46px;
    --m3e-shape-container-color: var(--md-sys-color-tertiary);
    top: 70px;
    right: 40px;
    animation:
      spin 18s linear infinite,
      drift 7s ease-in-out infinite alternate-reverse;
  }
  @keyframes spin {
    to {
      rotate: 360deg;
    }
  }
  @keyframes drift {
    to {
      translate: 6px 8px;
    }
  }
  /* While an ad shows, the card grows to fit it and the tips step aside. */
  .promo.with-ad {
    height: auto;
    padding: 12px;
    justify-content: flex-start;
  }
  .promo.with-ad m3e-shape {
    opacity: 0.18;
  }
  .sidebar-ad {
    position: relative;
    z-index: 1;
  }
  .credit {
    position: relative;
    z-index: 1;
    margin-top: 8px;
  }
</style>
