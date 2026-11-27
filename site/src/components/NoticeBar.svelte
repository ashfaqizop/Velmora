<script lang="ts">
  // An announcement from the admin site, floating at the bottom of the page.
  // Maintenance messages stay put; the others can be closed.
  import { onMount } from "svelte";
  import { fly } from "svelte/transition";
  import { dismiss, liveNotice, type Notice } from "../lib/notices";

  let notice = $state<Notice | null>(null);

  onMount(() => {
    let alive = true;
    // A moment after load, so it never competes with the page's first paint.
    const t = setTimeout(async () => {
      const n = await liveNotice();
      if (alive) notice = n;
    }, 1200);
    return () => {
      alive = false;
      clearTimeout(t);
    };
  });

  const ICON = { info: "campaign", warning: "warning", maintenance: "construction" } as const;

  function close() {
    if (notice) dismiss(notice);
    notice = null;
  }
</script>

{#if notice}
  <aside class="notice {notice.kind}" role={notice.kind === "maintenance" ? "alert" : "status"} transition:fly={{ y: 40, duration: 420, opacity: 0 }}>
    <span class="ic"><m3e-icon name={ICON[notice.kind]}></m3e-icon></span>
    <div class="text">
      <strong>{notice.title}</strong>
      {#if notice.body}<span>{notice.body}</span>{/if}
    </div>
    {#if notice.link_url}
      <a class="go" href={notice.link_url} target="_blank" rel="noopener">{notice.link_label || "Learn more"}</a>
    {/if}
    {#if notice.kind !== "maintenance"}
      <m3e-icon-button class="x" aria-label="Close" onclick={close}><m3e-icon name="close"></m3e-icon></m3e-icon-button>
    {/if}
  </aside>
{/if}

<style>
  .notice {
    position: fixed;
    z-index: 60;
    left: 50%;
    bottom: max(16px, env(safe-area-inset-bottom));
    translate: -50% 0;
    width: min(600px, calc(100vw - 32px));
    box-sizing: border-box;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 8px 12px 14px;
    border-radius: 22px;
    background: color-mix(in srgb, var(--md-sys-color-surface-container-high, #1d2030) 88%, transparent);
    border: 1px solid color-mix(in srgb, var(--md-sys-color-outline-variant, #444) 50%, transparent);
    box-shadow: 0 18px 48px -12px rgb(0 0 0 / 0.55);
    backdrop-filter: blur(18px) saturate(1.3);
    color: var(--md-sys-color-on-surface, #eee);
    font-size: 14px;
    line-height: 1.4;
    --tone: var(--md-sys-color-primary, #9ab4ff);
  }
  .notice.warning {
    --tone: #ffc56b;
  }
  .notice.maintenance {
    --tone: #ff8a80;
    border-color: color-mix(in srgb, #ff5449 50%, transparent);
  }
  .ic {
    flex: none;
    display: grid;
    place-items: center;
    width: 38px;
    height: 38px;
    border-radius: 14px;
    background: color-mix(in srgb, var(--tone) 20%, transparent);
    color: var(--tone);
    --m3e-icon-size: 22px;
  }
  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .text span {
    color: var(--md-sys-color-on-surface-variant, #bbb);
    font-size: 13px;
  }
  .go {
    flex: none;
    padding: 8px 14px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--tone) 18%, transparent);
    color: var(--tone);
    font-weight: 700;
    font-size: 13px;
    text-decoration: none;
    white-space: nowrap;
  }
  .go:hover {
    background: color-mix(in srgb, var(--tone) 28%, transparent);
  }
  .x {
    flex: none;
  }
  @media (max-width: 520px) {
    .notice {
      flex-wrap: wrap;
    }
    .go {
      order: 4;
      margin-left: 50px;
    }
  }
</style>
