<script lang="ts">
  // Your hand on the algorithm: pin a movie to a day, or block it for good.
  import { onMount } from "svelte";
  import { call, say, sayError } from "../api";
  import MoviePicker from "./MoviePicker.svelte";
  import type { Movie } from "./types";

  interface Override { id: number; action: "pin" | "block"; k: string; day: string | null; note: string; name: string | null; year: number | null }

  let rows = $state<Override[]>([]);
  let loading = $state(true);
  let action = $state<"pin" | "block">("pin");
  let movie = $state<Movie | null>(null);
  let day = $state(new Date().toISOString().slice(0, 10));
  let note = $state("");
  let busy = $state(false);

  async function load() {
    try {
      rows = ((await call<{ rows: Override[] }>("theatreOverrides")) as unknown as { rows: Override[] }).rows;
    } catch (e) {
      sayError(e);
    } finally {
      loading = false;
    }
  }
  onMount(load);

  async function add() {
    if (!movie) return;
    busy = true;
    try {
      await call("theatreOverrideSet", { op: "add", action, k: movie.k, day: action === "pin" ? day : undefined, note });
      say(action === "pin" ? "Pinned. The algorithm will use it for that day if it fits." : "Blocked. It won't be scheduled again.");
      movie = null;
      note = "";
      await load();
    } catch (e) {
      sayError(e);
    } finally {
      busy = false;
    }
  }

  async function remove(id: number) {
    busy = true;
    try {
      await call("theatreOverrideSet", { op: "del", id });
      rows = rows.filter((r) => r.id !== id);
    } catch (e) {
      sayError(e);
    } finally {
      busy = false;
    }
  }
</script>

<div class="grid cols-2">
  <section class="card">
    <h2><m3e-icon name="push_pin"></m3e-icon>Add a pin or a block</h2>
    <div class="form">
      <m3e-filter-chip-set aria-label="Kind">
        <m3e-filter-chip selected={action === "pin"} onclick={() => (action = "pin")}>Pin to a day</m3e-filter-chip>
        <m3e-filter-chip selected={action === "block"} onclick={() => (action = "block")}>Block</m3e-filter-chip>
      </m3e-filter-chip-set>

      {#if movie}
        <div class="picked">
          {#if movie.poster}<img src={movie.poster} alt="" width="40" height="60" />{/if}
          <div><strong>{movie.name}</strong><div class="muted">{movie.year ?? ""}</div></div>
          <m3e-button variant="text" onclick={() => (movie = null)}>Change</m3e-button>
        </div>
      {:else}
        <MoviePicker onpick={(m) => (movie = m)} />
      {/if}

      {#if action === "pin"}
        <label class="lab">
          <span>Day (UTC)</span>
          <span class="field"><input type="date" bind:value={day} min={new Date().toISOString().slice(0, 10)} /></span>
        </label>
      {/if}
      <label class="field"><input bind:value={note} maxlength="200" placeholder="Note to yourself (optional)" aria-label="Note" /></label>
      <m3e-button variant="filled" disabled={busy || !movie || (action === "pin" && !day)} onclick={add}>{action === "pin" ? "Pin it" : "Block it"}</m3e-button>
      <p class="muted small">
        {action === "pin" ? "Pinned titles go into that day's lineup when the daily job drafts it, if they fit. To put one on at an exact time, add a screening instead." : "A blocked movie is skipped by every future lineup. A screening that's already scheduled stays until you cancel it."}
      </p>
    </div>
  </section>

  <section class="card">
    <h2><m3e-icon name="list"></m3e-icon>Current ({rows.length})</h2>
    {#if loading}
      <m3e-loading-indicator></m3e-loading-indicator>
    {:else if !rows.length}
      <p class="muted">Nothing pinned or blocked.</p>
    {:else}
      <ul class="list">
        {#each rows as r (r.id)}
          <li>
            <span class="chip {r.action === 'pin' ? 'info' : 'bad'}">{r.action === "pin" ? `Pin · ${r.day}` : "Block"}</span>
            <span class="what"><strong>{r.name ?? r.k}</strong>{#if r.year}<span class="muted"> ({r.year})</span>{/if}{#if r.note}<span class="muted"> · {r.note}</span>{/if}</span>
            <m3e-icon-button aria-label="Remove" disabled={busy} onclick={() => remove(r.id)}><m3e-icon name="close"></m3e-icon></m3e-icon-button>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
</div>

<style>
  .form {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .picked {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 10px;
    border-radius: 16px;
    background: var(--md-sys-color-surface-container-high);
  }
  .picked img {
    border-radius: 6px;
    object-fit: cover;
  }
  .picked > div {
    flex: 1;
  }
  .lab {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 13px;
    font-weight: 700;
  }
  .field input {
    color-scheme: dark;
  }
  .small {
    margin: 0;
    font-size: 12.5px;
  }
  .list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .list li {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .what {
    flex: 1;
    min-width: 0;
    font-size: 14px;
  }
</style>
