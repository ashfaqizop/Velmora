<script lang="ts">
  // A ring of bars around the cover that moves with the music. The levels are
  // real: mpv measures the loudness of each channel (an astats filter the full
  // view switches on) and the ring plays back the last moments of it, left
  // channel on one side, right on the other.
  import { onMount } from "svelte";
  import { musicPlayer } from "../../lib/music/player.svelte";
  import { reducedMotion } from "../../lib/motion";

  let { size = 520, color = "#b69cff" }: { size?: number; color?: string } = $props();

  let canvas = $state<HTMLCanvasElement>();
  const BARS = 72;

  onMount(() => {
    musicPlayer.setVisualizer(true);
    const still = reducedMotion();
    const half = BARS / 2;
    const left = new Float32Array(half);
    const right = new Float32Array(half);
    let frame = 0;
    let lastPush = 0;
    let smooth = 0;

    const draw = (now: number) => {
      const ctx = canvas?.getContext("2d");
      if (ctx && canvas) {
        const dpr = devicePixelRatio || 1;
        const px = size * dpr;
        if (canvas.width !== px) {
          canvas.width = px;
          canvas.height = px;
        }
        const m = musicPlayer.meter;
        const live = musicPlayer.playing && now - m.at < 400;
        // Push a new sample about 30 times a second; the ring scrolls through history.
        if (now - lastPush > 33) {
          lastPush = now;
          left.copyWithin(1, 0);
          right.copyWithin(1, 0);
          left[0] = live ? m.left : left[0] * 0.85;
          right[0] = live ? m.right : right[0] * 0.85;
        }
        smooth += ((live ? (m.left + m.right) / 2 : 0) - smooth) * 0.2;

        ctx.clearRect(0, 0, px, px);
        const cx = px / 2;
        const inner = px * 0.36;
        const span = px * 0.12;
        ctx.lineCap = "round";
        ctx.lineWidth = Math.max(2, px / 140);
        for (let i = 0; i < BARS; i++) {
          const side = i < half ? left : right;
          const k = i < half ? i : BARS - 1 - i;
          const v = still ? 0.15 : Math.pow(side[k] ?? 0, 1.6);
          const a = (i / BARS) * Math.PI * 2 - Math.PI / 2;
          const len = 4 * dpr + v * span * (0.6 + smooth);
          const r0 = inner + smooth * 6 * dpr;
          ctx.strokeStyle = color;
          ctx.globalAlpha = 0.25 + v * 0.75;
          ctx.beginPath();
          ctx.moveTo(cx + Math.cos(a) * r0, cx + Math.sin(a) * r0);
          ctx.lineTo(cx + Math.cos(a) * (r0 + len), cx + Math.sin(a) * (r0 + len));
          ctx.stroke();
        }
        ctx.globalAlpha = 1;
      }
      frame = requestAnimationFrame(draw);
    };
    frame = requestAnimationFrame(draw);
    return () => {
      cancelAnimationFrame(frame);
      musicPlayer.setVisualizer(false);
    };
  });
</script>

<canvas bind:this={canvas} class="viz" style:width={`${size}px`} style:height={`${size}px`} aria-hidden="true"></canvas>

<style>
  .viz {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    pointer-events: none;
  }
</style>
