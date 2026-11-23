// Which player plays: mpv, or the built-in one (WebView2's own media engine)
// when mpv can't run on this PC. Settings → Playback → Player shows this.

export type EngineChoice = "auto" | "mpv" | "builtin";

const CHOICE_KEY = "velmora.player.engine";
const PROFILE_KEY = "velmora.player.profile";

function read(key: string): string | null {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}

function write(key: string, value: string | null) {
  try {
    if (value === null) localStorage.removeItem(key);
    else localStorage.setItem(key, value);
  } catch {
    // Not remembered.
  }
}

class PlayerEngine {
  /** What the user picked. Auto uses mpv and falls back when it can't start. */
  choice = $state<EngineChoice>(((c) => (c === "mpv" || c === "builtin" ? c : "auto"))(read(CHOICE_KEY)));
  /** The way of starting mpv that last worked on this PC (see mpv.ts). */
  profile = $state<string | null>(read(PROFILE_KEY));
  /** mpv couldn't start at all this session. */
  mpvDown = $state(false);
  /** Why, in a line. */
  reason = $state<string | null>(null);
  /** mpv's own log from the failed start, for "Copy details". */
  log = $state("");

  /** Music and video should use the built-in player. */
  get builtin(): boolean {
    return this.choice === "builtin" || (this.choice === "auto" && this.mpvDown);
  }

  setChoice(c: EngineChoice) {
    this.choice = c;
    write(CHOICE_KEY, c);
    // Asking for mpv again gives it another go.
    if (c !== "builtin") this.mpvDown = false;
  }

  remember(profile: string) {
    this.profile = profile;
    write(PROFILE_KEY, profile);
    this.mpvDown = false;
    this.reason = null;
  }

  /** Forget the way that worked, so the next start tries the best one first. */
  resetProfile() {
    this.profile = null;
    write(PROFILE_KEY, null);
    this.mpvDown = false;
  }

  down(reason: string, log: string) {
    this.mpvDown = true;
    this.reason = reason;
    this.log = log;
  }

  /** Everything worth sending when asking for help. */
  details(): string {
    return [
      `Velmora player report`,
      `Engine: ${this.choice}${this.builtin ? " (using the built-in player)" : ""}`,
      `mpv graphics: ${this.profile ?? "not settled"}`,
      this.reason ? `Reason: ${this.reason}` : "",
      `WebView: ${navigator.userAgent}`,
      `Screen: ${screen.width}x${screen.height} @${devicePixelRatio}`,
      `CPU threads: ${navigator.hardwareConcurrency ?? "?"}`,
      "",
      this.log ? `mpv log:\n${this.log}` : "No mpv log.",
    ]
      .filter((l, i, a) => l !== "" || a[i - 1] !== "")
      .join("\n");
  }
}

export const playerEngine = new PlayerEngine();
