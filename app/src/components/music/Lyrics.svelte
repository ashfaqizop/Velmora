<script lang="ts">
  // Synced lyrics: big lines, the current one bright, the rest dimmed, kept
  // centred as the song plays. Click a line to jump there. Plain lyrics just
  // scroll.
  import * as api from "../../lib/api";
  import { musicPlayer } from "../../lib/music/player.svelte";
  import { artists } from "../../lib/music/ui.svelte";
  import type { MusicLyrics } from "../../lib/types";

  const cache = new Map<string, MusicLyrics | null>();

  let lyrics = $state<MusicLyrics | null>(null);
  let loading = $state(false);
  let box = $state<HTMLElement>();
  let userScrolled = 0;

  const cur = $derived(musicPlayer.current);

  $effect(() => {
    const song = cur;
    if (!song) return;
    if (cache.has(song.id)) {
      lyrics = cache.get(song.id) ?? null;
      return;
    }
    lyrics = null;
    loading = true;
    api
      .musicLyrics(song.id, song.title, artists(song), song.duration)
      .then((l) => {
        cache.set(song.id, l);
        if (musicPlayer.current?.id === song.id) lyrics = l;
      })
      .catch(() => cache.set(song.id, null))
      .finally(() => (loading = false));
  });

  // The line being sung: the last one that has started (a little early, so it lands on the beat).
  const active = $derived.by(() => {
    if (!lyrics?.synced) return -1;
    const t = musicPlayer.position + 0.3;
    let idx = -1;
    for (let i = 0; i < lyrics.lines.length; i++) {
      if ((lyrics.lines[i].time ?? 0) <= t) idx = i;
      else break;
    }
    return idx;
  });

  // Centre the current line by scrolling the lyrics box only. (scrollIntoView
  // also scrolled every box around it, which pushed the whole player up.)
  $effect(() => {
    const i = active;
    if (i < 0 || !box || Date.now() - userScrolled < 4000) return;
    const el = box.children[i] as HTMLElement | undefined;
    if (!el) return;
    const top = el.offsetTop - box.clientHeight / 2 + el.offsetHeight / 2;
    box.scrollTo({ top: Math.max(0, top), behavior: "smooth" });
  });
</script>

<div class="lyrics" bind:this={box} onwheel={() => (userScrolled = Date.now())}>
  {#if loading}
    <p class="note">Finding lyrics…</p>
  {:else if !lyrics}
    <p class="note">No lyrics for this song.</p>
  {:else}
    {#each lyrics.lines as line, i (i)}
      {#if lyrics.synced}
        <button
          class="line"
          class:active={i === active}
          class:past={i < active}
          onclick={() => line.time !== null && musicPlayer.seek(line.time)}
        >
          {line.text || "♪"}
        </button>
      {:else}
        <p class="plain">{line.text}&nbsp;</p>
      {/if}
    {/each}
    <p class="source">Lyrics from {lyrics.source}</p>
  {/if}
</div>

<style>
  .lyrics {
    position: absolute;
    inset: 0;
    overflow-y: auto;
    scrollbar-width: none;
    /* Half the panel above and below, so the first and last lines can sit in the middle. */
    padding: 45cqh 8px 45cqh 4px;
    box-sizing: border-box;
    mask-image: linear-gradient(180deg, transparent 0%, black 18%, black 82%, transparent 100%);
  }
  .line {
    all: unset;
    display: block;
    cursor: pointer;
    padding: 8px 0;
    font-size: clamp(22px, 2.3vw, 34px);
    font-weight: 700;
    line-height: 1.25;
    color: rgb(255 255 255 / 30%);
    transform-origin: left center;
    transition:
      color 300ms ease,
      transform 400ms cubic-bezier(0.2, 0, 0, 1),
      filter 300ms ease;
    filter: blur(0.4px);
  }
  .line:hover {
    color: rgb(255 255 255 / 60%);
  }
  .line.past {
    color: rgb(255 255 255 / 45%);
  }
  .line.active {
    color: #fff;
    transform: scale(1.04);
    filter: none;
  }
  .plain {
    margin: 0;
    font-size: 20px;
    line-height: 1.6;
    color: rgb(255 255 255 / 80%);
  }
  .note,
  .source {
    color: rgb(255 255 255 / 55%);
    font-size: 14px;
  }
  .source {
    margin-top: 24px;
  }
</style>
