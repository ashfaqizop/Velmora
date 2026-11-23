// The built-in video player, for PCs where mpv can't run.
//
// ffmpeg (webvideo.rs) turns the source into a fragmented MP4 that a
// <video> element plays as it arrives. This class drives that and answers
// to the mpv property and command names the player page uses (pause, seek,
// aid, sid, sub-add, volume, af...), and reports the same properties mpv
// would (time-pos, duration, track-list, eof-reached...). So the player page
// runs it the way it runs mpv.
//
// A session starts at one point in the film. A seek inside what's loaded
// just moves the <video>; a seek further away starts a new session there.
// The film's time is the session's offset plus the <video>'s own clock.
import type { MpvEvent } from "tauri-plugin-libmpv-api";
import * as api from "./api";
import { AudioChain } from "./webchain";

interface Cue {
  start: number;
  end: number;
  text: string;
}

/** WebVTT (or SRT, which is close enough) to cues, `shift` seconds later. */
export function parseVtt(text: string, shift = 0): Cue[] {
  const cues: Cue[] = [];
  const time = (s: string) => {
    const m = s.trim().match(/^(?:(\d+):)?(\d{1,2}):(\d{2})[.,](\d{1,3})/);
    if (!m) return NaN;
    return Number(m[1] ?? 0) * 3600 + Number(m[2]) * 60 + Number(m[3]) + Number(m[4].padEnd(3, "0")) / 1000;
  };
  for (const block of text.replace(/\r/g, "").split(/\n\n+/)) {
    const lines = block.split("\n");
    const at = lines.findIndex((l) => l.includes("-->"));
    if (at < 0) continue;
    const [a, b] = lines[at].split("-->");
    const start = time(a);
    const end = time(b);
    if (!Number.isFinite(start) || !Number.isFinite(end)) continue;
    const body = lines
      .slice(at + 1)
      .join("\n")
      .replace(/<[^>]*>/g, "")
      .replace(/\{\\[^}]*\}/g, "")
      .replace(/&amp;/g, "&")
      .replace(/&lt;/g, "<")
      .replace(/&gt;/g, ">")
      .replace(/&nbsp;/g, " ")
      .trim();
    if (body) cues.push({ start: start + shift, end: end + shift, text: body });
  }
  return cues.sort((x, y) => x.start - y.start);
}

/** Codecs WebView2 plays from an MP4 as they are. */
const PLAYABLE_AUDIO = new Set(["aac", "mp3", "opus", "flac"]);

function playableVideo(v: api.WebProbe["video"]): boolean {
  if (!v) return true;
  if (v.codec === "h264") return /^yuvj?420p$/.test(v.pixFmt);
  return v.codec === "vp9" || v.codec === "av1" || v.codec === "vp8";
}

const langs = (list: string) =>
  list
    .split(",")
    .map((l) => l.trim().toLowerCase())
    .filter(Boolean);

interface SubTrack {
  /** mpv's id: the probe's subtitle streams first, then added files. */
  id: number;
  title?: string;
  lang?: string;
  /** A subtitle stream in the film (index among them), or a separate file. */
  stream?: { index: number; text: boolean };
  /** Cues for a separate file, in film time. */
  cues?: Cue[];
}

type PropCb = (ev: { name: string; data: unknown }) => void;
type EventCb = (ev: MpvEvent) => void;

class WebVideo {
  readonly kind = "builtin" as const;

  private stage: HTMLElement | null = null;
  private video: HTMLVideoElement | null = null;
  private subEl: HTMLDivElement | null = null;
  private ctx: AudioContext | null = null;
  private chain: AudioChain | null = null;

  private props = new Set<PropCb>();
  private events = new Set<EventCb>();

  private alang: string[] = [];
  private slang: string[] = [];
  private volume = 100;
  private muted = false;
  private af = "";

  private input: string | null = null;
  private probe: api.WebProbe | null = null;
  private loadSeq = 0;
  private startAt = 0;
  private wantPaused = false;
  private session: { token: string; offset: number; plan: api.WebPlan } | null = null;
  private firstLoad = false;
  private retries = 0;
  private audio = 0;
  private subs: SubTrack[] = [];
  private sid: number | null = null;
  /** Cues of the session's text subtitle in use, re-read as ffmpeg writes them. */
  private streamCues: Cue[] = [];
  private subTimer: ReturnType<typeof setInterval> | undefined;
  private frame = 0;
  private shownText = "";

  // ── Set up ────────────────────────────────────────────────────────

  /** Put the <video> into the player page and take the player's options. */
  attach(stage: HTMLElement, options: { alang?: string; slang?: string; volume?: number; af?: string }) {
    this.detach();
    this.stage = stage;
    this.alang = langs(options.alang ?? "");
    this.slang = langs(options.slang ?? "");
    this.volume = options.volume ?? 100;
    this.af = options.af ?? "";
    this.muted = false;

    const video = document.createElement("video");
    video.crossOrigin = "anonymous";
    video.playsInline = true;
    video.preload = "auto";
    video.className = "web-video";
    const sub = document.createElement("div");
    sub.className = "web-subs";
    stage.append(video, sub);
    this.video = video;
    this.subEl = sub;

    this.ctx ??= new AudioContext({ latencyHint: "playback" });
    this.chain = new AudioChain(this.ctx);
    this.ctx.createMediaElementSource(video).connect(this.chain.input);
    this.chain.setAf(this.af);
    this.chain.setMuted(this.muted, this.volume);
    this.wire(video);

    const tick = () => {
      this.renderSubs();
      this.frame = requestAnimationFrame(tick);
    };
    this.frame = requestAnimationFrame(tick);
  }

  /** Stop and take the <video> out. */
  detach() {
    this.loadSeq++;
    cancelAnimationFrame(this.frame);
    clearInterval(this.subTimer);
    if (this.video) {
      this.video.pause();
      this.video.removeAttribute("src");
      this.video.load();
    }
    this.stage?.replaceChildren();
    this.stage = this.video = this.subEl = null;
    this.chain = null;
    this.session = null;
    this.input = null;
    this.probe = null;
    this.subs = [];
    this.sid = null;
    this.streamCues = [];
    this.shownText = "";
    this.props.clear();
    this.events.clear();
  }

  observe(cb: PropCb): () => void {
    this.props.add(cb);
    return () => this.props.delete(cb);
  }

  listen(cb: EventCb): () => void {
    this.events.add(cb);
    return () => this.events.delete(cb);
  }

  private prop(name: string, data: unknown) {
    for (const cb of this.props) cb({ name, data });
  }

  private emit(ev: MpvEvent) {
    for (const cb of this.events) cb(ev);
  }

  /** The film's time now. */
  private get now(): number {
    return (this.session?.offset ?? 0) + (this.video?.currentTime ?? 0);
  }

  private get live(): boolean {
    return !this.probe?.duration;
  }

  private wire(video: HTMLVideoElement) {
    const mine = () => video === this.video;
    video.addEventListener("play", () => mine() && this.prop("pause", false));
    video.addEventListener("pause", () => mine() && this.session && this.prop("pause", true));
    video.addEventListener("timeupdate", () => mine() && this.prop("time-pos", this.now));
    video.addEventListener("waiting", () => mine() && this.prop("paused-for-cache", true));
    video.addEventListener("playing", () => {
      if (!mine()) return;
      this.retries = 0;
      this.prop("paused-for-cache", false);
    });
    video.addEventListener("loadeddata", () => {
      if (!mine() || !this.firstLoad) return;
      this.firstLoad = false;
      this.emit({ event: "file-loaded" } as MpvEvent);
    });
    video.addEventListener("ended", () => {
      if (!mine() || !this.session) return;
      const d = this.probe?.duration ?? 0;
      // ffmpeg stopped early (the connection dropped): pick up where it got to.
      if (d > 0 && this.now < d - 15 && this.retries < 3) {
        this.retries++;
        this.restart(this.now);
        return;
      }
      this.prop("eof-reached", true);
    });
    video.addEventListener("error", () => {
      if (!mine() || !this.session) return;
      if (this.firstLoad || this.retries >= 2) {
        this.emit({ event: "end-file", reason: "error" } as unknown as MpvEvent);
        return;
      }
      this.retries++;
      this.restart(this.now);
    });
  }

  // ── Sessions ──────────────────────────────────────────────────────

  private plan(start: number, burn: number | null = null): api.WebPlan {
    const p = this.probe!;
    const a = p.audio[this.audio];
    return {
      start,
      audio: p.audio.length ? this.audio : null,
      copyVideo: playableVideo(p.video) && burn === null,
      copyAudio: !a || PLAYABLE_AUDIO.has(a.codec),
      // Converting costs CPU; small PCs get 720p.
      maxHeight: (navigator.hardwareConcurrency ?? 4) <= 4 ? 720 : 1080,
      burnSub: burn,
      textSubs: p.subs.filter((s) => s.text).map((s) => s.index),
      live: this.live,
    };
  }

  private burnIndex(): number | null {
    const t = this.subs.find((s) => s.id === this.sid);
    return t?.stream && !t.stream.text ? t.stream.index : null;
  }

  private async open(start: number) {
    if (!this.input || !this.probe || !this.video) return;
    const seq = ++this.loadSeq;
    const plan = this.plan(start, this.burnIndex());
    const opened = await api.webVideoOpen(this.input, plan);
    if (seq !== this.loadSeq || !this.video) return;
    this.session = { token: opened.token, offset: opened.offset, plan };
    this.streamCues = [];
    this.video.src = opened.url;
    this.video.load();
    this.prop("time-pos", start);
    this.watchStreamSubs();
    if (!this.wantPaused) {
      await this.ctx?.resume().catch(() => {});
      this.video.play().catch(() => {});
    }
  }

  /** Start again from `at` (a far seek, another audio track, picture subtitles). */
  private restart(at: number) {
    this.open(Math.max(0, at)).catch(() => this.emit({ event: "end-file", reason: "error" } as unknown as MpvEvent));
  }

  private async loadfile(input: string) {
    if (!this.video) return;
    const seq = ++this.loadSeq;
    this.session = null;
    this.video.removeAttribute("src");
    this.video.load();
    this.input = input;
    this.retries = 0;
    this.firstLoad = true;
    try {
      const probe = await api.webVideoProbe(input);
      if (seq !== this.loadSeq) return;
      this.probe = probe;
    } catch (e) {
      if (seq !== this.loadSeq) return;
      console.warn("built-in player couldn't read the stream", e);
      this.emit({ event: "end-file", reason: "error" } as unknown as MpvEvent);
      return;
    }
    const p = this.probe;
    // Audio: the first in a preferred language, else the default.
    const byLang = this.alang.map((l) => p.audio.find((a) => a.lang?.toLowerCase() === l)).find(Boolean);
    this.audio = (byLang ?? p.audio.find((a) => a.default) ?? p.audio[0])?.index ?? 0;
    this.subs = p.subs.map((s) => ({
      id: s.index + 1,
      title: s.title ?? undefined,
      lang: s.lang ?? undefined,
      stream: { index: s.index, text: s.text },
    }));
    // Subtitles: a text track in a preferred language, as mpv's slang does.
    const pick = this.slang.map((l) => p.subs.find((s) => s.text && s.lang?.toLowerCase() === l)).find(Boolean);
    this.sid = pick ? pick.index + 1 : null;
    if (p.duration) this.prop("duration", p.duration);
    this.publishTracks();
    const start = this.startAt;
    this.startAt = 0;
    await this.open(start);
  }

  private seek(to: number) {
    const v = this.video;
    if (!v || !this.session || this.live) return;
    const d = this.probe?.duration ?? Infinity;
    const target = Math.max(0, Math.min(to, d - 1));
    const local = target - this.session.offset;
    for (let i = 0; i < v.buffered.length; i++) {
      if (local >= v.buffered.start(i) - 0.25 && local <= v.buffered.end(i)) {
        v.currentTime = Math.max(0, local);
        return;
      }
    }
    this.restart(target);
  }

  // ── Tracks and subtitles ──────────────────────────────────────────

  private publishTracks() {
    const p = this.probe;
    const list: unknown[] = [];
    if (p?.video) list.push({ id: 1, type: "video", selected: true, codec: p.video.codec });
    p?.audio.forEach((a) =>
      list.push({
        id: a.index + 1,
        type: "audio",
        title: a.title ?? undefined,
        lang: a.lang ?? undefined,
        selected: a.index === this.audio,
        "demux-channels": a.channels,
      }),
    );
    for (const s of this.subs) list.push({ id: s.id, type: "sub", title: s.title, lang: s.lang, selected: s.id === this.sid });
    this.prop("track-list", list);
  }

  /** While a subtitle track of the film is on, re-read what ffmpeg has written. */
  private watchStreamSubs() {
    clearInterval(this.subTimer);
    const t = this.subs.find((s) => s.id === this.sid);
    const s = this.session;
    if (!t?.stream?.text || !s) return;
    const read = async () => {
      const text = await api.webVideoSubs(s.token, t.stream!.index).catch(() => "");
      if (this.session !== s || this.sid !== t.id) return;
      this.streamCues = parseVtt(text, s.offset);
    };
    read();
    this.subTimer = setInterval(read, 1500);
  }

  private setSid(value: string) {
    const prevBurn = this.burnIndex();
    this.sid = value === "no" ? null : Number(value) || null;
    this.streamCues = [];
    this.publishTracks();
    // Picture subtitles are drawn by ffmpeg, so turning them on or off restarts.
    if (this.burnIndex() !== prevBurn && this.session) this.restart(this.now);
    else this.watchStreamSubs();
  }

  private async subAdd(url: string, flags: string, title?: string, lang?: string) {
    const cues = parseVtt(await api.webSubtitle(url));
    const id = Math.max(0, ...this.subs.map((s) => s.id)) + 1;
    this.subs.push({ id, title: title || undefined, lang: lang || undefined, cues });
    const wanted = flags === "select" || (this.sid === null && !!lang && this.slang.includes(lang.toLowerCase()));
    if (wanted) {
      this.sid = id;
      clearInterval(this.subTimer);
    }
    this.publishTracks();
  }

  private renderSubs() {
    const el = this.subEl;
    if (!el) return;
    const t = this.now;
    const track = this.subs.find((s) => s.id === this.sid);
    const cues = track?.cues ?? (track?.stream?.text ? this.streamCues : []);
    let text = "";
    if (cues.length) {
      // Cues are sorted by start; look at the ones that started already.
      let lo = 0;
      let hi = cues.length;
      while (lo < hi) {
        const mid = (lo + hi) >> 1;
        if (cues[mid].start <= t) lo = mid + 1;
        else hi = mid;
      }
      const on: string[] = [];
      for (let i = lo - 1; i >= 0 && i >= lo - 20; i--) if (cues[i].end > t) on.unshift(cues[i].text);
      text = on.join("\n");
    }
    if (text !== this.shownText) {
      this.shownText = text;
      el.textContent = text;
      el.classList.toggle("on", !!text);
    }
  }

  // ── mpv's names ───────────────────────────────────────────────────

  async set(name: string, value: string | number | boolean): Promise<void> {
    const v = this.video;
    switch (name) {
      case "pause":
        this.wantPaused = !!value;
        if (!v || !this.session) return;
        if (value) v.pause();
        else {
          await this.ctx?.resume().catch(() => {});
          await v.play().catch(() => {});
        }
        return;
      case "start":
        this.startAt = value === "none" ? 0 : Number(value) || 0;
        return;
      case "volume":
        this.volume = Number(value) || 0;
        this.chain?.setMuted(this.muted, this.volume);
        this.prop("volume", this.volume);
        return;
      case "mute":
        this.muted = !!value;
        this.chain?.setMuted(this.muted, this.volume);
        this.prop("mute", this.muted);
        return;
      case "af":
        this.af = String(value);
        this.chain?.setAf(this.af);
        return;
      case "aid": {
        const id = Number(value);
        if (!this.probe || !Number.isFinite(id) || id < 1 || id - 1 === this.audio) return;
        this.audio = id - 1;
        this.publishTracks();
        if (this.session) this.restart(this.now);
        return;
      }
      case "sid":
        this.setSid(String(value));
        return;
      // volume-max and the rest: nothing to do here.
    }
  }

  async cmd(name: string, args: (string | number | boolean)[] = []): Promise<void> {
    switch (name) {
      case "loadfile":
        return this.loadfile(String(args[0]));
      case "seek": {
        const n = Number(args[0]) || 0;
        const mode = String(args[1] ?? "relative");
        this.seek(mode.startsWith("absolute") ? n : this.now + n);
        return;
      }
      case "sub-add":
        return this.subAdd(String(args[0]), String(args[1] ?? "auto"), args[2] ? String(args[2]) : undefined, args[3] ? String(args[3]) : undefined);
      case "stop":
        this.detach();
        return;
    }
  }
}

export const webVideo = new WebVideo();
