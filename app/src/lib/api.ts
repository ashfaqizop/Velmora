// Thin wrappers over the Tauri commands. Outside Tauri (plain `npm run dev`
// in a browser) they fall back to mock data so the UI can be worked on alone.
import { Channel, invoke } from "@tauri-apps/api/core";
import * as mock from "./mock";
import type { AppInfo, ConfigStatus, ProviderRef, Rail, SearchBatch, TitleDetail } from "./types";

export const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export function appInfo(): Promise<AppInfo> {
  return inTauri ? invoke<AppInfo>("app_info") : mock.appInfo();
}

export function refreshConfig(): Promise<ConfigStatus> {
  return inTauri ? invoke<ConfigStatus>("refresh_config") : mock.refreshConfig();
}

export function home(): Promise<Rail[]> {
  return inTauri ? invoke<Rail[]>("home") : mock.home();
}

export function detail(origin: ProviderRef): Promise<TitleDetail> {
  return inTauri ? invoke<TitleDetail>("detail", { origin }) : mock.detail(origin);
}

/** Calls `onBatch` once per provider as results arrive; resolves when all are in. */
export function searchStream(query: string, onBatch: (b: SearchBatch) => void): Promise<void> {
  if (!inTauri) return mock.searchStream(query, onBatch);
  const channel = new Channel<SearchBatch>();
  channel.onmessage = onBatch;
  return invoke("search_stream", { query, onBatch: channel });
}
