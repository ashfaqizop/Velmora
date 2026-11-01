<script lang="ts">
  import Logo from "./Logo.svelte";
  import { REPO_URL, RELEASES_URL } from "../lib/release";

  const LINKS = [
    ["/changelog/", "What's new"],
    ["/ani/", "Ani"],
    ["/universeapi/", "UniverseAPI engine"],
    [REPO_URL, "Source code"],
    [RELEASES_URL, "All releases"],
    [`${REPO_URL}/issues`, "Report a problem"],
    [`${REPO_URL}/blob/main/LICENSE`, "License (GPL-3.0)"],
    [`${REPO_URL}/blob/main/SECURITY.md`, "Security"],
  ];
</script>

<footer>
  <div class="wrap">
    <div class="top">
      <div class="brand">
        <Logo size={30} />
        <p>Every source, one search. A free Windows app for movies, series and anime, powered by the open-source <a href="/universeapi/">UniverseAPI engine</a>.</p>
      </div>
      <nav aria-label="Links">
        {#each LINKS as [href, label] (label)}
          <a {href} target={href.startsWith("/") ? undefined : "_blank"} rel="noopener">{label}</a>
        {/each}
      </nav>
    </div>
    <p class="legal">
      Velmora doesn't host, upload or control any of the content it finds. It searches third-party websites the way a browser does, and those sites
      are responsible for what they serve. Please respect the law where you live. Movie and show names on this page are made up for illustration.
    </p>
    <p class="copy">© {new Date().getFullYear()} Velmora contributors · Free software under GPL-3.0-or-later</p>
  </div>
</footer>

<style>
  footer {
    margin-top: 120px;
    padding: 64px 0 40px;
    border-top: 1px solid var(--line);
    background: linear-gradient(180deg, transparent, rgb(123 108 255 / 5%));
    --md-sys-color-on-surface: #f4f1fb;
  }
  .top {
    display: flex;
    justify-content: space-between;
    gap: 48px;
    flex-wrap: wrap;
  }
  .brand {
    max-width: 420px;
  }
  .brand p a {
    color: #c9c1ff;
  }
  .brand p {
    margin: 16px 0 0;
    font-size: 14.5px;
    line-height: 1.6;
    color: var(--muted);
  }
  nav {
    display: grid;
    grid-template-columns: repeat(2, auto);
    gap: 10px 40px;
    align-content: start;
  }
  nav a {
    font-size: 14.5px;
    text-decoration: none;
    color: var(--muted);
    transition: color 0.2s;
  }
  nav a:hover {
    color: #fff;
  }
  .legal {
    margin: 48px 0 0;
    max-width: 860px;
    font-size: 12.5px;
    line-height: 1.6;
    color: rgb(255 255 255 / 40%);
  }
  @media (max-width: 600px) {
    footer {
      margin-top: 80px;
      padding-top: 48px;
    }
    nav {
      gap: 12px 28px;
    }
  }
  .copy {
    margin: 12px 0 0;
    font-size: 12.5px;
    color: rgb(255 255 255 / 40%);
  }
</style>
