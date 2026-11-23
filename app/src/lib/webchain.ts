// mpv's audio filters, rebuilt in Web Audio for the built-in player.
//
// The players hand mpv an `af` string (the equalizer, night mode, dialogue
// boost, even volume, and the visualizer's meter). The built-in player gets
// the same string and builds the same thing from Web Audio nodes: RBJ
// biquads for the equalizer, compressors for night mode and even volume,
// and analysers for the meter.

/** One filter from an mpv `lavfi=[...]` chain. */
interface Filter {
  name: string;
  args: Record<string, string>;
  raw: string;
}

/** Every filter in an mpv `af` string, in order. */
export function parseAf(af: string): Filter[] {
  const out: Filter[] = [];
  for (const m of af.matchAll(/lavfi=\[([^\]]*)\]/g)) {
    for (const part of m[1].split(",")) {
      const eq = part.indexOf("=");
      const name = (eq < 0 ? part : part.slice(0, eq)).trim();
      const rest = eq < 0 ? "" : part.slice(eq + 1);
      const args: Record<string, string> = {};
      for (const kv of rest.split(":")) {
        const i = kv.indexOf("=");
        if (i > 0) args[kv.slice(0, i)] = kv.slice(i + 1);
      }
      if (name) out.push({ name, args, raw: rest });
    }
  }
  return out;
}

const num = (v: string | undefined, fallback: number) => {
  const n = parseFloat(v ?? "");
  return Number.isFinite(n) ? n : fallback;
};

/** mpv's volume is cubic: 50% is an eighth of the amplitude. */
export const amplitude = (volume: number) => Math.max(0, volume / 100) ** 3;

const dB = (rms: number) => (rms > 0 ? 20 * Math.log10(rms) : -Infinity);

/** Levels the way mpv's astats filter reports them, in dB as text. */
export type MeterLevels = Record<string, string>;

/** input → filters from `af` → volume → speakers, with an optional meter. */
export class AudioChain {
  readonly input: GainNode;
  private master: GainNode;
  private nodes: AudioNode[] = [];
  private meters: [AnalyserNode, AnalyserNode] | null = null;
  private splitter: ChannelSplitterNode | null = null;
  private meterTimer: ReturnType<typeof setInterval> | undefined;

  constructor(
    private ctx: AudioContext,
    private onMeter?: (levels: MeterLevels) => void,
  ) {
    this.input = ctx.createGain();
    this.master = ctx.createGain();
    this.master.connect(ctx.destination);
    this.input.connect(this.master);
  }

  /** mpv's volume scale (100 = as is, up to 200 with boost). */
  setVolume(volume: number) {
    this.master.gain.setTargetAtTime(amplitude(volume), this.ctx.currentTime, 0.03);
  }

  setMuted(muted: boolean, volume: number) {
    this.setVolume(muted ? 0 : volume);
  }

  /** Rebuild the filters from an mpv `af` string. */
  setAf(af: string) {
    const ctx = this.ctx;
    this.input.disconnect();
    for (const n of this.nodes) n.disconnect();
    this.nodes = [];
    let viz = false;
    for (const f of parseAf(af)) {
      switch (f.name) {
        case "volume": {
          const g = ctx.createGain();
          g.gain.value = 10 ** (num(f.raw.replace(/db$/i, ""), 0) / 20);
          this.nodes.push(g);
          break;
        }
        case "equalizer":
        case "bass":
        case "treble": {
          const b = ctx.createBiquadFilter();
          b.type = f.name === "bass" ? "lowshelf" : f.name === "treble" ? "highshelf" : "peaking";
          b.frequency.value = num(f.args.f, 1000);
          b.Q.value = num(f.args.w, 1);
          b.gain.value = num(f.args.g, 0);
          this.nodes.push(b);
          break;
        }
        case "acompressor": {
          // Night mode. mpv's threshold and makeup are linear.
          const c = ctx.createDynamicsCompressor();
          c.threshold.value = Math.max(-100, dB(num(f.args.threshold, 0.125)));
          c.ratio.value = Math.min(20, num(f.args.ratio, 2));
          c.attack.value = num(f.args.attack, 20) / 1000;
          c.release.value = num(f.args.release, 250) / 1000;
          c.knee.value = 6;
          const makeup = ctx.createGain();
          makeup.gain.value = num(f.args.makeup, 1);
          this.nodes.push(c, makeup);
          break;
        }
        case "dynaudnorm": {
          // Even volume: a slow, gentle compressor with some lift.
          const c = ctx.createDynamicsCompressor();
          c.threshold.value = -28;
          c.knee.value = 12;
          c.ratio.value = 3;
          c.attack.value = 0.05;
          c.release.value = 0.6;
          const lift = ctx.createGain();
          lift.gain.value = 1.8;
          this.nodes.push(c, lift);
          break;
        }
        case "pan": {
          // Dialogue boost on surround: lift the centre channel. Web Audio
          // downmixes to stereo anyway, so a gentle speech-band lift does it.
          const b = ctx.createBiquadFilter();
          b.type = "peaking";
          b.frequency.value = 2200;
          b.Q.value = 0.9;
          b.gain.value = 5;
          this.nodes.push(b);
          break;
        }
        case "astats":
          viz = true;
          break;
      }
    }
    let node: AudioNode = this.input;
    for (const n of this.nodes) {
      node.connect(n);
      node = n;
    }
    node.connect(this.master);
    this.setMeter(viz && !!this.onMeter);
  }

  private setMeter(on: boolean) {
    if (!on) {
      clearInterval(this.meterTimer);
      this.meterTimer = undefined;
      if (this.splitter) this.master.disconnect(this.splitter);
      this.splitter = null;
      this.meters = null;
      return;
    }
    if (this.meters) return;
    const splitter = this.ctx.createChannelSplitter(2);
    const meters: [AnalyserNode, AnalyserNode] = [this.ctx.createAnalyser(), this.ctx.createAnalyser()];
    this.splitter = splitter;
    this.meters = meters;
    this.master.connect(splitter);
    meters.forEach((a, i) => {
      a.fftSize = 1024;
      splitter.connect(a, i);
    });
    const buf = new Float32Array(1024);
    const rms = (a: AnalyserNode) => {
      a.getFloatTimeDomainData(buf);
      let sum = 0;
      for (const x of buf) sum += x * x;
      return Math.sqrt(sum / buf.length);
    };
    this.meterTimer = setInterval(() => {
      if (!this.meters) return;
      const l = rms(this.meters[0]);
      const r = rms(this.meters[1]);
      this.onMeter?.({
        "lavfi.astats.1.RMS_level": String(dB(l)),
        "lavfi.astats.2.RMS_level": String(dB(r)),
        "lavfi.astats.Overall.RMS_level": String(dB((l + r) / 2)),
      });
    }, 33);
  }
}
