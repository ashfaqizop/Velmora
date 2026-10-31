// What the buttons on Ani's cards do. Ani never calls these; they run only
// when the user clicks, through the same paths the rest of the app uses.
import { M3eSnackbar } from "@m3e/web/snackbar";
import * as api from "../api";
import { openDownload, torrentConsent } from "../overlays";
import { isTorrent } from "../sources";
import { downloads, library, nav, settings } from "../state.svelte";
import type { Source, Target, TitleSummary } from "../types";
import { personalize } from "./personalize.svelte";

export const openTitle = (t: TitleSummary) => nav.go({ name: "title", title: t });

export function play(t: TitleSummary, target: Target = { type: "movie" }) {
  // Series without a chosen episode open on their page, where resume lives.
  if (t.kind !== "movie" && target.type === "movie") return openTitle(t);
  nav.go({ name: "player", title: t, target, mode: settings.audioMode });
}

export function toggleList(t: TitleSummary) {
  const had = library.has(t);
  library.toggle(t);
  M3eSnackbar.open(had ? `${t.name} removed from My List` : `${t.name} added to My List`);
}

export function pickDownload(t: TitleSummary, target: Target = { type: "movie" }) {
  if (t.kind !== "movie" && target.type === "movie") return openTitle(t);
  openDownload(t, target);
}

/** File name base the downloader uses: "Dune (2021)" or "Frieren S01E03". */
function fileLabel(t: TitleSummary, tg: Target) {
  if (tg.type === "episode") {
    const pad = (n: number) => String(n).padStart(2, "0");
    return `${t.name} S${pad(tg.season)}E${pad(tg.episode)}`;
  }
  if (tg.type === "season") return `${t.name} Season ${tg.season}`;
  return t.year ? `${t.name} (${t.year})` : t.name;
}

/** Start downloading one exact link from a link card. */
export async function downloadLink(source: Source, t: TitleSummary, target: Target) {
  if (isTorrent(source) && !(await torrentConsent())) return;
  try {
    const episode: [number, number] | null = target.type === "episode" ? [target.season, target.episode] : null;
    await api.downloadStart(source, fileLabel(t, target), episode);
    personalize.download(t, target);
    downloads.start();
    M3eSnackbar.open(`${t.name} added to Downloads`, "View", { actionCallback: () => nav.go({ name: "downloads" }) });
  } catch (e) {
    M3eSnackbar.open(String(e));
  }
}
