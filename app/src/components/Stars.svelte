<script lang="ts">
  // Five stars with halves for a 1 to 10 score. With `onpick` it's an input:
  // hover shows the score, click sets it, the left half of a star is a half.
  let { value = 0, size = 18, onpick }: { value?: number; size?: number; onpick?: (score: number) => void } = $props();

  let hover = $state(0);
  const shown = $derived(hover || value);

  function at(e: PointerEvent | MouseEvent, i: number) {
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    return i * 2 + (e.clientX - r.left < r.width / 2 ? 1 : 2);
  }

  function key(e: KeyboardEvent) {
    if (!onpick) return;
    if (e.key === "ArrowRight" || e.key === "ArrowUp") onpick(Math.min(10, (value || 0) + 1));
    else if (e.key === "ArrowLeft" || e.key === "ArrowDown") onpick(Math.max(1, (value || 2) - 1));
    else return;
    e.preventDefault();
  }
</script>

{#if onpick}
  <div
    class="stars input"
    style:--s="{size}px"
    role="slider"
    tabindex="0"
    aria-label="Your rating"
    aria-valuemin={1}
    aria-valuemax={10}
    aria-valuenow={value || 0}
    aria-valuetext={value ? `${value / 2} out of 5 stars` : "Not rated"}
    onkeydown={key}
    onpointerleave={() => (hover = 0)}
  >
    {#each Array(5) as _, i (i)}
      <button type="button" tabindex="-1" class="star" onpointermove={(e) => (hover = at(e, i))} onclick={(e) => onpick(at(e, i))} aria-hidden="true">
        <span class="fill" style:--f="{Math.max(0, Math.min(2, shown - i * 2)) * 50}%"></span>
      </button>
    {/each}
  </div>
{:else}
  <span class="stars" style:--s="{size}px" role="img" aria-label="{value / 2} out of 5 stars">
    {#each Array(5) as _, i (i)}
      <span class="star"><span class="fill" style:--f="{Math.max(0, Math.min(2, value - i * 2)) * 50}%"></span></span>
    {/each}
  </span>
{/if}

<style>
  .stars {
    display: inline-flex;
    gap: calc(var(--s) * 0.12);
    line-height: 0;
  }
  .star {
    position: relative;
    width: var(--s);
    height: var(--s);
    padding: 0;
    border: 0;
    background: color-mix(in srgb, var(--md-sys-color-on-surface) 18%, transparent);
    clip-path: polygon(50% 0, 61.8% 35.1%, 100% 38.2%, 69.1% 61.8%, 80.9% 100%, 50% 76.4%, 19.1% 100%, 30.9% 61.8%, 0 38.2%, 38.2% 35.1%);
  }
  .fill {
    position: absolute;
    inset: 0 auto 0 0;
    width: var(--f);
    background: #f8d27a;
    transition: width 120ms ease;
  }
  .input .star {
    cursor: pointer;
    transition: transform 160ms cubic-bezier(0.2, 0.8, 0.2, 1.4);
  }
  .input .star:hover {
    transform: scale(1.15);
  }
  .input:focus-visible {
    outline: 2px solid var(--md-sys-color-primary);
    outline-offset: 4px;
    border-radius: 6px;
  }
</style>
