// App-wide state: navigation, the home feed, and the local library
// (My List and watch history live in localStorage until sync lands).
import * as api from "./api";
import type { AudioMode, HistoryEntry, MediaKind, Rail, Target, TitleDetail, TitleSummary } from "./types";

export type Browse = "movie" | "series" | "anime";

export type Route =
  | { name: "home" }
  | { name: "browse"; kind: Browse }
  | { name: "collection"; id: string }
  | { name: "mylist" }
  | { name: "downloads" }
  | { name: "search"; query: string }
  | { name: "title"; title: TitleSummary }
  | { name: "player"; title: TitleSummary; target: Target; mode: AudioMode; detail?: TitleDetail };

class Nav {
  route = $state<Route>({ name: "home" });
  private stack: Route[] = [];

  go(route: Route) {
    if (JSON.stringify(route) === JSON.stringify(this.route)) return;
    // Typing in search refines the current search instead of stacking pages.
    if (!(route.name === "search" && this.route.name === "search")) this.stack.push(this.route);
    this.route = route;
  }

  /** Swap the current page without adding a history step (next episode). */
  replace(route: Route) {
    this.route = route;
  }

  back() {
    this.route = this.stack.pop() ?? { name: "home" };
  }

  get canGoBack() {
    return this.route.name !== "home";
  }
}

export const nav = new Nav();

/** Resolved once the splash has refreshed the remote config. */
let releaseConfig!: () => void;
export const configReady = new Promise<void>((r) => (releaseConfig = r));
export const markConfigReady = () => releaseConfig();

class Feed {
  rails = $state<Rail[]>([]);
  loading = $state(true);
  error = $state<string | null>(null);

  async load() {
    this.loading = true;
    this.error = null;
    try {
      await configReady;
      this.rails = await api.home();
    } catch (e) {
      this.error = String(e);
    } finally {
      this.loading = false;
    }
  }

  rail(id: string) {
    return this.rails.find((r) => r.id === id);
  }

  /** Titles with artwork wide enough for the hero, newest trends first. */
  get spotlight(): TitleSummary[] {
    const picks: TitleSummary[] = [];
    for (const id of ["trending_movies", "trending_series", "trending_anime"]) {
      const top = this.rail(id)?.items.filter((i) => i.backdrop || i.poster).slice(0, 2) ?? [];
      picks.push(...top);
    }
    return dedupe(picks).slice(0, 5);
  }

  ofKind(kind: Browse): TitleSummary[] {
    const match = (k: MediaKind) => (kind === "series" ? k === "series" || k === "asian_drama" : k === kind);
    return dedupe(this.rails.flatMap((r) => r.items).filter((i) => match(i.kind)));
  }
}

export const feed = new Feed();

export const keyOf = (t: TitleSummary) => `${t.origin.provider}:${t.origin.id}`;

function dedupe(items: TitleSummary[]) {
  const seen = new Set<string>();
  return items.filter((i) => {
    const k = keyOf(i);
    if (seen.has(k)) return false;
    seen.add(k);
    return true;
  });
}

function load<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(`velmora.${key}`);
    return raw ? (JSON.parse(raw) as T) : fallback;
  } catch {
    return fallback;
  }
}

function save(key: string, value: unknown) {
  localStorage.setItem(`velmora.${key}`, JSON.stringify(value));
}

/** Past this share of the runtime a title counts as watched. */
const FINISHED = 0.95;

class Library {
  watchlist = $state<TitleSummary[]>(load("watchlist", []));
  history = $state<HistoryEntry[]>(load("history", []));

  has(t: TitleSummary) {
    const k = keyOf(t);
    return this.watchlist.some((w) => keyOf(w) === k);
  }

  toggle(t: TitleSummary) {
    const k = keyOf(t);
    this.watchlist = this.has(t) ? this.watchlist.filter((w) => keyOf(w) !== k) : [$state.snapshot(t), ...this.watchlist];
    save("watchlist", this.watchlist);
  }

  /** Unfinished titles, most recent first (one entry per title). */
  get continueWatching(): HistoryEntry[] {
    return this.history
      .filter((h) => h.duration > 0 && h.position / h.duration < FINISHED)
      .sort((a, b) => b.updatedAt - a.updatedAt);
  }

  /** The last thing watched for a title (any episode). */
  last(t: TitleSummary): HistoryEntry | undefined {
    const k = keyOf(t);
    return this.history.find((h) => keyOf(h.title) === k);
  }

  /** Where to resume `target`, in seconds; 0 when it should start over. */
  resumeAt(t: TitleSummary, target: Target): number {
    const h = this.last(t);
    if (!h || h.duration <= 0) return 0;
    const same = target.type === "episode" ? h.season === target.season && h.episode === target.episode : h.episode === undefined;
    if (!same || h.position < 30 || h.position / h.duration >= FINISHED) return 0;
    return Math.floor(h.position);
  }

  /** Record playback progress; keeps one entry per title, newest first. */
  saveProgress(t: TitleSummary, target: Target, position: number, duration: number) {
    if (!(duration > 0) || !(position >= 0)) return;
    const k = keyOf(t);
    const entry: HistoryEntry = {
      title: $state.snapshot(t),
      season: target.type === "episode" ? target.season : undefined,
      episode: target.type === "episode" ? target.episode : undefined,
      position,
      duration,
      updatedAt: Date.now(),
    };
    this.history = [entry, ...this.history.filter((h) => keyOf(h.title) !== k)].slice(0, 200);
    save("history", this.history);
  }

  forget(t: TitleSummary) {
    const k = keyOf(t);
    this.history = this.history.filter((h) => keyOf(h.title) !== k);
    save("history", this.history);
  }
}

export const library = new Library();

export const kindLabel: Record<MediaKind, string> = {
  movie: "Movie",
  series: "Series",
  anime: "Anime",
  asian_drama: "Drama",
};

export function meta(t: TitleSummary) {
  return [t.year, kindLabel[t.kind]].filter(Boolean).join(" • ");
}

export function formatLeft(seconds: number) {
  const m = Math.max(0, Math.round(seconds / 60));
  return m >= 60 ? `${Math.floor(m / 60)}h ${m % 60}m left` : `${m}m left`;
}

/** The episode after `target`, using the title's season list. */
export function nextEpisode(detail: TitleDetail | undefined, target: Target): Target | null {
  if (!detail || target.type !== "episode") return null;
  const seasons = [...detail.seasons].sort((a, b) => a.number - b.number);
  const si = seasons.findIndex((s) => s.number === target.season);
  if (si < 0) return null;
  const eps = [...seasons[si].episodes].sort((a, b) => a.number - b.number);
  const ei = eps.findIndex((e) => e.number === target.episode);
  if (ei >= 0 && ei + 1 < eps.length) return { type: "episode", season: target.season, episode: eps[ei + 1].number };
  const following = seasons[si + 1];
  const first = following?.episodes.slice().sort((a, b) => a.number - b.number)[0];
  return first ? { type: "episode", season: following.number, episode: first.number } : null;
}

export function targetLabel(target: Target) {
  return target.type === "episode" ? `S${target.season} E${target.episode}` : "";
}

export function formatTime(seconds: number | null | undefined) {
  const s = Math.max(0, Math.floor(seconds ?? 0));
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = String(s % 60).padStart(2, "0");
  return h ? `${h}:${String(m).padStart(2, "0")}:${sec}` : `${m}:${sec}`;
}
