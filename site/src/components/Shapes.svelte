<script lang="ts">
  import { onMount } from "svelte";
  import { clock, SHAPES, type ShapeSpot } from "../lib/morph.svelte";

  // Decorative Material 3 Expressive shapes: each one spins, floats and
  // morphs into the next shape every few seconds. Purely visual.

  let { spots }: { spots: ShapeSpot[] } = $props();

  onMount(() => {
    clock.start();
    return () => clock.stop();
  });
</script>

<div class="shapes" aria-hidden="true">
  {#each spots as s, i (i)}
    <span
      class="spot"
      style:left={s.x}
      style:top={s.y}
      style:--size="{s.size}px"
      style:--color={s.color}
      style:--spin="{Math.abs(s.spin ?? 18)}s"
      style:--dir={(s.spin ?? 18) < 0 ? "reverse" : "normal"}
      style:--bob="{6 + (i % 4)}s"
      style:opacity={s.opacity ?? 0.9}
      style:filter={s.blur ? `blur(${s.blur}px)` : undefined}
    >
      <span class="turn">
        <m3e-shape name={SHAPES[(clock.tick + (s.offset ?? i * 3)) % SHAPES.length]}></m3e-shape>
      </span>
    </span>
  {/each}
</div>

<style>
  .shapes {
    position: absolute;
    inset: 0;
    z-index: -1;
    pointer-events: none;
    overflow: visible;
  }
  .spot {
    position: absolute;
    width: var(--size);
    height: var(--size);
    translate: -50% -50%;
    animation: bob var(--bob) ease-in-out infinite alternate;
  }
  .turn {
    display: block;
    width: 100%;
    height: 100%;
    animation: spin var(--spin) linear infinite;
    animation-direction: var(--dir);
  }
  m3e-shape {
    --m3e-shape-size: var(--size);
    --m3e-shape-container-color: var(--color);
    --m3e-shape-transition: clip-path 1.1s cubic-bezier(0.34, 1.4, 0.5, 1);
    filter: drop-shadow(0 0 28px color-mix(in srgb, var(--color) 55%, transparent));
  }
  /* Smaller on phones, so they frame the content instead of covering it. */
  @media (max-width: 600px) {
    .spot {
      scale: 0.55;
    }
  }
  @keyframes spin {
    to {
      rotate: 360deg;
    }
  }
  @keyframes bob {
    from {
      transform: translateY(-10px);
    }
    to {
      transform: translateY(12px);
    }
  }
</style>
