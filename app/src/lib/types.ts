// Mirrors of the UniverseAPI engine's serde types (snake_case enums).

export type ProviderId = string;
export type MediaKind = "movie" | "series" | "anime" | "asian_drama";

export interface ProviderRef {
  provider: ProviderId;
  id: string;
}

export interface ExternalIds {
  imdb?: string;
  tmdb?: number;
  mal?: number;
  anilist?: number;
}

export interface TitleSummary {
  origin: ProviderRef;
  name: string;
  original_name?: string;
  kind: MediaKind;
  year?: number;
  poster?: string;
  backdrop?: string;
  overview?: string;
  rating?: number;
  ids: ExternalIds;
  badges?: string[];
  also_on?: ProviderRef[];
}

export interface Episode {
  season: number;
  number: number;
  title?: string;
  overview?: string;
  air_date?: string;
  thumbnail?: string;
}

export interface Season {
  number: number;
  name?: string;
  episodes: Episode[];
}

export interface TitleDetail {
  summary: TitleSummary;
  genres: string[];
  cast: string[];
  directors: string[];
  runtime_minutes?: number;
  trailer?: string;
  screenshots: string[];
  seasons: Season[];
}

export interface Rail {
  id: string;
  title: string;
  items: TitleSummary[];
}

export interface SearchBatch {
  provider: ProviderId;
  items: TitleSummary[];
  error?: string;
}

export interface AppInfo {
  version: string;
  platform: string;
  tmdb: boolean;
}

export interface ConfigStatus {
  updated: boolean;
  message?: string | null;
}

/** A resume point, written by the player. */
export interface HistoryEntry {
  title: TitleSummary;
  season?: number;
  episode?: number;
  position: number;
  duration: number;
  updatedAt: number;
}
