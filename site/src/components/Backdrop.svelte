<script lang="ts">
  // Slow drifting color and a light grain behind every page.
  // "engine" swaps the violet for UniverseAPI's crimson.
  let { tone = "brand" }: { tone?: "brand" | "engine" } = $props();
</script>

<div class="aurora" class:engine={tone === "engine"} aria-hidden="true">
  <span class="blob b1"></span>
  <span class="blob b2"></span>
  <span class="blob b3"></span>
</div>
<div class="grain" aria-hidden="true"></div>

<style>
  .aurora {
    position: fixed;
    inset: 0;
    z-index: -2;
    overflow: hidden;
    pointer-events: none;
  }
  .blob {
    position: absolute;
    border-radius: 50%;
    filter: blur(110px);
    opacity: 0.55;
    animation: drift 26s ease-in-out infinite alternate;
  }
  .b1 {
    width: 52vw;
    height: 52vw;
    left: -12vw;
    top: -18vw;
    background: #4b2fd6;
  }
  .b2 {
    width: 42vw;
    height: 42vw;
    right: -10vw;
    top: -6vw;
    background: #ff5a5a;
    opacity: 0.32;
    animation-duration: 32s;
    animation-delay: -8s;
  }
  .b3 {
    width: 46vw;
    height: 46vw;
    left: 30vw;
    top: 40vh;
    background: #7b6cff;
    opacity: 0.22;
    animation-duration: 38s;
    animation-delay: -14s;
  }
  .engine .b1 {
    background: #8a0d16;
  }
  .engine .b2 {
    background: #ff3b47;
    opacity: 0.22;
  }
  .engine .b3 {
    background: #5b1a3a;
    opacity: 0.3;
  }
  @keyframes drift {
    to {
      transform: translate(6vw, 8vh) scale(1.15);
    }
  }
  .grain {
    position: fixed;
    inset: 0;
    z-index: -1;
    pointer-events: none;
    opacity: 0.07;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='160' height='160'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='2' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E");
  }

</style>
