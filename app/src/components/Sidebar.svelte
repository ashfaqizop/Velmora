<script lang="ts">
  import { onMount } from "svelte";
  import { fade, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import PoweredBy from "./PoweredBy.svelte";
  import { ani, downloads, library, nav, type Route } from "../lib/state.svelte";

  const main: { label: string; icon: string; route: Route }[] = [
    { label: "Home", icon: "home", route: { name: "home" } },
    { label: "Movies", icon: "movie", route: { name: "browse", kind: "movie" } },
    { label: "Series", icon: "tv", route: { name: "browse", kind: "series" } },
    { label: "Anime", icon: "animation", route: { name: "browse", kind: "anime" } },
    { label: "My List", icon: "bookmarks", route: { name: "mylist" } },
    { label: "Downloads", icon: "download", route: { name: "downloads" } },
    { label: "Settings", icon: "settings", route: { name: "settings" } },
  ];

  // Ani sits after the browse pages, and only when the user turned it on.
  const ANI = { label: "Ani", icon: "auto_awesome", route: { name: "ani" } as Route };
  const items = $derived(ani.enabled ? [...main.slice(0, 4), ANI, ...main.slice(4)] : main);

  function isActive(r: Route) {
    const cur = nav.route;
    if (cur.name !== r.name) return false;
    if (cur.name === "browse" && r.name === "browse") return cur.kind === r.kind;
    return true;
  }

  // Rotating tips for the card under the menu: a short heading and one useful line.
  const TIPS: [string, string][] = [
    ["Better movies. Brighter days.", "Every source, one search away."],
    ["Ctrl K finds anything.", "Movies, series and anime from every source at once."],
    ["Space pauses. F goes full screen.", "Arrow keys skip ten seconds either way."],
    ["Stop anywhere.", "Continue Watching remembers the exact second."],
    ["Downloads play offline.", "Grab an episode before the flight."],
    ["When a link dies, Velmora moves on.", "The next source starts at the same spot."],
    ["Sub or dub stays the way you set it.", "Change it on any anime page."],
    ["Your AniList can update itself.", "Connect it in Settings, then just watch."],
    ["Purple isn't mandatory.", "Eight accent colours are waiting in Settings."],
    ["Torrents work best behind a VPN.", "Peers can see your IP while one runs."],
    ["A new pick every night.", "Tonight's pick on Home changes daily."],
    ["4K when you want it.", "Turn on auto 4K in Settings, or choose it per title."],
  ];
  const TIP_MS = 9000;

  // The first launch always opens on the brand line; later ones start anywhere.
  let tip = $state(localStorage.getItem("velmora.onboarded") === "1" ? Math.floor(Math.random() * TIPS.length) : 0);

  onMount(() => {
    const timer = setInterval(() => {
      if (!document.hidden) tip = (tip + 1) % TIPS.length;
    }, TIP_MS);
    return () => clearInterval(timer);
  });
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

  <div class="promo">
    <m3e-shape name="flower" class="s1"></m3e-shape>
    <m3e-shape name="soft-burst" class="s2"></m3e-shape>
    <m3e-shape name="circle" class="s3"></m3e-shape>
    <div class="copy" aria-live="off">
      {#key tip}
        <div class="tip" in:fly={{ y: 10, duration: 500, delay: 200, easing: cubicOut }} out:fade={{ duration: 200 }}>
          <strong class="display">{TIPS[tip][0]}</strong>
          <span>{TIPS[tip][1]}</span>
        </div>
      {/key}
    </div>
    <div class="credit"><PoweredBy height={10} /></div>
  </div>
</aside>

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
    height: 200px;
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
  /* A soft shade along the bottom keeps the copy readable over the shapes. */
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
  /* Outgoing and incoming tips share one cell, so the card never jumps. */
  .copy {
    position: relative;
    z-index: 1;
    display: grid;
    align-items: end;
  }
  .tip {
    grid-area: 1 / 1;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .copy strong {
    font-size: 22px;
    line-height: 1.15;
    text-wrap: balance;
  }
  .copy span {
    font-size: 12px;
    color: rgb(255 255 255 / 82%);
  }
  .credit {
    position: relative;
    z-index: 1;
    margin-top: 8px;
  }
</style>
