<script lang="ts">
  // An announcement from the admin site, at the top of the page. Maintenance
  // messages can't be closed; the others can, once per version.
  import { onMount } from "svelte";
  import { fly } from "svelte/transition";
  import * as api from "../lib/api";
  import { spring, travel } from "../lib/motion.svelte";
  import { notices } from "../lib/notices.svelte";

  onMount(() => notices.start());

  const ICON = { info: "campaign", warning: "warning", maintenance: "construction" } as const;
  const n = $derived(notices.current);
</script>

{#if n}
  {#key n.id}
    <aside class="notice {n.kind}" role={n.kind === "maintenance" ? "alert" : "status"} in:fly={{ y: travel(-12), opacity: 0, ...spring("default-spatial") }}>
      <span class="ic"><m3e-icon name={ICON[n.kind]}></m3e-icon></span>
      <div class="text">
        <strong>{n.title}</strong>
        {#if n.body}<span>{n.body}</span>{/if}
      </div>
      {#if n.link_url}
        <m3e-button variant="tonal" size="small" onclick={() => api.openExternal(n.link_url)}>{n.link_label || "Learn more"}</m3e-button>
      {/if}
      {#if n.kind !== "maintenance"}
        <m3e-icon-button aria-label="Close" onclick={() => notices.dismiss(n)}><m3e-icon name="close"></m3e-icon></m3e-icon-button>
      {/if}
    </aside>
  {/key}
{/if}

<style>
  .notice {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 16px;
    padding: 10px 8px 10px 12px;
    border-radius: 20px;
    background: color-mix(in srgb, var(--tone) 12%, var(--md-sys-color-surface-container));
    border: 1px solid color-mix(in srgb, var(--tone) 30%, transparent);
    color: var(--md-sys-color-on-surface);
    font-size: 14px;
    line-height: 1.4;
    --tone: var(--md-sys-color-primary);
  }
  .notice.warning {
    --tone: #ffc56b;
  }
  .notice.maintenance {
    --tone: var(--md-sys-color-error);
  }
  .ic {
    flex: none;
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    border-radius: 12px;
    background: color-mix(in srgb, var(--tone) 22%, transparent);
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
    color: var(--md-sys-color-on-surface-variant);
    font-size: 13px;
  }
</style>
