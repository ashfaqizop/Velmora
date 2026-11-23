<script lang="ts">
  // The little status at the end of the username box. A ring spins while the
  // name is being checked; when the answer arrives the same ring closes and
  // draws a tick (free) or a cross (not available).
  type Check = "idle" | "same" | "checking" | "free" | "taken" | "bad";

  let { state }: { state: Check } = $props();

  const shown = $derived(state === "checking" ? "checking" : state === "free" ? "ok" : state === "taken" || state === "bad" ? "no" : "none");
</script>

<span class="status" data-s={shown} aria-hidden="true">
  <svg viewBox="0 0 24 24">
    <circle class="ring" cx="12" cy="12" r="10" pathLength="100" />
    <path class="mark tick" d="M7.2 12.6l3.1 3.1 6.5-6.9" pathLength="100" />
    <path class="mark x1" d="M8.6 8.6l6.8 6.8" pathLength="100" />
    <path class="mark x2" d="M15.4 8.6l-6.8 6.8" pathLength="100" />
  </svg>
</span>

<style>
  .status {
    --busy: var(--status-busy, #6c9bff);
    --ok: var(--status-ok, #3ecf8e);
    --no: var(--status-no, #ff6b6b);
    display: inline-grid;
    width: 24px;
    height: 24px;
    transition:
      opacity 200ms ease,
      scale 320ms cubic-bezier(0.34, 1.56, 0.64, 1);
  }
  .status[data-s="none"] {
    opacity: 0;
    scale: 0.6;
  }
  .status[data-s="ok"],
  .status[data-s="no"] {
    animation: pop 440ms cubic-bezier(0.34, 1.56, 0.64, 1) 120ms both;
  }
  svg {
    width: 100%;
    height: 100%;
    overflow: visible;
  }
  .status[data-s="no"] svg {
    animation: shake 380ms ease-in-out 160ms both;
  }

  .ring {
    fill: transparent;
    stroke: var(--busy);
    stroke-width: 2.4;
    stroke-linecap: round;
    stroke-dasharray: 28 72;
    transform-origin: 12px 12px;
    transform: rotate(-90deg);
    transition:
      stroke-dasharray 340ms cubic-bezier(0.2, 0.8, 0.2, 1),
      stroke 240ms ease,
      fill 300ms ease;
  }
  .status[data-s="checking"] .ring {
    animation: spin 0.9s linear infinite;
  }
  .status[data-s="ok"] .ring {
    stroke: var(--ok);
    stroke-dasharray: 100 0;
    fill: color-mix(in srgb, var(--ok) 16%, transparent);
  }
  .status[data-s="no"] .ring {
    stroke: var(--no);
    stroke-dasharray: 100 0;
    fill: color-mix(in srgb, var(--no) 16%, transparent);
  }

  .mark {
    fill: none;
    stroke-width: 2.4;
    stroke-linecap: round;
    stroke-linejoin: round;
    stroke-dasharray: 100;
    stroke-dashoffset: 100;
    transition: stroke-dashoffset 280ms cubic-bezier(0.2, 0.8, 0.2, 1);
  }
  .tick {
    stroke: var(--ok);
  }
  .x1,
  .x2 {
    stroke: var(--no);
  }
  .status[data-s="ok"] .tick {
    stroke-dashoffset: 0;
    transition-delay: 240ms;
  }
  .status[data-s="no"] .x1 {
    stroke-dashoffset: 0;
    transition-delay: 240ms;
  }
  .status[data-s="no"] .x2 {
    stroke-dashoffset: 0;
    transition-delay: 330ms;
  }

  @keyframes spin {
    from {
      transform: rotate(-90deg);
    }
    to {
      transform: rotate(270deg);
    }
  }
  @keyframes pop {
    0% {
      scale: 0.82;
    }
    60% {
      scale: 1.16;
    }
    100% {
      scale: 1;
    }
  }
  @keyframes shake {
    0%,
    100% {
      translate: 0 0;
    }
    20% {
      translate: -3px 0;
    }
    40% {
      translate: 3px 0;
    }
    60% {
      translate: -2px 0;
    }
    80% {
      translate: 2px 0;
    }
  }
  @keyframes breathe {
    50% {
      opacity: 0.35;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .status[data-s="checking"] .ring {
      animation: breathe 1.2s ease-in-out infinite;
    }
    .status,
    .status svg,
    .ring,
    .mark {
      animation-duration: 1ms !important;
      transition-duration: 1ms !important;
      transition-delay: 0s !important;
    }
  }
</style>
