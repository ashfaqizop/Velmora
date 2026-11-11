<script lang="ts">
  // Artwork for a song, album, playlist or artist, at the size it's shown.
  import { art } from "../../lib/music/ui.svelte";
  import type { MusicThumb } from "../../lib/types";

  let {
    thumbs,
    size = 160,
    round = false,
    wide = false,
    alt = "",
  }: { thumbs: MusicThumb[] | null | undefined; size?: number; round?: boolean; wide?: boolean; alt?: string } = $props();

  const src = $derived(art(thumbs, size));
  let failed = $state(false);
</script>

<div class="art" class:round class:wide style:--size={`${size}px`}>
  {#if src && !failed}
    <img {src} {alt} loading="lazy" decoding="async" onerror={() => (failed = true)} />
  {:else}
    <m3e-icon name="music_note"></m3e-icon>
  {/if}
</div>

<style>
  .art {
    width: var(--size);
    aspect-ratio: 1;
    flex: none;
    border-radius: 14px;
    overflow: hidden;
    display: grid;
    place-items: center;
    background: var(--md-sys-color-surface-container-high);
    color: var(--md-sys-color-on-surface-variant);
  }
  .art.wide {
    aspect-ratio: 16 / 9;
  }
  .art.round {
    border-radius: 50%;
  }
  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
</style>
