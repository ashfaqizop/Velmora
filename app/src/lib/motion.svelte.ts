// How Velmora moves.
//
// Material 3 Expressive describes motion as springs, not durations: spatial
// springs (position, size, shape) may overshoot a little; effects springs
// (opacity, colour) never do. Each comes fast, default and slow. The values
// below are M3 Expressive's own. Browsers can't run a spring in CSS, so each
// one is sampled into a `linear()` easing with the time it takes to settle,
// and put on :root as --spring-<speed>-<kind> ("520ms linear(...)"), ready
// for `transition` and `animation`. Svelte transitions take the same springs
// through `spring()`.
//
// The Motion setting (Settings → Appearance) picks how much of it plays:
// Full, Reduced (fades only, nothing slides or grows) or Off. "System"
// follows Windows' Animation effects switch.

export type SpringName = "fast-spatial" | "default-spatial" | "slow-spatial" | "fast-effects" | "default-effects" | "slow-effects";

/** M3 Expressive: [damping ratio, stiffness]. */
const SPRINGS: Record<SpringName, [number, number]> = {
  "fast-spatial": [0.6, 800],
  "default-spatial": [0.8, 380],
  "slow-spatial": [0.8, 200],
  "fast-effects": [1, 3800],
  "default-effects": [1, 1600],
  "slow-effects": [1, 800],
};

/** Where a unit spring released at 0 is after `t` seconds. */
function springAt(zeta: number, stiffness: number, t: number): number {
  const w = Math.sqrt(stiffness);
  if (zeta < 1) {
    const wd = w * Math.sqrt(1 - zeta * zeta);
    return 1 - Math.exp(-zeta * w * t) * (Math.cos(wd * t) + ((zeta * w) / wd) * Math.sin(wd * t));
  }
  return 1 - Math.exp(-w * t) * (1 + w * t);
}

export interface Spring {
  /** Seconds until it's within 0.1% and stays there. */
  duration: number;
  /** For Svelte transitions: progress 0..1 to position (may pass 1). */
  ease: (t: number) => number;
  /** For CSS: a `linear()` easing that traces the spring. */
  css: string;
}

function build(zeta: number, stiffness: number): Spring {
  // Settle: the envelope e^(-ζωt) under 0.1%, checked against the real curve.
  const w = Math.sqrt(stiffness);
  let duration = Math.min(2, Math.log(1000) / (Math.min(zeta, 1) * w));
  // Critically damped springs crawl the last bit; trim when it's invisible.
  while (duration > 0.12 && Math.abs(1 - springAt(zeta, stiffness, duration * 0.9)) < 0.002) duration *= 0.9;
  const ease = (p: number) => (p >= 1 ? 1 : springAt(zeta, stiffness, p * duration));
  const steps = Math.max(24, Math.round(duration * 90));
  const points: string[] = [];
  for (let i = 0; i <= steps; i++) points.push((i === steps ? 1 : ease(i / steps)).toFixed(4).replace(/\.?0+$/, "") || "0");
  return { duration, ease, css: `linear(${points.join(", ")})` };
}

const built = Object.fromEntries(Object.entries(SPRINGS).map(([k, [z, s]]) => [k, build(z, s)])) as Record<SpringName, Spring>;

export function springOf(name: SpringName): Spring {
  return built[name];
}

// ── The Motion setting ──────────────────────────────────────────────

export type MotionPref = "system" | "full" | "reduced" | "off";
export type MotionLevel = "full" | "reduced" | "off";

const KEY = "velmora.motion";
const query = typeof matchMedia !== "undefined" ? matchMedia("(prefers-reduced-motion: reduce)") : null;

class Motion {
  pref = $state<MotionPref>(((v) => (v === "full" || v === "reduced" || v === "off" ? v : "system"))(read()));
  private systemReduced = $state(query?.matches ?? false);
  /** A view transition is running; Svelte transitions stand still so the two don't fight. */
  vt = false;

  constructor() {
    query?.addEventListener("change", (e) => (this.systemReduced = e.matches));
  }

  get level(): MotionLevel {
    if (this.pref === "system") return this.systemReduced ? "reduced" : "full";
    return this.pref;
  }

  set(pref: MotionPref) {
    this.pref = pref;
    try {
      localStorage.setItem(KEY, pref);
    } catch {
      // Not remembered.
    }
    apply();
  }
}

function read(): string | null {
  try {
    return localStorage.getItem(KEY);
  } catch {
    return null;
  }
}

export const motion = new Motion();

/** Anything short of full motion: no sliding, growing or bouncing. */
export const reducedMotion = () => motion.level !== "full";

/** Put the springs and the level on the page. Runs at start and on change. */
export function apply() {
  if (typeof document === "undefined") return;
  const root = document.documentElement;
  const level = motion.level;
  root.dataset.motion = level;
  for (const [name, s] of Object.entries(built) as [SpringName, Spring][]) {
    const ms = level === "off" ? 0 : level === "reduced" && name.endsWith("spatial") ? Math.round(built["default-effects"].duration * 1000) : Math.round(s.duration * 1000);
    const curve = level === "off" ? "linear" : level === "reduced" && name.endsWith("spatial") ? built["default-effects"].css : s.css;
    root.style.setProperty(`--spring-${name}`, `${ms}ms ${curve}`);
    root.style.setProperty(`--spring-${name}-duration`, `${ms}ms`);
    root.style.setProperty(`--spring-${name}-easing`, curve);
    // The Material components read their own tokens; give them the same springs.
    root.style.setProperty(`--md-sys-motion-spring-${name}`, `${ms}ms ${curve}`);
  }
  if (level === "off") {
    for (const d of ["short", "medium", "long", "extra-long"]) for (let i = 1; i <= 4; i++) root.style.setProperty(`--md-sys-motion-duration-${d}-${i}`, "0ms");
  } else {
    for (const d of ["short", "medium", "long", "extra-long"]) for (let i = 1; i <= 4; i++) root.style.removeProperty(`--md-sys-motion-duration-${d}-${i}`);
  }
}

$effect.root(() => {
  $effect(() => {
    void motion.level;
    apply();
  });
});

/**
 * Settings for a Svelte transition, scaled to the Motion level: `spatial`
 * moves are dropped under Reduced (opacity only), and everything is instant
 * under Off or while a view transition is running.
 */
export function spring(name: SpringName = "default-spatial", extra: { delay?: number } = {}) {
  const level = motion.level;
  const still = level === "off" || motion.vt;
  const s = level === "reduced" && name.endsWith("spatial") ? built["default-effects"] : built[name];
  return { duration: still ? 0 : Math.round(s.duration * 1000), easing: s.ease, delay: still ? 0 : (extra.delay ?? 0) };
}

/** Distance for a slide, or none when motion is reduced. */
export const travel = (px: number) => (motion.level === "full" ? px : 0);
