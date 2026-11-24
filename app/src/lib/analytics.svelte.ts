// Anonymous usage statistics: which titles and sites get played, which player
// runs. Counted on this PC, sent once a day as plain totals. No ID, no account,
// nothing that says who. On by default; Settings > Privacy switches it off and
// forgets what was counted. The server side is site/api/_lib/stats.js.
import { M3eSnackbar } from "@m3e/web/snackbar";
import * as api from "./api";
import type { Source, TitleSummary } from "./types";

const ON = "velmora.analytics.on";
const PENDING = "velmora.analytics.pending";
const NOTICE = "velmora.analytics.notice";

type Counts = Record<string, number>;
interface Day {
  titles: Counts;
  sources: Counts;
  engines: Counts;
  quality: Counts;
  /** Songs listened to (YouTube video ids), counted once a day. */
  music?: Counts;
  /** Plays started per UTC hour ("00".."23"), so Theatre can put screenings at busy hours. */
  hours?: Counts;
  app: string;
}
type Pending = Record<string, Day>;

const utcDay = (t = Date.now()) => new Date(t).toISOString().slice(0, 10);

/** TMDB's own name for a title ("movie/603", "tv/1396"), or null when we can't tell. */
export function titleKey(t: TitleSummary): string | null {
  if (t.origin.provider === "tmdb" && /^(movie|tv)\/\d{1,9}$/.test(t.origin.id)) return t.origin.id;
  const id = t.ids?.tmdb;
  return id ? `${t.kind === "movie" ? "movie" : "tv"}/${id}` : null;
}

export function qualityBucket(resolution?: number): string {
  if (!resolution) return "other";
  if (resolution >= 2160) return "2160";
  if (resolution >= 1440) return "1440";
  if (resolution >= 1080) return "1080";
  if (resolution >= 720) return "720";
  return resolution >= 480 ? "480" : "other";
}

function readPending(): Pending {
  try {
    const v = JSON.parse(localStorage.getItem(PENDING) ?? "{}");
    return v && typeof v === "object" ? v : {};
  } catch {
    return {};
  }
}

function writePending(p: Pending) {
  try {
    localStorage.setItem(PENDING, JSON.stringify(p));
  } catch {
    // Not kept; one day's counts at worst.
  }
}

const bump = (c: Counts, k: string, by = 1) => (c[k] = Math.min((c[k] ?? 0) + by, 200));
const list = (c: Counts) => Object.entries(c).map(([k, n]) => ({ k, n }));

function batch(day: string, d: Day) {
  return {
    v: 1,
    day,
    app: d.app,
    titles: Object.keys(d.titles).map((k) => ({ k })),
    sources: list(d.sources),
    engines: list(d.engines),
    quality: list(d.quality),
    music: Object.keys(d.music ?? {}).map((k) => ({ k })),
    hours: list(d.hours ?? {}),
  };
}

class Analytics {
  enabled = $state(localStorage.getItem(ON) !== "0");
  private version = "";

  /** Call once at startup. */
  async start() {
    try {
      this.version = (await api.appInfo()).version;
    } catch {
      // Without a version there is nothing to send yet.
    }
    setTimeout(() => this.flush(), 20_000);
    setInterval(() => this.flush(), 6 * 3_600_000);
    this.noticeLater(0);
  }

  setEnabled(on: boolean) {
    this.enabled = on;
    try {
      localStorage.setItem(ON, on ? "1" : "0");
    } catch {
      // Not remembered.
    }
    // Off means off: what was counted but not sent goes too.
    if (!on) writePending({});
  }

  /** Something was watched for a minute: count the title, the site, the player and the quality. */
  watched(title: TitleSummary, source: Source, engine: "mpv" | "builtin") {
    if (!this.enabled || !this.version) return;
    const key = titleKey(title);
    const pending = readPending();
    const day = (pending[utcDay()] ??= { titles: {}, sources: {}, engines: {}, quality: {}, app: this.version });
    day.app = this.version;
    if (key) bump(day.titles, key);
    bump(day.sources, String(source.provider));
    bump(day.engines, engine);
    bump(day.quality, qualityBucket(source.tags?.resolution));
    bump((day.hours ??= {}), String(new Date().getUTCHours()).padStart(2, "0"));
    writePending(pending);
  }

  /** A song was heard properly (the same rule as the play history). */
  listened(videoId: string) {
    if (!this.enabled || !this.version || !/^[A-Za-z0-9_-]{11}$/.test(videoId)) return;
    const pending = readPending();
    const day = (pending[utcDay()] ??= { titles: {}, sources: {}, engines: {}, quality: {}, music: {}, app: this.version });
    bump((day.music ??= {}), videoId);
    writePending(pending);
  }

  /** Send finished days. Today's counts wait until the day is over. */
  async flush() {
    if (!this.enabled || !navigator.onLine) return;
    const pending = readPending();
    const today = utcDay();
    const oldest = utcDay(Date.now() - 8 * 86_400_000);
    let changed = false;
    for (const [day, d] of Object.entries(pending)) {
      if (day >= today) continue;
      if (day < oldest) {
        delete pending[day];
        changed = true;
        continue;
      }
      try {
        if (await api.analyticsReport(batch(day, d))) {
          delete pending[day];
          changed = true;
        }
      } catch {
        // Try again next time.
      }
    }
    if (changed) writePending(pending);
  }

  /** Exactly what would be sent, as text, for Settings > Privacy. */
  preview(): string {
    const pending = readPending();
    const days = Object.keys(pending).sort();
    if (!days.length) return "Nothing is waiting to be sent.";
    return JSON.stringify(
      days.map((day) => batch(day, pending[day])),
      null,
      2,
    );
  }

  /** Once, after the first-run setup: say it is on and where to turn it off. */
  private noticeLater(tries: number) {
    try {
      if (localStorage.getItem(NOTICE)) return;
      if (!localStorage.getItem("velmora.onboarded")) {
        if (tries < 20) setTimeout(() => this.noticeLater(tries + 1), 30_000);
        return;
      }
      localStorage.setItem(NOTICE, "1");
    } catch {
      return;
    }
    if (!this.enabled) return;
    setTimeout(
      () => M3eSnackbar.open("Velmora shares anonymous usage counts (which titles and sites get played, never who). You can turn that off in Settings > Privacy.", true),
      8000,
    );
  }
}

export const analytics = new Analytics();
