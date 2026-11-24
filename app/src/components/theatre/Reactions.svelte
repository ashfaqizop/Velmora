<script lang="ts">
  // Reactions float up over the movie. Many of the same at once grow bigger.
  import { onDestroy, onMount } from "svelte";
  import { room, type Reaction } from "../../lib/room.svelte";

  interface Particle {
    id: number;
    e: Reaction;
    x: number;
    drift: number;
    size: number;
    dur: number;
  }

  let particles = $state<Particle[]>([]);
  let seq = 0;
  const recent: { e: Reaction; t: number }[] = [];

  function spawn(e: Reaction) {
    const now = Date.now();
    recent.push({ e, t: now });
    while (recent.length && now - recent[0].t > 1500) recent.shift();
    const same = recent.filter((r) => r.e === e).length;
    const size = Math.min(1.9, 1 + (same - 1) * 0.12);
    const p: Particle = { id: ++seq, e, x: Math.random() * 100, drift: (Math.random() - 0.5) * 80, size, dur: 2400 + Math.random() * 900 };
    particles = [...particles.slice(-60), p];
    setTimeout(() => (particles = particles.filter((q) => q.id !== p.id)), p.dur + 50);
  }

  onMount(() => {
    room.onReact = (e) => spawn(e);
  });
  onDestroy(() => (room.onReact = null));
</script>

<div class="layer" aria-hidden="true">
  {#each particles as p (p.id)}
    <span class="p" style:left="{p.x}%" style:--drift="{p.drift}px" style:--size={p.size} style:--dur="{p.dur}ms">{p.e}</span>
  {/each}
</div>

<style>
  .layer {
    position: absolute;
    right: 24px;
    bottom: 110px;
    width: 160px;
    height: 60vh;
    pointer-events: none;
    z-index: 74;
    overflow: visible;
  }
  .p {
    position: absolute;
    bottom: 0;
    font-size: calc(30px * var(--size));
    line-height: 1;
    filter: drop-shadow(0 4px 10px rgb(0 0 0 / 50%));
    animation: rise var(--dur) cubic-bezier(0.2, 0.7, 0.3, 1) forwards;
  }
  @keyframes rise {
    0% {
      transform: translate(0, 0) scale(0.4);
      opacity: 0;
    }
    12% {
      transform: translate(calc(var(--drift) * 0.15), -6vh) scale(1.1);
      opacity: 1;
    }
    100% {
      transform: translate(var(--drift), -58vh) scale(0.9);
      opacity: 0;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .p {
      animation-duration: 1ms;
    }
  }
</style>
