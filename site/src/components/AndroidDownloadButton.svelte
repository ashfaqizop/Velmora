<script lang="ts">
  import { screen } from "../lib/media.svelte";
  import { latest } from "../lib/latest.svelte";
  import { megabytes, RELEASES_URL } from "../lib/release";

  // The Android download: the 64-bit APK, which nearly every phone wants.
  // Falls back to the releases page if GitHub can't be reached.
  let {
    size = "large",
    details = true,
    variant = "filled",
  }: { size?: "medium" | "large"; details?: boolean; variant?: "filled" | "tonal" } = $props();

  const r = $derived(latest.android);
  const apk = $derived(r?.android?.arm64 ?? r?.android?.universal);
  const href = $derived(apk?.url ?? RELEASES_URL);
</script>

<div class="dl">
  <m3e-button variant={variant} size={screen.phone ? "medium" : size} shape="rounded" {href} class="cta" class:tonal={variant === "tonal"}>
    <m3e-icon slot="icon" name="android"></m3e-icon>
    Download for Android
  </m3e-button>
  {#if details}
    <span class="meta">
      {#if apk && r}
        Version {r.version} · {megabytes(apk.size)} · Android 8 and up
      {:else if latest.loaded}
        Android 8 and up · opens the releases page
      {:else}
        Android 8 and up, phones and tablets
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
    --m3e-filled-button-container-color: #3ddc84;
    --m3e-filled-button-label-text-color: #06210f;
    --m3e-filled-button-icon-color: #06210f;
    --m3e-filled-button-hover-label-text-color: #06210f;
    --m3e-filled-button-hover-icon-color: #06210f;
    --m3e-filled-button-focus-label-text-color: #06210f;
    --m3e-filled-button-focus-icon-color: #06210f;
    --m3e-filled-button-pressed-label-text-color: #06210f;
    --m3e-filled-button-pressed-icon-color: #06210f;
    --m3e-filled-button-hover-state-layer-color: #fff;
    --m3e-filled-button-pressed-state-layer-color: #fff;
    --m3e-tonal-button-container-color: rgb(61 220 132 / 16%);
    --m3e-tonal-button-label-text-color: #b6f5d1;
    --m3e-tonal-button-icon-color: #8ff0b8;
    --m3e-tonal-button-hover-label-text-color: #b6f5d1;
    --m3e-tonal-button-hover-icon-color: #8ff0b8;
    --m3e-tonal-button-focus-label-text-color: #b6f5d1;
    --m3e-tonal-button-pressed-label-text-color: #b6f5d1;
    border-radius: 999px;
    box-shadow:
      0 0 0 1px rgb(61 220 132 / 45%),
      0 18px 60px -14px rgb(61 220 132 / 55%);
    transition: transform 0.25s cubic-bezier(0.2, 0.7, 0.2, 1);
  }
  .cta.tonal {
    box-shadow: 0 0 0 1px rgb(61 220 132 / 30%);
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
