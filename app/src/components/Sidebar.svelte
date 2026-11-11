<script lang="ts">
  import { fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import PoweredBy from "./PoweredBy.svelte";
  import AdSlot from "./AdSlot.svelte";
  import Art from "./music/Art.svelte";
  import { ani, downloads, library, nav, type Route } from "../lib/state.svelte";
  import { musicSession } from "../lib/music/session.svelte";
  import { musicUi } from "../lib/music/ui.svelte";
  import { reducedMotion } from "../lib/motion";

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

  // Ani sits after the browse pages, and only when the user turned it on;
  // Music comes right after.
  const ANI = { label: "Ani", icon: "wand_stars", route: { name: "ani" } as Route };
  const MUSIC = { label: "Music", icon: "library_music", route: { name: "music" } as Route };
  const items = $derived([...main.slice(0, 4), ...(ani.enabled ? [ANI] : []), MUSIC, ...main.slice(4)]);

  const MUSIC_ITEMS: { label: string; icon: string; route: Route }[] = [
    { label: "Home", icon: "home", route: { name: "music" } },
    { label: "Explore", icon: "explore", route: { name: "music-explore" } },
    { label: "Library", icon: "library_music", route: { name: "music-library", tab: "playlists" } },
    { label: "Liked songs", icon: "favorite", route: { name: "music-playlist", id: "LM" } },
    { label: "Downloaded", icon: "download_done", route: { name: "music-library", tab: "downloaded" } },
  ];

  const mode = $derived(nav.musicMode ? "music" : "velmora");

  function isMusicActive(r: Route) {
    const cur = nav.route;
    if (r.name === "music-playlist" && cur.name === "music-playlist") return r.id === cur.id;
    if (r.name === "music-library" && cur.name === "music-library") return (r.tab === "downloaded") === (cur.tab === "downloaded");
    if (r.name === "music" && (cur.name === "music-feed" || cur.name === "music-search")) return false;
    return cur.name === r.name;
  }

  // Your playlists, listed under the music menu once signed in.
  $effect(() => {
    // Once per session: checking `playlists.length` alone looped forever on an
    // account with no playlists (each empty answer re-ran this effect).
    if (mode === "music" && musicSession.signedIn && !musicUi.playlistsLoaded) musicUi.loadPlaylists();
  });

  // Items leave one way and arrive from the other, a few ms apart.
  const D = () => (reducedMotion() ? 0 : 1);
  const enter = (i: number, from: number) => ({ x: from * D(), duration: 360 * D(), delay: (60 + i * 28) * D(), easing: cubicOut, opacity: 0 });
  const leave = (i: number, to: number) => ({ x: to * D(), duration: 220 * D(), delay: i * 14 * D(), easing: cubicOut, opacity: 0 });

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
  <div class="stack">
    {#key mode}
      {#if mode === "velmora"}
        <m3e-nav-menu class="menu layer">
          {#each items as item, i (item.label)}
            <m3e-nav-menu-item
              selected={isActive(item.route)}
              onclick={() => nav.go(item.route)}
              in:fly={enter(i, -36)}
              out:fly={leave(i, -36)}
            >
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
      {:else}
        <div class="layer music">
          <button class="leave" onclick={() => nav.leaveMusic()} in:fly={enter(0, 36)} out:fly={leave(0, 36)}>
            <m3e-icon name="arrow_back"></m3e-icon>Velmora
          </button>
          <div class="brand" in:fly={enter(1, 36)} out:fly={leave(1, 36)}>
            <span class="mark on-shape" aria-hidden="true">
              <m3e-shape name="soft-burst"></m3e-shape>
              <m3e-icon name="music_note" filled></m3e-icon>
            </span>
            <strong class="display">Music</strong>
          </div>
          <m3e-nav-menu class="menu">
            {#each MUSIC_ITEMS as item, i (item.label)}
              <m3e-nav-menu-item
                selected={isMusicActive(item.route)}
                onclick={() => nav.go(item.route)}
                in:fly={enter(i + 2, 36)}
                out:fly={leave(i + 2, 36)}
              >
                <m3e-icon slot="icon" name={item.icon}></m3e-icon>
                <m3e-icon slot="selected-icon" name={item.icon} filled></m3e-icon>
                <span slot="label">{item.label}</span>
              </m3e-nav-menu-item>
            {/each}
          </m3e-nav-menu>
          {#if musicSession.signedIn && musicUi.playlists.length}
            <span class="heading" in:fly={enter(7, 36)}>Your playlists</span>
            <div class="playlists">
              {#each musicUi.playlists as p, i (p.id)}
                <button
                  class="pl"
                  class:on={nav.route.name === "music-playlist" && nav.route.id === p.id}
                  onclick={() => nav.go({ name: "music-playlist", id: p.id })}
                  in:fly={enter(Math.min(i, 8) + 8, 36)}
                >
                  <Art thumbs={p.thumbnails} size={32} alt="" />
                  <span>{p.title}</span>
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    {/key}
  </div>

  <div class="promo" class:with-ad={adShown} class:music={mode === "music"}>
    <m3e-shape name={mode === "music" ? "soft-boom" : "flower"} class="s1"></m3e-shape>
    <m3e-shape name={mode === "music" ? "very-sunny" : "soft-burst"} class="s2"></m3e-shape>
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
  /* Both modes share one cell, so the old one leaves while the new one arrives. */
  .stack {
    flex: 1;
    min-height: 0;
    display: grid;
    overflow: hidden;
  }
  .layer {
    grid-area: 1 / 1;
    min-height: 0;
  }
  .music {
    display: flex;
    flex-direction: column;
    gap: 6px;
    overflow-y: auto;
    scrollbar-width: none;
  }
  .leave {
    all: unset;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-radius: 999px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 600;
    color: var(--md-sys-color-on-surface-variant);
    align-self: flex-start;
    --m3e-icon-size: 18px;
    transition: background 160ms ease;
  }
  .leave:hover {
    background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
    color: var(--md-sys-color-on-surface);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 4px 12px 10px;
  }
  .mark {
    --m3e-shape-size: 40px;
    --m3e-shape-container-color: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
    --m3e-icon-size: 20px;
    animation: turn 24s linear infinite;
  }
  @keyframes turn {
    to {
      rotate: 360deg;
    }
  }
  .brand strong {
    font-size: 30px;
    font-weight: 400;
  }
  .heading {
    margin: 14px 12px 4px;
    font-size: 11.5px;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--md-sys-color-on-surface-variant);
  }
  .playlists {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .pl {
    all: unset;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 10px;
    border-radius: 12px;
    cursor: pointer;
    font-size: 13.5px;
    min-width: 0;
  }
  .pl span {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .pl:hover {
    background: color-mix(in srgb, var(--md-sys-color-on-surface) 7%, transparent);
  }
  .pl.on {
    background: color-mix(in srgb, var(--md-sys-color-primary) 18%, transparent);
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
