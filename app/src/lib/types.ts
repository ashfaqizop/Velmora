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

export interface Person {
  name: string;
  /** Character for cast, job for crew. */
  role?: string;
  photo?: string;
}

export interface TitleExtra {
  tagline?: string;
  release_date?: string;
  status?: string;
  certification?: string;
  original_language?: string;
  languages: string[];
  countries: string[];
  studios: string[];
  vote_count?: number;
  rating_source?: string;
  budget?: number;
  revenue?: number;
  season_count?: number;
  episode_count?: number;
  format?: string;
  source_material?: string;
  season_label?: string;
  awards?: string;
  homepage?: string;
  cast: Person[];
  crew: Person[];
  keywords: string[];
  recommendations: TitleSummary[];
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
  extra: TitleExtra;
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

export type Target =
  | { type: "movie" }
  | { type: "episode"; season: number; episode: number }
  | { type: "season"; season: number };

export type AudioMode = "sub" | "dub";

export interface ReleaseTags {
  resolution?: number;
  codec?: "h264" | "h265" | "av1" | "vp9";
  bit_depth?: number;
  hdr?: "sdr" | "hdr10" | "hdr_plus" | "dolby_vision";
  source?: string;
  audio?: string[];
  languages?: string[];
  multi_audio: boolean;
  has_subs: boolean;
  french?: "VF" | "VOSTFR" | "VFQ";
  size_bytes?: number;
  is_pack: boolean;
}

export interface Track {
  lang?: string;
  label: string;
  url?: string;
  default: boolean;
}

export interface Source {
  provider: ProviderId;
  host: string;
  label: string;
  url: string;
  format: string;
  headers?: Record<string, string>;
  tags: ReleaseTags;
  filename?: string;
  size_bytes?: number;
  subtitles?: Track[];
  audio?: Track[];
  expires_at?: number;
  torrent?: { magnet: string; info_hash?: string; seeders?: number; leechers?: number; file_index?: number };
  is_pack: boolean;
  verified?: boolean;
  score: number;
}

export interface SourcesReport {
  sources: Source[];
  errors: { provider: ProviderId; message: string }[];
}

export interface OpenedStream {
  id: string;
  url: string;
  torrent: boolean;
  subtitles: { label: string; lang?: string | null; url: string }[];
}

export interface TorrentProgress {
  downloaded: number;
  total: number;
  uploaded: number;
  finished: boolean;
}

/** Engine preferences (universeapi::config::Preferences). */
export interface Preferences {
  preferred_resolution: number;
  max_auto_resolution: number;
  preferred_codec: "h264" | "h265" | "av1" | "vp9";
  audio_languages: string[];
  subtitle_languages: string[];
  show_adult: boolean;
  include_torrents: boolean;
  include_french: boolean;
  provider_order: Partial<Record<MediaKind, ProviderId[]>>;
  disabled_providers: ProviderId[];
  verify_sources: boolean;
  max_concurrent_downloads: number;
  seed_after_download: boolean;
  region: string | null;
}

export type DownloadState =
  | { state: "queued" }
  | { state: "running" }
  | { state: "paused" }
  | { state: "extracting" }
  | { state: "completed"; path: string }
  | { state: "failed"; error: string }
  | { state: "cancelled" };

export interface DownloadStatus {
  id: number;
  title: string;
  state: DownloadState;
  downloaded: number;
  total: number | null;
  bytes_per_sec: number;
}

export interface RuntimeProgress {
  label: string;
  received: number;
  total: number;
}
