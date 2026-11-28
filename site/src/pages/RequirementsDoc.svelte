<script lang="ts">
  import { onMount } from "svelte";
  import { fly } from "svelte/transition";
  import PlatformSwitch from "../components/PlatformSwitch.svelte";
  import { latest } from "../lib/latest.svelte";
  import { detectPlatform, initialPlatform, type Platform } from "../lib/platform";
  import { megabytes } from "../lib/release";
  import { inview } from "./inview";
  import Note from "./Note.svelte";
  import PageShell from "./PageShell.svelte";
  import Related from "./Related.svelte";
  import Toc from "./Toc.svelte";

  let platform = $state<Platform>("windows");
  let detected = $state<Platform | null>(null);

  // What this browser can tell us about the device. Nothing leaves the page.
  interface Check { label: string; value: string; state: "ok" | "no" | "info" }
  let checks = $state<Check[]>([]);
  let ready = $state(false);

  onMount(() => {
    if (!latest.loaded) latest.load();
    detected = detectPlatform();
    platform = initialPlatform();
    const nav = navigator as Navigator & { deviceMemory?: number; userAgentData?: { platform?: string } };
    const ua = nav.userAgent;
    if (detected === "android") {
      const ver = Number(/Android (\d+)/i.exec(ua)?.[1] ?? 0);
      const list: Check[] = [
        ver
          ? ver >= 8
            ? { label: "Android", value: `Android ${ver}`, state: "ok" }
            : { label: "Android", value: `Android ${ver}. Velmora needs 8 or newer.`, state: "no" }
          : { label: "Android", value: "Your browser doesn't say which version", state: "info" },
        { label: "Processor", value: /arm64|aarch64/i.test(ua) ? "64-bit: get the arm64 APK" : "Get the arm64 APK; if it won't install, the universal one", state: "info" },
      ];
      if (nav.deviceMemory) list.push({ label: "Memory (rounded down by your browser)", value: `${nav.deviceMemory} GB or more`, state: "info" });
      list.push({ label: "Screen", value: `${Math.round(screen.width * devicePixelRatio)} × ${Math.round(screen.height * devicePixelRatio)}`, state: "info" });
      checks = list;
      ready = true;
      return;
    }
    const windows = /Windows NT 10\.0/.test(ua) || nav.userAgentData?.platform === "Windows";
    const old = /Windows NT [5-6]\./.test(ua);
    const list: Check[] = [];
    list.push(
      windows
        ? { label: "Operating system", value: "Windows 10 or 11", state: "ok" }
        : old
          ? { label: "Operating system", value: "An older Windows. Velmora needs 10 or 11.", state: "no" }
          : { label: "Operating system", value: "Not Windows. On a phone or tablet, get Velmora for Android.", state: "no" },
    );
    list.push({ label: "Processor threads", value: nav.hardwareConcurrency ? String(nav.hardwareConcurrency) : "Your browser doesn't say", state: "info" });
    if (nav.deviceMemory) list.push({ label: "Memory (rounded down by your browser)", value: `${nav.deviceMemory} GB or more`, state: "info" });
    const w = Math.round(screen.width * devicePixelRatio);
    const h = Math.round(screen.height * devicePixelRatio);
    list.push({ label: "Screen", value: `${w} × ${h}${w >= 3800 ? ", a 4K screen" : ""}`, state: "info" });
    checks = list;
    ready = true;
  });

  const ANDROID_REQS = [
    { icon: "android", title: "Android 8 or newer", text: "Phones and tablets. Both 64-bit phones (nearly all of them) and older 32-bit ones are supported, each with its own APK." },
    { icon: "storage", title: "About 300 MB free", text: "The app itself is about 130 MB. Downloads and the Music cache come on top, as much as you choose to keep." },
    { icon: "language", title: "Internet", text: "Needed for your first sign-in and for searching and playing online sources. After the first sign-in, Velmora opens offline, and downloads play offline." },
  ];

  const REQS = [
    { icon: "desktop_windows", title: "Windows 10 or 11, 64-bit", text: "On a phone or tablet, Velmora for Android needs Android 8 or newer. iPhone, iPad, Mac and Linux come later." },
    { icon: "person", title: "No administrator rights", text: "It installs for your own Windows user account only." },
    { icon: "language", title: "Internet", text: "Needed for your first sign-in, the first launch (about 80 MB of tools), and for searching and playing online sources. After the first sign-in, Velmora opens offline, and downloaded episodes play offline." },
  ];
</script>

<PageShell
  eyebrow="System requirements"
  title="Will it run on"
  accent={platform === "android" ? "your phone?" : "your PC?"}
  crumb="System requirements"
  lead={platform === "android"
    ? "Almost any phone or tablet from the last several years. Here's exactly what Velmora for Android needs."
    : "Almost any Windows 10 or 11 PC. Here's exactly what Velmora needs, and what makes 4K smooth."}
  spots={[
    { x: "91%", y: "190px", size: 118, color: "#2dd4bf", spin: 24, opacity: 0.5 },
    { x: "5%", y: "380px", size: 58, color: "#ffb74d", spin: -14, offset: 3, opacity: 0.65 },
  ]}
>
  {#snippet hero()}
    <PlatformSwitch bind:value={platform} {detected} />
  {/snippet}

  {#key platform}
  <div class="pg-wide pg-split body" in:fly={{ y: 12, duration: 380 }}>
    <Toc target="doc" />
    <article id="doc" class="pg-prose">
      {#if platform === "android"}
      <section class="pg-section first" id="needs">
        <h2 class="pg-title" use:inview>What it <em>needs</em></h2>
        <div class="cards">
          {#each ANDROID_REQS as r, i (r.title)}
            <div class="pg-card green" use:inview={i * 0.06}>
              <m3e-icon name={r.icon}></m3e-icon>
              <h3>{r.title}</h3>
              <p>{r.text}</p>
            </div>
          {/each}
        </div>
      </section>

      <section class="pg-section" id="space">
        <h2 class="pg-title" use:inview>Storage <em>space</em></h2>
        <table use:inview>
          <tbody>
            <tr><th scope="row">The APK</th><td>{latest.android?.android?.arm64 ? megabytes(latest.android?.android?.arm64.size) : "About 130 MB"}, everything included. Nothing is fetched on first launch.</td></tr>
            <tr><th scope="row">Films you download</th><td>In <code>Movies/Velmora</code>, encrypted so only Velmora plays them. Settings › Downloads picks the quality, from 480p to the best there is.</td></tr>
            <tr><th scope="row">Music</th><td>Songs you download, plus a cache of what you play, both cleared from Settings › Storage.</td></tr>
          </tbody>
        </table>
      </section>

      <section class="pg-section" id="video">
        <h2 class="pg-title" use:inview>Video and <em>sound</em></h2>
        <p>Velmora plays video with Android's own player engine, ExoPlayer, so the phone's video chip does the decoding. H.264 plays everywhere; HEVC and AV1 depend on the chip, and most phones from the last few years handle at least HEVC. If a file can't play on yours, Velmora moves to the next source by itself.</p>
        <p>Cinema sound (AC3, E-AC3 and DTS) usually needs special hardware. Velmora brings its own decoder for it, so films with that sound play with sound on every phone.</p>
        <Note label="Mobile data" icon="signal_cellular_alt" variant="info">
          On mobile data, Velmora picks 720p or lower and asks before streaming a torrent. Downloads wait for Wi-Fi. All three are settings.
        </Note>
      </section>

      <section class="pg-section" id="check">
        <h2 class="pg-title" use:inview>What your <em>browser can tell us</em></h2>
        {#if detected === "android"}
          <p>A web page can see only a little about your phone. This is that little. It's worked out in your browser and goes nowhere.</p>
          <ul class="checks" aria-live="polite">
            {#if !ready}<li class="wait">Checking…</li>{/if}
            {#each checks as c, i (c.label)}
              <li class={c.state} style="--i: {i}">
                <m3e-icon name={c.state === "ok" ? "check_circle" : c.state === "no" ? "error" : "info"} filled></m3e-icon>
                <span class="lbl">{c.label}</span>
                <strong>{c.value}</strong>
              </li>
            {/each}
          </ul>
        {:else}
          <p>Open this page on your phone and it checks the phone's Android version for you.</p>
        {/if}
      </section>
      {:else}
      <section class="pg-section first" id="needs">
        <h2 class="pg-title" use:inview>What it <em>needs</em></h2>
        <div class="cards">
          {#each REQS as r, i (r.title)}
            <div class="pg-card" use:inview={i * 0.06}>
              <m3e-icon name={r.icon}></m3e-icon>
              <h3>{r.title}</h3>
              <p>{r.text}</p>
            </div>
          {/each}
        </div>
      </section>

      <section class="pg-section" id="space">
        <h2 class="pg-title" use:inview>Disk <em>space</em></h2>
        <table use:inview>
          <tbody>
            <tr><th scope="row">The installer</th><td>{latest.current?.installer ? megabytes(latest.current.installer.size) : "A small download"}</td></tr>
            <tr><th scope="row">First launch</th><td>About 80 MB: the video player (mpv) and the download tools, each checked with SHA-256</td></tr>
            <tr><th scope="row">Episodes you download</th><td>Whatever you save, in a folder you choose</td></tr>
            <tr><th scope="row">Theatre screenings</th><td>While a screening is on, Velmora buffers the film ahead of you on disk, up to 2 GB, and deletes it afterwards</td></tr>
          </tbody>
        </table>
      </section>

      <section class="pg-section" id="graphics">
        <h2 class="pg-title" use:inview>Graphics and <em>4K</em></h2>
        <p>Velmora plays video with mpv. It asks your graphics card to do the decoding whenever that's safe, which is what keeps 4K smooth without using much of the processor. You can pick the video renderer in Settings: the best one, Direct3D 11, or OpenGL for older cards.</p>
        <p>What your card can decode in hardware decides how easy 4K is. HEVC and AV1 are the heavy ones: newer graphics cards decode them in hardware, older ones fall back to the processor, which still works but works harder. If mpv can't run on your PC at all, a built-in player takes over so Play still works.</p>
        <Note label="There is no minimum we can honestly promise" icon="info" variant="info">
          Velmora doesn't set a minimum graphics card or processor, and we haven't tested every PC. If 4K stutters on yours, try a different renderer in Settings, or a lower-resolution link. If it still doesn't play, <a href="/contact/">tell us your PC</a>.
        </Note>
      </section>

      <section class="pg-section" id="check">
        <h2 class="pg-title" use:inview>What your <em>browser can tell us</em></h2>
        <p>A web page can see only a little about your PC. This is that little. It's worked out in your browser and goes nowhere.</p>
        <ul class="checks" aria-live="polite">
          {#if !ready}
            <li class="wait">Checking…</li>
          {/if}
          {#each checks as c, i (c.label)}
            <li class={c.state} style="--i: {i}">
              <m3e-icon name={c.state === "ok" ? "check_circle" : c.state === "no" ? "error" : "info"} filled></m3e-icon>
              <span class="lbl">{c.label}</span>
              <strong>{c.value}</strong>
            </li>
          {/each}
        </ul>
        {#if detected === "android"}
          <p class="small">You're on a phone, so this is about the phone. Switch to Android above for what it needs.</p>
        {/if}
        <p class="small">Whether your graphics card can decode a codec in hardware isn't something a web page can see. Velmora checks that itself when it plays something.</p>
      </section>

      {/if}

      <section class="pg-section" id="not-yet">
        <h2 class="pg-title" use:inview>Not <em>supported yet</em></h2>
        <p><a href="/android/">Velmora for Android</a> is out, for phones and tablets on Android 8 and up. macOS, Linux, iPhone and iPad come later: iPhone, iPad and Mac as native SwiftUI apps with Liquid Glass, and Linux builds. Smart TVs and consoles aren't planned yet. There are no dates, and this page will say when that changes.</p>
      </section>
    </article>
  </div>
  {/key}

  <Related
    items={[
      { href: "/download/", icon: "download", title: "Download", text: "Free for Windows 10 and 11, and Android 8 and up." },
      { href: "/docs/install/", icon: "menu_book", title: "Install guide", text: "Every step, on Windows and Android." },
      { href: "/faq/", icon: "help", title: "FAQ", text: "Short answers to common questions." },
    ]}
  />
</PageShell>

<style>
  .body {
    margin-top: 12px;
  }
  section {
    scroll-margin-top: 96px;
  }
  section.first {
    padding-top: 44px;
  }
  h2 {
    margin-bottom: 14px;
  }
  p {
    margin: 0 0 14px;
  }
  .small {
    font-size: 14px;
  }
  .cards {
    display: grid;
    gap: 14px;
    margin-top: 24px;
  }
  .pg-card m3e-icon {
    color: var(--blue);
    --m3e-icon-size: 26px;
  }
  .pg-card.green m3e-icon {
    color: #3ddc84;
  }
  code {
    padding: 2px 8px;
    border-radius: 8px;
    background: rgb(255 255 255 / 8%);
    color: var(--cream);
    font-size: 0.92em;
  }
  table {
    width: 100%;
    margin-top: 18px;
    border-collapse: separate;
    border-spacing: 0;
    border: 1px solid var(--line);
    border-radius: 22px;
    overflow: hidden;
  }
  th,
  td {
    padding: 15px 18px;
    text-align: left;
    vertical-align: top;
    border-bottom: 1px solid var(--line);
    font-size: 15.5px;
  }
  th {
    width: 32%;
    color: var(--ink);
    font-weight: 600;
    background: var(--glass);
  }
  td {
    color: var(--muted);
  }
  tr:last-child th,
  tr:last-child td {
    border-bottom: 0;
  }
  .checks {
    list-style: none;
    margin: 20px 0 14px;
    padding: 0;
    display: grid;
    gap: 10px;
  }
  .checks li {
    display: grid;
    grid-template-columns: 28px minmax(0, 1fr) auto;
    align-items: center;
    gap: 12px;
    padding: 14px 18px;
    border-radius: 18px;
    border: 1px solid var(--line);
    background: var(--glass);
    animation: pop 0.55s cubic-bezier(0.34, 1.4, 0.5, 1) both;
    animation-delay: calc(var(--i) * 0.09s);
  }
  .checks li.ok m3e-icon {
    color: #4fd18b;
  }
  .checks li.no m3e-icon {
    color: #ff8a80;
  }
  .checks li.info m3e-icon {
    color: var(--blue);
  }
  .wait {
    color: var(--muted);
    padding: 14px 18px;
  }
  .lbl {
    color: var(--muted);
    font-size: 14px;
  }
  .checks strong {
    font-size: 15px;
    text-align: right;
    color: var(--ink);
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: translateY(10px) scale(0.97);
    }
  }
  @media (max-width: 600px) {
    .checks li {
      grid-template-columns: 24px minmax(0, 1fr);
    }
    .checks strong {
      grid-column: 2;
      text-align: left;
    }
    th {
      width: 40%;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .checks li {
      animation: none;
    }
  }
</style>
