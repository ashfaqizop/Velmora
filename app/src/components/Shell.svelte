<script lang="ts">
  import { fade, fly } from "svelte/transition";
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
  import { feed, library, nav } from "../lib/state.svelte";

  let { revealed = false }: { revealed?: boolean } = $props();

  let about: ReturnType<typeof About>;
  let scroller: HTMLElement;

  const BROWSE = {
    movie: { title: "Movies", subtitle: "Trending films from every source" },
    series: { title: "Series", subtitle: "Shows and dramas people are watching" },
    anime: { title: "Anime", subtitle: "Seasonal hits and all-time favourites" },
  } as const;

  // Every navigation starts at the top of the page.
  $effect(() => {
    void nav.route;
    scroller?.scrollTo({ top: 0 });
  });
</script>

<div class="shell" class:revealed>
  <div class="top"><TopBar onabout={() => about.open()} /></div>
  <div class="side"><Sidebar /></div>

  <main class="content" class:wide={nav.route.name !== "home"} bind:this={scroller}>
    {#if nav.canGoBack}
      <div class="back" in:fade={{ duration: 200 }}>
        <m3e-button variant="text" size="small" onclick={() => nav.back()}>
          <m3e-icon slot="icon" name="arrow_back"></m3e-icon>Back
        </m3e-button>
      </div>
    {/if}

    {#key nav.route}
      <div class="page" in:fly={{ y: 12, duration: 380, easing: cubicOut }}>
        {#if nav.route.name === "home"}
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
        {:else if nav.route.name === "search"}
          <Search query={nav.route.query} />
        {:else if nav.route.name === "title"}
          <Title title={nav.route.title} />
        {/if}
      </div>
    {/key}
  </main>

  {#if nav.route.name === "home"}
    <div class="panel" transition:fade={{ duration: 200 }}><SidePanel /></div>
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
  .back {
    margin: -4px 0 8px -8px;
  }
  .panel {
    grid-area: panel;
    width: var(--app-panel-width);
    min-height: 0;
  }
  @media (max-width: 1280px) {
    .panel {
      display: none;
    }
  }
</style>
