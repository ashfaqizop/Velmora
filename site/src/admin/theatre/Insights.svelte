<script lang="ts">
  // How Theatre is doing: per-screening numbers, the hours and genres that
  // fill rooms, where bookings come from, whether shows start, and what the
  // source checks say about the drafts.
  import { onMount } from "svelte";
  import { call, sayError } from "../api";
  import { count } from "../format";
  import Hours from "../insight/Hours.svelte";

  interface Show {
    id: string;
    day: string;
    k: string;
    name: string;
    kind: string;
    special: boolean;
    status: string;
    mode: string;
    starts_at: string;
    capacity: number;
    expected: number;
    booked: number;
    waitlist: number;
    attended: number;
    noshow: number;
    came_in: number;
    max_tickets: number;
    social_tickets: number;
    start_ok: number;
    start_failed: number;
    start_why: Record<string, number>;
  }
  interface Probe {
    id: string;
    k: string;
    name: string;
    starts_at: string;
    found: number;
    miss: number;
    verdict: "proven" | "waiting" | "drop" | "keep";
  }
  interface Data {
    from: string;
    to: string;
    shows: Show[];
    probes: Probe[];
    rule: { minChecks: number; minFound: number };
    totals: { screenings: number; cancelled: number; tickets: number; came_in: number; noshow: number; max: number; social: number; started: number; failed: number; fail_rate: number | null; lobby_ads: number };
    hours: { hour: number; screenings: number; avg: number }[];
    genres: { genre: string; screenings: number; avg: number }[];
  }

  let days = $state(30);
  let data = $state<Data | null>(null);
  let loading = $state(true);

  async function load() {
    loading = true;
    try {
      data = await call<Data>("theatreInsights", { days });
    } catch (e) {
      sayError(e);
    } finally {
      loading = false;
    }
  }
  onMount(load);

  const pct = (n: number, d: number) => (d ? `${Math.round((n / d) * 100)}%` : "–");
  const when = (iso: string) => new Date(iso).toLocaleString(undefined, { weekday: "short", day: "numeric", month: "short", hour: "2-digit", minute: "2-digit" });
  const VERDICT = { proven: ["ok", "Played here before"], keep: ["ok", "Keeps"], waiting: ["", "Waiting for checks"], drop: ["bad", "Will be dropped"] } as const;
  const peakGenre = $derived(Math.max(1, ...(data?.genres.map((g) => g.avg) ?? [1])));
</script>

<div class="bar">
  <m3e-filter-chip-set aria-label="Period">
    {#each [7, 30, 90] as d (d)}
      <m3e-filter-chip selected={days === d} onclick={() => { days = d; void load(); }}>{d} days</m3e-filter-chip>
    {/each}
  </m3e-filter-chip-set>
</div>

{#if !data}
  <div class="center"><m3e-loading-indicator></m3e-loading-indicator></div>
{:else}
  {@const t = data.totals}
  <div class="grid cols-4" class:busy={loading}>
    <div class="card stat"><span class="muted">Screenings held</span><strong>{count(t.screenings)}</strong><span class="muted">{t.cancelled} cancelled</span></div>
    <div class="card stat"><span class="muted">People who came in</span><strong>{count(t.came_in)}</strong><span class="muted">{count(t.tickets)} tickets · {pct(t.noshow, t.tickets)} no-shows</span></div>
    <div class="card stat" class:warn={(t.fail_rate ?? 0) > 0.15}>
      <span class="muted">Shows that failed to start</span>
      <strong>{t.fail_rate === null ? "–" : `${Math.round(t.fail_rate * 100)}%`}</strong>
      <span class="muted">{t.started + t.failed ? `${count(t.failed)} of ${count(t.started + t.failed)} reports` : "Apps report this from the next release"}</span>
    </div>
    <div class="card stat"><span class="muted">Lobby ads shown</span><strong>{count(t.lobby_ads)}</strong><span class="muted">Public tier only</span></div>
  </div>

  <div class="grid cols-2">
    <section class="card">
      <h2><m3e-icon name="confirmation_number"></m3e-icon>Where tickets come from</h2>
      <div class="split">
        <div>
          <span class="muted">App vs Social</span>
          <div class="pill"><span style:width={pct(t.tickets - t.social, t.tickets)}></span></div>
          <span class="legend"><b>{pct(t.tickets - t.social, t.tickets)}</b> app · <b>{pct(t.social, t.tickets)}</b> Social</span>
        </div>
        <div>
          <span class="muted">Public vs Max</span>
          <div class="pill alt"><span style:width={pct(t.tickets - t.max, t.tickets)}></span></div>
          <span class="legend"><b>{pct(t.tickets - t.max, t.tickets)}</b> Public · <b>{pct(t.max, t.tickets)}</b> Max</span>
        </div>
      </div>
      <p class="muted small">App vs Social is counted from bookings made after this update.</p>
    </section>

    <section class="card">
      <h2><m3e-icon name="theaters"></m3e-icon>Genres that fill rooms</h2>
      {#if !data.genres.length}
        <p class="muted small">No finished screenings in this period.</p>
      {:else}
        <ul class="genres">
          {#each data.genres as g (g.genre)}
            <li>
              <span>{g.genre}</span>
              <span class="track"><span style:width="{(g.avg / peakGenre) * 100}%"></span></span>
              <b>{g.avg.toFixed(1)}</b>
              <span class="muted">{g.screenings} {g.screenings === 1 ? "show" : "shows"}</span>
            </li>
          {/each}
        </ul>
        <p class="muted small">Average people who came in, by the film's main genre.</p>
      {/if}
    </section>
  </div>

  <section class="card">
    <h2><m3e-icon name="schedule"></m3e-icon>Best start times</h2>
    <Hours values={data.hours.map((h) => Math.round(h.avg * 10) / 10)} unit="people on average" />
  </section>

  <section class="card">
    <h2><m3e-icon name="fact_check"></m3e-icon>Source checks on drafts</h2>
    <p class="muted small top">Apps check whether drafted films have a source. A draft with {data.rule.minChecks}+ checks that finds one less than {Math.round(data.rule.minFound * 100)}% of the time is dropped when the lineup is finalized. Change the rule under Tuning.</p>
    {#if !data.probes.length}
      <div class="empty">No drafts waiting right now.</div>
    {:else}
      {#each data.probes as p (p.id)}
        {@const v = VERDICT[p.verdict]}
        <div class="probe">
          <span class="pname">{p.name}<span class="muted small">{when(p.starts_at)}</span></span>
          <span class="track"><span style:width={pct(p.found, p.found + p.miss)}></span></span>
          <span class="muted small">{p.found} found · {p.miss} missed</span>
          <span class="chip {v[0]}">{v[1]}</span>
        </div>
      {/each}
    {/if}
  </section>

  <section class="card table">
    <h2><m3e-icon name="list"></m3e-icon>Every screening</h2>
    {#if !data.shows.length}
      <div class="empty">No screenings in this period.</div>
    {:else}
      <div class="scroll">
        <table>
          <thead>
            <tr><th>When</th><th>Film</th><th>Seats</th><th>Tickets</th><th>Came in</th><th>No-shows</th><th>Max</th><th>Social</th><th>Started</th></tr>
          </thead>
          <tbody>
            {#each data.shows as s (s.id)}
              <tr class:cancelled={s.status === "cancelled"}>
                <td class="nowrap">{when(s.starts_at)}</td>
                <td>
                  {s.name}
                  {#if s.special}<span class="chip info">Special</span>{/if}
                  {#if s.kind === "pick"}<span class="chip">Pick</span>{/if}
                  {#if s.status !== "scheduled"}<span class="chip">{s.status}</span>{/if}
                </td>
                <td>{s.mode === "fixed" ? s.capacity : "open"}</td>
                <td>{s.booked}{s.waitlist ? ` +${s.waitlist}` : ""}</td>
                <td><b>{s.came_in}</b>{s.expected ? `/${s.expected}` : ""}</td>
                <td>{pct(s.noshow, s.booked)}</td>
                <td>{s.max_tickets}</td>
                <td>{s.social_tickets}</td>
                <td title={Object.entries(s.start_why).filter(([, n]) => n).map(([k, n]) => `${k}: ${n}`).join(", ")}>
                  {s.start_ok + s.start_failed ? `${s.start_ok}/${s.start_ok + s.start_failed}` : "–"}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
      <p class="muted small">Came in is shown against what the algorithm expected. Started is "played / tried" from app reports; hover for why the rest failed.</p>
    {/if}
  </section>
{/if}

<style>
  .bar {
    display: flex;
    justify-content: flex-end;
  }
  .busy {
    opacity: 0.6;
  }
  .stat {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .stat span {
    font-size: 13px;
  }
  .stat strong {
    font-family: var(--display);
    font-weight: 400;
    font-size: 40px;
    line-height: 1.05;
  }
  .stat.warn {
    border-color: color-mix(in srgb, #ff5449 55%, transparent);
  }
  .small {
    font-size: 13px;
    line-height: 1.5;
    margin: 12px 0 0;
  }
  .small.top {
    margin: 0 0 12px;
  }
  .split {
    display: grid;
    gap: 16px;
  }
  .split > div {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 14px;
  }
  .pill {
    height: 14px;
    border-radius: 999px;
    overflow: hidden;
    background: var(--md-sys-color-tertiary, #f4a);
  }
  .pill span {
    display: block;
    height: 100%;
    background: var(--md-sys-color-primary);
  }
  .pill.alt {
    background: #ffc56b;
  }
  .genres {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .genres li,
  .probe {
    display: grid;
    grid-template-columns: minmax(100px, 180px) minmax(0, 1fr) auto auto;
    align-items: center;
    gap: 12px;
    font-size: 14px;
  }
  .probe {
    padding: 8px 0;
    border-top: 1px solid color-mix(in srgb, var(--md-sys-color-outline-variant) 30%, transparent);
  }
  .pname {
    display: flex;
    flex-direction: column;
  }
  .pname .small {
    margin: 0;
  }
  .track {
    height: 10px;
    border-radius: 999px;
    background: var(--md-sys-color-surface-container-highest);
    overflow: hidden;
  }
  .track span {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: var(--md-sys-color-primary);
  }
  .scroll {
    overflow-x: auto;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 14px;
  }
  th {
    text-align: left;
    font-size: 12px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--md-sys-color-on-surface-variant);
    padding: 8px;
  }
  td {
    padding: 8px;
    border-top: 1px solid color-mix(in srgb, var(--md-sys-color-outline-variant) 30%, transparent);
  }
  td .chip {
    margin-left: 6px;
  }
  tr.cancelled td {
    opacity: 0.5;
  }
  .nowrap {
    white-space: nowrap;
  }
</style>
