<script lang="ts">
  // Velmora Admin: the gate, then a left rail and the page you picked.
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import mark from "../assets/velmora-mark.webp";
  import Gate from "./Gate.svelte";
  import { boot, gate, leave } from "./access.svelte";
  import { go, PAGES, route, startRoute } from "./route.svelte";
  import OverviewView from "./OverviewView.svelte";
  import TheatreView from "./theatre/TheatreView.svelte";
  import UsersView from "./UsersView.svelte";
  import ReportsView from "./ReportsView.svelte";
  import ReviewsView from "./ReviewsView.svelte";
  import NamesView from "./NamesView.svelte";
  import AuditView from "./AuditView.svelte";
  import MaxView from "./control/MaxView.svelte";
  import AnalyticsView from "./insight/AnalyticsView.svelte";
  import CurationView from "./insight/CurationView.svelte";
  import NoticesView from "./control/NoticesView.svelte";
  import AdsView from "./control/AdsView.svelte";
  import ConfigView from "./control/ConfigView.svelte";

  const ADMIN_HOST = "admin.getvelmora.dpdns.org";

  onMount(() => {
    const host = location.hostname;
    if (host !== ADMIN_HOST && host !== "localhost" && host !== "127.0.0.1") {
      location.replace(`https://${ADMIN_HOST}/admin/`);
      return;
    }
    startRoute();
    void boot();
  });
</script>

{#if gate.phase !== "ok"}
  <Gate />
{:else}
  <div class="shell">
    <aside class="rail">
      <a class="brand" href="#/overview" aria-label="Velmora Admin">
        <img src={mark} alt="" width="34" height="34" />
        <span>Velmora <em>Admin</em></span>
      </a>
      <nav aria-label="Sections">
        {#each PAGES as p (p.id)}
          <a href="#/{p.id}" class:on={route.page === p.id} aria-current={route.page === p.id ? "page" : undefined}>
            <m3e-icon name={p.icon} filled={route.page === p.id}></m3e-icon><span>{p.label}</span>
          </a>
        {/each}
      </nav>
      <m3e-button class="out" variant="text" onclick={() => leave()}>
        <m3e-icon slot="icon" name="logout"></m3e-icon>Sign out
      </m3e-button>
    </aside>

    <main class="main">
      {#key route.page}
        <div class="view" in:fade={{ duration: 200 }}>
          {#if route.page === "overview"}
            <OverviewView />
          {:else if route.page === "analytics"}
            <AnalyticsView />
          {:else if route.page === "curation"}
            <CurationView />
          {:else if route.page === "theatre"}
            <TheatreView />
          {:else if route.page === "users"}
            <UsersView />
          {:else if route.page === "reports"}
            <ReportsView />
          {:else if route.page === "reviews"}
            <ReviewsView />
          {:else if route.page === "names"}
            <NamesView />
          {:else if route.page === "max"}
            <MaxView />
          {:else if route.page === "notices"}
            <NoticesView />
          {:else if route.page === "ads"}
            <AdsView />
          {:else if route.page === "config"}
            <ConfigView />
          {:else}
            <AuditView />
          {/if}
        </div>
      {/key}
    </main>
  </div>
{/if}

<style>
  .shell {
    min-height: 100vh;
    display: grid;
    grid-template-columns: 232px minmax(0, 1fr);
    background: var(--md-sys-color-surface, #0a0d16);
    color: var(--md-sys-color-on-surface);
  }
  .rail {
    position: sticky;
    top: 0;
    height: 100vh;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 12px;
    border-right: 1px solid color-mix(in srgb, var(--md-sys-color-outline-variant) 40%, transparent);
    background: color-mix(in srgb, var(--md-sys-color-surface-container-low) 70%, transparent);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 8px;
    color: inherit;
    text-decoration: none;
    font-weight: 800;
    font-size: 16px;
  }
  .brand img {
    border-radius: 10px;
  }
  .brand em {
    font-family: var(--display);
    font-style: italic;
    font-weight: 400;
    color: var(--cream);
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
  }
  nav a {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 44px;
    padding: 0 14px;
    border-radius: 999px;
    color: var(--md-sys-color-on-surface-variant);
    text-decoration: none;
    font-size: 14px;
    font-weight: 600;
    transition:
      background-color 160ms ease,
      color 160ms ease;
    --m3e-icon-size: 22px;
  }
  nav a:hover {
    background: color-mix(in srgb, var(--md-sys-color-on-surface) 6%, transparent);
    color: var(--md-sys-color-on-surface);
  }
  nav a.on {
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
  }
  .out {
    align-self: flex-start;
  }
  .main {
    min-width: 0;
    padding: 12px;
  }
  .view {
    width: 100%;
  }
  @media (max-width: 800px) {
    .shell {
      grid-template-columns: minmax(0, 1fr);
    }
    .rail {
      position: sticky;
      z-index: 10;
      height: auto;
      flex-direction: row;
      align-items: center;
      gap: 8px;
      padding: 8px 12px;
      border-right: 0;
      border-bottom: 1px solid color-mix(in srgb, var(--md-sys-color-outline-variant) 40%, transparent);
      backdrop-filter: blur(16px);
      overflow-x: auto;
    }
    .brand span,
    nav a span,
    .out {
      display: none;
    }
    nav {
      flex-direction: row;
    }
    nav a {
      padding: 0 12px;
    }
    .main {
      padding-top: 12px;
    }
  }
</style>
