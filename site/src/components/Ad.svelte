<script lang="ts">
  import { onMount } from "svelte";
  import { BANNER_HOST, BANNERS, NATIVE, type BannerSize } from "../lib/ads";

  // One Adsterra placement on the website.
  //   banner: 728x90, 468x60 or 320x50, whichever fits the screen
  //   box:    300x250
  //   rail:   160x600, fixed to the right edge on very wide screens only
  //   native: the Native Banner widget
  //
  // Banners run in their own iframe, because Adsterra's banner code reads a
  // global `atOptions` and two on one page would overwrite each other.
  // Local dev shows nothing.
  let { kind = "banner" }: { kind?: "banner" | "box" | "rail" | "native" } = $props();

  const live = typeof location !== "undefined" && location.hostname !== "localhost" && location.hostname !== "127.0.0.1";

  let size = $state<BannerSize | null>(null);
  let nativeBox = $state<HTMLElement>();

  const dims = (s: BannerSize) => s.split("x").map(Number) as [number, number];

  onMount(() => {
    if (!live) return;
    if (kind === "native") {
      if (nativeBox) {
        const s = document.createElement("script");
        s.async = true;
        s.dataset.cfasync = "false";
        s.src = NATIVE.script;
        nativeBox.before(s);
      }
      return;
    }
    if (kind === "box") size = "300x250";
    else if (kind === "rail") size = innerWidth >= 1640 ? "160x600" : null;
    else size = innerWidth >= 760 ? "728x90" : innerWidth >= 500 ? "468x60" : "320x50";
  });

  const doc = (s: BannerSize) => {
    const [w, h] = dims(s);
    const key = BANNERS[s];
    return (
      `<!doctype html><html><body style="margin:0;background:transparent">` +
      `<script>atOptions={key:'${key}',format:'iframe',height:${h},width:${w},params:{}};<\/script>` +
      `<script src="${BANNER_HOST}/${key}/invoke.js"><\/script></body></html>`
    );
  };
</script>

{#if size}
  {@const [w, h] = dims(size)}
  <aside class="ad" class:rail={kind === "rail"} aria-label="Advertisement">
    <span class="tag">Advertisement</span>
    <iframe title="Advertisement" srcdoc={doc(size)} width={w} height={h} scrolling="no" loading="lazy"></iframe>
  </aside>
{:else if kind === "native" && live}
  <aside class="ad native" aria-label="Advertisement">
    <span class="tag">Advertisement</span>
    <div id={NATIVE.container} bind:this={nativeBox}></div>
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
  .ad.rail {
    position: fixed;
    right: 24px;
    top: 120px;
    width: auto;
    margin: 0;
    z-index: 5;
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
