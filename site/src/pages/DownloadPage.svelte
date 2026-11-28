<script lang="ts">
  import { onMount } from "svelte";
  import { fade, fly } from "svelte/transition";
  import AndroidDownloadButton from "../components/AndroidDownloadButton.svelte";
  import PlatformSwitch from "../components/PlatformSwitch.svelte";
  import { detectPlatform, initialPlatform, type Platform } from "../lib/platform";
  import DownloadButton from "../components/DownloadButton.svelte";
  import DownloadCount from "../components/DownloadCount.svelte";
  import { SIGNING_SHA256, STEPS as ANDROID_STEPS } from "../lib/android";
  import { latest } from "../lib/latest.svelte";
  import { megabytes, REPO_URL, RELEASES_URL, shortDate } from "../lib/release";
  import ApkInstallMock from "./ApkInstallMock.svelte";
  import { inview } from "./inview";
  import InstallerMock from "./InstallerMock.svelte";
  import Note from "./Note.svelte";
  import PageShell from "./PageShell.svelte";
  import PlatformsSoon from "./PlatformsSoon.svelte";
  import Related from "./Related.svelte";
  import Steps from "./Steps.svelte";

  // The page opens on the platform you're browsing from; the switch shows the other.
  let platform = $state<Platform>("windows");
  let detected = $state<Platform | null>(null);

  onMount(() => {
    if (!latest.loaded) latest.load();
    detected = detectPlatform();
    platform = initialPlatform();
  });

  const win = $derived(latest.current);
  const droid = $derived(latest.android);
  const apk = $derived(droid?.android?.arm64 ?? droid?.android?.universal);

  const WIN_STEPS = [
    { title: "Download the installer", text: "One small file, built by GitHub Actions from the public source. It installs for your own Windows account only, so it never asks for administrator rights." },
    { title: "Get past SmartScreen", text: "Velmora isn't code-signed yet, so Windows may warn you. Click More info, then Run anyway. The install guide shows exactly what to click." },
    { title: "Open it once", text: "The first launch fetches the video player and download tools, about 80 MB, and checks each file's SHA-256. This needs the internet." },
    { title: "Make your free account", text: "A one-time sign-in in your browser with Google, Discord or an emailed code. After that, Velmora opens offline too." },
    { title: "Search and press Play", text: "Updates take care of themselves: Velmora installs a new version the next time you open it." },
  ];

  const TRUST = [
    { icon: "code", title: "Built from public source", text: "The code is on GitHub under GPL-3.0, and GitHub Actions builds every installer and APK from it, so what you download is what the source says." },
    { icon: "verified", title: "Signed", text: "Windows updates are signed and checked before they install. Every APK carries the same signing key, so Android only accepts updates from us." },
    { icon: "fact_check", title: "Checked downloads", text: "The player and download tools Velmora fetches on first launch are verified with SHA-256 before they're used." },
  ];

  const META = {
    windows: { name: "Windows", icon: "desktop_windows", lead: "One small installer. No card, no subscription, and nothing to configure before your first search." },
    android: { name: "Android", icon: "android", lead: "The whole app for phones and tablets. No ads, the same account as desktop, and it updates itself." },
  } as const;
</script>

<PageShell eyebrow="Download" title="Velmora for" accent={META[platform].name} crumb="Download" lead={META[platform].lead}>
  {#snippet hero()}
    <PlatformSwitch bind:value={platform} {detected} />

    {#key platform}
      <div class="heroRow {platform}" in:fly={{ y: 14, duration: 420 }}>
        <div class="cta">
          {#if platform === "windows"}
            <DownloadButton />
          {:else}
            <AndroidDownloadButton />
            {#if droid?.android?.universal && droid.android.arm64}
              <p class="alt">Older 32-bit phone? <a href={droid.android.universal.url}>Get the universal APK</a> ({megabytes(droid.android.universal.size)}).</p>
            {/if}
          {/if}
          <DownloadCount />
          <div class="links">
            <m3e-button variant="text" size="small" href="/changelog/"><m3e-icon slot="icon" name="history"></m3e-icon>Release notes</m3e-button>
            <m3e-button variant="text" size="small" href={RELEASES_URL} target="_blank"><m3e-icon slot="icon" name="folder_open"></m3e-icon>All releases</m3e-button>
            {#if platform === "android"}
              <m3e-button variant="text" size="small" href="/android/"><m3e-icon slot="icon" name="android"></m3e-icon>About the app</m3e-button>
            {:else}
              <m3e-button variant="text" size="small" href={REPO_URL} target="_blank"><m3e-icon slot="icon" name="code"></m3e-icon>Source</m3e-button>
            {/if}
          </div>
          {#if detected && detected !== platform}
            <p class="hint">
              <m3e-icon name="info"></m3e-icon>
              {platform === "android" ? "You're on a computer: open this page on your phone to install, or send yourself the link." : "You're on a phone: this installer is for a Windows PC."}
            </p>
          {/if}
        </div>
        <div class="art">
          {#if platform === "windows"}<InstallerMock />{:else}<ApkInstallMock />{/if}
        </div>
      </div>
    {/key}
  {/snippet}

  {#key platform}
    <section class="pg-section" in:fade={{ duration: 300 }}>
      <div class="pg-wide">
        <div class="facts {platform}">
          {#if platform === "windows"}
            <div class="fact" use:inview>
              <m3e-icon name="download"></m3e-icon>
              <strong>{win ? `Version ${win.version}` : "Latest version"}</strong>
              <span>{win?.date ? `Released ${shortDate(win.date)}` : "Always the newest release"}</span>
            </div>
            <div class="fact" use:inview={0.06}>
              <m3e-icon name="storage"></m3e-icon>
              <strong>{win?.installer ? megabytes(win.installer.size) : "A small installer"}</strong>
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
          {:else}
            <div class="fact" use:inview>
              <m3e-icon name="download"></m3e-icon>
              <strong>{droid ? `Version ${droid.version}` : "Latest version"}</strong>
              <span>{droid?.date ? `Released ${shortDate(droid.date)}` : "Always the newest release"}</span>
            </div>
            <div class="fact" use:inview={0.06}>
              <m3e-icon name="storage"></m3e-icon>
              <strong>{apk ? megabytes(apk.size) : "One APK"}</strong>
              <span>Everything included, nothing fetched later</span>
            </div>
            <div class="fact" use:inview={0.12}>
              <m3e-icon name="phone_android"></m3e-icon>
              <strong>Android 8 and up</strong>
              <span>Phones and tablets, 64-bit and 32-bit</span>
            </div>
            <div class="fact" use:inview={0.18}>
              <m3e-icon name="block"></m3e-icon>
              <strong>No ads</strong>
              <span>Not one, on Public or Max</span>
            </div>
          {/if}
        </div>
      </div>
    </section>

    <section class="pg-section" in:fade={{ duration: 300 }}>
      <div class="pg-wide narrow">
        <span class="eyebrow" use:inview>In {platform === "windows" ? "five" : "four"} steps</span>
        <h2 class="pg-title" use:inview={0.05}>From download to <em>first Play</em></h2>
        <Steps steps={platform === "windows" ? WIN_STEPS : ANDROID_STEPS} />
        {#if platform === "windows"}
          <Note label="Windows warns about the installer" icon="shield" variant="secondary">
            That's SmartScreen reacting to an app it hasn't seen often, because certificates for code signing are expensive. The warning isn't a verdict on Velmora. The
            <a href="/docs/install/">install guide</a> shows what it looks like and where to click.
          </Note>
        {:else}
          <Note label="Android warns about apps from outside the Play Store" icon="shield" variant="secondary">
            That's Play Protect reacting to an app it hasn't seen, not a verdict on Velmora. Every APK is signed with the same key; its SHA-256
            fingerprint is <code>{SIGNING_SHA256}</code>.
          </Note>
        {/if}
      </div>
    </section>
  {/key}

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
      { href: "/android/", icon: "android", title: "Velmora for Android", text: "What's in the Android app, how to install it, and what the next updates bring." },
      { href: "/docs/account/", icon: "account_circle", title: "Your free account", text: "Sign in once, keep your list and progress on your PC and your phone." },
    ]}
  />
</PageShell>

<style>
  /* ── Hero ── */
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
  .alt {
    margin: -4px 0 0;
    font-size: 14px;
    color: var(--muted);
  }
  .alt a {
    color: #8ff0b8;
  }
  .links {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
    margin-left: -12px;
  }
  .hint {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    max-width: 440px;
    margin: 2px 0 0;
    font-size: 13.5px;
    color: var(--muted);
    --m3e-icon-size: 18px;
  }
  .art {
    display: grid;
    place-items: center;
    flex: 1 1 280px;
  }

  /* ── Facts ── */
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
  .android .fact m3e-icon {
    color: #3ddc84;
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
  code {
    font-size: 12px;
    word-break: break-all;
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
