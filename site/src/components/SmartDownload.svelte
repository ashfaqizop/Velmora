<script lang="ts">
  import AndroidDownloadButton from "./AndroidDownloadButton.svelte";
  import DownloadButton from "./DownloadButton.svelte";

  // One download, for the device you're on: Android phones get the APK,
  // everything else the Windows installer. The other one is a line below.
  const android = typeof navigator !== "undefined" && /android/i.test(navigator.userAgent);
</script>

<div class="smart">
  {#if android}
    <AndroidDownloadButton />
    <p class="also">
      <m3e-icon name="desktop_windows"></m3e-icon>
      <span>Also on Windows. Open <b>getvelmora.dpdns.org</b> on your PC to get it.</span>
    </p>
  {:else}
    <DownloadButton />
    <p class="also">
      <m3e-icon name="android"></m3e-icon>
      <span>Also on Android. Open this site on your phone, or <a href="/android/">see Velmora for Android</a>.</span>
    </p>
  {/if}
</div>

<style>
  .smart {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
  }
  .smart :global(.dl) {
    align-items: center;
  }
  .also {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    margin: 0;
    max-width: 420px;
    font-size: 13px;
    line-height: 1.45;
    color: var(--muted);
    text-align: left;
    --m3e-icon-size: 18px;
  }
  .also m3e-icon {
    flex: none;
    color: #8ff0b8;
  }
  .also b {
    color: var(--ink);
    font-weight: 600;
  }
  .also a {
    color: #8ff0b8;
  }
</style>
