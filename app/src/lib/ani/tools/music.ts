// Music tools: YouTube Music search, charts, new releases, artists, albums
// and the user's liked songs. Songs, albums, artists and playlists come back
// with refs (s1, s2), so the cards Ani shows are made from real data and play
// only when the user presses play.
//
// Everything here needs the user signed in to YouTube Music in Velmora.
import * as api from "../../api";
import { musicSession } from "../../music/session.svelte";
import type { MusicItem, MusicSearchFilter } from "../../types";
import type { RefBook } from "../refs";
import type { ResearchTool } from "./types";
import { str } from "./types";

const SHOWN = 20;
const SIGNED_OUT = {
  error: "The user isn't signed in to YouTube Music in Velmora. Say they can sign in from Music in the sidebar; don't make up songs.",
};

function brief(i: MusicItem, refs: RefBook) {
  return {
    ref: refs.addMusic(i),
    kind: i.kind,
    title: i.title,
    ...(i.artists.length ? { artists: i.artists.map((a) => a.name).join(", ") } : i.subtitle ? { subtitle: i.subtitle } : {}),
    ...(i.album ? { album: i.album } : {}),
    ...(i.duration ? { seconds: i.duration } : {}),
    ...(i.explicit ? { explicit: true } : {}),
    ...(i.playCount ? { plays: i.playCount } : {}),
  };
}

const list = (items: MusicItem[], refs: RefBook) => items.slice(0, SHOWN).map((i) => brief(i, refs));

const FILTERS: MusicSearchFilter[] = ["all", "songs", "videos", "albums", "artists", "playlists"];

const musicSearch: ResearchTool = {
  decl: {
    name: "music_search",
    description:
      "Search YouTube Music for songs, albums, artists, music videos or playlists. Returns items with refs (s1...). Use it for any music question: a song, an artist's work, a mood ('chill lo-fi playlists').",
    parameters: {
      type: "object",
      properties: {
        query: { type: "string", description: "What to search for." },
        kind: { type: "string", enum: FILTERS, description: "Narrow to one kind. Defaults to all." },
      },
      required: ["query"],
    },
  },
  label: (a) => `Searching YouTube Music for "${str(a.query) ?? ""}"`,
  async run(args, { refs }) {
    if (!musicSession.signedIn) return SIGNED_OUT;
    const q = str(args.query);
    if (!q) return { error: "Give a query." };
    const kind = FILTERS.includes(args.kind as MusicSearchFilter) ? (args.kind as MusicSearchFilter) : "all";
    const r = await api.musicSearch(q, kind);
    return {
      ...(r.top ? { top: brief(r.top, refs) } : {}),
      sections: r.shelves.map((s) => ({ title: s.title, items: list(s.items, refs) })),
    };
  },
};

const musicCharts: ResearchTool = {
  decl: {
    name: "music_charts",
    description: "YouTube Music's charts (top songs, top music videos, trending) and new releases. Returns items with refs.",
    parameters: {
      type: "object",
      properties: { which: { type: "string", enum: ["charts", "new_releases"], description: "Defaults to charts." } },
    },
  },
  label: (a) => (a.which === "new_releases" ? "Checking new releases" : "Checking the music charts"),
  async run(args, { refs }) {
    if (!musicSession.signedIn) return SIGNED_OUT;
    const feed = await api.musicFeed(args.which === "new_releases" ? "FEmusic_new_releases" : "FEmusic_charts");
    return { sections: feed.shelves.slice(0, 5).map((s) => ({ title: s.title, items: list(s.items, refs) })) };
  },
};

const musicArtist: ResearchTool = {
  decl: {
    name: "music_artist",
    description: "An artist's page: top songs, albums, singles and similar artists. Pass an artist ref from music_search.",
    parameters: { type: "object", properties: { ref: { type: "string", description: "An artist ref (s...)." } }, required: ["ref"] },
  },
  label: () => "Looking at the artist",
  async run(args, { refs }) {
    if (!musicSession.signedIn) return SIGNED_OUT;
    const it = refs.music(String(args.ref ?? ""));
    if (!it || it.kind !== "artist") return { error: "That isn't an artist ref from this chat." };
    const a = await api.musicArtist(it.id);
    return {
      name: a.name,
      ...(a.description ? { about: a.description.slice(0, 600) } : {}),
      ...(a.monthlyAudience ? { audience: a.monthlyAudience } : {}),
      sections: a.shelves.map((s) => ({ title: s.title, items: list(s.items, refs) })),
    };
  },
};

const musicAlbum: ResearchTool = {
  decl: {
    name: "music_album",
    description: "An album's or playlist's songs. Pass an album or playlist ref.",
    parameters: { type: "object", properties: { ref: { type: "string", description: "An album or playlist ref (s...)." } }, required: ["ref"] },
  },
  label: () => "Opening the album",
  async run(args, { refs }) {
    if (!musicSession.signedIn) return SIGNED_OUT;
    const it = refs.music(String(args.ref ?? ""));
    if (!it || (it.kind !== "album" && it.kind !== "playlist")) return { error: "That isn't an album or playlist ref from this chat." };
    if (it.kind === "album") {
      const a = await api.musicAlbum(it.id);
      return { title: a.title, year: a.year, artists: a.artists.map((x) => x.name).join(", "), songs: list(a.tracks, refs) };
    }
    const p = await api.musicPlaylist(it.id);
    return { title: p.title, songs: list(p.tracks, refs) };
  },
};

const musicRadio: ResearchTool = {
  decl: {
    name: "music_similar",
    description: "Songs like one song (YouTube Music's radio for it). Pass a song ref. Good for 'more like this'.",
    parameters: { type: "object", properties: { ref: { type: "string", description: "A song ref (s...)." } }, required: ["ref"] },
  },
  label: () => "Finding similar songs",
  async run(args, { refs }) {
    if (!musicSession.signedIn) return SIGNED_OUT;
    const it = refs.music(String(args.ref ?? ""));
    if (!it || (it.kind !== "song" && it.kind !== "video")) return { error: "That isn't a song ref from this chat." };
    const q = await api.musicRadio(it.id);
    return { songs: list(q.tracks.filter((t) => t.id !== it.id), refs) };
  },
};

const musicLiked: ResearchTool = {
  decl: {
    name: "music_liked",
    description: "The user's liked songs on YouTube Music, newest first. Use to learn their music taste before recommending.",
    parameters: { type: "object", properties: {} },
  },
  label: () => "Reading your liked songs",
  async run(_args, { refs }) {
    if (!musicSession.signedIn) return SIGNED_OUT;
    const p = await api.musicLiked(60);
    return { total: p.tracks.length, songs: list(p.tracks, refs) };
  },
};

export const musicTools: ResearchTool[] = [musicSearch, musicCharts, musicArtist, musicAlbum, musicRadio, musicLiked];
