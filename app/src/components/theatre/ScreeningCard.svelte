<script lang="ts">
  // One screening: when it airs (in your time), seats or "open", people you
  // follow who are going, and what you can do now (book, cancel, enter).
  import Avatar from "../Avatar.svelte";
  import { theatre, type Screening } from "../../lib/theatre.svelte";
  import { clock, dayLabel, runtime, span } from "./time";

  let { s, big = false, onenter }: { s: Screening; big?: boolean; onenter?: (s: Screening) => void } = $props();

  const phase = $derived(theatre.phase(s));
  const ticket = $derived(theatre.ticket(s));
  const opens = $derived(theatre.opensAt(s));
  const going = $derived(theatre.friends[s.id] ?? []);
  const busy = $derived(theatre.busy === s.id);
  const start = $derived(Date.parse(s.starts_at));
  const end = $derived(Date.parse(s.ends_at));
  const left = $derived(theatre.seatsLeft(s));
  const progress = $derived(phase === "live" ? Math.min(1, (theatre.now - start) / (end - start)) : 0);
  const canEnter = $derived(["lobby", "live", "after"].includes(phase) && (s.mode === "open" || ticket === "booked"));

  const when = $derived(
    phase === "upcoming"
      ? `${dayLabel(start, theatre.now)}, ${clock(start)}`
      : phase === "lobby"
        ? `Starts in ${span(start - theatre.now)}`
        : phase === "live"
          ? `${span(theatre.now - start)} in · ${span(end - theatre.now)} left`
          : phase === "after"
            ? `Chat open for ${span(end + 30 * 60_000 - theatre.now)}`
            : "Finished",
  );

  const seats = $derived(
    s.mode === "open" ? "Open · no ticket needed" : left > 0 ? `${left} of ${s.capacity} seats left` : "Sold out · waitlist open",
  );
</script>

<article class="card" class:big class:live={phase === "live"} style:--bg={s.backdrop ? `url(${s.backdrop})` : "none"}>
  {#if big}<span class="art" aria-hidden="true"></span>{/if}
  <img class="poster" src={s.poster ?? ""} alt="" loading="lazy" />

  <div class="info">
    <div class="tags">
      {#if phase === "live"}<span class="tag on-air"><span class="dot"></span>Live</span>
      {:else if phase === "lobby"}<span class="tag lobby">Lobby open</span>
      {:else if phase === "after"}<span class="tag lobby">Just ended</span>{/if}
      {#if s.kind === "pick"}<span class="tag pick"><m3e-icon name="auto_awesome" filled></m3e-icon>Tonight's pick</span>{/if}
    </div>
    <h3>{s.name}{#if s.year}<span class="year"> {s.year}</span>{/if}</h3>
    <p class="when"><m3e-icon name="schedule"></m3e-icon>{when}</p>
    <p class="meta">{runtime(s.runtime)} · {seats}</p>
    {#if phase === "live"}<span class="bar" aria-hidden="true"><span style:width="{progress * 100}%"></span></span>{/if}
    {#if going.length}
      <div class="going">
        <span class="faces">{#each going.slice(0, 3) as f (f.id)}<span class="face" title={f.display_name}><Avatar value={f.avatar} size={22} /></span>{/each}</span>
        <span>{going[0].display_name}{going.length > 1 ? ` and ${going.length - 1} more` : ""} going</span>
      </div>
    {/if}
  </div>

  <div class="actions">
    {#if phase === "over"}
      <span class="muted">Finished</span>
    {:else if canEnter}
      <m3e-button variant="filled" size={big ? "medium" : "small"} onclick={() => onenter?.(s)}>
        <m3e-icon slot="icon" name={phase === "live" ? "play_arrow" : "door_open"} filled></m3e-icon>{phase === "live" ? "Join" : "Enter"}
      </m3e-button>
    {:else if ticket === "booked"}
      <m3e-button variant="tonal" size={big ? "medium" : "small"} disabled>
        <m3e-icon slot="icon" name="confirmation_number" filled></m3e-icon>Booked
      </m3e-button>
      <m3e-icon-button aria-label="Cancel ticket" title="Cancel ticket" disabled={busy} onclick={() => theatre.cancel(s)}><m3e-icon name="close"></m3e-icon></m3e-icon-button>
    {:else if ticket === "waitlist"}
      <m3e-button variant="tonal" size={big ? "medium" : "small"} disabled>
        <m3e-icon slot="icon" name="hourglass_top"></m3e-icon>On the waitlist
      </m3e-button>
      <m3e-icon-button aria-label="Leave the waitlist" title="Leave the waitlist" disabled={busy} onclick={() => theatre.cancel(s)}><m3e-icon name="close"></m3e-icon></m3e-icon-button>
    {:else if opens}
      <m3e-button variant="tonal" size={big ? "medium" : "small"} disabled>
        <m3e-icon slot="icon" name="lock_clock"></m3e-icon>Booking opens {clock(opens)}
      </m3e-button>
    {:else if phase === "upcoming" || s.mode === "fixed"}
      <m3e-button variant="filled" size={big ? "medium" : "small"} disabled={busy} onclick={() => theatre.book(s)}>
        <m3e-icon slot="icon" name="confirmation_number"></m3e-icon>{s.mode === "fixed" && left <= 0 ? "Join waitlist" : "Book"}
      </m3e-button>
    {/if}
  </div>
</article>

<style>
  .card {
    position: relative;
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 12px;
    border-radius: 24px;
    background: var(--md-sys-color-surface-container);
    overflow: hidden;
    isolation: isolate;
  }
  .card.big {
    min-height: 260px;
    align-items: flex-end;
    padding: 24px;
    border-radius: 32px;
  }
  .art {
    position: absolute;
    inset: 0;
    z-index: -1;
    background:
      linear-gradient(90deg, var(--md-sys-color-surface-container) 18%, color-mix(in srgb, var(--md-sys-color-surface-container) 55%, transparent) 60%, transparent),
      linear-gradient(0deg, var(--md-sys-color-surface-container), transparent 60%),
      var(--bg) center / cover no-repeat;
  }
  .poster {
    flex: none;
    width: 64px;
    aspect-ratio: 2 / 3;
    border-radius: 12px;
    object-fit: cover;
    background: var(--md-sys-color-surface-container-highest);
  }
  .big .poster {
    width: 128px;
    border-radius: 18px;
    box-shadow: 0 20px 40px -20px rgb(0 0 0 / 80%);
  }
  .info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .tags:empty {
    display: none;
  }
  .tag {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 2px 10px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 700;
    --m3e-icon-size: 14px;
  }
  .on-air {
    background: var(--md-sys-color-error);
    color: var(--md-sys-color-on-error);
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: currentColor;
    animation: pulse 1.4s ease-in-out infinite;
  }
  .lobby {
    background: var(--md-sys-color-tertiary-container);
    color: var(--md-sys-color-on-tertiary-container);
  }
  .pick {
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
  }
  h3 {
    margin: 0;
    font-size: 17px;
    line-height: 1.25;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .big h3 {
    font-family: var(--display-font, inherit);
    font-size: 34px;
    white-space: normal;
  }
  .year {
    font-weight: 400;
    color: var(--md-sys-color-on-surface-variant);
  }
  .when {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    --m3e-icon-size: 16px;
  }
  .live .when {
    color: var(--md-sys-color-error);
  }
  .meta,
  .muted {
    margin: 0;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .bar {
    display: block;
    height: 4px;
    max-width: 320px;
    border-radius: 4px;
    background: color-mix(in srgb, var(--md-sys-color-on-surface) 14%, transparent);
    overflow: hidden;
  }
  .bar span {
    display: block;
    height: 100%;
    background: var(--md-sys-color-error);
    transition: width 1s linear;
  }
  .going {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .faces {
    display: flex;
  }
  .face {
    display: inline-grid;
    margin-left: -6px;
    border-radius: 50%;
    box-shadow: 0 0 0 2px var(--md-sys-color-surface-container);
  }
  .face:first-child {
    margin-left: 0;
  }
  .actions {
    flex: none;
    display: flex;
    align-items: center;
    gap: 4px;
  }
  @keyframes pulse {
    50% {
      opacity: 0.35;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .dot {
      animation: none;
    }
  }
</style>
