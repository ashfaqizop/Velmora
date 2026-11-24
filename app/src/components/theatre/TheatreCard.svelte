<script lang="ts">
  // A screening on Home's "Tonight in Theatre" row. Opens the Theatre page.
  import { nav } from "../../lib/state.svelte";
  import { theatre, type Screening } from "../../lib/theatre.svelte";
  import { clock, dayLabel, span } from "./time";

  let { s }: { s: Screening } = $props();

  const phase = $derived(theatre.phase(s));
  const ticket = $derived(theatre.ticket(s));
  const start = $derived(Date.parse(s.starts_at));
</script>

<button type="button" class="card" style:--bg={s.backdrop ? `url(${s.backdrop})` : "none"} onclick={() => nav.go({ name: "theatre" })}>
  <span class="art" aria-hidden="true"></span>
  <span class="top">
    {#if phase === "live"}<span class="chip live"><span class="dot"></span>Live</span>
    {:else if phase === "lobby"}<span class="chip lobby">Lobby open · {span(start - theatre.now)}</span>
    {:else}<span class="chip">{dayLabel(start, theatre.now)} · {clock(start)}</span>{/if}
    {#if ticket === "booked"}<span class="chip mine"><m3e-icon name="confirmation_number" filled></m3e-icon>Booked</span>{/if}
  </span>
  <span class="bottom">
    {#if s.kind === "pick"}<span class="pick">Tonight's pick</span>{/if}
    <strong>{s.name}</strong>
  </span>
</button>

<style>
  .card {
    all: unset;
    position: relative;
    flex: none;
    width: 300px;
    aspect-ratio: 16 / 9;
    border-radius: 24px;
    overflow: hidden;
    cursor: pointer;
    isolation: isolate;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: 14px;
    box-sizing: border-box;
    transition: transform 260ms cubic-bezier(0.2, 0.8, 0.2, 1);
  }
  .card:hover {
    transform: translateY(-3px);
  }
  .card:focus-visible {
    outline: 3px solid var(--md-sys-color-primary);
    outline-offset: 2px;
  }
  .art {
    position: absolute;
    inset: 0;
    z-index: -1;
    background:
      linear-gradient(0deg, rgb(0 0 0 / 85%), transparent 65%),
      var(--bg) center / cover no-repeat,
      var(--md-sys-color-surface-container-high);
    transition: transform 500ms cubic-bezier(0.2, 0.8, 0.2, 1);
  }
  .card:hover .art {
    transform: scale(1.04);
  }
  .top {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 3px 10px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 700;
    background: rgb(0 0 0 / 55%);
    color: #fff;
    backdrop-filter: blur(8px);
    --m3e-icon-size: 14px;
  }
  .chip.live {
    background: var(--md-sys-color-error);
    color: var(--md-sys-color-on-error);
  }
  .chip.lobby {
    background: var(--md-sys-color-tertiary-container);
    color: var(--md-sys-color-on-tertiary-container);
  }
  .chip.mine {
    background: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: currentColor;
    animation: pulse 1.4s ease-in-out infinite;
  }
  .bottom {
    display: flex;
    flex-direction: column;
    gap: 2px;
    color: #fff;
  }
  .bottom strong {
    font-size: 18px;
    line-height: 1.2;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .pick {
    font-size: 12px;
    font-weight: 700;
    color: #f8e8c8;
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
    .card,
    .art {
      transition: none;
    }
  }
</style>
