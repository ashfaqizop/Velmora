<script lang="ts">
  import { onMount } from "svelte";
  import DownloadButton from "../components/DownloadButton.svelte";
  import DownloadCount from "../components/DownloadCount.svelte";
  import { latest } from "../lib/latest.svelte";
  import { megabytes, REPO_URL, RELEASES_URL, shortDate } from "../lib/release";
  import { inview } from "./inview";
  import InstallerMock from "./InstallerMock.svelte";
  import Note from "./Note.svelte";
  import PageShell from "./PageShell.svelte";
  import PlatformsSoon from "./PlatformsSoon.svelte";
  import Related from "./Related.svelte";
  import Steps from "./Steps.svelte";

  onMount(() => {
    if (!latest.loaded) latest.load();
  });

  const r = $derived(latest.current);

  const STEPS = [
    { title: "Download the installer", text: "One small file, built by GitHub Actions from the public source. It installs for your own Windows account only, so it never asks for administrator rights." },
    { title: "Get past SmartScreen", text: "Velmora isn't code-signed yet, so Windows may warn you. Click More info, then Run anyway. The install guide shows exactly what to click." },
    { title: "Open it once", text: "The first launch fetches the video player and download tools, about 80 MB, and checks each file's SHA-256. This needs the internet." },
    { title: "Make your free account", text: "A one-time sign-in in your browser with Google, Discord or an emailed code. After that, Velmora opens offline too." },
    { title: "Search and press Play", text: "Updates take care of themselves: Velmora installs a new version the next time you open it." },
  ];

  const TRUST = [
    { icon: "code", title: "Built from public source", text: "The code is on GitHub under GPL-3.0, and GitHub Actions builds every installer from it, so what you download is what the source says." },
    { icon: "verified", title: "Signed updates", text: "Every update is signed, and Velmora checks the signature before it installs anything." },
    { icon: "fact_check", title: "Checked downloads", text: "The player and download tools Velmora fetches on first launch are verified with SHA-256 before they're used." },
  ];
</script>

<PageShell eyebrow="Download" title="Velmora for" accent="Windows" crumb="Download" lead="One small installer. No card, no subscription, and nothing to configure before your first search.">
  {#snippet hero()}
    <div class="heroRow">
      <div class="cta">
        <DownloadButton />
        <DownloadCount />
        <div class="links">
          <m3e-button variant="text" size="small" href="/changelog/"><m3e-icon slot="icon" name="history"></m3e-icon>Release notes</m3e-button>
          <m3e-button variant="text" size="small" href={RELEASES_URL} target="_blank"><m3e-icon slot="icon" name="folder_open"></m3e-icon>All releases</m3e-button>
          <m3e-button variant="text" size="small" href={REPO_URL} target="_blank"><m3e-icon slot="icon" name="code"></m3e-icon>Source</m3e-button>
        </div>
      </div>
      <InstallerMock />
    </div>
  {/snippet}

  <section class="pg-section">
    <div class="pg-wide">
      <div class="facts">
        <div class="fact" use:inview>
          <m3e-icon name="download"></m3e-icon>
          <strong>{r ? `Version ${r.version}` : "Latest version"}</strong>
          <span>{r?.date ? `Released ${shortDate(r.date)}` : "Always the newest release"}</span>
        </div>
        <div class="fact" use:inview={0.06}>
          <m3e-icon name="storage"></m3e-icon>
          <strong>{r?.installer ? megabytes(r.installer.size) : "A small installer"}</strong>
          <span>Plus about 80 MB fetched on first launch</span>
        </div>
        <div class="fact" use:inview={0.12}>
          <m3e-icon name="desktop_windows"></m3e-icon>
          <strong>Windows 10 and 11</strong>
          <span>64-bit. <a href="/docs/system-requirements/">Full requirements</a></span>
        </div>
        <div class="fact" use:inview={0.18}>
          <m3e-icon name="person"></m3e-icon>
          <strong>No admin needed</strong>
          <span>Installs for your user account only</span>
        </div>
      </div>
    </div>
  </section>

  <section class="pg-section">
    <div class="pg-wide narrow">
      <span class="eyebrow" use:inview>In five steps</span>
      <h2 class="pg-title" use:inview={0.05}>From download to <em>first Play</em></h2>
      <Steps steps={STEPS} />
      <Note label="Windows warns about the installer" icon="shield" variant="secondary">
        That's SmartScreen reacting to an app it hasn't seen often, because certificates for code signing are expensive. The warning isn't a verdict on Velmora. The
        <a href="/docs/install/">install guide</a> shows what it looks like and where to click.
      </Note>
    </div>
  </section>

  <section class="pg-section">
    <div class="pg-wide">
      <span class="eyebrow" use:inview>Why you can trust it</span>
      <h2 class="pg-title" use:inview={0.05}>Open, signed <em>and checked</em></h2>
      <div class="pg-grid">
        {#each TRUST as t, i (t.title)}
          <div class="pg-card" use:inview={i * 0.07}>
            <m3e-icon name={t.icon}></m3e-icon>
            <h3>{t.title}</h3>
            <p>{t.text}</p>
          </div>
        {/each}
      </div>
    </div>
  </section>

  <PlatformsSoon />

  <Related
    items={[
      { href: "/docs/install/", icon: "menu_book", title: "Install guide", text: "Every step with pictures, including the SmartScreen warning and updates." },
      { href: "/docs/system-requirements/", icon: "memory", title: "System requirements", text: "What your PC needs, and a quick check of what yours has." },
      { href: "/docs/account/", icon: "account_circle", title: "Your free account", text: "Sign in once, keep your list and progress on every PC." },
    ]}
  />
</PageShell>

<style>
  .heroRow {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 48px;
    margin-top: 32px;
    flex-wrap: wrap;
  }
  .cta {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 14px;
  }
  .links {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
    margin-left: -12px;
  }
  .facts {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(230px, 1fr));
    gap: 14px;
  }
  .fact {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 22px 24px;
    border-radius: 26px;
    border: 1px solid var(--line);
    background: var(--glass);
  }
  .fact m3e-icon {
    color: var(--blue);
    --m3e-icon-size: 26px;
    margin-bottom: 8px;
  }
  .fact strong {
    font-size: 19px;
  }
  .fact span {
    color: var(--muted);
    font-size: 14px;
  }
  .fact a {
    color: #c9c1ff;
  }
  .narrow {
    width: min(820px, 100% - 48px);
    margin-inline: auto;
  }
  .pg-card m3e-icon {
    color: var(--blue);
    --m3e-icon-size: 28px;
  }
  @media (max-width: 600px) {
    .narrow {
      width: calc(100% - 32px);
    }
    .heroRow {
      gap: 28px;
    }
  }
</style>
