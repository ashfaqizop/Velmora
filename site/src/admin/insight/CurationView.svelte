<script lang="ts">
  // What Social puts in front of people: featured accounts, staff picks, and
  // who is kept off the leaderboards.
  import { onMount } from "svelte";
  import Avatar from "../../auth/Avatar.svelte";
  import { call, say, sayError } from "../api";
  import { ago } from "../format";
  import MoviePicker from "../theatre/MoviePicker.svelte";
  import type { Movie } from "../theatre/types";

  interface Profile {
    id: string;
    username: string;
    display_name: string;
    avatar: string;
    verified: boolean;
    hidden: boolean;
    banned: boolean;
    followers_count: number;
  }
  interface Featured {
    id: number;
    kind: "person" | "title";
    user_id: string | null;
    k: string | null;
    name: string;
    poster: string;
    note: string;
    profile?: Profile | null;
  }
  interface Hidden {
    user_id: string;
    created_at: string;
    profile: Profile | null;
  }

  let people = $state<Featured[]>([]);
  let titles = $state<Featured[]>([]);
  let hidden = $state<Hidden[]>([]);
  let loading = $state(true);
  let busy = $state(false);

  let personQ = $state("");
  let personNote = $state("");
  let titleNote = $state("");
  let hideQ = $state("");

  async function load() {
    try {
      const r = await call<{ people: Featured[]; titles: Featured[]; hidden: Hidden[] }>("curation");
      people = r.people;
      titles = r.titles;
      hidden = r.hidden;
    } catch (e) {
      sayError(e);
    } finally {
      loading = false;
    }
  }
  onMount(load);

  async function act(body: Record<string, unknown>, done?: string) {
    busy = true;
    try {
      await call("curationSet", body);
      if (done) say(done);
      await load();
      return true;
    } catch (e) {
      sayError(e);
      return false;
    } finally {
      busy = false;
    }
  }

  async function addPerson() {
    if (await act({ op: "addPerson", value: personQ.trim(), note: personNote.trim() }, "Featured.")) personQ = personNote = "";
  }
  async function addTitle(m: Movie) {
    if (await act({ op: "addTitle", value: m.k, note: titleNote.trim() }, `${m.name} is a staff pick.`)) titleNote = "";
  }
  async function note(f: Featured) {
    const next = prompt("Short note shown with it (140 characters)", f.note);
    if (next !== null) await act({ op: "note", id: f.id, note: next }, "Note saved.");
  }
  async function hide() {
    if (await act({ op: "hide", value: hideQ.trim() }, "Off the leaderboards.")) hideQ = "";
  }
</script>

<div class="adm-page">
  <div class="adm-head">
    <div class="grow">
      <h1>Curation</h1>
      <p>Featured people and staff picks show at the top of Social's home page, in the order set here. Hidden people stay on Social but never appear on a leaderboard.</p>
    </div>
  </div>

  {#if loading}
    <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
  {:else}
    <div class="grid cols-2 top">
      <section class="card">
        <h2><m3e-icon name="verified"></m3e-icon>Featured people</h2>
        <div class="add">
          <span class="field"><m3e-icon name="alternate_email"></m3e-icon><input bind:value={personQ} placeholder="username" aria-label="Username" autocomplete="off" spellcheck="false" /></span>
          <span class="field"><input bind:value={personNote} maxlength="140" placeholder="Why them (optional, shown)" aria-label="Note" /></span>
          <m3e-button variant="filled" disabled={busy || !personQ.trim()} onclick={addPerson}><m3e-icon slot="icon" name="add"></m3e-icon>Feature</m3e-button>
        </div>
        {#if !people.length}
          <div class="empty">No one featured. Social shows the usual rows.</div>
        {/if}
        {#each people as f, i (f.id)}
          <div class="item">
            {#if f.profile}<Avatar value={f.profile.avatar} size={40} />{/if}
            <div class="who">
              <strong>{f.profile?.display_name ?? "Gone"}</strong>
              <span class="muted small">@{f.profile?.username ?? "?"}{f.note ? ` · ${f.note}` : ""}</span>
              {#if f.profile?.hidden || f.profile?.banned}<span class="chip warn">Hidden or banned: not shown</span>{/if}
            </div>
            <div class="acts">
              <m3e-icon-button aria-label="Move up" title="Move up" disabled={busy || i === 0} onclick={() => act({ op: "move", id: f.id, by: -1 })}><m3e-icon name="arrow_upward"></m3e-icon></m3e-icon-button>
              <m3e-icon-button aria-label="Move down" title="Move down" disabled={busy || i === people.length - 1} onclick={() => act({ op: "move", id: f.id, by: 1 })}><m3e-icon name="arrow_downward"></m3e-icon></m3e-icon-button>
              <m3e-icon-button aria-label="Note" title="Note" disabled={busy} onclick={() => note(f)}><m3e-icon name="edit_note"></m3e-icon></m3e-icon-button>
              <m3e-icon-button aria-label="Remove" title="Remove" disabled={busy} onclick={() => act({ op: "remove", id: f.id }, "Removed.")}><m3e-icon name="close"></m3e-icon></m3e-icon-button>
            </div>
          </div>
        {/each}
      </section>

      <section class="card">
        <h2><m3e-icon name="auto_awesome"></m3e-icon>Staff picks</h2>
        <div class="add col">
          <span class="field"><input bind:value={titleNote} maxlength="140" placeholder="Note under the poster (optional), then search" aria-label="Note" /></span>
          <MoviePicker shows label="Search movies and shows" onpick={addTitle} />
        </div>
        {#if !titles.length}
          <div class="empty">No staff picks yet.</div>
        {/if}
        {#each titles as f, i (f.id)}
          <div class="item">
            {#if f.poster}<img src={f.poster} alt="" width="34" height="51" />{/if}
            <div class="who">
              <strong>{f.name}</strong>
              <span class="muted small">{f.k}{f.note ? ` · ${f.note}` : ""}</span>
            </div>
            <div class="acts">
              <m3e-icon-button aria-label="Move up" title="Move up" disabled={busy || i === 0} onclick={() => act({ op: "move", id: f.id, by: -1 })}><m3e-icon name="arrow_upward"></m3e-icon></m3e-icon-button>
              <m3e-icon-button aria-label="Move down" title="Move down" disabled={busy || i === titles.length - 1} onclick={() => act({ op: "move", id: f.id, by: 1 })}><m3e-icon name="arrow_downward"></m3e-icon></m3e-icon-button>
              <m3e-icon-button aria-label="Note" title="Note" disabled={busy} onclick={() => note(f)}><m3e-icon name="edit_note"></m3e-icon></m3e-icon-button>
              <m3e-icon-button aria-label="Remove" title="Remove" disabled={busy} onclick={() => act({ op: "remove", id: f.id }, "Removed.")}><m3e-icon name="close"></m3e-icon></m3e-icon-button>
            </div>
          </div>
        {/each}
      </section>
    </div>

    <section class="card">
      <h2><m3e-icon name="leaderboard"></m3e-icon>Kept off the leaderboards</h2>
      <div class="add">
        <span class="field"><m3e-icon name="alternate_email"></m3e-icon><input bind:value={hideQ} placeholder="username" aria-label="Username" autocomplete="off" spellcheck="false" /></span>
        <m3e-button variant="tonal" disabled={busy || !hideQ.trim()} onclick={hide}><m3e-icon slot="icon" name="visibility_off"></m3e-icon>Hide from leaderboards</m3e-button>
      </div>
      {#if !hidden.length}
        <div class="empty">Everyone can show up on the leaderboards.</div>
      {/if}
      <div class="hidden-grid">
        {#each hidden as h (h.user_id)}
          <div class="item">
            {#if h.profile}<Avatar value={h.profile.avatar} size={36} />{/if}
            <div class="who">
              <strong>{h.profile?.display_name ?? "Gone"}</strong>
              <span class="muted small">@{h.profile?.username ?? "?"} · {ago(h.created_at)}</span>
            </div>
            <m3e-button variant="text" disabled={busy} onclick={() => act({ op: "unhide", value: h.user_id }, "Back on the leaderboards.")}>Show again</m3e-button>
          </div>
        {/each}
      </div>
    </section>
  {/if}
</div>

<style>
  .top {
    align-items: start;
  }
  .add {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
    align-items: center;
    margin-bottom: 12px;
  }
  .add.col {
    flex-direction: column;
    align-items: stretch;
  }
  .add .field {
    flex: 1;
    min-width: 180px;
  }
  .item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px;
    border-radius: 16px;
  }
  .item:hover {
    background: color-mix(in srgb, var(--md-sys-color-on-surface) 4%, transparent);
  }
  .item img {
    border-radius: 6px;
    object-fit: cover;
  }
  .who {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .small {
    font-size: 13px;
  }
  .acts {
    display: flex;
  }
  .hidden-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 4px 12px;
  }
</style>
