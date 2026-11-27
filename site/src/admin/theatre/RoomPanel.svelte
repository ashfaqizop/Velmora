<script lang="ts">
  // One screening's room: who got in, the chat mode, and who may speak.
  // Chat is never stored, so there is no message history here by design; the
  // controls act on the room as it is now.
  import { onMount } from "svelte";
  import Avatar from "../../auth/Avatar.svelte";
  import { call, say, sayError } from "../api";
  import { ago, count } from "../format";
  import { clockOf, local, PHASE_LABEL, phaseOf, type Screening, type Who } from "./types";

  let { id, onclose }: { id: string; onclose: () => void } = $props();

  interface Person {
    user_id: string;
    status: string;
    joined_at: string | null;
    p: Who | null;
    ban: "mute" | "remove" | null;
  }

  let dialog = $state<HTMLElement & { show(): void; hide(): void }>();
  let s = $state<Screening | null>(null);
  let people = $state<Person[]>([]);
  let entered = $state(0);
  let busy = $state(false);
  let grant = $state("");

  async function load() {
    try {
      const r = (await call<{ screening: Screening; people: Person[]; entered: number }>("theatreRoom", { sid: id })) as unknown as { screening: Screening; people: Person[]; entered: number };
      s = r.screening;
      people = r.people;
      entered = r.entered;
    } catch (e) {
      sayError(e);
      onclose();
    }
  }

  onMount(() => {
    dialog?.show();
    void load();
    // The room moves; keep the numbers fresh while this is open.
    const t = setInterval(load, 20_000);
    return () => clearInterval(t);
  });

  async function run(action: string, body: Record<string, unknown>, done: string) {
    busy = true;
    try {
      await call(action, { sid: id, ...body });
      say(done);
      await load();
    } catch (e) {
      sayError(e);
    } finally {
      busy = false;
    }
  }

  const MODES = [
    { id: "normal", label: "Normal" },
    { id: "slow", label: "Slow (10 s)" },
    { id: "closed", label: "Reactions only" },
  ] as const;

  async function grantSeat(e: Event) {
    e.preventDefault();
    if (!grant.trim()) return;
    await run("theatreTicketSet", { op: "grant", user: grant.trim() }, "Seat granted.");
    grant = "";
  }

  const phase = $derived(s ? phaseOf(s) : "upcoming");
  const roomOpen = $derived(phase === "lobby" || phase === "live" || phase === "after");
</script>

<m3e-dialog bind:this={dialog} dismissible onclosed={onclose}>
  <span slot="header">{s?.name ?? "Room"}</span>

  {#if !s}
    <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
  {:else}
    <div class="body">
      <div class="top">
        <span class="chip {phase === 'live' ? 'bad' : roomOpen ? 'warn' : ''}">{PHASE_LABEL[phase]}</span>
        <span class="muted">{local(s.starts_at)} to {clockOf(s.ends_at)}</span>
        {#if s.special}<span class="chip info">Special</span>{/if}
        {#if s.status === "cancelled"}<span class="chip bad">Cancelled</span>{/if}
      </div>

      <div class="stats">
        <div><strong>{count(s.booked)}</strong><span>booked of {count(s.capacity)}</span></div>
        <div><strong>{count(s.waitlisted)}</strong><span>waitlisted</span></div>
        <div><strong>{count(entered)}</strong><span>came in so far</span></div>
      </div>

      <div class="lab">
        <span>Chat</span>
        <m3e-filter-chip-set aria-label="Chat mode">
          {#each MODES as m (m.id)}
            <m3e-filter-chip selected={s.chat_mode === m.id} onclick={() => s!.chat_mode !== m.id && run("theatreChat", { mode: m.id }, `Chat: ${m.label}.`)}>{m.label}</m3e-filter-chip>
          {/each}
        </m3e-filter-chip-set>
        <span class="muted small">"Reactions only" shuts text chat for everyone in this room right away. Nothing is stored, so there's no message history to read.</span>
      </div>

      <form class="grant" onsubmit={grantSeat}>
        <label class="field"><input bind:value={grant} placeholder="Give someone a seat (@username)" aria-label="Username to give a seat" autocomplete="off" spellcheck="false" /></label>
        <m3e-button variant="tonal" disabled={busy || !grant.trim() || s.status !== "scheduled"} onclick={grantSeat}>Give seat</m3e-button>
      </form>

      <div class="lab">
        <span>People ({people.length})</span>
        {#if !people.length}
          <span class="muted small">No one has a ticket or has come in yet.</span>
        {:else}
          <ul class="people">
            {#each people as p (p.user_id)}
              <li>
                {#if p.p}<Avatar value={p.p.avatar} size={32} />{/if}
                <span class="who">
                  <strong>{p.p?.display_name ?? "Unknown"}</strong>
                  <span class="muted">@{p.p?.username} · {p.joined_at ? `came in ${ago(p.joined_at)}` : p.status}</span>
                </span>
                {#if p.ban}<span class="chip {p.ban === 'remove' ? 'bad' : 'warn'}">{p.ban === "remove" ? "Removed" : "Muted"}</span>{/if}
                <span class="row-actions">
                  {#if p.ban}
                    <m3e-button variant="text" disabled={busy} onclick={() => run("theatreMember", { user: p.user_id, what: "clear" }, "Restored.")}>Restore</m3e-button>
                  {:else}
                    <m3e-button variant="text" disabled={busy} onclick={() => run("theatreMember", { user: p.user_id, what: "mute" }, "Muted in this room.")}>Mute</m3e-button>
                    <m3e-button variant="text" disabled={busy} onclick={() => run("theatreMember", { user: p.user_id, what: "remove" }, "Removed from this room.")}>Remove</m3e-button>
                  {/if}
                  {#if p.status === "booked" && s.status === "scheduled"}
                    <m3e-button variant="text" disabled={busy} onclick={() => run("theatreTicketSet", { op: "revoke", user: p.user_id }, "Seat taken back.")}>Take seat</m3e-button>
                  {/if}
                </span>
              </li>
            {/each}
          </ul>
          <span class="muted small">Muting is immediate. Removing closes the room to them the next time it checks, which can take a minute.</span>
        {/if}
      </div>
    </div>
  {/if}

  <div slot="actions" class="actions"><m3e-button variant="text" onclick={() => dialog?.hide()}>Close</m3e-button></div>
</m3e-dialog>

<style>
  .body {
    display: flex;
    flex-direction: column;
    gap: 12px;
    width: 100%;
  }
  .top {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
    font-size: 14px;
  }
  .stats {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 10px;
  }
  .stats div {
    display: flex;
    flex-direction: column;
    padding: 12px 14px;
    border-radius: 16px;
    background: var(--md-sys-color-surface-container-high);
  }
  .stats strong {
    font-family: var(--display);
    font-weight: 400;
    font-size: 28px;
    line-height: 1;
  }
  .stats span {
    font-size: 12.5px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .lab {
    display: flex;
    flex-direction: column;
    gap: 8px;
    font-size: 13px;
    font-weight: 700;
  }
  .small {
    font-weight: 400;
    font-size: 12.5px;
  }
  .grant {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .grant .field {
    flex: 1;
    min-width: 0;
  }
  .people {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    max-height: 320px;
    overflow-y: auto;
  }
  .people li {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 4px;
    font-weight: 400;
  }
  .people li + li {
    border-top: 1px solid color-mix(in srgb, var(--md-sys-color-outline-variant) 30%, transparent);
  }
  .who {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    font-size: 14px;
  }
  .who .muted {
    font-size: 12px;
  }
  .row-actions {
    gap: 0;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
  }
</style>
