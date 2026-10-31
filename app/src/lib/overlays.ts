// App-wide dialogs live once in App.svelte; pages open them through here.
import type { Target, TitleDetail, TitleSummary } from "./types";

type Overlays = {
  torrentNotice: () => Promise<boolean>;
  download: (title: TitleSummary, target: Target, detail?: TitleDetail) => void;
};

const handlers: Partial<Overlays> = {};

export function registerOverlays(h: Overlays) {
  Object.assign(handlers, h);
}

const NOTICE_KEY = "velmora.torrentNoticeAccepted";

/** Show the one-time torrent/VPN notice; resolves false if the user backs out. */
export async function torrentConsent(): Promise<boolean> {
  if (localStorage.getItem(NOTICE_KEY) === "1") return true;
  const ok = (await handlers.torrentNotice?.()) ?? false;
  if (ok) localStorage.setItem(NOTICE_KEY, "1");
  return ok;
}

export function openDownload(title: TitleSummary, target: Target, detail?: TitleDetail) {
  handlers.download?.(title, target, detail);
}
