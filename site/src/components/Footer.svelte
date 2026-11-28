<script lang="ts">
  import Logo from "./Logo.svelte";
  import WindowsIcon from "./WindowsIcon.svelte";
  import { REPO_URL, RELEASES_URL } from "../lib/release";

  // Four short columns instead of one long list.
  const GROUPS: { title: string; links: [string, string][] }[] = [
    {
      title: "Get Velmora",
      links: [
        ["/download/?for=windows", "Windows"],
        ["/android/", "Android"],
        ["/changelog/", "What's new"],
        [RELEASES_URL, "All releases"],
      ],
    },
    {
      title: "Learn",
      links: [
        ["/features/", "Features"],
        ["/docs/install/", "Install guide"],
        ["/docs/system-requirements/", "Requirements"],
        ["/docs/account/", "Your account"],
        ["/faq/", "FAQ"],
        ["/ani/", "Ani"],
      ],
    },
    {
      title: "Compare",
      links: [
        ["/free-netflix-alternative/", "Netflix alternative"],
        ["/velmora-vs-stremio/", "vs Stremio"],
        ["/velmora-vs-kodi/", "vs Kodi"],
        ["/velmora-vs-plex/", "vs Plex"],
        ["/stremio-alternatives/", "Stremio alternatives"],
        ["/kodi-alternatives/", "Kodi alternatives"],
        ["/best-free-movie-apps-for-windows/", "Best free movie apps"],
      ],
    },
    {
      title: "Open source",
      links: [
        [REPO_URL, "Source code"],
        ["/universeapi/", "UniverseAPI engine"],
        ["/stats/", "By the numbers"],
        [`${REPO_URL}/issues`, "Report a problem"],
        [`${REPO_URL}/blob/main/SECURITY.md`, "Security"],
        ["/contact/", "Contact"],
      ],
    },
  ];

  const LEGAL: [string, string][] = [
    ["/privacy/", "Privacy"],
    ["/terms/", "Terms"],
    ["/disclaimer/", "Disclaimer and DMCA"],
    [`${REPO_URL}/blob/main/LICENSE`, "GPL-3.0"],
  ];

  const ext = (href: string) => (href.startsWith("/") ? {} : { target: "_blank", rel: "noopener" });
</script>

<footer>
  <div class="wrap">
    <div class="top">
      <div class="brand">
        <Logo size={30} />
        <p>Every source, one search. Free for Windows and Android, powered by the open-source <a href="/universeapi/">UniverseAPI engine</a>.</p>
        <div class="get">
          <a href="/download/?for=windows" class="win"><WindowsIcon size={15} />Windows</a>
          <a href="/android/" class="droid"><m3e-icon name="android"></m3e-icon>Android</a>
        </div>
      </div>
      <nav aria-label="Links">
        {#each GROUPS as g (g.title)}
          <div class="col">
            <span class="head">{g.title}</span>
            {#each g.links as [href, label] (href)}
              <a {href} {...ext(href)}>{label}</a>
            {/each}
          </div>
        {/each}
      </nav>
    </div>

    <div class="bottom">
      <p class="copy">© {new Date().getFullYear()} Velmora contributors</p>
      <div class="legal-links">
        {#each LEGAL as [href, label] (href)}<a {href} {...ext(href)}>{label}</a>{/each}
      </div>
    </div>
    <p class="legal">
      Velmora doesn't host, upload or control any of the content it finds. It searches third-party websites the way a browser does, and those sites
      are responsible for what they serve. Please respect the law where you live. Movie and show names on this page are made up for illustration.
    </p>
  </div>
</footer>

<style>
  footer {
    margin-top: 120px;
    padding: 56px 0 36px;
    border-top: 1px solid var(--line);
    background: linear-gradient(180deg, transparent, rgb(50 113 254 / 5%));
    --md-sys-color-on-surface: #f4f1fb;
  }
  .top {
    display: grid;
    grid-template-columns: minmax(220px, 300px) minmax(0, 1fr);
    gap: 56px;
  }
  .brand p {
    margin: 14px 0 0;
    font-size: 14px;
    line-height: 1.6;
    color: var(--muted);
  }
  .brand p a {
    color: #c9c1ff;
  }
  .get {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 18px;
  }
  .get a {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 13px 6px 10px;
    border-radius: 999px;
    border: 1px solid var(--line);
    background: rgb(255 255 255 / 4%);
    font-size: 13px;
    font-weight: 600;
    color: var(--ink);
    text-decoration: none;
    transition:
      border-color 0.2s,
      background 0.2s;
    --m3e-icon-size: 16px;
  }
  .get a:hover {
    border-color: rgb(50 113 254 / 60%);
    background: rgb(50 113 254 / 10%);
  }
  .get a.win :global(svg) {
    color: #8fb4ff;
  }
  .get a.droid m3e-icon {
    color: #3ddc84;
  }
  .get a.droid:hover {
    border-color: rgb(61 220 132 / 55%);
    background: rgb(61 220 132 / 10%);
  }
  nav {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 28px;
  }
  .col {
    display: flex;
    flex-direction: column;
    gap: 9px;
    min-width: 0;
  }
  .head {
    margin-bottom: 4px;
    font-size: 11.5px;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--ink);
  }
  .col a {
    font-size: 14px;
    text-decoration: none;
    color: var(--muted);
    transition: color 0.2s;
  }
  .col a:hover {
    color: #fff;
  }
  .bottom {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    align-items: center;
    gap: 12px 24px;
    margin-top: 44px;
    padding-top: 20px;
    border-top: 1px solid var(--line);
  }
  .copy {
    margin: 0;
    font-size: 12.5px;
    color: rgb(255 255 255 / 45%);
  }
  .legal-links {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 18px;
  }
  .legal-links a {
    font-size: 12.5px;
    color: rgb(255 255 255 / 55%);
    text-decoration: none;
  }
  .legal-links a:hover {
    color: #fff;
  }
  .legal {
    margin: 14px 0 0;
    max-width: 860px;
    font-size: 12px;
    line-height: 1.6;
    color: rgb(255 255 255 / 35%);
  }
  @media (max-width: 960px) {
    .top {
      grid-template-columns: minmax(0, 1fr);
      gap: 36px;
    }
  }
  @media (max-width: 600px) {
    footer {
      margin-top: 80px;
      padding-top: 44px;
    }
    nav {
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 28px 20px;
    }
  }
</style>
