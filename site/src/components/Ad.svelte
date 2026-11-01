<script lang="ts">
  import { onMount } from "svelte";

  // One Adsterra placement. Keys come from Vercel environment variables, so
  // with none set (local dev, forks) this renders nothing at all.
  //
  // Banners run inside their own iframe: Adsterra's banner code reads a global
  // `atOptions`, so two banners on one page would otherwise overwrite each other.
  let { kind = "banner" }: { kind?: "banner" | "native" } = $props();

  const env = import.meta.env;
  const host = env.VITE_ADSTERRA_HOST || "www.highperformanceformat.com";
  const SIZES = [
    { min: 760, key: env.VITE_ADSTERRA_BANNER_728, width: 728, height: 90 },
    { min: 0, key: env.VITE_ADSTERRA_BANNER_300, width: 300, height: 250 },
    { min: 0, key: env.VITE_ADSTERRA_BANNER_320, width: 320, height: 50 },
  ];

  let banner = $state<{ key: string; width: number; height: number } | null>(null);
  let nativeBox = $state<HTMLElement>();
  const nativeSrc = env.VITE_ADSTERRA_NATIVE;
  // The container id Adsterra expects is the hash in the script's path.
  const nativeId = nativeSrc ? `container-${new URL(nativeSrc, location.href).pathname.split("/").filter(Boolean)[0]}` : "";

  onMount(() => {
    if (kind === "banner") {
      const fit = SIZES.find((s) => s.key && innerWidth >= s.min);
      if (fit?.key) banner = { key: fit.key, width: fit.width, height: fit.height };
      return;
    }
    if (nativeSrc && nativeBox) {
      const s = document.createElement("script");
      s.async = true;
      s.dataset.cfasync = "false";
      s.src = nativeSrc;
      nativeBox.before(s);
    }
  });

  const doc = (b: { key: string; width: number; height: number }) =>
    `<!doctype html><html><body style="margin:0;background:transparent">` +
    `<script>atOptions={key:'${b.key}',format:'iframe',height:${b.height},width:${b.width},params:{}};<\/script>` +
    `<script src="https://${host}/${b.key}/invoke.js"><\/script></body></html>`;
</script>

{#if kind === "banner" && banner}
  <aside class="ad" aria-label="Advertisement">
    <span class="tag">Advertisement</span>
    <iframe
      title="Advertisement"
      srcdoc={doc(banner)}
      width={banner.width}
      height={banner.height}
      scrolling="no"
      loading="lazy"
      referrerpolicy="no-referrer-when-downgrade"
    ></iframe>
  </aside>
{:else if kind === "native" && nativeSrc}
  <aside class="ad native" aria-label="Advertisement">
    <span class="tag">Advertisement</span>
    <div id={nativeId} bind:this={nativeBox}></div>
  </aside>
{/if}

<style>
  .ad {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    margin: 96px auto 0;
    width: min(var(--max), 100% - 48px);
  }
  .tag {
    font-size: 11px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: rgb(255 255 255 / 35%);
  }
  iframe {
    border: 0;
    border-radius: 12px;
    max-width: 100%;
    background: var(--glass);
  }
  .native > div {
    width: 100%;
    padding: 16px;
    border-radius: 24px;
    border: 1px solid var(--line);
    background: var(--glass);
  }
</style>
