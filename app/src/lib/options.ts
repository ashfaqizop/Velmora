// Static choices shown in Settings and onboarding.
import type { MediaKind, ProviderId } from "./types";

/** Mirrors ProviderId::display_name in the engine. */
const PROVIDER_NAMES: Record<string, string> = {
  cinemeta: "Cinemeta",
  tmdb: "TMDB",
  anilist: "AniList",
  hd_hub: "HDHub",
  hi_anime: "HiAnime",
  zoko_anime: "ZokoAnime",
  movie_box: "MovieBox",
  four_k_hd_hub: "4KHDHub",
  dramachi: "Dramachi",
  anime_sama: "Anime-Sama",
  french_stream: "French-Stream",
  coflix: "Coflix",
  yts: "YTS",
  eztv: "EZTV",
  nyaa: "Nyaa",
};

export function providerLabel(id: ProviderId) {
  return PROVIDER_NAMES[id] ?? id.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
}

/** Movies and series share one order in practice; both are shown so each can differ. */
export const PROVIDER_GROUPS: { kind: MediaKind; label: string }[] = [
  { kind: "movie", label: "Movies" },
  { kind: "series", label: "Series" },
  { kind: "anime", label: "Anime" },
  { kind: "asian_drama", label: "Asian drama" },
];

/** ISO 639-1 code and name. */
export const LANGUAGES: [string, string][] = [
  ["en", "English"],
  ["hi", "Hindi"],
  ["ja", "Japanese"],
  ["ko", "Korean"],
  ["zh", "Chinese"],
  ["fr", "French"],
  ["es", "Spanish"],
  ["pt", "Portuguese"],
  ["de", "German"],
  ["it", "Italian"],
  ["ar", "Arabic"],
  ["tr", "Turkish"],
  ["ru", "Russian"],
  ["ta", "Tamil"],
  ["te", "Telugu"],
  ["ml", "Malayalam"],
  ["bn", "Bengali"],
  ["id", "Indonesian"],
  ["th", "Thai"],
  ["vi", "Vietnamese"],
];
