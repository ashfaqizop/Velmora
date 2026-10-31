// Thin typed wrappers over the Tauri commands in src-tauri/src/commands.rs.
import { Channel, invoke } from "@tauri-apps/api/core";
import type {
  AppInfo,
  AudioMode,
  ConfigStatus,
  DownloadStatus,
  LocalFile,
  OpenedStream,
  Preferences,
  ProviderRef,
  Rail,
  SearchBatch,
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
