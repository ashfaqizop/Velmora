<script lang="ts">
  import { screen } from "./lib/media.svelte";
  import { onMount } from "svelte";
  import Backdrop from "./components/Backdrop.svelte";
  import Nav from "./components/Nav.svelte";
  import Footer from "./components/Footer.svelte";
  import PoweredBy from "./components/PoweredBy.svelte";
  import DownloadButton from "./components/DownloadButton.svelte";
  import Shapes from "./components/Shapes.svelte";
  import { latest } from "./lib/latest.svelte";
  import { REPO_URL } from "./lib/release";
  import { reveal } from "./lib/reveal";
  import Dots from "./components/Dots.svelte";

  onMount(() => latest.load());

  const SOURCE_URL = `${REPO_URL}/tree/main/universeapi`;

  // Sources by kind. The orbit shows kinds, not site names.
  const ORBIT = [
    { label: "Streaming APIs", ring: 1, angle: 20 },
    { label: "4K releases", ring: 1, angle: 140 },
    { label: "Season packs", ring: 1, angle: 260 },
    { label: "Anime", ring: 2, angle: 70 },
    { label: "Asian drama", ring: 2, angle: 190 },
    { label: "Torrents", ring: 2, angle: 310 },
    { label: "TMDB", ring: 3, angle: 0 },
    { label: "AniList", ring: 3, angle: 120 },
    { label: "Cinemeta", ring: 3, angle: 240 },
  ];

  const STATS = [
    ["12", "sources"],
    ["15+", "file hosts resolved"],
    ["3", "metadata services"],
    ["1 KB", "probe per link"],
  ];

  const PIPELINE = [
    ["01", "Search", "One query goes out to every source that's switched on."],
    ["02", "Merge", "Results stream back as they arrive. The same title from different sites becomes one."],
    ["03", "Gather", "Pick a title and every source is asked for links, in parallel."],
    ["04", "Probe", "Each link gets a 1 KB range request. Ad pages and decoys are dropped."],
    ["05", "Rank", "Quality, codec, language, size and your preferences decide the order."],
    ["06", "Resolve", "Host pages are turned into files and streams a player can open."],
    ["07", "Play", "A local proxy adds the right headers and rewrites manifests for mpv."],
  ];

  const LAYERS = [
    {
      tag: "metadata",
      title: "Metadata",
      text: "TMDB when a key is set, Cinemeta without one, and AniList for anime: trending, airing today and MAL ids.",
      wide: true,
    },
    { tag: "providers", title: "Sources", text: "Streaming APIs, direct-download sites, anime and drama catalogs, and torrent indexers ranked below direct links." },
    { tag: "resolvers", title: "Resolvers", text: "More than 15 file hosts and video players, turned into playable HLS, DASH or MP4." },
    { tag: "ranking", title: "Verify and rank", text: "A 1 KB range probe before anything is shown, then ranking by your quality, codec and language." },
    { tag: "torrent", title: "Torrents", text: "Streams while it downloads, with a seekable reader. Finds the right episode inside a season pack." },
    {
      tag: "downloads",
      title: "Downloads",
      text: "A live-sized queue with pause and resume. HLS and DASH are remuxed to MKV with every audio and subtitle track.",
      wide: true,
    },
    { tag: "proxy", title: "Player proxy", text: "Register a source, get a local URL. Referer, cookies and byte ranges are handled for the player." },
    { tag: "infra", title: "Resilience", text: "Retries with jittered backoff, Cloudflare detection, a browser-fingerprint fallback, and a memory plus disk cache." },
  ];

  const MIRROR_ORDER = [
    ["Emergency override", "Pinned in the registry when a site is in trouble."],
    ["Last good mirror", "Remembered for 24 hours after it last worked."],
    ["Live host config", "The site's own published list of current domains."],
    ["Mirror list", "The fallback list kept in the registry."],
  ];

  const SPECS = [
    ["Language", "Rust (2021 edition), async on Tokio"],
    ["License", "GPL-3.0-or-later"],
    ["Platforms", "Windows today, Android groundwork in CI"],
    ["Metadata", "TMDB · Cinemeta · AniList"],
    ["Torrents", "librqbit, on by default"],
    ["Remux", "ffmpeg sidecar, HLS and DASH to MKV"],
    ["Cache", "Two levels: memory and disk, with TTLs"],
    ["Config", "registry.json on the config branch, read at launch"],
    ["Testing", "Unit tests, clippy and a daily live smoke test in CI"],
  ];

  // Card rows that swipe sideways on phones.
  let layersTrack = $state<HTMLElement>();
</script>

<Backdrop tone="engine" />
<Nav />

<main id="top" class="engine">
  <!-- Hero: the real lockup over an orbit of everything the engine talks to. -->
  <section class="hero">
    <Shapes
      spots={[
        { x: "4%", y: "170px", size: 64, color: "#ff6b73", spin: 14, offset: 2 },
        { x: "46%", y: "120px", size: 40, color: "#f3c7c9", spin: -10, offset: 6 },
        { x: "6%", y: "640px", size: 100, color: "#8a0d16", spin: -20, opacity: 0.8 },
      ]}
    />
    <div class="wrap hero-grid">
      <div class="copy">
        <span class="mono-tag">open source · rust · gpl-3.0</span>
        <div class="lockup"><PoweredBy variant="full" height={92} /></div>
        <h1>The engine<br />under <span class="red">Velmora.</span></h1>
        <p class="sub">
          UniverseAPI searches every source at once, checks each link, ranks what's left and hands the best one to the player. One Rust crate does the
          catalog, streaming, downloads and torrents.
        </p>
        <div class="ctas">
          <m3e-button variant="filled" size={screen.big} href={SOURCE_URL} target="_blank" class="red-btn">
            <m3e-icon slot="icon" name="code"></m3e-icon>Read the source
          </m3e-button>
          <m3e-button variant="text" size={screen.big} href="#pipeline" class="ghost-btn">
            How it works<m3e-icon slot="trailing-icon" name="arrow_downward"></m3e-icon>
          </m3e-button>
        </div>
      </div>

      <div class="orbit" aria-hidden="true">
        <div class="halo"></div>
        {#each [1, 2, 3] as r (r)}<div class="ring r{r}"></div>{/each}
        {#each ORBIT as o, i (o.label)}
          <div class="spin r{o.ring}" style:--start="{o.angle}deg" style:--i={i}>
            <span class="node"><i></i>{o.label}</span>
          </div>
        {/each}
        <svg class="star" viewBox="0 0 200 200">
          <defs>
            <radialGradient id="core" cx="50%" cy="50%" r="50%">
              <stop offset="0%" stop-color="#ffffff" />
              <stop offset="30%" stop-color="#f3c7c9" />
              <stop offset="100%" stop-color="#c0111a" />
            </radialGradient>
          </defs>
          <g fill="url(#core)">
            <rect x="88" y="0" width="24" height="200" />
            <rect x="0" y="88" width="200" height="24" />
            <rect x="88" y="0" width="24" height="200" transform="rotate(45 100 100)" />
            <rect x="88" y="0" width="24" height="200" transform="rotate(-45 100 100)" />
          </g>
        </svg>
      </div>
    </div>

    <div class="stats wrap" use:reveal>
      {#each STATS as [n, label] (label)}
        <div><strong>{n}</strong><span>{label}</span></div>
      {/each}
    </div>
  </section>

  <!-- Pipeline -->
  <section id="pipeline">
    <Shapes
      spots={[
        { x: "94%", y: "120px", size: 120, color: "#e3202c", spin: 24, opacity: 0.5 },
      ]}
    />
    <div class="wrap">
      <div use:reveal>
        <span class="mono-tag">// pipeline</span>
        <h2>From a search<br />to a <span class="red">playing video.</span></h2>
        <p class="lead">Seven steps, most of them in parallel. It usually takes a few seconds.</p>
      </div>
      <div class="pipe" use:reveal={0.1}>
        <div class="track"><span class="pulse"></span></div>
        {#each PIPELINE as [n, title, text], i (n)}
          <div class="stage" style:--i={i}>
            <span class="dot"></span>
            <span class="n">{n}</span>
            <strong>{title}</strong>
            <span class="t">{text}</span>
          </div>
        {/each}
      </div>
    </div>
  </section>

  <!-- Terminal -->
  <section>
    <div class="wrap split">
      <div use:reveal>
        <span class="mono-tag">// cli</span>
        <h2>Try it from <span class="red">a terminal.</span></h2>
        <p class="lead">
          The engine has a small developer CLI. It's the same code Velmora runs, with nothing in between, and CI uses it for a live check against the
          real sites every day.
        </p>
        <p class="lead">If a critical check fails, the workflow opens an issue on its own, usually before anyone notices in the app.</p>
      </div>
      <div class="term" use:reveal={0.12} aria-label="Example terminal session">
        <div class="term-bar"><i></i><i></i><i></i><span>universeapi</span></div>
        <pre><span class="p">$</span> universeapi sources "dune"
<span class="c">› searching 12 sources…</span>
<span class="c">› 9 answered · merged into 1 title</span>
<span class="c">› probing 31 links · 7 dropped as decoys</span>

<span class="ok">✓</span> <b>2160p</b>  HEVC  HDR   5.1  14.2 GB  direct
<span class="ok">✓</span> <b>2160p</b>  HEVC  SDR   5.1   8.9 GB  direct
<span class="ok">✓</span> <b>1080p</b>  H.264       5.1   3.1 GB  stream
<span class="ok">✓</span> <b>1080p</b>  HEVC        2.0   1.8 GB  stream
<span class="ok">✓</span> <b>1080p</b>  H.264       5.1   2.4 GB  torrent
<span class="dim">  … 19 more</span>

<span class="p">$</span> <span class="caret"></span></pre>
      </div>
    </div>
  </section>

  <!-- Layers -->
  <section>
    <div class="wrap">
      <div use:reveal>
        <span class="mono-tag">// inside the crate</span>
        <h2>Every layer, <span class="red">one library.</span></h2>
      </div>
      <div class="layers swipe" bind:this={layersTrack}>
        {#each LAYERS as l, i (l.tag)}
          <article class:wide={l.wide} use:reveal={(i % 3) * 0.06}>
            <span class="ltag">{l.tag}</span>
            <strong>{l.title}</strong>
            <span>{l.text}</span>
          </article>
        {/each}
      </div>
      {#if screen.phone}<Dots track={layersTrack} labels={LAYERS.map((x) => x.title)} label="Layers" />{/if}
    </div>
  </section>

  <!-- Remote config -->
  <section>
    <div class="wrap split">
      <div use:reveal>
        <span class="mono-tag">// remote config</span>
        <h2>Fixed with a commit, <span class="red">not a release.</span></h2>
        <p class="lead">
          Sites move domains all the time. Nothing domain-specific is built into the app: on every launch Velmora reads a small registry file and
          layers it over its defaults. When a site moves, a one-line commit fixes it for everyone.
        </p>
        <ol class="order">
          {#each MIRROR_ORDER as [title, text], i (title)}
            <li><span class="on">{i + 1}</span><div><strong>{title}</strong><span>{text}</span></div></li>
          {/each}
        </ol>
      </div>
      <div class="diff" use:reveal={0.12} aria-label="Example registry change">
        <div class="term-bar"><i></i><i></i><i></i><span>registry.json</span></div>
        <pre><span class="dim">  &#123;</span>
<span class="dim">    "mirrors": &#123;</span>
<span class="del">-     "catalog": ["mirror-one.example"]</span>
<span class="add">+     "catalog": ["mirror-two.example"]</span>
<span class="dim">    &#125;,</span>
<span class="dim">    "host_aliases": &#123;</span>
<span class="add">+     "filehost": ["filehosting"]</span>
<span class="dim">    &#125;</span>
<span class="dim">  &#125;</span></pre>
        <div class="commit">
          <m3e-icon name="check_circle" filled></m3e-icon>
          <span><b>fix: catalog moved domains</b> · every Velmora picks it up on next launch</span>
        </div>
      </div>
    </div>
  </section>

  <!-- Code -->
  <section>
    <div class="wrap split rev">
      <div class="code" use:reveal aria-label="Rust example">
        <div class="term-bar"><i></i><i></i><i></i><span>main.rs</span></div>
        <pre><span class="k">let</span> engine = <span class="ty">Engine</span>::<span class="fn">new</span>(<span class="ty">EngineOptions</span> &#123;
    registry: <span class="ty">Registry</span>::<span class="fn">default</span>().<span class="fn">with_env_overrides</span>(),
    cache_dir: <span class="ty">Some</span>(app_cache_dir),
    ..<span class="ty">Default</span>::<span class="fn">default</span>()
&#125;)?;
engine.<span class="fn">refresh_registry_default</span>().<span class="k">await</span>?;

<span class="k">let</span> rails = engine.<span class="fn">home</span>().<span class="k">await</span>;
<span class="k">let</span> hits = engine.<span class="fn">search</span>(<span class="s">"dune"</span>).<span class="k">await</span>;
<span class="k">let</span> report = engine
    .<span class="fn">sources</span>(&amp;hits[<span class="num">0</span>], <span class="ty">Target</span>::<span class="ty">Movie</span>, <span class="ty">AudioMode</span>::<span class="ty">Sub</span>)
    .<span class="k">await</span>;</pre>
      </div>
      <div use:reveal={0.1}>
        <span class="mono-tag">// usage</span>
        <h2>A few lines <span class="red">to embed.</span></h2>
        <p class="lead">
          UniverseAPI is a plain library with no UI. Velmora's Tauri shell calls it directly, and anything else written in Rust can too. Secrets
          never live in the repo: TMDB keys come from the environment.
        </p>
      </div>
    </div>
  </section>

  <!-- Specs -->
  <section>
    <div class="wrap narrow">
      <div use:reveal>
        <span class="mono-tag">// spec sheet</span>
        <h2>The <span class="red">details.</span></h2>
      </div>
      <dl class="specs" use:reveal={0.1}>
        {#each SPECS as [k, v] (k)}
          <div><dt>{k}</dt><dd>{v}</dd></div>
        {/each}
      </dl>
    </div>
  </section>

  <section>
    <div class="wrap">
      <div class="end" use:reveal>
        <Shapes
          spots={[
            { x: "9%", y: "30%", size: 110, color: "#e3202c", spin: 18 },
            { x: "91%", y: "70%", size: 120, color: "#8a0d16", spin: -22, offset: 5 },
            { x: "84%", y: "20%", size: 40, color: "#f3c7c9", spin: 12, offset: 8 },
          ]}
        />
        <div class="end-lockup"><PoweredBy variant="full" height={64} /></div>
        <h2>See it <span class="red">in action.</span></h2>
        <p class="lead">Velmora is the app built on UniverseAPI. It's free, and so is the engine.</p>
        <div class="end-ctas">
          <DownloadButton details={false} />
          <m3e-button variant="text" size={screen.big} href={SOURCE_URL} target="_blank" class="ghost-btn">
            <m3e-icon slot="icon" name="code"></m3e-icon>Read the source
          </m3e-button>
        </div>
      </div>
    </div>
  </section>
</main>

<Footer />

<style>
  .engine {
    --red: #e3202c;
    --red-soft: #ff6b73;
    --pink: #f3c7c9;
    --mono: "JetBrains Mono Variable", ui-monospace, "Cascadia Code", monospace;
    --md-sys-color-on-surface: #f4f1fb;
  }
  .red {
    color: var(--red-soft);
  }
  .mono-tag {
    font-family: var(--mono);
    font-size: 13px;
    letter-spacing: 0.04em;
    color: var(--red-soft);
  }
  h1,
  h2 {
    font-family: var(--mono);
    font-weight: 800;
    letter-spacing: -0.04em;
    margin: 14px 0 0;
  }
  h1 {
    font-size: clamp(44px, 5.8vw, 84px);
    line-height: 0.98;
  }
  h2 {
    font-size: clamp(32px, 4vw, 56px);
    line-height: 1.02;
  }

  /* Hero */
  .hero {
    padding-top: 140px;
  }
  .hero-grid {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: 40px;
    align-items: center;
  }
  .lockup {
    margin-top: 22px;
    animation: in 1s 0.05s both cubic-bezier(0.2, 0.7, 0.2, 1);
  }
  .copy h1 {
    margin-top: 32px;
    animation: in 1s 0.15s both cubic-bezier(0.2, 0.7, 0.2, 1);
  }
  .sub {
    margin: 22px 0 0;
    max-width: 560px;
    font-size: 18px;
    line-height: 1.65;
    color: var(--muted);
    animation: in 1s 0.3s both cubic-bezier(0.2, 0.7, 0.2, 1);
  }
  .ctas,
  .end-ctas {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px;
    margin-top: 34px;
    animation: in 1s 0.45s both cubic-bezier(0.2, 0.7, 0.2, 1);
  }
  .red-btn {
    --m3e-filled-button-container-color: var(--red);
    --m3e-filled-button-label-text-color: #fff;
    --m3e-filled-button-icon-color: #fff;
    --m3e-filled-button-hover-label-text-color: #fff;
    --m3e-filled-button-hover-icon-color: #fff;
    --m3e-filled-button-focus-label-text-color: #fff;
    --m3e-filled-button-focus-icon-color: #fff;
    --m3e-filled-button-pressed-label-text-color: #fff;
    --m3e-filled-button-pressed-icon-color: #fff;
    border-radius: 999px;
    box-shadow: 0 18px 50px -14px rgb(227 32 44 / 70%);
  }
  .ghost-btn {
    --m3e-text-button-label-text-color: var(--ink);
    --m3e-text-button-icon-color: var(--ink);
  }
  @keyframes in {
    from {
      opacity: 0;
      transform: translateY(24px);
    }
  }

  /* Orbit */
  .orbit {
    position: relative;
    aspect-ratio: 1;
    width: min(560px, 100%);
    margin-inline: auto;
    animation: in 1.4s 0.3s both cubic-bezier(0.2, 0.7, 0.2, 1);
  }
  .halo {
    position: absolute;
    inset: 22%;
    border-radius: 50%;
    background: radial-gradient(circle, rgb(255 80 90 / 45%), rgb(192 17 26 / 15%) 45%, transparent 70%);
    filter: blur(20px);
    animation: breathe 5s ease-in-out infinite;
  }
  @keyframes breathe {
    50% {
      transform: scale(1.15);
      opacity: 0.75;
    }
  }
  .ring {
    position: absolute;
    border-radius: 50%;
    border: 1px solid rgb(255 255 255 / 9%);
  }
  .ring.r1 {
    inset: 26%;
    border-style: dashed;
    border-color: rgb(255 107 115 / 22%);
  }
  .ring.r2 {
    inset: 13%;
  }
  .ring.r3 {
    inset: 0;
    border-color: rgb(255 255 255 / 6%);
  }
  .star {
    position: absolute;
    inset: 41%;
    width: 18%;
    height: 18%;
    filter: drop-shadow(0 0 24px rgb(255 60 70 / 80%));
    animation: turn 30s linear infinite;
  }
  @keyframes turn {
    to {
      rotate: 360deg;
    }
  }
  /* Each node rides a rotating arm; the label counter-rotates to stay upright. */
  .spin {
    --dur: 60s;
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    rotate: var(--start);
    animation: orbit var(--dur) linear infinite;
  }
  .spin.r1 {
    --radius: 24%;
    --dur: 48s;
  }
  .spin.r2 {
    --radius: 37%;
    --dur: 70s;
    animation-direction: reverse;
  }
  .spin.r3 {
    --radius: 50%;
    --dur: 95s;
  }
  @keyframes orbit {
    from {
      rotate: var(--start);
    }
    to {
      rotate: calc(var(--start) + 360deg);
    }
  }
  .node {
    position: absolute;
    left: 50%;
    top: calc(50% - var(--radius));
    translate: -50% -50%;
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 6px 12px;
    border-radius: 999px;
    white-space: nowrap;
    font-family: var(--mono);
    font-size: 12px;
    background: rgb(24 14 20 / 85%);
    border: 1px solid rgb(255 107 115 / 25%);
    backdrop-filter: blur(8px);
    animation: upright var(--dur) linear infinite;
    animation-direction: inherit;
  }
  @keyframes upright {
    from {
      rotate: calc(var(--start) * -1);
    }
    to {
      rotate: calc(var(--start) * -1 - 360deg);
    }
  }
  .node i {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--red-soft);
    box-shadow: 0 0 10px var(--red);
  }

  /* Stats */
  .stats {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    margin-top: 80px;
    border: 1px solid var(--line);
    border-radius: 28px;
    overflow: hidden;
    background: rgb(255 255 255 / 2%);
  }
  .stats div {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 30px 28px;
  }
  .stats div + div {
    border-left: 1px solid var(--line);
  }
  .stats strong {
    font-family: var(--mono);
    font-size: 48px;
    font-weight: 800;
    letter-spacing: -0.04em;
    line-height: 1;
    background: linear-gradient(180deg, #fff, var(--pink));
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
  }
  .stats span {
    font-family: var(--mono);
    font-size: 12.5px;
    color: var(--muted);
  }

  /* Pipeline */
  .pipe {
    position: relative;
    display: grid;
    grid-template-columns: repeat(7, minmax(0, 1fr));
    gap: 14px;
    margin-top: 56px;
  }
  .track {
    position: absolute;
    left: 0;
    right: 0;
    top: 7px;
    height: 2px;
    background: rgb(255 255 255 / 8%);
    overflow: hidden;
  }
  .pulse {
    position: absolute;
    top: 0;
    width: 22%;
    height: 100%;
    background: linear-gradient(90deg, transparent, var(--red-soft), #fff, transparent);
    animation: run 3.6s cubic-bezier(0.6, 0, 0.4, 1) infinite;
  }
  @keyframes run {
    from {
      left: -22%;
    }
    to {
      left: 100%;
    }
  }
  .stage {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .dot {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #1a1015;
    border: 2px solid var(--red-soft);
    box-shadow: 0 0 0 4px rgb(255 107 115 / 10%);
    animation: blink 3.6s calc(var(--i) * 0.45s) infinite;
  }
  @keyframes blink {
    0%,
    20%,
    100% {
      background: #1a1015;
    }
    8% {
      background: var(--red-soft);
      box-shadow: 0 0 18px 4px rgb(255 80 90 / 60%);
    }
  }
  .n {
    margin-top: 14px;
    font-family: var(--mono);
    font-size: 12px;
    color: var(--red-soft);
  }
  .stage strong {
    font-family: var(--mono);
    font-size: 18px;
    font-weight: 700;
  }
  .t {
    font-size: 14px;
    line-height: 1.55;
    color: var(--muted);
  }

  /* Split layouts */
  .split {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1.05fr);
    gap: 64px;
    align-items: center;
  }
  .split.rev {
    grid-template-columns: minmax(0, 1.1fr) minmax(0, 1fr);
  }

  /* Terminal-style panels */
  .term,
  .diff,
  .code {
    border-radius: 22px;
    overflow: hidden;
    border: 1px solid rgb(255 255 255 / 10%);
    background: #0f0b10;
    box-shadow:
      0 40px 100px -40px rgb(227 32 44 / 45%),
      inset 0 1px 0 rgb(255 255 255 / 5%);
  }
  .term-bar {
    display: flex;
    align-items: center;
    gap: 7px;
    height: 40px;
    padding: 0 16px;
    border-bottom: 1px solid var(--line);
    background: rgb(255 255 255 / 3%);
  }
  .term-bar i {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: rgb(255 255 255 / 14%);
  }
  .term-bar i:first-child {
    background: var(--red);
  }
  .term-bar span {
    margin-left: 10px;
    font-family: var(--mono);
    font-size: 12px;
    color: var(--muted);
  }
  pre {
    margin: 0;
    padding: 22px 22px 26px;
    overflow-x: auto;
    font-family: var(--mono);
    font-size: 13.5px;
    line-height: 1.75;
    color: #e9e3ee;
  }
  .p {
    color: var(--red-soft);
  }
  .c,
  .dim {
    color: #8a8094;
  }
  .ok {
    color: #7ee2a8;
  }
  pre b {
    color: #fff;
  }
  .caret {
    display: inline-block;
    width: 9px;
    height: 17px;
    vertical-align: -3px;
    background: var(--red-soft);
    animation: caret 1.1s steps(1) infinite;
  }
  @keyframes caret {
    50% {
      opacity: 0;
    }
  }
  .del {
    display: block;
    background: rgb(227 32 44 / 16%);
    color: #ffb3b8;
  }
  .add {
    display: block;
    background: rgb(126 226 168 / 12%);
    color: #b5f5cf;
  }
  .commit {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 14px 20px;
    border-top: 1px solid var(--line);
    font-family: var(--mono);
    font-size: 12.5px;
    color: var(--muted);
    --m3e-icon-size: 18px;
  }
  .commit m3e-icon {
    color: #7ee2a8;
  }
  .commit b {
    color: #fff;
    font-weight: 600;
  }
  .k {
    color: #ff7b85;
  }
  .ty {
    color: #f3c7c9;
  }
  .fn {
    color: #9fd3ff;
  }
  .s {
    color: #b5f5cf;
  }
  .num {
    color: #ffcf8a;
  }

  /* Mirror order */
  .order {
    list-style: none;
    margin: 32px 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .order li {
    display: flex;
    gap: 14px;
    align-items: flex-start;
  }
  .on {
    flex: none;
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    border-radius: 10px;
    font-family: var(--mono);
    font-size: 13px;
    font-weight: 700;
    background: rgb(227 32 44 / 18%);
    color: var(--red-soft);
  }
  .order div {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .order strong {
    font-size: 15.5px;
  }
  .order span {
    font-size: 14px;
    color: var(--muted);
  }

  /* Layers */
  .layers {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 14px;
    margin-top: 48px;
  }
  .layers article {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 28px;
    border-radius: 26px;
    border: 1px solid var(--line);
    background:
      radial-gradient(120% 90% at 100% 0%, rgb(227 32 44 / 10%), transparent 55%),
      rgb(255 255 255 / 2.5%);
    transition:
      border-color 0.35s,
      transform 0.4s cubic-bezier(0.2, 0.7, 0.2, 1),
      opacity 0.9s cubic-bezier(0.2, 0.7, 0.2, 1);
  }
  .layers article:hover {
    border-color: rgb(255 107 115 / 40%);
    transform: translateY(-4px);
  }
  .layers article.wide {
    grid-column: span 2;
  }
  .ltag {
    align-self: flex-start;
    padding: 3px 10px;
    border-radius: 8px;
    font-family: var(--mono);
    font-size: 11.5px;
    color: var(--red-soft);
    background: rgb(227 32 44 / 12%);
    border: 1px solid rgb(227 32 44 / 25%);
  }
  .layers strong {
    font-family: var(--mono);
    font-size: 21px;
    font-weight: 700;
    letter-spacing: -0.02em;
    margin-top: 6px;
  }
  .layers span:last-child {
    font-size: 15px;
    line-height: 1.6;
    color: var(--muted);
  }

  /* Specs */
  .narrow {
    max-width: 900px;
  }
  .specs {
    margin: 40px 0 0;
    border-top: 1px solid var(--line);
  }
  .specs div {
    display: grid;
    grid-template-columns: 200px minmax(0, 1fr);
    gap: 24px;
    padding: 18px 4px;
    border-bottom: 1px solid var(--line);
  }
  dt {
    font-family: var(--mono);
    font-size: 13px;
    color: var(--red-soft);
  }
  dd {
    margin: 0;
    font-size: 15.5px;
  }

  .end {
    position: relative;
    isolation: isolate;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: 88px 32px;
    border-radius: 44px;
    border: 1px solid rgb(255 255 255 / 10%);
    background:
      radial-gradient(60% 80% at 50% 0%, rgb(227 32 44 / 32%), transparent 60%),
      radial-gradient(60% 90% at 90% 100%, rgb(91 26 58 / 50%), transparent 60%),
      #140b10;
  }
  .end .lead {
    margin-bottom: 4px;
  }
  .end-ctas {
    justify-content: center;
    animation: none;
  }

  @media (max-width: 1000px) {
    .hero-grid,
    .split,
    .split.rev {
      grid-template-columns: minmax(0, 1fr);
    }
    .split.rev .code {
      order: 2;
    }
    .pipe {
      grid-template-columns: minmax(0, 1fr);
      gap: 24px;
      padding-left: 34px;
    }
    .track {
      left: 7px;
      right: auto;
      top: 0;
      bottom: 0;
      width: 2px;
      height: auto;
    }
    .pulse {
      display: none;
    }
    .dot {
      position: absolute;
      left: -34px;
      top: 0;
    }
    .n {
      margin-top: 0;
    }
    .layers {
      grid-template-columns: minmax(0, 1fr);
    }
    .layers article.wide {
      grid-column: auto;
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
    .specs div {
      grid-template-columns: minmax(0, 1fr);
      gap: 4px;
    }
  }
  @media (max-width: 600px) {
    .hero {
      padding-top: 104px;
    }
    /* The lockup is drawn at a fixed height; scale it to the screen. */
    .lockup {
      transform-origin: left top;
      scale: 0.68;
      margin-bottom: -30px;
    }
    .orbit {
      width: 100%;
    }
    .node {
      font-size: 10px;
      padding: 4px 8px;
    }
    .stats div {
      padding: 22px 16px;
    }
    .stats strong {
      font-size: 34px;
    }
    pre {
      font-size: 12px;
      padding: 16px;
    }
    .layers article {
      padding: 22px;
    }
    .end {
      padding: 64px 20px;
      border-radius: 32px;
    }
  }
</style>
