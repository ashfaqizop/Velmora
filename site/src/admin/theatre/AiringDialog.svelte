<script lang="ts">
  // Add a screening by hand (a "special"), or edit one that exists. Movies
  // only. You pick the exact start; the form warns about clashes but never
  // blocks, because a runtime has no limit and overlaps are sometimes the point.
  import { onMount, untrack } from "svelte";
  import { call, say, sayError } from "../api";
  import MoviePicker from "./MoviePicker.svelte";
  import { clockOf, fromInput, local, toInput, utc, type Movie, type Screening } from "./types";

  let { row = null, onclose, onsaved }: { row?: Screening | null; onclose: () => void; onsaved: () => void } = $props();

  interface Check {
    runtime: number;
    ends_at: string;
    title: { name: string; year: number | null; poster: string | null; runtime: number | null } | null;
    overlaps: { id: string; name: string; starts_at: string; ends_at: string; capacity: number; status: string }[];
    warnings: string[];
  }

  // The parent re-creates this dialog for a different screening, so the form starts from one snapshot.
  const initial = untrack(() => row);
  const editing = $derived(row !== null);
  let dialog = $state<HTMLElement & { show(): void; hide(): void }>();
  let movie = $state<Movie | null>(initial ? { k: initial.k, name: initial.name, year: initial.year, poster: initial.poster } : null);
  let start = $state(initial ? toInput(initial.starts_at) : "");
  let runtime = $state(initial ? String(initial.runtime) : "");
  let mode = $state<"open" | "fixed">(initial?.mode ?? "open");
  let capacity = $state(String(initial?.capacity ?? 60));
  let booking = $state<"max_first" | "now">("max_first");
  let note = $state(initial?.note ?? "");
  let check = $state<Check | null>(null);
  let checkError = $state("");
  let busy = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;
  let seq = 0;

  onMount(() => dialog?.show());

  const iso = $derived(fromInput(start));

  // Re-check whenever the movie, time, runtime or seats change.
  $effect(() => {
    const k = movie?.k;
    const at = iso;
    const rt = runtime;
    const cap = capacity;
    clearTimeout(timer);
    check = null;
    checkError = "";
    if (!k || !at) return;
    timer = setTimeout(async () => {
      const mine = ++seq;
      try {
        const r = await call<Check>("theatreCheck", { k, starts_at: at, runtime: rt ? Number(rt) : undefined, capacity: Number(cap) || 60, exclude: row?.id });
        if (mine === seq) check = r as unknown as Check;
      } catch (e) {
        if (mine === seq) checkError = (e as Error).message;
      }
    }, 400);
    return () => clearTimeout(timer);
  });

  async function save(status: "draft" | "scheduled") {
    if (!movie || !iso) return;
    busy = true;
    try {
      const r = await call<{ warnings: string[] }>("theatreSave", {
        id: row?.id,
        k: movie.k,
        starts_at: iso,
        runtime: runtime ? Number(runtime) : undefined,
        mode,
        capacity: Number(capacity) || 60,
        note,
        status,
        booking,
      });
      say(editing ? "Saved." : status === "scheduled" ? "Published. Booking is open." : "Saved as a draft.");
      for (const w of (r as unknown as { warnings: string[] }).warnings ?? []) say(w);
      onsaved();
      dialog?.hide();
    } catch (e) {
      sayError(e);
    } finally {
      busy = false;
    }
  }

  const ready = $derived(Boolean(movie && iso) && !busy);
  const wasDraft = $derived(!row || row.status === "draft");
</script>

<m3e-dialog bind:this={dialog} dismissible onclosed={onclose}>
  <span slot="header">{editing ? "Edit screening" : "Add a screening"}</span>

  <div class="body">
    {#if movie}
      <div class="picked">
        {#if movie.poster}<img src={movie.poster} alt="" width="48" height="72" />{/if}
        <div>
          <strong>{movie.name}</strong>
          <div class="muted">{movie.year ?? ""}{check?.runtime ? ` · ${check.runtime} min` : ""}</div>
        </div>
        {#if !editing}<m3e-button variant="text" onclick={() => (movie = null)}>Change</m3e-button>{/if}
      </div>
    {:else}
      <MoviePicker onpick={(m) => (movie = m)} />
    {/if}

    <label class="lab">
      <span>Starts at (your time)</span>
      <span class="field"><input type="datetime-local" bind:value={start} aria-label="Start time" /></span>
      {#if iso}<span class="muted small">{utc(iso)} for everyone else's clock; each person sees their own.</span>{/if}
    </label>

    <div class="two">
      <label class="lab">
        <span>Runtime (minutes)</span>
        <span class="field"><input type="number" min="20" max="600" bind:value={runtime} placeholder={check?.title?.runtime ? String(check.title.runtime) : "from TMDB"} /></span>
      </label>
      <label class="lab">
        <span>Seats</span>
        <span class="field"><input type="number" min="1" max="1000" bind:value={capacity} /></span>
      </label>
    </div>

    <div class="lab">
      <span>Seating</span>
      <m3e-filter-chip-set aria-label="Seating">
        <m3e-filter-chip selected={mode === "open"} onclick={() => (mode = "open")}>Open: drop in, no ticket</m3e-filter-chip>
        <m3e-filter-chip selected={mode === "fixed"} onclick={() => (mode = "fixed")}>Ticketed: seats can sell out</m3e-filter-chip>
      </m3e-filter-chip-set>
    </div>

    {#if wasDraft}
      <div class="lab">
        <span>Booking opens</span>
        <m3e-filter-chip-set aria-label="Booking opens">
          <m3e-filter-chip selected={booking === "max_first"} onclick={() => (booking = "max_first")}>Max first, everyone later</m3e-filter-chip>
          <m3e-filter-chip selected={booking === "now"} onclick={() => (booking = "now")}>Everyone now</m3e-filter-chip>
        </m3e-filter-chip-set>
      </div>
    {/if}

    <label class="lab">
      <span>Public note (optional)</span>
      <span class="field"><input bind:value={note} maxlength="200" placeholder="Director's cut, a special night…" /></span>
    </label>

    {#if checkError}<div class="err">{checkError}</div>{/if}
    {#if check}
      <div class="check">
        <span class="muted">Ends {clockOf(check.ends_at)}</span>
        {#each check.warnings as w (w)}<div class="warn"><m3e-icon name="warning"></m3e-icon>{w}</div>{/each}
        {#if check.overlaps.length}
          <ul>
            {#each check.overlaps as o (o.id)}
              <li><span class="mono">{local(o.starts_at)}</span> {o.name} <span class="chip">{o.status}</span></li>
            {/each}
          </ul>
        {:else}
          <span class="muted">Nothing else is on at that time.</span>
        {/if}
      </div>
    {/if}
  </div>

  <div slot="actions" class="actions">
    <m3e-button variant="text" onclick={() => dialog?.hide()}>Cancel</m3e-button>
    {#if wasDraft}
      <m3e-button variant="tonal" disabled={!ready} onclick={() => save("draft")}>Save as draft</m3e-button>
      <m3e-button variant="filled" disabled={!ready} onclick={() => save("scheduled")}><m3e-icon slot="icon" name="event_available"></m3e-icon>Publish</m3e-button>
    {:else}
      <m3e-button variant="filled" disabled={!ready} onclick={() => save("scheduled")}>Save changes</m3e-button>
    {/if}
  </div>
</m3e-dialog>

<style>
  .body {
    display: flex;
    flex-direction: column;
    gap: 12px;
    width: 100%;
  }
  .picked {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px;
    border-radius: 18px;
    background: var(--md-sys-color-surface-container-high);
  }
  .picked img {
    border-radius: 8px;
    object-fit: cover;
  }
  .picked > div {
    flex: 1;
    min-width: 0;
  }
  .lab {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 13px;
    font-weight: 700;
  }
  .lab .small {
    font-weight: 400;
    font-size: 12.5px;
  }
  .two {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }
  .field input {
    color-scheme: dark;
  }
  .check {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px 14px;
    border-radius: 16px;
    background: var(--md-sys-color-surface-container-high);
    font-size: 13.5px;
  }
  .check ul {
    margin: 0;
    padding-left: 18px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .warn {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    color: #ffd08a;
    --m3e-icon-size: 18px;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    flex-wrap: wrap;
  }
</style>
