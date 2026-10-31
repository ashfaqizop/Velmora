<script lang="ts">
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import * as api from "../lib/api";
  import { formatBytes } from "../lib/sources";
  import { downloads, nav } from "../lib/state.svelte";
  import type { DownloadStatus } from "../lib/types";

  onMount(() => downloads.watch());

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
    if (d.state.state !== "completed") return;
    nav.go({
      name: "player",
      title: { origin: { provider: "local", id: d.state.path }, name: d.title, kind: "movie", ids: {} },
      target: { type: "movie" },
      mode: "sub",
      file: d.state.path,
    });
  }
</script>

<div class="page">
  <header>
    <m3e-heading variant="headline" size="large" level={1}>Downloads</m3e-heading>
    {#if finished}
      <m3e-button variant="text" onclick={() => act(api.downloadsClear())}>
        <m3e-icon slot="icon" name="clear_all"></m3e-icon>Clear finished
      </m3e-button>
    {/if}
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
  {:else}
    <div class="empty">
      <m3e-shape name="soft-burst" class="art"><m3e-icon name="download"></m3e-icon></m3e-shape>
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
    --m3e-shape-size: 120px;
    --m3e-shape-container-color: var(--md-sys-color-primary-container);
    color: var(--md-sys-color-on-primary-container);
    display: grid;
    place-items: center;
    --m3e-icon-size: 48px;
    margin-bottom: 8px;
  }
</style>
