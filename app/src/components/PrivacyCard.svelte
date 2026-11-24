<script lang="ts">
  // Settings > Privacy: sharing what you watch on Velmora Social, and the
  // anonymous usage statistics (and exactly what they are).
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import { analytics } from "../lib/analytics.svelte";
  import { account } from "../lib/account.svelte";
  import { activity } from "../lib/activity.svelte";

  let sharingBusy = $state(false);

  async function setSharing(on: boolean) {
    sharingBusy = true;
    try {
      await account.update({ share_activity: on });
      if (!on) activity.clear();
    } catch (e) {
      M3eSnackbar.open((e as Error).message);
    } finally {
      sharingBusy = false;
    }
  }

  let preview = $state<string | null>(null);

  const checked = (e: Event) => (e.currentTarget as HTMLElement & { checked: boolean }).checked;

  function toggle(on: boolean) {
    analytics.setEnabled(on);
    if (!on) preview = null;
  }
</script>

<div class="rows">
  {#if account.signedIn}
    <label class="row" data-setting="share-activity">
      <div class="text">
        <strong>Share what I watch on Velmora Social</strong>
        <span>
          What you start, finish, add to My List and rate shows on your profile, in your followers' feeds and in your stats. 18+
          titles are never shared. Turning this off hides what was shared and stops new posts.
        </span>
      </div>
      <m3e-switch
        checked={account.profile?.share_activity !== false}
        disabled={sharingBusy || account.offline}
        aria-label="Share what I watch on Velmora Social"
        onchange={(e: Event) => setSharing(checked(e))}
      ></m3e-switch>
    </label>

    <m3e-divider></m3e-divider>
  {/if}

  <label class="row" data-setting="analytics">
    <div class="text">
      <strong>Share anonymous usage statistics</strong>
      <span>
        Once a day Velmora sends plain totals: how many people watched each title, which sites and which player were used, and the
        quality. It's what makes the "Popular on Velmora" row and the stats page possible. On by default; turn it off here any time.
      </span>
    </div>
    <m3e-switch checked={analytics.enabled} aria-label="Share anonymous usage statistics" onchange={(e: Event) => toggle(checked(e))}></m3e-switch>
  </label>

  <m3e-divider></m3e-divider>

  <div class="facts" data-setting="analytics-details">
    <div>
      <strong>What is sent</strong>
      <ul>
        <li>The titles watched for at least a minute, by their TMDB number, counted once per day.</li>
        <li>The site each play came from, which player ran (mpv or the built-in one), and the video quality.</li>
        <li>The songs you listened to in Music, by their YouTube number, counted once per day.</li>
        <li>The Velmora version and the day.</li>
      </ul>
    </div>
    <div>
      <strong>What is never sent</strong>
      <ul>
        <li>Your name, email, username or account. This is separate from signing in.</li>
        <li>A device or install ID, your searches, your list, or your history.</li>
        <li>Your IP address is seen by our server like any website's, but isn't stored with the counts.</li>
      </ul>
    </div>
  </div>

  <div class="peek">
    {#if preview === null}
      <m3e-button variant="tonal" disabled={!analytics.enabled} onclick={() => (preview = analytics.preview())}>
        <m3e-icon slot="icon" name="visibility"></m3e-icon>Show what would be sent
      </m3e-button>
    {:else}
      <pre>{preview}</pre>
      <m3e-button variant="text" onclick={() => (preview = null)}>Hide</m3e-button>
    {/if}
  </div>
</div>

<style>
  .rows {
    display: flex;
    flex-direction: column;
    gap: 16px;
    min-width: 0;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 16px;
  }
  .text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }
  .text span {
    font-size: 13px;
    line-height: 1.5;
    color: var(--md-sys-color-on-surface-variant);
  }
  .facts {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: 20px;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .facts strong {
    color: var(--md-sys-color-on-surface);
    font-size: 14px;
  }
  ul {
    margin: 8px 0 0;
    padding-left: 18px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    line-height: 1.5;
  }
  .peek {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }
  pre {
    margin: 0;
    width: 100%;
    max-height: 280px;
    overflow: auto;
    padding: 12px 14px;
    border-radius: 14px;
    font-size: 12px;
    background: var(--md-sys-color-surface-container);
    color: var(--md-sys-color-on-surface);
  }
</style>
