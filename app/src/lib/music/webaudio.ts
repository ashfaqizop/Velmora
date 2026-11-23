// The built-in music player, for PCs where mpv can't run.
//
// WebView2 (Chromium) plays the song in an <audio> element, and Web Audio
// does what mpv's filters do: the equalizer (the same RBJ biquads), the
// night and even-volume compressors, the volume and the visualizer's meter.
// It answers to mpv's own property and command names (see io.ts), so the
// music player runs it the same way it runs mpv.
//
// Two elements take turns. While one plays, the next song loads in the
// other; when the first ends, the second starts at once, so albums stay
// gapless. Every URL goes through the loopback server (stream.rs), which
// adds the stream's headers and allows CORS, without which Web Audio would
// only hear silence.
import type { MpvEvent } from "tauri-plugin-libmpv-api";
import * as api from "../api";
import { AudioChain } from "../webchain";
import type { PlayerIO } from "./io";

interface Slot {
  el: HTMLAudioElement;
  /** The song in it has started loading for real (not an empty element). */
  loaded: boolean;
}

class WebAudioIO implements PlayerIO {
  readonly kind = "builtin" as const;

  private ctx: AudioContext | null = null;
  private slots: Slot[] = [];
  /** The slot playing now; the other holds the next song, if `queued`. */
  private cur = 0;
  private queued = false;
  private listeners = new Set<(ev: MpvEvent) => void>();

  private startAt = 0;
  private headers: [string, string][] = [];
  private ua: string | null = null;
  private volume = 100;
  private speed = 1;
  private keepPitch = true;
  private af = "";
  private loadSeq = 0;

  private chain: AudioChain | null = null;

  private emit(ev: MpvEvent) {
    for (const cb of this.listeners) cb(ev);
  }

  private prop(name: string, data: unknown) {
    this.emit({ event: "property-change", name, data } as unknown as MpvEvent);
  }

  private get el(): HTMLAudioElement {
    return this.slots[this.cur].el;
  }

  private get other(): Slot {
    return this.slots[1 - this.cur];
  }

  /** Build the elements and the Web Audio graph the first time. */
  private setup() {
    if (this.ctx) return;
    const ctx = new AudioContext({ latencyHint: "playback" });
    this.ctx = ctx;
    this.chain = new AudioChain(ctx, (levels) => {
      if (!this.el.paused) this.prop("af-metadata/viz", levels);
    });
    this.chain.setVolume(this.volume);
    for (let i = 0; i < 2; i++) {
      const el = new Audio();
      el.crossOrigin = "anonymous";
      el.preload = "auto";
      ctx.createMediaElementSource(el).connect(this.chain.input);
      this.slots.push({ el, loaded: false });
      this.wire(i);
    }
    this.chain.setAf(this.af);
  }

  /** Element events, as mpv would report them. Only the playing slot speaks,
   *  except that a next song which fails to load is dropped. */
  private wire(i: number) {
    const el = this.slots[i].el;
    const mine = () => i === this.cur;
    el.addEventListener("play", () => mine() && this.prop("pause", false));
    el.addEventListener("pause", () => mine() && this.slots[i].loaded && this.prop("pause", true));
    el.addEventListener("timeupdate", () => mine() && this.prop("time-pos", el.currentTime));
    el.addEventListener("durationchange", () => {
      if (mine() && Number.isFinite(el.duration)) this.prop("duration", el.duration);
    });
    el.addEventListener("waiting", () => mine() && this.prop("paused-for-cache", true));
    el.addEventListener("playing", () => mine() && this.prop("paused-for-cache", false));
    el.addEventListener("canplay", () => mine() && this.prop("paused-for-cache", false));
    el.addEventListener("loadedmetadata", () => {
      if (!mine()) return;
      if (this.startAt > 0) el.currentTime = this.startAt;
      this.emit({ event: "file-loaded" } as MpvEvent);
      if (Number.isFinite(el.duration)) this.prop("duration", el.duration);
    });
    el.addEventListener("ended", () => mine() && this.onEnded());
    el.addEventListener("error", () => {
      if (!this.slots[i].loaded) return;
      if (mine()) this.emit({ event: "end-file", reason: "error" } as unknown as MpvEvent);
      else this.clearQueued();
    });
  }

  private onEnded() {
    this.emit({ event: "end-file", reason: "eof" } as unknown as MpvEvent);
    if (!this.queued) return;
    // Gapless: the next song is loaded in the other slot; start it now.
    this.queued = false;
    const old = this.slots[this.cur];
    this.cur = 1 - this.cur;
    old.el.removeAttribute("src");
    old.loaded = false;
    old.el.load();
    const el = this.el;
    this.applyRate(el);
    el.play().catch(() => this.prop("pause", true));
    this.emit({ event: "file-loaded" } as MpvEvent);
    this.prop("time-pos", el.currentTime);
    if (Number.isFinite(el.duration)) this.prop("duration", el.duration);
  }

  private applyRate(el: HTMLAudioElement) {
    el.playbackRate = this.speed;
    el.preservesPitch = this.keepPitch;
  }

  /** Every URL the player hands over becomes a loopback one. */
  private async resolve(url: string): Promise<string> {
    if (/^http:\/\/127\.0\.0\.1:/.test(url)) return url;
    if (/^https?:\/\//i.test(url)) {
      const headers: [string, string][] = [...this.headers];
      if (this.ua) headers.push(["User-Agent", this.ua]);
      return api.webMediaUrl(url, headers);
    }
    return api.webFileUrl(url);
  }

  private async loadfile(url: string, append: boolean) {
    this.setup();
    await this.ctx?.resume().catch(() => {});
    const seq = ++this.loadSeq;
    const src = await this.resolve(url);
    if (append) {
      // A replace that came in meanwhile wins.
      if (seq !== this.loadSeq) return;
      const slot = this.other;
      slot.loaded = true;
      slot.el.src = src;
      this.applyRate(slot.el);
      slot.el.load();
      this.queued = true;
      return;
    }
    if (seq !== this.loadSeq) return;
    this.clearQueued();
    const slot = this.slots[this.cur];
    slot.el.pause();
    slot.loaded = true;
    slot.el.src = src;
    this.applyRate(slot.el);
    slot.el.load();
  }

  private clearQueued() {
    const slot = this.other;
    this.queued = false;
    if (!slot) return;
    slot.loaded = false;
    slot.el.pause();
    slot.el.removeAttribute("src");
    slot.el.load();
  }

  // ── mpv's names ───────────────────────────────────────────────────

  async set(name: string, value: string | number | boolean): Promise<void> {
    switch (name) {
      case "volume":
        this.volume = Number(value) || 0;
        this.chain?.setVolume(this.volume);
        return;
      case "pause":
        if (!this.slots.length) return;
        if (value) this.el.pause();
        else {
          await this.ctx?.resume().catch(() => {});
          await this.el.play().catch(() => this.prop("pause", true));
        }
        return;
      case "start":
        this.startAt = value === "none" ? 0 : Number(value) || 0;
        return;
      case "speed":
        this.speed = Math.min(4, Math.max(0.25, Number(value) || 1));
        for (const s of this.slots) this.applyRate(s.el);
        return;
      case "audio-pitch-correction":
        this.keepPitch = value === true || value === "yes";
        for (const s of this.slots) this.applyRate(s.el);
        return;
      case "af":
        this.af = String(value);
        this.chain?.setAf(this.af);
        return;
      case "user-agent":
        this.ua = String(value) || null;
        return;
      case "http-header-fields":
        this.headers = String(value)
          .split(",")
          .map((h): [string, string] => {
            const i = h.indexOf(":");
            return i > 0 ? [h.slice(0, i).trim(), h.slice(i + 1).trim()] : ["", ""];
          })
          .filter((h) => h[0] !== "");
        return;
      // vid, force-window, audio-files, audio-device, audio-exclusive,
      // volume-max: nothing to do without mpv.
    }
  }

  async get<T = unknown>(name: string): Promise<T> {
    switch (name) {
      case "pause":
        return (this.slots.length ? this.el.paused : true) as T;
      case "time-pos":
        return (this.slots.length ? this.el.currentTime : 0) as T;
      case "duration":
        return (this.slots.length && Number.isFinite(this.el.duration) ? this.el.duration : 0) as T;
      case "audio-device-list":
        return [{ name: "auto", description: "Windows default (built-in player)" }] as T;
    }
    return null as T;
  }

  async cmd(name: string, args: (string | number)[] = []): Promise<void> {
    switch (name) {
      case "loadfile":
        return this.loadfile(String(args[0]), args[1] === "append");
      case "seek": {
        if (!this.slots.length) return;
        const t = Number(args[0]) || 0;
        const mode = String(args[1] ?? "relative");
        const el = this.el;
        el.currentTime = Math.max(0, mode.startsWith("absolute") ? t : el.currentTime + t);
        return;
      }
      case "playlist-clear":
        this.clearQueued();
        return;
      case "stop":
        this.loadSeq++;
        for (const s of this.slots) {
          s.loaded = false;
          s.el.pause();
          s.el.removeAttribute("src");
          s.el.load();
        }
        this.queued = false;
        return;
    }
  }

  async listen(cb: (ev: MpvEvent) => void): Promise<() => void> {
    this.listeners.add(cb);
    return () => this.listeners.delete(cb);
  }
}

export const webAudioIO = new WebAudioIO();
