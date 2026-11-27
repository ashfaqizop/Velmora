// The app's ad rules (what /ads.json serves). The admin site saves edits in
// Redis under cfg:ads; until it does, or if Redis can't answer, these defaults
// apply. They match DEFAULTS in app/src/lib/ads.svelte.ts.

export const SLOTS = ["home","search","grid","title","downloads","sidebar","sports","live","pause","end","music","nowplaying","lobby"];
export const UNITS = ["native", "728x90", "468x60", "320x50", "300x250", "160x300", "160x600"];

export const DEFAULT_RULES = {
  "v": 1,
  "enabled": true,
  "minLaunch": 2,
  "graceSeconds": 120,
  "refreshSeconds": 90,
  "maxPerSession": 60,
  "pauseDelaySeconds": 5,
  "slots": {
    "home": { "enabled": true, "unit": "native", "max": 8 },
    "search": { "enabled": true, "unit": "native", "max": 8 },
    "grid": { "enabled": true, "unit": "native", "max": 12 },
    "title": { "enabled": true, "unit": "728x90", "max": 10 },
    "downloads": { "enabled": true, "unit": "728x90", "max": 10 },
    "sidebar": { "enabled": true, "unit": "160x300", "max": 20 },
    "sports": { "enabled": true, "unit": "native", "max": 8 },
    "live": { "enabled": true, "unit": "native", "max": 8 },
    "pause": { "enabled": true, "unit": "300x250", "max": 10 },
    "end": { "enabled": true, "unit": "300x250", "max": 10 },
    "music": { "enabled": true, "unit": "native", "max": 10 },
    "nowplaying": { "enabled": true, "unit": "300x250", "max": 8 },
    "lobby": { "enabled": true, "unit": "300x250", "max": 10 } }
};

// Bounds for each number, so a typo can't flood anyone with ads.
export const BOUNDS = {
  minLaunch: [1, 20],
  graceSeconds: [0, 3600],
  refreshSeconds: [60, 3600],
  maxPerSession: [0, 500],
  pauseDelaySeconds: [1, 120],
  slotMax: [0, 100],
};

/** A clean copy of the rules, or a message saying what is wrong. */
export function cleanRules(input) {
  if (!input || typeof input !== "object") return { error: "Rules must be an object." };
  const out = { v: 1, enabled: input.enabled !== false };
  for (const k of ["minLaunch", "graceSeconds", "refreshSeconds", "maxPerSession", "pauseDelaySeconds"]) {
    const [lo, hi] = BOUNDS[k];
    const n = input[k];
    if (!Number.isInteger(n) || n < lo || n > hi) return { error: `${k} must be a whole number from ${lo} to ${hi}.` };
    out[k] = n;
  }
  out.slots = {};
  for (const id of SLOTS) {
    const s = input.slots?.[id] ?? DEFAULT_RULES.slots[id];
    const [lo, hi] = BOUNDS.slotMax;
    if (!UNITS.includes(s.unit)) return { error: `The ${id} slot has an unknown unit.` };
    if (!Number.isInteger(s.max) || s.max < lo || s.max > hi) return { error: `The ${id} cap must be from ${lo} to ${hi}.` };
    out.slots[id] = { enabled: s.enabled !== false, unit: s.unit, max: s.max };
  }
  return { rules: out };
}
