<script lang="ts">
  // The full player's seek bar, the Material 3 Expressive way: the played
  // part is a wave that moves while the song plays and flattens out when it
  // pauses; the rest is a flat track. A slider in every other way: drag or
  // click to seek, arrow keys for 5 s (Page Up/Down 30 s, Home/End).
  import { onDestroy } from "svelte";
  import { motion } from "../../lib/motion.svelte";

  let {
    value,
    max,
    playing,
    color = "#fff",
    label = "Seek",
    oninput,
    onchange,
  }: {
    value: number;
    max: number;
    playing: boolean;
    color?: string;
    label?: string;
    /** While dragging: where the thumb is. */
    oninput?: (v: number) => void;
    /** On release: where to seek. */
    onchange?: (v: number) => void;
  } = $props();

  const H = 28;
  const MID = H / 2;
  const WAVELENGTH = 40;
  const AMPLITUDE = 3;
  /** Space between the played part, the thumb and the rest. */
  const GAP = 6;

  let width = $state(0);
  let dragging = $state<number | null>(null);
  let el = $state<HTMLElement>();
  /** 0..1 of the wave's height; eases to 0 on pause. */
  let amp = 0;
  let phase = 0;
  let path = $state("");
  let frame = 0;
  let last = 0;

  const frac = $derived(max > 0 ? Math.min(1, Math.max(0, (dragging ?? value) / max)) : 0);
  const x = $derived(frac * width);

  /** The played part's wave, from 0 to `end`. */
  function wave(end: number, a: number): string {
    if (end <= 1) return "";
    if (a < 0.02) return `M0 ${MID} L${end.toFixed(1)} ${MID}`;
    const k = (2 * Math.PI) / WAVELENGTH;
    let d = `M0 ${(MID + Math.sin(phase) * AMPLITUDE * a).toFixed(2)}`;
    for (let px = 2; px <= end; px += 2) d += ` L${px} ${(MID + Math.sin(px * k + phase) * AMPLITUDE * a).toFixed(2)}`;
    return d;
  }

  function tick(now: number) {
    const dt = Math.min(0.05, (now - (last || now)) / 1000);
    last = now;
    const moving = playing && dragging === null && motion.level === "full";
    // The wave grows in and settles flat on its own timescale.
    amp += ((moving ? 1 : 0) - amp) * Math.min(1, dt * 7);
    if (moving) phase -= dt * 5.5;
    path = wave(Math.max(0, x - GAP), amp);
    if (amp > 0.01 || moving) frame = requestAnimationFrame(tick);
    else {
      frame = 0;
      last = 0;
    }
  }

  // Run the animation while it has something to do; otherwise draw once.
  $effect(() => {
    void [playing, x, dragging, motion.level];
    if (!frame) frame = requestAnimationFrame(tick);
  });
  onDestroy(() => cancelAnimationFrame(frame));

  function at(e: PointerEvent): number {
    const r = el!.getBoundingClientRect();
    return Math.round(Math.min(1, Math.max(0, (e.clientX - r.left) / r.width)) * max);
  }

  function down(e: PointerEvent) {
    if (!max) return;
    el!.setPointerCapture(e.pointerId);
    dragging = at(e);
    oninput?.(dragging);
  }
  function move(e: PointerEvent) {
    if (dragging === null) return;
    dragging = at(e);
    oninput?.(dragging);
  }
  function up(e: PointerEvent) {
    if (dragging === null) return;
    const v = at(e);
    dragging = null;
    onchange?.(v);
  }

  function key(e: KeyboardEvent) {
    const step = { ArrowRight: 5, ArrowUp: 5, ArrowLeft: -5, ArrowDown: -5, PageUp: 30, PageDown: -30 }[e.key];
    let v: number | null = null;
    if (step !== undefined) v = value + step;
    else if (e.key === "Home") v = 0;
    else if (e.key === "End") v = max;
    if (v === null) return;
    e.preventDefault();
    e.stopPropagation();
    onchange?.(Math.min(max, Math.max(0, v)));
  }
</script>

<div
  class="seek"
  class:dragging={dragging !== null}
  bind:this={el}
  bind:clientWidth={width}
  role="slider"
  tabindex="0"
  aria-label={label}
  aria-valuemin={0}
  aria-valuemax={Math.round(max)}
  aria-valuenow={Math.round(dragging ?? value)}
  style:--c={color}
  onpointerdown={down}
  onpointermove={move}
  onpointerup={up}
  onpointercancel={() => (dragging = null)}
  onkeydown={key}
>
  <svg width={width} height={H} viewBox={`0 0 ${Math.max(1, width)} ${H}`} aria-hidden="true">
    {#if x + GAP < width}
      <line class="rest" x1={Math.min(width, x + GAP)} y1={MID} x2={width - 2} y2={MID} />
      <circle class="end" cx={width - 2} cy={MID} r="2" />
    {/if}
    <path class="played" d={path} />
  </svg>
  <span class="thumb" style:transform={`translateX(${x}px)`}></span>
</div>

<style>
  .seek {
    position: relative;
    flex: 1;
    height: 28px;
    cursor: pointer;
    touch-action: none;
    outline: none;
  }
  svg {
    display: block;
    overflow: visible;
  }
  .played {
    fill: none;
    stroke: var(--c);
    stroke-width: 4;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .rest {
    stroke: color-mix(in srgb, var(--c) 28%, transparent);
    stroke-width: 4;
    stroke-linecap: round;
  }
  .end {
    fill: var(--c);
  }
  .thumb {
    position: absolute;
    left: -2px;
    top: 2px;
    width: 4px;
    height: 24px;
    border-radius: 2px;
    background: var(--c);
    transition: height var(--spring-fast-spatial), top var(--spring-fast-spatial);
  }
  .seek:hover .thumb,
  .seek.dragging .thumb {
    top: -1px;
    height: 30px;
  }
  .seek:focus-visible .thumb {
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--c) 45%, transparent);
  }
</style>
