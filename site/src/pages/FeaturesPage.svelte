<script lang="ts">
  import { onMount } from "svelte";
  import DownloadButton from "../components/DownloadButton.svelte";
  import { latest } from "../lib/latest.svelte";
  import { inview } from "./inview";
  import PageShell from "./PageShell.svelte";
  import Related from "./Related.svelte";
  import Toc from "./Toc.svelte";

  onMount(() => {
    if (!latest.loaded) latest.load();
  });

  interface Feature { icon: string; title: string; text: string }
  interface Group { id: string; title: string; accent: string; lead: string; items: Feature[] }

  // The big ones, in the carousel.
  const HEADLINES = [
    { shape: "12-sided-cookie", color: "#3271fe", title: "One search, every source", sub: "Movies, series, anime and dramas, all at once" },
    { shape: "soft-burst", color: "#ff8a65", title: "The best link, picked", sub: "Ranked by quality, codec, language and size" },
    { shape: "4-leaf-clover", color: "#8fb4ff", title: "A real player inside", sub: "MKV, HEVC, AV1 and 4K, no codec packs" },
    { shape: "puffy-diamond", color: "#2dd4bf", title: "Never a dead end", sub: "A failed link moves to the next source at once" },
    { shape: "sunny", color: "#ffb74d", title: "Theatre", sub: "Everyone watches the same film at the same moment" },
    { shape: "8-leaf-clover", color: "#f8e8c8", title: "Your music, too", sub: "YouTube Music with synced lyrics and a 10-band EQ" },
  ];

  const GROUPS: Group[] = [
    {
      id: "search",
      title: "Find it",
      accent: "fast",
      lead: "Velmora does the searching, sorting and link checking, so you get to the watching.",
      items: [
        { icon: "travel_explore", title: "One search, every source", text: "Velmora checks every source at once for movies, series, anime and dramas. The same title found on different sites shows up once." },
        { icon: "leaderboard", title: "The best link, picked", text: "Links are ranked by quality, codec, language and size, and found while you read a title's page, so Play starts at once." },
        { icon: "sync_alt", title: "Never stuck on a dead link", text: "If a link fails, the next source starts at the same second. You don't go back to search." },
        { icon: "hub", title: "An open engine underneath", text: "Search and links come from UniverseAPI, an open-source engine you can read, run and build on." },
      ],
    },
    {
      id: "playback",
      title: "Watch it",
      accent: "properly",
      lead: "A player that handles what you throw at it, and sound that fits the room.",
      items: [
        { icon: "play_circle", title: "A real player inside", text: "Built on mpv. MKV, HEVC, AV1, AC3 and 4K play without codec packs or extra apps." },
        { icon: "devices_fold", title: "A fallback that always plays", text: "On PCs where mpv can't run, a built-in player takes over, so Play still works." },
        { icon: "graphic_eq", title: "Sound that fits the room", text: "Dialogue boost, Night mode, even volume between sources, and volume up to 200% for quiet videos." },
        { icon: "subtitles", title: "Subtitles and audio tracks", text: "Switch subtitle and audio tracks while you watch, and a failed switch tells you plainly instead of freezing." },
      ],
    },
    {
      id: "library",
      title: "Keep it",
      accent: "yours",
      lead: "Your list, your place in every episode, and your files for the flight.",
      items: [
        { icon: "bookmark", title: "Pick up where you stopped", text: "Continue Watching remembers the exact second, and the next episode is one click away." },
        { icon: "download", title: "Downloads for the flight", text: "Save episodes to a folder you choose and watch them offline." },
        { icon: "sync", title: "AniList and Trakt sync", text: "Connect once and what you finish gets marked as watched on its own." },
        { icon: "cloud_sync", title: "The same on every PC", text: "Your list, progress and settings follow your free account. Sign in on a new PC and they're there." },
      ],
    },
    {
      id: "together",
      title: "Watch it",
      accent: "together",
      lead: "Velmora Social and Theatre turn a solo app into something you can share.",
      items: [
        { icon: "theaters", title: "Theatre", text: "Scheduled screenings that everyone on Velmora watches at the same moment. Book a seat, and the lobby and live chat open 30 minutes before. The chat is never saved." },
        { icon: "groups", title: "Velmora Social", text: "Profiles, follows, and a feed of what the people you follow are watching, on a website of its own. There are no direct messages." },
        { icon: "star", title: "Ratings, reviews and lists", text: "Rate what you finish, write a short review, and build public lists. Sharing what you watch is on by default and one switch turns it off." },
        { icon: "leaderboard", title: "Trending and leaderboards", text: "See what people on Velmora are watching this week, and who rates and follows the most." },
      ],
    },
    {
      id: "more",
      title: "More than",
      accent: "movies",
      lead: "Music, sport, live channels and an assistant that knows your taste.",
      items: [
        { icon: "library_music", title: "Your music, too", text: "Sign in with Google and your YouTube Music comes with you: mixes, playlists, likes, synced lyrics and music videos. Gapless albums, a 10-band EQ, even volume, and downloads as MP3, M4A or Opus." },
        { icon: "sports_soccer", title: "Sports and Live TV", text: "A Where to watch list for sport that names official broadcasters only, and live channels from a public playlist. Some channels may not play." },
        { icon: "psychology", title: "Ani, an assistant", text: "Ask for something like the show you just finished. Ani uses Google Gemini with your own free API key, and your key and taste profile stay on your PC." },
      ],
    },
    {
      id: "trust",
      title: "Open and",
      accent: "respectful",
      lead: "Free, open source, and clear about what it does with your data.",
      items: [
        { icon: "code", title: "Free and open source", text: "The whole app is free under GPL-3.0, with no subscription. Velmora Public shows a few quiet ads, never while a video plays. Max has none." },
        { icon: "verified_user", title: "Signed updates", text: "Velmora updates itself, and checks every update's signature before installing it." },
        { icon: "privacy_tip", title: "You hold the switches", text: "Anonymous usage totals have no ID and no account attached, and one switch in Settings, under Privacy, turns them off. You can export or delete your account any time." },
        { icon: "palette", title: "Make it yours", text: "Material 3 Expressive design that moves on springs, with accent colors to pick from and motion you can tone down." },
      ],
    },
  ];
</script>

<PageShell
  eyebrow="Features"
  title="Everything you need."
  accent="Nothing you don't."
  crumb="Features"
  lead="Velmora does the searching, sorting and link checking, so you get to the watching. Here's all of it."
>
  {#snippet hero()}
    <div class="cta"><DownloadButton /></div>
  {/snippet}

  <section class="pg-section tight">
    <div class="pg-wide">
      <m3e-carousel variant="uncontained" class="car" aria-label="Headline features">
        {#each HEADLINES as h (h.title)}
          <m3e-carousel-item label={h.title} style="--c: {h.color}">
            <div class="slide">
              <span class="art"><m3e-shape name={h.shape}></m3e-shape></span>
            </div>
            <span slot="title">{h.title}</span>
            <span slot="subtitle">{h.sub}</span>
          </m3e-carousel-item>
        {/each}
      </m3e-carousel>
    </div>
  </section>

  <div class="pg-wide pg-split body">
    <Toc target="doc" />
    <div id="doc">
      {#each GROUPS as g (g.id)}
        <section class="pg-section grp" id={g.id}>
          <h2 class="pg-title" use:inview>{g.title}{" "}<em>{g.accent}</em></h2>
          <p class="pg-lead" use:inview={0.05}>{g.lead}</p>
          <div class="pg-grid two">
            {#each g.items as f, i (f.title)}
              <article class="pg-card" use:inview={i * 0.06}>
                <span class="ico"><m3e-icon name={f.icon}></m3e-icon></span>
                <h3>{f.title}</h3>
                <p>{f.text}</p>
              </article>
            {/each}
          </div>
        </section>
      {/each}
    </div>
  </div>

  <Related
    items={[
      { href: "/download/", icon: "download", title: "Download Velmora", text: "Free for Windows 10 and 11. One small installer." },
      { href: "/faq/", icon: "help", title: "Questions answered", text: "Is it free, is it safe, what does it collect, and more." },
      { href: "/changelog/", icon: "history", title: "What's new", text: "Every release, with what changed and what got fixed." },
    ]}
  />
</PageShell>

<style>
  .cta {
    margin-top: 28px;
  }
  section.tight {
    padding-top: 48px;
  }
  .car {
    display: block;
    --m3e-carousel-height: clamp(260px, 30vw, 340px);
    --m3e-carousel-item-width: min(340px, 78vw);
    --m3e-carousel-item-gap: 14px;
    --m3e-carousel-item-shape: 32px;
  }
  .slide {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    padding-bottom: 64px;
    background: radial-gradient(120% 100% at 80% 0%, color-mix(in srgb, var(--c) 55%, #0a0d16), #0a0d16 75%);
  }
  .art {
    display: grid;
    place-items: center;
  }
  .art m3e-shape {
    --m3e-shape-size: clamp(100px, 14vw, 150px);
    --m3e-shape-container-color: var(--c);
    filter: drop-shadow(0 0 36px color-mix(in srgb, var(--c) 70%, transparent));
    animation: spin 40s linear infinite;
  }
  @keyframes spin {
    to {
      rotate: 360deg;
    }
  }
  .body {
    margin-top: 12px;
  }
  .grp {
    scroll-margin-top: 96px;
  }
  .grp:first-child {
    padding-top: 44px;
  }
  .pg-grid.two {
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  }
  .ico {
    display: inline-grid;
    place-items: center;
    width: 52px;
    height: 52px;
    border-radius: 18px;
    background: color-mix(in srgb, var(--blue) 18%, transparent);
    color: var(--blue);
    --m3e-icon-size: 26px;
    transition: transform 0.4s cubic-bezier(0.34, 1.6, 0.5, 1);
  }
  .pg-card:hover .ico {
    transform: rotate(-8deg) scale(1.08);
  }
  @media (prefers-reduced-motion: reduce) {
    .art m3e-shape {
      animation: none;
    }
  }
</style>
