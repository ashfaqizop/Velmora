<script lang="ts">
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import * as api from "../lib/api";
  import { formatBytes } from "../lib/sources";
  import { downloads, nav } from "../lib/state.svelte";
  import type { DownloadStatus, LocalFile } from "../lib/types";

  onMount(() => downloads.watch());

  let folder = $state("");
  let library = $state<LocalFile[]>([]);
  api.downloadFolder().then((f) => (folder = f)).catch(() => {});

  const loadLibrary = () =>
    api
      .downloadsLibrary()
      .then((l) => (library = l))
      .catch((e) => console.warn("downloads library failed", e));

  // Re-read the folder whenever a download finishes, so it moves into "On this PC".
  const completed = $derived(downloads.items.filter((d) => d.state.state === "completed").length);
  $effect(() => {
    void completed;
    loadLibrary();
  });

  // Files finished this session already show in the list above.
  const shown = $derived(new Set(downloads.items.map((d) => (d.state.state === "completed" ? d.state.path : ""))));
  const onDisk = $derived(library.filter((f) => !shown.has(f.path)));

  function playFile(path: string, name: string) {
    nav.go({
      name: "player",
      title: { origin: { provider: "local", id: path }, name, kind: "movie", ids: {} },
      target: { type: "movie" },
      mode: "sub",
      file: path,
    });
  }

  const ago = (secs: number) =>
    new Date(secs * 1000).toLocaleDateString(undefined, { day: "numeric", month: "short", year: "numeric" });

  const STATE_LABEL: Record<string, string> = {
    queued: "Waiting",
    running: "Downloading",
    paused: "Paused",
    extracting: "Unpacking",
    completed: "Done",
    failed: "Failed",
    cancelled: "Cancelled",
  };

  const finished = $derived(downloads.items.some((d) => ["completed", "failed", "cancelled"].includes(d.state.state)));

  function percent(d: DownloadStatus) {
    if (d.state.state === "completed") return 100;
    return d.total ? Math.min(100, (d.downloaded / d.total) * 100) : 0;
  }

  function detail(d: DownloadStatus) {
    if (d.state.state === "failed") return d.state.error;
    const size = d.total ? `${formatBytes(d.downloaded)} of ${formatBytes(d.total)}` : formatBytes(d.downloaded);
    if (d.state.state !== "running" || !d.bytes_per_sec) return size;
    const left = d.total ? Math.max(0, d.total - d.downloaded) / d.bytes_per_sec : 0;
    const eta = left ? ` · ${left >= 3600 ? `${Math.round(left / 3600)} h` : left >= 60 ? `${Math.round(left / 60)} min` : `${Math.round(left)} s`} left` : "";
    return `${size} · ${formatBytes(d.bytes_per_sec)}/s${eta}`;
  }

  const act = (p: Promise<unknown>) => p.catch((e) => console.warn(e)).finally(() => downloads.refresh());

  function play(d: DownloadStatus) {
    if (d.state.state === "completed") playFile(d.state.path, d.title);
  }
</script>

<div class="page">
  <header>
    <div class="title">
      <m3e-heading variant="headline" size="large" level={1}>Downloads</m3e-heading>
      {#if folder}<span class="folder" title={folder}>Saving to {folder}</span>{/if}
    </div>
    <div class="header-actions">
      {#if finished}
        <m3e-button variant="text" onclick={() => act(api.downloadsClear())}>
          <m3e-icon slot="icon" name="clear_all"></m3e-icon>Clear finished
        </m3e-button>
      {/if}
      <m3e-button variant="tonal" onclick={() => api.openDownloadFolder().catch((e) => console.warn(e))}>
        <m3e-icon slot="icon" name="folder_open"></m3e-icon>Open folder
      </m3e-button>
    </div>
  </header>

  {#if downloads.items.length}
    <div class="list">
      {#each [...downloads.items].reverse() as d (d.id)}
        {@const s = d.state.state}
        <m3e-card variant="filled" class="item" in:fade={{ duration: 200 }}>
          <div slot="content" class="row">
            <div class="icon" class:done={s === "completed"} class:bad={s === "failed"}>
              <m3e-icon name={s === "completed" ? "check" : s === "failed" ? "error" : s === "paused" ? "pause" : "download"}></m3e-icon>
            </div>
            <div class="main">
              <div class="top">
                <span class="name">{d.title}</span>
                <span class="state">{STATE_LABEL[s]}</span>
              </div>
              {#if s === "running" || s === "extracting"}
                <m3e-linear-progress-indicator
                  variant="wavy"
                  mode={d.total ? "determinate" : "indeterminate"}
                  value={percent(d)}
                ></m3e-linear-progress-indicator>
              {:else if s !== "completed"}
                <m3e-linear-progress-indicator value={percent(d)}></m3e-linear-progress-indicator>
              {/if}
              <span class="detail" class:error={s === "failed"}>{detail(d)}</span>
            </div>
            <div class="actions">
              {#if s === "completed"}
                <m3e-icon-button variant="filled" aria-label="Play" onclick={() => play(d)}>
                  <m3e-icon name="play_arrow" filled></m3e-icon>
                </m3e-icon-button>
                <m3e-icon-button
                  aria-label="Show in folder"
                  onclick={() => d.state.state === "completed" && act(api.revealFile(d.state.path))}
                >
                  <m3e-icon name="folder_open"></m3e-icon>
                </m3e-icon-button>
              {:else if s === "running" || s === "queued"}
                <m3e-icon-button aria-label="Pause" onclick={() => act(api.downloadPause(d.id))}>
                  <m3e-icon name="pause"></m3e-icon>
                </m3e-icon-button>
              {:else if s === "paused" || s === "failed"}
                <m3e-icon-button aria-label="Resume" onclick={() => act(api.downloadResume(d.id))}>
                  <m3e-icon name={s === "failed" ? "refresh" : "play_arrow"}></m3e-icon>
                </m3e-icon-button>
              {/if}
              {#if s !== "completed" && s !== "cancelled"}
                <m3e-icon-button aria-label="Cancel" onclick={() => act(api.downloadCancel(d.id))}>
                  <m3e-icon name="close"></m3e-icon>
                </m3e-icon-button>
              {/if}
            </div>
          </div>
        </m3e-card>
      {/each}
    </div>
  {/if}

  {#if onDisk.length}
    <section class="disk">
      <m3e-heading variant="title" size="large" level={2}>On this PC</m3e-heading>
      <div class="list">
        {#each onDisk as f (f.path)}
          <m3e-card variant="filled" class="item">
            <div slot="content" class="row">
              <div class="icon done"><m3e-icon name="movie"></m3e-icon></div>
              <div class="main">
                <span class="name">{f.title}</span>
                <span class="detail">{formatBytes(f.size)} · {ago(f.modified)}</span>
              </div>
              <div class="actions">
                <m3e-icon-button variant="filled" aria-label="Play" onclick={() => playFile(f.path, f.title)}>
                  <m3e-icon name="play_arrow" filled></m3e-icon>
                </m3e-icon-button>
                <m3e-icon-button aria-label="Show in folder" onclick={() => act(api.revealFile(f.path))}>
                  <m3e-icon name="folder_open"></m3e-icon>
                </m3e-icon-button>
              </div>
            </div>
          </m3e-card>
        {/each}
      </div>
    </section>
  {/if}

  {#if !downloads.items.length && !onDisk.length}
    <div class="empty">
      <span class="on-shape art">
        <m3e-shape name="soft-burst"></m3e-shape>
        <m3e-icon name="download"></m3e-icon>
      </span>
      <m3e-heading variant="title" size="large">Nothing downloaded yet</m3e-heading>
      <p>Use Download on any movie or episode to keep it for offline viewing.</p>
    </div>
  {/if}
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    gap: 24px;
    padding-bottom: 32px;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }
  .title {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }
  .folder {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .header-actions {
    flex: none;
    display: flex;
    gap: 8px;
  }
  .disk {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .item {
    --m3e-card-shape: var(--md-sys-shape-corner-large, 16px);
  }
  .row {
    display: flex;
    align-items: center;
    gap: 16px;
  }
  .icon {
    flex: none;
    display: grid;
    place-items: center;
    width: 48px;
    height: 48px;
    border-radius: 16px;
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
  }
  .icon.done {
    background: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
  }
  .icon.bad {
    background: var(--md-sys-color-error-container);
    color: var(--md-sys-color-on-error-container);
  }
  .main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .top {
    display: flex;
    justify-content: space-between;
    gap: 12px;
  }
  .name {
    font-weight: 700;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .state {
    flex: none;
    font-size: 13px;
    color: var(--md-sys-color-primary);
  }
  .detail {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
    font-variant-numeric: tabular-nums;
  }
  .detail.error {
    color: var(--md-sys-color-error);
  }
  .actions {
    flex: none;
    display: flex;
    gap: 4px;
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 64px 0;
    text-align: center;
    color: var(--md-sys-color-on-surface-variant);
  }
  .empty p {
    margin: 0;
  }
  .art {
    margin-bottom: 8px;
  }
  /* Set on the elements themselves: the shape paints its own colour, and the
     icon has to sit above it in a contrasting one. */
  .art m3e-shape {
    --m3e-shape-size: 120px;
    --m3e-shape-container-color: var(--md-sys-color-primary-container);
  }
  .art m3e-icon {
    position: relative;
    z-index: 1;
    --m3e-icon-size: 48px;
    color: var(--md-sys-color-on-primary-container);
  }
</style>
