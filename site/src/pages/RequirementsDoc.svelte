<script lang="ts">
  import { onMount } from "svelte";
  import { latest } from "../lib/latest.svelte";
  import { megabytes } from "../lib/release";
  import { inview } from "./inview";
  import Note from "./Note.svelte";
  import PageShell from "./PageShell.svelte";
  import Related from "./Related.svelte";
  import Toc from "./Toc.svelte";

  // What this browser can tell us about the PC. Nothing leaves the page.
  interface Check { label: string; value: string; state: "ok" | "no" | "info" }
  let checks = $state<Check[]>([]);
  let ready = $state(false);

  onMount(() => {
    if (!latest.loaded) latest.load();
    const nav = navigator as Navigator & { deviceMemory?: number; userAgentData?: { platform?: string } };
    const ua = nav.userAgent;
    const windows = /Windows NT 10\.0/.test(ua) || nav.userAgentData?.platform === "Windows";
    const old = /Windows NT [5-6]\./.test(ua);
    const list: Check[] = [];
    list.push(
      windows
        ? { label: "Operating system", value: "Windows 10 or 11", state: "ok" }
        : old
          ? { label: "Operating system", value: "An older Windows. Velmora needs 10 or 11.", state: "no" }
          : { label: "Operating system", value: "Not Windows. Velmora is Windows only for now.", state: "no" },
    );
    list.push({ label: "Processor threads", value: nav.hardwareConcurrency ? String(nav.hardwareConcurrency) : "Your browser doesn't say", state: "info" });
    if (nav.deviceMemory) list.push({ label: "Memory (rounded down by your browser)", value: `${nav.deviceMemory} GB or more`, state: "info" });
    const w = Math.round(screen.width * devicePixelRatio);
    const h = Math.round(screen.height * devicePixelRatio);
    list.push({ label: "Screen", value: `${w} × ${h}${w >= 3800 ? ", a 4K screen" : ""}`, state: "info" });
    checks = list;
    ready = true;
  });

  const REQS = [
    { icon: "desktop_windows", title: "Windows 10 or 11, 64-bit", text: "Windows only for now. Android is in production development and drops soon; iPhone, iPad, Mac and Linux come later." },
    { icon: "person", title: "No administrator rights", text: "It installs for your own Windows user account only." },
    { icon: "language", title: "Internet", text: "Needed for your first sign-in, the first launch (about 80 MB of tools), and for searching and playing online sources. After the first sign-in, Velmora opens offline, and downloaded episodes play offline." },
  ];
</script>

<PageShell
  eyebrow="System requirements"
  title="Will it run on"
  accent="your PC?"
  crumb="System requirements"
  lead="Almost any Windows 10 or 11 PC. Here's exactly what Velmora needs, and what makes 4K smooth."
  spots={[
    { x: "91%", y: "190px", size: 118, color: "#2dd4bf", spin: 24, opacity: 0.5 },
    { x: "5%", y: "380px", size: 58, color: "#ffb74d", spin: -14, offset: 3, opacity: 0.65 },
  ]}
>
  <div class="pg-wide pg-split body">
    <Toc target="doc" />
    <article id="doc" class="pg-prose">
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
        <p class="small">Whether your graphics card can decode a codec in hardware isn't something a web page can see. Velmora checks that itself when it plays something.</p>
      </section>

      <section class="pg-section" id="not-yet">
        <h2 class="pg-title" use:inview>Not <em>supported yet</em></h2>
        <p>Android is in production development and drops soon, as a full build for Android 8 and up with Material You or Liquid Glass. macOS, Linux, iPhone and iPad come later: iPhone, iPad and Mac as native SwiftUI apps with Liquid Glass, and Linux builds. Smart TVs and consoles aren't planned yet. There are no dates, and this page will say when that changes.</p>
      </section>
    </article>
  </div>

  <Related
    items={[
      { href: "/download/", icon: "download", title: "Download", text: "Free for Windows 10 and 11." },
      { href: "/docs/install/", icon: "menu_book", title: "Install guide", text: "Every step, including the SmartScreen warning." },
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
