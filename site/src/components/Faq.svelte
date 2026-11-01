<script lang="ts">
  import { reveal } from "../lib/reveal";

  const FAQ = [
    [
      "Is Velmora free?",
      "Yes. The app is free and open source under GPL-3.0, with no subscription and no ads inside it. This website shows a few ads to help pay for its hosting.",
    ],
    [
      "Why does Windows warn me about the installer?",
      "Velmora isn't code-signed yet, because signing certificates are expensive. Windows SmartScreen warns about any unsigned app it hasn't seen often. The source is public and GitHub Actions builds every installer from it. Click More info, then Run anyway.",
    ],
    [
      "Does Velmora host any movies or shows?",
      "No. Velmora doesn't host, upload or store any content. It searches third-party sites the way a browser does and plays what they serve. You're responsible for what you watch and for following the law where you live.",
    ],
    ["Do I need an account?", "No. You can connect AniList or Trakt if you want your progress synced there, but nothing needs signing up for."],
    [
      "What is Ani, and does it cost anything?",
      "Ani is an optional assistant inside Velmora. It uses Google Gemini with your own API key, which you can get free from Google AI Studio. Your key and the taste profile Ani builds stay on your PC.",
    ],
    ["Is there a Mac, Linux or Android version?", "Not yet. Velmora is Windows only for now, and Android is planned."],
    ["How do updates work?", "Velmora checks for new versions when it starts and installs them itself. Every update is signed, and the app checks the signature before installing."],
    ["Something doesn't work. Where do I report it?", "Open an issue on GitHub with what you tried and what happened. Screenshots help a lot."],
  ];
</script>

<section id="faq">
  <div class="wrap narrow">
    <div use:reveal>
      <span class="eyebrow">FAQ</span>
      <h2 class="title">Good <em>questions.</em></h2>
    </div>
    <m3e-accordion class="list" use:reveal={0.1}>
      {#each FAQ as [q, a] (q)}
        <m3e-expansion-panel>
          <span slot="header" class="q">{q}</span>
          <p>{a}</p>
        </m3e-expansion-panel>
      {/each}
    </m3e-accordion>
  </div>
</section>

<style>
  .narrow {
    max-width: 860px;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-top: 48px;
    --m3e-expansion-panel-container-color: rgb(255 255 255 / 4%);
    --m3e-expansion-panel-shape: 22px;
    --m3e-expansion-header-collapsed-height: auto;
    --m3e-expansion-header-expanded-height: auto;
  }
  .q {
    display: block;
    padding: 16px 0;
    font-size: 17px;
    font-weight: 600;
    line-height: 1.35;
    /* m3e cuts headers off with an ellipsis; long questions should wrap. */
    white-space: normal;
  }
  p {
    margin: 0;
    font-size: 15.5px;
    line-height: 1.65;
    color: var(--muted);
  }
</style>
