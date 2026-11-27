<script lang="ts">
  // One person's tickets and no-shows. Three no-shows in 30 days push their
  // booking window back an hour; fix a wrong one here.
  import Avatar from "../../auth/Avatar.svelte";
  import { call, say, sayError } from "../api";
  import { ago } from "../format";
  import { local, type Who } from "./types";

  interface Ticket {
    screening_id: string;
    status: string;
    is_max: boolean;
    created_at: string;
    joined_at: string | null;
    s: { name: string; starts_at: string; ends_at: string; status: string } | null;
  }

  let who = $state("");
  let profile = $state<Who | null>(null);
  let tickets = $state<Ticket[]>([]);
  let noshows = $state(0);
  let loading = $state(false);
  let busy = $state(false);
  let searched = $state(false);

  async function find(e?: Event) {
    e?.preventDefault();
    if (!who.trim()) return;
    loading = true;
    try {
      const r = (await call<{ profile: Who; tickets: Ticket[]; noshows: number }>("theatreTickets", { user: who.trim() })) as unknown as { profile: Who; tickets: Ticket[]; noshows: number };
      profile = r.profile;
      tickets = r.tickets;
      noshows = r.noshows;
    } catch (err) {
      profile = null;
      tickets = [];
      sayError(err);
    } finally {
      loading = false;
      searched = true;
    }
  }

  async function set(t: Ticket, op: string, done: string) {
    if (!profile) return;
    busy = true;
    try {
      await call("theatreTicketSet", { sid: t.screening_id, user: profile.id, op });
      say(done);
      await find();
    } catch (e) {
      sayError(e);
    } finally {
      busy = false;
    }
  }

  const ended = (t: Ticket) => (t.s ? Date.parse(t.s.ends_at) < Date.now() : false);
  const tone = (s: string) => (s === "attended" ? "ok" : s === "noshow" ? "bad" : s === "waitlist" || s === "released" ? "warn" : s === "booked" ? "info" : "");
</script>

<form class="search" onsubmit={find}>
  <label class="field">
    <m3e-icon name="search"></m3e-icon>
    <input bind:value={who} placeholder="@username or account id" aria-label="Find someone's tickets" autocomplete="off" spellcheck="false" />
  </label>
  <m3e-button variant="filled" disabled={loading || !who.trim()} onclick={() => find()}>Look up</m3e-button>
</form>

{#if profile}
  <section class="card">
    <div class="who">
      <Avatar value={profile.avatar} size={48} />
      <div>
        <strong>{profile.display_name}</strong> <span class="muted">@{profile.username}</span>
        <div class="chips">
          <span class="chip {noshows >= 3 ? 'bad' : noshows ? 'warn' : 'ok'}">{noshows} no-show{noshows === 1 ? "" : "s"} in 30 days</span>
          {#if noshows >= 3}<span class="chip bad">Booking opens an hour later for them</span>{/if}
        </div>
      </div>
    </div>

    {#if !tickets.length}
      <p class="muted none">No tickets yet.</p>
    {:else}
      <ul class="list">
        {#each tickets as t (t.screening_id)}
          <li>
            <div class="what">
              <strong>{t.s?.name ?? "A screening that's gone"}</strong>
              <span class="muted">{t.s ? local(t.s.starts_at) : ""} · booked {ago(t.created_at)}{t.is_max ? " · Max" : ""}</span>
            </div>
            <span class="chip {tone(t.status)}">{t.status === "noshow" ? "no-show" : t.status}</span>
            <span class="row-actions">
              {#if t.status === "noshow"}
                <m3e-button variant="text" disabled={busy} onclick={() => set(t, "clear", "No-show cleared.")}>Clear no-show</m3e-button>
              {:else if ended(t) && t.status === "booked"}
                <m3e-button variant="text" disabled={busy} onclick={() => set(t, "noshow", "Marked as a no-show.")}>Mark no-show</m3e-button>
              {/if}
              {#if ended(t) && t.status !== "attended"}
                <m3e-button variant="text" disabled={busy} onclick={() => set(t, "attended", "Marked as attended.")}>Mark attended</m3e-button>
              {/if}
              {#if !ended(t) && (t.status === "booked" || t.status === "waitlist") && t.s?.status === "scheduled"}
                <m3e-button variant="text" disabled={busy} onclick={() => set(t, "revoke", "Seat taken back.")}>Take back</m3e-button>
              {/if}
            </span>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
{:else if searched && !loading}
  <div class="card empty">Nobody found.</div>
{/if}

<style>
  .search {
    display: flex;
    gap: 10px;
    align-items: center;
    max-width: 520px;
  }
  .search .field {
    flex: 1;
    min-width: 0;
  }
  .who {
    display: flex;
    gap: 12px;
    align-items: center;
    margin-bottom: 14px;
  }
  .chips {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    margin-top: 8px;
  }
  .none {
    margin: 0;
    font-size: 14px;
  }
  .list {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .list li {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 0;
    flex-wrap: wrap;
  }
  .list li + li {
    border-top: 1px solid color-mix(in srgb, var(--md-sys-color-outline-variant) 30%, transparent);
  }
  .what {
    flex: 1;
    min-width: 200px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 14.5px;
  }
  .what .muted {
    font-size: 12.5px;
  }
  .row-actions {
    gap: 0;
  }
</style>
