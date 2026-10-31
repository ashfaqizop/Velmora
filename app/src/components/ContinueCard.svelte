<script lang="ts">
  import type { HistoryEntry } from "../lib/types";
  import { formatLeft, nav, settings } from "../lib/state.svelte";
  import type { Target } from "../lib/types";

  let { entry }: { entry: HistoryEntry } = $props();

  const t = $derived(entry.title);
  const art = $derived(t.backdrop ?? t.poster);
  const percent = $derived(entry.duration ? (entry.position / entry.duration) * 100 : 0);
  const target = $derived<Target>(
    entry.episode !== undefined ? { type: "episode", season: entry.season ?? 1, episode: entry.episode } : { type: "movie" },
  );
  const sub = $derived(
    [entry.episode ? `S${entry.season ?? 1} E${entry.episode}` : null, formatLeft(entry.duration - entry.position)]
      .filter(Boolean)
      .join(" • "),
  );
</script>

<m3e-card actionable variant="filled" class="cw" onclick={() => nav.go({ name: "player", title: t, target, mode: settings.audioMode })}>
  <div class="art" style:background-image={art ? `url("${art}")` : undefined}>
    <span class="play"><m3e-icon name="play_arrow" filled></m3e-icon></span>
    <m3e-linear-progress-indicator value={percent}></m3e-linear-progress-indicator>
  </div>
  <div class="info">
    <span class="name">{t.name}</span>
    <span class="meta">{sub}</span>
  </div>
</m3e-card>

<style>
  .cw {
    --m3e-card-padding: 0;
    --m3e-card-shape: var(--md-sys-shape-corner-large, 16px);
    width: 280px;
    flex: none;
    overflow: hidden;
  }
  .art {
    position: relative;
    aspect-ratio: 16 / 9;
    background: var(--md-sys-color-surface-container-highest) center / cover;
  }
  .play {
    position: absolute;
    left: 12px;
    bottom: 18px;
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    border-radius: 50%;
    color: #fff;
    background: rgb(0 0 0 / 50%);
    backdrop-filter: blur(8px);
    transition: transform var(--md-sys-motion-spring-fast-spatial, 350ms ease);
  }
  .cw:hover .play {
    transform: scale(1.12);
  }
  m3e-linear-progress-indicator {
    position: absolute;
    left: 12px;
    right: 12px;
    bottom: 8px;
    --m3e-linear-progress-indicator-thickness: 4px;
  }
  .info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 10px 14px 12px;
  }
  .name {
    font-size: 14px;
    font-weight: 500;
  }
  .meta {
    font-size: 12px;
    color: var(--md-sys-color-on-surface-variant);
  }
</style>
