<script lang="ts">
  // /open/?to=title/movie/603 (or music/<videoId>, profile/<username>).
  // Tries the app first; if the page is still here a moment later, the app
  // probably isn't installed, so the download is offered instead.
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import mark from "./assets/velmora-mark.webp";
  import { latest } from "./lib/latest.svelte";
  import { RELEASES_URL } from "./lib/release";

  const to = new URLSearchParams(location.search).get("to") ?? "";
  const valid = /^(title\/(movie|tv)\/\d{1,9}|music\/[A-Za-z0-9_-]{11}|profile\/[A-Za-z0-9_]{3,20})$/.test(to);
  const link = `velmora://${to}`;
  const what = to.startsWith("music/") ? "this song" : to.startsWith("profile/") ? "this profile" : "this title";

  let waited = $state(false);

  onMount(() => {
    latest.load();
    if (!valid) {
      waited = true;
      return;
    }
    location.href = link;
    const t = setTimeout(() => (waited = true), 1600);
    // Leaving the page (the app took over) means there's nothing to offer.
    const away = () => clearTimeout(t);
    window.addEventListener("blur", away, { once: true });
    return () => clearTimeout(t);
  });
</script>

<main class="stage">
  <section class="card" in:fade={{ duration: 220 }}>
    <img src={mark} alt="" width="64" height="64" />
    {#if !valid}
      <h1>That link doesn't lead anywhere</h1>
      <p>It may have been cut short. Try the button where you found it again.</p>
    {:else if !waited}
      <m3e-loading-indicator></m3e-loading-indicator>
      <h1>Opening Velmora…</h1>
      <p>Your browser may ask first. Choose Open.</p>
    {:else}
      <h1>Didn't open?</h1>
      <p>You need the Velmora app to open {what}. It's free for Windows.</p>
      <div class="row">
        <m3e-button variant="filled" href={latest.current?.installer?.url ?? RELEASES_URL}>
          <m3e-icon slot="icon" name="download"></m3e-icon>Get Velmora
        </m3e-button>
        <m3e-button variant="text" href={link}>Try again</m3e-button>
      </div>
    {/if}
  </section>
</main>

<style>
  .stage {
    min-height: 100vh;
    display: grid;
    place-items: center;
    padding: 32px 16px;
    background:
      radial-gradient(40% 40% at 20% 15%, color-mix(in srgb, var(--md-sys-color-primary) 28%, transparent), transparent 70%),
      var(--md-sys-color-surface, #0a0d16);
    color: var(--md-sys-color-on-surface);
  }
  .card {
    width: min(440px, 100%);
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 14px;
    padding: 36px 28px;
    border-radius: 32px;
    background: var(--md-sys-color-surface-container);
  }
  img {
    border-radius: 18px;
  }
  h1 {
    margin: 0;
    font-family: var(--display);
    font-weight: 400;
    font-size: 34px;
  }
  p {
    margin: 0;
    line-height: 1.6;
    color: var(--md-sys-color-on-surface-variant);
  }
  .row {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    justify-content: center;
  }
</style>
