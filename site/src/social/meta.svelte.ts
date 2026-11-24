// Names, posters and genres for titles (from TMDB) and songs (from YouTube),
// looked up through /api/social/titles and /songs and kept for this visit.
import { SvelteMap } from "svelte/reactivity";
import { MAIN_SITE } from "../auth/config";

export interface TitleMeta {
  k: string;
  kind: "movie" | "tv" | "anime";
  name: string;
  year: number | null;
  poster: string | null;
  backdrop: string | null;
  overview: string;
  genres: string[];
  runtime: number | null;
  seasons: number | null;
  tmdb: number | null;
}

export interface SongMeta {
  id: string;
  title: string;
  artist: string;
}

/** Missing: not asked yet. null: TMDB doesn't know it (or it's 18+). */
export const titles = new SvelteMap<string, TitleMeta | null>();
export const songs = new SvelteMap<string, SongMeta | null>();

const asked = new Set<string>();

async function batch<T>(ids: string[], url: (chunk: string[]) => string, pick: (j: unknown) => Record<string, T | null>, into: SvelteMap<string, T | null>, prefix: string) {
  const todo = [...new Set(ids)].filter((id) => !asked.has(prefix + id));
  if (!todo.length) return;
  todo.forEach((id) => asked.add(prefix + id));
  const chunks: string[][] = [];
  for (let i = 0; i < todo.length; i += 40) chunks.push(todo.slice(i, i + 40));
  await Promise.all(
    chunks.map(async (chunk) => {
      try {
        const r = await fetch(url(chunk));
        if (!r.ok) throw new Error(String(r.status));
        const found = pick(await r.json());
        for (const id of chunk) into.set(id, found[id] ?? null);
      } catch {
        // Try again next time something asks.
        chunk.forEach((id) => asked.delete(prefix + id));
      }
    }),
  );
}

export const loadTitles = (ks: string[]) =>
  batch(
    ks.filter((k) => /^(movie|tv)\/\d{1,9}$/.test(k)),
    (c) => `/api/social/titles?ks=${c.join(",")}`,
    (j) => (j as { titles?: Record<string, TitleMeta | null> }).titles ?? {},
    titles,
    "t:",
  );

export const loadSongs = (ids: string[]) =>
  batch(
    ids.filter((id) => /^[A-Za-z0-9_-]{11}$/.test(id)),
    (c) => `/api/social/songs?ids=${c.join(",")}`,
    (j) => (j as { songs?: Record<string, SongMeta | null> }).songs ?? {},
    songs,
    "s:",
  );

/** "Open in Velmora" for a title or song (the website page tries the app first). */
export const openInApp = (to: string) => `${MAIN_SITE}/open/?to=${to}`;
export const songThumb = (id: string) => `https://i.ytimg.com/vi/${id}/hqdefault.jpg`;

export const kindLabel = (k: string, meta?: TitleMeta | null) => (meta?.kind === "anime" ? "Anime" : k.startsWith("movie/") ? "Movie" : "Series");
