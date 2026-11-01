<script lang="ts">
  import { screen } from "./lib/media.svelte";
  import { onMount } from "svelte";
  import Nav from "./components/Nav.svelte";
  import AppMock from "./components/AppMock.svelte";
  import DownloadButton from "./components/DownloadButton.svelte";
  import Features from "./components/Features.svelte";
  import AniSection from "./components/AniSection.svelte";
  import Privacy from "./components/Privacy.svelte";
  import Install from "./components/Install.svelte";
  import Backdrop from "./components/Backdrop.svelte";
  import Faq from "./components/Faq.svelte";
  import Footer from "./components/Footer.svelte";
  import Ad from "./components/Ad.svelte";
  import Shapes from "./components/Shapes.svelte";
  import PoweredBy from "./components/PoweredBy.svelte";
  import AndroidSoon from "./components/AndroidSoon.svelte";
  import DownloadCount from "./components/DownloadCount.svelte";
  import { latest } from "./lib/latest.svelte";
  import { REPO_URL } from "./lib/release";
  import { reveal } from "./lib/reveal";

  onMount(() => latest.load());

  const STATS = [
    ["10+", "sources searched at once"],
    ["4K", "HEVC, AV1 and HDR in the player"],
    ["0", "accounts, servers or trackers"],
    ["100%", "open source, GPL-3.0"],
  ];
</script>

<Backdrop />
<Nav />

<main id="top">
  <section class="hero">
    <Shapes
      spots={[
        { x: "9%", y: "200px", size: 120, color: "#7b6cff", spin: 22 },
        { x: "90%", y: "240px", size: 92, color: "#ff5a5a", spin: -16, offset: 4 },
        { x: "17%", y: "480px", size: 52, color: "#ffb74d", spin: 12, offset: 7 },
        { x: "82%", y: "530px", size: 64, color: "#c38bff", spin: -20, offset: 2 },
      ]}
    />
    <div class="wrap center">
      <a class="pill" href="#ani">
        <span class="new">New</span>Velmora 0.2: meet Ani, and better sound
        <m3e-icon name="arrow_forward"></m3e-icon>
      </a>
      <h1 class="display">Every source.<br /><em>One search.</em></h1>
      <p class="sub">
        Velmora is a free Windows app for movies, series and anime. It searches many sites at once, picks the best link and plays it in a built-in 4K
        player.
      </p>
      <div class="ctas">
        <DownloadButton />
        <m3e-button variant="text" size={screen.big} href={REPO_URL} target="_blank" class="gh">
          <m3e-icon slot="icon" name="code"></m3e-icon>View the source
        </m3e-button>
      </div>
      <div class="pills">
        <DownloadCount />
        <a class="soon" href="#android"><m3e-icon name="android"></m3e-icon>Android coming soon</a>
      </div>
    </div>
    <div class="mock wrap"><AppMock /></div>
  </section>

  <div class="stats wrap" use:reveal>
    {#each STATS as [n, label] (label)}
      <div><strong class="display">{n}</strong><span>{label}</span></div>
    {/each}
  </div>

  <a class="engine-link" href="/universeapi/" use:reveal>
    <PoweredBy height={screen.phone ? 12 : 18} />
    <span class="engine-text">See how the engine works</span>
    <m3e-icon name="arrow_forward"></m3e-icon>
  </a>

  <Features />
  <Ad kind="native" />
  <AniSection />
  <Privacy />
  <Install />
  <section id="android">
    <div class="wrap" use:reveal><AndroidSoon /></div>
  </section>
  <Ad kind="banner" />
  <Faq />

  <section class="final">
    <div class="wrap">
      <div class="end" use:reveal>
        <Shapes
          spots={[
            { x: "8%", y: "26%", size: 110, color: "#ff5a5a", spin: 18, opacity: 0.85 },
            { x: "92%", y: "72%", size: 130, color: "#7b6cff", spin: -24, offset: 5 },
            { x: "86%", y: "18%", size: 44, color: "#ffb74d", spin: 10, offset: 8 },
          ]}
        />
        <h2 class="title">Tonight's pick is <em>waiting.</em></h2>
        <p class="lead">Free, open source and ready in a minute.</p>
        <DownloadButton />
        <div class="end-count"><DownloadCount /></div>
      </div>
    </div>
  </section>
</main>

<Footer />

<style>
  .hero {
    padding-top: 150px;
  }
  .center {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
  }
  .pill {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    padding: 6px 14px 6px 6px;
    border-radius: 999px;
    border: 1px solid rgb(255 255 255 / 12%);
    background: rgb(255 255 255 / 5%);
    backdrop-filter: blur(10px);
    font-size: 14px;
    text-decoration: none;
    color: var(--muted);
    --m3e-icon-size: 16px;
    transition: border-color 0.3s;
    animation: in 0.9s 0.05s both cubic-bezier(0.2, 0.7, 0.2, 1);
  }
  .pill:hover {
    border-color: rgb(123 108 255 / 60%);
  }
  .new {
    padding: 3px 10px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 700;
    background: linear-gradient(90deg, var(--coral), var(--indigo));
    color: #fff;
  }
  h1 {
    margin: 28px 0 0;
    font-size: clamp(46px, 12vw, 132px);
    line-height: 0.92;
    letter-spacing: -0.025em;
    animation: in 1s 0.15s both cubic-bezier(0.2, 0.7, 0.2, 1);
  }
  h1 em {
    font-style: italic;
    background: linear-gradient(100deg, #ff8a7a 0%, var(--coral) 25%, #c38bff 60%, var(--indigo) 90%);
    background-size: 200% 100%;
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
    padding-right: 0.08em;
    animation: shimmer 9s ease-in-out infinite alternate;
  }
  @keyframes shimmer {
    to {
      background-position: 100% 0;
    }
  }
  .sub {
    margin: 28px 0 0;
    max-width: 640px;
    font-size: clamp(17px, 1.6vw, 20px);
    line-height: 1.6;
    color: var(--muted);
    animation: in 1s 0.3s both cubic-bezier(0.2, 0.7, 0.2, 1);
  }
  .ctas {
    display: flex;
    align-items: flex-start;
    gap: 16px;
    margin-top: 40px;
    animation: in 1s 0.45s both cubic-bezier(0.2, 0.7, 0.2, 1);
  }
  .ctas :global(.dl) {
    align-items: center;
  }
  .gh {
    --m3e-text-button-label-text-color: var(--ink);
    --m3e-text-button-icon-color: var(--ink);
  }
  .pills {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 10px;
    margin-top: 22px;
    animation: in 1s 0.55s both cubic-bezier(0.2, 0.7, 0.2, 1);
  }
  .soon {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 5px 12px 5px 9px;
    border-radius: 999px;
    font-size: 13px;
    font-weight: 600;
    text-decoration: none;
    color: #8ff0b8;
    background: rgb(61 220 132 / 10%);
    border: 1px solid rgb(61 220 132 / 22%);
    --m3e-icon-size: 16px;
  }
  @keyframes in {
    from {
      opacity: 0;
      transform: translateY(24px);
    }
  }
  .mock {
    margin-top: 80px;
    animation: in 1.2s 0.5s both cubic-bezier(0.2, 0.7, 0.2, 1);
  }

  .stats {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    margin-top: 120px;
    border-top: 1px solid var(--line);
    border-bottom: 1px solid var(--line);
  }
  .stats div {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 36px 24px;
  }
  .stats div + div {
    border-left: 1px solid var(--line);
  }
  .stats strong {
    font-size: 56px;
    line-height: 1;
    background: linear-gradient(180deg, #fff, #b9b0d6);
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
  }
  .stats span {
    font-size: 14px;
    color: var(--muted);
  }

  .engine-link {
    display: flex;
    width: fit-content;
    align-items: center;
    gap: 14px;
    margin: 40px auto 0;
    padding: 12px 20px;
    border-radius: 999px;
    border: 1px solid var(--line);
    background: rgb(255 255 255 / 3%);
    text-decoration: none;
    font-size: 14px;
    color: var(--muted);
    --md-sys-color-on-surface-variant: var(--muted);
    --md-sys-color-on-surface: #f4f1fb;
    --m3e-icon-size: 18px;
    transition: border-color 0.3s;
  }
  .engine-link:hover {
    border-color: rgb(227 32 44 / 50%);
  }
  .end {
    position: relative;
    isolation: isolate;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: 96px 32px;
    border-radius: 44px;
    overflow: hidden;
    border: 1px solid rgb(255 255 255 / 10%);
    background:
      radial-gradient(60% 80% at 20% 0%, rgb(255 90 90 / 30%), transparent 60%),
      radial-gradient(60% 90% at 90% 100%, rgb(123 108 255 / 40%), transparent 60%),
      #140f22;
  }
  .end-count {
    margin-top: 18px;
  }
  .end .lead {
    margin-bottom: 36px;
  }
  .end :global(.dl) {
    align-items: center;
  }

  @media (max-width: 760px) {
    .hero {
      padding-top: 120px;
    }
    .ctas {
      flex-direction: column;
      align-items: center;
    }
    .stats {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
    .stats div:nth-child(3) {
      border-left: 0;
    }
    .stats div:nth-child(n + 3) {
      border-top: 1px solid var(--line);
    }
  }
  @media (max-width: 600px) {
    .hero {
      padding-top: 104px;
    }
    .pill {
      font-size: 12.5px;
      text-align: left;
    }
    h1 {
      margin-top: 22px;
    }
    .sub {
      margin-top: 20px;
    }
    .ctas {
      margin-top: 30px;
      gap: 6px;
    }
    .mock {
      margin-top: 48px;
    }
    .stats {
      margin-top: 72px;
    }
    .stats div {
      padding: 24px 16px;
    }
    .stats strong {
      font-size: 40px;
    }
    /* Just the mark and an arrow on phones. */
    .engine-link {
      gap: 8px;
      margin-top: 28px;
      padding: 9px 14px;
    }
    .engine-text {
      display: none;
    }
    .end {
      padding: 64px 20px;
      border-radius: 32px;
    }
  }
</style>
