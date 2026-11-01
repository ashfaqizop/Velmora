// Sound options for the player: night mode, dialogue boost, even volume and
// volume boost. They live in localStorage and turn into an mpv `af` chain.

export type SoundOption = "night" | "dialogue" | "normalize" | "boost";

/** Layouts with the center channel third (FL FR FC ...), as mpv names them. */
const CENTER_THIRD = new Set(["3.0", "4.0", "5.0", "5.0(side)", "5.1", "5.1(side)", "6.0", "6.1", "6.1(back)", "7.0", "7.1", "7.1(wide)", "7.1(wide-side)"]);

/** Highest volume with boost on and off. */
export const BOOSTED_MAX = 200;
export const NORMAL_MAX = 100;

const flag = (key: string, fallback: boolean) => {
  const v = localStorage.getItem(`velmora.audio.${key}`);
  return v === null ? fallback : v === "1";
};

class Sound {
  night = $state(flag("night", false));
  dialogue = $state(flag("dialogue", false));
  normalize = $state(flag("normalize", false));
  boost = $state(flag("boost", true));

  get maxVolume() {
    return this.boost ? BOOSTED_MAX : NORMAL_MAX;
  }

  set(option: SoundOption, on: boolean) {
    this[option] = on;
    localStorage.setItem(`velmora.audio.${option}`, on ? "1" : "0");
  }

  /**
   * The mpv `af` value for the current options. `layout` is the playing
   * track's channel layout (mpv's `demux-channels`), which decides how
   * dialogue boost works.
   */
  filters(layout?: string): string {
    const chain: string[] = [];
    if (this.dialogue) chain.push(dialogueFilter(layout));
    // Squash the loud peaks, then lift the result so quiet speech gets louder.
    if (this.night) chain.push("lavfi=[acompressor=threshold=0.05:ratio=6:attack=10:release=250:makeup=3]");
    // Smooth out level changes between scenes and between sources.
    if (this.normalize) chain.push("lavfi=[dynaudnorm=f=250:g=15:p=0.9:m=8]");
    return chain.join(",");
  }
}

/**
 * Surround tracks: raise the center channel, where speech lives, and lower
 * the rest so nothing clips. Stereo has no center, so lift the speech band
 * and trim the low rumble instead.
 */
function dialogueFilter(layout?: string): string {
  const name = layout?.trim();
  const count = name && CENTER_THIRD.has(name) ? channelCount(name) : 0;
  if (count >= 3) {
    const gains = Array.from({ length: count }, (_, i) => `c${i}=${i === 2 ? "1.4" : "0.7"}*c${i}`);
    return `lavfi=[pan=${name}|${gains.join("|")}]`;
  }
  return "lavfi=[equalizer=f=2200:t=q:w=0.9:g=6,equalizer=f=120:t=q:w=0.8:g=-4]";
}

/** Channel count of a layout name like "5.1(side)" or "7.1". */
function channelCount(layout: string): number {
  const [main, lfe] = layout.replace(/\(.*\)/, "").split(".").map(Number);
  return (main || 0) + (lfe || 0);
}

export const sound = new Sound();
