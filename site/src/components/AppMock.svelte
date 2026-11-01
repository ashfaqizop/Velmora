<script lang="ts">
  import Logo from "./Logo.svelte";
  import Shapes from "./Shapes.svelte";

  // A drawing of the app, not a screenshot: the titles are made up and the
  // posters are gradients, so no studio artwork ends up on the site.
  const POSTERS = [
    { name: "Paper Moons", a: "#ff7a59", b: "#3b1d6e", motif: "sun", progress: 0.62 },
    { name: "Low Tide", a: "#2dd4bf", b: "#0f2747", motif: "waves", progress: 0.3 },
    { name: "The Ninth Lantern", a: "#fbbf24", b: "#4a1530", motif: "lantern", progress: 0.85 },
    { name: "Echo Valley", a: "#a78bfa", b: "#14213d", motif: "peaks", progress: 0.12 },
    { name: "Glass Orchard", a: "#f472b6", b: "#1e1b4b", motif: "sun", progress: 0.48 },
    { name: "Saltwater Kings", a: "#60a5fa", b: "#1b1035", motif: "waves", progress: 0.71 },
  ];
  const NAV = [
    ["home", "Home", true],
    ["search", "Search", false],
    ["wand_stars", "Ani", false],
    ["bookmark", "My List", false],
    ["download", "Downloads", false],
    ["settings", "Settings", false],
  ] as const;
</script>

<div class="stage" aria-hidden="true">
  <!-- Behind the window: they peek out around its edges. -->
  <Shapes
    spots={[
      { x: "3%", y: "6%", size: 190, color: "#7b6cff", spin: 26 },
      { x: "96%", y: "2%", size: 130, color: "#ff5a5a", spin: -18, offset: 4 },
      { x: "101%", y: "52%", size: 84, color: "#2dd4bf", spin: 14, offset: 7 },
      { x: "5%", y: "96%", size: 120, color: "#ffb74d", spin: -16, offset: 2 },
      { x: "93%", y: "100%", size: 200, color: "#c38bff", spin: 30, offset: 9 },
    ]}
  />
  <div class="window">
    <div class="titlebar">
      <Logo size={18} />
      <div class="search"><m3e-icon name="search"></m3e-icon>Search movies, series and anime</div>
      <span class="dots"><i></i><i></i><i></i></span>
    </div>
    <div class="body">
      <nav>
        {#each NAV as [icon, label, on] (label)}
          <span class="nav" class:on><m3e-icon name={icon} filled={on}></m3e-icon>{label}</span>
        {/each}
        <div class="promo">
          <strong>Can't hear the dialogue?</strong>
          <span>Turn on Dialogue boost in the player.</span>
        </div>
      </nav>
      <main>
        <div class="feature">
          <div class="art">
            <span class="orb"></span>
            <span class="ridge r1"></span>
            <span class="ridge r2"></span>
          </div>
          <div class="feature-text">
            <span class="kicker">Tonight's pick</span>
            <strong class="display">Midnight Harbor</strong>
            <span class="facts">2026 · Mystery · 2h 4m · 4K HDR</span>
            <div class="buttons">
              <span class="play"><m3e-icon name="play_arrow" filled></m3e-icon>Play</span>
              <span class="ghost"><m3e-icon name="add"></m3e-icon>My List</span>
            </div>
          </div>
        </div>
        <span class="rail-title">Continue watching</span>
        <div class="rail">
          {#each POSTERS as p, i (p.name)}
            <div class="poster m-{p.motif}" style:--a={p.a} style:--b={p.b} style:--d="{i * 0.08}s">
              <span class="motif"></span>
              <span class="pname display">{p.name}</span>
              <span class="bar"><i style:width="{p.progress * 100}%"></i></span>
            </div>
          {/each}
        </div>
      </main>
    </div>
  </div>

  <div class="float sources">
    <span class="f-label">Best link</span>
    <div class="src top"><b>4K</b><span>HEVC · 5.1 · 6.2 GB</span><m3e-icon name="check_circle" filled></m3e-icon></div>
    <div class="src"><b>1080p</b><span>H.264 · 2.1 GB</span></div>
    <div class="src"><b>720p</b><span>H.264 · 900 MB</span></div>
  </div>

  <div class="float ani">
    <span class="on-shape mark"><m3e-shape name="very-sunny"></m3e-shape><m3e-icon name="wand_stars" filled></m3e-icon></span>
    <span>Since you finished <b>Low Tide</b>, try <b>Echo Valley</b>. Same writer, slower burn.</span>
  </div>

  <div class="float sound">
    <m3e-icon name="record_voice_over"></m3e-icon>Dialogue boost
    <span class="toggle"><i></i></span>
  </div>
</div>

<style>
  .stage {
    position: relative;
    /* Keeps the shapes behind the window but above the page background. */
    isolation: isolate;
    width: min(1100px, 100%);
    margin: 0 auto;
    perspective: 1800px;
  }
  .window {
    position: relative;
    border-radius: 22px;
    overflow: hidden;
    background: #14111d;
    border: 1px solid rgb(255 255 255 / 10%);
    box-shadow:
      0 0 0 1px rgb(0 0 0 / 40%),
      0 40px 120px -30px rgb(123 108 255 / 45%),
      0 30px 80px -40px rgb(255 90 90 / 35%);
    transform: rotateX(14deg) scale(0.96);
    transform-origin: 50% 0;
    animation: settle 1.6s 0.3s cubic-bezier(0.2, 0.7, 0.2, 1) forwards;
    font-size: 13px;
  }
  @keyframes settle {
    to {
      transform: none;
    }
  }
  .titlebar {
    display: flex;
    align-items: center;
    gap: 20px;
    height: 52px;
    padding: 0 18px;
    border-bottom: 1px solid var(--line);
    --md-sys-color-on-surface: #f4f1fb;
  }
  .search {
    flex: 1;
    max-width: 420px;
    margin-inline: auto;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 32px;
    padding: 0 14px;
    border-radius: 999px;
    background: rgb(255 255 255 / 6%);
    color: var(--muted);
    font-size: 12px;
    --m3e-icon-size: 16px;
  }
  .dots {
    display: flex;
    gap: 6px;
  }
  .dots i {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: rgb(255 255 255 / 14%);
  }
  .body {
    display: grid;
    grid-template-columns: 178px minmax(0, 1fr);
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 14px 10px;
    border-right: 1px solid var(--line);
  }
  .nav {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 36px;
    padding: 0 12px;
    border-radius: 999px;
    color: var(--muted);
    --m3e-icon-size: 18px;
  }
  .nav.on {
    background: rgb(123 108 255 / 22%);
    color: #fff;
  }
  .promo {
    margin-top: auto;
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 14px;
    border-radius: 16px;
    background: linear-gradient(160deg, rgb(255 90 90 / 18%), rgb(123 108 255 / 18%));
    font-size: 11px;
    color: var(--muted);
  }
  .promo strong {
    color: #fff;
    font-size: 12px;
  }
  main {
    padding: 16px 18px 20px;
    min-width: 0;
  }
  .feature {
    position: relative;
    height: 230px;
    border-radius: 18px;
    overflow: hidden;
  }
  .art {
    position: absolute;
    inset: 0;
    background:
      radial-gradient(120% 90% at 85% 10%, #ff8a65 0%, transparent 45%),
      radial-gradient(90% 80% at 20% 100%, #3b2a8c 0%, transparent 60%),
      linear-gradient(180deg, #2a1846, #0d0b1c);
  }
  .orb {
    position: absolute;
    right: 16%;
    top: 22%;
    width: 110px;
    height: 110px;
    border-radius: 50%;
    background: radial-gradient(circle at 40% 40%, #fff3d6, #ffb36b 55%, #ff6b5a);
    box-shadow: 0 0 80px 20px rgb(255 140 90 / 45%);
  }
  .ridge {
    position: absolute;
    inset: auto 0 0;
    height: 55%;
    background: #120d24;
    clip-path: polygon(0 60%, 14% 38%, 26% 55%, 42% 22%, 58% 50%, 70% 30%, 84% 52%, 100% 34%, 100% 100%, 0 100%);
  }
  .ridge.r1 {
    opacity: 0.6;
    transform: translateY(-14%);
    background: #2a1d55;
  }
  .feature-text {
    position: absolute;
    inset: auto 0 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 22px;
    background: linear-gradient(0deg, rgb(10 8 20 / 85%), transparent);
  }
  .kicker {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: #ffb4a6;
  }
  .feature-text strong {
    font-size: 40px;
    line-height: 1;
  }
  .facts {
    font-size: 11px;
    color: var(--muted);
  }
  .buttons {
    display: flex;
    gap: 8px;
    margin-top: 8px;
  }
  .play,
  .ghost {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 32px;
    padding: 0 16px 0 12px;
    border-radius: 999px;
    font-weight: 700;
    font-size: 12px;
    --m3e-icon-size: 18px;
  }
  .play {
    background: #fff;
    color: #140f22;
  }
  .ghost {
    background: rgb(255 255 255 / 12%);
  }
  .rail-title {
    display: block;
    margin: 18px 0 10px;
    font-family: var(--display);
    font-size: 22px;
  }
  .rail {
    display: grid;
    grid-template-columns: repeat(6, minmax(0, 1fr));
    gap: 12px;
  }
  .poster {
    position: relative;
    aspect-ratio: 2 / 3;
    border-radius: 12px;
    overflow: hidden;
    background: linear-gradient(165deg, var(--a), var(--b) 75%);
    animation: rise 1s calc(0.9s + var(--d)) both cubic-bezier(0.2, 0.7, 0.2, 1);
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(16px);
    }
  }
  .motif {
    position: absolute;
    inset: 0;
  }
  .m-sun .motif {
    background: radial-gradient(circle at 50% 38%, rgb(255 255 255 / 85%) 0 14%, transparent 15%);
    mix-blend-mode: soft-light;
  }
  .m-waves .motif {
    background: repeating-radial-gradient(circle at 50% 120%, rgb(255 255 255 / 22%) 0 3px, transparent 3px 14px);
  }
  .m-peaks .motif {
    background: rgb(0 0 0 / 35%);
    clip-path: polygon(0 75%, 30% 40%, 50% 60%, 72% 30%, 100% 65%, 100% 100%, 0 100%);
  }
  .m-lantern .motif {
    background: radial-gradient(ellipse 18% 24% at 50% 42%, #fff7d1, rgb(255 200 80 / 60%) 60%, transparent 70%);
  }
  .pname {
    position: absolute;
    inset: auto 10px 18px;
    font-size: 15px;
    line-height: 1.05;
    text-shadow: 0 2px 12px rgb(0 0 0 / 50%);
  }
  .bar {
    position: absolute;
    inset: auto 10px 8px;
    height: 3px;
    border-radius: 3px;
    background: rgb(255 255 255 / 25%);
  }
  .bar i {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: #fff;
  }

  /* Floating cards around the window. */
  .float {
    position: absolute;
    border-radius: 18px;
    background: rgb(28 23 42 / 82%);
    border: 1px solid rgb(255 255 255 / 12%);
    backdrop-filter: blur(18px) saturate(1.4);
    box-shadow: 0 24px 60px -20px rgb(0 0 0 / 70%);
    font-size: 13px;
    animation:
      pop 0.9s both cubic-bezier(0.2, 0.7, 0.2, 1),
      bob 7s ease-in-out infinite;
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: translateY(20px) scale(0.96);
    }
  }
  @keyframes bob {
    50% {
      translate: 0 -8px;
    }
  }
  .sources {
    left: -48px;
    top: 34%;
    width: 250px;
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    animation-delay: 1.4s, 2.3s;
  }
  .f-label {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--muted);
  }
  .src {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border-radius: 12px;
    color: var(--muted);
    font-size: 12px;
    --m3e-icon-size: 18px;
  }
  .src b {
    min-width: 46px;
    color: #fff;
  }
  .src.top {
    background: linear-gradient(90deg, rgb(123 108 255 / 35%), rgb(255 90 90 / 20%));
    color: #fff;
  }
  .src m3e-icon {
    margin-left: auto;
    color: #b8f5c9;
  }
  .ani {
    right: -40px;
    top: 18%;
    width: 290px;
    padding: 14px;
    display: flex;
    gap: 12px;
    align-items: flex-start;
    line-height: 1.5;
    color: var(--muted);
    animation-delay: 1.7s, 1.2s;
  }
  .ani b {
    color: #fff;
    font-weight: 600;
  }
  .mark {
    flex: none;
    --m3e-shape-size: 34px;
    --m3e-shape-container-color: var(--indigo);
    color: #fff;
    --m3e-icon-size: 18px;
  }
  .mark m3e-icon {
    z-index: 1;
  }
  .sound {
    right: 8%;
    bottom: -26px;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 16px;
    font-weight: 600;
    --m3e-icon-size: 20px;
    animation-delay: 2s, 0.6s;
  }
  .toggle {
    width: 38px;
    height: 22px;
    border-radius: 999px;
    background: var(--indigo);
    position: relative;
    margin-left: 6px;
  }
  .toggle i {
    position: absolute;
    right: 3px;
    top: 3px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #fff;
  }

  @media (max-width: 900px) {
    .float.sources,
    .float.ani {
      display: none;
    }
    .body {
      grid-template-columns: minmax(0, 1fr);
    }
    nav {
      display: none;
    }
    .rail {
      grid-template-columns: repeat(3, minmax(0, 1fr));
    }
    .poster:nth-child(n + 4) {
      display: none;
    }
    .search {
      display: none;
    }
  }
  @media (max-width: 600px) {
    .window {
      border-radius: 16px;
      font-size: 11px;
    }
    .titlebar {
      height: 42px;
      padding: 0 12px;
    }
    main {
      padding: 10px 10px 14px;
    }
    .feature {
      height: 170px;
      border-radius: 12px;
    }
    .orb {
      width: 64px;
      height: 64px;
      top: 16%;
    }
    .feature-text {
      padding: 14px;
    }
    .feature-text strong {
      font-size: 26px;
    }
    .rail-title {
      font-size: 17px;
      margin: 12px 0 8px;
    }
    .rail {
      gap: 8px;
    }
    .pname {
      font-size: 11px;
      inset: auto 6px 14px;
    }
    .sound {
      right: 50%;
      translate: 50% 0;
      bottom: -22px;
      padding: 9px 14px;
      font-size: 12px;
      white-space: nowrap;
    }
  }
</style>
