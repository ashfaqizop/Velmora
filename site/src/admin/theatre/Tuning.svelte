<script lang="ts">
  // The algorithm's settings. The next draft (the daily job, or "Run now" on
  // the Lineup tab) uses them; screenings already on the lineup stay as they are.
  import { onMount } from "svelte";
  import { call, say, sayError } from "../api";
  import { ago } from "../format";

  interface Knobs {
    minPerDay: number;
    maxPerDay: number;
    perInstalls: number;
    budget: number;
    maxOverlap: number;
    repeatDays: number;
    pickRepeatDays: number;
    pick: boolean;
    gemMinRating: number;
    gemMinVotes: number;
    playsWeight: number;
    trendWeight: number;
    minRuntime: number;
    excludeGenres: string[];
    maxPerGenre: number;
    probeMinChecks: number;
    probeMinFound: number;
  }
  type NumKey = Exclude<keyof Knobs, "pick" | "excludeGenres">;

  const GROUPS: { title: string; icon: string; fields: { key: NumKey; label: string; hint: string; step?: number; pct?: boolean }[] }[] = [
    {
      title: "How many",
      icon: "format_list_numbered",
      fields: [
        { key: "minPerDay", label: "At least, per day", hint: "Even on a quiet day" },
        { key: "maxPerDay", label: "At most, per day", hint: "However busy it gets" },
        { key: "perInstalls", label: "One more per … installs", hint: "Daily installs that earn another screening" },
        { key: "maxOverlap", label: "Running at once", hint: "Screenings allowed to overlap" },
        { key: "budget", label: "Seat budget", hint: "People connected at once across overlapping shows" },
      ],
    },
    {
      title: "What gets picked",
      icon: "movie",
      fields: [
        { key: "playsWeight", label: "Weight of Velmora plays", hint: "What people here watch", step: 0.5 },
        { key: "trendWeight", label: "Weight of TMDB trending", hint: "What the world watches", step: 0.5 },
        { key: "minRuntime", label: "Shortest film (min)", hint: "Skip anything shorter" },
        { key: "repeatDays", label: "Days before a repeat", hint: "Same title, any slot" },
        { key: "maxPerGenre", label: "Most per main genre", hint: "0 means no cap; pins ignore it" },
      ],
    },
    {
      title: "Tonight's pick",
      icon: "auto_awesome",
      fields: [
        { key: "gemMinRating", label: "Lowest TMDB rating", hint: "Out of 10", step: 0.1 },
        { key: "gemMinVotes", label: "Fewest TMDB votes", hint: "So the rating means something" },
        { key: "pickRepeatDays", label: "Days before a pick repeats", hint: "Same hidden gem" },
      ],
    },
    {
      title: "Source checks",
      icon: "fact_check",
      fields: [
        { key: "probeMinChecks", label: "Checks before deciding", hint: "Reports from apps per draft" },
        { key: "probeMinFound", label: "Drop below this found rate", hint: "Share of checks that found a source", step: 5, pct: true },
      ],
    },
  ];

  let knobs = $state<Knobs | null>(null);
  let saved = $state<Knobs | null>(null);
  let defaults = $state<Knobs | null>(null);
  let genres = $state<string[]>([]);
  let custom = $state(false);
  let savedAt = $state<number | null>(null);
  let busy = $state(false);

  async function load() {
    try {
      const r = await call<{ knobs: Knobs; defaults: Knobs; genres: string[]; custom: boolean; saved_at: number | null }>("theatreKnobs");
      knobs = structuredClone(r.knobs);
      saved = r.knobs;
      defaults = r.defaults;
      genres = r.genres;
      custom = r.custom;
      savedAt = r.saved_at;
    } catch (e) {
      sayError(e);
    }
  }
  onMount(load);

  const dirty = $derived(Boolean(knobs && saved && JSON.stringify(knobs) !== JSON.stringify(saved)));

  async function save(reset = false) {
    if (reset && !confirm("Back to the built-in settings?")) return;
    busy = true;
    try {
      await call("theatreKnobsSet", reset ? { reset: true } : { knobs: $state.snapshot(knobs) });
      say(reset ? "Back to the defaults." : "Saved. The next draft uses these.");
      await load();
    } catch (e) {
      sayError(e);
    } finally {
      busy = false;
    }
  }

  function toggleGenre(g: string) {
    if (!knobs) return;
    knobs.excludeGenres = knobs.excludeGenres.includes(g) ? knobs.excludeGenres.filter((x) => x !== g) : [...knobs.excludeGenres, g];
  }

  // The found rate is stored as 0..1 and edited as a percentage.
  const shown = (k: NumKey, pct?: boolean) => (knobs ? (pct ? Math.round(knobs[k] * 100) : knobs[k]) : 0);
  function set(k: NumKey, v: string, pct?: boolean) {
    if (!knobs) return;
    const n = Number(v);
    if (Number.isFinite(n)) knobs[k] = pct ? n / 100 : n;
  }
</script>

{#if !knobs || !defaults}
  <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
{:else}
  <div class="bar">
    <span class="muted">{custom ? `Your settings, saved ${ago(savedAt ? new Date(savedAt * 1000).toISOString() : null)}.` : "Using the built-in settings."} Applies to the next draft.</span>
    <div class="row-actions">
      {#if custom}<m3e-button variant="text" disabled={busy} onclick={() => save(true)}><m3e-icon slot="icon" name="restart_alt"></m3e-icon>Use defaults</m3e-button>{/if}
      <m3e-button variant="text" disabled={busy || !dirty} onclick={() => (knobs = structuredClone($state.snapshot(saved!)))}>Undo edits</m3e-button>
      <m3e-button variant="filled" disabled={busy || !dirty} onclick={() => save()}><m3e-icon slot="icon" name="save"></m3e-icon>Save</m3e-button>
    </div>
  </div>

  <div class="grid cols-2 groups">
    {#each GROUPS as g (g.title)}
      <section class="card">
        <h2><m3e-icon name={g.icon}></m3e-icon>{g.title}</h2>
        {#if g.title === "Tonight's pick"}
          <label class="sw">
            <m3e-switch checked={knobs.pick} onchange={(e: Event) => (knobs!.pick = (e.currentTarget as HTMLInputElement).checked)} aria-label="Tonight's pick"></m3e-switch>
            <span>{knobs.pick ? "One hidden gem a day" : "No pick, trending only"}</span>
          </label>
        {/if}
        <div class="fields">
          {#each g.fields as f (f.key)}
            <label class="lab">
              <span>{f.label}</span>
              <span class="field">
                <input type="number" step={f.step ?? 1} value={shown(f.key, f.pct)} oninput={(e) => set(f.key, (e.currentTarget as HTMLInputElement).value, f.pct)} />
                {#if f.pct}<span class="muted">%</span>{/if}
              </span>
              <span class="muted hint">{f.hint}{knobs[f.key] !== defaults[f.key] ? ` · default ${f.pct ? Math.round(defaults[f.key] * 100) + "%" : defaults[f.key]}` : ""}</span>
            </label>
          {/each}
        </div>
      </section>
    {/each}
  </div>

  <section class="card">
    <h2><m3e-icon name="block"></m3e-icon>Never schedule these genres</h2>
    <m3e-filter-chip-set aria-label="Excluded genres">
      {#each genres as g (g)}
        <m3e-filter-chip selected={knobs.excludeGenres.includes(g)} onclick={() => toggleGenre(g)}>{g}</m3e-filter-chip>
      {/each}
    </m3e-filter-chip-set>
    <p class="muted hint">A film is skipped if any of its genres is picked here. Pinned titles still air.</p>
  </section>
{/if}

<style>
  .bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
  }
  .groups {
    align-items: start;
  }
  .fields {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 12px;
  }
  .hint {
    font-weight: 400;
    font-size: 12px;
    margin: 0;
  }
  p.hint {
    margin-top: 10px;
  }
  .sw {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 12px;
    font-weight: 700;
  }
</style>
