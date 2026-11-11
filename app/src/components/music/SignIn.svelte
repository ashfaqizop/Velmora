<script lang="ts">
  // Music needs a Google sign-in; nothing else in Velmora does. This shows a
  // faded preview of the charts behind the sign-in card.
  import { onMount } from "svelte";
  import { fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import * as api from "../../lib/api";
  import Art from "./Art.svelte";
  import { musicSession } from "../../lib/music/session.svelte";
  import type { MusicItem } from "../../lib/types";

  let preview = $state<MusicItem[]>([]);

  onMount(async () => {
    try {
      const charts = await api.musicFeed("FEmusic_charts");
      preview = charts.shelves.flatMap((s) => s.items).filter((i) => i.thumbnails.length).slice(0, 24);
    } catch {
      // The card works without the preview.
    }
  });
</script>

<div class="gate">
  <div class="wall" aria-hidden="true">
    {#each preview as p, i (p.id + i)}
      <div class="tile" style:--d={`${(i % 6) * 90}ms`}>
        <Art thumbs={p.thumbnails} size={150} round={p.kind === "artist"} />
      </div>
    {/each}
  </div>

  <div class="card" in:fly={{ y: 24, duration: 520, easing: cubicOut }}>
    <span class="badge on-shape" aria-hidden="true">
      <m3e-shape name="soft-burst"></m3e-shape>
      <m3e-icon name="music_note" filled></m3e-icon>
    </span>
    {#if musicSession.expired}
      <h1 class="display">Sign in again</h1>
      <p>Google ended your YouTube Music session. Sign in once more and everything picks up where it was.</p>
    {:else}
      <h1 class="display">Your music, in Velmora</h1>
      <p>Your mixes, playlists, likes and library from YouTube Music, with synced lyrics and the music videos.</p>
    {/if}
    <m3e-button variant="filled" size="large" disabled={musicSession.signingIn} onclick={() => musicSession.signIn()}>
      <m3e-icon slot="icon" name="login"></m3e-icon>
      {musicSession.signingIn ? "Finish signing in in the window…" : "Sign in with Google"}
    </m3e-button>
    <ul>
      <li><m3e-icon name="lock"></m3e-icon>You sign in on Google's own page. Your password never reaches Velmora.</li>
      <li><m3e-icon name="info"></m3e-icon>Only Music needs it. Movies, series, anime and sports stay as they are.</li>
      <li><m3e-icon name="logout"></m3e-icon>Sign out any time in Settings, under Music.</li>
    </ul>
  </div>
</div>

<style>
  .gate {
    position: relative;
    min-height: calc(100vh - 140px);
    display: grid;
    place-items: center;
    overflow: hidden;
    border-radius: 32px;
  }
  .wall {
    position: absolute;
    inset: -40px;
    display: grid;
    grid-template-columns: repeat(auto-fill, 150px);
    gap: 16px;
    justify-content: center;
    opacity: 0.22;
    filter: blur(1.5px) saturate(1.1);
    transform: rotate(-6deg) scale(1.05);
    mask-image: radial-gradient(60% 60% at 50% 50%, transparent 0%, black 85%);
  }
  .tile {
    animation: rise 900ms cubic-bezier(0.2, 0, 0, 1) both;
    animation-delay: var(--d);
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(20px);
    }
  }
  .card {
    position: relative;
    max-width: 520px;
    padding: 40px 40px 32px;
    border-radius: 36px;
    background: color-mix(in srgb, var(--md-sys-color-surface-container) 92%, transparent);
    backdrop-filter: blur(18px);
    box-shadow: 0 30px 80px rgb(0 0 0 / 45%);
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 14px;
  }
  .badge {
    --m3e-shape-size: 64px;
    --m3e-shape-container-color: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
    --m3e-icon-size: 30px;
  }
  h1 {
    margin: 6px 0 0;
    font-size: 40px;
    font-weight: 400;
    line-height: 1.05;
  }
  p {
    margin: 0;
    color: var(--md-sys-color-on-surface-variant);
    line-height: 1.55;
  }
  ul {
    list-style: none;
    margin: 8px 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  li {
    display: flex;
    gap: 10px;
    align-items: flex-start;
    --m3e-icon-size: 18px;
  }
</style>
