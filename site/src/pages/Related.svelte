<script lang="ts">
  // "Keep going": cards linking to the neighbouring pages.
  import { inview } from "./inview";

  let { items }: { items: { href: string; icon: string; title: string; text: string }[] } = $props();
</script>

<section class="pg-section">
  <div class="pg-wide">
    <span class="eyebrow">Keep going</span>
    <div class="pg-grid">
      {#each items as it, i (it.href)}
        <a class="pg-card" href={it.href} use:inview={i * 0.06}>
          <m3e-icon name={it.icon}></m3e-icon>
          <h3>{it.title}</h3>
          <p>{it.text}</p>
        </a>
      {/each}
    </div>
  </div>
</section>

<style>
  m3e-icon {
    --m3e-icon-size: 28px;
    color: var(--blue);
  }
</style>
