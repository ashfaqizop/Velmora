<script lang="ts">
  import { onMount } from "svelte";
  import Splash from "./components/Splash.svelte";
  import Shell from "./components/Shell.svelte";
  import Onboarding from "./components/Onboarding.svelte";
  import TorrentNotice from "./components/TorrentNotice.svelte";
  import DownloadDialog from "./components/DownloadDialog.svelte";
  import Player from "./pages/Player.svelte";
  import { registerOverlays } from "./lib/overlays";
  import { downloads, feed, nav } from "./lib/state.svelte";

  let splash = $state(true);
  let onboarding = $state(false);
  let notice: ReturnType<typeof TorrentNotice>;
  let downloadDialog: ReturnType<typeof DownloadDialog>;

  // The shell renders under the splash so it is laid out when revealed;
  // the feed waits for the config refresh before calling the engine.
  feed.load();

  onMount(() => {
    registerOverlays({
      torrentNotice: () => notice.ask(),
      download: (title, target) => downloadDialog.open(title, target),
    });
  });

  function splashDone() {
    splash = false;
    onboarding = localStorage.getItem("velmora.onboarded") !== "1";
    downloads.refresh();
  }
</script>

<Shell revealed={!splash} />

{#if nav.route.name === "player"}
  {@const r = nav.route}
  {#key r}
    <Player title={r.title} target={r.target} mode={r.mode} detail={r.detail} file={r.file} />
  {/key}
{/if}

{#if onboarding}
  <Onboarding ondone={() => (onboarding = false)} />
{/if}

{#if splash}
  <Splash ondone={splashDone} />
{/if}

<TorrentNotice bind:this={notice} />
<DownloadDialog bind:this={downloadDialog} />
