<script lang="ts">
  // Settings → Playback → Player: mpv or the built-in player, which graphics
  // mode mpv settled on, and a report to send when it won't start.
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import { playerEngine, type EngineChoice } from "../lib/engine.svelte";
  import { profileLabel } from "../lib/mpv";
  import { musicPlayer } from "../lib/music/player.svelte";

  const CHOICES: [EngineChoice, string][] = [
    ["auto", "Automatic"],
    ["mpv", "mpv"],
    ["builtin", "Built-in"],
  ];

  const status = $derived.by(() => {
    if (playerEngine.choice === "builtin") return "Using the built-in player, as you chose.";
    if (playerEngine.mpvDown) {
      return playerEngine.choice === "auto"
        ? "mpv couldn't start on this PC, so Music uses the built-in player."
        : "mpv couldn't start on this PC. Pick Automatic to use the built-in player instead.";
    }
    return playerEngine.profile ? `Using mpv, in ${profileLabel(playerEngine.profile)} graphics mode.` : "Using mpv. It hasn't started yet this time.";
  });

  async function pick(c: EngineChoice) {
    if (playerEngine.choice === c) return;
    playerEngine.setChoice(c);
    await musicPlayer.switchEngine().catch(() => {});
  }

  function startOver() {
    playerEngine.resetProfile();
    M3eSnackbar.open("mpv tries the best graphics mode first next time it starts.");
  }

  async function copy() {
    try {
      await navigator.clipboard.writeText(playerEngine.details());
      M3eSnackbar.open("Copied. Paste it in a message to whoever helps you.");
    } catch {
      M3eSnackbar.open("Couldn't copy the details");
    }
  }
</script>

<div class="rows">
  <div class="row" data-setting="player-engine">
    <div class="text">
      <strong>Player</strong>
      <span>
        mpv plays nearly everything with the best picture. The built-in player is the one inside Velmora's window and runs on any
        PC; ffmpeg converts what it can't play as is, which takes some CPU. Automatic uses mpv and falls back when it can't start.
      </span>
    </div>
    <m3e-segmented-button aria-label="Player">
      {#each CHOICES as [c, name] (c)}
        <m3e-button-segment checked={playerEngine.choice === c} onclick={() => pick(c)}>{name}</m3e-button-segment>
      {/each}
    </m3e-segmented-button>
  </div>
  <m3e-divider></m3e-divider>
  <div class="row" data-setting="player-graphics">
    <div class="text">
      <strong>{status}</strong>
      <span>
        {#if playerEngine.mpvDown && playerEngine.reason}
          mpv said: {playerEngine.reason}
        {:else}
          If mpv fails to start, Velmora tries simpler graphics modes, down to software, and remembers the one that works.
        {/if}
      </span>
    </div>
    <div class="actions">
      <m3e-button variant="text" onclick={startOver} disabled={!playerEngine.profile && !playerEngine.mpvDown}>Start over</m3e-button>
      <m3e-button variant="tonal" onclick={copy}><m3e-icon slot="icon" name="content_copy"></m3e-icon>Copy details</m3e-button>
    </div>
  </div>
</div>

<style>
  .rows {
    display: flex;
    flex-direction: column;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    padding: 14px 4px;
  }
  .text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .text strong {
    font-weight: 700;
  }
  .text span {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }
</style>
