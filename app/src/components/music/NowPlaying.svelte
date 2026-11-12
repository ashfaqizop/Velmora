<script lang="ts">
  import { M3eSnackbar } from "@m3e/web/snackbar";
  // The floating now-playing pill, across the whole app while music is
  // loaded. It shrinks to a round cover-and-play button while a page scrolls
  // down, and opens the full player on click. Tinted by the artwork.
  import { fly, scale } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import MorphCover from "./MorphCover.svelte";
  import { musicPlayer } from "../../lib/music/player.svelte";
  import { art, artists, likes, musicUi } from "../../lib/music/ui.svelte";
  import { artColors } from "../../lib/music/color";
  import { nav } from "../../lib/state.svelte";

  const cur = $derived(musicPlayer.current);
  const cover = $derived(cur ? art(cur.thumbnails, 120) : null);
  const progress = $derived(musicPlayer.duration ? Math.min(1, musicPlayer.position / musicPlayer.duration) : 0);
  const shown = $derived(!!cur && nav.route.name !== "player" && !musicUi.full);
  const compact = $derived(musicUi.compact);

  // Artwork colors for the whole player (pill and full view).
  $effect(() => {
    const url = cur ? art(cur.thumbnails, 60) : null;
    artColors(url).then((c) => (musicUi.colors = c));
  });

  function seek(e: MouseEvent) {
    const bar = e.currentTarget as HTMLElement;
    const r = bar.getBoundingClientRect();
    musicPlayer.seek(((e.clientX - r.left) / r.width) * musicPlayer.duration);
  }

  const openFull = (tab: "lyrics" | "queue" = "lyrics") => {
    musicUi.fullTab = tab;
    musicUi.full = true;
  };

  /** Stop the music and hide the bar, with a way back for a slip of the mouse. */
  async function dismiss() {
    const undo = await musicPlayer.dismiss();
    M3eSnackbar.open("Music stopped", "Undo", { actionCallback: undo });
  }
</script>

{#if shown && cur}
  <div
    class="pill"
    class:compact
    style:--np={musicUi.colors?.base ?? "var(--md-sys-color-primary)"}
    style:--np-deep={musicUi.colors?.deep ?? "var(--md-sys-color-surface-container-high)"}
    transition:fly={{ y: 80, duration: 420, easing: cubicOut }}
  >
    <button class="cover" aria-label="Open the player" onclick={() => openFull()}>
      <MorphCover src={cover} size={compact ? 56 : 52} playing={musicPlayer.playing} lobes={8} />
      {#if compact}
        <span class="mini-play" transition:scale={{ duration: 180 }}>
          <m3e-icon name={musicPlayer.playing ? "pause" : "play_arrow"} filled></m3e-icon>
        </span>
      {/if}
    </button>
    {#if compact}
      <button class="mini-hit" aria-label={musicPlayer.playing ? "Pause" : "Play"} onclick={() => musicPlayer.toggle()}></button>
    {:else}
      <button class="text" onclick={() => openFull()}>
        <strong>{cur.title}</strong>
        <span>{musicPlayer.error ?? (musicPlayer.loading ? "Loading…" : artists(cur))}</span>
      </button>
      <div class="controls">
        <button class="ctl" aria-label="Previous" onclick={() => musicPlayer.prev()}><m3e-icon name="skip_previous" filled></m3e-icon></button>
        <button class="ctl play" aria-label={musicPlayer.playing ? "Pause" : "Play"} onclick={() => musicPlayer.toggle()}>
          {#if musicPlayer.loading || musicPlayer.buffering}
            <m3e-loading-indicator class="busy"></m3e-loading-indicator>
          {:else}
            <m3e-icon name={musicPlayer.playing ? "pause" : "play_arrow"} filled></m3e-icon>
          {/if}
        </button>
        <button class="ctl" aria-label="Next" onclick={() => musicPlayer.next()}><m3e-icon name="skip_next" filled></m3e-icon></button>
      </div>
      <div class="side">
        <button class="ctl small" class:on={likes.liked(cur.id)} aria-label="Like" onclick={() => likes.toggle(cur)}>
          <m3e-icon name="favorite" filled={likes.liked(cur.id)}></m3e-icon>
        </button>
        <button class="ctl small" aria-label="Queue" onclick={() => openFull("queue")}><m3e-icon name="queue_music"></m3e-icon></button>
        <button class="ctl small" aria-label="Open the player" onclick={() => openFull()}><m3e-icon name="open_in_full"></m3e-icon></button>
        <button class="ctl small close" aria-label="Close the player" title="Stop and close" onclick={dismiss}><m3e-icon name="close"></m3e-icon></button>
      </div>
      <button class="bar" aria-label="Seek" onclick={seek}>
        <span class="fill" style:width={`${progress * 100}%`}></span>
      </button>
    {/if}
  </div>
{/if}

<style>
  .pill {
    position: fixed;
    left: calc(var(--app-sidebar-width) + (100vw - var(--app-sidebar-width)) / 2);
    bottom: 22px;
    transform: translateX(-50%);
    z-index: 600;
    display: flex;
    align-items: center;
    gap: 12px;
    width: min(720px, calc(100vw - var(--app-sidebar-width) - 64px));
    height: 72px;
    padding: 0 10px 0 10px;
    box-sizing: border-box;
    border-radius: 28px;
    overflow: hidden;
    background:
      radial-gradient(120% 140% at 0% 0%, color-mix(in srgb, var(--np) 34%, transparent), transparent 60%),
      color-mix(in srgb, var(--np-deep) 70%, var(--md-sys-color-surface-container-high));
    backdrop-filter: blur(24px) saturate(1.3);
    border: 1px solid color-mix(in srgb, var(--np) 30%, transparent);
    box-shadow: 0 18px 50px rgb(0 0 0 / 50%);
    transition:
      width 420ms cubic-bezier(0.2, 0, 0, 1),
      height 420ms cubic-bezier(0.2, 0, 0, 1),
      border-radius 420ms cubic-bezier(0.2, 0, 0, 1),
      background 600ms ease;
  }
  .pill.compact {
    width: 76px;
    height: 76px;
    border-radius: 38px;
    padding: 0;
    justify-content: center;
  }
  button {
    all: unset;
    cursor: pointer;
  }
  .cover {
    position: relative;
    display: grid;
    place-items: center;
    flex: none;
  }
  .mini-play {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    color: #fff;
    background: rgb(0 0 0 / 35%);
    border-radius: 50%;
    pointer-events: none;
  }
  .mini-hit {
    position: absolute;
    inset: 0;
  }
  .text {
    display: flex;
    flex-direction: column;
    min-width: 0;
    flex: 1;
  }
  .text strong,
  .text span {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .text strong {
    font-size: 14.5px;
  }
  .text span {
    font-size: 12.5px;
    color: var(--md-sys-color-on-surface-variant);
  }
  .controls,
  .side {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .ctl {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    border-radius: 50%;
    color: var(--md-sys-color-on-surface);
    transition:
      background 150ms ease,
      border-radius 250ms ease;
  }
  .ctl:hover {
    background: color-mix(in srgb, var(--md-sys-color-on-surface) 10%, transparent);
  }
  .ctl.play {
    width: 50px;
    height: 50px;
    border-radius: 18px;
    background: var(--np);
    color: var(--np-on, #fff);
    --m3e-icon-size: 28px;
  }
  .ctl.play:hover {
    border-radius: 50%;
    background: var(--np);
  }
  .busy {
    --m3e-loading-indicator-size: 30px;
  }
  .ctl.close:hover {
    color: var(--md-sys-color-error);
  }
  .ctl.small {
    width: 36px;
    height: 36px;
    color: var(--md-sys-color-on-surface-variant);
    --m3e-icon-size: 20px;
  }
  .ctl.on {
    color: var(--brand-coral, #ff5a5a);
  }
  .bar {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    height: 6px;
  }
  .fill {
    display: block;
    height: 3px;
    margin-top: 3px;
    background: var(--np);
    border-radius: 0 3px 3px 0;
    transition: width 250ms linear;
  }
  .bar:hover .fill {
    height: 5px;
    margin-top: 1px;
  }
</style>
