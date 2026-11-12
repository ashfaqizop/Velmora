<script lang="ts">
  // The full music player: the artwork's glow behind everything, the cover in
  // its living shape inside a ring that moves with the music, big controls,
  // and synced lyrics or the queue on the right. With video on, the music
  // video plays where the cover was.
  import { onMount, tick } from "svelte";
  import { fade, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import MorphCover from "./MorphCover.svelte";
  import Visualizer from "./Visualizer.svelte";
  import Lyrics from "./Lyrics.svelte";
  import Art from "./Art.svelte";
  import AdSlot from "../AdSlot.svelte";
  import { EQ_PRESETS, musicPlayer } from "../../lib/music/player.svelte";
  import { art, clock, likes, musicUi } from "../../lib/music/ui.svelte";
  import { sound } from "../../lib/audio.svelte";
  import { nav } from "../../lib/state.svelte";
  import { slider } from "../../lib/slider";
  import { M3eSnackbar } from "@m3e/web/snackbar";

  const cur = $derived(musicPlayer.current);
  const cover = $derived(cur ? art(cur.thumbnails, 600) : null);
  const tint = $derived(musicUi.colors?.base ?? "var(--md-sys-color-primary)");
  const deep = $derived(musicUi.colors?.deep ?? "#120b24");
  // Video is chosen; the page only turns see-through once mpv is drawing it,
  // so a slow or failed video never leaves a transparent window.
  const video = $derived(musicPlayer.video);
  const showingVideo = $derived(musicPlayer.video && musicPlayer.videoShown);

  let stage = $state<HTMLElement>();
  let stageSize = $state(420);
  let eqOpen = $state(false);
  let speedOpen = $state(false);
  const SPEEDS = [0.75, 1, 1.25, 1.5, 2];
  let seeking = $state<number | null>(null);
  let adShown = $state(false);
  let winH = $state(innerHeight);
  const tall = $derived(winH >= 900);

  function close() {
    musicUi.full = false;
  }

  // ── Up next ──
  const firstAuto = $derived(musicPlayer.queue.findIndex((q, i) => i > musicPlayer.index && musicPlayer.auto.has(q.id)));
  const hasUpNext = $derived(musicPlayer.queue.length > musicPlayer.index + 1);
  let naming = $state(false);
  let listName = $state("");
  let saving = $state(false);

  function startSave() {
    listName = musicPlayer.source ? `${musicPlayer.source} mix` : "My queue";
    naming = true;
  }

  async function save() {
    const title = listName.trim();
    if (!title || saving) return;
    saving = true;
    try {
      await musicPlayer.saveAsPlaylist(title);
      naming = false;
      M3eSnackbar.open(`Saved as "${title}"`);
      musicUi.loadPlaylists();
    } catch (e) {
      M3eSnackbar.open(typeof e === "string" ? e : "Couldn't save the playlist.");
    } finally {
      saving = false;
    }
  }

  // Drag to reorder. Songs can't go above the one playing.
  let dragFrom = $state<number | null>(null);
  let dropAt = $state<{ i: number; after: boolean } | null>(null);

  function dragStart(e: DragEvent, i: number) {
    dragFrom = i;
    e.dataTransfer?.setData("text/plain", String(i));
    if (e.dataTransfer) e.dataTransfer.effectAllowed = "move";
  }

  function dragOver(e: DragEvent, i: number) {
    if (dragFrom === null) return;
    e.preventDefault();
    const box = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const after = i === musicPlayer.index || e.clientY > box.top + box.height / 2;
    dropAt = { i, after };
  }

  function drop(e: DragEvent) {
    e.preventDefault();
    const from = dragFrom;
    const at = dropAt;
    dragEnd();
    if (from === null || !at) return;
    let to = from < at.i ? (at.after ? at.i : at.i - 1) : at.after ? at.i + 1 : at.i;
    to = Math.max(musicPlayer.index + 1, Math.min(musicPlayer.queue.length - 1, to));
    if (from === musicPlayer.index) return;
    musicPlayer.move(from, to);
  }

  function dragEnd() {
    dragFrom = null;
    dropAt = null;
  }

  // The music video is drawn by mpv behind the page; this places it where the cover was.
  async function placeVideo() {
    await tick();
    if (!stage || !video) return;
    const r = stage.getBoundingClientRect();
    musicPlayer.placeVideo({
      left: r.left / innerWidth,
      right: 1 - r.right / innerWidth,
      top: r.top / innerHeight,
      bottom: 1 - r.bottom / innerHeight,
    });
  }

  $effect(() => {
    document.documentElement.classList.toggle("music-video", showingVideo);
  });
  $effect(() => {
    if (video) placeVideo();
  });

  onMount(() => {
    const measure = () => {
      if (!stage) return;
      stageSize = Math.round(Math.min(stage.clientWidth, stage.clientHeight));
      placeVideo();
    };
    measure();
    const ro = new ResizeObserver(measure);
    if (stage) ro.observe(stage);
    return () => {
      ro.disconnect();
      document.documentElement.classList.remove("music-video");
      // Video only plays in the full view; back to audio when it closes.
      if (musicPlayer.video) musicPlayer.setVideo(false);
      musicPlayer.placeVideo({ left: 0, right: 0, top: 0, bottom: 0 });
    };
  });

  function onKey(e: KeyboardEvent) {
    const t = e.target as HTMLElement | null;
    if (t && (t.tagName === "INPUT" || t.tagName === "TEXTAREA" || t.isContentEditable)) return;
    if (e.key === "Escape") return close();
    if (e.key === "ArrowUp" && !e.ctrlKey) {
      e.preventDefault();
      musicPlayer.setVolume(musicPlayer.volume + 5);
    } else if (e.key === "ArrowDown" && !e.ctrlKey) {
      e.preventDefault();
      musicPlayer.setVolume(musicPlayer.volume - 5);
    } else if (e.key === "ArrowRight" && !e.ctrlKey) {
      musicPlayer.seek(musicPlayer.position + 5);
    } else if (e.key === "ArrowLeft" && !e.ctrlKey) {
      musicPlayer.seek(musicPlayer.position - 5);
    }
  }

  const goArtist = (id: string | null) => {
    if (!id) return;
    close();
    nav.go({ name: "music-artist", id });
  };
</script>

<svelte:window bind:innerHeight={winH} onkeydown={onKey} onresize={placeVideo} />

{#if cur}
  <div
    class="full"
    class:video={showingVideo}
    style:--np={tint}
    style:--np-deep={deep}
    transition:fly={{ y: 60, duration: 480, easing: cubicOut }}
  >
    {#if !showingVideo}
      <div class="glow" style:background-image={cover ? `url("${cover}")` : undefined} transition:fade={{ duration: 400 }}></div>
    {/if}
    <div class="shade"></div>

    <header>
      <button class="icon" aria-label="Close the player" onclick={close}><m3e-icon name="keyboard_arrow_down"></m3e-icon></button>
      <span class="from">{musicPlayer.source ? `Playing from ${musicPlayer.source}` : "Now playing"}</span>
      <span class="grow"></span>
      <m3e-segmented-button class="mode">
        <m3e-button-segment checked={!video} onclick={() => musicPlayer.setVideo(false)}>
          <m3e-icon slot="icon" name="music_note"></m3e-icon>Song
        </m3e-button-segment>
        <m3e-button-segment checked={video} onclick={() => musicPlayer.setVideo(true)}>
          <m3e-icon slot="icon" name="music_video"></m3e-icon>Video
        </m3e-button-segment>
      </m3e-segmented-button>
      <div class="eq-wrap">
        <button
          class="icon"
          class:on={musicPlayer.prefs.speed !== 1}
          aria-label="Speed"
          onclick={() => {
            speedOpen = !speedOpen;
            eqOpen = false;
          }}
        >
          {#if musicPlayer.prefs.speed === 1}<m3e-icon name="speed"></m3e-icon>{:else}<span class="speed-now">{musicPlayer.prefs.speed}×</span>{/if}
        </button>
        {#if speedOpen}
          <div class="eq" transition:fly={{ y: -8, duration: 180 }}>
            <strong>Speed</strong>
            {#each SPEEDS as v (v)}
              <button class:sel={musicPlayer.prefs.speed === v} onclick={() => musicPlayer.setPrefs({ speed: v })}>
                {v === 1 ? "Normal" : `${v}×`}{#if musicPlayer.prefs.speed === v}<m3e-icon name="check"></m3e-icon>{/if}
              </button>
            {/each}
            <label class="pitch">
              <span>Keep pitch</span>
              <m3e-switch
                checked={musicPlayer.prefs.keepPitch}
                onchange={(e: Event) => musicPlayer.setPrefs({ keepPitch: (e.currentTarget as HTMLElement & { checked: boolean }).checked })}
              ></m3e-switch>
            </label>
            <span class="hint">Any speed from 0.5× to 2× is in Settings, under Music.</span>
          </div>
        {/if}
      </div>
      <div class="eq-wrap">
        <button
          class="icon"
          class:on={musicPlayer.eq !== "flat"}
          aria-label="Equalizer"
          onclick={() => {
            eqOpen = !eqOpen;
            speedOpen = false;
          }}
        >
          <m3e-icon name="equalizer"></m3e-icon>
        </button>
        {#if eqOpen}
          <div class="eq" transition:fly={{ y: -8, duration: 180 }}>
            <strong>Equalizer</strong>
            {#each Object.entries(EQ_PRESETS) as [id, p] (id)}
              <button class:sel={musicPlayer.eq === id} onclick={() => musicPlayer.setEq(id)}>
                {p.label}{#if musicPlayer.eq === id}<m3e-icon name="check"></m3e-icon>{/if}
              </button>
            {/each}
            {#each musicPlayer.eqState.saved as c (c.name)}
              <button class:sel={musicPlayer.eq === `user:${c.name}`} onclick={() => musicPlayer.setEq(`user:${c.name}`)}>
                {c.name}{#if musicPlayer.eq === `user:${c.name}`}<m3e-icon name="check"></m3e-icon>{/if}
              </button>
            {/each}
            {#if musicPlayer.eq === "custom"}<button class="sel">Custom<m3e-icon name="check"></m3e-icon></button>{/if}
            <button
              class="edit"
              onclick={() => {
                eqOpen = false;
                musicUi.full = false;
                nav.go({ name: "settings", page: "music", setting: "eq" });
              }}><m3e-icon name="tune"></m3e-icon>Edit the curve</button
            >
            <span class="hint">Night mode, even volume and 200% are in Settings, under Sound.</span>
          </div>
        {/if}
      </div>
    </header>

    <div class="body">
      <section class="left">
        <div class="stage" bind:this={stage}>
          {#if !showingVideo}
            <Visualizer size={stageSize} color={tint} />
            <div class="cover">
              <MorphCover src={cover} size={Math.round(stageSize * 0.66)} playing={musicPlayer.playing} lobes={12} react />
            </div>
            {#if video}<span class="vid-wait">Loading the video…</span>{/if}
          {/if}
        </div>

        <div class="meta">
          <div class="titles">
            <h1 class="display">{cur.title}</h1>
            <span class="by">
              {#each cur.artists as a, k (k)}
                {#if k > 0},&nbsp;{/if}
                {#if a.id}<button class="link" onclick={() => goArtist(a.id)}>{a.name}</button>{:else}{a.name}{/if}
              {:else}{cur.subtitle ?? ""}{/each}
              {#if cur.album}<span class="album"> · {cur.album}</span>{/if}
            </span>
          </div>
          <button class="icon big" class:liked={likes.liked(cur.id)} aria-label="Like" onclick={() => likes.toggle(cur)}>
            <m3e-icon name="favorite" filled={likes.liked(cur.id)}></m3e-icon>
          </button>
        </div>

        <div class="seek">
          <span>{clock(seeking ?? musicPlayer.position)}</span>
          <m3e-slider class="slider" min="0" max={Math.max(1, Math.floor(musicPlayer.duration))} step="1" aria-label="Seek" use:slider>
            <m3e-slider-thumb
              value={Math.floor(seeking ?? musicPlayer.position)}
              oninput={(e: Event) => (seeking = (e.currentTarget as HTMLElement & { value: number }).value)}
              onchange={(e: Event) => {
                musicPlayer.seek((e.currentTarget as HTMLElement & { value: number }).value);
                seeking = null;
              }}
            ></m3e-slider-thumb>
          </m3e-slider>
          <span>{clock(musicPlayer.duration)}</span>
        </div>

        <div class="controls">
          <button class="icon" class:on={musicPlayer.shuffle} aria-label="Shuffle" onclick={() => musicPlayer.toggleShuffle()}>
            <m3e-icon name="shuffle"></m3e-icon>
          </button>
          <button class="icon mid" aria-label="Previous" onclick={() => musicPlayer.prev()}><m3e-icon name="skip_previous" filled></m3e-icon></button>
          <button class="play" aria-label={musicPlayer.playing ? "Pause" : "Play"} onclick={() => musicPlayer.toggle()}>
            {#if musicPlayer.loading || musicPlayer.buffering}
              <m3e-loading-indicator></m3e-loading-indicator>
            {:else}
              <m3e-icon name={musicPlayer.playing ? "pause" : "play_arrow"} filled></m3e-icon>
            {/if}
          </button>
          <button class="icon mid" aria-label="Next" onclick={() => musicPlayer.next()}><m3e-icon name="skip_next" filled></m3e-icon></button>
          <button class="icon" class:on={musicPlayer.repeat !== "off"} aria-label="Repeat" onclick={() => musicPlayer.cycleRepeat()}>
            <m3e-icon name={musicPlayer.repeat === "one" ? "repeat_one" : "repeat"}></m3e-icon>
          </button>
        </div>

        <div class="volume">
          <m3e-icon name={musicPlayer.volume === 0 ? "volume_off" : "volume_up"}></m3e-icon>
          <m3e-slider class="slider" min="0" max={sound.maxVolume} step="1" aria-label="Volume" use:slider>
            <m3e-slider-thumb
              value={Math.round(musicPlayer.volume)}
              oninput={(e: Event) => musicPlayer.setVolume((e.currentTarget as HTMLElement & { value: number }).value)}
            ></m3e-slider-thumb>
          </m3e-slider>
        </div>
        {#if musicPlayer.error}<p class="err">{musicPlayer.error}</p>{/if}
      </section>

      <section class="right">
        <m3e-segmented-button class="tabs">
          <m3e-button-segment checked={musicUi.fullTab === "lyrics"} onclick={() => (musicUi.fullTab = "lyrics")}>
            <m3e-icon slot="icon" name="lyrics"></m3e-icon>Lyrics
          </m3e-button-segment>
          <m3e-button-segment checked={musicUi.fullTab === "queue"} onclick={() => (musicUi.fullTab = "queue")}>
            <m3e-icon slot="icon" name="queue_music"></m3e-icon>Up next
          </m3e-button-segment>
        </m3e-segmented-button>
        <div class="panel" class:roomy={!adShown}>
          {#if musicUi.fullTab === "lyrics"}
            <Lyrics />
          {:else}
            <div class="queue">
              <div class="q-tools">
                <label class="autoplay">
                  <m3e-switch
                    checked={musicPlayer.autoplay}
                    onchange={(e: Event) => musicPlayer.setAutoplay((e.currentTarget as HTMLElement & { checked: boolean }).checked)}
                  ></m3e-switch>
                  <span>Autoplay</span>
                </label>
                <span class="grow"></span>
                <button class="chip" disabled={!musicPlayer.active} onclick={startSave}>
                  <m3e-icon name="playlist_add"></m3e-icon>Save as playlist
                </button>
                <button class="chip" disabled={!hasUpNext} onclick={() => musicPlayer.clearUpNext()}>
                  <m3e-icon name="clear_all"></m3e-icon>Clear
                </button>
              </div>
              {#if naming}
                <form class="q-name" onsubmit={(e) => (e.preventDefault(), save())}>
                  <!-- svelte-ignore a11y_autofocus -->
                  <input bind:value={listName} aria-label="Playlist name" maxlength="150" autofocus />
                  <button type="submit" class="chip on" disabled={saving || !listName.trim()}>{saving ? "Saving…" : "Save"}</button>
                  <button type="button" class="chip" onclick={() => (naming = false)}>Cancel</button>
                </form>
              {/if}
              {#each musicPlayer.queue as q, i (q.id + i)}
                {#if i >= musicPlayer.index}
                  {#if i === firstAuto}
                    <div class="q-head">
                      <m3e-icon name="all_inclusive"></m3e-icon>
                      <span>Autoplay</span><small>Songs like the ones you played</small>
                    </div>
                  {/if}
                  <div
                    class="q"
                    class:now={i === musicPlayer.index}
                    class:dragging={dragFrom === i}
                    class:drop-before={dropAt?.i === i && !dropAt.after}
                    class:drop-after={dropAt?.i === i && dropAt.after}
                    draggable={i !== musicPlayer.index}
                    role="listitem"
                    ondragstart={(e) => dragStart(e, i)}
                    ondragover={(e) => dragOver(e, i)}
                    ondrop={drop}
                    ondragend={dragEnd}
                  >
                    {#if i !== musicPlayer.index}
                      <span class="grip" aria-hidden="true"><m3e-icon name="drag_indicator"></m3e-icon></span>
                    {/if}
                    <button class="q-main" onclick={() => musicPlayer.jump(i)}>
                      <Art thumbs={q.thumbnails} size={44} alt="" />
                      <span class="q-text">
                        <strong>{q.title}</strong>
                        <span>{q.artists.map((a) => a.name).join(", ") || q.subtitle || ""}</span>
                      </span>
                      <span class="q-len">{q.duration ? clock(q.duration) : ""}</span>
                    </button>
                    {#if i !== musicPlayer.index}
                      <button class="icon small" aria-label="Remove from the queue" onclick={() => musicPlayer.removeAt(i)}>
                        <m3e-icon name="close"></m3e-icon>
                      </button>
                    {/if}
                  </div>
                {/if}
              {/each}
            </div>
          {/if}
        </div>
        <!-- The ad has its own space under the panel and never covers it. On a
             short window it only shows with Up next, so lyrics keep the room. -->
        {#if musicUi.fullTab === "queue" || tall}
          <div class="ad"><AdSlot id="nowplaying" bind:shown={adShown} /></div>
        {/if}
      </section>
    </div>
  </div>
{/if}

<style>
  .full {
    position: fixed;
    inset: 0;
    z-index: 700;
    display: flex;
    flex-direction: column;
    color: #fff;
    background: var(--np-deep);
    /* clip, not hidden: a hidden box can still be scrolled by code. */
    overflow: clip;
    isolation: isolate;
  }
  .full.video {
    background: transparent;
  }
  .glow {
    position: absolute;
    inset: -15%;
    z-index: -2;
    background-size: cover;
    background-position: center;
    filter: blur(90px) saturate(1.6) brightness(0.75);
    animation: breathe 22s ease-in-out infinite alternate;
  }
  @keyframes breathe {
    to {
      transform: scale(1.15) rotate(4deg);
    }
  }
  .shade {
    position: absolute;
    inset: 0;
    z-index: -1;
    background:
      radial-gradient(60% 70% at 25% 45%, transparent, rgb(0 0 0 / 35%)),
      linear-gradient(180deg, rgb(0 0 0 / 25%), rgb(0 0 0 / 55%));
  }
  .video .shade {
    background: linear-gradient(90deg, transparent 55%, rgb(10 8 16 / 92%) 62%);
  }
  header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 18px 28px 0;
  }
  .from {
    font-size: 13px;
    font-weight: 600;
    letter-spacing: 0.04em;
    color: rgb(255 255 255 / 70%);
  }
  .grow {
    flex: 1;
  }
  button {
    all: unset;
    cursor: pointer;
  }
  .icon {
    display: grid;
    place-items: center;
    width: 44px;
    height: 44px;
    border-radius: 50%;
    color: rgb(255 255 255 / 85%);
    transition:
      background 150ms ease,
      color 150ms ease;
  }
  .icon:hover {
    background: rgb(255 255 255 / 12%);
  }
  .icon.on {
    color: var(--np);
  }
  .icon.mid {
    --m3e-icon-size: 34px;
    width: 56px;
    height: 56px;
  }
  .icon.big {
    --m3e-icon-size: 28px;
  }
  .icon.liked {
    color: var(--brand-coral, #ff5a5a);
  }
  .icon.small {
    width: 32px;
    height: 32px;
    --m3e-icon-size: 18px;
  }
  .eq-wrap {
    position: relative;
  }
  .eq .edit {
    margin-top: 4px;
    justify-content: flex-start;
    gap: 8px;
    color: var(--np);
    font-weight: 700;
    --m3e-icon-size: 18px;
  }
  .speed-now {
    font-size: 13px;
    font-weight: 800;
  }
  .pitch {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 10px 4px;
    font-size: 14px;
    cursor: pointer;
  }
  .eq {
    position: absolute;
    right: 0;
    top: 52px;
    z-index: 5;
    width: 240px;
    padding: 10px;
    border-radius: 20px;
    background: rgb(20 16 30 / 92%);
    backdrop-filter: blur(20px);
    box-shadow: 0 16px 40px rgb(0 0 0 / 50%);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .eq strong {
    padding: 6px 10px 8px;
  }
  .eq button {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 10px;
    border-radius: 12px;
    font-size: 14px;
  }
  .eq button:hover,
  .eq .sel {
    background: rgb(255 255 255 / 10%);
  }
  .hint {
    padding: 8px 10px 4px;
    font-size: 11.5px;
    color: rgb(255 255 255 / 55%);
  }
  .body {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(0, 1.1fr) minmax(360px, 0.9fr);
    gap: 40px;
    padding: 10px 48px 28px;
  }
  .left {
    display: flex;
    flex-direction: column;
    min-height: 0;
    gap: 14px;
  }
  .stage {
    position: relative;
    flex: 1;
    min-height: 200px;
    display: grid;
    place-items: center;
  }
  .vid-wait {
    position: absolute;
    bottom: 0;
    font-size: 12.5px;
    color: rgb(255 255 255 / 65%);
  }
  .cover {
    position: relative;
    filter: drop-shadow(0 30px 60px rgb(0 0 0 / 50%));
  }
  .meta {
    display: flex;
    align-items: center;
    gap: 16px;
  }
  .titles {
    flex: 1;
    min-width: 0;
  }
  h1 {
    margin: 0;
    font-size: clamp(30px, 3.2vw, 46px);
    font-weight: 400;
    line-height: 1.05;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .by {
    font-size: 16px;
    color: rgb(255 255 255 / 72%);
  }
  .link:hover {
    text-decoration: underline;
    color: #fff;
  }
  .album {
    color: rgb(255 255 255 / 50%);
  }
  .seek,
  .volume {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 12.5px;
    font-variant-numeric: tabular-nums;
    color: rgb(255 255 255 / 70%);
  }
  .slider {
    flex: 1;
    --m3e-slider-active-track-color: var(--np);
    --m3e-slider-handle-color: #fff;
    --m3e-slider-inactive-track-color: rgb(255 255 255 / 20%);
  }
  .volume {
    max-width: 260px;
    align-self: center;
    width: 100%;
  }
  .controls {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 18px;
  }
  .play {
    display: grid;
    place-items: center;
    width: 78px;
    height: 78px;
    border-radius: 28px;
    background: var(--np);
    color: #fff;
    --m3e-icon-size: 40px;
    box-shadow: 0 14px 34px color-mix(in srgb, var(--np) 45%, transparent);
    transition: border-radius 320ms cubic-bezier(0.2, 0, 0, 1);
  }
  .play:hover {
    border-radius: 50%;
  }
  .err {
    margin: 0;
    text-align: center;
    color: #ffb4ab;
    font-size: 13px;
  }
  .right {
    display: flex;
    flex-direction: column;
    gap: 14px;
    min-height: 0;
    overflow: hidden;
  }
  .tabs {
    align-self: flex-start;
  }
  /* The panel clips its contents: lyrics and the queue scroll inside it and
     never spill over the ad or the rest of the page. */
  .panel {
    flex: 1;
    min-height: 120px;
    position: relative;
    overflow: hidden;
    container-type: size;
  }
  .queue {
    position: absolute;
    inset: 0;
    overflow-y: auto;
    scrollbar-width: thin;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .q {
    display: flex;
    align-items: center;
    border-radius: 16px;
  }
  .q.now {
    background: color-mix(in srgb, var(--np) 28%, rgb(255 255 255 / 8%));
  }
  .q {
    position: relative;
  }
  .q.dragging {
    opacity: 0.4;
  }
  .q.drop-before::before,
  .q.drop-after::after {
    content: "";
    position: absolute;
    left: 10px;
    right: 10px;
    height: 3px;
    border-radius: 2px;
    background: var(--np);
    pointer-events: none;
  }
  .q.drop-before::before {
    top: -2px;
  }
  .q.drop-after::after {
    bottom: -2px;
  }
  .grip {
    display: grid;
    place-items: center;
    width: 20px;
    margin-left: 4px;
    cursor: grab;
    color: rgb(255 255 255 / 40%);
    --m3e-icon-size: 18px;
  }
  .q:hover .grip {
    color: rgb(255 255 255 / 75%);
  }
  .q-tools {
    position: sticky;
    top: 0;
    z-index: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    margin-bottom: 6px;
    border-radius: 18px;
    background: color-mix(in srgb, var(--np-deep) 55%, transparent);
    backdrop-filter: blur(16px);
  }
  .grow {
    flex: 1;
  }
  .autoplay {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 32px;
    padding: 0 12px;
    border-radius: 16px;
    font-size: 13px;
    font-weight: 600;
    color: #fff;
    background: rgb(255 255 255 / 10%);
    --m3e-icon-size: 18px;
  }
  .chip:hover:not(:disabled) {
    background: rgb(255 255 255 / 18%);
  }
  .chip.on {
    background: var(--np);
  }
  .chip:disabled {
    opacity: 0.45;
    cursor: default;
  }
  .q-name {
    display: flex;
    gap: 8px;
    padding: 0 4px 10px;
  }
  .q-name input {
    flex: 1;
    min-width: 0;
    height: 32px;
    padding: 0 12px;
    border-radius: 16px;
    border: 1px solid rgb(255 255 255 / 25%);
    background: rgb(0 0 0 / 25%);
    color: #fff;
    font: inherit;
    font-size: 14px;
  }
  .q-head {
    display: flex;
    align-items: baseline;
    gap: 8px;
    padding: 16px 10px 6px;
    font-weight: 700;
    --m3e-icon-size: 18px;
  }
  .q-head m3e-icon {
    align-self: center;
    color: var(--np);
  }
  .q-head small {
    font-size: 12.5px;
    font-weight: 400;
    color: rgb(255 255 255 / 55%);
  }
  .q-main {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 10px;
    border-radius: 16px;
  }
  .q-main:hover {
    background: rgb(255 255 255 / 7%);
  }
  .q-text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .q-text strong,
  .q-text span {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .q-text span,
  .q-len {
    font-size: 12.5px;
    color: rgb(255 255 255 / 60%);
  }
  .q.now strong {
    color: #fff;
  }
  .ad {
    flex: none;
  }
  @media (max-width: 1100px) {
    .body {
      grid-template-columns: 1fr;
      overflow-y: auto;
    }
  }
</style>
