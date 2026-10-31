// Thin typed wrappers over the Tauri commands in src-tauri/src/commands.rs.
import { Channel, invoke } from "@tauri-apps/api/core";
import type {
  AppInfo,
  AudioMode,
  ConfigStatus,
  DownloadStatus,
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

export const detail = (origin: ProviderRef) => invoke<TitleDetail>("detail", { origin });

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
