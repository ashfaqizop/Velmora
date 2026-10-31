<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { fade, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import {
    command,
    listenEvents,
    observeProperties,
    setProperty,
    type MpvConfig,
    type MpvObservableProperty,
  } from "tauri-plugin-libmpv-api";
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import * as api from "../lib/api";
  import { startMpv, stopMpv } from "../lib/mpv";
  import type { AudioMode, OpenedStream, Source, SourcesReport, Target, TitleDetail, TitleSummary } from "../lib/types";
  import { formatTime, library, nav, nextEpisode, targetLabel } from "../lib/state.svelte";

  let { title, target, mode, detail }: { title: TitleSummary; target: Target; mode: AudioMode; detail?: TitleDetail } =
    $props();

  const OBSERVED = [
    ["pause", "flag"],
    ["time-pos", "double", "none"],
    ["duration", "double", "none"],
    ["paused-for-cache", "flag"],
    ["cache-buffering-state", "int64", "none"],
    ["volume", "double"],
    ["mute", "flag"],
    ["track-list", "node"],
    ["eof-reached", "flag", "none"],
  ] as const satisfies MpvObservableProperty[];

  type Phase = "sources" | "opening" | "playing" | "error";
  type MpvTrack = { id: number; type: "video" | "audio" | "sub"; title?: string; lang?: string; selected?: boolean };

  let phase = $state<Phase>("sources");
  let status = $state("Finding sources");
  let report = $state<SourcesReport | null>(null);
  let current = $state(-1);
  let failed = $state<number[]>([]);
  let stream: OpenedStream | null = null;

  let paused = $state(false);
  let time = $state(0);
  let duration = $state(0);
  let buffering = $state(false);
  let bufferPct = $state<number | null>(null);
  let volume = $state(Number(localStorage.getItem("velmora.volume") ?? 100));
  let muted = $state(false);
  let tracks = $state<MpvTrack[]>([]);
  let ended = $state(false);
  let torrentInfo = $state<string | null>(null);

  let seeking = $state<number | null>(null);
  let controls = $state(true);
  let sheet = $state(false);
  let fullscreen = $state(false);
  let upNext = $state<number | null>(null);

  const win = getCurrentWindow();
  const cleanups: (() => void)[] = [];
  let hideTimer: ReturnType<typeof setTimeout> | undefined;
  let saveTimer: ReturnType<typeof setInterval> | undefined;
  let upNextTimer: ReturnType<typeof setInterval> | undefined;
  let torrentTimer: ReturnType<typeof setInterval> | undefined;
  let alive = true;

  const episode = $derived<[number, number] | null>(target.type === "episode" ? [target.season, target.episode] : null);
  const next = $derived(nextEpisode(detail, target));
  const source = $derived(report && current >= 0 ? report.sources[current] : null);
  const audioTracks = $derived(tracks.filter((t) => t.type === "audio"));
  const subTracks = $derived(tracks.filter((t) => t.type === "sub"));
  const subsOff = $derived(!subTracks.some((t) => t.selected));
  const episodeTitle = $derived.by(() => {
    if (target.type !== "episode") return "";
    const ep = detail?.seasons.find((s) => s.number === target.season)?.episodes.find((e) => e.number === target.episode);
    return [targetLabel(target), ep?.title].filter(Boolean).join(" · ");
  });

  // ── Lifecycle ──────────────────────────────────────────────────────

  onMount(() => {
    document.documentElement.classList.add("player-active");
    start();
    const onKey = (e: KeyboardEvent) => key(e);
    window.addEventListener("keydown", onKey);
    cleanups.push(() => window.removeEventListener("keydown", onKey));
  });

  onDestroy(() => {
    alive = false;
    saveProgress();
    clearTimeout(hideTimer);
    clearInterval(saveTimer);
    clearInterval(upNextTimer);
    clearInterval(torrentTimer);
    cleanups.forEach((f) => f());
    document.documentElement.classList.remove("player-active");
    if (stream) api.closeStream(stream.id).catch(() => {});
    stopMpv();
    if (fullscreen) win.setFullscreen(false);
  });

  async function start() {
    const config: MpvConfig = {
      initialOptions: {
        vo: "gpu-next",
        hwdec: "auto-safe",
        "keep-open": "yes",
        "force-window": "yes",
        idle: "yes",
        "osd-level": 0,
        "input-default-bindings": "no",
        "input-vo-keyboard": "no",
        cache: "yes",
        "demuxer-max-bytes": "200MiB",
        "demuxer-max-back-bytes": "64MiB",
        "network-timeout": 30,
        ytdl: "no",
        alang: mode === "dub" ? "en,eng" : "ja,jpn,en,eng",
        slang: "en,eng",
        "sub-auto": "fuzzy",
        volume,
      },
      observedProperties: OBSERVED,
    };
    try {
      await startMpv(config);
      cleanups.push(await observeProperties(OBSERVED, onProperty));
      cleanups.push(
        await listenEvents((ev) => {
          if (ev.event === "file-loaded") onLoaded();
          else if (ev.event === "end-file" && ev.reason === "error") onSourceFailed();
        }),
      );
    } catch (e) {
      fail(`The player couldn't start: ${e}`);
      return;
    }

    try {
      report = await api.sources(title, target, mode);
    } catch (e) {
      fail(String(e));
      return;
    }
    if (!alive) return;
    if (!report.sources.length) {
      fail("No source has this one right now.");
      return;
    }
    await play(0, library.resumeAt(title, target));
  }

  function onProperty(ev: { name: string; data: unknown }) {
    switch (ev.name) {
      case "pause":
        paused = ev.data as boolean;
        break;
      case "time-pos":
        time = (ev.data as number | null) ?? 0;
        break;
      case "duration":
        duration = (ev.data as number | null) ?? 0;
        break;
      case "paused-for-cache":
        buffering = ev.data as boolean;
        break;
      case "cache-buffering-state":
        bufferPct = ev.data as number | null;
        break;
      case "volume":
        volume = ev.data as number;
        break;
      case "mute":
        muted = ev.data as boolean;
        break;
      case "track-list":
        tracks = (ev.data as MpvTrack[]) ?? [];
        break;
      case "eof-reached":
        if (ev.data === true && !ended) onEnded();
        break;
    }
  }

  // ── Sources ────────────────────────────────────────────────────────

  async function play(index: number, startAt = 0) {
    const src = report?.sources[index];
    if (!src) return;
    current = index;
    phase = "opening";
    ended = false;
    upNext = null;
    status = src.torrent || src.format === "torrent" ? "Connecting to peers" : `Opening ${providerName(src)}`;

    const previous = stream;
    stream = null;
    if (previous) api.closeStream(previous.id).catch(() => {});

    try {
      const opened = await api.openStream(src, episode);
      if (!alive) {
        api.closeStream(opened.id).catch(() => {});
        return;
      }
      stream = opened;
      if (opened.torrent) watchTorrent(opened.id);
      await setProperty("start", startAt > 0 ? String(startAt) : "none");
      await command("loadfile", [opened.url]);
    } catch (e) {
      console.warn("open failed", e);
      onSourceFailed();
    }
  }

  async function onLoaded() {
    if (!alive) return;
    phase = "playing";
    await setProperty("start", "none").catch(() => {});
    for (const sub of stream?.subtitles ?? []) {
      await command("sub-add", [sub.url, "auto", sub.label, sub.lang ?? ""]).catch(() => {});
    }
    clearInterval(saveTimer);
    saveTimer = setInterval(saveProgress, 10_000);
    poke();
  }

  function onSourceFailed() {
    if (!report || !alive) return;
    if (!failed.includes(current)) failed = [...failed, current];
    const nextIndex = report.sources.findIndex((_, i) => !failed.includes(i));
    if (nextIndex < 0) {
      fail("Every source failed to play. Try again later.");
      return;
    }
    M3eSnackbar.open(`${providerName(report.sources[current])} didn't play, trying the next source`);
    play(nextIndex, Math.floor(time));
  }

  function watchTorrent(id: string) {
    clearInterval(torrentTimer);
    torrentInfo = null;
    torrentTimer = setInterval(async () => {
      const p = await api.torrentStatus(id).catch(() => null);
      if (!p || !alive) return;
      const mb = (n: number) => `${(n / 1_048_576).toFixed(0)} MB`;
      torrentInfo = p.finished ? "Fully downloaded" : `${mb(p.downloaded)} of ${mb(p.total)}`;
      if (p.finished) clearInterval(torrentTimer);
    }, 1000);
  }

  function fail(message: string) {
    phase = "error";
    status = message;
  }

  function pickSource(i: number) {
    sheet = false;
    if (i === current && phase === "playing") return;
    failed = failed.filter((x) => x !== i);
    play(i, Math.floor(time));
  }

  function retry() {
    failed = [];
    phase = "sources";
    status = "Finding sources";
    report = null;
    start();
  }

  // ── Playback ───────────────────────────────────────────────────────

  function saveProgress() {
    if (phase === "playing" && duration > 0) library.saveProgress(title, target, time, duration);
  }

  const togglePause = () => setProperty("pause", !paused);
  const seekBy = (s: number) => command("seek", [s, "relative"]);
  const seekTo = (s: number) => command("seek", [s, "absolute"]);

  function setVolume(v: number) {
    volume = v;
    localStorage.setItem("velmora.volume", String(Math.round(v)));
    setProperty("volume", v);
    if (muted && v > 0) setProperty("mute", false);
  }

  const setAudio = (id: number) => setProperty("aid", id);
  const setSub = (id: number | "no") => setProperty("sid", id === "no" ? "no" : id);

  async function toggleFullscreen() {
    fullscreen = !(await win.isFullscreen());
    await win.setFullscreen(fullscreen);
  }

  function onEnded() {
    ended = true;
    saveProgress();
    if (!next) return;
    upNext = 8;
    clearInterval(upNextTimer);
    upNextTimer = setInterval(() => {
      if (upNext === null) return clearInterval(upNextTimer);
      upNext -= 1;
      if (upNext <= 0) {
        clearInterval(upNextTimer);
        playNext();
      }
    }, 1000);
  }

  function playNext() {
    if (!next) return;
    saveProgress();
    nav.replace({ name: "player", title, target: next, mode, detail });
  }

  function leave() {
    if (fullscreen) win.setFullscreen(false);
    nav.back();
  }

  // Controls fade out after a few idle seconds while video plays.
  function poke() {
    controls = true;
    clearTimeout(hideTimer);
    hideTimer = setTimeout(() => {
      if (phase === "playing" && !paused && !sheet && seeking === null) controls = false;
    }, 3000);
  }

  function key(e: KeyboardEvent) {
    if ((e.target as HTMLElement)?.tagName === "INPUT") return;
    poke();
    switch (e.key) {
      case " ":
      case "k":
        e.preventDefault();
        togglePause();
        break;
      case "ArrowLeft":
        seekBy(-10);
        break;
      case "ArrowRight":
        seekBy(10);
        break;
      case "ArrowUp":
        setVolume(Math.min(130, volume + 5));
        break;
      case "ArrowDown":
        setVolume(Math.max(0, volume - 5));
        break;
      case "f":
        toggleFullscreen();
        break;
      case "m":
        setProperty("mute", !muted);
        break;
      case "Escape":
        if (sheet) sheet = false;
        else if (fullscreen) toggleFullscreen();
        else leave();
        break;
    }
  }

  // ── Labels ─────────────────────────────────────────────────────────

  function providerName(s: Source) {
    return s.provider.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
  }

  function sourceChips(s: Source) {
    const t = s.tags;
    const size = s.size_bytes ?? t.size_bytes;
    return [
      t.resolution ? (t.resolution >= 2160 ? "4K" : `${t.resolution}p`) : null,
      t.codec === "h265" ? "HEVC" : t.codec?.toUpperCase(),
      t.hdr && t.hdr !== "sdr" ? (t.hdr === "dolby_vision" ? "Dolby Vision" : "HDR") : null,
      t.french ?? null,
      t.multi_audio ? "Multi audio" : null,
      size ? `${(size / 1_073_741_824).toFixed(1)} GB` : null,
      s.torrent?.seeders != null ? `${s.torrent.seeders} seeds` : null,
    ].filter(Boolean) as string[];
  }

  const trackName = (t: MpvTrack) => t.title || (t.lang ? t.lang.toUpperCase() : `Track ${t.id}`);
</script>

<div
  class="player"
  class:hide-cursor={!controls && phase === "playing"}
  role="application"
  aria-label="Video player"
  onmousemove={poke}
>
  <!-- The video area itself: click toggles pause, double-click toggles full screen. -->
  <button
    class="surface"
    aria-label={paused ? "Play" : "Pause"}
    tabindex="-1"
    onclick={() => phase === "playing" && togglePause()}
    ondblclick={toggleFullscreen}
  ></button>

  {#if phase !== "playing"}
    <div
      class="backdrop"
      style:background-image={title.backdrop || title.poster ? `url("${title.backdrop ?? title.poster}")` : undefined}
      transition:fade={{ duration: 400 }}
    ></div>
  {/if}

  {#if controls || phase !== "playing"}
    <header class="top" data-tauri-drag-region transition:fade={{ duration: 200 }}>
      <m3e-icon-button variant="tonal" aria-label="Back" onclick={leave}>
        <m3e-icon name="arrow_back"></m3e-icon>
      </m3e-icon-button>
      <div class="heading" data-tauri-drag-region>
        <span class="name display">{title.name}</span>
        {#if episodeTitle}<span class="sub">{episodeTitle}</span>{/if}
      </div>
      {#if report?.sources.length}
        <m3e-button variant="tonal" onclick={() => (sheet = !sheet)}>
          <m3e-icon slot="icon" name="dns"></m3e-icon>
          {source ? [sourceChips(source)[0], providerName(source)].filter(Boolean).join(" · ") : "Sources"}
        </m3e-button>
      {/if}
    </header>
  {/if}

  {#if phase === "sources" || phase === "opening" || (phase === "playing" && buffering)}
    <div class="center" transition:fade={{ duration: 250 }}>
      <m3e-loading-indicator variant="contained"></m3e-loading-indicator>
      <span class="status">
        {#if phase === "playing"}Buffering{bufferPct != null ? ` ${bufferPct}%` : ""}{:else}{status}{/if}
      </span>
      {#if torrentInfo}<span class="hint">{torrentInfo}</span>{/if}
      {#if phase === "sources"}<span class="hint">Checking every provider, this can take a few seconds</span>{/if}
    </div>
  {:else if phase === "error"}
    <div class="center" transition:fade={{ duration: 250 }}>
      <m3e-icon name="error" class="big"></m3e-icon>
      <span class="status">{status}</span>
      {#if report?.errors.length}
        <span class="hint">{report.errors.map((e) => `${e.provider}: ${e.message}`).join(" · ")}</span>
      {/if}
      <div class="row">
        <m3e-button variant="tonal" onclick={leave}>Go back</m3e-button>
        <m3e-button variant="filled" onclick={retry}>
          <m3e-icon slot="icon" name="refresh"></m3e-icon>Try again
        </m3e-button>
      </div>
    </div>
  {/if}

  {#if upNext !== null && next}
    <m3e-card variant="elevated" class="up-next" in:fly={{ x: 40, duration: 400, easing: cubicOut }}>
      <div slot="content" class="up-next-body">
        <span class="overline">Up next · {targetLabel(next)}</span>
        <span class="countdown">Playing in {upNext}s</span>
        <div class="row">
          <m3e-button variant="text" onclick={() => (upNext = null)}>Cancel</m3e-button>
          <m3e-button variant="filled" onclick={playNext}>
            <m3e-icon slot="icon" name="skip_next" filled></m3e-icon>Play now
          </m3e-button>
        </div>
      </div>
    </m3e-card>
  {/if}

  {#if phase === "playing" && (controls || paused)}
    <footer class="bottom" transition:fade={{ duration: 200 }}>
      <div class="timeline">
        <span class="time">{formatTime(seeking ?? time)}</span>
        <m3e-slider class="seek" min="0" max={Math.max(1, Math.floor(duration))} step="1" aria-label="Seek">
          <m3e-slider-thumb
            value={Math.floor(seeking ?? time)}
            oninput={(e: Event) => (seeking = (e.currentTarget as HTMLInputElement & { value: number }).value)}
            onchange={(e: Event) => {
              seekTo((e.currentTarget as HTMLInputElement & { value: number }).value);
              seeking = null;
            }}
          ></m3e-slider-thumb>
        </m3e-slider>
        <span class="time">{formatTime(duration)}</span>
      </div>

      <div class="buttons">
        <m3e-icon-button variant="filled" size="medium" aria-label={paused ? "Play" : "Pause"} onclick={togglePause}>
          <m3e-icon name={paused ? "play_arrow" : "pause"} filled></m3e-icon>
        </m3e-icon-button>
        <m3e-icon-button aria-label="Back 10 seconds" onclick={() => seekBy(-10)}>
          <m3e-icon name="replay_10"></m3e-icon>
        </m3e-icon-button>
        <m3e-icon-button aria-label="Forward 10 seconds" onclick={() => seekBy(10)}>
          <m3e-icon name="forward_10"></m3e-icon>
        </m3e-icon-button>
        {#if next}
          <m3e-icon-button aria-label="Next episode" onclick={playNext}>
            <m3e-icon name="skip_next"></m3e-icon>
          </m3e-icon-button>
        {/if}

        <div class="volume">
          <m3e-icon-button aria-label={muted ? "Unmute" : "Mute"} onclick={() => setProperty("mute", !muted)}>
            <m3e-icon name={muted || volume === 0 ? "volume_off" : volume < 50 ? "volume_down" : "volume_up"}></m3e-icon>
          </m3e-icon-button>
          <m3e-slider min="0" max="130" step="1" aria-label="Volume">
            <m3e-slider-thumb
              value={Math.round(muted ? 0 : volume)}
              oninput={(e: Event) => setVolume((e.currentTarget as HTMLInputElement & { value: number }).value)}
            ></m3e-slider-thumb>
          </m3e-slider>
        </div>

        <span class="spacer"></span>

        {#if audioTracks.length > 1}
          <m3e-icon-button id="btn-audio" aria-label="Audio track">
            <m3e-icon name="music_note"></m3e-icon>
            <m3e-menu-trigger for="menu-audio"></m3e-menu-trigger>
          </m3e-icon-button>
          <m3e-menu id="menu-audio">
            {#each audioTracks as t (t.id)}
              <m3e-menu-item-radio checked={!!t.selected} onclick={() => setAudio(t.id)}>{trackName(t)}</m3e-menu-item-radio>
            {/each}
          </m3e-menu>
        {/if}

        <m3e-icon-button id="btn-subs" aria-label="Subtitles" disabled={!subTracks.length}>
          <m3e-icon name={subsOff ? "subtitles_off" : "subtitles"}></m3e-icon>
          <m3e-menu-trigger for="menu-subs"></m3e-menu-trigger>
        </m3e-icon-button>
        <m3e-menu id="menu-subs">
          <m3e-menu-item-radio checked={subsOff} onclick={() => setSub("no")}>Off</m3e-menu-item-radio>
          {#each subTracks as t (t.id)}
            <m3e-menu-item-radio checked={!!t.selected} onclick={() => setSub(t.id)}>{trackName(t)}</m3e-menu-item-radio>
          {/each}
        </m3e-menu>

        <m3e-icon-button aria-label={fullscreen ? "Exit full screen" : "Full screen"} onclick={toggleFullscreen}>
          <m3e-icon name={fullscreen ? "fullscreen_exit" : "fullscreen"}></m3e-icon>
        </m3e-icon-button>
      </div>
    </footer>
  {/if}

  {#if sheet && report}
    <aside class="sheet" transition:fly={{ x: 380, duration: 350, easing: cubicOut }}>
      <header>
        <m3e-heading variant="title" size="large" level={2}>Sources</m3e-heading>
        <m3e-icon-button aria-label="Close" onclick={() => (sheet = false)}>
          <m3e-icon name="close"></m3e-icon>
        </m3e-icon-button>
      </header>
      <m3e-list>
        {#each report.sources as s, i (i)}
          <m3e-list-action onclick={() => pickSource(i)}>
            <span slot="leading" class="res" class:active={i === current}>{sourceChips(s)[0] ?? "SD"}</span>
            {providerName(s)} · {s.host}
            <span slot="supporting-text">{sourceChips(s).slice(1).join(" · ") || s.label}</span>
            {#if i === current}
              <m3e-icon slot="trailing" name="check_circle" filled class="ok"></m3e-icon>
            {:else if failed.includes(i)}
              <m3e-icon slot="trailing" name="error" class="bad"></m3e-icon>
            {/if}
          </m3e-list-action>
        {/each}
      </m3e-list>
      {#if report.errors.length}
        <m3e-divider></m3e-divider>
        <p class="errors">
          Not available from {report.errors.map((e) => e.provider).join(", ")}.
        </p>
      {/if}
    </aside>
  {/if}
</div>

<style>
  .player {
    position: fixed;
    inset: 0;
    z-index: 60;
    color: #fff;
    background: transparent;
  }
  .player.hide-cursor {
    cursor: none;
  }
  .surface {
    all: unset;
    position: absolute;
    inset: 0;
    cursor: inherit;
  }
  .backdrop {
    position: absolute;
    inset: 0;
    z-index: -1;
    background: #0c0a13 center / cover no-repeat;
    pointer-events: none;
  }
  .backdrop::after {
    content: "";
    position: absolute;
    inset: 0;
    backdrop-filter: blur(24px) brightness(0.35);
  }

  .top {
    position: absolute;
    inset: 0 0 auto;
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px 24px 48px;
    background: linear-gradient(180deg, rgb(0 0 0 / 70%), transparent);
  }
  .heading {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .name {
    font-size: 20px;
    font-weight: 700;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .sub {
    font-size: 13px;
    opacity: 0.8;
  }

  .center {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
    text-align: center;
    max-width: 560px;
    pointer-events: none;
  }
  .center .row {
    pointer-events: auto;
  }
  .status {
    font-size: 18px;
    font-weight: 500;
  }
  .hint {
    font-size: 13px;
    opacity: 0.7;
  }
  .big {
    --m3e-icon-size: 56px;
    color: var(--md-sys-color-error);
  }
  .row {
    display: flex;
    gap: 10px;
    justify-content: flex-end;
  }

  .bottom {
    position: absolute;
    inset: auto 0 0;
    padding: 64px 24px 18px;
    background: linear-gradient(0deg, rgb(0 0 0 / 80%), transparent);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .timeline {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .seek {
    flex: 1;
  }
  .time {
    font-size: 13px;
    font-variant-numeric: tabular-nums;
    min-width: 56px;
    text-align: center;
  }
  .buttons {
    display: flex;
    align-items: center;
    gap: 6px;
    --m3e-icon-button-icon-color: #fff;
  }
  .volume {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-left: 8px;
  }
  .volume m3e-slider {
    width: 120px;
  }
  .spacer {
    flex: 1;
  }

  .up-next {
    position: absolute;
    right: 24px;
    bottom: 140px;
    width: 300px;
  }
  .up-next-body {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .overline {
    font-size: 12px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--md-sys-color-primary);
  }
  .countdown {
    font-size: 18px;
    font-weight: 600;
    color: var(--md-sys-color-on-surface);
  }

  .sheet {
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    width: 380px;
    padding: 16px;
    box-sizing: border-box;
    overflow-y: auto;
    background: var(--md-sys-color-surface-container-low);
    color: var(--md-sys-color-on-surface);
    border-radius: var(--md-sys-shape-corner-extra-large, 28px) 0 0 var(--md-sys-shape-corner-extra-large, 28px);
    box-shadow: -12px 0 40px rgb(0 0 0 / 40%);
  }
  .sheet header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
  }
  .res {
    display: grid;
    place-items: center;
    min-width: 48px;
    height: 32px;
    padding-inline: 6px;
    border-radius: 10px;
    font-size: 12px;
    font-weight: 700;
    background: var(--md-sys-color-surface-container-highest);
  }
  .res.active {
    background: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
  }
  .ok {
    color: var(--md-sys-color-primary);
  }
  .bad {
    color: var(--md-sys-color-error);
  }
  .errors {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
</style>
