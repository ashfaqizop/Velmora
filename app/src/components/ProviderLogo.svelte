<script lang="ts">
  // A source's logo (bundled in public/providers), or its first letter when
  // there isn't one.
  import { providerLabel } from "../lib/options";
  import type { ProviderId } from "../lib/types";

  const KNOWN = new Set([
    "anilist", "anime_sama", "cinemeta", "coflix", "dramachi", "eztv", "four_k_hd_hub", "french_stream", "hd_hub",
    "hi_anime", "iptv_org", "movie_box", "nyaa", "tmdb", "youtube_music", "yts", "zoko_anime",
  ]);

  let { id, size = 20 }: { id: ProviderId | string; size?: number } = $props();
  let broken = $state(false);
  const has = $derived(KNOWN.has(id) && !broken);
</script>

<span class="logo" style:--size="{size}px" aria-hidden="true">
  {#if has}
    <img src="/providers/{id}.png" alt="" loading="lazy" decoding="async" onerror={() => (broken = true)} />
  {:else}
    {providerLabel(id as ProviderId).charAt(0)}
  {/if}
</span>

<style>
  .logo {
    flex: none;
    display: inline-grid;
    place-items: center;
    width: var(--size);
    height: var(--size);
    border-radius: calc(var(--size) * 0.28);
    overflow: hidden;
    font-size: calc(var(--size) * 0.55);
    font-weight: 800;
    line-height: 1;
    background: var(--md-sys-color-surface-container-highest);
    color: var(--md-sys-color-on-surface-variant);
    vertical-align: middle;
  }
  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
</style>
