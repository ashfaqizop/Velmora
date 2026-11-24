<script lang="ts">
  // The Theatre-wide switches: pause, the banner, and the reminder wording.
  import { onMount } from "svelte";
  import { call, say, sayError } from "../api";
  import type { Settings } from "./types";

  let loaded = $state(false);
  let paused = $state(false);
  let banner = $state("");
  let reminder = $state("");
  let saved = $state<Settings>({ paused: false, banner: "", reminder: "" });
  let busy = $state(false);

  async function load() {
    try {
      const s = ((await call<{ settings: Settings }>("theatreSettings")) as unknown as { settings: Settings }).settings;
      saved = s;
      paused = s.paused;
      banner = s.banner;
      reminder = s.reminder;
    } catch (e) {
      sayError(e);
    } finally {
      loaded = true;
    }
  }
  onMount(load);

  async function save(body: Partial<Settings>, done: string) {
    busy = true;
    try {
      await call("theatreSettingsSet", body);
      say(done);
      await load();
    } catch (e) {
      sayError(e);
      await load();
    } finally {
      busy = false;
    }
  }
</script>

{#if !loaded}
  <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
{:else}
  <div class="grid cols-2">
    <section class="card" class:paused={saved.paused}>
      <h2><m3e-icon name="pause_circle"></m3e-icon>Pause Theatre</h2>
      <div class="sw">
        <m3e-switch checked={paused} disabled={busy} onchange={(e: Event) => save({ paused: (e.currentTarget as HTMLInputElement).checked }, (e.currentTarget as HTMLInputElement).checked ? "Theatre is paused." : "Theatre is back on.")} aria-label="Pause Theatre"></m3e-switch>
        <span>{saved.paused ? "Paused" : "Running"}</span>
      </div>
      <p class="muted">
        While paused, nobody can book or join a waitlist, and the daily job drafts no new screenings. People already in a lobby or a show stay in. Use it for an outage or an emergency, and say why in the banner.
      </p>
    </section>

    <section class="card">
      <h2><m3e-icon name="campaign"></m3e-icon>Banner</h2>
      <label class="field"><input bind:value={banner} maxlength="200" placeholder="Shown at the top of Theatre in the app and on Social" aria-label="Banner text" /></label>
      <div class="row-actions">
        <m3e-button variant="filled" disabled={busy || banner.trim() === saved.banner} onclick={() => save({ banner }, banner.trim() ? "Banner is live." : "Banner cleared.")}>Save</m3e-button>
        {#if saved.banner}<m3e-button variant="text" disabled={busy} onclick={() => { banner = ""; void save({ banner: "" }, "Banner cleared."); }}>Clear</m3e-button>{/if}
      </div>
      <p class="muted">Leave it empty for no banner. To warn about one screening, put a note on that screening instead.</p>
    </section>

    <section class="card wide">
      <h2><m3e-icon name="chat_bubble"></m3e-icon>Lobby reminder</h2>
      <label class="field"><input bind:value={reminder} maxlength="160" placeholder="{'{title}'} starts in 30 minutes. Chat is open." aria-label="Reminder text" /></label>
      <div class="row-actions">
        <m3e-button variant="filled" disabled={busy || reminder.trim() === saved.reminder} onclick={() => save({ reminder }, "Reminder saved.")}>Save</m3e-button>
        {#if saved.reminder}<m3e-button variant="text" disabled={busy} onclick={() => { reminder = ""; void save({ reminder: "" }, "Back to the default."); }}>Use the default</m3e-button>{/if}
      </div>
      <p class="muted">What the app says 30 minutes before a screening you booked. Put <span class="mono">{"{title}"}</span> where the film's name goes. Empty uses the default.</p>
    </section>
  </div>
{/if}

<style>
  .grid {
    align-items: start;
  }
  .wide {
    grid-column: 1 / -1;
  }
  .paused {
    border-color: color-mix(in srgb, var(--amber) 60%, transparent);
  }
  .sw {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 12px;
    font-weight: 700;
  }
  p {
    margin: 12px 0 0;
    font-size: 13.5px;
    line-height: 1.5;
  }
  .card .field {
    margin-bottom: 12px;
  }
</style>
