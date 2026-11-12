// Music downloads: the chosen format, the songs on their way, and a note
// when a batch has landed. MP3, M4A and Opus go through musicdl.rs as
// ordinary tagged files; "Velmora only" keeps the old protected download.
import { listen } from "@tauri-apps/api/event";
import { SvelteMap } from "svelte/reactivity";
import { M3eSnackbar } from "@m3e/web/snackbar";
import * as api from "../api";
import { downloads } from "../state.svelte";
import type { MusicItem } from "../types";

export type DownloadFormat = api.ExportFormat | "vault";
const FORMAT_KEY = "velmora.music.downloadFormat";

function readFormat(): DownloadFormat {
  try {
    const v = localStorage.getItem(FORMAT_KEY);
    return v === "mp3" || v === "opus" || v === "vault" ? v : "m4a";
  } catch {
    return "m4a";
  }
}

const finished = (j: api.ExportJob) => j.stage === "done" || j.stage === "failed";

class MusicExports {
  jobs = new SvelteMap<string, api.ExportJob>();
  format = $state<DownloadFormat>(readFormat());
  /** Goes up each time a song lands, so a library list can reload. */
  landed = $state(0);
  private started = false;
  private batch = { done: 0, failed: 0, error: "" };

  init() {
    if (this.started) return;
    this.started = true;
    listen<api.ExportJob>("music-export", (e) => this.onJob(e.payload));
    api
      .musicExports()
      .then((list) => list.forEach((j) => this.jobs.set(j.videoId, j)))
      .catch(() => {});
  }

  get active(): api.ExportJob[] {
    return [...this.jobs.values()].filter((j) => !finished(j));
  }

  setFormat(f: DownloadFormat) {
    this.format = f;
    try {
      localStorage.setItem(FORMAT_KEY, f);
    } catch {
      // Not remembered.
    }
  }

  /** Download songs in the chosen format. An album passes its name, cover
   *  and track numbers. Returns how many started. */
  async download(items: MusicItem[], album?: { title: string; thumbnails: MusicItem["thumbnails"] }): Promise<number> {
    const list = items
      .map((t, i) => ({ t, n: i + 1 }))
      .filter(({ t }) => (t.kind === "song" || t.kind === "video") && !t.localPath);
    if (!list.length) return 0;
    const songs = list.map(({ t }) => {
      const s = $state.snapshot(t) as MusicItem;
      // Album rows often come without a picture; the album's is the right one.
      if (album && !s.thumbnails.length) s.thumbnails = $state.snapshot(album.thumbnails) as MusicItem["thumbnails"];
      return s;
    });
    if (this.format === "vault") {
      for (const s of songs) await api.musicDownload(s).catch(() => {});
      downloads.refresh();
      return songs.length;
    }
    return api.musicExport(songs, this.format, album?.title ?? null, album ? list.map(({ n }) => n) : null);
  }

  private onJob(j: api.ExportJob) {
    const before = this.jobs.get(j.videoId)?.stage;
    this.jobs.set(j.videoId, j);
    if (j.stage === "done" && before !== "done") {
      this.landed++;
      this.batch.done++;
    } else if (j.stage === "failed" && before !== "failed") {
      this.batch.failed++;
      this.batch.error = j.error ?? "";
    }
    if (!this.active.length && (this.batch.done || this.batch.failed)) {
      const { done, failed, error } = this.batch;
      const ok = done ? `Downloaded ${done} song${done === 1 ? "" : "s"}` : "";
      const bad = failed ? `${failed} didn't${error ? ` (${error})` : ""}` : "";
      M3eSnackbar.open([ok, bad].filter(Boolean).join(". ") + ".");
      this.batch = { done: 0, failed: 0, error: "" };
    }
  }

  /** Clear finished rows from the list. */
  clearFinished() {
    for (const [id, j] of this.jobs) if (finished(j)) this.jobs.delete(id);
    api.musicExportsClear().catch(() => {});
  }
}

export const musicExports = new MusicExports();
