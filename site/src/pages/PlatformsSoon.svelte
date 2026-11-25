<script lang="ts">
  // Other platforms: shown as coming soon, with no dates, because none exist yet.
  import { inview } from "./inview";

  const PLATFORMS = [
    { icon: "android", shape: "12-sided-cookie", color: "#3ddc84", name: "Android", text: "A full native build, designed with Material You so it takes its colors from your wallpaper." },
    { icon: "phone_iphone", shape: "soft-burst", color: "#8fb4ff", name: "iPhone and iPad", text: "A native SwiftUI app, designed with Apple's Liquid Glass." },
    { icon: "laptop_mac", shape: "puffy-diamond", color: "#c9c1ff", name: "Mac", text: "A native SwiftUI app with Liquid Glass, made for macOS." },
    { icon: "terminal", shape: "4-leaf-clover", color: "#ffb74d", name: "Linux", text: "Linux builds, so Velmora runs on more than Windows." },
  ];
</script>

<section class="pg-section">
  <div class="pg-wide">
    <span class="eyebrow" use:inview>More platforms</span>
    <h2 class="pg-title" use:inview={0.05}>Coming <em>soon</em></h2>
    <p class="pg-lead" use:inview={0.1}>Velmora runs on Windows today. These are on the way. There are no dates yet, and this page will change when there's something to download.</p>
    <div class="grid">
      {#each PLATFORMS as p, i (p.name)}
        <article class="pg-card plat" style="--c: {p.color}" use:inview={i * 0.07}>
          <span class="art">
            <m3e-shape name={p.shape}></m3e-shape>
            <m3e-icon name={p.icon}></m3e-icon>
          </span>
          <span class="soon">Coming soon</span>
          <h3>{p.name}</h3>
          <p>{p.text}</p>
        </article>
      {/each}
    </div>
    <p class="ask pg-lead" use:inview>Want one first? <a href="/contact/">Tell us which</a>.</p>
  </div>
</section>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: 16px;
    margin-top: 34px;
  }
  .plat {
    overflow: hidden;
  }
  .art {
    position: relative;
    display: grid;
    place-items: center;
    width: 76px;
    height: 76px;
  }
  .art m3e-shape {
    position: absolute;
    inset: 0;
    --m3e-shape-size: 76px;
    --m3e-shape-container-color: color-mix(in srgb, var(--c) 30%, transparent);
    --m3e-shape-transition: clip-path 0.9s cubic-bezier(0.34, 1.4, 0.5, 1);
    animation: spin 30s linear infinite;
  }
  .plat:hover .art m3e-shape {
    --m3e-shape-container-color: color-mix(in srgb, var(--c) 55%, transparent);
  }
  .art m3e-icon {
    position: relative;
    color: var(--c);
    --m3e-icon-size: 32px;
  }
  .soon {
    position: absolute;
    top: 22px;
    right: 22px;
    padding: 4px 11px;
    border-radius: 999px;
    font-size: 11.5px;
    font-weight: 700;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--c);
    background: color-mix(in srgb, var(--c) 14%, transparent);
    border: 1px solid color-mix(in srgb, var(--c) 40%, transparent);
  }
  .ask {
    margin-top: 28px;
  }
  .ask a {
    color: #c9c1ff;
  }
  @keyframes spin {
    to {
      rotate: 360deg;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .art m3e-shape {
      animation: none;
    }
  }
</style>
