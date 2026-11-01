<script lang="ts">
  import Logo from "./Logo.svelte";
  import { latest } from "../lib/latest.svelte";
  import { REPO_URL, RELEASES_URL } from "../lib/release";

  let scrolled = $state(false);
  let open = $state(false);
  const LINKS = [
    ["/#features", "Features"],
    ["/ani/", "Ani"],
    ["/universeapi/", "Engine"],
    ["/#install", "Install"],
    ["/changelog/", "What's new"],
    ["/#faq", "FAQ"],
  ];
</script>

<svelte:window onscroll={() => (scrolled = scrollY > 12)} onkeydown={(e) => e.key === "Escape" && (open = false)} />

<header class:scrolled={scrolled || open}>
  <div class="bar wrap">
    <a href="/" class="brand" aria-label="Velmora home"><Logo size={26} /></a>
    <nav aria-label="Sections">
      {#each LINKS as [href, label] (href)}<a {href}>{label}</a>{/each}
    </nav>
    <div class="actions">
      <m3e-icon-button href={REPO_URL} target="_blank" aria-label="Velmora on GitHub">
        <m3e-icon name="code"></m3e-icon>
      </m3e-icon-button>
      <m3e-button variant="tonal" size="small" href={latest.current?.installer?.url ?? RELEASES_URL}>
        <m3e-icon slot="icon" name="download"></m3e-icon>Download
      </m3e-button>
      <m3e-icon-button class="menu-btn" aria-label={open ? "Close menu" : "Menu"} aria-expanded={open} onclick={() => (open = !open)}>
        <m3e-icon name={open ? "close" : "menu"}></m3e-icon>
      </m3e-icon-button>
    </div>
  </div>
  {#if open}
    <nav class="sheet" aria-label="Sections">
      {#each LINKS as [href, label] (href)}<a {href} onclick={() => (open = false)}>{label}</a>{/each}
      <a href={REPO_URL} target="_blank" rel="noopener">Source code</a>
    </nav>
  {/if}
</header>

<style>
  header {
    position: fixed;
    inset: 0 0 auto;
    z-index: 50;
    transition:
      background 0.3s,
      border-color 0.3s,
      backdrop-filter 0.3s;
    border-bottom: 1px solid transparent;
    --md-sys-color-on-surface: #f4f1fb;
  }
  header.scrolled {
    background: rgb(12 10 19 / 72%);
    backdrop-filter: blur(18px) saturate(1.3);
    border-color: var(--line);
  }
  .bar {
    display: flex;
    align-items: center;
    gap: 24px;
    height: 72px;
  }
  .brand {
    display: flex;
    text-decoration: none;
  }
  nav {
    display: flex;
    gap: 4px;
    margin-inline: auto;
  }
  nav a {
    padding: 8px 14px;
    border-radius: 999px;
    font-size: 14px;
    font-weight: 500;
    text-decoration: none;
    color: var(--muted);
    transition:
      color 0.2s,
      background 0.2s;
  }
  nav a:hover {
    color: #fff;
    background: rgb(255 255 255 / 6%);
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .menu-btn,
  .sheet {
    display: none;
  }
  @media (max-width: 860px) {
    nav {
      display: none;
    }
    .actions {
      margin-left: auto;
    }
    .menu-btn {
      display: inline-flex;
    }
    .sheet {
      display: flex;
      flex-direction: column;
      gap: 2px;
      width: calc(100% - 32px);
      margin: 0 auto;
      padding: 8px 0 16px;
      animation: drop 0.3s cubic-bezier(0.2, 0.7, 0.2, 1);
    }
    .sheet a {
      padding: 14px 12px;
      border-radius: 14px;
      font-size: 17px;
      font-weight: 600;
      text-decoration: none;
      color: var(--ink);
    }
    .sheet a:hover {
      background: rgb(255 255 255 / 6%);
    }
  }
  @media (max-width: 420px) {
    .actions > m3e-icon-button:first-child {
      display: none;
    }
    .bar {
      gap: 12px;
      height: 64px;
    }
  }
  @keyframes drop {
    from {
      opacity: 0;
      transform: translateY(-8px);
    }
  }
</style>
