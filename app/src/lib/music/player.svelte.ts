// The music player: the queue, and playback through Velmora's mpv.
//
// mpv lives once per window, shared with the video player. Music runs it
// audio-only (or with video when the user turns that on); when a movie or
// show starts, the video player takes mpv over and music stops where it was,
// ready to resume. Streams come from yt-dlp (anonymously, see the engine's
// music::ytdlp) and play straight from YouTube's servers with the headers
// yt-dlp hands back.
import {
  command,
  getProperty,
  listenEvents,
  observeProperties,
  setProperty,
  setVideoMarginRatio,
  type MpvConfig,
  type MpvObservableProperty,
  type VideoMarginRatio,
} from "tauri-plugin-libmpv-api";
import { SvelteSet } from "svelte/reactivity";
import * as api from "../api";
import { sound } from "../audio.svelte";
import { startMpv, stopMpv } from "../mpv";
import type { MusicItem, MusicStream } from "../types";

export type Repeat = "off" | "all" | "one";

const OBSERVED = [
  ["pause", "flag"],
  ["time-pos", "double", "none"],
  ["duration", "double", "none"],
  ["paused-for-cache", "flag"],
  // True once mpv is actually drawing video (music videos).
  ["vo-configured", "flag"],
  // Loudness from the visualizer's astats filter (only there while the full view is open).
  ["af-metadata/viz", "node", "none"],
] as const satisfies MpvObservableProperty[];

/** Equalizer presets: [frequency Hz, gain dB] per band. */
export const EQ_PRESETS: Record<string, { label: string; bands: [number, number][] }> = {
  flat: { label: "Flat", bands: [] },
  bass: { label: "Bass boost", bands: [[60, 6], [150, 4], [400, 0]] },
  vshape: { label: "Bass and treble", bands: [[60, 5], [250, 1], [1000, -2], [4000, 2], [12000, 5]] },
  vocal: { label: "Vocals", bands: [[100, -2], [400, 0], [1500, 3], [3500, 4], [8000, 1]] },
  treble: { label: "Treble boost", bands: [[4000, 3], [8000, 5], [14000, 5]] },
  acoustic: { label: "Acoustic", bands: [[80, 3], [250, 1], [1000, 1], [3000, 2], [10000, 2]] },
  electronic: { label: "Electronic", bands: [[60, 5], [150, 3], [1000, -1], [5000, 2], [12000, 4]] },
  late: { label: "Late night", bands: [[60, -3], [250, 0], [2500, 2], [8000, -2]] },
};
const EQ_KEY = "velmora.music.eq";

function eqFilter(preset: string): string {
  const bands = EQ_PRESETS[preset]?.bands ?? [];
  if (!bands.length) return "";
  return `@eq:lavfi=[${bands.map(([f, g]) => `equalizer=f=${f}:t=q:w=1:g=${g}`).join(",")}]`;
}

function readEq(): string {
  try {
    const v = localStorage.getItem(EQ_KEY) ?? "flat";
    return EQ_PRESETS[v] ? v : "flat";
  } catch {
    return "flat";
  }
}

/** dB ("-23.4", "-inf") to 0..1. */
function level(v: unknown): number {
  const n = typeof v === "string" ? parseFloat(v) : typeof v === "number" ? v : NaN;
  if (!Number.isFinite(n)) return 0;
  return Math.max(0, Math.min(1, (n + 50) / 50));
}

/** googlevideo URLs last about six hours; use a resolved one for five. */
const STREAM_TTL = 5 * 60 * 60 * 1000;
/** Resolve the next track this long before the current one ends. */
const PREFETCH_AT = 25;
const VOLUME_KEY = "velmora.music.volume";
const AUTOPLAY_KEY = "velmora.music.autoplay";
/** Save the playback position this often while playing. */
const SAVE_EVERY = 30_000;
/** A play counts once this much of the song was heard (or half of a short one). */
const PLAY_COUNTS_AT = 30;

/** The queue as saved between launches (musicdb.rs). */
interface SavedQueue {
  queue: MusicItem[];
  index: number;
  position: number;
  source: string | null;
  shuffle: boolean;
  repeat: Repeat;
  /** Songs autoplay added. */
  auto: string[];
  /** The queue before shuffle, as song IDs. */
  ordered: string[];
  listNext: string | null;
  radioNext: string | null;
}

function readAutoplay(): boolean {
  try {
    return localStorage.getItem(AUTOPLAY_KEY) !== "0";
  } catch {
    return true;
  }
}

/** Take from each list in turn: a, b, c, a, b, c... */
function interleave<T>(lists: T[][]): T[] {
  const out: T[] = [];
  const longest = Math.max(0, ...lists.map((l) => l.length));
  for (let i = 0; i < longest; i++) for (const l of lists) if (i < l.length) out.push(l[i]);
  return out;
}

function readVolume(): number {
  try {
    const v = Number(localStorage.getItem(VOLUME_KEY));
    return Number.isFinite(v) && v > 0 ? Math.min(100, v) : 80;
  } catch {
    return 80;
  }
}

/** Headers for mpv. The User-Agent goes in its own option, and a header whose
 *  value has a comma can't be passed (mpv splits the list on commas). */
function headerOptions(s: MusicStream): { ua?: string; fields: string } {
  let ua: string | undefined;
  const fields: string[] = [];
  for (const [k, v] of s.headers) {
    if (k.toLowerCase() === "user-agent") ua = v;
    else if (!v.includes(",")) fields.push(`${k}: ${v}`);
  }
  return { ua, fields: fields.join(",") };
}

function shuffled<T>(list: T[]): T[] {
  const a = [...list];
  for (let i = a.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [a[i], a[j]] = [a[j], a[i]];
  }
  return a;
}

class MusicPlayer {
  /** What plays, in order. */
  queue = $state<MusicItem[]>([]);
  index = $state(-1);
  playing = $state(false);
  /** Waiting on yt-dlp or the network. */
  loading = $state(false);
  buffering = $state(false);
  position = $state(0);
  duration = $state(0);
  volume = $state(readVolume());
  shuffle = $state(false);
  repeat = $state<Repeat>("off");
  /** Show the music video when there is one. */
  video = $state(false);
  /** mpv is drawing the video right now. The full view stays opaque until then. */
  videoShown = $state(false);
  error = $state<string | null>(null);
  /** Where the queue came from, for the "Playing from" line. */
  source = $state<string | null>(null);
  /** Equalizer preset name (EQ_PRESETS). */
  eq = $state(readEq());
  /** When the queue runs out, keep going with songs like the last few. */
  autoplay = $state(readAutoplay());
  /** Songs autoplay added. Up next lists them under their own heading. */
  readonly auto = new SvelteSet<string>();
  /** Live loudness for the visualizer, written ~30 times a second. Not reactive on purpose. */
  readonly meter = { left: 0, right: 0, at: 0 };
  private viz = false;

  /** The order before shuffle, to restore it. */
  private ordered: MusicItem[] = [];
  /** More of the list the user picked (a long playlist), still to fetch. */
  private listNext: string | null = null;
  /** More of autoplay's radio. */
  private radioNext: string | null = null;
  /** The song being listened to, and for how long, for the play history. */
  private heard: MusicItem | null = null;
  private heardFor = 0;
  private lastPos = 0;
  private savedAt = 0;
  private saveTimer: ReturnType<typeof setTimeout> | undefined;
  private owned = false;
  private cleanups: (() => void)[] = [];
  private streams = new Map<string, { s: MusicStream; at: number; video: boolean }>();
  private loadSeq = 0;
  private loadedId: string | null = null;
  private resumeAt = 0;
  private prefetching = false;

  get current(): MusicItem | null {
    return this.queue[this.index] ?? null;
  }

  get active(): boolean {
    return this.current !== null;
  }

  get upNext(): MusicItem[] {
    return this.queue.slice(this.index + 1);
  }

  // ── mpv ──────────────────────────────────────────────────────────

  private config(withControls: boolean): MpvConfig {
    const options: Record<string, string | number | boolean> = {
      vo: "gpu-next",
      hwdec: "auto-safe",
      "force-window": "no",
      idle: "yes",
      "keep-open": "no",
      "osd-level": 0,
      "input-default-bindings": "no",
      "input-vo-keyboard": "no",
      "audio-display": "no",
      cache: "yes",
      "demuxer-max-bytes": "64MiB",
      "network-timeout": 20,
      ytdl: "no",
      volume: this.volume,
      "volume-max": sound.maxVolume,
      af: this.filters(),
    };
    // Media keys and the Windows media overlay, on mpv builds that have them.
    if (withControls) options["media-controls"] = "yes";
    return { initialOptions: options, observedProperties: OBSERVED };
  }

  private async ensureMpv() {
    if (this.owned) return;
    // Listen before mpv starts: it reports each observed property once right
    // after init, and a listener added later misses that first report (which
    // left the play button showing Play while a song played).
    this.cleanups.push(
      await observeProperties(OBSERVED, ({ name, data }) => {
        switch (name) {
          case "pause":
            this.playing = !(data as boolean);
            break;
          case "vo-configured":
            this.videoShown = this.video && !!data;
            break;
          case "time-pos": {
            const t = (data as number | null) ?? 0;
            // Only steady playback counts as listening, not a seek.
            if (this.playing && t > this.lastPos && t - this.lastPos < 3) this.heardFor += t - this.lastPos;
            this.lastPos = t;
            this.position = t;
            this.maybePrefetch();
            if (this.playing && Date.now() - this.savedAt > SAVE_EVERY) this.persist(0);
            break;
          }
          case "duration":
            if (data) this.duration = data as number;
            break;
          case "paused-for-cache":
            this.buffering = data as boolean;
            break;
          case "af-metadata/viz": {
            const m = (data ?? {}) as Record<string, unknown>;
            const overall = level(m["lavfi.astats.Overall.RMS_level"]);
            this.meter.left = level(m["lavfi.astats.1.RMS_level"]) || overall;
            this.meter.right = level(m["lavfi.astats.2.RMS_level"]) || overall;
            this.meter.at = performance.now();
            break;
          }
        }
      }),
    );
    this.cleanups.push(
      await listenEvents((ev) => {
        if (ev.event === "file-loaded") this.failures = 0;
        if (ev.event !== "end-file") return;
        if (ev.reason === "eof") this.onEnded();
        else if (ev.reason === "error") this.onFailed();
      }),
    );
    try {
      try {
        await startMpv(this.config(true));
      } catch {
        await startMpv(this.config(false));
      }
    } catch (e) {
      this.release();
      throw e;
    }
    this.owned = true;
  }

  /** Read the real pause state from mpv, in case an update was missed. */
  private async syncPause() {
    if (!this.owned) return;
    try {
      this.playing = !(await getProperty("pause", "flag"));
    } catch {
      // Keep what we have.
    }
  }

  /** The video player is taking mpv over. Remember where we were. */
  async yieldToVideo() {
    if (!this.owned) return;
    this.resumeAt = this.position;
    this.playing = false;
    this.videoShown = false;
    this.release();
  }

  private release() {
    this.cleanups.forEach((f) => f());
    this.cleanups = [];
    this.owned = false;
  }

  // ── Sound ────────────────────────────────────────────────────────

  /** The audio filter chain: equalizer, the shared sound options, and the
   *  visualizer's loudness meter while the full view is open. */
  private filters(): string {
    return [eqFilter(this.eq), sound.filters(), this.viz ? "@viz:lavfi=[astats=metadata=1:reset=1]" : ""]
      .filter(Boolean)
      .join(",");
  }

  /** Re-apply the chain after a sound setting changed. */
  applyFilters() {
    if (this.owned) setProperty("af", this.filters()).catch(() => {});
  }

  setEq(preset: string) {
    if (!EQ_PRESETS[preset]) return;
    this.eq = preset;
    try {
      localStorage.setItem(EQ_KEY, preset);
    } catch {
      // Not remembered.
    }
    this.applyFilters();
  }

  /** The full view turns the loudness meter on while it's open. */
  setVisualizer(on: boolean) {
    if (this.viz === on) return;
    this.viz = on;
    if (!on) {
      this.meter.left = 0;
      this.meter.right = 0;
    }
    this.applyFilters();
  }

  /** Where the music video goes in the window, as margins (0..1). */
  placeVideo(margins: VideoMarginRatio) {
    if (this.owned) setVideoMarginRatio(margins).catch(() => {});
  }

  // ── Streams ──────────────────────────────────────────────────────

  private async stream(item: MusicItem, video: boolean): Promise<MusicStream> {
    const hit = this.streams.get(item.id);
    if (hit && hit.video === video && Date.now() - hit.at < STREAM_TTL) return hit.s;
    const s = await api.musicStream(item.id, video);
    this.streams.set(item.id, { s, at: Date.now(), video });
    return s;
  }

  private async load(startAt = 0) {
    const item = this.current;
    if (!item) return;
    const seq = ++this.loadSeq;
    if (this.heard?.id !== item.id) {
      this.logPlay();
      this.heard = item;
      this.heardFor = 0;
    }
    this.lastPos = startAt;
    this.persist();
    // Reloading the same song (a switch to or from video) keeps its length.
    const same = this.duration > 0 && this.loadedId === item.id;
    this.loadedId = item.id;
    this.loading = true;
    this.error = null;
    this.videoShown = false;
    this.position = startAt;
    this.duration = item.duration ?? (same ? this.duration : 0);
    try {
      await this.ensureMpv();
      if (item.localPath) {
        // A downloaded song plays from this PC, no network needed.
        const local = await api.openLocal(item.localPath);
        if (seq !== this.loadSeq) return;
        await setProperty("http-header-fields", "").catch(() => {});
        await setProperty("audio-files", "").catch(() => {});
        await setProperty("vid", "no").catch(() => {});
        await setProperty("start", startAt > 0 ? String(startAt) : "none");
        await command("loadfile", [local.url]);
        await setProperty("pause", false);
        this.playing = true;
        return;
      }
      const s = await this.stream(item, this.video);
      if (seq !== this.loadSeq) return;
      const { ua, fields } = headerOptions(s);
      if (ua) await setProperty("user-agent", ua).catch(() => {});
      await setProperty("http-header-fields", fields).catch(() => {});
      await setProperty("start", startAt > 0 ? String(startAt) : "none");
      const withVideo = this.video && !!s.videoUrl;
      if (this.video && !s.videoUrl) {
        this.video = false;
        this.error = "This song has no video, so it plays as audio.";
      }
      await setProperty("vid", withVideo ? "auto" : "no").catch(() => {});
      await setProperty("force-window", withVideo ? "yes" : "no").catch(() => {});
      if (withVideo && s.videoUrl && s.videoUrl !== s.audioUrl) {
        await setProperty("audio-files", s.audioUrl).catch(() => {});
        await command("loadfile", [s.videoUrl]);
      } else {
        await setProperty("audio-files", "").catch(() => {});
        await command("loadfile", [withVideo && s.videoUrl ? s.videoUrl : s.audioUrl]);
      }
      await setProperty("pause", false);
      this.playing = true;
      this.prefetching = false;
    } catch (e) {
      if (seq === this.loadSeq) {
        this.error = typeof e === "string" ? e : "This song couldn't be played.";
        this.onFailed();
      }
    } finally {
      if (seq === this.loadSeq) this.loading = false;
    }
  }

  /** Resolve the next song ahead of time, so it starts without a wait. */
  private maybePrefetch() {
    if (this.current?.localPath) return;
    if (this.prefetching || !this.duration || this.duration - this.position > PREFETCH_AT) return;
    const next = this.queue[this.index + 1];
    if (!next) {
      this.extend();
      return;
    }
    this.prefetching = true;
    this.stream(next, this.video).catch(() => {});
  }

  // ── Queue ────────────────────────────────────────────────────────

  /** Play a list from one song. Nothing in the list needs resolving yet. */
  async playList(items: MusicItem[], start = 0, source: string | null = null) {
    const playable = items.filter((i) => i.kind === "song" || i.kind === "video");
    if (!playable.length) return;
    const first = items[start] && playable.includes(items[start]) ? playable.indexOf(items[start]) : 0;
    this.ordered = playable;
    this.listNext = null;
    this.radioNext = null;
    this.auto.clear();
    this.source = source;
    if (this.shuffle) {
      const rest = shuffled(playable.filter((_, i) => i !== first));
      this.queue = [playable[first], ...rest];
      this.index = 0;
    } else {
      this.queue = playable;
      this.index = first;
    }
    await this.load();
  }

  /** Play one song; autoplay carries on with songs like it. */
  async playSong(item: MusicItem, source: string | null = null) {
    await this.playList([item], 0, source);
    this.extend(item.id);
  }

  /** Start YouTube Music's own queue for a playlist, shuffle or radio button. */
  async playTarget(target: { videoId: string | null; playlistId: string | null; params: string | null }, source: string | null = null) {
    const page = await api.musicQueue(target.videoId, target.playlistId, target.params);
    await this.playList(page.tracks, 0, source);
    this.listNext = page.next;
  }

  /** The last few songs the user picked (not autoplay's), newest first. */
  private seeds(): string[] {
    const upTo = this.queue.slice(0, this.index + 1);
    const picked = upTo.filter((q) => !this.auto.has(q.id));
    const from = picked.length ? picked : upTo;
    return [...new Set(from.slice(-3).map((q) => q.id))].reverse();
  }

  private append(tracks: MusicItem[], auto: boolean) {
    const have = new Set(this.queue.map((q) => q.id));
    const fresh = tracks.filter((t) => (t.kind === "song" || t.kind === "video") && !have.has(t.id));
    if (!fresh.length) return;
    if (auto) for (const t of fresh) this.auto.add(t.id);
    this.queue = [...this.queue, ...fresh];
    this.ordered = [...this.ordered, ...fresh];
    this.persist();
  }

  /** Add more when the queue runs out: the rest of the picked list first,
   *  then (with autoplay on) a mix of radios from the last few songs. */
  private extending = false;
  private async extend(seed?: string) {
    if (this.extending) return;
    this.extending = true;
    try {
      if (this.listNext) {
        const page = await api.musicQueueMore(this.listNext);
        this.listNext = page.next;
        this.append(page.tracks, false);
        return;
      }
      if (!this.autoplay || this.repeat !== "off") return;
      if (this.radioNext) {
        const page = await api.musicQueueMore(this.radioNext);
        this.radioNext = page.next;
        this.append(page.tracks, true);
        return;
      }
      const seeds = seed ? [seed] : this.seeds();
      if (!seeds.length) return;
      const pages = await Promise.allSettled(seeds.map((id) => api.musicRadio(id)));
      const lists = pages.flatMap((p) => (p.status === "fulfilled" ? [p.value] : []));
      if (!lists.length) return;
      this.radioNext = lists[0].next;
      this.append(interleave(lists.map((l) => l.tracks)), true);
    } catch {
      // The queue just ends.
    } finally {
      this.extending = false;
    }
  }

  /** Put the queue in place; without shuffle the play order is the queue. */
  private setQueue(q: MusicItem[]) {
    this.queue = q;
    if (!this.shuffle) this.ordered = [...q];
    this.persist();
  }

  playNext(item: MusicItem) {
    if (!this.active) return this.playSong(item);
    const q = [...this.queue];
    q.splice(this.index + 1, 0, item);
    if (this.shuffle) this.ordered = [...this.ordered, item];
    this.setQueue(q);
  }

  /** Add to the end of what the user queued, ahead of autoplay's songs. */
  addToQueue(item: MusicItem) {
    if (!this.active) return this.playSong(item);
    const q = [...this.queue];
    const firstAuto = q.findIndex((x, i) => i > this.index && this.auto.has(x.id));
    q.splice(firstAuto < 0 ? q.length : firstAuto, 0, item);
    if (this.shuffle) this.ordered = [...this.ordered, item];
    this.setQueue(q);
  }

  removeAt(i: number) {
    if (i === this.index || i < 0 || i >= this.queue.length) return;
    const q = [...this.queue];
    const [gone] = q.splice(i, 1);
    if (i < this.index) this.index -= 1;
    if (this.shuffle) this.ordered = this.ordered.filter((o) => o !== gone);
    this.setQueue(q);
  }

  /** Drag-and-drop in Up next. A song dropped among the user's own songs
   *  stops counting as autoplay's. */
  move(from: number, to: number) {
    const n = this.queue.length;
    if (from === to || from < 0 || to < 0 || from >= n || to >= n) return;
    const q = [...this.queue];
    const [song] = q.splice(from, 1);
    q.splice(to, 0, song);
    let idx = this.index;
    if (from === idx) idx = to;
    else if (from < idx && to >= idx) idx -= 1;
    else if (from > idx && to <= idx) idx += 1;
    this.index = idx;
    const before = q[to - 1];
    if (!before || to - 1 <= idx || !this.auto.has(before.id)) this.auto.delete(song.id);
    this.setQueue(q);
  }

  /** Empty Up next; the current song keeps playing. */
  clearUpNext() {
    const cur = this.current;
    if (!cur) return;
    this.listNext = null;
    this.radioNext = null;
    this.auto.clear();
    this.index = 0;
    this.ordered = [cur];
    this.setQueue([cur]);
  }

  setAutoplay(on: boolean) {
    this.autoplay = on;
    try {
      localStorage.setItem(AUTOPLAY_KEY, on ? "1" : "0");
    } catch {
      // Not remembered.
    }
    if (!on) {
      // Take out autoplay's songs that haven't played yet.
      const q = this.queue.filter((x, i) => i <= this.index || !this.auto.has(x.id));
      this.radioNext = null;
      this.auto.clear();
      if (q.length !== this.queue.length) this.setQueue(q);
    } else if (this.active && this.index + 1 >= this.queue.length) {
      this.extend();
    }
  }

  /** Save the current song and Up next as a new YouTube Music playlist. */
  async saveAsPlaylist(title: string): Promise<string> {
    const ids = [...new Set(this.queue.slice(Math.max(0, this.index)).map((q) => q.id))];
    if (!ids.length) throw "The queue is empty.";
    return api.musicPlaylistCreate(title, ids);
  }

  jump(i: number) {
    if (i < 0 || i >= this.queue.length) return;
    this.index = i;
    return this.load();
  }

  // ── Saved between launches ───────────────────────────────────────

  /** Save the queue soon (several changes in a row save once). */
  persist(delay = 1200) {
    clearTimeout(this.saveTimer);
    this.saveTimer = setTimeout(() => this.save(), delay);
  }

  private save() {
    this.savedAt = Date.now();
    const saved: SavedQueue = {
      queue: $state.snapshot(this.queue) as MusicItem[],
      index: this.index,
      position: Math.floor(this.position),
      source: this.source,
      shuffle: this.shuffle,
      repeat: this.repeat,
      auto: [...this.auto],
      ordered: this.ordered.map((o) => o.id),
      listNext: this.listNext,
      radioNext: this.radioNext,
    };
    api.musicQueueSave(saved).catch(() => {});
  }

  /** At launch: put back the last queue, paused where it stopped. */
  async restore() {
    const s = await api.musicQueueLoad<SavedQueue>().catch(() => null);
    if (!s || !Array.isArray(s.queue) || !s.queue.length || this.active) return;
    const index = Math.min(Math.max(0, Math.floor(s.index) || 0), s.queue.length - 1);
    const byId = new Map(s.queue.map((q) => [q.id, q]));
    this.queue = s.queue;
    this.ordered = (s.ordered ?? []).map((id) => byId.get(id)).filter((q): q is MusicItem => !!q);
    if (this.ordered.length !== s.queue.length) this.ordered = [...s.queue];
    this.index = index;
    this.source = s.source ?? null;
    this.shuffle = !!s.shuffle;
    this.repeat = s.repeat === "all" || s.repeat === "one" ? s.repeat : "off";
    this.auto.clear();
    for (const id of s.auto ?? []) this.auto.add(id);
    this.listNext = s.listNext ?? null;
    this.radioNext = s.radioNext ?? null;
    this.position = this.resumeAt = Math.max(0, s.position || 0);
    this.duration = s.queue[index].duration ?? 0;
    this.playing = false;
  }

  /** Add the song that just stopped to the play history, if enough was heard. */
  private logPlay() {
    const item = this.heard;
    const secs = Math.round(this.heardFor);
    this.heard = null;
    this.heardFor = 0;
    if (!item) return;
    const len = item.duration ?? 0;
    if (secs >= PLAY_COUNTS_AT || (len > 0 && secs >= len / 2)) {
      api.musicPlayLog($state.snapshot(item) as MusicItem, secs).catch(() => {});
    }
  }

  // ── Controls ─────────────────────────────────────────────────────

  async toggle() {
    if (!this.active) return;
    if (!this.owned) return this.load(this.resumeAt);
    // Ask mpv rather than trust our copy, so the button always does what it shows.
    let paused = !this.playing;
    try {
      paused = (await getProperty("pause", "flag")) as boolean;
    } catch {
      // Use our copy.
    }
    await setProperty("pause", !paused).catch(() => {});
    this.playing = paused;
    this.syncPause();
    this.persist();
  }

  async next() {
    if (this.index + 1 < this.queue.length) {
      this.index += 1;
      return this.load();
    }
    if (this.repeat === "all" && this.queue.length) {
      this.index = 0;
      return this.load();
    }
    await this.extend();
    if (this.index + 1 < this.queue.length) {
      this.index += 1;
      return this.load();
    }
  }

  async prev() {
    if (this.position > 3 || this.index === 0) return this.seek(0);
    this.index -= 1;
    return this.load();
  }

  seek(to: number) {
    if (!this.owned) {
      this.resumeAt = this.position = Math.max(0, to);
      this.persist();
      return;
    }
    return command("seek", [Math.max(0, to), "absolute+exact"]).catch(() => {});
  }

  setVolume(v: number) {
    this.volume = Math.max(0, Math.min(sound.maxVolume, v));
    try {
      localStorage.setItem(VOLUME_KEY, String(this.volume));
    } catch {
      // Not remembered.
    }
    if (this.owned) setProperty("volume", this.volume).catch(() => {});
  }

  toggleShuffle() {
    this.shuffle = !this.shuffle;
    const cur = this.current;
    if (!cur) return;
    if (this.shuffle) {
      const rest = shuffled(this.queue.filter((_, i) => i !== this.index));
      this.queue = [cur, ...rest];
      this.index = 0;
    } else {
      this.queue = [...this.ordered];
      this.index = Math.max(0, this.queue.findIndex((q) => q.id === cur.id));
    }
    this.persist();
  }

  cycleRepeat() {
    this.repeat = this.repeat === "off" ? "all" : this.repeat === "all" ? "one" : "off";
    this.persist();
  }

  /** Switch between audio and the music video, from the same spot. */
  async setVideo(on: boolean) {
    if (this.video === on) return;
    this.video = on;
    if (!on) this.videoShown = false;
    if (this.active) await this.load(this.position);
  }

  /** Stop and clear everything. */
  async close() {
    this.loadSeq += 1;
    this.logPlay();
    this.auto.clear();
    this.listNext = null;
    this.radioNext = null;
    this.queue = [];
    this.index = -1;
    this.playing = false;
    this.videoShown = false;
    this.position = 0;
    this.duration = 0;
    this.persist(0);
    if (this.owned) {
      this.release();
      await stopMpv();
    }
  }

  private onEnded() {
    // Each full play counts, also each time round on repeat.
    this.logPlay();
    if (this.repeat === "one") {
      this.load();
      return;
    }
    this.next();
  }

  private failures = 0;
  private onFailed() {
    // A music video that won't play falls back to the song, from the same spot.
    if (this.video) {
      this.video = false;
      this.videoShown = false;
      this.load(this.position);
      this.error = "The video couldn't be played, so the song plays as audio.";
      return;
    }
    // Skip a song that won't play, but don't spin through a broken queue.
    this.failures += 1;
    if (this.failures > 3) {
      this.failures = 0;
      this.playing = false;
      this.error = "Several songs in a row wouldn't play. Check your connection, or update yt-dlp in Settings.";
      return;
    }
    if (this.index + 1 < this.queue.length) {
      this.index += 1;
      this.load();
    }
  }
}

export const musicPlayer = new MusicPlayer();
