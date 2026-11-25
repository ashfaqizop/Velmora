<script lang="ts">
  import { onMount } from "svelte";
  import DownloadButton from "../components/DownloadButton.svelte";
  import { latest } from "../lib/latest.svelte";
  import { inview } from "./inview";
  import Note from "./Note.svelte";
  import PageShell from "./PageShell.svelte";
  import Related from "./Related.svelte";
  import SmartScreenMock from "./SmartScreenMock.svelte";
  import Toc from "./Toc.svelte";

  onMount(() => {
    if (!latest.loaded) latest.load();
  });
</script>

<PageShell
  eyebrow="Install guide"
  title="Install Velmora"
  accent="in a minute."
  crumb="Install guide"
  lead="Download, click past one Windows warning, open it once. Here is every step, including what each screen looks like."
  spots={[
    { x: "92%", y: "180px", size: 120, color: "#3271fe", spin: 22, opacity: 0.55 },
    { x: "4%", y: "400px", size: 52, color: "#2dd4bf", spin: -14, offset: 6, opacity: 0.65 },
  ]}
>
  <div class="pg-wide pg-split body">
    <Toc target="doc" />
    <article id="doc" class="pg-prose">
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
        <p>From Velmora 0.6.0 you sign in once with a free account: Google, Discord or an emailed code, in your browser. After that first sign-in, Velmora opens offline too. The <a href="/docs/account/">account guide</a> walks through it.</p>
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
    </article>
  </div>

  <Related
    items={[
      { href: "/docs/system-requirements/", icon: "memory", title: "System requirements", text: "What your PC needs, and a quick check of what yours has." },
      { href: "/docs/account/", icon: "account_circle", title: "Your free account", text: "Sign in once and keep your list and progress on every PC." },
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
  code {
    padding: 2px 8px;
    border-radius: 8px;
    background: rgb(255 255 255 / 8%);
    color: var(--cream);
    font-size: 0.92em;
  }
  .cta {
    margin-top: 20px;
  }
</style>
