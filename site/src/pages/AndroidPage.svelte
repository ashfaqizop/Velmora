<script lang="ts">
  import { onMount } from "svelte";
  import AndroidDownloadButton from "../components/AndroidDownloadButton.svelte";
  import Logo from "../components/Logo.svelte";
  import { FAQ, FEATURES, ROADMAP, SIGNING_SHA256, STEPS } from "../lib/android";
  import { latest } from "../lib/latest.svelte";
  import { megabytes, REPO_URL, RELEASES_URL, shortDate } from "../lib/release";
  import { inview } from "./inview";
  import Note from "./Note.svelte";
  import PageShell from "./PageShell.svelte";
  import Steps from "./Steps.svelte";

  onMount(() => {
    if (!latest.loaded) latest.load();
  });

  const r = $derived(latest.android);
  const arm64 = $derived(r?.android?.arm64);
  const universal = $derived(r?.android?.universal);

  const POSTERS = [
    ["#ff7a59", "#3b1d6e"],
    ["#2dd4bf", "#0f2747"],
    ["#a78bfa", "#14213d"],
    ["#f472b6", "#1e1b4b"],
  ];
</script>

<PageShell
  eyebrow="Velmora for Android"
  title="All of Velmora,"
  accent="in your pocket."
  crumb="Android"
  lead="Movies, series, anime, live TV, sports, Theatre and music on your phone and tablet. The same free account as desktop, no ads, and it opens offline once you've signed in."
  spots={[
    { x: "91%", y: "180px", size: 120, color: "#3ddc84", spin: 20, opacity: 0.45 },
    { x: "6%", y: "360px", size: 58, color: "#3271fe", spin: -16, offset: 4, opacity: 0.7 },
    { x: "78%", y: "430px", size: 40, color: "#f8e8c8", spin: 12, offset: 2, opacity: 0.75 },
  ]}
>
  {#snippet hero()}
    <div class="release">
      <div class="get">
        <span class="out"><i></i>Out now{r ? ` · version ${r.version}` : ""}</span>
        <AndroidDownloadButton />
        <p class="alt">
          {#if universal}
            Older 32-bit phone? <a href={universal.url}>Get the universal APK</a> ({megabytes(universal.size)}).
          {:else}
            Every file is on the <a href={RELEASES_URL} target="_blank" rel="noopener">releases page</a>.
          {/if}
        </p>
        <div class="facts">
          <span><m3e-icon name="phone_android"></m3e-icon>Phones and tablets</span>
          <span><m3e-icon name="android"></m3e-icon>Android 8 and up</span>
          <span><m3e-icon name="block"></m3e-icon>No ads</span>
          <span><m3e-icon name="update"></m3e-icon>Updates itself</span>
          {#if r?.date}<span class="muted"><m3e-icon name="history"></m3e-icon>Released {shortDate(r.date)}</span>{/if}
        </div>
      </div>

      <div class="phone" aria-hidden="true">
        <div class="screen">
          <div class="notch"></div>
          <div class="top"><Logo size={14} /><span class="icons"><m3e-icon name="download"></m3e-icon><m3e-icon name="search"></m3e-icon></span></div>
          <div class="hero-card"><span class="orb"></span><span class="tag">Tonight in Theatre</span><strong class="display">Midnight Harbor</strong></div>
          <span class="row-title display">Continue watching</span>
          <div class="row">
            {#each POSTERS as [a, b] (a)}<span class="p" style:--a={a} style:--b={b}><i></i></span>{/each}
          </div>
          <span class="row-title display">Live now</span>
          <div class="live">
            <span class="ch"><b></b>Sports 1</span><span class="ch"><b></b>News 24</span>
          </div>
          <div class="tabs">
            <m3e-icon name="home" filled></m3e-icon><m3e-icon name="search"></m3e-icon><m3e-icon name="theaters"></m3e-icon><m3e-icon
              name="library_music"
            ></m3e-icon>
          </div>
        </div>
      </div>
    </div>
  {/snippet}

  <section class="pg-section">
    <div class="pg-wide">
      <span class="eyebrow" use:inview>What's in it</span>
      <h2 class="pg-title" use:inview={0.05}>The whole app, <em>not a lite version.</em></h2>
      <div class="features">
        {#each FEATURES as f, i (f.title)}
          <article class="pg-card feat" style="--c: {f.color}" use:inview={(i % 4) * 0.06}>
            <span class="art">
              <m3e-shape name={f.shape}></m3e-shape>
              <m3e-icon name={f.icon}></m3e-icon>
            </span>
            <h3>{f.title}</h3>
            <p>{f.text}</p>
          </article>
        {/each}
      </div>
    </div>
  </section>

  <section class="pg-section">
    <div class="pg-wide narrow">
      <span class="eyebrow" use:inview>Install</span>
      <h2 class="pg-title" use:inview={0.05}>Ready in <em>a minute.</em></h2>
      <Steps steps={STEPS} />
      <Note label="Android warns about apps from outside the Play Store" icon="shield" variant="secondary">
        That's Play Protect reacting to an app it hasn't seen, not a verdict on Velmora. Every APK is built by GitHub Actions from the
        <a href={REPO_URL} target="_blank" rel="noopener">public source</a> and signed with the same key, so updates install over each other.
        Its SHA-256 fingerprint is <code>{SIGNING_SHA256}</code>.
      </Note>
    </div>
  </section>

  <section class="pg-section">
    <div class="pg-wide">
      <div class="pg-grid duo">
        <div class="pg-card echo" use:inview>
          <m3e-icon name="swap_horiz"></m3e-icon>
          <h3>Moving from Echo Music?</h3>
          <p>
            Velmora's Music is built on Echo, so your library fits. Make a backup in Echo, then Settings › Transfer from Echo Music brings over liked
            songs, playlists, albums, artists and your history.
          </p>
        </div>
        <div class="pg-card sync" use:inview={0.08}>
          <m3e-icon name="devices"></m3e-icon>
          <h3>One account, every screen</h3>
          <p>
            Sign in with the account you use on Windows. My List and where you stopped follow you, links from Velmora Social open in the app, and
            Theatre seats are the same seats.
          </p>
        </div>
      </div>
    </div>
  </section>

  <section class="pg-section">
    <div class="pg-wide">
      <span class="eyebrow" use:inview>What's next</span>
      <h2 class="pg-title" use:inview={0.05}>It works today. <em>Now it gets beautiful.</em></h2>
      <p class="pg-lead" use:inview={0.1}>
        Everything in Velmora for Android works and has been tested on real phones and a tablet. The next updates are about how it looks and moves.
        They arrive through the app; there's nothing to download again.
      </p>
      <ol class="road">
        {#each ROADMAP as s, i (s.title)}
          <li class="pg-card" use:inview={i * 0.07}>
            <span class="when">{s.when}</span>
            <span class="rico"><m3e-icon name={s.icon}></m3e-icon></span>
            <h3>{s.title}</h3>
            <p>{s.text}</p>
          </li>
        {/each}
      </ol>
    </div>
  </section>

  <section class="pg-section">
    <div class="pg-wide">
      <span class="eyebrow">Questions</span>
      <h2 class="pg-title">What people <em>ask first.</em></h2>
      <div class="pg-grid qa">
        {#each FAQ as f, i (f.q)}
          <div class="pg-card" use:inview={i * 0.06}>
            <h3>{f.q}</h3>
            <p>{f.a}</p>
          </div>
        {/each}
      </div>
    </div>
  </section>

  <section class="pg-section">
    <div class="pg-wide">
      <div class="pg-grid end">
        <a class="pg-card credit" href="https://github.com/EchoMusicApp/Echo-Music" target="_blank" rel="noopener" use:inview>
          <span class="tag">Built on</span>
          <h3>Echo Music</h3>
          <p>
            Velmora for Android is built on Echo Music by iad1tya, the open-source YouTube Music app (GPL-3.0). Its player, lyrics and much of its design
            are the base we build on. Thank you.
          </p>
          <span class="go">See Echo Music<m3e-icon name="arrow_outward"></m3e-icon></span>
        </a>
        <div class="pg-card follow" use:inview={0.08}>
          <span class="tag">Follow along</span>
          <h3>See each update land</h3>
          <p>Previews go up on Instagram, and every change is public on GitHub.</p>
          <div class="links">
            <a href="https://instagram.com/stepbyte.studios" target="_blank" rel="noopener"><m3e-icon name="photo_camera"></m3e-icon>@stepbyte.studios</a>
            <a href="https://instagram.com/ashfaqizop" target="_blank" rel="noopener"><m3e-icon name="photo_camera"></m3e-icon>@ashfaqizop</a>
            <a href="/changelog/"><m3e-icon name="history"></m3e-icon>What's new</a>
            <a href={REPO_URL} target="_blank" rel="noopener"><m3e-icon name="star"></m3e-icon>Star on GitHub</a>
          </div>
        </div>
      </div>
    </div>
  </section>
</PageShell>

<style>
  /* ── Hero: the download and a phone ── */
  .release {
    --green: #3ddc84;
    position: relative;
    overflow: hidden;
    margin-top: 40px;
    display: grid;
    grid-template-columns: minmax(0, 1fr) 260px;
    gap: 40px;
    align-items: end;
    padding: 36px 40px 0;
    border-radius: 36px;
    border: 1px solid rgb(61 220 132 / 24%);
    background:
      radial-gradient(70% 90% at 100% 100%, rgb(61 220 132 / 18%), transparent 60%),
      radial-gradient(60% 80% at 0% 0%, rgb(50 113 254 / 14%), transparent 60%),
      var(--glass);
    backdrop-filter: blur(14px);
  }
  .get {
    padding-bottom: 36px;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 18px;
  }
  .out {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: 12.5px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--green);
  }
  .out i {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--green);
    animation: ping 2s infinite;
  }
  @keyframes ping {
    0% {
      box-shadow: 0 0 0 0 rgb(61 220 132 / 60%);
    }
    70%,
    100% {
      box-shadow: 0 0 0 10px rgb(61 220 132 / 0%);
    }
  }
  .alt {
    margin: -6px 0 0;
    font-size: 14px;
    color: var(--muted);
  }
  .alt a {
    color: #8ff0b8;
  }
  .facts {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .facts span {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 7px 14px 7px 11px;
    border-radius: 999px;
    background: rgb(255 255 255 / 5%);
    border: 1px solid var(--line);
    font-size: 14px;
    color: var(--ink);
    --m3e-icon-size: 18px;
  }
  .facts .muted {
    color: var(--muted);
  }

  .phone {
    width: 260px;
    height: 360px;
    padding: 11px 11px 0;
    border-radius: 38px 38px 0 0;
    background: #26222f;
    box-shadow:
      inset 0 0 0 1px rgb(255 255 255 / 12%),
      0 -24px 70px -20px rgb(61 220 132 / 40%);
    animation: peek 6s ease-in-out infinite alternate;
  }
  @keyframes peek {
    to {
      transform: translateY(-12px);
    }
  }
  .screen {
    position: relative;
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 9px;
    padding: 28px 12px 0;
    border-radius: 28px 28px 0 0;
    overflow: hidden;
    background: #0a0d16;
    font-size: 10px;
  }
  .notch {
    position: absolute;
    left: 50%;
    top: 9px;
    width: 10px;
    height: 10px;
    translate: -50% 0;
    border-radius: 50%;
    background: #000;
  }
  .top {
    display: flex;
    justify-content: space-between;
    align-items: center;
    color: var(--muted);
  }
  .icons {
    display: flex;
    gap: 8px;
    --m3e-icon-size: 14px;
  }
  .hero-card {
    position: relative;
    height: 110px;
    border-radius: 16px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    gap: 2px;
    padding: 9px;
    background:
      radial-gradient(100% 90% at 85% 10%, #ff8a65, transparent 50%),
      linear-gradient(180deg, #2a1846, #0d0b1c);
  }
  .orb {
    position: absolute;
    right: 18px;
    top: 16px;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: radial-gradient(circle at 40% 40%, #fff3d6, #ffb36b 55%, #ff6b5a);
  }
  .hero-card .tag {
    position: relative;
    font-size: 8.5px;
    letter-spacing: 0.08em;
    color: #ffd2c2;
  }
  .hero-card strong {
    position: relative;
    font-size: 19px;
  }
  .row-title {
    font-size: 13px;
  }
  .row {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 6px;
  }
  .p {
    position: relative;
    aspect-ratio: 2 / 3;
    border-radius: 8px;
    overflow: hidden;
    background: linear-gradient(165deg, var(--a), var(--b) 78%);
  }
  .p i {
    position: absolute;
    left: 0;
    bottom: 0;
    height: 3px;
    width: 60%;
    background: #3271fe;
  }
  .p:nth-child(2) i {
    width: 30%;
  }
  .p:nth-child(3) i {
    width: 85%;
  }
  .p:nth-child(4) i {
    width: 15%;
  }
  .live {
    display: flex;
    gap: 6px;
  }
  .ch {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 5px 8px;
    border-radius: 8px;
    background: rgb(255 255 255 / 6%);
    color: var(--ink);
  }
  .ch b {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #ff5a5a;
  }
  .tabs {
    margin-top: auto;
    display: flex;
    justify-content: space-around;
    padding: 10px 0 12px;
    border-top: 1px solid var(--line);
    color: var(--muted);
    --m3e-icon-size: 17px;
  }
  .tabs m3e-icon:first-child {
    color: #8fb4ff;
  }

  /* ── Features ── */
  .features {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: 16px;
    margin-top: 34px;
  }
  .feat h3 {
    margin-top: 18px;
  }
  .art {
    position: relative;
    display: grid;
    place-items: center;
    width: 64px;
    height: 64px;
  }
  .art m3e-shape {
    position: absolute;
    inset: 0;
    --m3e-shape-size: 64px;
    --m3e-shape-container-color: color-mix(in srgb, var(--c) 26%, transparent);
    animation: spin 30s linear infinite;
  }
  .feat:hover .art m3e-shape {
    --m3e-shape-container-color: color-mix(in srgb, var(--c) 48%, transparent);
  }
  .art m3e-icon {
    position: relative;
    color: var(--c);
    --m3e-icon-size: 28px;
  }
  @keyframes spin {
    to {
      rotate: 360deg;
    }
  }

  /* ── Install ── */
  .narrow {
    width: min(820px, 100% - 48px);
    margin-inline: auto;
  }
  code {
    font-size: 12px;
    word-break: break-all;
  }

  /* ── Echo and sync ── */
  .duo {
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  }
  .duo m3e-icon {
    --m3e-icon-size: 30px;
  }
  .echo {
    background:
      radial-gradient(120% 120% at 100% 0%, rgb(248 232 200 / 12%), transparent 60%),
      var(--glass);
  }
  .echo m3e-icon {
    color: var(--cream);
  }
  .sync {
    background:
      radial-gradient(120% 120% at 100% 0%, rgb(50 113 254 / 16%), transparent 60%),
      var(--glass);
  }
  .sync m3e-icon {
    color: #8fb4ff;
  }

  /* ── What's next ── */
  .road {
    list-style: none;
    margin: 34px 0 0;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: 16px;
  }
  .road li {
    position: relative;
  }
  .when {
    position: absolute;
    top: 22px;
    right: 22px;
    padding: 4px 11px;
    border-radius: 999px;
    font-size: 11.5px;
    font-weight: 700;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: #c9c1ff;
    background: rgb(201 193 255 / 12%);
    border: 1px solid rgb(201 193 255 / 30%);
  }
  .rico {
    display: grid;
    place-items: center;
    width: 52px;
    height: 52px;
    border-radius: 18px;
    background: rgb(201 193 255 / 12%);
    color: #c9c1ff;
    --m3e-icon-size: 26px;
  }

  /* ── Questions and the end ── */
  .qa h3 {
    margin-top: 0;
  }
  .tag {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--muted);
  }
  .credit {
    background:
      radial-gradient(120% 120% at 100% 0%, rgb(230 60 40 / 16%), transparent 60%),
      var(--glass);
  }
  .credit h3,
  .follow h3 {
    font-size: 24px;
  }
  .go {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    margin-top: 18px;
    font-weight: 600;
    color: #ffb3a7;
    --m3e-icon-size: 18px;
  }
  .follow {
    background:
      radial-gradient(120% 120% at 100% 0%, rgb(61 220 132 / 12%), transparent 60%),
      var(--glass);
  }
  .links {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 18px;
  }
  .links a {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 8px 14px 8px 11px;
    border-radius: 999px;
    border: 1px solid var(--line);
    background: rgb(255 255 255 / 5%);
    color: var(--ink);
    text-decoration: none;
    font-size: 14px;
    --m3e-icon-size: 18px;
  }
  .links a:hover {
    border-color: rgb(61 220 132 / 55%);
    background: rgb(61 220 132 / 10%);
  }

  @media (max-width: 760px) {
    .release {
      grid-template-columns: minmax(0, 1fr);
      padding: 28px 22px 0;
      border-radius: 28px;
    }
    .get {
      padding-bottom: 0;
    }
    .phone {
      justify-self: center;
      width: 220px;
      height: 300px;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .phone,
    .art m3e-shape,
    .out i {
      animation: none;
    }
  }
</style>
