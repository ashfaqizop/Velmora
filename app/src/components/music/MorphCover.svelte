<script lang="ts">
  // The cover in a living shape. While music plays it's a soft cookie that
  // slowly turns and breathes with the music; paused, it settles into a
  // rounded square. Drawn as an SVG clip path updated each frame, and only
  // while something is changing.
  import { onMount } from "svelte";
  import { reducedMotion } from "../../lib/motion";
  import { musicPlayer } from "../../lib/music/player.svelte";

  let {
    src,
    size = 56,
    playing = false,
    lobes = 9,
    react = false,
  }: { src: string | null; size?: number; playing?: boolean; lobes?: number; react?: boolean } = $props();

  const id = `morph-${Math.random().toString(36).slice(2)}`;
  let path = $state("");

  // Current shape, eased toward the target each frame.
  let amp = 0; // lobe depth
  let square = 1; // 1 = rounded square, 0 = circle
  let spin = 0;
  let pulse = 0;

  function shape(): string {
    const pts: string[] = [];
    const n = 2 + square * 3; // superellipse exponent: 2 circle .. 5 squircle
    const steps = 120;
    for (let i = 0; i <= steps; i++) {
      const t = (i / steps) * Math.PI * 2;
      const c = Math.cos(t);
      const s = Math.sin(t);
      const sq = 1 / Math.pow(Math.pow(Math.abs(c), n) + Math.pow(Math.abs(s), n), 1 / n);
      const r = 0.92 * sq * (1 + amp * Math.cos(lobes * t + spin)) * (1 + pulse * 0.05);
      pts.push(`${(c * r).toFixed(4)} ${(s * r).toFixed(4)}`);
    }
    return `M${pts.join("L")}Z`;
  }

  onMount(() => {
    let frame = 0;
    let last = performance.now();
    const still = reducedMotion();
    const tick = (now: number) => {
      const dt = Math.min(64, now - last) / 1000;
      last = now;
      const on = playing && !still;
      const k = 1 - Math.pow(0.002, dt); // smooth easing per second
      amp += ((on ? 0.055 : 0) - amp) * k;
      square += ((on ? 0 : 1) - square) * k;
      if (on) spin += dt * 0.35;
      if (react && on) {
        const m = musicPlayer.meter;
        const fresh = now - m.at < 300;
        const target = fresh ? Math.max(0, (m.left + m.right) / 2 - 0.35) * 2.2 : 0;
        pulse += (target - pulse) * Math.min(1, dt * 14);
      } else pulse *= 0.9;
      path = shape();
      const settled = !on && Math.abs(amp) < 0.0005 && Math.abs(1 - square) < 0.0005 && pulse < 0.001;
      frame = settled ? 0 : requestAnimationFrame(tick);
    };
    const start = () => {
      if (!frame) {
        last = performance.now();
        frame = requestAnimationFrame(tick);
      }
    };
    path = shape();
    start();
    // Wake the loop when playback starts again.
    const wake = setInterval(() => playing && start(), 250);
    return () => {
      cancelAnimationFrame(frame);
      clearInterval(wake);
    };
  });
</script>

<svg class="morph" width={size} height={size} viewBox="-1 -1 2 2" aria-hidden="true">
  <defs>
    <clipPath id={id}><path d={path} /></clipPath>
  </defs>
  <g clip-path={`url(#${id})`}>
    <rect x="-1" y="-1" width="2" height="2" class="bg" />
    {#if src}<image href={src} x="-1" y="-1" width="2" height="2" preserveAspectRatio="xMidYMid slice" />{/if}
  </g>
</svg>

<style>
  .morph {
    display: block;
    flex: none;
    overflow: visible;
  }
  .bg {
    fill: var(--md-sys-color-surface-container-highest);
  }
</style>
