<script lang="ts">
  import AniKey from "../components/AniKey.svelte";
  import { ani } from "../lib/state.svelte";

  ani.loadKey().catch((e) => console.warn("ani key status", e));
</script>

<div class="ani">
  <header class="intro">
    <span class="on-shape mark" aria-hidden="true">
      <m3e-shape name="very-sunny"></m3e-shape>
      <m3e-icon name="wand_stars" filled></m3e-icon>
    </span>
    <h1 class="display">Ask Ani</h1>
    <p>Recommendations from what you watch, answers about any title, and links found by UniverseAPI.</p>
  </header>

  {#if ani.key && !ani.key.set}
    <m3e-card variant="filled" class="setup">
      <div slot="content" class="setup-body">
        <m3e-heading variant="title" size="medium" level={2}>Add your Gemini key</m3e-heading>
        <p>Ani runs on Google's Gemini with a key you bring. Getting one is free and takes a minute.</p>
        <AniKey />
      </div>
    </m3e-card>
  {/if}
</div>

<style>
  .ani {
    display: flex;
    flex-direction: column;
    gap: 28px;
    max-width: 860px;
    margin: 0 auto;
    padding: 40px 0;
  }
  .intro {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 14px;
  }
  .mark {
    --m3e-shape-size: 72px;
    --m3e-shape-container-color: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
    --m3e-icon-size: 34px;
  }
  .mark m3e-shape {
    animation: spin 30s linear infinite;
  }
  @keyframes spin {
    to {
      rotate: 360deg;
    }
  }
  h1 {
    margin: 0;
    font-size: 52px;
    font-weight: 400;
    line-height: 1;
  }
  p {
    margin: 0;
    line-height: 1.55;
    color: var(--md-sys-color-on-surface-variant);
  }
  .setup-body {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 8px;
  }
</style>
