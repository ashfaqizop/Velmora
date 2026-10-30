<script lang="ts">
  import name from "../assets/universeapi-name.png";
  import full from "../assets/universeapi-wordmark.png";

  // compact: "Powered by" + asterisk + name on one line.
  // full: the whole UniverseAPI lockup with its tagline (About dialog).
  let { variant = "compact", height = 14 }: { variant?: "compact" | "full"; height?: number } = $props();
</script>

{#snippet asterisk(size: number)}
  <svg class="asterisk" width={size} height={size} viewBox="0 0 200 200" aria-hidden="true">
    <defs>
      <radialGradient id="ua-glow" cx="50%" cy="50%" r="50%">
        <stop offset="0%" stop-color="#ffffff" />
        <stop offset="30%" stop-color="#f3c7c9" />
        <stop offset="100%" stop-color="#c0111a" />
      </radialGradient>
    </defs>
    <g fill="url(#ua-glow)">
      <rect x="88" y="0" width="24" height="200" />
      <rect x="0" y="88" width="200" height="24" />
      <rect x="88" y="0" width="24" height="200" transform="rotate(45 100 100)" />
      <rect x="88" y="0" width="24" height="200" transform="rotate(-45 100 100)" />
    </g>
  </svg>
{/snippet}

{#if variant === "full"}
  <span class="full" role="img" aria-label="UniverseAPI, Velmora quad-engine by StepByte Studios">
    {@render asterisk(height * 0.42)}
    <span class="mask" style:height="{height}px" style:aspect-ratio="1563 / 483" style:--src="url({full})"></span>
  </span>
{:else}
  <span class="compact">
    <span class="label">Powered by</span>
    {@render asterisk(height)}
    <span
      class="mask"
      role="img"
      aria-label="UniverseAPI"
      style:height="{height}px"
      style:aspect-ratio="1563 / 225"
      style:--src="url({name})"
    ></span>
  </span>
{/if}

<style>
  .compact {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .label {
    font-size: 12px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    opacity: 0.8;
  }
  .full {
    display: inline-flex;
    align-items: flex-start;
    gap: 6px;
  }
  .asterisk {
    flex: none;
  }
  .mask {
    display: inline-block;
    background: var(--md-sys-color-on-surface);
    -webkit-mask: var(--src) center / contain no-repeat;
    mask: var(--src) center / contain no-repeat;
  }
</style>
