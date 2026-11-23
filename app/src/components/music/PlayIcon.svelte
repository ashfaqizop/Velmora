<script lang="ts">
  // Play and pause as one icon that turns into the other: the outgoing
  // shape shrinks and turns away while the incoming one grows in, on a
  // spring. The button around it changes shape too (see `.morph` users).
  let { playing, size = 24 }: { playing: boolean; size?: number } = $props();
</script>

<span class="pi" class:playing style:--size={`${size}px`} aria-hidden="true">
  <m3e-icon class="a" name="play_arrow" filled></m3e-icon>
  <m3e-icon class="b" name="pause" filled></m3e-icon>
</span>

<style>
  .pi {
    position: relative;
    display: inline-grid;
    width: var(--size);
    height: var(--size);
    --m3e-icon-size: var(--size);
  }
  .pi m3e-icon {
    grid-area: 1 / 1;
    transition:
      transform var(--spring-fast-spatial),
      opacity var(--spring-fast-effects);
  }
  .a {
    opacity: 1;
    transform: none;
  }
  .b {
    opacity: 0;
    transform: scale(0.4) rotate(-90deg);
  }
  .playing .a {
    opacity: 0;
    transform: scale(0.4) rotate(90deg);
  }
  .playing .b {
    opacity: 1;
    transform: none;
  }
</style>
