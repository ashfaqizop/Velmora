// Compact shapes for tool results. Gemini sees refs and the few fields it
// needs to reason; the full objects stay in the RefBook for the UI.
import * as api from "../../api";
import { audioLabel, formatBytes, hostLabel, releaseTags, subtitleLabel } from "../../sources";
import { providerLabel } from "../../options";
import type { RefBook } from "../refs";
import type { Source, Target, TitleDetail, TitleSummary } from "../../types";
import { num, str } from "./types";

const clipText = (s: string | undefined, n: number) => (s && s.length > n ? `${s.slice(0, n).trimEnd()}…` : s);

export function brief(t: TitleSummary, refs: RefBook) {
  return {
    ref: refs.addTitle(t),
    name: t.name,
    original_name: t.original_name,
    year: t.year,
    kind: t.kind,
    rating: t.rating ? Math.round(t.rating * 10) / 10 : undefined,
    overview: clipText(t.overview, 200),
  };
}

export function briefs(list: TitleSummary[], refs: RefBook, n = 12) {
  const seen = new Set<string>();
  const out = [];
  for (const t of list) {
    const k = `${t.origin.provider}:${t.origin.id}`;
    if (seen.has(k)) continue;
    seen.add(k);
    out.push(brief(t, refs));
    if (out.length >= n) break;
  }
  return out;
}

/** The title a `ref` argument names, or a clear error for the model. */
export function titleArg(args: Record<string, unknown>, refs: RefBook): TitleSummary {
  const ref = str(args.ref);
  const t = ref ? refs.title(ref) : undefined;
  if (!t) throw new Error(`Unknown title ref "${ref ?? ""}". Use a ref from an earlier result.`);
  return t;
}

/** Episode, season or movie, from optional `season` and `episode` arguments. */
export function targetArg(t: TitleSummary, args: Record<string, unknown>): Target {
  const season = num(args.season);
  const episode = num(args.episode);
  if (t.kind === "movie") return { type: "movie" };
  if (season !== undefined && episode !== undefined) return { type: "episode", season, episode };
  return { type: "episode", season: season ?? 1, episode: episode ?? 1 };
}

// Title details are fetched by several tools in one turn; share the request.
const details = new Map<string, Promise<TitleDetail>>();
export function detailOf(t: TitleSummary): Promise<TitleDetail> {
  const k = `${t.origin.provider}:${t.origin.id}`;
  let p = details.get(k);
  if (!p) {
    p = api.detail(t.origin);
    p.catch(() => details.delete(k));
    details.set(k, p);
    if (details.size > 60) details.delete(details.keys().next().value!);
  }
  return p;
}

/** TMDB path for a title ("movie/123", "tv/456"), when it has a TMDB id. */
export function tmdbPath(t: TitleSummary): string | undefined {
  if (t.origin.provider === "tmdb") return t.origin.id;
  if (t.ids.tmdb) return `${t.kind === "movie" ? "movie" : "tv"}/${t.ids.tmdb}`;
  return undefined;
}

/** An AniList media node from a raw query, as a TitleSummary. */
export function anilistSummary(m: Record<string, any>): TitleSummary {
  const title = m.title ?? {};
  return {
    origin: { provider: "anilist", id: String(m.id) },
    name: title.english ?? title.romaji ?? "Untitled",
    original_name: title.romaji && title.romaji !== title.english ? title.romaji : undefined,
    kind: "anime",
    year: m.seasonYear ?? m.startDate?.year ?? undefined,
    poster: m.coverImage?.extraLarge ?? m.coverImage?.large ?? undefined,
    backdrop: m.bannerImage ?? undefined,
    overview: typeof m.description === "string" ? m.description.replace(/<[^>]+>/g, "") : undefined,
    rating: m.averageScore ? m.averageScore / 10 : undefined,
    ids: { anilist: m.id, mal: m.idMal ?? undefined },
    badges: m.format ? [String(m.format).replace(/_/g, " ")] : [],
  };
}

export const ANILIST_MEDIA =
  "id idMal title { romaji english } coverImage { extraLarge large } bannerImage description(asHtml: false) averageScore seasonYear startDate { year } format status episodes";

export function link(s: Source, title: TitleSummary, target: Target, refs: RefBook) {
  const t = s.tags;
  const size = s.size_bytes ?? t.size_bytes;
  return {
    ref: refs.addLink(s, title, target),
    provider: providerLabel(s.provider),
    host: hostLabel(s),
    quality: t.resolution ? (t.resolution >= 2160 ? "4K" : `${t.resolution}p`) : undefined,
    codec: t.codec,
    hdr: t.hdr && t.hdr !== "sdr" ? t.hdr : undefined,
    size: size ? formatBytes(size) : undefined,
    format: s.format,
    audio: audioLabel(s),
    subtitles: subtitleLabel(s),
    release: releaseTags(s).join(", ") || undefined,
    french: t.french,
    season_pack: s.is_pack || undefined,
    torrent: s.torrent ? { seeders: s.torrent.seeders, leechers: s.torrent.leechers } : undefined,
    verified: s.verified,
  };
}

export { clipText };
