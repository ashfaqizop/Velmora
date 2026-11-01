<script lang="ts">
  import wordmark from "../assets/velmora-wordmark.png";

  // `size` is the height of the mark in px; everything else scales from it.
  let { size = 32, word = true }: { size?: number; word?: boolean } = $props();
</script>

<span class="logo" style:font-size="{size}px" role="img" aria-label="Velmora">
  <svg class="mark" viewBox="50 120 1810 1630" aria-hidden="true">
    <g fill="var(--brand-coral)" stroke="var(--brand-coral)" stroke-width="34" stroke-linejoin="round">
      <polygon points="690,146 1278,146 1838,936 1810,966 1236,966 672,180" />
      <polygon points="628,966 1212,966 1770,1700 1752,1730 1166,1730 606,1000" />
    </g>
    <rect x="73" y="1377" width="505" height="353" rx="26" fill="var(--logo-square, var(--brand-indigo))" />
  </svg>
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
    width: 1.11em;
    flex: none;
    overflow: visible;
  }
  .word {
    height: 0.62em;
    aspect-ratio: 983 / 174;
    background: var(--logo-word-color, var(--md-sys-color-on-surface));
    -webkit-mask: var(--wordmark) center / contain no-repeat;
    mask: var(--wordmark) center / contain no-repeat;
  }
</style>
