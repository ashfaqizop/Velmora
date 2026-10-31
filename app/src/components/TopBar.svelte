<script lang="ts">
  import { onMount } from "svelte";
  import Logo from "./Logo.svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { nav } from "../lib/state.svelte";

  let { onabout }: { onabout: () => void } = $props();

  let input: HTMLInputElement;
  let query = $state(nav.route.name === "search" ? nav.route.query : "");
  let timer: ReturnType<typeof setTimeout> | undefined;
  let maximized = $state(false);

  // Keep the box in sync when leaving search from elsewhere.
  $effect(() => {
    if (nav.route.name !== "search" && document.activeElement !== input) query = "";
  });

  function submit(now = false) {
    clearTimeout(timer);
    const q = query.trim();
    if (q.length < 2) return;
    const go = () => nav.go({ name: "search", query: q });
    if (now) go();
    else timer = setTimeout(go, 380);
  }

  const win = getCurrentWindow();
  const minimize = () => win.minimize();
  const toggleMax = async () => {
    await win.toggleMaximize();
    maximized = await win.isMaximized();
  };
  const close = () => win.close();

  onMount(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        input.focus();
        input.select();
      }
    };
    window.addEventListener("keydown", onKey);
    win.isMaximized().then((m) => (maximized = m));
    return () => window.removeEventListener("keydown", onKey);
  });
</script>

<header class="bar" data-tauri-drag-region>
  <button class="home" onclick={() => nav.go({ name: "home" })} aria-label="Velmora home">
    <Logo size={32} />
  </button>

  <div class="search" data-tauri-drag-region>
    <m3e-search-bar clearable onclear={() => (query = "")}>
      <m3e-icon slot="leading" name="search"></m3e-icon>
      <input
        bind:this={input}
        slot="input"
        placeholder="Search movies, series, anime..."
        aria-label="Search"
        bind:value={query}
        oninput={() => submit()}
        onkeydown={(e) => e.key === "Enter" && submit(true)}
      />
      <span slot="trailing" class="kbd">Ctrl K</span>
    </m3e-search-bar>
  </div>

  <div class="actions">
    <m3e-icon-button id="btn-downloads" aria-label="Downloads" onclick={() => nav.go({ name: "downloads" })}>
      <m3e-icon name="download"></m3e-icon>
    </m3e-icon-button>
    <m3e-tooltip for="btn-downloads">Downloads</m3e-tooltip>

    <m3e-button id="btn-profile" variant="text" class="profile">
      <m3e-avatar slot="icon" class="avatar"><m3e-icon name="person"></m3e-icon></m3e-avatar>
      Guest
      <m3e-icon slot="trailing-icon" name="expand_more"></m3e-icon>
      <m3e-menu-trigger for="profile-menu"></m3e-menu-trigger>
    </m3e-button>
    <m3e-menu id="profile-menu">
      <m3e-menu-item onclick={() => nav.go({ name: "mylist" })}>
        <m3e-icon slot="icon" name="bookmarks"></m3e-icon>My List
      </m3e-menu-item>
      <m3e-menu-item onclick={() => nav.go({ name: "downloads" })}>
        <m3e-icon slot="icon" name="download"></m3e-icon>Downloads
      </m3e-menu-item>
      <m3e-menu-item onclick={() => nav.go({ name: "settings" })}>
        <m3e-icon slot="icon" name="settings"></m3e-icon>Settings
      </m3e-menu-item>
      <m3e-divider></m3e-divider>
      <m3e-menu-item onclick={onabout}>
        <m3e-icon slot="icon" name="info"></m3e-icon>About Velmora
      </m3e-menu-item>
    </m3e-menu>
  </div>

  <div class="window" role="group" aria-label="Window controls">
    <m3e-icon-button size="extra-small" aria-label="Minimize" onclick={minimize}>
      <m3e-icon name="remove"></m3e-icon>
    </m3e-icon-button>
    <m3e-icon-button size="extra-small" aria-label={maximized ? "Restore" : "Maximize"} onclick={toggleMax}>
      <m3e-icon name={maximized ? "filter_none" : "crop_square"}></m3e-icon>
    </m3e-icon-button>
    <m3e-icon-button size="extra-small" class="close" aria-label="Close" onclick={close}>
      <m3e-icon name="close"></m3e-icon>
    </m3e-icon-button>
  </div>
</header>

<style>
  .bar {
    height: var(--app-titlebar-height);
    display: grid;
    grid-template-columns: var(--app-sidebar-width) minmax(0, 1fr) auto auto;
    align-items: center;
    gap: 16px;
    padding-inline: 0 12px;
  }
  .home {
    all: unset;
    cursor: pointer;
    padding-left: 28px;
    height: 32px;
    display: flex;
    align-items: center;
    border-radius: 8px;
  }
  .home:focus-visible {
    outline: 2px solid var(--md-sys-color-primary);
    outline-offset: 4px;
  }
  .search {
    display: flex;
    justify-content: center;
  }
  m3e-search-bar {
    width: min(640px, 100%);
    --m3e-search-bar-container-height: 48px;
    --m3e-search-bar-container-color: var(--md-sys-color-surface-container-high);
  }
  .kbd {
    font-size: 12px;
    padding: 4px 8px;
    border-radius: 8px;
    color: var(--md-sys-color-on-surface-variant);
    background: var(--md-sys-color-surface-container-highest);
    margin-right: 8px;
    white-space: nowrap;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .profile {
    --m3e-button-label-text-color: var(--md-sys-color-on-surface);
  }
  .avatar {
    --m3e-avatar-size: 28px;
  }
  .window {
    display: flex;
    gap: 2px;
    margin-left: 8px;
    align-self: start;
    padding-top: 8px;
  }
  .window .close:hover {
    --m3e-icon-button-icon-color: var(--md-sys-color-error);
  }
</style>
