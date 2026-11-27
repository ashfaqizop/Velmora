// The Theatre algorithm's settings. The admin site saves them in Redis under
// cfg:lineup; these defaults apply until it does (or if Redis can't answer).

import { redis } from "./license.js";

export const DEFAULT_KNOBS = {
  /** Screenings per day: min + one more for every `perInstalls` daily installs, up to max. */
  minPerDay: 4,
  maxPerDay: 12,
  perInstalls: 30,
  /** People connected at once that Theatre may use (Supabase Realtime free: ~200). */
  budget: 160,
  /** Screenings allowed to run at the same time. */
  maxOverlap: 3,
  /** Days before a title can air again; days before a hidden gem can be the pick again. */
  repeatDays: 14,
  pickRepeatDays: 60,
  /** Tonight's pick: one highly rated film few people here have seen. */
  pick: true,
  gemMinRating: 7.4,
  gemMinVotes: 800,
  /** How much Velmora's own plays count against TMDB's trending rank. */
  playsWeight: 3,
  trendWeight: 1,
  minRuntime: 40,
  /** Genre mix: never these, and at most this many sharing a main genre (0 = no cap). */
  excludeGenres: [],
  maxPerGenre: 3,
  /** Source checks: a draft is dropped once it has this many checks and finds a source less often than this. */
  probeMinChecks: 3,
  probeMinFound: 0.5,
};

// [min, max, whole number?]
const RANGES = {
  minPerDay: [1, 24, true],
  maxPerDay: [1, 24, true],
  perInstalls: [1, 10000, true],
  budget: [10, 1000, true],
  maxOverlap: [1, 10, true],
  repeatDays: [0, 365, true],
  pickRepeatDays: [0, 3650, true],
  gemMinRating: [5, 9.5, false],
  gemMinVotes: [0, 100000, true],
  playsWeight: [0, 10, false],
  trendWeight: [0, 10, false],
  minRuntime: [20, 200, true],
  maxPerGenre: [0, 24, true],
  probeMinChecks: [1, 50, true],
  probeMinFound: [0, 1, false],
};

export const GENRES = [
  "Action", "Adventure", "Animation", "Comedy", "Crime", "Documentary", "Drama", "Family", "Fantasy", "History",
  "Horror", "Music", "Mystery", "Romance", "Science Fiction", "TV Movie", "Thriller", "War", "Western",
];

/** A clean copy, or a message saying what is wrong. */
export function cleanKnobs(input) {
  if (!input || typeof input !== "object") return { error: "Settings must be an object." };
  const out = { pick: input.pick !== false };
  for (const [k, [lo, hi, whole]] of Object.entries(RANGES)) {
    const n = input[k];
    if (typeof n !== "number" || !Number.isFinite(n) || n < lo || n > hi || (whole && !Number.isInteger(n))) {
      return { error: `${k} must be ${whole ? "a whole number" : "a number"} from ${lo} to ${hi}.` };
    }
    out[k] = n;
  }
  if (out.minPerDay > out.maxPerDay) return { error: "The minimum per day can't be above the maximum." };
  const ex = Array.isArray(input.excludeGenres) ? input.excludeGenres : [];
  if (ex.some((g) => !GENRES.includes(g))) return { error: "Unknown genre in the excluded list." };
  out.excludeGenres = [...new Set(ex)];
  return { knobs: out };
}

export async function loadKnobs() {
  try {
    const saved = await redis("GET", "cfg:lineup");
    if (saved) return { ...DEFAULT_KNOBS, ...JSON.parse(saved) };
  } catch {
    // Storage hiccup: the defaults are a safe answer.
  }
  return { ...DEFAULT_KNOBS };
}
