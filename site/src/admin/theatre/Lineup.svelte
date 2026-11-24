<script lang="ts">
  // The next week of screenings (and yesterday's), grouped by day in your
  // time zone. Add one by hand, edit, manage the room, cancel, or run the
  // algorithm for a day.
  import { onMount } from "svelte";
  import { call, say, sayError } from "../api";
  import AiringDialog from "./AiringDialog.svelte";
  import AlgorithmDialog from "./AlgorithmDialog.svelte";
  import RoomPanel from "./RoomPanel.svelte";
  import { dayLabel, local, PHASE_LABEL, phaseOf, utc, type Screening } from "./types";

  let rows = $state<Screening[]>([]);
  let loading = $state(true);
  let adding = $state(false);
  let editing = $state<Screening | null>(null);
  let algo = $state(false);
  let room = $state<string | null>(null);
  let cancelling = $state<string | null>(null);
  let why = $state("");
  let busy = $state(false);

  async function load() {
    try {
      const r = (await call<{ rows: Screening[] }>("theatreLineup", { back: 1, ahead: 7 })) as unknown as { rows: Screening[] };
      rows = r.rows;
    } catch (e) {
      sayError(e);
    } finally {
      loading = false;
    }
  }
  onMount(load);

  const groups = $derived.by(() => {
    const out: { label: string; items: Screening[] }[] = [];
    for (const s of rows) {
      const label = dayLabel(s.starts_at);
      const g = out.find((x) => x.label === label);
      if (g) g.items.push(s);
      else out.push({ label, items: [s] });
    }
    return out;
  });

  async function cancel(s: Screening) {
    busy = true;
    try {
      const r = (await call<{ people: number }>("theatreCancel", { id: s.id, why })) as unknown as { people: number };
      say(r.people ? `Cancelled. ${r.people} people will see it next time they open Theatre.` : "Cancelled.");
      cancelling = null;
      why = "";
      await load();
    } catch (e) {
      sayError(e);
    } finally {
      busy = false;
    }
  }

  const over = (s: Screening) => phaseOf(s) === "over";
  const seatText = (s: Screening) => (s.mode === "fixed" ? `${s.booked}/${s.capacity} seats` : `${s.booked} in · open`);
</script>

<div class="bar">
  <m3e-button variant="filled" onclick={() => (adding = true)}><m3e-icon slot="icon" name="add"></m3e-icon>Add a screening</m3e-button>
  <m3e-button variant="tonal" onclick={() => (algo = true)}><m3e-icon slot="icon" name="auto_awesome"></m3e-icon>Lineup algorithm</m3e-button>
  <span class="muted note">Times are in your time zone. Each screening also shows UTC.</span>
</div>

{#if loading}
  <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
{:else if !rows.length}
  <div class="card empty">Nothing scheduled in the next week. The daily job drafts tomorrow's lineup around 00:10 UTC, or run the algorithm yourself.</div>
{:else}
  {#each groups as g (g.label)}
    <section class="day">
      <h3>{g.label}</h3>
      <div class="card list">
        {#each g.items as s (s.id)}
          {@const phase = phaseOf(s)}
          <div class="item" class:dim={over(s) || s.status === "cancelled"}>
            {#if s.poster}<img src={s.poster} alt="" width="44" height="66" loading="lazy" />{:else}<span class="blank"></span>{/if}
            <div class="main">
              <div class="t">
                <strong>{s.name}</strong>{#if s.year}<span class="muted"> ({s.year})</span>{/if}
              </div>
              <div class="meta">
                <span class="mono">{local(s.starts_at)}</span>
                <span class="muted mono">{utc(s.starts_at)}</span>
                <span class="muted">{s.runtime} min</span>
                {#if s.special}<span class="chip info">Special</span>{/if}
                {#if s.kind === "pick" && !s.special}<span class="chip">Tonight's pick</span>{/if}
                {#if s.status === "cancelled"}<span class="chip bad">Cancelled</span>
                {:else if s.status === "draft"}<span class="chip warn">Draft</span>
                {:else if phase === "live"}<span class="chip bad">Live</span>
                {:else if phase === "lobby" || phase === "after"}<span class="chip warn">{PHASE_LABEL[phase]}</span>
                {:else if phase === "over"}<span class="chip">Over</span>{/if}
                <span class="chip">{seatText(s)}</span>
                {#if s.waitlisted}<span class="chip">{s.waitlisted} waiting</span>{/if}
                {#if s.chat_mode !== "normal"}<span class="chip warn">Chat: {s.chat_mode === "closed" ? "reactions only" : "slow"}</span>{/if}
              </div>
              {#if s.note}<div class="muted note2">{s.note}</div>{/if}
              {#if cancelling === s.id}
                <div class="cancel">
                  <label class="field"><input bind:value={why} maxlength="200" placeholder="Reason people will see (optional)" aria-label="Reason" /></label>
                  <m3e-button variant="filled" disabled={busy} onclick={() => cancel(s)}>Cancel screening</m3e-button>
                  <m3e-button variant="text" disabled={busy} onclick={() => { cancelling = null; why = ""; }}>Keep it</m3e-button>
                </div>
              {/if}
            </div>
            {#if s.status !== "cancelled" && !over(s) && cancelling !== s.id}
              <div class="acts">
                {#if s.status === "scheduled"}<m3e-button variant="text" onclick={() => (room = s.id)}><m3e-icon slot="icon" name="forum"></m3e-icon>Room</m3e-button>{/if}
                <m3e-button variant="text" onclick={() => (editing = s)}><m3e-icon slot="icon" name="edit"></m3e-icon>Edit</m3e-button>
                <m3e-button variant="text" onclick={() => (cancelling = s.id)}><m3e-icon slot="icon" name="close"></m3e-icon>Cancel</m3e-button>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </section>
  {/each}
{/if}

{#if adding}<AiringDialog onclose={() => (adding = false)} onsaved={load} />{/if}
{#if editing}{#key editing.id}<AiringDialog row={editing} onclose={() => (editing = null)} onsaved={load} />{/key}{/if}
{#if algo}<AlgorithmDialog onclose={() => (algo = false)} onchanged={load} />{/if}
{#if room}<RoomPanel id={room} onclose={() => { room = null; void load(); }} />{/if}

<style>
  .bar {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }
  .note {
    font-size: 12.5px;
    margin-left: auto;
  }
  .day {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  h3 {
    margin: 8px 0 0;
    font-size: 13px;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--md-sys-color-on-surface-variant);
  }
  .list {
    padding: 6px 12px;
  }
  .item {
    display: flex;
    gap: 14px;
    align-items: flex-start;
    padding: 12px 0;
    transition: opacity 160ms ease;
  }
  .item + .item {
    border-top: 1px solid color-mix(in srgb, var(--md-sys-color-outline-variant) 30%, transparent);
  }
  .item.dim {
    opacity: 0.55;
  }
  img,
  .blank {
    width: 44px;
    height: 66px;
    border-radius: 8px;
    object-fit: cover;
    background: var(--md-sys-color-surface-container-highest);
    flex: none;
  }
  .main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .t strong {
    font-size: 15.5px;
  }
  .meta {
    display: flex;
    gap: 6px 10px;
    align-items: center;
    flex-wrap: wrap;
    font-size: 13px;
  }
  .note2 {
    font-size: 13px;
  }
  .acts {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    flex: none;
  }
  .cancel {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
  }
  .cancel .field {
    flex: 1;
    min-width: 200px;
  }
  @media (max-width: 700px) {
    .item {
      flex-wrap: wrap;
    }
    .acts {
      width: 100%;
      justify-content: flex-start;
    }
  }
</style>
