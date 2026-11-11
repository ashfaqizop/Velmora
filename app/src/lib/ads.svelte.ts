// The ad rules for the Public tier. Max installs never see an ad.
//
// The rules (on/off, caps, timings, which slot uses which unit) come from the
// site's /ads.json, so they can change without an app update; DEFAULTS is the
// fallback. Ads themselves are pages on the site shown in a sandboxed iframe
// (components/AdSlot.svelte), so the app never runs ad code.
//
// What keeps them quiet:
// - nothing on the first launch after install, and nothing in the first two
//   minutes of any launch
// - one ad on screen at a time: the sidebar ad steps aside when a page ad shows
// - each slot has a per-session cap, and so does the whole app
// - an ad only loads once it scrolls into view, and refreshes at most every
//   90 seconds while the window has focus
// - nothing while the player (except the pause and end cards) or Ani is open
// - a slot that doesn't fill collapses to nothing
import * as api from "./api";
import { license } from "./license.svelte";
import { nav } from "./state.svelte";

export type SlotId = "home" | "search" | "grid" | "title" | "downloads" | "sidebar" | "sports" | "live" | "pause" | "end";
export type Unit = "native" | "728x90" | "468x60" | "320x50" | "300x250" | "160x300" | "160x600";

interface SlotRule {
  enabled: boolean;
  unit: Unit;
  /** Ads this slot may show per session. */
  max: number;
}

export interface AdRules {
  v: number;
  enabled: boolean;
  /** The first launch that shows ads (2: none on the first one). */
  minLaunch: number;
  /** Quiet time after each launch. */
  graceSeconds: number;
  refreshSeconds: number;
  /** Ads per session across every slot. */
  maxPerSession: number;
  /** How long the player stays paused before the pause card shows. */
  pauseDelaySeconds: number;
  slots: Record<SlotId, SlotRule>;
}

export const SITE = "https://getvelmora.dpdns.org";

const slot = (unit: Unit, max: number): SlotRule => ({ enabled: true, unit, max });

export const DEFAULTS: AdRules = {
  v: 1,
  enabled: true,
  minLaunch: 2,
  graceSeconds: 120,
  refreshSeconds: 90,
  maxPerSession: 60,
  pauseDelaySeconds: 5,
  slots: {
    home: slot("native", 8),
    search: slot("native", 8),
    grid: slot("native", 12),
    title: slot("728x90", 10),
    downloads: slot("728x90", 10),
    sidebar: slot("160x300", 20),
    sports: slot("native", 8),
    live: slot("native", 8),
    pause: slot("300x250", 10),
    end: slot("300x250", 10),
  },
};

const UNITS = new Set<Unit>(["native", "728x90", "468x60", "320x50", "300x250", "160x300", "160x600"]);

/** Pixel size of a banner unit; native ads size themselves. */
export function unitSize(unit: Unit): { width: number; height: number } | null {
  if (unit === "native") return null;
  const [width, height] = unit.split("x").map(Number);
  return { width, height };
}

const num = (v: unknown, fallback: number, min: number, max: number) =>
  typeof v === "number" && Number.isFinite(v) ? Math.min(max, Math.max(min, v)) : fallback;

/** The site's rules over the defaults, with every value checked and clamped. */
function merge(remote: unknown): AdRules {
  if (!remote || typeof remote !== "object") return DEFAULTS;
  const r = remote as Record<string, any>;
  if (r.v !== 1) return DEFAULTS;
  const slots = { ...DEFAULTS.slots };
  for (const id of Object.keys(slots) as SlotId[]) {
    const s = r.slots?.[id];
    if (!s || typeof s !== "object") continue;
    slots[id] = {
      enabled: s.enabled !== false,
      unit: UNITS.has(s.unit) ? s.unit : DEFAULTS.slots[id].unit,
      max: num(s.max, DEFAULTS.slots[id].max, 0, 100),
    };
  }
  return {
    v: 1,
    enabled: r.enabled !== false,
    // Never earlier than the second launch, whatever the file says.
    minLaunch: num(r.minLaunch, DEFAULTS.minLaunch, 2, 50),
    graceSeconds: num(r.graceSeconds, DEFAULTS.graceSeconds, 30, 3600),
    refreshSeconds: num(r.refreshSeconds, DEFAULTS.refreshSeconds, 60, 3600),
    maxPerSession: num(r.maxPerSession, DEFAULTS.maxPerSession, 0, 500),
    pauseDelaySeconds: num(r.pauseDelaySeconds, DEFAULTS.pauseDelaySeconds, 2, 120),
    slots,
  };
}

const REPORT_EVERY = 10 * 60 * 1000;

class Ads {
  rules = $state<AdRules>(DEFAULTS);
  /** Set once the launch's quiet time has passed. */
  private awake = $state(false);
  /** Ads shown this session, per slot. */
  private views = $state<Record<string, number>>({});
  private total = $state(0);
  /** The page ad on screen right now, if any. */
  holder = $state<string | null>(null);
  private unsent: Record<string, number> = {};
  private started = false;

  async init() {
    if (this.started) return;
    this.started = true;
    try {
      this.rules = merge(await api.adsConfig());
    } catch {
      // Defaults.
    }
    setTimeout(() => (this.awake = true), this.rules.graceSeconds * 1000);
    setInterval(() => this.report(), REPORT_EVERY);
    addEventListener("beforeunload", () => this.report());
  }

  /** Ads are on for this install and this moment. */
  get on(): boolean {
    return (
      license.loaded &&
      !license.max &&
      license.status.launches >= this.rules.minLaunch &&
      this.rules.enabled &&
      this.awake &&
      this.total < this.rules.maxPerSession
    );
  }

  /** Pages where only the player's own cards may show. */
  private get quietRoute(): boolean {
    return nav.route.name === "player" || nav.route.name === "ani";
  }

  /** May this slot be on screen at all? Max, a remote switch-off or a quiet
   *  page hides an ad at once; caps only stop new ones (see `allowed`). */
  canShow(id: SlotId): boolean {
    const rule = this.rules.slots[id];
    if (!license.loaded || license.max || !this.rules.enabled || !rule?.enabled) return false;
    if (id === "pause" || id === "end") return nav.route.name === "player";
    if (this.quietRoute) return false;
    // The sidebar ad waits while a page ad is on screen.
    if (id === "sidebar") return this.holder === null;
    return true;
  }

  /** May this slot load a new ad (or refresh) now? */
  allowed(id: SlotId): boolean {
    return this.on && this.canShow(id) && (this.views[id] ?? 0) < this.rules.slots[id].max;
  }

  /** A page ad asks to show. Only one at a time. */
  claim(instance: string): boolean {
    if (this.holder === null) this.holder = instance;
    return this.holder === instance;
  }

  release(instance: string) {
    if (this.holder === instance) this.holder = null;
  }

  src(id: SlotId): string {
    return `${SITE}/ad/${this.rules.slots[id].unit}`;
  }

  unit(id: SlotId): Unit {
    return this.rules.slots[id].unit;
  }

  /** An ad in this slot actually showed. */
  counted(id: SlotId) {
    this.views[id] = (this.views[id] ?? 0) + 1;
    this.total += 1;
    this.unsent[id] = (this.unsent[id] ?? 0) + 1;
  }

  private report() {
    if (!Object.keys(this.unsent).length) return;
    const counts = this.unsent;
    this.unsent = {};
    api.adsReport(counts).catch(() => {});
  }
}

export const ads = new Ads();
