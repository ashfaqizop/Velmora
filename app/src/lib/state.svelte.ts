// App-wide state: navigation, the home feed, and the local library
// (My List and watch history live in localStorage until sync lands).
import * as api from "./api";
import type { HistoryEntry, MediaKind, Rail, TitleSummary } from "./types";

export type Browse = "movie" | "series" | "anime";

export type Route =
  | { name: "home" }
  | { name: "browse"; kind: Browse }
  | { name: "collection"; id: string }
  | { name: "mylist" }
  | { name: "downloads" }
  | { name: "search"; query: string }
  | { name: "title"; title: TitleSummary };

class Nav {
  route = $state<Route>({ name: "home" });
  private stack: Route[] = [];

  go(route: Route) {
    if (JSON.stringify(route) === JSON.stringify(this.route)) return;
    // Typing in search refines the current search instead of stacking pages.
    if (!(route.name === "search" && this.route.name === "search")) this.stack.push(this.route);
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

  /** Unfinished titles, most recent first. */
  get continueWatching(): HistoryEntry[] {
    return this.history
      .filter((h) => h.duration > 0 && h.position / h.duration < 0.95)
      .sort((a, b) => b.updatedAt - a.updatedAt);
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
