<script lang="ts">
  // The parametric equalizer: ten bands on a frequency curve. Drag a point to
  // move it (left/right is frequency, up/down is gain), scroll on it to make it
  // narrower or wider, double-click to flatten it. Arrow keys work on a
  // focused point. The curve is computed from the same RBJ biquads mpv's
  // filters use, so what you see is what you hear.
  import { M3eSnackbar } from "@m3e/web/snackbar";
  import { EQ_PRESETS, EQ_RANGE, eqPreamp, musicPlayer, type EqBand } from "../../lib/music/player.svelte";

  const W = 720;
  const H = 240;
  const PAD = { l: 34, r: 12, t: 12, b: 24 };
  const F_MIN = 20;
  const F_MAX = 20000;
  const FS = 48000;

  const x = (f: number) => PAD.l + ((Math.log10(f) - Math.log10(F_MIN)) / (Math.log10(F_MAX) - Math.log10(F_MIN))) * (W - PAD.l - PAD.r);
  const fx = (px: number) => 10 ** (Math.log10(F_MIN) + ((px - PAD.l) / (W - PAD.l - PAD.r)) * (Math.log10(F_MAX) - Math.log10(F_MIN)));
  const y = (db: number) => PAD.t + ((EQ_RANGE - db) / (EQ_RANGE * 2)) * (H - PAD.t - PAD.b);
  const gy = (py: number) => EQ_RANGE - ((py - PAD.t) / (H - PAD.t - PAD.b)) * EQ_RANGE * 2;

  /** dB of one RBJ biquad at frequency f. */
  function bandDb(b: EqBand, kind: "low" | "peak" | "high", f: number): number {
    if (Math.abs(b.g) < 0.01) return 0;
    const A = 10 ** (b.g / 40);
    const w0 = (2 * Math.PI * b.f) / FS;
    const cos = Math.cos(w0);
    const alpha = Math.sin(w0) / (2 * b.q);
    let b0: number, b1: number, b2: number, a0: number, a1: number, a2: number;
    if (kind === "peak") {
      b0 = 1 + alpha * A;
      b1 = -2 * cos;
      b2 = 1 - alpha * A;
      a0 = 1 + alpha / A;
      a1 = -2 * cos;
      a2 = 1 - alpha / A;
    } else {
      const s = 2 * Math.sqrt(A) * alpha;
      if (kind === "low") {
        b0 = A * (A + 1 - (A - 1) * cos + s);
        b1 = 2 * A * (A - 1 - (A + 1) * cos);
        b2 = A * (A + 1 - (A - 1) * cos - s);
        a0 = A + 1 + (A - 1) * cos + s;
        a1 = -2 * (A - 1 + (A + 1) * cos);
        a2 = A + 1 + (A - 1) * cos - s;
      } else {
        b0 = A * (A + 1 + (A - 1) * cos + s);
        b1 = -2 * A * (A - 1 + (A + 1) * cos);
        b2 = A * (A + 1 + (A - 1) * cos - s);
        a0 = A + 1 - (A - 1) * cos + s;
        a1 = 2 * (A - 1 - (A + 1) * cos);
        a2 = A + 1 - (A - 1) * cos - s;
      }
    }
    const w = (2 * Math.PI * f) / FS;
    const c1 = Math.cos(w);
    const s1 = Math.sin(w);
    const c2 = Math.cos(2 * w);
    const s2 = Math.sin(2 * w);
    const nr = b0 + b1 * c1 + b2 * c2;
    const ni = -(b1 * s1 + b2 * s2);
    const dr = a0 + a1 * c1 + a2 * c2;
    const di = -(a1 * s1 + a2 * s2);
    return 10 * Math.log10((nr * nr + ni * ni) / (dr * dr + di * di));
  }

  const eq = $derived(musicPlayer.eqState);
  const kindOf = (i: number) => (i === 0 ? "low" : i === eq.bands.length - 1 ? "high" : "peak");
  const pre = $derived(eqPreamp(eq));

  /** The whole curve, preamp included. */
  const curve = $derived.by(() => {
    const pts: string[] = [];
    for (let i = 0; i <= 160; i++) {
      const f = 10 ** (Math.log10(F_MIN) + (i / 160) * (Math.log10(F_MAX) - Math.log10(F_MIN)));
      let db = pre;
      eq.bands.forEach((b, j) => (db += bandDb(b, kindOf(j), f)));
      db = Math.max(-EQ_RANGE - 3, Math.min(EQ_RANGE + 3, db));
      pts.push(`${x(f).toFixed(1)},${y(db).toFixed(1)}`);
    }
    return pts.join(" ");
  });

  let svg = $state<SVGSVGElement>();
  let dragging = $state<number | null>(null);
  let focused = $state<number | null>(null);

  /** Pointer position in the SVG's own units. */
  function local(e: PointerEvent): [number, number] {
    const r = svg!.getBoundingClientRect();
    return [((e.clientX - r.left) / r.width) * W, ((e.clientY - r.top) / r.height) * H];
  }

  function down(e: PointerEvent, i: number) {
    dragging = i;
    focused = i;
    (e.currentTarget as Element).setPointerCapture(e.pointerId);
  }

  function move(e: PointerEvent, i: number) {
    if (dragging !== i) return;
    const [px, py] = local(e);
    musicPlayer.setBand(i, {
      f: Math.round(Math.min(F_MAX, Math.max(F_MIN, fx(px)))),
      g: Math.round(Math.min(EQ_RANGE, Math.max(-EQ_RANGE, gy(py))) * 2) / 2,
    });
  }

  function wheel(e: WheelEvent, i: number) {
    e.preventDefault();
    const q = eq.bands[i].q * (e.deltaY > 0 ? 1 / 1.12 : 1.12);
    musicPlayer.setBand(i, { q: Math.round(q * 100) / 100 });
  }

  function key(e: KeyboardEvent, i: number) {
    const b = eq.bands[i];
    const step: Record<string, Partial<EqBand>> = {
      ArrowUp: { g: Math.min(EQ_RANGE, b.g + 0.5) },
      ArrowDown: { g: Math.max(-EQ_RANGE, b.g - 0.5) },
      ArrowRight: { f: Math.min(F_MAX, Math.round(b.f * 1.06)) },
      ArrowLeft: { f: Math.max(F_MIN, Math.round(b.f / 1.06)) },
      PageUp: { q: Math.min(8, b.q * 1.12) },
      PageDown: { q: Math.max(0.2, b.q / 1.12) },
      "0": { g: 0 },
    };
    if (step[e.key]) {
      e.preventDefault();
      musicPlayer.setBand(i, step[e.key]);
    }
  }

  const hz = (f: number) => (f >= 1000 ? `${(f / 1000).toFixed(f >= 10000 ? 0 : 1).replace(/\.0$/, "")}k` : `${Math.round(f)}`);
  const sel = $derived(focused ?? dragging);

  let naming = $state(false);
  let name = $state("");
  function save() {
    if (!name.trim()) return;
    musicPlayer.saveEqAs(name);
    M3eSnackbar.open(`Saved "${name.trim()}"`);
    naming = false;
    name = "";
  }
</script>

<div class="eq-editor">
  <div class="presets" role="radiogroup" aria-label="Equalizer curves">
    {#each Object.entries(EQ_PRESETS) as [id, p] (id)}
      <button class="chip" class:on={eq.key === id} role="radio" aria-checked={eq.key === id} onclick={() => musicPlayer.setEq(id)}>{p.label}</button>
    {/each}
    {#each eq.saved as s (s.name)}
      <span class="chip saved" class:on={eq.key === `user:${s.name}`}>
        <button role="radio" aria-checked={eq.key === `user:${s.name}`} onclick={() => musicPlayer.setEq(`user:${s.name}`)}>{s.name}</button>
        <button class="x" aria-label="Delete {s.name}" onclick={() => musicPlayer.deleteEq(s.name)}><m3e-icon name="close"></m3e-icon></button>
      </span>
    {/each}
    {#if eq.key === "custom"}<span class="chip on ghost">Custom</span>{/if}
  </div>

  <svg bind:this={svg} viewBox="0 0 {W} {H}" class="graph" role="group" aria-label="Equalizer curve">
    {#each [-12, -6, 0, 6, 12] as db (db)}
      <line x1={PAD.l} x2={W - PAD.r} y1={y(db)} y2={y(db)} class:zero={db === 0} class="grid" />
      <text x={PAD.l - 6} y={y(db) + 4} class="axis" text-anchor="end">{db > 0 ? `+${db}` : db}</text>
    {/each}
    {#each [50, 100, 200, 500, 1000, 2000, 5000, 10000] as f (f)}
      <line x1={x(f)} x2={x(f)} y1={PAD.t} y2={H - PAD.b} class="grid v" />
      <text x={x(f)} y={H - 6} class="axis" text-anchor="middle">{hz(f)}</text>
    {/each}
    <polyline points={`${x(F_MIN)},${y(0)} ${curve} ${x(F_MAX)},${y(0)}`} class="fill" />
    <polyline points={curve} class="curve" />
    {#each eq.bands as b, i (i)}
      <g
        class="band"
        class:active={sel === i}
        transform="translate({x(b.f)} {y(b.g)})"
        role="slider"
        tabindex="0"
        aria-label="{hz(b.f)} Hz band"
        aria-valuemin={-EQ_RANGE}
        aria-valuemax={EQ_RANGE}
        aria-valuenow={b.g}
        aria-valuetext="{b.g > 0 ? '+' : ''}{b.g} dB at {hz(b.f)} Hz"
        onpointerdown={(e) => down(e, i)}
        onpointermove={(e) => move(e, i)}
        onpointerup={() => (dragging = null)}
        onpointercancel={() => (dragging = null)}
        onwheel={(e) => wheel(e, i)}
        ondblclick={() => musicPlayer.setBand(i, { g: 0 })}
        onkeydown={(e) => key(e, i)}
        onfocus={() => (focused = i)}
        onblur={() => (focused = null)}
      >
        <circle r="14" class="hit" />
        <circle r={sel === i ? 8 : 6} class="dot" />
      </g>
    {/each}
  </svg>

  <div class="readout">
    {#if sel !== null && eq.bands[sel]}
      {@const b = eq.bands[sel]}
      <span><strong>{hz(b.f)} Hz</strong> · {b.g > 0 ? "+" : ""}{b.g.toFixed(1)} dB · width {b.q.toFixed(2)}{sel === 0 ? " · low shelf" : sel === eq.bands.length - 1 ? " · high shelf" : ""}</span>
    {:else}
      <span>Drag a point to shape the sound. Scroll on it for width, double-click to flatten.</span>
    {/if}
  </div>

  <div class="tools">
    <label class="pre">
      <span>Preamp {pre > 0 ? "+" : ""}{pre.toFixed(1)} dB</span>
      <input
        type="range"
        min="-12"
        max="12"
        step="0.5"
        value={eq.preamp}
        aria-label="Preamp"
        oninput={(e) => musicPlayer.setPreamp(Number((e.currentTarget as HTMLInputElement).value))}
      />
    </label>
    <label class="auto">
      <m3e-switch
        checked={eq.autoPreamp}
        onchange={(e: Event) => musicPlayer.setPreamp(eq.preamp, (e.currentTarget as HTMLElement & { checked: boolean }).checked)}
      ></m3e-switch>
      <span>Prevent clipping</span>
    </label>
    <span class="grow"></span>
    {#if naming}
      <form class="name" onsubmit={(e) => (e.preventDefault(), save())}>
        <!-- svelte-ignore a11y_autofocus -->
        <input bind:value={name} placeholder="Name" maxlength="40" autofocus aria-label="Curve name" />
        <m3e-button variant="filled" size="small" disabled={!name.trim()} onclick={save}>Save</m3e-button>
        <m3e-button variant="text" size="small" onclick={() => (naming = false)}>Cancel</m3e-button>
      </form>
    {:else}
      <m3e-button variant="tonal" onclick={() => (naming = true)}>
        <m3e-icon slot="icon" name="bookmark_add"></m3e-icon>Save curve
      </m3e-button>
      <m3e-button variant="text" disabled={eq.key === "flat"} onclick={() => musicPlayer.setEq("flat")}>Reset</m3e-button>
    {/if}
  </div>
</div>

<style>
  .eq-editor {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 6px 4px;
  }
  .presets {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .chip {
    all: unset;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    height: 32px;
    padding: 0 14px;
    border-radius: 16px;
    font-size: 13.5px;
    font-weight: 600;
    background: var(--md-sys-color-surface-container-highest);
    color: var(--md-sys-color-on-surface);
  }
  .chip.on {
    background: var(--md-sys-color-primary);
    color: var(--md-sys-color-on-primary);
  }
  .chip.ghost {
    cursor: default;
  }
  .chip.saved {
    padding: 0 4px 0 0;
  }
  .chip.saved button {
    all: unset;
    cursor: pointer;
    padding: 0 6px 0 14px;
    height: 32px;
  }
  .chip.saved .x {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    padding: 0;
    border-radius: 50%;
    --m3e-icon-size: 16px;
  }
  .graph {
    width: 100%;
    height: auto;
    border-radius: 16px;
    background: var(--md-sys-color-surface-container-lowest);
    touch-action: none;
    user-select: none;
  }
  .grid {
    stroke: var(--md-sys-color-outline-variant);
    stroke-width: 1;
    opacity: 0.5;
  }
  .grid.zero {
    opacity: 1;
  }
  .axis {
    font-size: 10px;
    fill: var(--md-sys-color-on-surface-variant);
  }
  .fill {
    fill: color-mix(in srgb, var(--md-sys-color-primary) 16%, transparent);
    stroke: none;
  }
  .curve {
    fill: none;
    stroke: var(--md-sys-color-primary);
    stroke-width: 2.5;
    stroke-linejoin: round;
  }
  .band {
    cursor: grab;
    outline: none;
  }
  .band:active {
    cursor: grabbing;
  }
  .hit {
    fill: transparent;
  }
  .dot {
    fill: var(--md-sys-color-surface);
    stroke: var(--md-sys-color-primary);
    stroke-width: 2.5;
    transition: r 120ms;
  }
  .band.active .dot,
  .band:focus-visible .dot {
    fill: var(--md-sys-color-primary);
  }
  .readout {
    font-size: 13px;
    color: var(--md-sys-color-on-surface-variant);
    min-height: 18px;
  }
  .readout strong {
    color: var(--md-sys-color-on-surface);
  }
  .tools {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px 18px;
  }
  .pre {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13.5px;
    font-weight: 600;
  }
  .pre input {
    width: 160px;
    accent-color: var(--md-sys-color-primary);
  }
  .auto {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13.5px;
    cursor: pointer;
  }
  .grow {
    flex: 1;
  }
  .name {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .name input {
    height: 34px;
    width: 160px;
    padding: 0 12px;
    border-radius: 17px;
    border: 1px solid var(--md-sys-color-outline);
    background: transparent;
    color: var(--md-sys-color-on-surface);
    font: inherit;
  }
</style>
