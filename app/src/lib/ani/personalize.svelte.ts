// Personalize: what the user searches, opens, streams, downloads and finishes,
// plus the notes Ani keeps about their taste. Ani reads it before answering.
// Everything stays in this device's localStorage.
import type { MediaKind, Target, TitleDetail, TitleSummary } from "../types";

export type ActivityKind = "search" | "view" | "stream" | "finish" | "download";

/** A title as Personalize keeps it: enough to name it, show its poster and open it. */
export type SlimTitle = Pick<TitleSummary, "origin" | "name" | "year" | "kind" | "poster" | "ids"> & {
  genres?: string[];
};

export interface Activity {
  kind: ActivityKind;
  at: number;
  query?: string;
  title?: SlimTitle;
  /** "S1 E3" for episodes. */
  episode?: string;
  /** Where it played from: a provider name, or "download". */
  source?: string;
}

export interface Note {
  id: string;
  text: string;
  at: number;
}

const MAX_EVENTS = 400;
const MAX_NOTES = 60;
/** The same thing again within this window refreshes the old entry instead of adding one. */
const REPEAT_MS = 30 * 60_000;

const keyOf = (t: Pick<TitleSummary, "origin">) => `${t.origin.provider}:${t.origin.id}`;
const episodeOf = (target: Target) => (target.type === "episode" ? `S${target.season} E${target.episode}` : undefined);

function load<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(`velmora.ani.${key}`);
    return raw ? (JSON.parse(raw) as T) : fallback;
  } catch {
    return fallback;
  }
}

function save(key: string, value: unknown) {
  try {
    localStorage.setItem(`velmora.ani.${key}`, JSON.stringify(value));
  } catch (e) {
    console.warn("personalize save failed", e);
  }
}

class Personalize {
  /** Off stops recording; what's already saved stays until cleared. */
  learning = $state(localStorage.getItem("velmora.ani.learning") !== "0");
  events = $state<Activity[]>(load("events", []));
  notes = $state<Note[]>(load("notes", []));

  setLearning(on: boolean) {
    this.learning = on;
    localStorage.setItem("velmora.ani.learning", on ? "1" : "0");
  }

  // ── Recording ──

  search(query: string) {
    const q = query.trim();
    if (q) this.record({ kind: "search", at: Date.now(), query: q });
  }

  view(detail: TitleDetail) {
    this.record({ kind: "view", at: Date.now(), title: this.slim(detail.summary, detail.genres) });
  }

  stream(t: TitleSummary, target: Target, source: string) {
    this.record({ kind: "stream", at: Date.now(), title: this.slim(t), episode: episodeOf(target), source });
  }

  finish(t: TitleSummary, target: Target) {
    this.record({ kind: "finish", at: Date.now(), title: this.slim(t), episode: episodeOf(target) });
  }

  download(t: TitleSummary, target: Target) {
    this.record({ kind: "download", at: Date.now(), title: this.slim(t), episode: episodeOf(target) });
  }

  private record(e: Activity) {
    if (!this.learning) return;
    const same = (x: Activity) =>
      x.kind === e.kind &&
      e.at - x.at < REPEAT_MS &&
      (e.title ? x.title && keyOf(x.title) === keyOf(e.title) && x.episode === e.episode : x.query?.toLowerCase() === e.query?.toLowerCase());
    this.events = [e, ...this.events.filter((x) => !same(x))].slice(0, MAX_EVENTS);
    save("events", this.events);
  }

  /** Genres come from the title page; later events for the same title reuse them. */
  private slim(t: TitleSummary, genres?: string[]): SlimTitle {
    const k = keyOf(t);
    const known = genres?.length ? genres : this.events.find((x) => x.title && keyOf(x.title) === k)?.title?.genres;
    return {
      origin: { ...t.origin },
      name: t.name,
      year: t.year,
      kind: t.kind,
      poster: t.poster,
      ids: { ...t.ids },
      genres: known?.length ? [...known] : undefined,
    };
  }

  // ── Notes (written by Ani, removable by the user) ──

  addNote(text: string): Note {
    const clean = text.trim().slice(0, 280);
    const existing = this.notes.find((n) => n.text.toLowerCase() === clean.toLowerCase());
    if (existing) return existing;
    const note: Note = { id: crypto.randomUUID(), text: clean, at: Date.now() };
    this.notes = [note, ...this.notes].slice(0, MAX_NOTES);
    save("notes", this.notes);
    return note;
  }

  forgetNote(id: string) {
    this.notes = this.notes.filter((n) => n.id !== id);
    save("notes", this.notes);
  }

  forgetEvent(at: number) {
    this.events = this.events.filter((e) => e.at !== at);
    save("events", this.events);
  }

  clear() {
    this.events = [];
    this.notes = [];
    save("events", []);
    save("notes", []);
  }

  // ── Reading ──

  get counts() {
    const c = { search: 0, view: 0, stream: 0, finish: 0, download: 0 } as Record<ActivityKind, number>;
    for (const e of this.events) c[e.kind]++;
    return c;
  }

  /** Top genres, kinds and decades, weighted by how strongly each event shows interest. */
  stats() {
    const weight: Record<ActivityKind, number> = { search: 0, view: 1, stream: 2, download: 2, finish: 3 };
    const genres = new Map<string, number>();
    const kinds = new Map<MediaKind, number>();
    const decades = new Map<string, number>();
    const add = <K>(m: Map<K, number>, k: K, w: number) => m.set(k, (m.get(k) ?? 0) + w);
    for (const e of this.events) {
      const w = weight[e.kind];
      if (!e.title || !w) continue;
      for (const g of e.title.genres ?? []) add(genres, g, w);
      add(kinds, e.title.kind, w);
      if (e.title.year) add(decades, `${Math.floor(e.title.year / 10) * 10}s`, w);
    }
    const top = <K>(m: Map<K, number>, n: number) => [...m.entries()].sort((a, b) => b[1] - a[1]).slice(0, n);
    return { genres: top(genres, 8), kinds: top(kinds, 4), decades: top(decades, 4) };
  }

  /** Distinct titles from the most recent events of the given kinds. */
  recentTitles(kinds: ActivityKind[], n = 15): (SlimTitle & { last: ActivityKind; at: number; episode?: string })[] {
    const seen = new Set<string>();
    const out: (SlimTitle & { last: ActivityKind; at: number; episode?: string })[] = [];
    for (const e of this.events) {
      if (!e.title || !kinds.includes(e.kind)) continue;
      const k = keyOf(e.title);
      if (seen.has(k)) continue;
      seen.add(k);
      out.push({ ...e.title, last: e.kind, at: e.at, episode: e.episode });
      if (out.length >= n) break;
    }
    return out;
  }

  recentSearches(n = 12) {
    return this.events.filter((e) => e.kind === "search").slice(0, n).map((e) => e.query!);
  }

  /** A compact profile for Ani's system prompt. */
  summary(): string {
    const lines: string[] = [];
    if (this.notes.length) {
      lines.push("Notes Ani saved earlier:");
      for (const n of this.notes.slice(0, 30)) lines.push(`- ${n.text}`);
    }
    const s = this.stats();
    if (s.genres.length) lines.push(`Favourite genres: ${s.genres.map(([g]) => g).join(", ")}`);
    if (s.kinds.length) lines.push(`Watches mostly: ${s.kinds.map(([k]) => k).join(", ")}`);
    if (s.decades.length) lines.push(`Decades: ${s.decades.map(([d]) => d).join(", ")}`);
    const watched = this.recentTitles(["finish", "stream", "download"], 15);
    if (watched.length) {
      lines.push("Recently watched or downloaded:");
      for (const t of watched) lines.push(`- ${t.name}${t.year ? ` (${t.year})` : ""}, ${t.kind}, ${t.last}${t.episode ? ` ${t.episode}` : ""}`);
    }
    const viewed = this.recentTitles(["view"], 10).filter((v) => !watched.some((w) => keyOf(w) === keyOf(v)));
    if (viewed.length) lines.push(`Recently looked at: ${viewed.map((t) => t.name).join(", ")}`);
    const searches = this.recentSearches(10);
    if (searches.length) lines.push(`Recent searches: ${searches.join(", ")}`);
    return lines.length ? lines.join("\n") : "Nothing recorded yet. This may be a new user.";
  }
}

export const personalize = new Personalize();
