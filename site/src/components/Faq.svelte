<script lang="ts">
  import { reveal } from "../lib/reveal";

  const FAQ = [
    [
      "Is Velmora free?",
      "Yes. The full app is free and open source under GPL-3.0, with no subscription. Velmora Public shows a few quiet ads, never while a video plays, and they help pay for its development. This website shows ads too.",
    ],
    [
      "What is Velmora Max?",
      "The same app with no ads at all. Max keys are given out by the developer, not sold. Each key works on one PC at a time: enter it in Settings, under License, and remove it there to move it to another PC. If you sign in to a Velmora account, the key is linked to it, and signing in on a new PC brings Max along (the old PC drops back to Public).",
    ],
    [
      "Why does Windows warn me about the installer?",
      "Velmora isn't code-signed yet, because signing certificates are expensive. Windows SmartScreen warns about any unsigned app it hasn't seen often. The source is public and GitHub Actions builds every installer from it. Click More info, then Run anyway.",
    ],
    [
      "Does Velmora host any movies or shows?",
      "No. Velmora doesn't host, upload or store any content. It searches third-party sites the way a browser does and plays what they serve. You're responsible for what you watch and for following the law where you live.",
    ],
    [
      "Do I need an account?",
      "Yes, from Velmora 0.6.0: a free account, made once in your browser with Google, Discord or an emailed code. It gives you a username and profile, keeps your list, progress and settings in step across PCs, and brings Max with you to a new one. Only that first sign-in needs the internet; after it, Velmora opens offline too. AniList and Trakt can still be connected separately.",
    ],
    [
      "What does Velmora collect?",
      "Your account stores your email, username, display name and picture, bio and links, and your list, progress and settings so they sync. Separately, the app sends once a day anonymous totals (which sites and players get used, which titles get watched) with no ID and no account attached; one switch in Settings, under Privacy, turns that off. The privacy policy has the details, and the public numbers are on the stats page.",
    ],
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
