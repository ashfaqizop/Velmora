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
  listenEvents,
  observeProperties,
  setProperty,
  type MpvConfig,
  type MpvObservableProperty,
} from "tauri-plugin-libmpv-api";
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
] as const satisfies MpvObservableProperty[];

/** googlevideo URLs last about six hours; use a resolved one for five. */
const STREAM_TTL = 5 * 60 * 60 * 1000;
/** Resolve the next track this long before the current one ends. */
const PREFETCH_AT = 25;
const VOLUME_KEY = "velmora.music.volume";

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
  error = $state<string | null>(null);
  /** Where the queue came from, for the "Playing from" line. */
  source = $state<string | null>(null);

  /** The order before shuffle, to restore it. */
  private ordered: MusicItem[] = [];
  /** Radio continuation, so the queue keeps going after the last song. */
  private radioNext: string | null = null;
  private owned = false;
  private cleanups: (() => void)[] = [];
  private streams = new Map<string, { s: MusicStream; at: number; video: boolean }>();
  private loadSeq = 0;
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
      af: sound.filters(),
    };
    // Media keys and the Windows media overlay, on mpv builds that have them.
    if (withControls) options["media-controls"] = "yes";
    return { initialOptions: options, observedProperties: OBSERVED };
  }

  private async ensureMpv() {
    if (this.owned) return;
    try {
      await startMpv(this.config(true));
    } catch {
      await startMpv(this.config(false));
    }
    this.owned = true;
    this.cleanups.push(
      await observeProperties(OBSERVED, ({ name, data }) => {
        switch (name) {
          case "pause":
            this.playing = !(data as boolean);
            break;
          case "time-pos":
            this.position = (data as number | null) ?? 0;
            this.maybePrefetch();
            break;
          case "duration":
            if (data) this.duration = data as number;
            break;
          case "paused-for-cache":
            this.buffering = data as boolean;
            break;
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
  }

  /** The video player is taking mpv over. Remember where we were. */
  async yieldToVideo() {
    if (!this.owned) return;
    this.resumeAt = this.position;
    this.playing = false;
    this.release();
  }

  private release() {
    this.cleanups.forEach((f) => f());
    this.cleanups = [];
    this.owned = false;
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
    this.loading = true;
    this.error = null;
    this.position = startAt;
    this.duration = item.duration ?? 0;
    try {
      await this.ensureMpv();
      const s = await this.stream(item, this.video);
      if (seq !== this.loadSeq) return;
      const { ua, fields } = headerOptions(s);
      if (ua) await setProperty("user-agent", ua).catch(() => {});
      await setProperty("http-header-fields", fields).catch(() => {});
      await setProperty("start", startAt > 0 ? String(startAt) : "none");
      const withVideo = this.video && !!s.videoUrl;
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
    if (this.prefetching || !this.duration || this.duration - this.position > PREFETCH_AT) return;
    const next = this.queue[this.index + 1];
    if (!next) {
      this.extendRadio();
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
    this.radioNext = null;
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

  /** Play one song, then a radio of songs like it. */
  async playSong(item: MusicItem, source: string | null = null) {
    await this.playList([item], 0, source);
    this.extendRadio(item.id);
  }

  /** Start YouTube Music's own queue for a playlist, shuffle or radio button. */
  async playTarget(target: { videoId: string | null; playlistId: string | null; params: string | null }, source: string | null = null) {
    const page = await api.musicQueue(target.videoId, target.playlistId, target.params);
    this.radioNext = page.next;
    await this.playList(page.tracks, 0, source);
    this.radioNext = page.next;
  }

  /** Append more songs like the last one when the queue runs out. */
  private radioBusy = false;
  private async extendRadio(seed?: string) {
    if (this.radioBusy || this.repeat !== "off") return;
    this.radioBusy = true;
    try {
      const page = this.radioNext
        ? await api.musicQueueMore(this.radioNext)
        : await api.musicRadio(seed ?? this.queue[this.queue.length - 1]?.id ?? "");
      const have = new Set(this.queue.map((q) => q.id));
      const fresh = page.tracks.filter((t) => !have.has(t.id));
      this.queue = [...this.queue, ...fresh];
      this.ordered = [...this.ordered, ...fresh];
      this.radioNext = page.next;
    } catch {
      // The queue just ends.
    } finally {
      this.radioBusy = false;
    }
  }

  playNext(item: MusicItem) {
    if (!this.active) return this.playSong(item);
    const q = [...this.queue];
    q.splice(this.index + 1, 0, item);
    this.queue = q;
  }

  addToQueue(item: MusicItem) {
    if (!this.active) return this.playSong(item);
    this.queue = [...this.queue, item];
  }

  removeAt(i: number) {
    if (i === this.index) return;
    const q = [...this.queue];
    q.splice(i, 1);
    this.queue = q;
    if (i < this.index) this.index -= 1;
  }

  jump(i: number) {
    if (i < 0 || i >= this.queue.length) return;
    this.index = i;
    return this.load();
  }

  // ── Controls ─────────────────────────────────────────────────────

  async toggle() {
    if (!this.active) return;
    if (!this.owned) return this.load(this.resumeAt);
    await setProperty("pause", this.playing);
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
    await this.extendRadio();
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
      this.resumeAt = to;
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
  }

  cycleRepeat() {
    this.repeat = this.repeat === "off" ? "all" : this.repeat === "all" ? "one" : "off";
  }

  /** Switch between audio and the music video, from the same spot. */
  async setVideo(on: boolean) {
    if (this.video === on) return;
    this.video = on;
    if (this.active) await this.load(this.position);
  }

  /** Stop and clear everything. */
  async close() {
    this.loadSeq += 1;
    this.queue = [];
    this.index = -1;
    this.playing = false;
    this.position = 0;
    this.duration = 0;
    if (this.owned) {
      this.release();
      await stopMpv();
    }
  }

  private onEnded() {
    if (this.repeat === "one") {
      this.load();
      return;
    }
    this.next();
  }

  private failures = 0;
  private onFailed() {
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
