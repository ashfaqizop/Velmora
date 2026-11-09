// Stores for sports (the Home rail and its pages) and live TV (favorites, play).
import { M3eSnackbar } from "@m3e/web/snackbar";
import * as api from "./api";
import { defaultRegion } from "./sports";
import { configReady, nav } from "./state.svelte";
import type { SportsMatch, TitleSummary, TvChannel } from "./types";

class SportsStore {
  live = $state<SportsMatch[]>([]);
  upcoming = $state<SportsMatch[]>([]);
  results = $state<SportsMatch[]>([]);
  loading = $state(false);
  loaded = $state(false);
  /** Every list failed, so there is nothing to show. */
  failed = $state(false);
  /** Where "where to watch" is asked for (ISO country code). */
  region = $state(defaultRegion());
  private last = 0;

  setRegion(code: string) {
    this.region = code;
    localStorage.setItem("velmora.region", code);
  }

  /** Loads live, upcoming and results. Skips a load that ran in the last 30 s unless forced. */
  async load(force = false) {
    if (this.loading) return;
    if (!force && Date.now() - this.last < 30_000) return;
    this.loading = true;
    this.last = Date.now();
    try {
      await configReady;
      const [live, upcoming, results] = await Promise.allSettled([api.sportsLive(), api.sportsUpcoming(2), api.sportsResults(2)]);
      if (live.status === "fulfilled") this.live = live.value;
      if (upcoming.status === "fulfilled") this.upcoming = upcoming.value;
      if (results.status === "fulfilled") this.results = results.value;
      this.failed = [live, upcoming, results].every((r) => r.status === "rejected");
    } finally {
      this.loaded = true;
      this.loading = false;
    }
  }
}

export const sports = new SportsStore();

export interface FavChannel {
  id: string;
  name: string;
  logo?: string;
  country?: string;
}

const FAVORITES = "velmora.tv.favorites";

function readFavorites(): FavChannel[] {
  try {
    const raw = JSON.parse(localStorage.getItem(FAVORITES) ?? "[]");
    return Array.isArray(raw) ? raw : [];
  } catch {
    return [];
  }
}

/** A channel dressed as a title, so the player can show its name and logo. */
const asTitle = (ch: TvChannel): TitleSummary => ({
  origin: { provider: "iptv_org", id: ch.id },
  name: ch.name,
  kind: "movie",
  poster: ch.logo,
  ids: {},
  badges: ["Live"],
});

class LiveStore {
  favorites = $state<FavChannel[]>(readFavorites());

  isFavorite(id: string) {
    return this.favorites.some((f) => f.id === id);
  }

  toggle(ch: FavChannel) {
    this.favorites = this.isFavorite(ch.id)
      ? this.favorites.filter((f) => f.id !== ch.id)
      : [...this.favorites, { id: ch.id, name: ch.name, logo: ch.logo, country: ch.country }];
    localStorage.setItem(FAVORITES, JSON.stringify(this.favorites));
  }

  /** Opens the player on a channel. */
  play(ch: TvChannel) {
    nav.go({ name: "player", title: asTitle(ch), target: { type: "movie" }, mode: "sub", channel: ch });
  }

  /** Looks a channel up by id and plays it (favorites, "where to watch"). */
  async playById(id: string) {
    try {
      const ch = await api.tvChannel(id);
      if (ch) this.play(ch);
      else M3eSnackbar.open("That channel isn't available right now");
    } catch (e) {
      M3eSnackbar.open(`Couldn't open the channel: ${e}`, true);
    }
  }
}

export const live = new LiveStore();
