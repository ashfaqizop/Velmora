// Personalize and app-info tools, all read-only. Ani writes notes with the
// answer-stage tools profile_note and profile_forget instead.
import * as api from "../../api";
import { keyOf, library, settings } from "../../state.svelte";
import { personalize } from "../personalize.svelte";
import type { ResearchTool } from "./types";
import { num } from "./types";
import { brief, titleArg } from "./shape";

const when = (at: number) => new Date(at).toISOString().slice(0, 10);

const profileRead: ResearchTool = {
  decl: {
    name: "profile_read",
    description: "The user's Personalize profile in full: Ani's saved notes, favourite genres, recent watching and searches.",
    parameters: { type: "object", properties: {} },
  },
  label: () => "Reading your Personalize profile",
  async run() {
    return { profile: personalize.summary(), notes: personalize.notes.map((n) => ({ text: n.text, saved: when(n.at) })) };
  },
};

const tasteStats: ResearchTool = {
  decl: {
    name: "taste_stats",
    description: "Counts behind the user's taste: top genres, kinds and decades, weighted by watching over browsing.",
    parameters: { type: "object", properties: {} },
  },
  label: () => "Working out your taste",
  async run() {
    const s = personalize.stats();
    return {
      genres: s.genres.map(([name, score]) => ({ name, score })),
      kinds: s.kinds.map(([name, score]) => ({ name, score })),
      decades: s.decades.map(([name, score]) => ({ name, score })),
      activity: personalize.counts,
    };
  },
};

const searchHistory: ResearchTool = {
  decl: {
    name: "search_history",
    description: "The user's recent searches in Velmora, newest first.",
    parameters: { type: "object", properties: { limit: { type: "integer" } } },
  },
  label: () => "Checking your recent searches",
  async run(args) {
    return { searches: personalize.recentSearches(Math.min(50, num(args.limit) ?? 25)) };
  },
};

const watchHistory: ResearchTool = {
  decl: {
    name: "watch_history",
    description: "What the user watched, finished or downloaded recently, newest first, with how far they got.",
    parameters: { type: "object", properties: { limit: { type: "integer" } } },
  },
  label: () => "Checking what you've watched",
  async run(args, { refs }) {
    const n = Math.min(40, num(args.limit) ?? 20);
    const progress = new Map(library.history.map((h) => [keyOf(h.title), h]));
    const recent = personalize.recentTitles(["finish", "stream", "download"], n);
    // Titles played before Personalize existed still have a resume point.
    for (const h of library.history) {
      if (recent.length >= n) break;
      if (!recent.some((r) => keyOf(r) === keyOf(h.title))) recent.push({ ...h.title, last: "stream", at: h.updatedAt });
    }
    return {
      titles: recent.map((t) => {
        const h = progress.get(keyOf(t));
        const full = h?.title ?? { ...t, ids: t.ids ?? {} };
        return {
          ...brief(full, refs),
          last: t.last,
          when: when(t.at),
          episode: t.episode ?? (h?.episode !== undefined ? `S${h.season ?? 1} E${h.episode}` : undefined),
          percent: h && h.duration > 0 ? Math.round((h.position / h.duration) * 100) : undefined,
        };
      }),
    };
  },
};

const continueWatching: ResearchTool = {
  decl: {
    name: "continue_watching",
    description: "Titles the user started and hasn't finished, most recent first.",
    parameters: { type: "object", properties: {} },
  },
  label: () => "Checking Continue Watching",
  async run(_args, { refs }) {
    return {
      titles: library.continueWatching.slice(0, 15).map((h) => ({
        ...brief(h.title, refs),
        episode: h.episode !== undefined ? `S${h.season ?? 1} E${h.episode}` : undefined,
        percent: Math.round((h.position / h.duration) * 100),
        minutes_left: Math.round((h.duration - h.position) / 60),
      })),
    };
  },
};

const watchlist: ResearchTool = {
  decl: {
    name: "watchlist",
    description: "The titles on the user's My List.",
    parameters: { type: "object", properties: {} },
  },
  label: () => "Checking My List",
  async run(_args, { refs }) {
    return { titles: library.watchlist.slice(0, 40).map((t) => brief(t, refs)) };
  },
};

const isWatched: ResearchTool = {
  decl: {
    name: "is_watched",
    description: "Whether the user finished a title, or a given episode, and where they stopped.",
    parameters: {
      type: "object",
      properties: { ref: { type: "string" }, season: { type: "integer" }, episode: { type: "integer" } },
      required: ["ref"],
    },
  },
  label: (a) => `Checking if you've seen ${a.ref}`,
  async run(args, { refs }) {
    const t = titleArg(args, refs);
    const k = keyOf(t);
    const season = num(args.season);
    const episode = num(args.episode);
    const ep = episode !== undefined ? `S${season ?? 1} E${episode}` : undefined;
    const finished = personalize.events.filter((e) => e.kind === "finish" && e.title && keyOf(e.title) === k);
    const h = library.last(t);
    return {
      finished: ep ? finished.some((e) => e.episode === ep) : finished.length > 0,
      finished_episodes: finished.map((e) => e.episode).filter(Boolean).slice(0, 30),
      last_position: h
        ? {
            episode: h.episode !== undefined ? `S${h.season ?? 1} E${h.episode}` : undefined,
            percent: h.duration > 0 ? Math.round((h.position / h.duration) * 100) : undefined,
            when: when(h.updatedAt),
          }
        : null,
      on_my_list: library.has(t),
    };
  },
};

const getSettings: ResearchTool = {
  decl: {
    name: "get_settings",
    description: "The user's playback preferences: quality, codec, audio and subtitle languages, anime sub or dub.",
    parameters: { type: "object", properties: {} },
  },
  label: () => "Checking your settings",
  async run() {
    const p = await settings.load();
    return {
      preferred_quality: p.preferred_resolution >= 2160 ? "4K" : `${p.preferred_resolution}p`,
      auto_4k: p.max_auto_resolution >= 2160,
      codec: p.preferred_codec,
      audio_languages: p.audio_languages,
      subtitle_languages: p.subtitle_languages,
      anime_audio: settings.audioMode,
      torrents: p.include_torrents,
      show_adult: p.show_adult,
    };
  },
};

const syncStatus: ResearchTool = {
  decl: {
    name: "sync_status",
    description: "Whether AniList and Trakt are connected, and as whom.",
    parameters: { type: "object", properties: {} },
  },
  label: () => "Checking AniList and Trakt",
  async run() {
    const s = await api.syncStatus();
    return { anilist: s.anilistUser ?? (s.anilistAvailable ? "not connected" : "not available"), trakt: s.traktUser ?? (s.traktAvailable ? "not connected" : "not available") };
  },
};

const appInfo: ResearchTool = {
  decl: {
    name: "app_info",
    description: "Velmora's version and platform. Updates install by themselves when Velmora starts.",
    parameters: { type: "object", properties: {} },
  },
  label: () => "Checking Velmora's version",
  async run() {
    const i = await api.appInfo();
    return { version: i.version, platform: i.platform, metadata: i.tmdb ? "TMDB" : "Cinemeta" };
  },
};

export const profileTools: ResearchTool[] = [
  profileRead,
  tasteStats,
  searchHistory,
  watchHistory,
  continueWatching,
  watchlist,
  isWatched,
  getSettings,
  syncStatus,
  appInfo,
];
