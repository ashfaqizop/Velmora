// TMDB lookups shared by the social site and Theatre, kept in Redis.
// Needs TMDB_READ_TOKEN (or TMDB_API_KEY) in Vercel's environment variables.

import { redis } from "./license.js";

export const TITLE_KEY = /^(movie|tv)\/\d{1,9}$/;
export const WEEK = 7 * 86400;

export async function tmdb(k) {
  const token = process.env.TMDB_READ_TOKEN;
  const key = process.env.TMDB_API_KEY;
  if (!token && !key) throw new Error("TMDB is not configured");
  const url = new URL(`https://api.themoviedb.org/3/${k}`);
  url.searchParams.set("language", "en-US");
  if (!token) url.searchParams.set("api_key", key);
  const r = await fetch(url, { headers: token ? { Authorization: `Bearer ${token}` } : {}, signal: AbortSignal.timeout(6000) });
  if (r.status === 404) return null;
  if (!r.ok) throw new Error(`tmdb ${r.status}`);
  const t = await r.json();
  if (t.adult) return null;
  const movie = k.startsWith("movie/");
  const date = movie ? t.release_date : t.first_air_date;
  const anime = !movie && (t.genres ?? []).some((g) => g.id === 16) && (t.origin_country ?? []).includes("JP");
  return {
    k,
    kind: movie ? "movie" : anime ? "anime" : "tv",
    name: String((movie ? t.title : t.name) ?? "").slice(0, 200),
    year: date ? Number(date.slice(0, 4)) || null : null,
    poster: t.poster_path ? `https://image.tmdb.org/t/p/w342${t.poster_path}` : null,
    backdrop: t.backdrop_path ? `https://image.tmdb.org/t/p/w1280${t.backdrop_path}` : null,
    overview: String(t.overview ?? "").slice(0, 700),
    genres: (t.genres ?? []).map((g) => g.name).slice(0, 6),
    runtime: movie ? t.runtime || null : (t.episode_run_time ?? [])[0] || null,
    seasons: movie ? null : t.number_of_seasons ?? null,
    tmdb: t.vote_average ? Math.round(t.vote_average * 10) / 10 : null,
  };
}


/** Any TMDB API path (lists like /trending/movie/week), as JSON. */
export async function tmdbGet(path, params = {}) {
  const token = process.env.TMDB_READ_TOKEN;
  const key = process.env.TMDB_API_KEY;
  if (!token && !key) throw new Error("TMDB is not configured");
  const url = new URL(`https://api.themoviedb.org/3${path}`);
  url.searchParams.set("language", "en-US");
  for (const [k, v] of Object.entries(params)) url.searchParams.set(k, String(v));
  if (!token) url.searchParams.set("api_key", key);
  const r = await fetch(url, { headers: token ? { Authorization: `Bearer ${token}` } : {}, signal: AbortSignal.timeout(8000) });
  if (!r.ok) throw new Error(`tmdb ${r.status}`);
  return r.json();
}

/** Look things up through a Redis cache: one MGET, then fetch what's missing. */
export async function cached(prefix, ids, fetchOne, ttl) {
  const keys = ids.map((id) => `${prefix}${id}`);
  const hits = await redis("MGET", ...keys);
  const out = {};
  const missing = [];
  ids.forEach((id, i) => {
    if (hits[i] === null || hits[i] === undefined) missing.push(id);
    else out[id] = hits[i] === "0" ? null : JSON.parse(hits[i]);
  });
  await Promise.all(
    missing.map(async (id) => {
      try {
        const v = await fetchOne(id);
        out[id] = v;
        await redis("SET", `${prefix}${id}`, v ? JSON.stringify(v) : "0", "EX", String(v ? ttl : 86400));
      } catch (e) {
        console.error("social lookup:", id, e.message);
        out[id] = null;
      }
    }),
  );
  return out;
}


/** TMDB details for titles, through the cache. */
export const titleDetails = (ks) => cached("tm:", ks, tmdb, WEEK);
