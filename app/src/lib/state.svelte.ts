// App-wide state: navigation, the home feed, and the local library
// (My List and watch history live in localStorage until sync lands).
import * as api from "./api";
import { transition } from "./vt";
import type {
  AniKeyStatus,
  AudioMode,
  DownloadStatus,
  HistoryEntry,
  MediaKind,
  Preferences,
  Rail,
  SportsMatch,
  Target,
  TitleDetail,
  TitleSummary,
  TvChannel,
} from "./types";

export type Browse = "movie" | "series" | "anime";

export type Route =
  | { name: "home" }
  | { name: "browse"; kind: Browse }
  | { name: "collection"; id: string }
  | { name: "mylist" }
  | { name: "downloads" }
  | { name: "search"; query: string }
  | { name: "title"; title: TitleSummary }
  /** `page` and `setting` open Settings on one page, at one row. */
  | { name: "settings"; page?: string; setting?: string }
  | { name: "ani" }
  | { name: "live" }
  | { name: "theatre"; tab?: "lineup" | "tickets" }
  | { name: "screening"; id: string }
  | { name: "sports" }
  | { name: "match"; match: SportsMatch }
  // Music mode (YouTube Music). Every route starting with "music" belongs to it.
  | { name: "music" }
  | { name: "music-explore"; tab?: "charts" | "new" | "moods" }
  | { name: "music-search"; query: string }
  | { name: "music-album"; id: string }
  | { name: "music-artist"; id: string }
  | { name: "music-playlist"; id: string }
  | { name: "music-library"; tab?: "liked" | "playlists" | "albums" | "artists" | "downloaded" }
  | { name: "music-feed"; browseId: string; params?: string | null; title: string }
  /** A list Ani showed, opened in full from "See all". */
  | { name: "list"; title: string; items: TitleSummary[] }
  | {
      name: "player";
      title: TitleSummary;
      target: Target;
      mode: AudioMode;
      detail?: TitleDetail;
      /** A finished download to play offline instead of looking up sources. */
      file?: string;
      /** A live TV channel: its streams stand in for the source search. */
      channel?: TvChannel;
      /** A source to try first (picked from "Where to watch"), by URL. */
      source?: string;
    };

/** Music mode: the routes that belong to YouTube Music. */
export const isMusic = (r: Route) => r.name.startsWith("music");

/** Pages in the sidebar. Moving between them is a sideways step, not a deeper one. */
function topLevel(r: Route): boolean {
  switch (r.name) {
    case "home":
    case "browse":
    case "mylist":
    case "downloads":
    case "settings":
    case "ani":
    case "live":
    case "sports":
    case "music":
    case "music-explore":
    case "music-library":
      return true;
    case "music-playlist":
      return r.id === "LM";
    default:
      return false;
  }
}

/** A page left behind: where it was scrolled to, and the card that was opened from it. */
interface Visit {
  route: Route;
  scroll: number;
  /** `data-vt-key` of the card that opened the next page, for the way back,
   *  and which of the cards with that key it was (a title can be in two rows). */
  key?: string;
  index?: number;
}

/** Every card on the page with this `data-vt-key`, in page order. */
const cardsWith = (key: string) => [...document.querySelectorAll(`[data-vt-key="${CSS.escape(key)}"]`)];

class Nav {
  route = $state<Route>({ name: "home" });
  private stack: Visit[] = [];
  private ahead: Visit[] = [];
  /** Where to land when leaving Music mode. */
  private lastVelmora: Route = { name: "home" };
  /** The scrolling page area (Shell sets it). */
  scroller: HTMLElement | null = null;
  canGoForward = $state(false);

  get musicMode() {
    return isMusic(this.route);
  }

  /** Leave Music mode for the page you were on before it. */
  leaveMusic() {
    this.go(this.lastVelmora);
  }

  private get scroll(): number {
    return this.scroller?.scrollTop ?? 0;
  }

  /** Scroll to `top` now, and again as the page grows (rows that load). */
  private restoreScroll(top: number) {
    const el = this.scroller;
    if (!el) return;
    el.scrollTop = top;
    if (top <= 0) return;
    const until = performance.now() + 900;
    const again = () => {
      if (!this.scroller || this.scroller !== el) return;
      if (Math.abs(el.scrollTop - top) > 2 && el.scrollHeight - el.clientHeight >= top - 2) el.scrollTop = top;
      if (el.scrollTop < top - 2 && performance.now() < until) requestAnimationFrame(again);
    };
    requestAnimationFrame(again);
  }

  /**
   * Open a page. `from` is the card that was clicked: its artwork grows into
   * the new page's (marked `data-vt-hero`), and shrinks back to it on Back.
   */
  go(route: Route, opts: { from?: Element | null; key?: string } = {}) {
    if (JSON.stringify(route) === JSON.stringify(this.route)) return;
    const prev = this.route;
    if (isMusic(route) && !isMusic(prev) && prev.name !== "player") this.lastVelmora = prev;
    // Typing in search refines the current search instead of stacking pages.
    const refining =
      (route.name === "search" && prev.name === "search") || (route.name === "music-search" && prev.name === "music-search");
    const visit: Visit = {
      route: prev,
      scroll: this.scroll,
      key: opts.key,
      index: opts.key && opts.from ? Math.max(0, cardsWith(opts.key).indexOf(opts.from)) : undefined,
    };
    const apply = () => {
      if (!refining) this.stack.push(visit);
      this.ahead = [];
      this.canGoForward = false;
      this.route = route;
    };
    const player = route.name === "player" || prev.name === "player";
    if (refining || player) {
      apply();
      if (!refining && !player) this.restoreScroll(0);
      return;
    }
    const kind = route.name === "search" || route.name === "music-search" ? "search" : topLevel(route) && topLevel(prev) ? "switch" : "forward";
    const from = opts.from;
    transition(kind, apply, {
      after: () => this.restoreScroll(0),
      shared: from ? [{ name: "hero", from: () => from, to: () => document.querySelector("[data-vt-hero]") }] : [],
    });
  }

  /** Swap the current page without adding a history step (next episode). */
  replace(route: Route) {
    this.route = route;
  }

  back() {
    const visit = this.stack.pop() ?? { route: { name: "home" } as Route, scroll: 0 };
    this.ahead.push({ route: this.route, scroll: this.scroll });
    this.canGoForward = true;
    this.travel("back", visit);
  }

  /** The mouse's forward button: undo a Back. */
  forward() {
    const visit = this.ahead.pop();
    if (!visit) return;
    this.stack.push({ route: this.route, scroll: this.scroll });
    this.canGoForward = this.ahead.length > 0;
    this.travel("forward", visit);
  }

  private travel(kind: "back" | "forward", visit: Visit) {
    const prev = this.route;
    const apply = () => (this.route = visit.route);
    if (prev.name === "player" || visit.route.name === "player") {
      apply();
      return;
    }
    const key = visit.key;
    const card = () => (key ? (cardsWith(key)[visit.index ?? 0] ?? cardsWith(key)[0]) : null);
    const sideways = topLevel(prev) && topLevel(visit.route);
    transition(sideways ? "switch" : kind, apply, {
      after: () => {
        this.restoreScroll(visit.scroll);
        // The card this page was opened from, brought into view in its row.
        card()?.scrollIntoView({ block: "nearest", inline: "nearest" });
      },
      shared: key ? [{ name: "hero", from: () => document.querySelector("[data-vt-hero]"), to: card }] : [],
    });
  }

  get canGoBack() {
    return this.route.name !== "home" && this.route.name !== "music";
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

/** Tonight's pick: one well-rated title with artwork, stable for the whole day. */
export function tonightPick(): TitleSummary | null {
  const pool = feed.rails.flatMap((r) => r.items).filter((i) => i.backdrop && (i.rating ?? 0) >= 7);
  if (!pool.length) return null;
  const day = Math.floor(Date.now() / 86_400_000);
  return pool[day % pool.length];
}

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

  /** Told about every change made on this PC, so sync can send it (see sync.svelte.ts). */
  onChange: ((e: { table: "list" | "progress"; key: string; deleted: boolean }) => void) | null = null;
  /** Told when you add something to My List yourself (Velmora Social activity). */
  onAdd: ((t: TitleSummary) => void) | null = null;

  has(t: TitleSummary) {
    const k = keyOf(t);
    return this.watchlist.some((w) => keyOf(w) === k);
  }

  toggle(t: TitleSummary) {
    const k = keyOf(t);
    const removing = this.has(t);
    this.watchlist = removing ? this.watchlist.filter((w) => keyOf(w) !== k) : [$state.snapshot(t), ...this.watchlist];
    save("watchlist", this.watchlist);
    this.onChange?.({ table: "list", key: k, deleted: removing });
    if (!removing) this.onAdd?.(t);
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
    this.onChange?.({ table: "progress", key: k, deleted: false });
  }

  forget(t: TitleSummary) {
    const k = keyOf(t);
    this.history = this.history.filter((h) => keyOf(h.title) !== k);
    save("history", this.history);
    this.onChange?.({ table: "progress", key: k, deleted: true });
  }

  // ── Changes that arrive from your account (sync). They don't count as edits made here. ──

  /** Put a list entry in, or take it out (title null). Newest first, by `at`. */
  applyList(key: string, title: TitleSummary | null, order: (k: string) => number) {
    const rest = this.watchlist.filter((w) => keyOf(w) !== key);
    const next = title ? [title, ...rest] : rest;
    next.sort((a, b) => order(keyOf(b)) - order(keyOf(a)));
    this.watchlist = next;
    save("watchlist", this.watchlist);
  }

  /** Put a progress entry in, or take it out (entry null). Keeps the newest 200. */
  applyProgress(key: string, entry: HistoryEntry | null) {
    const rest = this.history.filter((h) => keyOf(h.title) !== key);
    this.history = (entry ? [entry, ...rest] : rest).sort((a, b) => b.updatedAt - a.updatedAt).slice(0, 200);
    save("history", this.history);
  }

  /** Empty this PC's list and history (another account is signing in). */
  clear() {
    this.watchlist = [];
    this.history = [];
    save("watchlist", []);
    save("history", []);
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

/** `withHours` keeps the hour field even at 0, so labels don't change length mid-video. */
export function formatTime(seconds: number | null | undefined, withHours = false) {
  const s = Math.max(0, Math.floor(seconds ?? 0));
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = String(s % 60).padStart(2, "0");
  return h || withHours ? `${h}:${String(m).padStart(2, "0")}:${sec}` : `${m}:${sec}`;
}

const ACTIVE = new Set(["queued", "running", "extracting"]);

/** Download queue, polled from the engine while anything is moving. */
class Downloads {
  items = $state<DownloadStatus[]>([]);
  private timer: ReturnType<typeof setInterval> | undefined;
  private watchers = 0;

  get active() {
    return this.items.filter((d) => ACTIVE.has(d.state.state)).length;
  }

  async refresh() {
    try {
      this.items = await api.downloadsList();
    } catch (e) {
      console.warn("downloads list failed", e);
    }
    if (!this.active && !this.watchers) this.stop();
  }

  /** Keep polling while the caller is interested (the Downloads page). */
  watch(): () => void {
    this.watchers++;
    this.start();
    return () => {
      this.watchers--;
    };
  }

  /** Poll until the queue settles (after starting a download). */
  start() {
    if (!this.timer) this.timer = setInterval(() => this.refresh(), 1000);
    this.refresh();
  }

  private stop() {
    clearInterval(this.timer);
    this.timer = undefined;
  }
}

export const downloads = new Downloads();

/** Engine preferences plus a few UI-only choices kept in localStorage. */
class Settings {
  prefs = $state<Preferences | null>(null);
  audioMode = $state<AudioMode>((localStorage.getItem("velmora.audioMode") as AudioMode | null) ?? "sub");
  accent = $state(localStorage.getItem("velmora.accent") ?? DEFAULT_ACCENT);

  async load() {
    if (!this.prefs) this.prefs = await api.getPreferences();
    return this.prefs;
  }

  /** Apply a change and save it; the engine uses it from the next request on. */
  async update(patch: Partial<Preferences>) {
    const current = await this.load();
    const next = { ...$state.snapshot(current), ...patch } as Preferences;
    this.prefs = next;
    await api.setPreferences(next);
  }

  setAudioMode(m: AudioMode) {
    this.audioMode = m;
    localStorage.setItem("velmora.audioMode", m);
  }

  setAccent(color: string) {
    this.accent = color;
    localStorage.setItem("velmora.accent", color);
    applyAccent(color);
  }
}

export const DEFAULT_ACCENT = "#3271FE";

export function applyAccent(color: string) {
  document.getElementById("theme")?.setAttribute("color", color);
}

export const settings = new Settings();

/** Connectivity, from the WebView's online/offline events. */
class Network {
  online = $state(navigator.onLine);

  constructor() {
    window.addEventListener("offline", () => (this.online = false));
    window.addEventListener("online", () => {
      this.online = true;
      // Pick up where a failed launch left off.
      if (feed.error || !feed.rails.length) feed.load();
    });
  }
}

export const network = new Network();

export const ANI_CHAT_MODEL = "gemini-3.5-flash";
export const ANI_TOOL_MODEL = "gemini-3.1-flash-lite";

/** Ani's switch, models and key status. The key itself stays in the Rust side. */
class AniSettings {
  enabled = $state(localStorage.getItem("velmora.ani.enabled") === "1");
  chatModel = $state(localStorage.getItem("velmora.ani.chatModel") ?? ANI_CHAT_MODEL);
  toolModel = $state(localStorage.getItem("velmora.ani.toolModel") ?? ANI_TOOL_MODEL);
  key = $state<AniKeyStatus | null>(null);
  /** A question another page wants typed into Ani's box (the user still sends it). */
  prefill = $state<string | null>(null);

  /** Open Ani with a question ready to send. */
  ask(text: string) {
    if (!this.enabled) this.setEnabled(true);
    this.prefill = text;
    nav.go({ name: "ani" });
  }

  get ready() {
    return this.enabled && !!this.key?.set;
  }

  async loadKey() {
    if (!this.key) this.key = await api.aniKeyStatus();
    return this.key;
  }

  /** Throws Gemini's reason when the key is refused. */
  async saveKey(key: string | null) {
    this.key = await api.aniSetKey(key);
  }

  setEnabled(on: boolean) {
    this.enabled = on;
    localStorage.setItem("velmora.ani.enabled", on ? "1" : "0");
    if (!on && nav.route.name === "ani") nav.go({ name: "home" });
  }

  setChatModel(id: string) {
    this.chatModel = id;
    localStorage.setItem("velmora.ani.chatModel", id);
  }

  setToolModel(id: string) {
    this.toolModel = id;
    localStorage.setItem("velmora.ani.toolModel", id);
  }
}

export const ani = new AniSettings();
