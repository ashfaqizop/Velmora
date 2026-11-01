<script lang="ts">
  import { reveal } from "../lib/reveal";
  import DownloadButton from "./DownloadButton.svelte";
  import DownloadCount from "./DownloadCount.svelte";
  import Dots from "./Dots.svelte";
  import { screen } from "../lib/media.svelte";

  const STEPS = [
    ["Download the installer", "One small file, built by GitHub Actions from the public source."],
    ["Get past SmartScreen", "Velmora isn't code-signed yet, so Windows may warn you. Click More info, then Run anyway."],
    ["Open it once", "The first launch fetches the video player and download tools, about 80 MB, and checks each file's SHA-256."],
    ["Updates take care of themselves", "When a new version is out, Velmora installs it the next time you open the app."],
  ];

  let track = $state<HTMLElement>();
</script>

<section id="install">
  <div class="wrap">
    <div use:reveal>
      <span class="eyebrow">Install</span>
      <h2 class="title">Up and running <em>in a minute.</em></h2>
    </div>
    <ol bind:this={track}>
      {#each STEPS as [title, text], i (title)}
        <li use:reveal={screen.phone ? 0 : i * 0.08}>
          <span class="n display">{i + 1}</span>
          <strong>{title}</strong>
          <span class="text">{text}</span>
        </li>
      {/each}
    </ol>
    {#if screen.phone}<Dots {track} labels={STEPS.map((x) => x[0])} label="Install steps" />{/if}
    <div class="cta" use:reveal><DownloadButton /><DownloadCount /></div>
  </div>
</section>

<style>
  ol {
    list-style: none;
    margin: 56px 0 0;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 16px;
  }
  li {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 28px 24px;
    border-radius: 28px;
    border: 1px solid var(--line);
    background: var(--glass);
  }
  .n {
    font-size: 64px;
    line-height: 0.9;
    color: transparent;
    -webkit-text-stroke: 1.5px color-mix(in srgb, var(--indigo) 80%, transparent);
    margin-bottom: 12px;
  }
  li strong {
    font-size: 17px;
  }
  .text {
    font-size: 14.5px;
    line-height: 1.55;
    color: var(--muted);
  }
  .cta {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-start;
    gap: 20px;
    margin-top: 40px;
  }
  @media (max-width: 960px) {
    ol {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }
  /* Phones: the steps swipe sideways as cards. */
  @media (max-width: 600px) {
    ol {
      grid-template-columns: none;
      grid-auto-flow: column;
      grid-auto-columns: 80%;
      gap: 12px;
      margin: 36px -16px 0;
      padding: 0 16px 4px;
      overflow-x: auto;
      overscroll-behavior-x: contain;
      scroll-snap-type: x mandatory;
      scroll-padding-inline: 16px;
      scrollbar-width: none;
    }
    ol::-webkit-scrollbar {
      display: none;
    }
    li {
      scroll-snap-align: start;
      padding: 24px 22px;
    }
    .cta {
      justify-content: center;
    }
  }
</style>
