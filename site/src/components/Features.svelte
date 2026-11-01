<script lang="ts">
  import { reveal } from "../lib/reveal";
  import Shapes from "./Shapes.svelte";
  import { screen } from "../lib/media.svelte";
  import Dots from "./Dots.svelte";

  // Bento grid. `span` widens a card on large screens.
  const FEATURES: { icon: string; shape: string; title: string; text: string; span?: boolean; tone: string }[] = [
    {
      icon: "travel_explore",
      shape: "12-sided-cookie",
      title: "One search, every source",
      text: "Velmora checks every source at once for movies, series, anime and dramas. The same title found on different sites shows up once.",
      span: true,
      tone: "indigo",
    },
    {
      icon: "leaderboard",
      shape: "soft-burst",
      title: "The best link, picked",
      text: "Links are ranked by quality, codec, language and size, so the first one is usually the one you want.",
      tone: "coral",
    },
    {
      icon: "play_circle",
      shape: "4-leaf-clover",
      title: "A real player inside",
      text: "Built on mpv. MKV, HEVC, AV1, AC3 and 4K play without codec packs or extra apps.",
      tone: "violet",
    },
    {
      icon: "swap_horiz",
      shape: "sunny",
      title: "Never stuck on a dead link",
      text: "If a link fails, the next source starts at the same second. You don't go back to search.",
      tone: "teal",
    },
    {
      icon: "history",
      shape: "6-sided-cookie",
      title: "Pick up where you stopped",
      text: "Continue Watching remembers the exact second, and the next episode is one click away.",
      tone: "amber",
    },
    {
      icon: "graphic_eq",
      shape: "puffy-diamond",
      title: "Sound that fits the room",
      text: "Dialogue boost, Night mode, even volume between sources, and volume up to 200% for quiet videos.",
      span: true,
      tone: "coral",
    },
    {
      icon: "download",
      shape: "8-leaf-clover",
      title: "Downloads for the flight",
      text: "Save episodes to a folder you choose and watch them offline.",
      tone: "indigo",
    },
    {
      icon: "sync",
      shape: "9-sided-cookie",
      title: "AniList and Trakt sync",
      text: "Connect once and what you finish gets marked as watched on its own.",
      tone: "teal",
    },
    {
      icon: "palette",
      shape: "very-sunny",
      title: "Make it yours",
      text: "Material 3 Expressive design with eight accent colors to pick from.",
      tone: "violet",
    },
  ];


  // On phones the grid becomes a swipeable carousel (see the CSS).
  let track = $state<HTMLElement>();
</script>

<section id="features">
  <Shapes
    spots={[
      { x: "93%", y: "150px", size: 150, color: "#7b6cff", spin: 26, opacity: 0.55 },
      { x: "4%", y: "62%", size: 76, color: "#2dd4bf", spin: -14, offset: 6, opacity: 0.7 },
    ]}
  />
  <div class="wrap">
    <div use:reveal>
      <span class="eyebrow">Features</span>
      <h2 class="title">Everything you need. <em>Nothing you don't.</em></h2>
      <p class="lead">Velmora does the searching, sorting and link checking, so you get to the watching.</p>
    </div>
    <div class="grid" bind:this={track}>
      {#each FEATURES as f, i (f.title)}
        <article class="card tone-{f.tone}" class:span={f.span} use:reveal={screen.phone ? 0 : (i % 3) * 0.08}>
          <span class="on-shape icon"><m3e-shape name={f.shape}></m3e-shape><m3e-icon name={f.icon} filled></m3e-icon></span>
          <h3>{f.title}</h3>
          <p>{f.text}</p>
        </article>
      {/each}
    </div>
    {#if screen.phone}<Dots {track} labels={FEATURES.map((f) => f.title)} label="Features" />{/if}
  </div>
</section>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 16px;
    margin-top: 56px;
  }
  .card {
    --tone: var(--indigo);
    position: relative;
    overflow: hidden;
    padding: 28px;
    border-radius: 28px;
    border: 1px solid var(--line);
    background:
      radial-gradient(120% 80% at 100% 0%, color-mix(in srgb, var(--tone) 16%, transparent), transparent 60%),
      var(--glass);
    transition:
      transform 0.4s cubic-bezier(0.2, 0.7, 0.2, 1),
      border-color 0.4s,
      opacity 0.9s cubic-bezier(0.2, 0.7, 0.2, 1);
  }
  .card:hover {
    transform: translateY(-4px);
    border-color: color-mix(in srgb, var(--tone) 45%, transparent);
  }
  .card.span {
    grid-column: span 2;
  }
  .tone-coral {
    --tone: var(--coral);
  }
  .tone-violet {
    --tone: var(--violet);
  }
  .tone-teal {
    --tone: #2dd4bf;
  }
  .tone-amber {
    --tone: var(--amber);
  }
  .icon {
    --m3e-shape-size: 56px;
    --m3e-shape-container-color: color-mix(in srgb, var(--tone) 85%, #fff 0%);
    color: #120e1c;
    --m3e-icon-size: 26px;
    transition: rotate 0.8s cubic-bezier(0.2, 0.7, 0.2, 1);
  }
  .icon m3e-icon {
    z-index: 1;
  }
  .card:hover .icon m3e-shape {
    rotate: 30deg;
    transition: rotate 0.8s cubic-bezier(0.2, 0.7, 0.2, 1);
  }
  h3 {
    margin: 22px 0 8px;
    font-family: var(--display);
    font-weight: 400;
    font-size: 28px;
    line-height: 1.1;
  }
  p {
    margin: 0;
    max-width: 520px;
    font-size: 15px;
    line-height: 1.6;
    color: var(--muted);
  }
  @media (max-width: 960px) {
    .grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }
  /* Phones: one card at a time, swiped sideways, the next one peeking in. */
  @media (max-width: 620px) {
    .grid {
      grid-template-columns: none;
      grid-auto-flow: column;
      grid-auto-columns: 84%;
      gap: 12px;
      margin: 36px -16px 0;
      padding: 4px 16px 8px;
      overflow-x: auto;
      overscroll-behavior-x: contain;
      scroll-snap-type: x mandatory;
      scroll-padding-inline: 16px;
      scrollbar-width: none;
    }
    .grid::-webkit-scrollbar {
      display: none;
    }
    .card {
      scroll-snap-align: start;
      min-height: 250px;
    }
    .card:hover {
      transform: none;
    }
    .card {
      padding: 22px;
      border-radius: 24px;
    }
    h3 {
      font-size: 25px;
      margin-top: 16px;
    }
    .card.span {
      grid-column: auto;
    }
  }
</style>
