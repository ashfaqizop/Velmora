// Thin typed wrappers over the Tauri commands in src-tauri/src/commands.rs.
import { Channel, invoke } from "@tauri-apps/api/core";
import type {
  AniKeyStatus,
  AniModel,
  AppInfo,
  AudioMode,
  ConfigStatus,
  DownloadStatus,
  LicenseStatus,
  MusicAccount,
  MusicAlbum,
  MusicArtist,
  MusicFeed,
  MusicItem,
  MusicLibraryKind,
  MusicLyrics,
  MusicPlaylist,
  MusicRating,
  MusicSearchFilter,
  MusicSearchResults,
  MusicStatus,
  MusicStream,
  MusicTrackPage,
  YtDlpStatus,
  LocalFile,
  OpenedStream,
  Preferences,
  ProviderRef,
  Rail,
  Facet,
  Narrowed,
  SearchBatch,
  SportsDay,
  SportsMatch,
  StreamHealth,
  TvChannel,
  TvPage,
  TvQuery,
  WatchOption,
  Source,
  SourcesReport,
  Target,
  TitleDetail,
  TitleSummary,
  TorrentProgress,
  RuntimeProgress,
} from "./types";

export const appInfo = () => invoke<AppInfo>("app_info");

export const refreshConfig = () => invoke<ConfigStatus>("refresh_config");

export const home = () => invoke<Rail[]>("home");

/** The engine omits empty lists, so fill them in before the UI sees the detail. */
export async function detail(origin: ProviderRef): Promise<TitleDetail> {
  const d = await invoke<Partial<TitleDetail> & Pick<TitleDetail, "summary">>("detail", { origin });
  return {
    ...d,
    genres: d.genres ?? [],
    cast: d.cast ?? [],
    directors: d.directors ?? [],
    screenshots: d.screenshots ?? [],
    seasons: (d.seasons ?? []).map((s) => ({ ...s, episodes: s.episodes ?? [] })),
    extra: {
      ...d.extra,
      languages: d.extra?.languages ?? [],
      countries: d.extra?.countries ?? [],
      studios: d.extra?.studios ?? [],
      cast: d.extra?.cast ?? [],
      crew: d.extra?.crew ?? [],
      keywords: d.extra?.keywords ?? [],
      recommendations: d.extra?.recommendations ?? [],
    },
  };
}

/** Calls `onBatch` once per provider as results arrive; resolves when all are in. */
export function searchStream(query: string, onBatch: (b: SearchBatch) => void): Promise<void> {
  const channel = new Channel<SearchBatch>();
  channel.onmessage = onBatch;
  return invoke("search_stream", { query, onBatch: channel });
}

/** The exact matches for a query among its results, and the full list ranked. */
export const searchNarrow = (query: string, items: TitleSummary[]) => invoke<Narrowed>("search_narrow", { query, items });

// ── Sports and live TV ──

export const sportsDay = (date?: string) => invoke<SportsDay>("sports_day", { date: date ?? null });
export const sportsLive = () => invoke<SportsMatch[]>("sports_live");
export const sportsUpcoming = (days: number) => invoke<SportsMatch[]>("sports_upcoming", { days });
export const sportsResults = (days: number) => invoke<SportsMatch[]>("sports_results", { days });
export const sportsLookup = (id: string) => invoke<SportsMatch>("sports_lookup", { id });
export const sportsWatch = (item: SportsMatch, country: string) => invoke<WatchOption[]>("sports_watch", { item, country });

export const tvChannels = (query: TvQuery) => invoke<TvPage>("tv_channels", { query });
export const tvChannel = (id: string) => invoke<TvChannel | null>("tv_channel", { id });
export const tvCountries = () => invoke<Facet[]>("tv_countries");
export const tvCategories = () => invoke<Facet[]>("tv_categories");
export const tvCheck = (id: string) => invoke<StreamHealth[]>("tv_check", { id });
export const tvSources = (id: string) => invoke<Source[]>("tv_sources", { id });

export const sources = (title: TitleSummary, target: Target, mode: AudioMode) =>
  invoke<SourcesReport>("sources", { title, target, mode });

/** Hand a source to the loopback stream server; returns the URL for mpv. */
export const openStream = (source: Source, episode: [number, number] | null) =>
  invoke<OpenedStream>("open_stream", { source, episode });

export const closeStream = (id: string) => invoke<void>("close_stream", { id });

export const torrentStatus = (id: string) => invoke<TorrentProgress | null>("torrent_status", { id });

/** True once the mpv and ffmpeg components are on disk. */
export const runtimeReady = () => invoke<boolean>("runtime_ready");

export function installRuntime(onProgress: (p: RuntimeProgress) => void): Promise<void> {
  const channel = new Channel<RuntimeProgress>();
  channel.onmessage = onProgress;
  return invoke("install_runtime", { onProgress: channel });
}

export const getPreferences = () => invoke<Preferences>("get_preferences");

export const setPreferences = (preferences: Preferences) => invoke<void>("set_preferences", { preferences });

export const downloadStart = (source: Source, title: string, episode: [number, number] | null) =>
  invoke<number>("download_start", { source, title, episode });

export const downloadsList = () => invoke<DownloadStatus[]>("downloads_list");
export const downloadPause = (id: number) => invoke<void>("download_pause", { id });
export const downloadResume = (id: number) => invoke<void>("download_resume", { id });
export const downloadCancel = (id: number) => invoke<void>("download_cancel", { id });
export const downloadsClear = () => invoke<void>("downloads_clear");
export const revealFile = (path: string) => invoke<void>("reveal_file", { path });
export const downloadsLibrary = () => invoke<LocalFile[]>("downloads_library");
export const downloadExisting = (title: string) => invoke<string[]>("download_existing", { title });
export const downloadFolder = () => invoke<string>("download_folder");
export const openDownloadFolder = () => invoke<void>("open_download_folder");
export const setDownloadDir = (path: string | null) => invoke<string>("set_download_dir", { path });
export const defaultDownloadFolder = () => invoke<string>("default_download_folder");
/** What the player loads for a downloaded file, plus subtitle files beside it. */
export const openLocal = (path: string) => invoke<{ url: string; subtitles: string[] }>("open_local", { path });

export interface SyncStatus {
  anilistAvailable: boolean;
  anilistUser: string | null;
  traktAvailable: boolean;
  traktUser: string | null;
}

export const syncStatus = () => invoke<SyncStatus>("sync_status");
export const syncDisconnect = (service: "anilist" | "trakt") => invoke<void>("sync_disconnect", { service });
export const anilistLogin = () => invoke<void>("anilist_login");
export const traktLogin = () => invoke<void>("trakt_login");
export const syncWatched = (title: TitleSummary, target: Target) => invoke<string[]>("sync_watched", { title, target });

/** Open an https link (IMDb, TMDB, a title's homepage...) in the default browser. */
export const openExternal = (url: string) => invoke<void>("open_external", { url });

// ── Ani ──

export const aniKeyStatus = () => invoke<AniKeyStatus>("ani_key_status");
/** Checks the key with Gemini before saving it; `null` removes the saved key. */
export const aniSetKey = (key: string | null) => invoke<AniKeyStatus>("ani_set_key", { key });
export const aniModels = () => invoke<AniModel[]>("ani_models");

/** Stream one Gemini call; `onEvent` gets each raw response chunk. */
export function aniGenerate(id: string, model: string, body: unknown, onEvent: (chunk: unknown) => void): Promise<void> {
  const channel = new Channel<unknown>();
  channel.onmessage = onEvent;
  return invoke("ani_generate", { id, model, body, onEvent: channel });
}
export const aniStop = (id: string) => invoke<void>("ani_stop", { id });

export const tmdbRaw = (path: string, query: Record<string, string> = {}) => invoke<unknown>("tmdb_raw", { path, query });
export const tmdbList = (path: string, query: Record<string, string> = {}, field?: string, kind?: "movie" | "tv") =>
  invoke<TitleSummary[]>("tmdb_list", { path, query, field, kind });
export const anilistFind = (filters: Record<string, unknown>) => invoke<TitleSummary[]>("anilist_find", { filters });
export const anilistRaw = (query: string, variables: Record<string, unknown> = {}) =>
  invoke<unknown>("anilist_raw", { query, variables });

export const licenseStatus = () => invoke<LicenseStatus>("license_status");
/** Checks the key locally, then activates it on this PC. Rejects with a readable message. */
export const licenseActivate = (key: string) => invoke<LicenseStatus>("license_activate", { key });
/** Renews the activation if it's due; quiet when offline. */
export const licenseRefresh = () => invoke<LicenseStatus>("license_refresh");
export const licenseRemove = () => invoke<LicenseStatus>("license_remove");

/** The ad rules from the site, or null when it can't be reached. */
export const adsConfig = () => invoke<unknown>("ads_config");
export const adsReport = (counts: Record<string, number>) => invoke<void>("ads_report", { counts });

// ── Music ──────────────────────────────────────────────────────────────

export const musicStatus = () => invoke<MusicStatus>("music_status");
export const musicAccount = () => invoke<MusicAccount>("music_account");
/** Opens Google's sign-in window; the outcome arrives as a `music-status` event. */
export const musicSignIn = () => invoke<void>("music_sign_in");
export const musicSignOut = () => invoke<void>("music_sign_out");
export const musicHome = (next?: string | null) => invoke<MusicFeed>("music_home", { next: next ?? null });
export const musicFeed = (browseId: string, params?: string | null, next?: string | null) =>
  invoke<MusicFeed>("music_feed", { browseId, params: params ?? null, next: next ?? null });
export const musicSearch = (query: string, filter: MusicSearchFilter = "all") =>
  invoke<MusicSearchResults>("music_search", { query, filter });
export const musicAlbum = (id: string) => invoke<MusicAlbum>("music_album", { id });
export const musicArtist = (id: string) => invoke<MusicArtist>("music_artist", { id });
export const musicPlaylist = (id: string) => invoke<MusicPlaylist>("music_playlist", { id });
export const musicPlaylistMore = (token: string) => invoke<MusicTrackPage>("music_playlist_more", { token });
export const musicQueue = (videoId?: string | null, playlistId?: string | null, params?: string | null) =>
  invoke<MusicTrackPage>("music_queue", { videoId: videoId ?? null, playlistId: playlistId ?? null, params: params ?? null });
export const musicRadio = (videoId: string) => invoke<MusicTrackPage>("music_radio", { videoId });
export const musicQueueMore = (token: string) => invoke<MusicTrackPage>("music_queue_more", { token });
export const musicLyrics = (videoId: string, title: string, artist: string, duration?: number | null) =>
  invoke<MusicLyrics | null>("music_lyrics", { videoId, title, artist, duration: duration ?? null });
export const musicLiked = (limit?: number) => invoke<MusicPlaylist>("music_liked", { limit: limit ?? null });
export const musicLibrary = (kind: MusicLibraryKind) => invoke<MusicItem[]>("music_library", { kind });
export const musicOwnPlaylists = () => invoke<MusicItem[]>("music_own_playlists");
export const musicRate = (videoId: string, rating: MusicRating) => invoke<void>("music_rate", { videoId, rating });
export const musicPlaylistAdd = (playlistId: string, videoIds: string[]) =>
  invoke<void>("music_playlist_add", { playlistId, videoIds });
export const musicPlaylistRemove = (playlistId: string, videoId: string, setVideoId: string) =>
  invoke<void>("music_playlist_remove", { playlistId, videoId, setVideoId });
export const musicPlaylistCreate = (title: string, videoIds: string[]) =>
  invoke<string>("music_playlist_create", { title, videoIds });
export const musicSubscribe = (channelId: string, on: boolean) => invoke<void>("music_subscribe", { channelId, on });
/** Stream URLs, resolved by yt-dlp without the user's cookies. */
export const musicStream = (videoId: string, video = false, maxHeight?: number) =>
  invoke<MusicStream>("music_stream", { videoId, video, maxHeight: maxHeight ?? null });
export const musicYtdlpStatus = () => invoke<YtDlpStatus>("music_ytdlp_status");
export const musicYtdlpUpdate = () => invoke<YtDlpStatus>("music_ytdlp_update");
