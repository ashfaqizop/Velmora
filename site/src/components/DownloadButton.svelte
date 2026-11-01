<script lang="ts">
  import { screen } from "../lib/media.svelte";
  import { latest } from "../lib/latest.svelte";
  import { megabytes, RELEASES_URL } from "../lib/release";

  // The main call to action. Points straight at the newest installer; if
  // GitHub can't be reached it falls back to the releases page.
  let { size = "large", details = true }: { size?: "medium" | "large"; details?: boolean } = $props();

  const r = $derived(latest.current);
  const href = $derived(r?.installer?.url ?? RELEASES_URL);
</script>

<div class="dl">
  <m3e-button variant="filled" size={screen.phone ? "medium" : size} shape="rounded" {href} class="cta">
    <m3e-icon slot="icon" name="download" filled></m3e-icon>
    Download for Windows
  </m3e-button>
  {#if details}
    <span class="meta">
      {#if r?.installer}
        Version {r.version} · {megabytes(r.installer.size)} · Windows 10 and 11, 64-bit
      {:else if latest.loaded}
        Windows 10 and 11, 64-bit · opens the releases page
      {:else}
        Windows 10 and 11, 64-bit
      {/if}
    </span>
  {/if}
</div>

<style>
  .dl {
    max-width: 100%;
    display: inline-flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
  }
  .cta {
    --m3e-filled-button-container-color: #fff;
    --m3e-filled-button-label-text-color: #140f22;
    --m3e-filled-button-icon-color: #140f22;
    --m3e-filled-button-hover-label-text-color: #140f22;
    --m3e-filled-button-hover-icon-color: #140f22;
    --m3e-filled-button-focus-label-text-color: #140f22;
    --m3e-filled-button-focus-icon-color: #140f22;
    --m3e-filled-button-pressed-label-text-color: #140f22;
    --m3e-filled-button-pressed-icon-color: #140f22;
    --m3e-filled-button-hover-state-layer-color: var(--indigo);
    --m3e-filled-button-pressed-state-layer-color: var(--indigo);
    border-radius: 999px;
    box-shadow:
      0 0 0 1px rgb(255 255 255 / 30%),
      0 18px 60px -12px color-mix(in srgb, var(--indigo) 80%, transparent),
      0 10px 30px -10px color-mix(in srgb, var(--coral) 70%, transparent);
    transition: transform 0.25s cubic-bezier(0.2, 0.7, 0.2, 1);
  }
  .cta:hover {
    transform: translateY(-2px);
  }
  .meta {
    max-width: 100%;
    font-size: 13px;
    color: var(--muted);
  }
</style>
