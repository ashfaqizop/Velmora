<script lang="ts">
  // Settings → Music → Storage and history: what Music keeps on this PC, and
  // buttons to clear it. Downloads are separate and stay.
  import { onMount } from "svelte";
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import * as api from "../../lib/api";
  import { musicHome } from "../../lib/music/feeds.svelte";

  let info = $state<api.MusicStorage | null>(null);
  let saved = $state<{ songs: number; bytes: number } | null>(null);
  let busy = $state<"cache" | "history" | "songs" | null>(null);

  const refresh = () =>
    Promise.all([
      api
        .musicStorage()
        .then((s) => (info = s))
        .catch(() => {}),
      api
        .musicPrecacheSize()
        .then((s) => (saved = s))
        .catch(() => {}),
    ]);

  onMount(refresh);

  function size(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  }

  async function clear(what: "cache" | "history" | "songs") {
    busy = what;
    try {
      if (what === "songs") await api.musicPrecacheClear();
      else await api.musicStorageClear(what);
      if (what === "cache") musicHome.reset();
      M3eSnackbar.open(
        what === "cache" ? "Saved pages cleared." : what === "songs" ? "Saved songs cleared." : "Search and play history cleared.",
      );
      await refresh();
    } catch (e) {
      M3eSnackbar.open(`Couldn't clear it: ${e}`);
    } finally {
      busy = null;
    }
  }
</script>

<div class="rows">
  <div class="row" data-setting="music-cache">
    <div class="text">
      <strong>Saved pages</strong>
      <span>
        Home, your library, albums, artists and playlists, kept so they open at once and work offline.
        {#if info}{info.cachePages} page{info.cachePages === 1 ? "" : "s"}, {size(info.cacheBytes)}.{/if}
      </span>
    </div>
    <m3e-button variant="tonal" disabled={busy !== null || !info?.cachePages} onclick={() => clear("cache")}>
      <m3e-icon slot="icon" name="delete_sweep"></m3e-icon>{busy === "cache" ? "Clearing…" : "Clear"}
    </m3e-button>
  </div>
  <m3e-divider></m3e-divider>
  <div class="row" data-setting="music-saved">
    <div class="text">
      <strong>Saved songs</strong>
      <span>
        The next songs in your queue, saved ahead so they start at once. Not your downloads.
        {#if saved}{saved.songs} song{saved.songs === 1 ? "" : "s"}, {size(saved.bytes)}.{/if}
      </span>
    </div>
    <m3e-button variant="tonal" disabled={busy !== null || !saved?.songs} onclick={() => clear("songs")}>
      <m3e-icon slot="icon" name="delete_sweep"></m3e-icon>{busy === "songs" ? "Clearing…" : "Clear"}
    </m3e-button>
  </div>
  <m3e-divider></m3e-divider>
  <div class="row" data-setting="music-history">
    <div class="text">
      <strong>Search and play history</strong>
      <span>
        Recent searches in the search box, and the songs you've played.
        {#if info}{info.searches} search{info.searches === 1 ? "" : "es"}, {info.plays} play{info.plays === 1 ? "" : "s"}.{/if}
      </span>
    </div>
    <m3e-button variant="tonal" disabled={busy !== null || !(info?.searches || info?.plays)} onclick={() => clear("history")}>
      <m3e-icon slot="icon" name="history_toggle_off"></m3e-icon>{busy === "history" ? "Clearing…" : "Clear"}
    </m3e-button>
  </div>
  <m3e-divider></m3e-divider>
  <div class="row">
    <div class="text">
      <strong>On disk</strong>
      <span>
        {#if info}{size(info.fileBytes + (saved?.bytes ?? 0))} in total. Old pages are dropped after 30 days, and the oldest go first past 64 MB.
        {:else}Working it out…{/if}
      </span>
    </div>
  </div>
</div>

<style>
  .rows {
    display: flex;
    flex-direction: column;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    padding: 14px 4px;
  }
  .text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .text strong {
    font-weight: 700;
  }
  .text span {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
</style>
