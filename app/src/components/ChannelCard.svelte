<script lang="ts">
  import { live, type FavChannel } from "../lib/live.svelte";

  // One TV channel: logo, name, where it's from, and a heart to keep it.
  let {
    channel,
    onopen,
  }: { channel: FavChannel & { categories?: string[] }; onopen: () => void } = $props();

  const fav = $derived(live.isFavorite(channel.id));
  const initial = $derived(channel.name.trim().charAt(0).toUpperCase());
  const meta = $derived([channel.country, channel.categories?.[0]].filter(Boolean).join(" · "));
</script>

<div class="card">
  <button class="open" onclick={onopen} aria-label="Watch {channel.name}">
    <span class="logo">
      <span class="initial">{initial}</span>
      {#if channel.logo}
        <img src={channel.logo} alt="" loading="lazy" onerror={(e) => ((e.currentTarget as HTMLImageElement).style.display = "none")} />
      {/if}
    </span>
    <span class="text">
      <strong>{channel.name}</strong>
      {#if meta}<span>{meta}</span>{/if}
    </span>
  </button>
  <m3e-icon-button
    class="heart"
    size="small"
    aria-label={fav ? "Remove from favorites" : "Add to favorites"}
    aria-pressed={fav}
    onclick={() => live.toggle(channel)}
  >
    <m3e-icon name="favorite" filled={fav}></m3e-icon>
  </m3e-icon-button>
</div>

<style>
  .card {
    position: relative;
  }
  .open {
    all: unset;
    box-sizing: border-box;
    width: 100%;
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 12px 52px 12px 12px;
    border-radius: 20px;
    cursor: pointer;
    background: var(--md-sys-color-surface-container);
    transition: background 0.2s;
  }
  .open:hover,
  .open:focus-visible {
    background: var(--md-sys-color-surface-container-high);
  }
  .open:focus-visible {
    outline: 2px solid var(--md-sys-color-primary);
  }
  .logo {
    position: relative;
    flex: none;
    width: 56px;
    height: 56px;
    border-radius: 14px;
    overflow: hidden;
    display: grid;
    place-items: center;
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
  }
  .initial {
    font-family: var(--app-display-font);
    font-size: 28px;
  }
  .logo img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: contain;
    padding: 6px;
    background: #fff;
  }
  .text {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .text strong {
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .text span {
    font-size: 12.5px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .heart {
    position: absolute;
    right: 8px;
    top: 50%;
    translate: 0 -50%;
    color: var(--md-sys-color-on-surface-variant);
  }
  .heart[aria-pressed="true"] {
    color: #ff5a7a;
  }
</style>
