<script lang="ts">
  import { onMount } from "svelte";
  import { fly } from "svelte/transition";
  import AndroidDownloadButton from "../components/AndroidDownloadButton.svelte";
  import DownloadButton from "../components/DownloadButton.svelte";
  import PlatformSwitch from "../components/PlatformSwitch.svelte";
  import { SIGNING_SHA256 } from "../lib/android";
  import { latest } from "../lib/latest.svelte";
  import { detectPlatform, initialPlatform, type Platform } from "../lib/platform";
  import { megabytes } from "../lib/release";
  import ApkInstallMock from "./ApkInstallMock.svelte";
  import { inview } from "./inview";
  import Note from "./Note.svelte";
  import PageShell from "./PageShell.svelte";
  import Related from "./Related.svelte";
  import SmartScreenMock from "./SmartScreenMock.svelte";
  import Toc from "./Toc.svelte";

  let platform = $state<Platform>("windows");
  let detected = $state<Platform | null>(null);

  onMount(() => {
    if (!latest.loaded) latest.load();
    detected = detectPlatform();
    platform = initialPlatform();
  });

  const universal = $derived(latest.android?.android?.universal);

  const LEAD = {
    windows: "Download, click past one Windows warning, open it once. Here is every step, including what each screen looks like.",
    android: "Download the APK, allow the install once, sign in. Here is every step, including the screens Android shows you.",
  };
</script>

<PageShell
  eyebrow="Install guide"
  title="Install Velmora"
  accent="in a minute."
  crumb="Install guide"
  lead={LEAD[platform]}
  spots={[
    { x: "92%", y: "180px", size: 120, color: platform === "android" ? "#3ddc84" : "#3271fe", spin: 22, opacity: 0.55 },
    { x: "4%", y: "400px", size: 52, color: "#2dd4bf", spin: -14, offset: 6, opacity: 0.65 },
  ]}
>
  {#snippet hero()}
    <PlatformSwitch bind:value={platform} {detected} />
  {/snippet}

  {#key platform}
    <div class="pg-wide pg-split body" in:fly={{ y: 12, duration: 380 }}>
      <Toc target="doc" />
      <article id="doc" class="pg-prose">
        {#if platform === "windows"}
          <section class="pg-section first" id="before">
            <h2 class="pg-title" use:inview>Before you <em>start</em></h2>
            <p>You need a PC running Windows 10 or 11 (64-bit) and an internet connection for the first launch. That's all. You don't need administrator rights, a credit card or an account to download. The <a href="/docs/system-requirements/">requirements page</a> has the details.</p>
          </section>

          <section class="pg-section" id="download">
            <h2 class="pg-title" use:inview>1. <em>Download</em></h2>
            <p>Press the button below, or pick the file called <code>x64-setup.exe</code> on the <a href="/download/">download page</a>. It's one small installer, built by GitHub Actions from the public source.</p>
            <div class="cta" use:inview><DownloadButton /></div>
          </section>

          <section class="pg-section" id="smartscreen">
            <h2 class="pg-title" use:inview>2. Get past <em>SmartScreen</em></h2>
            <p>When you run the installer, Windows may say <strong>"Windows protected your PC"</strong>. Velmora isn't code-signed yet, because signing certificates are expensive, and SmartScreen warns about any unsigned app it hasn't seen often.</p>
            <p>Click <strong>More info</strong>, then <strong>Run anyway</strong>. The picture shows the whole thing.</p>
            <SmartScreenMock />
            <Note label="Is it safe to click Run anyway?" icon="shield" variant="secondary">
              The source is public and GitHub Actions builds every installer from it, so you can check what's inside. If you'd rather not run an unsigned installer, you can <a href="https://github.com/ashfaqizop/Velmora" target="_blank" rel="noopener">build it yourself</a>.
            </Note>
          </section>

          <section class="pg-section" id="setup">
            <h2 class="pg-title" use:inview>3. <em>Setup</em> runs</h2>
            <p>The installer puts Velmora in your own Windows user account, so it doesn't ask for administrator rights. It adds a Start menu entry and finishes in a few seconds.</p>
          </section>

          <section class="pg-section" id="first-launch">
            <h2 class="pg-title" use:inview>4. The <em>first launch</em></h2>
            <p>The first time you open Velmora, it fetches the video player and the download tools, about 80 MB, and checks each file's SHA-256 before it uses it. This is the one step that needs the internet. After it, the player is on your PC.</p>
          </section>

          <section class="pg-section" id="sign-in">
            <h2 class="pg-title" use:inview>5. Make your <em>account</em></h2>
            <p>You sign in once with a free account: Google, Discord or an emailed code, in your browser. After that first sign-in, Velmora opens offline too. The same account works in Velmora for Android. The <a href="/docs/account/">account guide</a> walks through it.</p>
          </section>

          <section class="pg-section" id="updates">
            <h2 class="pg-title" use:inview>Updates <em>take care of themselves</em></h2>
            <p>Velmora checks for a new version when it starts and installs it itself. Every update is signed, and the app checks the signature before installing. You'll find what changed on the <a href="/changelog/">what's new page</a>.</p>
          </section>

          <section class="pg-section" id="uninstall">
            <h2 class="pg-title" use:inview>Uninstalling</h2>
            <p>Open Windows Settings, go to Apps, find Velmora in the list and choose Uninstall. Your account and its synced list and progress live in your Velmora account, not on the PC. To remove those too, delete the account first: in Velmora, open Settings, then Velmora account.</p>
          </section>

          <section class="pg-section" id="stuck">
            <h2 class="pg-title" use:inview>If something <em>goes wrong</em></h2>
            <p>Check the <a href="/faq/">FAQ</a> first. If it still doesn't work, <a href="/contact/">tell us</a> what you tried and what happened. A screenshot helps a lot.</p>
          </section>
        {:else}
          <section class="pg-section first" id="before">
            <h2 class="pg-title" use:inview>Before you <em>start</em></h2>
            <p>You need a phone or tablet on Android 8 or newer, about 300 MB free, and the internet for your first sign-in. Velmora isn't on the Play Store, so you install it from this website, which takes one extra permission. The <a href="/docs/system-requirements/?for=android">requirements page</a> has the details.</p>
            {#if detected === "windows"}
              <Note label="You're on a computer" icon="smartphone" variant="info">
                Open <strong>getvelmora.dpdns.org/android</strong> on your phone and download it there. That's simpler than copying the file across.
              </Note>
            {/if}
          </section>

          <section class="pg-section" id="download">
            <h2 class="pg-title" use:inview>1. <em>Download</em> the APK</h2>
            <p>Press the button on your phone. It gets the 64-bit file, <code>android-arm64.apk</code>, which suits nearly every phone and tablet from the last several years.</p>
            <div class="cta" use:inview><AndroidDownloadButton /></div>
            <p class="small">
              An older or very cheap phone may be 32-bit. If the 64-bit file says "App not installed",
              {#if universal}get the <a href={universal.url}>universal APK</a> ({megabytes(universal.size)}) instead{:else}get the file ending in <code>android-universal.apk</code> from the releases page instead{/if}: it works on every Android phone.
            </p>
          </section>

          <section class="pg-section" id="allow">
            <h2 class="pg-title" use:inview>2. <em>Allow</em> the install</h2>
            <p>Open the downloaded file, from the notification or your Downloads. The first time, Android says your browser <strong>isn't allowed to install unknown apps</strong>. Tap <strong>Settings</strong>, switch on <strong>Allow from this source</strong>, and go back. Android only asks once per app you install from.</p>
            <div class="mock" use:inview><ApkInstallMock /></div>
            <p>Then tap <strong>Install</strong>. It takes a few seconds.</p>
          </section>

          <section class="pg-section" id="play-protect">
            <h2 class="pg-title" use:inview>3. If <em>Play Protect</em> asks</h2>
            <p>Google Play Protect may say it <strong>hasn't seen this app before</strong>, or suggest a scan. That's what it does for any app that isn't on the Play Store. Tap <strong>More details</strong>, then <strong>Install anyway</strong>, or let it scan: the scan comes back clean.</p>
            <Note label="How to know it's really Velmora" icon="verified" variant="secondary">
              Every Velmora APK is built by GitHub Actions from the <a href="https://github.com/ashfaqizop/Velmora" target="_blank" rel="noopener">public source</a> and signed with the same key, whose SHA-256 fingerprint is <code>{SIGNING_SHA256}</code>. Android refuses an update signed with any other key.
            </Note>
          </section>

          <section class="pg-section" id="sign-in">
            <h2 class="pg-title" use:inview>4. <em>Sign in</em></h2>
            <p>Open Velmora and sign in with your free account: Google, Discord or an emailed code. It opens your browser for a moment, then comes back to the app. Already use Velmora on Windows? Use the same account and your list and progress are there. After this one sign-in, Velmora opens offline too.</p>
          </section>

          <section class="pg-section" id="first-look">
            <h2 class="pg-title" use:inview>5. Make it <em>yours</em></h2>
            <p>Pick a look, Material You or Liquid Glass, and you're in. Music asks for a YouTube Music sign-in the first time you switch to it, like on desktop. Coming from Echo Music? Make a backup in Echo, then in Velmora open Settings › Transfer from Echo Music.</p>
          </section>

          <section class="pg-section" id="updates">
            <h2 class="pg-title" use:inview>Updates <em>come to you</em></h2>
            <p>When a new version is out, Velmora tells you and downloads it in the app. Tap Install when it's ready; your downloads, list and settings stay. You can check any time in Settings › System update.</p>
          </section>

          <section class="pg-section" id="uninstall">
            <h2 class="pg-title" use:inview>Uninstalling</h2>
            <p>Press and hold the Velmora icon, tap <strong>App info</strong>, then <strong>Uninstall</strong>. Downloaded films live in <code>Movies/Velmora</code> and stay until you delete that folder; they're encrypted, so only Velmora plays them. Your account, list and progress are online; to remove those too, delete the account first in the Windows app (Settings › Velmora account).</p>
          </section>

          <section class="pg-section" id="stuck">
            <h2 class="pg-title" use:inview>If something <em>goes wrong</em></h2>
            <ul>
              <li><strong>"App not installed"</strong>: try the universal APK, and check you have about 300 MB free.</li>
              <li><strong>"There was a problem parsing the package"</strong>: the download was cut short. Download it again.</li>
              <li><strong>Sign-in doesn't come back to the app</strong>: in the browser, tap Open Velmora, or type the short code it shows into the app.</li>
            </ul>
            <p>Still stuck? <a href="/contact/">Tell us</a> your phone, its Android version and what you saw. A screenshot helps a lot.</p>
          </section>
        {/if}
      </article>
    </div>
  {/key}

  <Related
    items={[
      { href: "/docs/system-requirements/", icon: "memory", title: "System requirements", text: "What your PC or phone needs, and a quick check of the one you're on." },
      { href: "/docs/account/", icon: "account_circle", title: "Your free account", text: "Sign in once and keep your list and progress on your PC and your phone." },
      { href: "/faq/", icon: "help", title: "FAQ", text: "Short answers to the questions people ask first." },
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
    color: var(--muted);
  }
  ul {
    margin: 0 0 14px;
    padding-left: 20px;
    line-height: 1.7;
  }
  code {
    padding: 2px 8px;
    border-radius: 8px;
    background: rgb(255 255 255 / 8%);
    color: var(--cream);
    font-size: 0.92em;
    word-break: break-all;
  }
  .cta {
    margin: 20px 0;
  }
  .mock {
    display: grid;
    place-items: center;
    margin: 26px 0;
  }
</style>
