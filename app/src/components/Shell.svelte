<script lang="ts">
  import { fade, fly, scale } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import TopBar from "./TopBar.svelte";
  import Sidebar from "./Sidebar.svelte";
  import SidePanel from "./SidePanel.svelte";
  import About from "./About.svelte";
  import Home from "../pages/Home.svelte";
  import Grid from "../pages/Grid.svelte";
  import Search from "../pages/Search.svelte";
  import Title from "../pages/Title.svelte";
  import Downloads from "../pages/Downloads.svelte";
  import Settings from "../pages/Settings.svelte";
  import Ani from "../pages/Ani.svelte";
  import Live from "../pages/Live.svelte";
  import Sports from "../pages/Sports.svelte";
  import Match from "../pages/Match.svelte";
  import MusicHome from "../pages/music/MusicHome.svelte";
  import MusicExplore from "../pages/music/MusicExplore.svelte";
  import MusicSearch from "../pages/music/MusicSearch.svelte";
  import MusicAlbum from "../pages/music/MusicAlbum.svelte";
  import MusicArtist from "../pages/music/MusicArtist.svelte";
  import MusicPlaylist from "../pages/music/MusicPlaylist.svelte";
  import MusicLibrary from "../pages/music/MusicLibrary.svelte";
  import FeedView from "./music/FeedView.svelte";
  import SignIn from "./music/SignIn.svelte";
  import { musicSession } from "../lib/music/session.svelte";
  import { musicUi } from "../lib/music/ui.svelte";
  import { reducedMotion } from "../lib/motion";
  import { onMount } from "svelte";
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import { ani, feed, library, nav, network } from "../lib/state.svelte";

  let { revealed = false }: { revealed?: boolean } = $props();

  let about: ReturnType<typeof About>;
  let scroller: HTMLElement;

  const BROWSE = {
    movie: { title: "Movies", subtitle: "Trending films from every source" },
    series: { title: "Series", subtitle: "Shows and dramas people are watching" },
    anime: { title: "Anime", subtitle: "Seasonal hits and all-time favourites" },
  } as const;

  // People who set Velmora up before Ani existed never saw its onboarding step.
  onMount(() => {
    if (localStorage.getItem("velmora.ani.enabled") !== null) return;
    if (localStorage.getItem("velmora.onboarded") !== "1") return;
    const timer = setTimeout(() => {
      localStorage.setItem("velmora.ani.enabled", "0");
      M3eSnackbar.open("New: Ani, an assistant that knows what you watch.", "Try it", {
        actionCallback: () => {
          ani.setEnabled(true);
          nav.go({ name: "ani" });
        },
      });
    }, 4000);
    return () => clearTimeout(timer);
  });

  // The right panel on Home folds into a slim strip; closed unless opened.
  let panelOpen = $state(localStorage.getItem("velmora.home.panel") === "1");
  function togglePanel() {
    panelOpen = !panelOpen;
    localStorage.setItem("velmora.home.panel", panelOpen ? "1" : "0");
  }

  // Music mode or Velmora mode; switching between them gets its own motion.
  const mode = $derived(nav.musicMode ? "music" : "velmora");

  let lastTop = 0;
  function onScroll() {
    if (!scroller) return;
    musicUi.scrolled(scroller.scrollTop - lastTop);
    lastTop = scroller.scrollTop;
  }

  // Every navigation starts at the top of the page.
  $effect(() => {
    void nav.route;
    scroller?.scrollTo({ top: 0 });
  });
</script>

<div class="shell" class:revealed>
  <div class="top"><TopBar onabout={() => about.open()} /></div>
  <div class="side"><Sidebar /></div>

  <main class="content" class:wide={nav.route.name !== "home"} class:music={mode === "music"} bind:this={scroller} onscroll={onScroll}>
    {#if !network.online}
      <m3e-callout variant="error" class="offline" transition:fade={{ duration: 200 }}>
        <m3e-icon slot="icon" name="wifi_off"></m3e-icon>
        <span slot="label">You're offline</span>
        Search and streaming need a connection. Finished downloads still play.
        {#if nav.route.name !== "downloads"}
          <m3e-button variant="text" size="small" onclick={() => nav.go({ name: "downloads" })}>Open Downloads</m3e-button>
        {/if}
      </m3e-callout>
    {/if}
    {#if nav.canGoBack}
      <div class="back" in:fade={{ duration: 200 }}>
        <m3e-button variant="text" size="small" onclick={() => nav.back()}>
          <m3e-icon slot="icon" name="arrow_back"></m3e-icon>Back
        </m3e-button>
      </div>
    {/if}

    {#key mode}
    <div class="mode" in:scale={{ start: reducedMotion() ? 1 : 0.97, opacity: 0, duration: reducedMotion() ? 0 : 460, easing: cubicOut }}>
    {#key nav.route}
      <div class="page" in:fly={{ y: 12, duration: 380, easing: cubicOut }}>
        {#if mode === "music" && !musicSession.signedIn && !(nav.route.name === "music-library" && nav.route.tab === "downloaded")}
          <SignIn />
        {:else if nav.route.name === "music"}
          <MusicHome />
        {:else if nav.route.name === "music-explore"}
          <MusicExplore tab={nav.route.tab} />
        {:else if nav.route.name === "music-search"}
          <MusicSearch query={nav.route.query} />
        {:else if nav.route.name === "music-album"}
          <MusicAlbum id={nav.route.id} />
        {:else if nav.route.name === "music-artist"}
          <MusicArtist id={nav.route.id} />
        {:else if nav.route.name === "music-playlist"}
          <MusicPlaylist id={nav.route.id} />
        {:else if nav.route.name === "music-library"}
          <MusicLibrary tab={nav.route.tab} />
        {:else if nav.route.name === "music-feed"}
          <FeedView browseId={nav.route.browseId} params={nav.route.params} title={nav.route.title} />
        {:else if nav.route.name === "home"}
          <Home />
        {:else if nav.route.name === "browse"}
          <Grid {...BROWSE[nav.route.kind]} items={feed.ofKind(nav.route.kind)} loading={feed.loading} />
        {:else if nav.route.name === "collection"}
          {@const rail = feed.rail(nav.route.id)}
          <Grid title={rail?.title ?? "Collection"} items={rail?.items ?? []} loading={feed.loading} />
        {:else if nav.route.name === "mylist"}
          <Grid title="My List" subtitle="Saved on this device" items={library.watchlist}>
            {#snippet empty()}
              <m3e-icon name="bookmark_add"></m3e-icon>
              <m3e-heading variant="title" size="large">Your list is empty</m3e-heading>
              <p>Tap My List on any title to keep it here.</p>
            {/snippet}
          </Grid>
        {:else if nav.route.name === "downloads"}
          <Downloads />
        {:else if nav.route.name === "settings"}
          <Settings />
        {:else if nav.route.name === "ani"}
          <Ani />
        {:else if nav.route.name === "live"}
          <Live />
        {:else if nav.route.name === "sports"}
          <Sports />
        {:else if nav.route.name === "match"}
          <Match match={nav.route.match} />
        {:else if nav.route.name === "list"}
          <Grid title={nav.route.title} subtitle="From Ani" items={nav.route.items} />
        {:else if nav.route.name === "search"}
          <Search query={nav.route.query} />
        {:else if nav.route.name === "title"}
          <Title title={nav.route.title} />
        {/if}
      </div>
    {/key}
    </div>
    {/key}
  </main>

  {#if nav.route.name === "home"}
    <div class="panel" class:open={panelOpen} transition:fade={{ duration: 200 }}>
      <button class="panel-toggle" aria-label={panelOpen ? "Hide the side panel" : "Show the side panel"} aria-expanded={panelOpen} onclick={togglePanel}>
        <m3e-icon name={panelOpen ? "right_panel_close" : "right_panel_open"}></m3e-icon>
      </button>
      {#if panelOpen}
        <div class="panel-body" transition:fly={{ x: 40, duration: 320, easing: cubicOut }}><SidePanel /></div>
      {/if}
    </div>
  {/if}
</div>

<About bind:this={about} />

<style>
  .shell {
    height: 100%;
    display: grid;
    grid-template-columns: var(--app-sidebar-width) minmax(0, 1fr) auto;
    grid-template-rows: var(--app-titlebar-height) minmax(0, 1fr);
    grid-template-areas:
      "top top top"
      "side content panel";
    background:
      radial-gradient(70% 50% at 100% 0%, color-mix(in srgb, var(--md-sys-color-primary) 10%, transparent), transparent 70%),
      var(--md-sys-color-background);
    opacity: 0;
    transform: scale(0.985);
    transition:
      opacity 700ms ease,
      transform 900ms var(--md-sys-motion-easing-emphasized-decelerate, ease-out);
  }
  .shell.revealed {
    opacity: 1;
    transform: none;
  }
  .top {
    grid-area: top;
  }
  .side {
    grid-area: side;
    min-height: 0;
  }
  .content {
    grid-area: content;
    min-width: 0;
    min-height: 0;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 4px var(--app-gutter) 0 4px;
    scrollbar-gutter: stable;
  }
  .offline {
    display: block;
    margin-bottom: 16px;
  }
  .back {
    margin: -4px 0 8px -8px;
  }
  .panel {
    grid-area: panel;
    position: relative;
    width: 52px;
    min-height: 0;
    transition: width 380ms cubic-bezier(0.2, 0, 0, 1);
  }
  .panel.open {
    width: var(--app-panel-width);
  }
  .panel-body {
    height: 100%;
    width: var(--app-panel-width);
  }
  .panel-toggle {
    all: unset;
    position: absolute;
    top: 6px;
    left: 4px;
    z-index: 2;
    width: 40px;
    height: 40px;
    display: grid;
    place-items: center;
    border-radius: 14px;
    cursor: pointer;
    color: var(--md-sys-color-on-surface-variant);
    background: var(--md-sys-color-surface-container);
    transition:
      background 160ms ease,
      border-radius 220ms ease;
  }
  .panel.open .panel-toggle {
    left: auto;
    right: 18px;
    top: 14px;
    background: color-mix(in srgb, var(--md-sys-color-surface-container-highest) 80%, transparent);
  }
  .panel-toggle:hover {
    border-radius: 50%;
    background: var(--md-sys-color-surface-container-highest);
  }
  .mode {
    min-height: 100%;
  }
  @media (max-width: 1280px) {
    .panel {
      display: none;
    }
  }
</style>
