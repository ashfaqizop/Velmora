<script lang="ts">
  // The rules the app's ads follow (served as /ads.json). Max never sees ads,
  // whatever is set here. Changes reach apps within about five minutes.
  import { onMount } from "svelte";
  import { call, say, sayError } from "../api";
  import { ago, count } from "../format";

  interface Slot {
    enabled: boolean;
    unit: string;
    max: number;
  }
  interface Rules {
    v: number;
    enabled: boolean;
    minLaunch: number;
    graceSeconds: number;
    refreshSeconds: number;
    maxPerSession: number;
    pauseDelaySeconds: number;
    slots: Record<string, Slot>;
  }
  interface Data {
    rules: Rules;
    custom: boolean;
    defaults: Rules;
    saved_at: number | null;
    views: { days: { day: string; views: number }[]; slots: Record<string, number> };
  }

  const UNITS = ["native", "728x90", "468x60", "320x50", "300x250", "160x300", "160x600"];
  const SLOT_NAMES: Record<string, string> = {
    home: "Home",
    search: "Search",
    grid: "Browse grids",
    title: "Title page",
    downloads: "Downloads",
    sidebar: "Sidebar",
    sports: "Sports",
    live: "Live TV",
    pause: "Pause card (player)",
    end: "End card (player)",
    music: "Music",
    nowplaying: "Now playing",
    lobby: "Theatre lobby",
  };
  const NUMBERS: { key: keyof Rules; label: string; hint: string; min: number; max: number }[] = [
    { key: "minLaunch", label: "First launch with ads", hint: "2 means none on the very first launch", min: 1, max: 20 },
    { key: "graceSeconds", label: "Quiet time after launch (s)", hint: "No ads for this long after the app opens", min: 0, max: 3600 },
    { key: "refreshSeconds", label: "Refresh at most every (s)", hint: "60 or more", min: 60, max: 3600 },
    { key: "maxPerSession", label: "Ads per session, all slots", hint: "0 turns them all off", min: 0, max: 500 },
    { key: "pauseDelaySeconds", label: "Pause card after (s)", hint: "How long the player stays paused first", min: 1, max: 120 },
  ];

  let data = $state<Data | null>(null);
  let rules = $state<Rules | null>(null);
  let busy = $state(false);

  async function load() {
    try {
      data = await call<Data>("ads");
      rules = structuredClone(data.rules);
    } catch (e) {
      sayError(e);
    }
  }
  onMount(load);

  const dirty = $derived(Boolean(data && rules && JSON.stringify(rules) !== JSON.stringify(data.rules)));
  const peak = $derived(Math.max(1, ...(data?.views.days.map((d) => d.views) ?? [1])));
  const total = $derived(data?.views.days.reduce((a, d) => a + d.views, 0) ?? 0);

  async function save(reset = false) {
    if (reset && !confirm("Go back to the built-in defaults?")) return;
    busy = true;
    try {
      await call("adsSave", reset ? { reset: true } : { rules: $state.snapshot(rules) });
      say(reset ? "Back to the defaults." : "Saved. Apps pick it up within about five minutes.");
      await load();
    } catch (e) {
      sayError(e);
    } finally {
      busy = false;
    }
  }

  const checked = (e: Event) => (e.currentTarget as HTMLInputElement).checked;
  const day = (s: string) => new Date(`${s}T12:00:00Z`).toLocaleDateString(undefined, { weekday: "short", day: "numeric" });
</script>

<div class="adm-page">
  <div class="adm-head">
    <div class="grow">
      <h1>App ads</h1>
      <p>On or off, how often, and which unit each slot uses. Only the Public tier sees ads; Max never does. The website's own ads aren't affected.</p>
    </div>
    {#if data}
      <div class="row-actions">
        {#if data.custom}<m3e-button variant="text" disabled={busy} onclick={() => save(true)}><m3e-icon slot="icon" name="restart_alt"></m3e-icon>Use defaults</m3e-button>{/if}
        <m3e-button variant="text" disabled={busy || !dirty} onclick={() => (rules = structuredClone($state.snapshot(data!.rules)))}>Undo edits</m3e-button>
        <m3e-button variant="filled" disabled={busy || !dirty} onclick={() => save()}><m3e-icon slot="icon" name="save"></m3e-icon>Save</m3e-button>
      </div>
    {/if}
  </div>

  {#if !data || !rules}
    <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
  {:else}
    <div class="grid cols-2 top">
      <section class="card master" class:off={!rules.enabled}>
        <h2><m3e-icon name="ads_click"></m3e-icon>All app ads</h2>
        <label class="sw">
          <m3e-switch checked={rules.enabled} onchange={(e: Event) => (rules!.enabled = checked(e))} aria-label="All app ads"></m3e-switch>
          <span>{rules.enabled ? "On" : "Off everywhere"}</span>
        </label>
        <p class="muted small">
          {data.custom ? `Your rules, saved ${ago(data.saved_at ? new Date(data.saved_at * 1000).toISOString() : null)}.` : "Using the built-in defaults."}
          Switching off here hides every slot in every app on its next check.
        </p>
      </section>

      <section class="card">
        <h2><m3e-icon name="insights"></m3e-icon>Ads shown, last 14 days</h2>
        <div class="bars" aria-label="Ads shown per day">
          {#each data.views.days as d (d.day)}
            <div class="bar" title="{day(d.day)}: {count(d.views)}">
              <span style="height: {Math.max(2, (d.views / peak) * 100)}%"></span>
            </div>
          {/each}
        </div>
        <p class="muted small">{count(total)} in total. Counted anonymously by the app, per slot and day.</p>
      </section>
    </div>

    <section class="card">
      <h2><m3e-icon name="schedule"></m3e-icon>Timing and caps</h2>
      <div class="nums">
        {#each NUMBERS as n (n.key)}
          <label class="lab">
            <span>{n.label}</span>
            <span class="field"><input type="number" min={n.min} max={n.max} step="1" bind:value={rules[n.key] as number} /></span>
            <span class="muted hint">{n.hint}{rules[n.key] !== data.defaults[n.key] ? ` · default ${data.defaults[n.key]}` : ""}</span>
          </label>
        {/each}
      </div>
    </section>

    <section class="card slots" class:dim={!rules.enabled}>
      <h2><m3e-icon name="view_carousel"></m3e-icon>Slots</h2>
      <div class="slot head muted">
        <span>Slot</span><span>Unit</span><span>Per session</span><span>Shown (14 d)</span><span></span>
      </div>
      {#each Object.keys(rules.slots) as id (id)}
        {@const s = rules.slots[id]}
        <div class="slot" class:off={!s.enabled}>
          <strong>{SLOT_NAMES[id] ?? id}</strong>
          <span class="field sel">
            <select bind:value={s.unit} aria-label="{SLOT_NAMES[id] ?? id} unit">
              {#each UNITS as u (u)}<option value={u}>{u === "native" ? "Native 4:1" : u}</option>{/each}
            </select>
          </span>
          <span class="field small-field"><input type="number" min="0" max="100" step="1" bind:value={s.max} aria-label="{SLOT_NAMES[id] ?? id} cap" /></span>
          <span class="muted">{count(data.views.slots[id] ?? 0)}</span>
          <m3e-switch checked={s.enabled} onchange={(e: Event) => (s.enabled = checked(e))} aria-label="{SLOT_NAMES[id] ?? id} on"></m3e-switch>
        </div>
      {/each}
    </section>
  {/if}
</div>

<style>
  .small {
    font-size: 13px;
    line-height: 1.5;
    margin: 12px 0 0;
  }
  .top {
    align-items: stretch;
  }
  .master.off {
    border-color: color-mix(in srgb, var(--amber) 60%, transparent);
  }
  .sw {
    display: flex;
    align-items: center;
    gap: 12px;
    font-weight: 700;
    font-size: 18px;
  }
  .bars {
    display: grid;
    grid-template-columns: repeat(14, 1fr);
    align-items: end;
    gap: 6px;
    height: 96px;
  }
  .bar {
    height: 100%;
    display: flex;
    align-items: flex-end;
  }
  .bar span {
    display: block;
    width: 100%;
    border-radius: 6px 6px 3px 3px;
    background: linear-gradient(to top, color-mix(in srgb, var(--md-sys-color-primary) 55%, transparent), var(--md-sys-color-primary));
    transition: height 400ms cubic-bezier(0.2, 0, 0, 1);
  }
  .nums {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(190px, 1fr));
    gap: 16px;
  }
  .hint {
    font-weight: 400;
    font-size: 12px;
  }
  .slots {
    padding-bottom: 8px;
  }
  .slots.dim .slot:not(.head) {
    opacity: 0.55;
  }
  .slot {
    display: grid;
    grid-template-columns: minmax(0, 1.3fr) minmax(0, 1fr) 110px 110px 56px;
    align-items: center;
    gap: 12px;
    padding: 8px 4px;
    border-top: 1px solid color-mix(in srgb, var(--md-sys-color-outline-variant) 30%, transparent);
  }
  .slot.head {
    border-top: 0;
    font-size: 12px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .slot.off strong {
    opacity: 0.55;
  }
  .slot .field {
    height: 38px;
  }
  .sel select {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: 0;
    background: none;
    color: var(--md-sys-color-on-surface);
    font: inherit;
    font-size: 14px;
  }
  .sel option {
    background: var(--md-sys-color-surface-container-high);
  }
  @media (max-width: 760px) {
    .slot {
      grid-template-columns: minmax(0, 1fr) 96px 56px;
    }
    .slot > :nth-child(2),
    .slot > :nth-child(4) {
      display: none;
    }
  }
</style>
