// Discord Rich Presence: the settings, and what each part of the app reports.
//
// Pages call `reportPlayer`, `reportMatch` and `clearDiscord`; Music and
// browsing are watched from here. Rust decides what wins and when to send, so
// reporting often is cheap: it drops anything that didn't really change.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { untrack } from "svelte";
import { musicPlayer } from "./music/player.svelte";
import { art } from "./music/ui.svelte";
import { nav, type Route } from "./state.svelte";
import type { AudioMode, SportsMatch, Target, TitleDetail, TitleSummary, TvChannel } from "./types";

export interface DiscordConfig {
  enabled: boolean;
  movies: boolean;
  series: boolean;
  anime: boolean;
  music: boolean;
  sports: boolean;
  liveTv: boolean;
  browsing: boolean;
  showDetails: boolean;
}

export type DiscordState = "unavailable" | "off" | "waiting" | "connected";

type Source = "player" | "music" | "match" | "browse";

const KEY = "velmora.discord";
const DEFAULTS: DiscordConfig = {
  enabled: false,
  movies: true,
  series: true,
  anime: true,
  music: true,
  sports: true,
  liveTv: true,
  browsing: false,
  showDetails: true,
};

function read(): DiscordConfig {
  try {
    return { ...DEFAULTS, ...JSON.parse(localStorage.getItem(KEY) ?? "{}") };
  } catch {
    return { ...DEFAULTS };
  }
}

function send(source: Source, presence: Record<string, unknown> | null) {
  invoke("discord_set", { source, presence }).catch(() => {});
}

/** When Velmora opened, for the browsing timer. */
const openedAt = Math.floor(Date.now() / 1000);

class Discord {
  config = $state<DiscordConfig>(read());
  state = $state<DiscordState>("off");
  #started = false;
  /** Changes every 5 s of music, so the bar is re-checked without spamming. */
  #musicTick = $derived(Math.floor(musicPlayer.position / 5));

  init() {
    if (this.#started) return;
    this.#started = true;
    listen<{ state: DiscordState }>("discord-status", (e) => (this.state = e.payload.state));
    invoke<{ state: DiscordState }>("discord_status")
      .then((s) => (this.state = s.state))
      .catch(() => {});
    this.#push();

    $effect.root(() => {
      // Music.
      $effect(() => {
        const cur = musicPlayer.current;
        void this.#musicTick;
        void musicPlayer.playing;
        void musicPlayer.duration;
        void musicPlayer.videoShown;
        untrack(() => {
          if (!cur) return send("music", null);
          const artist = cur.artists.map((a) => a.name).join(", ") || cur.subtitle || "YouTube Music";
          send("music", {
            kind: "music",
            title: cur.title,
            artist,
            album: cur.album,
            art: art(cur.thumbnails, 544),
            position: musicPlayer.position,
            duration: musicPlayer.duration || cur.duration || 0,
            paused: !musicPlayer.playing,
            video: musicPlayer.videoShown,
            link: cur.localPath ? null : `https://music.youtube.com/watch?v=${cur.id}`,
          });
        });
      });
      // Browsing (only shown when it's switched on and nothing else is playing).
      $effect(() => {
        const line = browseLine(nav.route);
        untrack(() => send("browse", line ? { kind: "browsing", line, start: openedAt } : null));
      });
    });
  }

  set(patch: Partial<DiscordConfig>) {
    this.config = { ...this.config, ...patch };
    localStorage.setItem(KEY, JSON.stringify(this.config));
    this.#push();
  }

  #push() {
    invoke("discord_configure", { config: $state.snapshot(this.config) }).catch(() => {});
  }
}

export const discord = new Discord();

export function clearDiscord(source: Source) {
  send(source, null);
}

function browseLine(r: Route): string | null {
  if (r.name === "player") return null;
  if (r.name.startsWith("music")) return "Exploring music";
  switch (r.name) {
    case "browse":
      return r.kind === "movie" ? "Browsing movies" : r.kind === "series" ? "Browsing series" : "Browsing anime";
    case "search":
      return "Searching for something to watch";
    case "sports":
    case "match":
      return "Checking the scores";
    case "live":
      return "Browsing live TV";
    case "downloads":
      return "Looking at downloads";
    case "mylist":
      return "Looking at My List";
    default:
      return "Browsing";
  }
}

function clock(s: number): string {
  const t = Math.max(0, Math.floor(s));
  const h = Math.floor(t / 3600);
  const m = Math.floor((t % 3600) / 60);
  const sec = String(t % 60).padStart(2, "0");
  return h ? `${h}:${String(m).padStart(2, "0")}:${sec}` : `${m}:${sec}`;
}

function regionName(code?: string): string | null {
  if (!code) return null;
  try {
    return new Intl.DisplayNames(["en"], { type: "region" }).of(code.toUpperCase()) ?? code;
  } catch {
    return code;
  }
}

export interface PlayerReport {
  title: TitleSummary;
  target: Target;
  detail?: TitleDetail;
  channel?: TvChannel;
  mode: AudioMode;
  time: number;
  duration: number;
  paused: boolean;
  /** When the player opened, Unix seconds; the clock for live TV. */
  started: number;
}

/** What the video player is showing. 18+ titles are never sent. */
export function reportPlayer(p: PlayerReport) {
  if (p.channel) {
    const where = regionName(p.channel.country);
    send("player", {
      kind: "liveTv",
      channel: p.channel.name,
      line2: ["Live TV", where, p.channel.categories[0]].filter(Boolean).join(" · "),
      art: p.channel.logo ?? null,
      start: p.started,
    });
    return;
  }
  const t = p.title;
  if (t.badges?.includes("18+")) return send("player", null);

  let line2: string;
  let still: string | undefined;
  if (p.target.type === "episode") {
    const { season, episode } = p.target;
    const ep = p.detail?.seasons.find((s) => s.number === season)?.episodes.find((e) => e.number === episode);
    still = ep?.thumbnail;
    const where = t.kind === "anime" && season <= 1 ? `Episode ${episode}` : `S${season} · E${episode}`;
    const parts = [where, ep?.title];
    if (t.kind === "anime") parts.push(p.mode === "dub" ? "Dub" : "Sub");
    line2 = parts.filter(Boolean).join(" · ");
    if (p.paused) line2 = `Paused · ${where}`;
  } else {
    line2 = p.paused
      ? `Paused · ${clock(p.time)} / ${clock(p.duration)}`
      : ["Movie", t.year].filter(Boolean).join(" · ");
  }

  const link = t.ids.tmdb
    ? `https://www.themoviedb.org/${t.kind === "movie" ? "movie" : "tv"}/${t.ids.tmdb}`
    : t.ids.anilist
      ? `https://anilist.co/anime/${t.ids.anilist}`
      : null;

  send("player", {
    kind: "video",
    title: t.name,
    media: t.kind,
    line2,
    art: still ?? t.poster ?? null,
    artText: [t.name, t.year, t.rating ? `★ ${t.rating.toFixed(1)}` : null].filter(Boolean).join(" · "),
    position: p.time,
    duration: p.duration,
    paused: p.paused,
    link,
  });
}

/** A live match page. Anything not live clears it. */
export function reportMatch(m: SportsMatch) {
  if (m.status !== "live") return send("match", null);
  const score = m.home_score != null && m.away_score != null ? `${m.home_score} – ${m.away_score}` : "Live";
  send("match", {
    kind: "match",
    title: m.title,
    line2: [score, m.phase].filter(Boolean).join(" · "),
    league: m.league.name,
    art: m.home?.badge ?? m.league.badge ?? null,
    start: m.kickoff ?? null,
  });
}
