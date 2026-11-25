<script lang="ts">
  import {
    CONDUCT_URL,
    CONTACT_EMAIL,
    CONTRIBUTING_URL,
    ISSUES_URL,
    NEW_BUG_URL,
    NEW_IDEA_URL,
    SECURITY_POLICY_URL,
    SECURITY_URL,
    SOCIAL_URL,
  } from "../lib/contact";
  import { RELEASES_URL, REPO_URL } from "../lib/release";
  import { inview } from "./inview";
  import Note from "./Note.svelte";
  import PageShell from "./PageShell.svelte";
  import Related from "./Related.svelte";
  import Steps from "./Steps.svelte";

  const WAYS = [
    { icon: "bug_report", title: "Report a bug", text: "Something broke or won't play. Open an issue with the bug report form.", href: NEW_BUG_URL, cta: "Open the form", ext: true, tone: "#ff8a65" },
    { icon: "lightbulb", title: "Suggest an idea", text: "A feature, a source or a screen you'd like. Tell us what you'd use it for.", href: NEW_IDEA_URL, cta: "Share an idea", ext: true, tone: "#ffb74d" },
    { icon: "shield", title: "Report a security problem", text: "Please don't post it publicly. Report it privately on GitHub and you'll hear back within a few days.", href: SECURITY_URL, cta: "Report privately", ext: true, tone: "#8fb4ff" },
    { icon: "gavel", title: "Copyright or legal", text: "Rights holders and legal questions have their own page, with what to include.", href: "/disclaimer/#dmca", cta: "Send a notice", ext: false, tone: "#2dd4bf" },
    { icon: "mail", title: "Write to us", text: "For anything that doesn't fit the rest. Bugs and ideas get answered faster on GitHub.", href: `mailto:${CONTACT_EMAIL}`, cta: CONTACT_EMAIL, ext: false, tone: "#c9c1ff" },
    { icon: "diversity_3", title: "Help build it", text: "Velmora is open source. Read how the code is organised and what a pull request needs.", href: CONTRIBUTING_URL, cta: "Read the guide", ext: true, tone: "#f8e8c8" },
  ];

  const REPORT = [
    { title: "Say what you did", text: "The title you searched, the source or link you picked, and what you pressed. The exact steps matter more than the symptom." },
    { title: "Say what happened", text: "Did it freeze, show an error, or play without sound? Copy the error text if there is any. A screenshot helps a lot." },
    { title: "Say what you run", text: "Your Velmora version, your Windows version, and your graphics card if it's about video." },
    { title: "Check for a match first", text: "Search the existing issues. If someone already reported it, add what you know there instead of starting another." },
  ];

  const BUILD = [
    { title: "Open an issue first for anything big", text: "A new source, a new screen, or a change to how links are ranked. We'll agree on the approach before you spend time on it. Small fixes can go straight to a pull request." },
    { title: "Set up", text: "You need Node 22, stable Rust and, on Windows, NASM. Then run npm ci and npm run tauri dev inside the app folder. On first run Velmora downloads mpv and ffmpeg, the same as a normal install." },
    { title: "Make the checks pass", text: "Every pull request runs the app's type and build checks and a full Windows build, and all of them must pass. The contributing guide lists them." },
    { title: "Be kind", text: "Everyone who takes part follows the code of conduct." },
  ];

  const LINKS = [
    { label: "Source code", href: REPO_URL },
    { label: "All releases", href: RELEASES_URL },
    { label: "Open issues", href: ISSUES_URL },
    { label: "Security policy", href: SECURITY_POLICY_URL },
    { label: "Code of conduct", href: CONDUCT_URL },
    { label: "Velmora Social", href: SOCIAL_URL },
  ];
</script>

<PageShell
  eyebrow="Contact"
  title="Talk to us,"
  accent="or help build it."
  crumb="Contact"
  lead="Velmora is made by a small team that reads everything. Pick the way that fits, and you'll reach the right place."
  spots={[
    { x: "92%", y: "190px", size: 116, color: "#ffb74d", spin: 22, opacity: 0.55 },
    { x: "5%", y: "390px", size: 56, color: "#3271fe", spin: -15, offset: 6, opacity: 0.7 },
  ]}
>
  <section class="pg-section tight">
    <div class="pg-wide">
      <div class="pg-grid ways">
        {#each WAYS as w, i (w.title)}
          <a class="pg-card way" href={w.href} target={w.ext ? "_blank" : undefined} rel={w.ext ? "noopener" : undefined} style="--tone: {w.tone}" use:inview={i * 0.06}>
            <span class="ico"><m3e-icon name={w.icon}></m3e-icon></span>
            <h3>{w.title}</h3>
            <p>{w.text}</p>
            <span class="go">{w.cta}<m3e-icon name={w.ext ? "arrow_outward" : "arrow_forward"}></m3e-icon></span>
          </a>
        {/each}
      </div>
    </div>
  </section>

  <section class="pg-section">
    <div class="pg-wide narrow">
      <span class="eyebrow" use:inview>Bug reports</span>
      <h2 class="pg-title" use:inview={0.05}>A report that <em>gets fixed</em></h2>
      <Steps steps={REPORT} />
    </div>
  </section>

  <section class="pg-section">
    <div class="pg-wide narrow">
      <span class="eyebrow" use:inview>Contribute</span>
      <h2 class="pg-title" use:inview={0.05}>Help <em>build it</em></h2>
      <Steps steps={BUILD} />
      <Note label="Security problems never go in a public issue" icon="shield" variant="secondary">
        Use the <a href={SECURITY_URL} target="_blank" rel="noopener">private report form</a>. The <a href={SECURITY_POLICY_URL} target="_blank" rel="noopener">security policy</a> says what's in scope and what to include.
      </Note>
    </div>
  </section>

  <section class="pg-section">
    <div class="pg-wide narrow">
      <span class="eyebrow" use:inview>Elsewhere</span>
      <div class="chips" use:inview>
        {#each LINKS as l (l.label)}
          <m3e-button variant="tonal" size="small" href={l.href} target="_blank">{l.label}<m3e-icon slot="trailing-icon" name="arrow_outward"></m3e-icon></m3e-button>
        {/each}
      </div>
    </div>
  </section>

  <Related
    items={[
      { href: "/faq/", icon: "help", title: "FAQ", text: "Your question may already have an answer." },
      { href: "/docs/install/", icon: "menu_book", title: "Install guide", text: "Step by step, with the SmartScreen warning explained." },
      { href: "/changelog/", icon: "history", title: "What's new", text: "See whether your problem was fixed in a newer version." },
    ]}
  />
</PageShell>

<style>
  section.tight {
    padding-top: 44px;
  }
  .narrow {
    width: min(820px, 100% - 48px);
    margin-inline: auto;
  }
  .ways {
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  }
  .way {
    display: flex;
    flex-direction: column;
    gap: 0;
  }
  .ico {
    display: inline-grid;
    place-items: center;
    width: 54px;
    height: 54px;
    border-radius: 18px;
    background: color-mix(in srgb, var(--tone) 18%, transparent);
    color: var(--tone);
    --m3e-icon-size: 28px;
    transition: transform 0.4s cubic-bezier(0.34, 1.6, 0.5, 1);
  }
  .way:hover .ico {
    transform: rotate(-8deg) scale(1.1);
  }
  .go {
    margin-top: auto;
    padding-top: 18px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-weight: 700;
    font-size: 14.5px;
    color: var(--tone);
    word-break: break-all;
    --m3e-icon-size: 18px;
  }
  .way:hover .go m3e-icon {
    translate: 3px 0;
  }
  .go m3e-icon {
    transition: translate 0.25s;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 16px;
  }
  @media (max-width: 600px) {
    .narrow {
      width: calc(100% - 32px);
    }
  }
</style>
