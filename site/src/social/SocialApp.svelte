<script lang="ts">
  // Velmora Social: the shell around every page (top bar, menu, footer) and
  // the switch between pages. See nav.svelte.ts for the addresses.
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import mark from "../assets/velmora-mark.webp";
  import Avatar from "../auth/Avatar.svelte";
  import { MAIN_SITE } from "../auth/config";
  import { beginSignIn, signOut } from "../lib/session";
  import { auth, boot, following, followsMe } from "./data.svelte";
  import { go, href, nav, profileHref, startNav } from "./nav.svelte";
  import HomeView from "./HomeView.svelte";
  import ProfileView from "./ProfileView.svelte";
  import SearchView from "./SearchView.svelte";
  import SettingsView from "./SettingsView.svelte";

  const SOCIAL_HOST = "social.getvelmora.dpdns.org";

  let busy = $state(false);
  let quick = $state("");

  onMount(() => {
    const host = location.hostname;
    if (host !== SOCIAL_HOST && host !== "localhost" && host !== "127.0.0.1") {
      location.replace(`https://${SOCIAL_HOST}/social/${location.search}`);
      return;
    }
    startNav();
    boot();
  });

  async function signIn() {
    busy = true;
    await beginSignIn();
  }

  async function leave() {
    await signOut();
    auth.me = null;
    auth.phase = "out";
    following.clear();
    followsMe.clear();
    go("/social/");
  }

  function search(e: Event) {
    e.preventDefault();
    const q = quick.trim();
    quick = "";
    go(q ? `/search?q=${encodeURIComponent(q)}` : "/search");
  }

  const key = $derived(nav.route.name === "profile" ? `profile:${nav.route.user.toLowerCase()}` : nav.route.name);
</script>

<div class="page">
  <div class="glow" aria-hidden="true"></div>

  <header class="bar">
    <a class="brand" href={href("/social/")} aria-label="Velmora Social home">
      <img src={mark} alt="" width="36" height="36" />
      <span class="word">Velmora <em>Social</em></span>
    </a>

    {#if nav.route.name !== "search"}
      <form class="quick" role="search" onsubmit={search}>
        <m3e-icon name="search"></m3e-icon>
        <input bind:value={quick} type="search" placeholder="Find people" aria-label="Find people" autocomplete="off" spellcheck="false" />
      </form>
    {/if}

    <div class="actions">
      {#if nav.route.name !== "search"}
        <m3e-icon-button class="search-btn" aria-label="Find people" href={href("/search")}><m3e-icon name="search"></m3e-icon></m3e-icon-button>
      {/if}
      {#if auth.phase === "in" && auth.me}
        <m3e-button id="me-button" variant="text" class="me-button">
          <span slot="icon" class="mini"><Avatar value={auth.me.avatar} size={28} /></span>
          <span class="me-name">{auth.me.display_name}</span>
          <m3e-icon slot="trailing-icon" name="keyboard_arrow_down"></m3e-icon>
          <m3e-menu-trigger for="me-menu"></m3e-menu-trigger>
        </m3e-button>
        <m3e-menu id="me-menu">
          <m3e-menu-item onclick={() => go(`/@${auth.me!.username}`)}>
            <m3e-icon slot="icon" name="account_circle"></m3e-icon>My profile
          </m3e-menu-item>
          <m3e-menu-item onclick={() => go("/settings")}>
            <m3e-icon slot="icon" name="edit"></m3e-icon>Edit profile
          </m3e-menu-item>
          <m3e-menu-item onclick={() => go(`/@${auth.me!.username}/visitors`)}>
            <m3e-icon slot="icon" name="visibility"></m3e-icon>Who visited
          </m3e-menu-item>
          <m3e-divider></m3e-divider>
          <m3e-menu-item onclick={leave}>
            <m3e-icon slot="icon" name="logout"></m3e-icon>Sign out
          </m3e-menu-item>
        </m3e-menu>
      {:else if auth.phase !== "loading"}
        <m3e-button variant="filled" disabled={busy} onclick={signIn}>
          <m3e-icon slot="icon" name="login"></m3e-icon>Sign in
        </m3e-button>
      {/if}
    </div>
  </header>

  <main class="wrap">
    {#if auth.phase === "loading"}
      <div class="center" in:fade={{ duration: 200, delay: 150 }}><m3e-loading-indicator></m3e-loading-indicator></div>
    {:else}
      {#key key}
        <div class="view" in:fade={{ duration: 220 }}>
          {#if nav.route.name === "home"}
            <HomeView />
          {:else if nav.route.name === "profile"}
            <ProfileView user={nav.route.user} tab={nav.route.tab} />
          {:else if nav.route.name === "search"}
            <SearchView q={nav.route.q} />
          {:else if nav.route.name === "settings"}
            <SettingsView />
          {:else}
            <section class="lost">
              <m3e-heading variant="headline" size="medium" level={1}>This page doesn't exist</m3e-heading>
              <m3e-button variant="filled" href={href("/social/")}>Go home</m3e-button>
            </section>
          {/if}
        </div>
      {/key}
    {/if}
  </main>

  <footer class="foot">
    {#if auth.me}<a href={profileHref(auth.me.username)}>@{auth.me.username}</a>{/if}
    <a href={MAIN_SITE}>getvelmora.dpdns.org</a>
    <a href="{MAIN_SITE}/privacy/">Privacy</a>
    <a href="{MAIN_SITE}/terms/">Terms</a>
  </footer>
</div>

<style>
  .page {
    position: relative;
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    overflow-x: clip;
    background: var(--md-sys-color-surface, #0a0d16);
    color: var(--md-sys-color-on-surface);
  }
  .glow {
    position: absolute;
    inset: -10% -10% auto;
    height: 720px;
    pointer-events: none;
    background:
      radial-gradient(40% 50% at 20% 10%, color-mix(in srgb, var(--md-sys-color-primary) 26%, transparent), transparent 70%),
      radial-gradient(30% 40% at 85% 0%, color-mix(in srgb, #f8e8c8 12%, transparent), transparent 70%);
  }
  .bar {
    position: sticky;
    top: 0;
    z-index: 10;
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 12px clamp(16px, 4vw, 40px);
    backdrop-filter: blur(16px);
    background: color-mix(in srgb, var(--md-sys-color-surface) 72%, transparent);
    border-bottom: 1px solid color-mix(in srgb, var(--md-sys-color-outline-variant) 40%, transparent);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    color: inherit;
    text-decoration: none;
    font-weight: 800;
    font-size: 18px;
    flex: none;
  }
  .brand img {
    border-radius: 10px;
  }
  .brand em {
    font-family: var(--display);
    font-style: italic;
    font-weight: 400;
    color: #f8e8c8;
  }
  .quick {
    flex: 1;
    max-width: 420px;
    margin: 0 auto;
    display: flex;
    align-items: center;
    gap: 10px;
    height: 44px;
    padding: 0 16px;
    border-radius: 999px;
    background: var(--md-sys-color-surface-container-high);
    color: var(--md-sys-color-on-surface-variant);
    transition:
      background-color 160ms ease,
      box-shadow 160ms ease;
  }
  .quick:focus-within {
    background: var(--md-sys-color-surface-container-highest);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--md-sys-color-primary) 60%, transparent);
  }
  .quick input {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: 0;
    background: none;
    color: var(--md-sys-color-on-surface);
    font: inherit;
    font-size: 15px;
  }
  .quick input::-webkit-search-cancel-button {
    display: none;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-left: auto;
    flex: none;
  }
  .search-btn {
    display: none;
  }
  .mini {
    display: inline-grid;
  }
  .me-button {
    --m3e-button-label-text-color: var(--md-sys-color-on-surface);
  }
  .me-name {
    max-width: 160px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .wrap {
    position: relative;
    width: min(980px, 100% - 32px);
    margin: 0 auto;
    padding: 40px 0 72px;
    flex: 1;
  }
  .view {
    display: flex;
    flex-direction: column;
    gap: 28px;
  }
  .center {
    display: grid;
    place-items: center;
    padding: 80px 0;
  }
  .lost {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    padding: 64px 0;
  }
  .foot {
    display: flex;
    flex-wrap: wrap;
    gap: 8px 20px;
    justify-content: center;
    padding: 24px 16px 32px;
    font-size: 13px;
  }
  .foot a {
    color: var(--md-sys-color-on-surface-variant);
    text-decoration: none;
  }
  .foot a:hover {
    color: var(--md-sys-color-on-surface);
  }
  @media (max-width: 720px) {
    .quick {
      display: none;
    }
    .search-btn {
      display: inline-flex;
    }
    .me-name {
      display: none;
    }
  }
  @media (max-width: 380px) {
    .word {
      display: none;
    }
  }
</style>
