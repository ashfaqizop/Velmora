// Finding a title's sources ahead of time.
//
// A title page asks for the sources of whatever Play would play (and of an
// episode you point at) as soon as you've settled on it. Each site's answer
// shows in the page's "where to watch" row as it arrives, and pressing Play
// then takes the result that's ready, or waits for the same search still
// running, instead of starting from nothing. The engine caches every site's
// answer (until its links expire), so coming back to a title is instant too.
import * as api from "./api";
import { keyOf } from "./state.svelte";
import type { AudioMode, ProviderId, Source, SourcesReport, Target, TitleDetail, TitleSummary } from "./types";

/**
 * The title to look sources up for: the one you opened, with what its
 * details page knows added (IMDb and other ids, every site it's on). The
 * title page and the player both use this, so they share one search.
 */
export function lookupTitle(title: TitleSummary, detail?: TitleDetail | null): TitleSummary {
  if (!detail) return title;
  const d = detail.summary;
  const also = [...(title.also_on ?? [])];
  for (const r of [d.origin, ...(d.also_on ?? [])]) {
    if (r.provider !== title.origin.provider && !also.some((x) => x.provider === r.provider)) also.push(r);
  }
  return { ...title, ids: { ...d.ids, ...title.ids }, also_on: also, year: title.year ?? d.year };
}

export interface FoundSite {
  provider: ProviderId;
  state: "looking" | "found" | "none" | "failed";
  sources: Source[];
  /** Answered from the cache. */
  cached: boolean;
}

/** One search: a title, a movie or episode, sub or dub. */
export class Finding {
  sites = $state<FoundSite[]>([]);
  report = $state<SourcesReport | null>(null);
  readonly started = Date.now();
  readonly promise: Promise<SourcesReport>;

  constructor(title: TitleSummary, target: Target, mode: AudioMode) {
    this.promise = api
      .sourcesWatch($state.snapshot(title) as TitleSummary, target, mode, (e) => this.event(e))
      .then((r) => {
        this.report = r;
        // Anything the plan named that never answered is done now.
        this.sites = this.sites.map((s) => (s.state === "looking" ? { ...s, state: "none" } : s));
        return r;
      });
  }

  get done(): boolean {
    return this.report !== null;
  }

  /** The source Play would start: the best ranked, once every site has answered. */
  get best(): Source | null {
    return this.report?.sources[0] ?? null;
  }

  private event(e: api.SourcesEvent) {
    if (e.kind === "plan") {
      this.sites = e.providers.map((provider) => ({ provider, state: "looking", sources: [], cached: false }));
      return;
    }
    const site: FoundSite = {
      provider: e.provider,
      state: e.error ? "failed" : e.sources.length ? "found" : "none",
      sources: e.sources,
      cached: e.cached,
    };
    const i = this.sites.findIndex((s) => s.provider === e.provider);
    if (i < 0) this.sites = [...this.sites, site];
    else this.sites = this.sites.map((s, j) => (j === i ? site : s));
  }
}

const KEY = "velmora.sources.ahead";
/** A finished search is reused this long; after that the engine's cache answers. */
const REUSE_MS = 10 * 60 * 1000;

function targetKey(t: Target): string {
  return t.type === "episode" ? `s${t.season}e${t.episode}` : t.type === "season" ? `s${t.season}` : "movie";
}

class Finder {
  /** Settings: look for sources while you browse a title. */
  ahead = $state(localStorage.getItem(KEY) !== "0");
  private found = new Map<string, Finding>();

  private key(title: TitleSummary, target: Target, mode: AudioMode): string {
    return `${title.ids?.imdb ?? keyOf(title)}|${targetKey(target)}|${mode}`;
  }

  setAhead(on: boolean) {
    this.ahead = on;
    localStorage.setItem(KEY, on ? "1" : "0");
  }

  /** The search for this title, running or done, if there is one. */
  peek(title: TitleSummary, target: Target, mode: AudioMode): Finding | null {
    const f = this.found.get(this.key(title, target, mode));
    return f && !(f.done && Date.now() - f.started > REUSE_MS) ? f : null;
  }

  /** Start (or join) the search for this title. */
  find(title: TitleSummary, target: Target, mode: AudioMode): Finding {
    const existing = this.peek(title, target, mode);
    if (existing) return existing;
    const key = this.key(title, target, mode);
    const f = new Finding(title, target, mode);
    this.found.set(key, f);
    f.promise.catch(() => this.found.get(key) === f && this.found.delete(key));
    // Keep the map small: the oldest searches go first.
    if (this.found.size > 40) this.found.delete(this.found.keys().next().value!);
    return f;
  }

  /** For Play: the sources, from the search already running or done, or a new one. */
  get(title: TitleSummary, target: Target, mode: AudioMode): Promise<SourcesReport> {
    return this.find(title, target, mode).promise;
  }

  /** A source didn't play: drop what's remembered so the next look is fresh. */
  forget(title: TitleSummary, target: Target, mode: AudioMode) {
    this.found.delete(this.key(title, target, mode));
    api.forgetSources($state.snapshot(title) as TitleSummary, target, mode).catch(() => {});
  }
}

export const finder = new Finder();
