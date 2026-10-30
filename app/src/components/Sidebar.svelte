<script lang="ts">
  import PoweredBy from "./PoweredBy.svelte";
  import { feed, library, nav, type Route } from "../lib/state.svelte";

  const main: { label: string; icon: string; route: Route }[] = [
    { label: "Home", icon: "home", route: { name: "home" } },
    { label: "Movies", icon: "movie", route: { name: "browse", kind: "movie" } },
    { label: "Series", icon: "tv", route: { name: "browse", kind: "series" } },
    { label: "Anime", icon: "animation", route: { name: "browse", kind: "anime" } },
    { label: "My List", icon: "bookmarks", route: { name: "mylist" } },
    { label: "Downloads", icon: "download", route: { name: "downloads" } },
  ];

  // Collections are the engine's home rails; dot colours echo the reference's genre list.
  const DOTS = ["#ff6b6b", "#ffb547", "#5ad17a", "#4fa3ff", "#a77bff", "#ff7ad9", "#3fd3c6", "#c792ea"];
  const collections = $derived(feed.rails.map((r, i) => ({ id: r.id, label: r.title, color: DOTS[i % DOTS.length] })));

  function isActive(r: Route) {
    const cur = nav.route;
    if (cur.name !== r.name) return false;
    if (cur.name === "browse" && r.name === "browse") return cur.kind === r.kind;
    return true;
  }
</script>

<aside class="sidebar">
  <m3e-nav-menu class="menu">
    {#each main as item (item.label)}
      <m3e-nav-menu-item selected={isActive(item.route)} onclick={() => nav.go(item.route)}>
        <m3e-icon slot="icon" name={item.icon}></m3e-icon>
        <m3e-icon slot="selected-icon" name={item.icon} filled></m3e-icon>
        <span slot="label">{item.label}</span>
        {#if item.route.name === "mylist" && library.watchlist.length}
          <span slot="badge">{library.watchlist.length}</span>
        {/if}
      </m3e-nav-menu-item>
    {/each}

    {#if collections.length}
      <m3e-divider></m3e-divider>
      <m3e-nav-menu-item-group>
        <m3e-heading slot="label" variant="label" size="large">Collections</m3e-heading>
        {#each collections as c (c.id)}
          <m3e-nav-menu-item
            selected={nav.route.name === "collection" && nav.route.id === c.id}
            onclick={() => nav.go({ name: "collection", id: c.id })}
          >
            <span slot="icon" class="dot" style:background={c.color}></span>
            <span slot="label">{c.label}</span>
          </m3e-nav-menu-item>
        {/each}
      </m3e-nav-menu-item-group>
    {/if}
  </m3e-nav-menu>

  <div class="promo">
    <m3e-shape name="flower" class="s1"></m3e-shape>
    <m3e-shape name="soft-burst" class="s2"></m3e-shape>
    <m3e-shape name="circle" class="s3"></m3e-shape>
    <div class="copy">
      <strong class="display">Better movies.<br />Brighter days.</strong>
      <span>Every source, one search away.</span>
    </div>
    <div class="credit"><PoweredBy height={10} /></div>
  </div>
</aside>

<style>
  .sidebar {
    height: 100%;
    display: flex;
    flex-direction: column;
    min-height: 0;
    padding: 4px 12px 16px;
    box-sizing: border-box;
  }
  .menu {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    --m3e-nav-menu-item-height: 48px;
    --m3e-nav-menu-item-shape: 999px;
    --m3e-nav-menu-item-selected-container-color: var(--md-sys-color-primary);
    --m3e-nav-menu-item-selected-label-color: var(--md-sys-color-on-primary);
    --m3e-nav-menu-padding-left: 0;
    --m3e-nav-menu-padding-right: 0;
  }
  .dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    margin-inline: 7px;
  }
  .promo {
    position: relative;
    flex: none;
    margin-top: 12px;
    height: 200px;
    border-radius: var(--md-sys-shape-corner-extra-large, 28px);
    overflow: hidden;
    padding: 20px;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    gap: 6px;
    background: linear-gradient(
      150deg,
      var(--md-sys-color-primary-container) 0%,
      color-mix(in srgb, var(--md-sys-color-tertiary-container) 70%, #000) 100%
    );
    color: var(--md-sys-color-on-primary-container);
  }
  .promo m3e-shape {
    position: absolute;
    opacity: 0.55;
  }
  .s1 {
    --m3e-shape-size: 72px;
    --m3e-shape-container-color: var(--md-sys-color-primary);
    top: 14px;
    left: 16px;
  }
  .s2 {
    --m3e-shape-size: 120px;
    --m3e-shape-container-color: var(--brand-coral);
    top: -36px;
    right: -30px;
    opacity: 0.35;
  }
  .s3 {
    --m3e-shape-size: 46px;
    --m3e-shape-container-color: var(--md-sys-color-tertiary);
    top: 70px;
    right: 40px;
  }
  .copy {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .copy strong {
    font-size: 19px;
    line-height: 1.2;
  }
  .copy span {
    font-size: 12px;
    opacity: 0.85;
  }
  .credit {
    position: relative;
    margin-top: 8px;
    opacity: 0.9;
  }
</style>
