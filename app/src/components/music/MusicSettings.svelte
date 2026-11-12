<script lang="ts">
  // Settings → Music: the YouTube Music account, the equalizer, and the
  // yt-dlp copy that plays the songs.
  import { onMount } from "svelte";
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import * as api from "../../lib/api";
  import { EQ_PRESETS, musicPlayer } from "../../lib/music/player.svelte";
  import { musicSession } from "../../lib/music/session.svelte";
  import { musicHome } from "../../lib/music/feeds.svelte";
  import type { YtDlpStatus } from "../../lib/types";

  let ytdlp = $state<YtDlpStatus | null>(null);
  let updating = $state(false);
  let confirmOut = $state(false);

  onMount(async () => {
    ytdlp = await api.musicYtdlpStatus().catch(() => null);
  });

  async function update() {
    updating = true;
    try {
      ytdlp = await api.musicYtdlpUpdate();
      M3eSnackbar.open(`yt-dlp is up to date${ytdlp.version ? ` (${ytdlp.version})` : ""}.`);
    } catch (e) {
      M3eSnackbar.open(`Couldn't update yt-dlp: ${e}`);
    } finally {
      updating = false;
    }
  }

  async function signOut() {
    if (!confirmOut) {
      confirmOut = true;
      setTimeout(() => (confirmOut = false), 4000);
      return;
    }
    await musicSession.signOut();
    await musicPlayer.close();
    musicHome.reset();
    await api.musicStorageClear("cache").catch(() => {});
    confirmOut = false;
    M3eSnackbar.open("Signed out of YouTube Music on this PC.");
  }
</script>

<div class="rows">
  <div class="row" data-setting="music-account">
    {#if musicSession.signedIn}
      <m3e-avatar class="face">
        {#if musicSession.account?.photo}<img src={musicSession.account.photo} alt="" />{:else}<m3e-icon name="person"></m3e-icon>{/if}
      </m3e-avatar>
      <div class="text">
        <strong>{musicSession.account?.name ?? "Signed in"}</strong>
        <span>{musicSession.account?.handle ?? "YouTube Music"} · Only Music uses this account.</span>
      </div>
      <m3e-button variant="text" onclick={signOut}>{confirmOut ? "Click again to sign out" : "Sign out"}</m3e-button>
    {:else}
      <div class="text">
        <strong>YouTube Music</strong>
        <span>
          {musicSession.expired
            ? "Google ended the session. Sign in again to keep using Music."
            : "Sign in with Google to use Music. You sign in on Google's own page."}
        </span>
      </div>
      <m3e-button variant="tonal" disabled={musicSession.signingIn} onclick={() => musicSession.signIn()}>
        <m3e-icon slot="icon" name="login"></m3e-icon>Sign in
      </m3e-button>
    {/if}
  </div>
  <m3e-divider></m3e-divider>
  <div class="row" data-setting="eq">
    <div class="text">
      <strong>Equalizer</strong>
      <span>Shapes the sound of music. Night mode and even volume above apply too.</span>
    </div>
    <m3e-form-field variant="outlined" class="eq">
      <m3e-select aria-label="Equalizer" onchange={(e: Event) => musicPlayer.setEq((e.currentTarget as HTMLElement & { value: string }).value)}>
        {#each Object.entries(EQ_PRESETS) as [id, p] (id)}
          <m3e-option value={id} selected={musicPlayer.eq === id}>{p.label}</m3e-option>
        {/each}
      </m3e-select>
    </m3e-form-field>
  </div>
  <m3e-divider></m3e-divider>
  <div class="row" data-setting="ytdlp">
    <div class="text">
      <strong>yt-dlp</strong>
      <span>
        {#if ytdlp?.installed}Version {ytdlp.version ?? "unknown"}. It updates itself every three days; update now if songs stop playing.
        {:else}Downloads on the first song you play (about 18 MB).{/if}
      </span>
    </div>
    <m3e-button variant="tonal" disabled={updating} onclick={update}>
      <m3e-icon slot="icon" name="update"></m3e-icon>{updating ? "Updating…" : ytdlp?.installed ? "Update now" : "Install now"}
    </m3e-button>
  </div>
</div>

<style>
  .rows {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .face {
    flex: none;
  }
  .face img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: 50%;
  }
  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .text strong {
    font-weight: 700;
  }
  .text span {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .eq {
    width: 200px;
  }
</style>
