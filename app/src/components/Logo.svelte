<script lang="ts">
  import wordmark from "../assets/velmora-wordmark.png";
  import mark from "../assets/velmora-mark.webp";
  import markSmall from "../assets/velmora-mark-small.webp";

  // `size` is the height of the mark in px; everything else scales from it.
  // The fox's face stands in below 64px, where the tails would be a blur.
  let { size = 32, word = true }: { size?: number; word?: boolean } = $props();
</script>

<span class="logo" style:font-size="{size}px" role="img" aria-label="Velmora">
  <img class="mark" src={size < 64 ? markSmall : mark} alt="" aria-hidden="true" draggable="false" />
  {#if word}
    <span class="word" style:--wordmark="url({wordmark})"></span>
  {/if}
</span>

<style>
  .logo {
    display: inline-flex;
    align-items: center;
    gap: 0.36em;
    line-height: 1;
  }
  .mark {
    height: 1em;
    width: 1em;
    flex: none;
    display: block;
  }
  .word {
    height: 0.62em;
    aspect-ratio: 983 / 174;
    background: var(--logo-word-color, var(--md-sys-color-on-surface));
    -webkit-mask: var(--wordmark) center / contain no-repeat;
    mask: var(--wordmark) center / contain no-repeat;
  }
</style>
