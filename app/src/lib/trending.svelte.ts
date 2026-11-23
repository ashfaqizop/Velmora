// "Popular on Velmora": the titles most watched across Velmora this week, from
// the anonymous counts (see analytics.svelte.ts). Shown on Home once enough
// different people have watched at least a few titles.
import * as api from "./api";
import type { TitleSummary } from "./types";

const KEY = "velmora.trending";
const FRESH_MS = 6 * 3_600_000;

class Trending {
  items = $state<TitleSummary[]>(read()?.items ?? []);

  async load() {
    const cached = read();
    if (cached && Date.now() - cached.at < FRESH_MS) {
      this.items = cached.items;
      return;
    }
    try {
      const r = await api.analyticsTrending();
      const keys = (r.titles ?? []).map((t) => t.k);
      const items = keys.length >= 4 ? await api.tmdbTitles(keys) : [];
      this.items = items;
      try {
        localStorage.setItem(KEY, JSON.stringify({ at: Date.now(), items }));
      } catch {
        // Fetched again next time.
      }
    } catch {
      // Offline or the server is busy: keep whatever was shown last.
    }
  }
}

function read(): { at: number; items: TitleSummary[] } | null {
  try {
    const v = JSON.parse(localStorage.getItem(KEY) ?? "null");
    return v && Array.isArray(v.items) ? v : null;
  } catch {
    return null;
  }
}

export const trending = new Trending();
