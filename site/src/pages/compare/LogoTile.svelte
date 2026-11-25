<script lang="ts">
  // An app's logo on a light tile, so every brand color reads on the dark page.
  // With no logo (Pluto TV isn't in the icon set) it shows a plain tile with the
  // app's first letter over one of the site's Material shapes.
  let { src, name, color = "#8fb4ff", shape = "soft-burst", size = 56 }: { src?: string; name: string; color?: string; shape?: string; size?: number } = $props();
</script>

<span class="tile" style:--size="{size}px" style:--c={color} title={name}>
  {#if src}
    <img {src} alt="{name} logo" width={size * 0.62} height={size * 0.62} loading="lazy" decoding="async" />
  {:else}
    <m3e-shape name={shape}></m3e-shape>
    <b>{name.slice(0, 1)}</b>
  {/if}
</span>

<style>
  .tile {
    position: relative;
    flex: none;
    display: inline-grid;
    place-items: center;
    width: var(--size);
    height: var(--size);
    border-radius: calc(var(--size) * 0.3);
    background: #f4f1fb;
    box-shadow:
      0 10px 28px -12px rgb(0 0 0 / 60%),
      0 0 0 1px rgb(255 255 255 / 30%) inset;
    overflow: hidden;
  }
  img {
    display: block;
    object-fit: contain;
  }
  m3e-shape {
    position: absolute;
    inset: 12%;
    --m3e-shape-size: calc(var(--size) * 0.76);
    --m3e-shape-container-color: color-mix(in srgb, var(--c) 35%, transparent);
  }
  b {
    position: relative;
    font-size: calc(var(--size) * 0.42);
    color: #1b1530;
  }
</style>
