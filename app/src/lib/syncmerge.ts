// The plain, testable parts of sync (see sync.svelte.ts): which settings travel
// between PCs, checking what comes back from the server, and the bookkeeping
// of what still has to be pushed. No Svelte, no network.

/** What the last edit to one list entry or one title's progress looked like on this PC. */
export interface Meta {
  /** When it was edited (ms). The newest edit wins when two PCs disagree. */
  at: number;
  /** It was removed. The mark stays so the removal reaches other PCs. */
  del: boolean;
  /** Not on the server yet. */
  dirty: boolean;
}
export type MetaMap = Record<string, Meta>;

/** Preferences that follow you. The download folder and the download count belong to the PC. */
export const PREF_KEYS = [
  "preferred_resolution",
  "max_auto_resolution",
  "preferred_codec",
  "audio_languages",
  "subtitle_languages",
  "show_adult",
  "include_torrents",
  "include_french",
  "provider_order",
  "disabled_providers",
  "verify_sources",
  "seed_after_download",
  "region",
] as const;

export interface SyncedSettings {
  v: 1;
  prefs: Record<string, unknown>;
  accent?: string;
  audioMode?: "sub" | "dub";
  motion?: "system" | "full" | "reduced" | "off";
  sourcesAhead?: boolean;
}

/** The synced preferences out of a full preferences object, in a fixed order. */
export function pickPrefs(prefs: Record<string, unknown> | null | undefined): Record<string, unknown> {
  const out: Record<string, unknown> = {};
  for (const k of PREF_KEYS) if (prefs && k in prefs) out[k] = prefs[k];
  return out;
}

const isStrings = (v: unknown, max: number, each = /^[\w-]{1,40}$/): v is string[] =>
  Array.isArray(v) && v.length <= max && v.every((s) => typeof s === "string" && each.test(s));

/**
 * Settings that came from the server, checked. Anything unexpected is dropped,
 * so a bad or old value can't break the app; null if it isn't settings at all.
 */
export function cleanSettings(raw: unknown): SyncedSettings | null {
  if (!raw || typeof raw !== "object") return null;
  const r = raw as Record<string, unknown>;
  const p = (r.prefs && typeof r.prefs === "object" ? r.prefs : {}) as Record<string, unknown>;
  const prefs: Record<string, unknown> = {};

  const num = (k: string, lo: number, hi: number) => {
    const v = p[k];
    if (typeof v === "number" && Number.isFinite(v) && v >= lo && v <= hi) prefs[k] = Math.round(v);
  };
  const flag = (k: string) => {
    if (typeof p[k] === "boolean") prefs[k] = p[k];
  };
  num("preferred_resolution", 0, 4320);
  num("max_auto_resolution", 0, 4320);
  if (typeof p.preferred_codec === "string" && /^[a-z0-9_]{2,12}$/.test(p.preferred_codec)) prefs.preferred_codec = p.preferred_codec;
  if (isStrings(p.audio_languages, 10, /^[a-z]{2,3}$/)) prefs.audio_languages = p.audio_languages;
  if (isStrings(p.subtitle_languages, 10, /^[a-z]{2,3}$/)) prefs.subtitle_languages = p.subtitle_languages;
  for (const k of ["show_adult", "include_torrents", "include_french", "verify_sources", "seed_after_download"]) flag(k);
  if (isStrings(p.disabled_providers, 40)) prefs.disabled_providers = p.disabled_providers;
  if (p.provider_order && typeof p.provider_order === "object" && !Array.isArray(p.provider_order)) {
    const order: Record<string, string[]> = {};
    for (const [kind, list] of Object.entries(p.provider_order as Record<string, unknown>)) {
      if (/^[a-z_]{2,20}$/.test(kind) && isStrings(list, 40)) order[kind] = list;
    }
    prefs.provider_order = order;
  }
  if (p.region === null || (typeof p.region === "string" && /^[A-Z]{2}$/.test(p.region))) prefs.region = p.region;

  const out: SyncedSettings = { v: 1, prefs };
  if (typeof r.accent === "string" && /^#[0-9a-fA-F]{6}$/.test(r.accent)) out.accent = r.accent;
  if (r.audioMode === "sub" || r.audioMode === "dub") out.audioMode = r.audioMode;
  if (r.motion === "system" || r.motion === "full" || r.motion === "reduced" || r.motion === "off") out.motion = r.motion;
  if (typeof r.sourcesAhead === "boolean") out.sourcesAhead = r.sourcesAhead;
  return out;
}

/** The entries still to push, oldest edit first. */
export function dirtyEntries(meta: MetaMap): [string, Meta][] {
  return Object.entries(meta)
    .filter(([, m]) => m.dirty)
    .sort((a, b) => a[1].at - b[1].at);
}

/** After a push: what was sent is clean, unless it was edited again meanwhile. */
export function settle(meta: MetaMap, sent: [string, Meta][]) {
  for (const [key, was] of sent) {
    const now = meta[key];
    if (now && now.at === was.at && now.del === was.del) now.dirty = false;
  }
}

export function chunk<T>(items: T[], size: number): T[][] {
  const out: T[][] = [];
  for (let i = 0; i < items.length; i += size) out.push(items.slice(i, i + size));
  return out;
}

/** Removed entries older than this are forgotten locally; every PC has long since seen them. */
export const TOMBSTONE_DAYS = 90;

export function pruneTombstones(meta: MetaMap, now = Date.now()) {
  for (const [key, m] of Object.entries(meta)) {
    if (m.del && !m.dirty && now - m.at > TOMBSTONE_DAYS * 86_400_000) delete meta[key];
  }
}
