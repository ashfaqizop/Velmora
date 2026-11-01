<script lang="ts">
  import { reveal } from "../lib/reveal";
  import { screen } from "../lib/media.svelte";
  import Dots from "./Dots.svelte";

  const ITEMS = [
    ["person_off", "No account", "Install it and start. There is nothing to sign up for."],
    ["cloud_off", "No server of ours", "Searching, ranking and playing all happen on your PC."],
    ["visibility_off", "No tracking", "The app sends no analytics, and the history it keeps stays local."],
    ["code", "Open source", "Every line is on GitHub under GPL-3.0, and GitHub Actions builds every release."],
  ];

  let track = $state<HTMLElement>();
</script>

<section>
  <div class="wrap">
    <div class="panel" use:reveal>
      <div class="glow" aria-hidden="true"></div>
      <span class="eyebrow">Private by design</span>
      <h2 class="title">Your PC. <em>Your business.</em></h2>
      <div class="row" bind:this={track}>
        {#each ITEMS as [icon, title, text], i (title)}
          <div class="item" use:reveal={screen.phone ? 0 : 0.1 + i * 0.08}>
            <m3e-icon name={icon}></m3e-icon>
            <strong>{title}</strong>
            <span>{text}</span>
          </div>
        {/each}
      </div>
      {#if screen.phone}<Dots {track} labels={ITEMS.map((x) => x[1])} label="Privacy" />{/if}
    </div>
  </div>
</section>

<style>
  .panel {
    position: relative;
    overflow: hidden;
    padding: 64px 56px;
    border-radius: 40px;
    border: 1px solid var(--line);
    background: linear-gradient(180deg, #17122a, #0f0c18);
  }
  .glow {
    position: absolute;
    inset: -40% -10% auto;
    height: 120%;
    background:
      radial-gradient(40% 50% at 30% 50%, rgb(123 108 255 / 28%), transparent 70%),
      radial-gradient(35% 45% at 75% 40%, rgb(255 90 90 / 18%), transparent 70%);
    filter: blur(20px);
    pointer-events: none;
  }
  .row {
    position: relative;
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 32px;
    margin-top: 48px;
  }
  .item {
    display: flex;
    flex-direction: column;
    gap: 8px;
    --m3e-icon-size: 30px;
  }
  .item m3e-icon {
    color: #c9c1ff;
    margin-bottom: 6px;
  }
  .item strong {
    font-size: 17px;
  }
  .item span {
    font-size: 14.5px;
    line-height: 1.55;
    color: var(--muted);
  }
  @media (max-width: 960px) {
    .row {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
    .panel {
      padding: 44px 28px;
    }
  }
  /* Phones: the four points swipe sideways as cards. */
  @media (max-width: 600px) {
    .row {
      grid-template-columns: none;
      grid-auto-flow: column;
      grid-auto-columns: 78%;
      gap: 12px;
      margin: 28px -22px 0;
      padding: 0 22px 4px;
      overflow-x: auto;
      overscroll-behavior-x: contain;
      scroll-snap-type: x mandatory;
      scroll-padding-inline: 22px;
      scrollbar-width: none;
    }
    .row::-webkit-scrollbar {
      display: none;
    }
    .item {
      scroll-snap-align: start;
      padding: 22px 20px;
      border-radius: 22px;
      background: rgb(255 255 255 / 5%);
      border: 1px solid var(--line);
    }
    .panel {
      padding: 36px 22px;
      border-radius: 30px;
    }
  }
</style>
