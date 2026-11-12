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
  import { license } from "./lib/license.svelte";
  import { ads } from "./lib/ads.svelte";
  import { musicSession } from "./lib/music/session.svelte";
  import { musicPlayer } from "./lib/music/player.svelte";
  import { likes, musicUi } from "./lib/music/ui.svelte";
  import { sound } from "./lib/audio.svelte";
  import NowPlaying from "./components/music/NowPlaying.svelte";
  import FullPlayer from "./components/music/FullPlayer.svelte";
  import SongMenu from "./components/music/SongMenu.svelte";
  import AddToPlaylist from "./components/music/AddToPlaylist.svelte";

  let splash = $state(true);
  let onboarding = $state(false);
  let notice: ReturnType<typeof TorrentNotice>;
  let downloadDialog: ReturnType<typeof DownloadDialog>;

  // The shell renders under the splash so it is laid out when revealed;
  // the feed waits for the config refresh before calling the engine.
  feed.load();
  // Max or Public decides whether ads run at all; the rules wait for that answer.
  license.load().then(() => ads.init());
  // YouTube Music sign-in state (only the Music pages need it).
  musicSession.init();

  // The sound options (night mode, even volume...) apply to music too.
  $effect(() => {
    void sound.filters();
    musicPlayer.applyFilters();
  });

  // Music shortcuts, in Music mode and the full player, when not typing.
  function musicKeys(e: KeyboardEvent) {
    if (!musicPlayer.active || !(nav.musicMode || musicUi.full)) return;
    const t = e.target as HTMLElement | null;
    if (t && (t.tagName === "INPUT" || t.tagName === "TEXTAREA" || t.isContentEditable)) return;
    const cur = musicPlayer.current;
    if (e.key === " " && !e.ctrlKey) {
      e.preventDefault();
      musicPlayer.toggle();
    } else if (e.ctrlKey && e.key === "ArrowRight") {
      e.preventDefault();
      musicPlayer.next();
    } else if (e.ctrlKey && e.key === "ArrowLeft") {
      e.preventDefault();
      musicPlayer.prev();
    } else if (e.key.toLowerCase() === "l" && !e.ctrlKey && cur) {
      likes.toggle(cur);
    } else if (e.key.toLowerCase() === "s" && !e.ctrlKey) {
      musicPlayer.toggleShuffle();
    } else if (e.key.toLowerCase() === "r" && !e.ctrlKey) {
      musicPlayer.cycleRepeat();
    } else if (e.key.toLowerCase() === "f" && !e.ctrlKey) {
      musicUi.full = !musicUi.full;
    }
  }

  onMount(() => {
    // The last queue comes back paused, where it stopped.
    musicPlayer.restore();
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
    <Player title={r.title} target={r.target} mode={r.mode} detail={r.detail} file={r.file} channel={r.channel} />
  {/key}
{/if}

{#if onboarding}
  <Onboarding ondone={() => (onboarding = false)} />
{/if}

{#if splash}
  <Splash ondone={splashDone} />
{/if}

<svelte:window onkeydown={musicKeys} />

{#if !splash}
  <NowPlaying />
  {#if musicUi.full}<FullPlayer />{/if}
{/if}
<SongMenu />
<AddToPlaylist />

<TorrentNotice bind:this={notice} />
<DownloadDialog bind:this={downloadDialog} />
