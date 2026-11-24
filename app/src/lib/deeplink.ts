// velmora:// links. The website's "Open in Velmora" buttons use them:
//   velmora://title/movie/603   a title page (TMDB movie or tv id)
//   velmora://music/dQw4w9WgXcQ  play a song (YouTube video id) with its radio
//   velmora://profile/username   someone's Velmora Social profile (opens the browser)
//   velmora://theatre/<id>       a Theatre screening (lobby or live)
// A link that arrives while Velmora is open comes through the single-instance
// plugin; one that started Velmora is read once at startup.
import { getCurrent, onOpenUrl } from "@tauri-apps/plugin-deep-link";
import { M3eSnackbar } from "@m3e/web/snackbar";
import * as api from "./api";
import { nav } from "./state.svelte";
import { musicPlayer } from "./music/player.svelte";

const SOCIAL = "https://social.getvelmora.dpdns.org";

export async function startDeepLinks() {
  try {
    const first = await getCurrent();
    if (first?.length) await open(first[0]);
  } catch {
    // No link started this launch.
  }
  onOpenUrl((urls) => {
    if (urls[0]) open(urls[0]);
  }).catch(() => {});
}

/** Follow one link. Anything unexpected is ignored rather than guessed at. */
export async function open(raw: string) {
  let url: URL;
  try {
    url = new URL(raw);
  } catch {
    return;
  }
  if (url.protocol !== "velmora:") return;
  const parts = [url.hostname, ...url.pathname.split("/")].filter(Boolean).map(decodeURIComponent);
  const [what, a, b] = parts;

  if (what === "title" && (a === "movie" || a === "tv") && /^\d{1,9}$/.test(b ?? "")) {
    try {
      const [title] = await api.tmdbTitles([`${a}/${b}`]);
      if (title) nav.go({ name: "title", title });
      else M3eSnackbar.open("Velmora couldn't find that title.");
    } catch {
      M3eSnackbar.open("Velmora couldn't open that title. Check your connection.");
    }
  } else if (what === "music" && /^[A-Za-z0-9_-]{11}$/.test(a ?? "")) {
    nav.go({ name: "music" });
    musicPlayer.playTarget({ videoId: a, playlistId: null, params: null }, "link").catch(() => M3eSnackbar.open("That song couldn't start."));
  } else if (what === "theatre" && /^[0-9a-f-]{36}$/.test(a ?? "")) {
    // A screening from Velmora Social: the screening page says whether it's open yet.
    nav.go({ name: "screening", id: a });
  } else if (what === "profile" && /^[A-Za-z0-9_]{3,20}$/.test(a ?? "")) {
    api.openExternal(`${SOCIAL}/@${a}`).catch(() => {});
  }
}
