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
  destroy,
  getProperty,
  init,
  listenEvents,
  setProperty,
  setVideoMarginRatio,
  type MpvConfig,
  type MpvEvent,
  type MpvObservableProperty,
  type VideoMarginRatio,
} from "tauri-plugin-libmpv-api";
import { listen } from "@tauri-apps/api/event";
import { SvelteSet } from "svelte/reactivity";
import * as api from "../api";
import { sound } from "../audio.svelte";
import { startMpv, stopMpv } from "../mpv";
import type { MusicItem, MusicStream } from "../types";

export type Repeat = "off" | "all" | "one";
export type AudioQuality = "low" | "normal" | "high";

/** Two mpv players. "a" is the window's own (shared with the video player,
 *  and the only one that can show a music video); "b" is a second,
 *  audio-only one with no window, used to crossfade. */
type Deck = "a" | "b";
const DECK_B = "music-b";

/** Music → Playback settings. */
export interface PlaybackPrefs {
  /** Seconds the songs overlap. 0 = gapless. */
  crossfade: number;
  quality: AudioQuality;
  /** Save the next few songs ahead of time. */
  precache: boolean;
  /** Size cap for those songs, in MB. */
  cacheMb: number;
  /** 0.5 to 2. */
  speed: number;
  /** Keep voices at their pitch when the speed changes. */
  keepPitch: boolean;
  /** Even songs out with YouTube's own loudness figures. */
  normalize: boolean;
  /** mpv's audio-device name; "auto" follows Windows. */
  device: string;
  /** WASAPI exclusive: Velmora takes the device, past the Windows mixer. */
  exclusive: boolean;
}

const PREFS_KEY = "velmora.music.playback";
const PREFS_DEFAULTS: PlaybackPrefs = {
  crossfade: 0,
  quality: "high",
  precache: true,
  cacheMb: 500,
  speed: 1,
  keepPitch: true,
  normalize: true,
  device: "auto",
  exclusive: false,
};

function cleanPrefs(p: Partial<PlaybackPrefs>): PlaybackPrefs {
  const n = { ...PREFS_DEFAULTS, ...p };
  return {
    crossfade: Math.min(12, Math.max(0, Math.round(Number(n.crossfade) || 0))),
    quality: n.quality === "low" || n.quality === "normal" ? n.quality : "high",
    precache: !!n.precache,
    cacheMb: Math.min(4000, Math.max(100, Math.round(Number(n.cacheMb) || 500))),
    speed: Math.min(2, Math.max(0.5, Math.round((Number(n.speed) || 1) * 100) / 100)),
    keepPitch: n.keepPitch !== false,
    normalize: n.normalize !== false,
    device: typeof n.device === "string" && n.device ? n.device.slice(0, 300) : "auto",
    exclusive: !!n.exclusive,
  };
}

/** Normalization never lifts a quiet song by more than this, to stay clear of clipping. */
const LIFT_MAX_DB = 4;
const CUT_MAX_DB = 15;

/** An output device as mpv lists it. */
export interface AudioDevice {
  name: string;
  description: string;
}

function readPrefs(): PlaybackPrefs {
  try {
    return cleanPrefs(JSON.parse(localStorage.getItem(PREFS_KEY) ?? "{}"));
  } catch {
    return { ...PREFS_DEFAULTS };
  }
}

/** Start getting the next song ready this long before a crossfade begins. */
const CROSSFADE_LEAD = 10;
/** How often the crossfade ramp moves. */
const FADE_TICK = 50;
/** Pre-cache waits this long into a song, so skipping through costs nothing. */
const PRECACHE_DELAY = 5000;
/** How many songs ahead to keep saved. */
const PRECACHE_AHEAD = 3;

/** A song saved on this PC, shaped like a stream. */
function localStream(item: MusicItem, path: string): MusicStream {
  return {
    videoId: item.id,
    title: item.title,
    duration: item.duration,
    audioUrl: path,
    videoUrl: null,
    videoHeight: null,
    headers: [],
    audioCodec: null,
    audioBitrate: null,
    ext: null,
    filesize: null,
  };
}

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

// ── Equalizer ────────────────────────────────────────────────────
//
// Ten bands, an octave apart. Each can be moved (frequency), raised or cut
// (gain, ±12 dB) and narrowed or widened (Q). The lowest is a low shelf and
// the highest a high shelf, so a bass or treble boost lifts everything past
// it rather than one hump; the rest are peaks. Built as mpv lavfi filters
// with the same RBJ biquads the curve in Settings is drawn from.

export const EQ_FREQS = [31, 62, 125, 250, 500, 1000, 2000, 4000, 8000, 16000];
export const EQ_RANGE = 12;
const EQ_Q = 1.1;

export interface EqBand {
  /** Centre (or corner, for the shelves) in Hz. */
  f: number;
  /** dB. */
  g: number;
  q: number;
}

const flatBands = (): EqBand[] => EQ_FREQS.map((f) => ({ f, g: 0, q: EQ_Q }));
const bandsOf = (gains: number[]): EqBand[] => EQ_FREQS.map((f, i) => ({ f, g: gains[i] ?? 0, q: EQ_Q }));

/** Built-in curves. The keys are the ones saved before, so a saved choice still loads. */
export const EQ_PRESETS: Record<string, { label: string; gains: number[] }> = {
  flat: { label: "Flat", gains: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
  bass: { label: "Bass boost", gains: [7, 6, 4.5, 2, 0, 0, 0, 0, 0, 0] },
  vshape: { label: "Bass and treble", gains: [5, 4.5, 3, 0.5, -1.5, -2, -0.5, 2, 4, 5] },
  vocal: { label: "Vocals", gains: [-3, -2.5, -1.5, 0, 2, 3.5, 4, 2.5, 0.5, -1] },
  treble: { label: "Treble boost", gains: [0, 0, 0, 0, 0, 0.5, 2, 4, 5.5, 6] },
  acoustic: { label: "Acoustic", gains: [3, 3, 2, 1, 1, 1, 2, 2.5, 2, 1.5] },
  electronic: { label: "Electronic", gains: [5, 4.5, 3, 0.5, -1, -0.5, 1, 2.5, 3.5, 4] },
  late: { label: "Late night", gains: [-3, -3, -2, 0, 0, 1, 2, 1.5, -1, -2] },
};

const EQ_KEY = "velmora.music.eq";
const EQ_STATE_KEY = "velmora.music.eq.v2";

interface EqState {
  /** A preset key, "custom", or "user:<name>". */
  key: string;
  bands: EqBand[];
  /** dB before the bands. */
  preamp: number;
  /** Lower the preamp by the biggest boost, so a boost can't clip. */
  autoPreamp: boolean;
  /** The user's own saved curves. */
  saved: { name: string; bands: EqBand[] }[];
}

function cleanBands(list: unknown): EqBand[] {
  if (!Array.isArray(list) || list.length !== EQ_FREQS.length) return flatBands();
  return list.map((b, i) => {
    const o = (b ?? {}) as Partial<EqBand>;
    return {
      f: Math.min(20000, Math.max(20, Number(o.f) || EQ_FREQS[i])),
      g: Math.min(EQ_RANGE, Math.max(-EQ_RANGE, Number(o.g) || 0)),
      q: Math.min(8, Math.max(0.2, Number(o.q) || EQ_Q)),
    };
  });
}

function readEqState(): EqState {
  try {
    const raw = JSON.parse(localStorage.getItem(EQ_STATE_KEY) ?? "null") as Partial<EqState> | null;
    if (raw) {
      return {
        key: typeof raw.key === "string" ? raw.key : "custom",
        bands: cleanBands(raw.bands),
        preamp: Math.min(12, Math.max(-12, Number(raw.preamp) || 0)),
        autoPreamp: raw.autoPreamp !== false,
        saved: Array.isArray(raw.saved)
          ? raw.saved.filter((x) => x && typeof x.name === "string").map((x) => ({ name: x.name.slice(0, 40), bands: cleanBands(x.bands) }))
          : [],
      };
    }
    // The preset picked before parametric bands existed.
    const old = localStorage.getItem(EQ_KEY) ?? "flat";
    const key = EQ_PRESETS[old] ? old : "flat";
    return { key, bands: bandsOf(EQ_PRESETS[key].gains), preamp: 0, autoPreamp: true, saved: [] };
  } catch {
    return { key: "flat", bands: flatBands(), preamp: 0, autoPreamp: true, saved: [] };
  }
}

/** The preamp in force: the user's, or less the biggest boost when automatic. */
export function eqPreamp(e: { bands: EqBand[]; preamp: number; autoPreamp: boolean }): number {
  const boost = Math.max(0, ...e.bands.map((b) => b.g));
  return e.autoPreamp ? e.preamp - boost : e.preamp;
}

function eqFilter(e: EqState): string {
  const parts: string[] = [];
  const pre = eqPreamp(e);
  if (Math.abs(pre) >= 0.05) parts.push(`volume=${pre.toFixed(2)}dB`);
  e.bands.forEach((b, i) => {
    if (Math.abs(b.g) < 0.05) return;
    const kind = i === 0 ? "bass" : i === e.bands.length - 1 ? "treble" : "equalizer";
    parts.push(`${kind}=f=${Math.round(b.f)}:t=q:w=${b.q.toFixed(2)}:g=${b.g.toFixed(2)}`);
  });
  return parts.length ? `@eq:lavfi=[${parts.join(",")}]` : "";
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
  /** The equalizer: bands, preamp, which curve, and saved curves. */
  eqState = $state<EqState>(readEqState());
  /** Which curve is in use: a preset key, "custom" or "user:<name>". */
  get eq(): string {
    return this.eqState.key;
  }
  /** When the queue runs out, keep going with songs like the last few. */
  autoplay = $state(readAutoplay());
  /** Songs autoplay added. Up next lists them under their own heading. */
  readonly auto = new SvelteSet<string>();
  /** A movie or show has the window's mpv; Music is parked until played again. */
  yielded = $state(false);
  /** Crossfade, quality, pre-cache, speed. */
  prefs = $state<PlaybackPrefs>(readPrefs());
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
  /** The deck playing now. */
  private deck: Deck = "a";
  private ready: Record<Deck, boolean> = { a: false, b: false };
  private cleanups: Record<Deck, (() => void)[]> = { a: [], b: [] };
  /** The next song, already in mpv's playlist for a gapless step. */
  private ahead: { id: string; deck: Deck; queued: boolean } | null = null;
  /** The next song, loading paused on the other deck for a crossfade. */
  private pending: { id: string; deck: Deck; ready: boolean } | null = null;
  private fade: { from: Deck; to: Deck; timer: ReturnType<typeof setInterval> } | null = null;
  private precacheRun = 0;
  /** Each deck's normalization gain (amplitude, 1 = none) for the song on it. */
  private gain: Record<Deck, number> = { a: 1, b: 1 };
  private loud = new Map<string, number | null>();
  private streams = new Map<string, { s: MusicStream; at: number }>();
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

  /** The deck playing now has its mpv running. */
  private get owned(): boolean {
    return this.ready[this.deck];
  }

  // ── mpv: two decks ───────────────────────────────────────────────

  private label(d: Deck): string | undefined {
    return d === "a" ? undefined : DECK_B;
  }

  private other(d: Deck): Deck {
    return d === "a" ? "b" : "a";
  }

  /** mpv's volume for a deck: the slider, with that song's normalization.
   *  mpv's volume is cubic, so a gain goes in as its cube root. */
  private levelFor(d: Deck, amount = 1): number {
    return Math.min(sound.maxVolume, this.volume * Math.cbrt(Math.max(0, this.gain[d] * amount)));
  }

  /** Normalization gain for a song, from YouTube's loudness figure. Loud
   *  songs come down to YouTube's level; quiet ones come up a little. */
  private async gainFor(item: MusicItem): Promise<number> {
    if (!this.prefs.normalize) return 1;
    let db = this.loud.get(item.id);
    if (db === undefined) {
      const key = `loud:${item.id}`;
      const saved = await api.musicCacheGet<{ db: number | null }>(key).catch(() => null);
      if (saved) db = saved.db;
      else {
        db = await Promise.race([
          api.musicLoudness(item.id).catch(() => null),
          new Promise<null>((r) => setTimeout(() => r(null), 4000)),
        ]);
        if (db !== null) api.musicCachePut(key, { db }).catch(() => {});
      }
      this.loud.set(item.id, db ?? null);
    }
    if (db == null) return 1;
    const change = Math.min(LIFT_MAX_DB, Math.max(-CUT_MAX_DB, -db));
    return 10 ** (change / 20);
  }

  /** Put a song's gain on a deck and set its volume to match. */
  private async applyGain(d: Deck, item: MusicItem) {
    const g = await this.gainFor(item);
    this.gain[d] = g;
    if (this.ready[d] && !this.fade && d === this.deck) setProperty("volume", this.levelFor(d), this.label(d)).catch(() => {});
  }

  private config(d: Deck, withControls: boolean): MpvConfig {
    const options: Record<string, string | number | boolean> = {
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
      // A song queued in mpv's own playlist follows with no gap.
      "gapless-audio": "yes",
      "prefetch-playlist": "yes",
      volume: d === this.deck ? this.volume : 0,
      "volume-max": sound.maxVolume,
      af: this.filters(),
      speed: this.prefs.speed,
      "audio-pitch-correction": this.prefs.keepPitch ? "yes" : "no",
      "audio-device": this.prefs.device,
      "audio-exclusive": this.prefs.exclusive ? "yes" : "no",
    };
    if (d === "a") {
      options.vo = "gpu-next";
      options.hwdec = "auto-safe";
      // Windows' media controls come from media.rs, so mpv's own stay off
      // (they'd be a second tile). The flag is kept for the init retry.
      void withControls;
    } else {
      // Audio only: the plugin then needs no window for it.
      options.vid = "no";
    }
    return { initialOptions: options, observedProperties: OBSERVED };
  }

  /** Events from one deck. The plugin sends deck b's to the label "music-b";
   *  listening with that target is how the main page hears them. */
  private listenDeck(d: Deck, cb: (ev: MpvEvent) => void) {
    if (d === "a") return listenEvents(cb);
    return listen<MpvEvent>(`mpv-event-${DECK_B}`, (e) => cb(e.payload), { target: DECK_B });
  }

  private async ensureDeck(d: Deck) {
    if (this.ready[d]) return;
    // Listen before mpv starts: it reports each observed property once right
    // after init, and a listener added later misses that first report (which
    // left the play button showing Play while a song played).
    this.cleanups[d].push(await this.listenDeck(d, (ev) => this.onDeckEvent(d, ev)));
    try {
      if (d === "a") {
        try {
          await startMpv(this.config("a", true));
        } catch {
          await startMpv(this.config("a", false));
        }
      } else {
        await init(this.config("b", false), DECK_B);
      }
    } catch (e) {
      this.release(d);
      throw e;
    }
    this.ready[d] = true;
  }

  private onDeckEvent(d: Deck, ev: MpvEvent) {
    if (d !== this.deck) {
      // The other deck only matters while it gets the next song ready.
      const p = this.pending;
      if (p?.deck === d && ev.event === "file-loaded") p.ready = true;
      else if (p?.deck === d && ev.event === "end-file" && (ev as { reason?: string }).reason === "error") this.pending = null;
      return;
    }
    switch (ev.event) {
      case "property-change": {
        const { name, data } = ev as { name: string; data: unknown };
        this.onProperty(d, name, data);
        return;
      }
      case "file-loaded":
        this.failures = 0;
        // A resume point is for one song, not the ones queued after it.
        setProperty("start", "none", this.label(d)).catch(() => {});
        this.schedulePrecache();
        return;
      case "end-file": {
        const reason = (ev as { reason?: string }).reason;
        if (reason === "eof") this.onEnded();
        else if (reason === "error") this.onFailed();
        return;
      }
    }
  }

  private onProperty(d: Deck, name: string, data: unknown) {
    switch (name) {
      case "pause":
        this.playing = !(data as boolean);
        break;
      case "vo-configured":
        this.videoShown = d === "a" && this.video && !!data;
        break;
      case "time-pos": {
        const t = (data as number | null) ?? 0;
        // Only steady playback counts as listening, not a seek.
        if (this.playing && t > this.lastPos && t - this.lastPos < 3) this.heardFor += t - this.lastPos;
        this.lastPos = t;
        this.position = t;
        this.maybePrefetch();
        this.maybeCrossfade();
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
  }

  /** Read the real pause state from mpv, in case an update was missed. */
  private async syncPause() {
    if (!this.owned) return;
    try {
      this.playing = !(await getProperty("pause", "flag", this.label(this.deck)));
    } catch {
      // Keep what we have.
    }
  }

  /** The video player is taking the window's mpv over. Remember where we were. */
  async yieldToVideo() {
    if (!this.ready.a && !this.ready.b) return;
    this.endFade();
    this.clearPending();
    this.ahead = null;
    this.resumeAt = this.position;
    this.playing = false;
    this.videoShown = false;
    this.yielded = true;
    if (this.ready.b) command("stop", [], DECK_B).catch(() => {});
    // Movies and shows share the device with the rest of Windows.
    if (this.prefs.exclusive && this.ready.a) {
      setProperty("audio-exclusive", "no")
        .then(() => command("ao-reload"))
        .catch(() => {});
    }
    this.deck = "a";
    this.release("a");
  }

  private release(d: Deck) {
    this.cleanups[d].forEach((f) => f());
    this.cleanups[d] = [];
    this.ready[d] = false;
  }

  /** Each deck that's running, with its label. */
  private running(): (string | undefined)[] {
    return (["a", "b"] as Deck[]).filter((d) => this.ready[d]).map((d) => this.label(d));
  }

  // ── Sound ────────────────────────────────────────────────────────

  /** The audio filter chain: equalizer, the shared sound options, and the
   *  visualizer's loudness meter while the full view is open. */
  private filters(): string {
    return [eqFilter(this.eqState), sound.filters(), this.viz ? "@viz:lavfi=[astats=metadata=1:reset=1]" : ""]
      .filter(Boolean)
      .join(",");
  }

  /** Re-apply the chain after a sound setting changed. */
  applyFilters() {
    const af = this.filters();
    for (const l of this.running()) setProperty("af", af, l).catch(() => {});
  }

  private saveEq() {
    try {
      localStorage.setItem(EQ_STATE_KEY, JSON.stringify($state.snapshot(this.eqState)));
    } catch {
      // Not remembered.
    }
  }

  /** Dragging a band re-applies at most this often; mpv rebuilds the filter each time. */
  private eqTimer: ReturnType<typeof setTimeout> | undefined;
  private applyEqSoon() {
    if (this.eqTimer) return;
    this.eqTimer = setTimeout(() => {
      this.eqTimer = undefined;
      this.applyFilters();
    }, 90);
  }

  /** Pick a built-in curve ("bass") or a saved one ("user:Car"). */
  setEq(key: string) {
    let bands: EqBand[] | null = null;
    if (EQ_PRESETS[key]) bands = bandsOf(EQ_PRESETS[key].gains);
    else if (key.startsWith("user:")) bands = this.eqState.saved.find((x) => `user:${x.name}` === key)?.bands ?? null;
    if (!bands) return;
    this.eqState = { ...this.eqState, key, bands: bands.map((b) => ({ ...b })) };
    this.saveEq();
    this.applyFilters();
  }

  /** Move one band. Any change makes the curve "custom". */
  setBand(i: number, patch: Partial<EqBand>) {
    const bands = cleanBands(this.eqState.bands.map((b, j) => (j === i ? { ...b, ...patch } : b)));
    this.eqState = { ...this.eqState, key: "custom", bands };
    this.saveEq();
    this.applyEqSoon();
  }

  setPreamp(db: number, auto = this.eqState.autoPreamp) {
    this.eqState = { ...this.eqState, preamp: Math.min(12, Math.max(-12, db)), autoPreamp: auto };
    this.saveEq();
    this.applyEqSoon();
  }

  /** Keep the current curve under a name. The same name replaces it. */
  saveEqAs(name: string) {
    const n = name.trim().slice(0, 40);
    if (!n) return;
    const saved = [...this.eqState.saved.filter((x) => x.name !== n), { name: n, bands: this.eqState.bands.map((b) => ({ ...b })) }];
    this.eqState = { ...this.eqState, saved, key: `user:${n}` };
    this.saveEq();
  }

  deleteEq(name: string) {
    const saved = this.eqState.saved.filter((x) => x.name !== name);
    const key = this.eqState.key === `user:${name}` ? "custom" : this.eqState.key;
    this.eqState = { ...this.eqState, saved, key };
    this.saveEq();
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
    if (this.ready.a) setVideoMarginRatio(margins).catch(() => {});
  }

  private applySpeed() {
    for (const l of this.running()) {
      setProperty("audio-pitch-correction", this.prefs.keepPitch ? "yes" : "no", l).catch(() => {});
      setProperty("speed", this.prefs.speed, l).catch(() => {});
    }
  }

  /** Output devices, from mpv. Starts the audio-only player if nothing is running. */
  async devices(): Promise<AudioDevice[]> {
    const d: Deck = this.ready.a ? "a" : "b";
    await this.ensureDeck(d);
    const list = (await getProperty<AudioDevice[]>("audio-device-list", "node", this.label(d)).catch(() => null)) ?? [];
    return list.filter((x) => x && typeof x.name === "string");
  }

  /** Switch off everything that changes the sound, so exclusive mode is bit-perfect. */
  bitPerfect() {
    this.setEq("flat");
    this.eqState = { ...this.eqState, preamp: 0 };
    this.saveEq();
    for (const o of ["dialogue", "night", "normalize"] as const) if (sound[o]) sound.set(o, false);
    this.setPrefs({ normalize: false, speed: 1 });
    this.setVolume(100);
    this.applyFilters();
  }

  /** Change Music → Playback settings; they apply straight away. */
  setPrefs(patch: Partial<PlaybackPrefs>) {
    const before = this.prefs;
    this.prefs = cleanPrefs({ ...before, ...patch });
    try {
      localStorage.setItem(PREFS_KEY, JSON.stringify(this.prefs));
    } catch {
      // Not remembered.
    }
    if (this.prefs.speed !== before.speed || this.prefs.keepPitch !== before.keepPitch) this.applySpeed();
    if (this.prefs.crossfade !== before.crossfade) {
      this.clearPending();
      this.dropAhead(true);
      // Let the next song be got ready again the new way.
      this.prefetching = false;
    }
    if (this.prefs.quality !== before.quality) this.streams.clear();
    if (this.prefs.device !== before.device) {
      for (const l of this.running()) setProperty("audio-device", this.prefs.device, l).catch(() => {});
    }
    if (this.prefs.exclusive !== before.exclusive) {
      this.clearPending();
      this.prefetching = false;
      for (const l of this.running()) {
        setProperty("audio-exclusive", this.prefs.exclusive ? "yes" : "no", l)
          .then(() => command("ao-reload", [], l))
          .catch(() => {});
      }
    }
    if (this.prefs.normalize !== before.normalize) {
      const cur = this.current;
      if (cur) this.applyGain(this.deck, cur);
      else this.gain = { a: 1, b: 1 };
    }
    if (this.prefs.precache && !before.precache) this.schedulePrecache();
    if (!this.prefs.precache) this.precacheRun++;
  }

  // ── Streams ──────────────────────────────────────────────────────

  private async stream(item: MusicItem, video: boolean): Promise<MusicStream> {
    const key = `${item.id}|${video}|${this.prefs.quality}`;
    const hit = this.streams.get(key);
    if (hit && Date.now() - hit.at < STREAM_TTL) return hit.s;
    const s = await api.musicStream(item.id, video, undefined, this.prefs.quality);
    this.streams.set(key, { s, at: Date.now() });
    return s;
  }

  /** Where a song plays from: a copy saved ahead of time if there is one,
   *  otherwise a fresh stream. Music videos always stream. */
  private async sourceFor(item: MusicItem, video: boolean): Promise<MusicStream> {
    if (!video) {
      const saved = await api.musicCached(item.id).catch(() => null);
      if (saved) return localStream(item, saved);
    }
    return this.stream(item, video);
  }

  private async load(startAt = 0) {
    const item = this.current;
    if (!item) return;
    const seq = ++this.loadSeq;
    this.yielded = false;
    this.endFade();
    this.clearPending();
    this.ahead = null;
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
    // A music video needs the window's own mpv.
    if (this.video && this.deck === "b") {
      command("stop", [], DECK_B).catch(() => {});
      this.deck = "a";
    }
    const d = this.deck;
    const l = this.label(d);
    // The loudness lookup runs alongside yt-dlp; the song waits only for the slower.
    const gain = this.gainFor(item);
    const setLevel = async () => {
      this.gain[d] = await gain;
      await setProperty("volume", this.levelFor(d), l).catch(() => {});
    };
    try {
      await this.ensureDeck(d);
      if (item.localPath) {
        // A downloaded song plays from this PC, no network needed.
        const local = await api.openLocal(item.localPath);
        if (seq !== this.loadSeq) return;
        await setProperty("http-header-fields", "", l).catch(() => {});
        await setProperty("audio-files", "", l).catch(() => {});
        await setProperty("vid", "no", l).catch(() => {});
        await setProperty("start", startAt > 0 ? String(startAt) : "none", l);
        await setLevel();
        if (seq !== this.loadSeq) return;
        await command("loadfile", [local.url], l);
        await setProperty("pause", false, l);
        this.playing = true;
        return;
      }
      const s = await this.sourceFor(item, this.video);
      if (seq !== this.loadSeq) return;
      const { ua, fields } = headerOptions(s);
      if (ua) await setProperty("user-agent", ua, l).catch(() => {});
      await setProperty("http-header-fields", fields, l).catch(() => {});
      await setProperty("start", startAt > 0 ? String(startAt) : "none", l);
      const withVideo = d === "a" && this.video && !!s.videoUrl;
      if (this.video && !s.videoUrl) {
        this.video = false;
        this.error = "This song has no video, so it plays as audio.";
      }
      await setProperty("vid", withVideo ? "auto" : "no", l).catch(() => {});
      if (d === "a") await setProperty("force-window", withVideo ? "yes" : "no", l).catch(() => {});
      await setLevel();
      if (seq !== this.loadSeq) return;
      if (withVideo && s.videoUrl && s.videoUrl !== s.audioUrl) {
        await setProperty("audio-files", s.audioUrl, l).catch(() => {});
        await command("loadfile", [s.videoUrl], l);
      } else {
        await setProperty("audio-files", "", l).catch(() => {});
        await command("loadfile", [withVideo && s.videoUrl ? s.videoUrl : s.audioUrl], l);
      }
      await setProperty("pause", false, l);
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

  /** Get the next song ready ahead of time: resolve it, and without
   *  crossfade queue it in mpv so it follows with no gap. */
  private maybePrefetch() {
    if (this.prefetching || !this.duration || this.duration - this.position > PREFETCH_AT) return;
    const next = this.queue[this.index + 1];
    if (!next) {
      this.extend();
      return;
    }
    this.prefetching = true;
    if (!next.localPath) this.stream(next, this.video).catch(() => {});
    this.appendNext();
  }

  /** Gapless: put the next song in mpv's own playlist. mpv opens it early
   *  and moves on with no silence; `onEnded` then just follows along. */
  private async appendNext() {
    const next = this.queue[this.index + 1];
    if (!next || this.ahead || this.video || this.repeat === "one" || (this.prefs.crossfade > 0 && !this.prefs.exclusive) || !this.owned) return;
    const d = this.deck;
    const seq = this.loadSeq;
    const mark = { id: next.id, deck: d, queued: false };
    this.ahead = mark;
    try {
      const url = next.localPath ? (await api.openLocal(next.localPath)).url : (await this.sourceFor(next, false)).audioUrl;
      if (seq !== this.loadSeq || this.ahead !== mark || this.queue[this.index + 1]?.id !== next.id) return;
      await command("loadfile", [url, "append"], this.label(d));
      mark.queued = true;
    } catch {
      if (this.ahead === mark) this.ahead = null;
    }
  }

  /** The queue changed: take back a queued song that's no longer next. */
  private dropAhead(force = false) {
    const next = this.queue[this.index + 1]?.id;
    const a = this.ahead;
    if (a && (force || a.id !== next)) {
      this.ahead = null;
      if (a.queued) command("playlist-clear", [], this.label(a.deck)).catch(() => {});
      this.prefetching = false;
    }
    if (this.pending && (force || this.pending.id !== next)) this.clearPending();
  }

  // ── Crossfade ────────────────────────────────────────────────────

  /** Near the end: load the next song paused on the other deck, then, once
   *  it's ready, fade the two against each other. */
  private maybeCrossfade() {
    const cf = this.prefs.crossfade;
    // Exclusive mode gives the device to one player, so there's no second one to fade in.
    if (cf <= 0 || this.prefs.exclusive || this.fade || this.video || !this.playing || this.repeat === "one" || !this.duration) return;
    const next = this.queue[this.index + 1];
    if (!next) return;
    const left = (this.duration - this.position) / this.prefs.speed;
    if (!this.pending && left <= cf + CROSSFADE_LEAD) {
      this.prepare(next);
      return;
    }
    // A song shorter than the fade just fades for as long as it has.
    if (this.pending?.ready && this.pending.id === next.id && left <= cf) this.startFade(Math.max(1, Math.min(cf, left)));
  }

  private async prepare(next: MusicItem) {
    const d = this.other(this.deck);
    const p = { id: next.id, deck: d, ready: false };
    this.pending = p;
    try {
      await this.ensureDeck(d);
      const s = next.localPath ? localStream(next, (await api.openLocal(next.localPath)).url) : await this.sourceFor(next, false);
      this.gain[d] = await this.gainFor(next);
      if (this.pending !== p) return;
      const l = this.label(d);
      const { ua, fields } = headerOptions(s);
      if (ua) await setProperty("user-agent", ua, l).catch(() => {});
      await setProperty("http-header-fields", fields, l).catch(() => {});
      await setProperty("volume", 0, l).catch(() => {});
      await setProperty("pause", true, l).catch(() => {});
      await setProperty("start", "none", l).catch(() => {});
      await setProperty("audio-files", "", l).catch(() => {});
      await setProperty("vid", "no", l).catch(() => {});
      await command("loadfile", [s.audioUrl], l);
    } catch {
      if (this.pending === p) this.pending = null;
    }
  }


  private startFade(seconds: number) {
    const p = this.pending;
    if (!p) return;
    this.pending = null;
    const from = this.deck;
    const to = p.deck;
    const lf = this.label(from);
    const lt = this.label(to);
    this.logPlay();
    this.deck = to;
    this.adopt(this.index + 1);
    this.playing = true;
    setProperty("pause", false, lt).catch(() => {});
    // The new deck reported its length while it waited; ask again.
    getProperty("duration", "double", lt)
      .then((v) => {
        if (v && this.deck === to) this.duration = v as number;
      })
      .catch(() => {});
    const started = performance.now();
    const ms = seconds * 1000;
    const timer = setInterval(() => {
      const t = Math.min(1, (performance.now() - started) / ms);
      // Equal power: together the two stay at a steady loudness.
      setProperty("volume", this.levelFor(from, Math.cos((t * Math.PI) / 2)), lf).catch(() => {});
      setProperty("volume", this.levelFor(to, Math.sin((t * Math.PI) / 2)), lt).catch(() => {});
      if (t >= 1) this.endFade();
    }, FADE_TICK);
    this.fade = { from, to, timer };
    // The new song loaded while its deck waited, so nothing saved the ones after it yet.
    this.schedulePrecache();
  }

  /** Finish (or cut short) a fade: the old song stops, the new one is at full volume. */
  private endFade() {
    const f = this.fade;
    if (!f) return;
    clearInterval(f.timer);
    this.fade = null;
    command("stop", [], this.label(f.from)).catch(() => {});
    setProperty("volume", this.levelFor(f.to), this.label(f.to)).catch(() => {});
  }

  private clearPending() {
    const p = this.pending;
    if (!p) return;
    this.pending = null;
    if (this.ready[p.deck]) command("stop", [], this.label(p.deck)).catch(() => {});
  }

  /** Playback moved to song `i` by itself (gapless or a crossfade). */
  private adopt(i: number) {
    const item = this.queue[i];
    if (!item) return;
    this.index = i;
    this.heard = item;
    this.heardFor = 0;
    this.lastPos = 0;
    this.loadedId = item.id;
    this.position = 0;
    this.duration = item.duration ?? 0;
    this.error = null;
    this.prefetching = false;
    this.persist();
    // Gapless: the next song is on the same deck, so its level changes here.
    if (!this.fade) this.applyGain(this.deck, item);
  }

  // ── Saved ahead ──────────────────────────────────────────────────

  /** A few seconds into a song, save the next three on disk. */
  private schedulePrecache() {
    if (!this.prefs.precache) return;
    const run = ++this.precacheRun;
    setTimeout(async () => {
      for (const item of this.queue.slice(this.index + 1, this.index + 1 + PRECACHE_AHEAD)) {
        if (run !== this.precacheRun || !this.prefs.precache) return;
        if (item.localPath) continue;
        await api.musicPrecache(item.id, this.prefs.quality, this.prefs.cacheMb).catch(() => null);
      }
    }, PRECACHE_DELAY);
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
    this.dropAhead();
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
    api.musicQueueSave(this.snapshot()).catch(() => {});
  }

  private snapshot(): SavedQueue {
    return {
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
  }

  /** At launch: put back the last queue, paused where it stopped. */
  async restore() {
    const s = await api.musicQueueLoad<SavedQueue>().catch(() => null);
    if (s) this.applySaved(s);
  }

  /** Put a saved queue in place, paused. Does nothing while something plays. */
  private applySaved(s: SavedQueue) {
    if (!Array.isArray(s.queue) || !s.queue.length || this.active) return;
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
    // Pausing mid-crossfade cuts the old song rather than leave it fading.
    this.endFade();
    const l = this.label(this.deck);
    // Ask mpv rather than trust our copy, so the button always does what it shows.
    let paused = !this.playing;
    try {
      paused = (await getProperty("pause", "flag", l)) as boolean;
    } catch {
      // Use our copy.
    }
    await setProperty("pause", !paused, l).catch(() => {});
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
    this.endFade();
    return command("seek", [Math.max(0, to), "absolute+exact"], this.label(this.deck)).catch(() => {});
  }

  setVolume(v: number) {
    this.volume = Math.max(0, Math.min(sound.maxVolume, v));
    try {
      localStorage.setItem(VOLUME_KEY, String(this.volume));
    } catch {
      // Not remembered.
    }
    // During a crossfade the ramp sets both decks from this.volume.
    if (this.owned && !this.fade) setProperty("volume", this.levelFor(this.deck), this.label(this.deck)).catch(() => {});
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
    this.dropAhead();
    this.persist();
  }

  cycleRepeat() {
    this.repeat = this.repeat === "off" ? "all" : this.repeat === "all" ? "one" : "off";
    // Repeat one replays the song, so nothing should follow it.
    if (this.repeat === "one") this.dropAhead(true);
    this.persist();
  }

  /** Switch between audio and the music video, from the same spot. */
  async setVideo(on: boolean) {
    if (this.video === on) return;
    this.video = on;
    if (!on) this.videoShown = false;
    if (this.active) await this.load(this.position);
  }

  /** The mini player's ×: stop and clear, but hand back an undo that puts
   *  the queue back, paused at the same spot. */
  async dismiss(): Promise<() => void> {
    const snap = this.snapshot();
    await this.close();
    return () => {
      this.applySaved(snap);
      this.persist();
    };
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
    this.endFade();
    this.clearPending();
    this.ahead = null;
    this.precacheRun++;
    this.deck = "a";
    if (this.ready.b) {
      this.release("b");
      await destroy(DECK_B).catch(() => {});
    }
    if (this.ready.a) {
      this.release("a");
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
    // Gapless: mpv already moved on to the song queued in its playlist.
    const a = this.ahead;
    this.ahead = null;
    if (a?.queued && a.deck === this.deck && this.queue[this.index + 1]?.id === a.id) {
      this.adopt(this.index + 1);
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
