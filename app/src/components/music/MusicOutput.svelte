<script lang="ts">
  // Settings → Music → Sound: where music plays, and exclusive mode.
  import { onMount } from "svelte";
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import { musicPlayer, type AudioDevice } from "../../lib/music/player.svelte";

  let devices = $state<AudioDevice[]>([]);
  let loading = $state(true);

  async function refresh() {
    loading = true;
    devices = await musicPlayer.devices().catch(() => []);
    loading = false;
  }
  onMount(refresh);

  const value = (e: Event) => (e.currentTarget as HTMLElement & { value: string }).value;
  const checked = (e: Event) => (e.currentTarget as HTMLElement & { checked: boolean }).checked;
  /** mpv calls the Windows default "auto"; the rest read "Speakers (Realtek…)". */
  const label = (d: AudioDevice) => (d.name === "auto" ? "Same as Windows" : d.description || d.name);
  // A device saved earlier that isn't plugged in now still shows, so the choice isn't lost.
  const options = $derived(
    devices.some((d) => d.name === musicPlayer.prefs.device) || musicPlayer.prefs.device === "auto"
      ? devices
      : [...devices, { name: musicPlayer.prefs.device, description: "Not connected" }],
  );

  function bitPerfect() {
    musicPlayer.bitPerfect();
    M3eSnackbar.open("EQ flat, normalization off, speed 1×, volume 100%. Nothing changes the sound now.");
  }
</script>

<div class="rows">
  <div class="row" data-setting="output-device">
    <div class="text">
      <strong>Output device</strong>
      <span>Only Music plays here; the rest of Windows keeps its own default.</span>
    </div>
    <div class="pick">
      <m3e-form-field variant="outlined" class="field">
        <m3e-select aria-label="Output device" disabled={loading} onchange={(e: Event) => musicPlayer.setPrefs({ device: value(e) })}>
          {#if !options.some((d) => d.name === "auto")}
            <m3e-option value="auto" selected={musicPlayer.prefs.device === "auto"}>Same as Windows</m3e-option>
          {/if}
          {#each options as d (d.name)}
            <m3e-option value={d.name} selected={musicPlayer.prefs.device === d.name}>{label(d)}</m3e-option>
          {/each}
        </m3e-select>
      </m3e-form-field>
      <m3e-icon-button aria-label="Look for devices again" onclick={refresh}><m3e-icon name="refresh"></m3e-icon></m3e-icon-button>
    </div>
  </div>
  <m3e-divider></m3e-divider>
  <label class="row" data-setting="exclusive">
    <div class="text">
      <strong>Exclusive mode</strong>
      <span>
        Music takes the device to itself and skips the Windows mixer. Other apps go silent while it plays, and crossfade
        turns off. Movies and shows always share the device.
      </span>
    </div>
    <m3e-switch checked={musicPlayer.prefs.exclusive} onchange={(e: Event) => musicPlayer.setPrefs({ exclusive: checked(e) })}></m3e-switch>
  </label>
  {#if musicPlayer.prefs.exclusive}
    <div class="row note">
      <div class="text">
        <span>For bit-perfect output, nothing may change the sound: EQ, normalization, Sound options, speed or a volume under 100%.</span>
      </div>
      <m3e-button variant="tonal" onclick={bitPerfect}><m3e-icon slot="icon" name="graphic_eq"></m3e-icon>Make it bit-perfect</m3e-button>
    </div>
  {/if}
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
  .row.note {
    padding-top: 0;
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
  .pick {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .field {
    width: 260px;
  }
</style>
