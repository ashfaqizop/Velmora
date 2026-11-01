<script lang="ts">
  // Position dots for a sideways-scrolling row of cards. They follow the
  // swipe, and tapping one scrolls to that card.
  let { track, labels, label = "Cards" }: { track: HTMLElement | undefined; labels: string[]; label?: string } = $props();

  let active = $state(0);

  $effect(() => {
    const el = track;
    if (!el) return;
    const onScroll = () => {
      const first = el.firstElementChild as HTMLElement | null;
      if (!first) return;
      const step = first.offsetWidth + parseFloat(getComputedStyle(el).columnGap || "0");
      // The last card can't reach the start edge, so the end of the scroll counts as last.
      const atEnd = el.scrollLeft + el.clientWidth >= el.scrollWidth - 4;
      active = atEnd ? labels.length - 1 : Math.min(labels.length - 1, Math.round(el.scrollLeft / step));
    };
    el.addEventListener("scroll", onScroll, { passive: true });
    return () => el.removeEventListener("scroll", onScroll);
  });

  function go(i: number) {
    const card = track?.children[i] as HTMLElement | undefined;
    if (!track || !card) return;
    const pad = parseFloat(getComputedStyle(track).scrollPaddingInlineStart || "0") || 0;
    track.scrollTo({ left: card.getBoundingClientRect().left - track.getBoundingClientRect().left + track.scrollLeft - pad, behavior: "smooth" });
  }
</script>

<div class="dots" role="tablist" aria-label={label}>
  {#each labels as l, i (l)}
    <button role="tab" aria-selected={i === active} aria-label={l} class:on={i === active} onclick={() => go(i)}></button>
  {/each}
</div>

<style>
  .dots {
    display: flex;
    justify-content: center;
    gap: 6px;
    margin-top: 18px;
  }
  button {
    width: 8px;
    height: 8px;
    padding: 0;
    border: 0;
    border-radius: 999px;
    background: rgb(255 255 255 / 22%);
    cursor: pointer;
    transition:
      width 0.35s cubic-bezier(0.2, 0.7, 0.2, 1),
      background 0.35s;
  }
  button.on {
    width: 26px;
    background: #fff;
  }
</style>
