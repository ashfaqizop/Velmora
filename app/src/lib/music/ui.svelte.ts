// Shared bits for the Music pages: artwork, opening and playing items, the
// song menu, likes and playlists, and the player's on-screen state.
import { M3eSnackbar } from "@m3e/web/snackbar";
import * as api from "../api";
import { nav } from "../state.svelte";
import type { MusicItem, MusicThumb } from "../types";
import type { ArtColors } from "./color";
import { musicPlayer } from "./player.svelte";

/** The best artwork URL for a size. Google's image URLs can be asked for any
 *  size, so a small thumbnail is upgraded instead of stretched. */
export function art(thumbs: MusicThumb[] | null | undefined, size = 226): string | null {
  if (!thumbs?.length) return null;
  const sorted = [...thumbs].sort((a, b) => a.width - b.width);
  const fit = sorted.find((t) => t.width >= size) ?? sorted[sorted.length - 1];
  const url = fit.url;
  // lh3.googleusercontent.com/...=w120-h120-l90-rj → any size we like.
  if (/googleusercontent\.com|ggpht\.com/.test(url)) {
    return url.replace(/=w\d+-h\d+[^&?#]*/, `=w${size * 2}-h${size * 2}-l90-rj`).replace(/=s\d+[^&?#]*/, `=s${size * 2}`);
  }
  return url;
}

export const artists = (i: MusicItem) =>
  i.artists.length ? i.artists.map((a) => a.name).join(", ") : (i.subtitle ?? "");

export function clock(seconds: number | null | undefined): string {
  if (!seconds || !Number.isFinite(seconds)) return "0:00";
  const s = Math.floor(seconds);
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const r = String(s % 60).padStart(2, "0");
  return h ? `${h}:${String(m).padStart(2, "0")}:${r}` : `${m}:${r}`;
}

export const playable = (i: MusicItem) => i.kind === "song" || i.kind === "video";

/** Open a card: pages for albums, artists and playlists; songs play. */
export function openItem(i: MusicItem, list?: MusicItem[], source?: string) {
  switch (i.kind) {
    case "album":
      return nav.go({ name: "music-album", id: i.id });
    case "artist":
      return nav.go({ name: "music-artist", id: i.id });
    case "playlist":
      return nav.go({ name: "music-playlist", id: i.id });
    case "category":
      return nav.go({ name: "music-feed", browseId: i.id, params: i.params, title: i.title });
    default: {
      const songs = list?.filter(playable);
      if (songs && songs.length > 1) return musicPlayer.playList(songs, songs.indexOf(i), source ?? null);
      return musicPlayer.playSong(i, source ?? null);
    }
  }
}

/** The play button on an album or playlist card: play it without opening it. */
export async function playCollection(i: MusicItem) {
  try {
    if (i.kind === "album") {
      const a = await api.musicAlbum(i.id);
      return musicPlayer.playList(a.tracks, 0, a.title);
    }
    if (i.kind === "playlist") {
      if (i.playVideoId) return musicPlayer.playTarget({ videoId: i.playVideoId, playlistId: null, params: null }, i.title);
      const p = await api.musicPlaylist(i.id);
      return musicPlayer.playList(p.tracks, 0, p.title);
    }
    if (i.kind === "artist") {
      const a = await api.musicArtist(i.id);
      const target = a.shuffle ?? a.radio;
      if (target) return musicPlayer.playTarget(target, a.name);
    }
    return openItem(i);
  } catch (e) {
    M3eSnackbar.open(`Couldn't play ${i.title}.`);
  }
}

export const shareLink = (i: MusicItem) => {
  switch (i.kind) {
    case "album":
    case "playlist":
      return i.id.startsWith("MPRE")
        ? `https://music.youtube.com/browse/${i.id}`
        : `https://music.youtube.com/playlist?list=${i.id.replace(/^VL/, "")}`;
    case "artist":
      return `https://music.youtube.com/channel/${i.id}`;
    default:
      return `https://music.youtube.com/watch?v=${i.id}`;
  }
};

// ── Likes ──────────────────────────────────────────────────────────

class Likes {
  /** Songs liked or disliked in this session, over what pages say. */
  private local = $state<Record<string, "like" | "dislike" | "none">>({});

  liked(id: string): boolean {
    return this.local[id] === "like";
  }

  /** Seed from the Liked songs list so hearts show filled everywhere. */
  seed(ids: string[]) {
    const next = { ...this.local };
    for (const id of ids) if (!next[id]) next[id] = "like";
    this.local = next;
  }

  async toggle(i: MusicItem) {
    const on = !this.liked(i.id);
    this.local = { ...this.local, [i.id]: on ? "like" : "none" };
    try {
      await api.musicRate(i.id, on ? "like" : "none");
      M3eSnackbar.open(on ? "Added to Liked songs" : "Removed from Liked songs");
    } catch (e) {
      this.local = { ...this.local, [i.id]: on ? "none" : "like" };
      M3eSnackbar.open(String(e));
    }
  }
}

export const likes = new Likes();

// ── Song menu and Add to playlist ──────────────────────────────────

export interface MenuContext {
  /** The playlist the row is in, when it can be removed from it. */
  playlistId?: string;
  onRemoved?: () => void;
}

class MusicUi {
  menu = $state<{ item: MusicItem; x: number; y: number; ctx: MenuContext } | null>(null);
  addTarget = $state<MusicItem[] | null>(null);
  /** The full player is open. */
  full = $state(false);
  /** Which side the full view shows: lyrics or the queue. */
  fullTab = $state<"lyrics" | "queue">("lyrics");
  /** The playing song's artwork colors, for the player's tint. */
  colors = $state<ArtColors | null>(null);
  /** The pill is shrunk while the page scrolls. */
  compact = $state(false);
  /** The user's playlists, for the sidebar and Add to playlist. */
  playlists = $state<MusicItem[]>([]);
  private compactTimer: ReturnType<typeof setTimeout> | undefined;

  openMenu(item: MusicItem, e: MouseEvent, ctx: MenuContext = {}) {
    e.preventDefault();
    e.stopPropagation();
    this.menu = { item, x: e.clientX, y: e.clientY, ctx };
  }

  closeMenu() {
    this.menu = null;
  }

  addToPlaylist(items: MusicItem[]) {
    this.addTarget = items.filter(playable);
  }

  /** The list has been asked for once (it can be empty: an account with no playlists). */
  playlistsLoaded = $state(false);

  async loadPlaylists() {
    this.playlistsLoaded = true;
    try {
      this.playlists = await api.musicOwnPlaylists();
    } catch {
      // Keeps the last list.
    }
  }

  /** The content scrolled; shrink the pill on the way down. */
  scrolled(delta: number) {
    if (delta > 4) this.compact = true;
    else if (delta < -4) this.compact = false;
    clearTimeout(this.compactTimer);
    this.compactTimer = setTimeout(() => (this.compact = false), 900);
  }
}

export const musicUi = new MusicUi();

/** A greeting for the time of day. */
export function greeting(): string {
  const h = new Date().getHours();
  if (h < 5) return "Still up?";
  if (h < 12) return "Good morning";
  if (h < 18) return "Good afternoon";
  return "Good evening";
}
