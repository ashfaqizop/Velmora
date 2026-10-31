// Thin typed wrappers over the Tauri commands in src-tauri/src/commands.rs.
import { Channel, invoke } from "@tauri-apps/api/core";
import type {
  AppInfo,
  AudioMode,
  ConfigStatus,
  OpenedStream,
  ProviderRef,
  Rail,
  SearchBatch,
  Source,
  SourcesReport,
  Target,
  TitleDetail,
  TitleSummary,
  TorrentProgress,
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
