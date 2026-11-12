// Settings is split into pages. This lists them, and every setting with the
// words people might search for, so "Find a setting" can jump straight to it.

export type SettingsPage =
  | "playback"
  | "sound"
  | "sources"
  | "downloads"
  | "music"
  | "music-storage"
  | "ani"
  | "accounts"
  | "appearance"
  | "license"
  | "about";

export interface PageInfo {
  id: SettingsPage;
  label: string;
  icon: string;
}

export const SETTINGS_GROUPS: { label: string; pages: PageInfo[] }[] = [
  {
    label: "Watch",
    pages: [
      { id: "playback", label: "Playback", icon: "play_circle" },
      { id: "sound", label: "Sound", icon: "graphic_eq" },
      { id: "sources", label: "Sources", icon: "hub" },
      { id: "downloads", label: "Downloads", icon: "download" },
    ],
  },
  {
    label: "Music",
    pages: [
      { id: "music", label: "Music", icon: "music_note" },
      { id: "music-storage", label: "Storage and history", icon: "database" },
    ],
  },
  {
    label: "Velmora",
    pages: [
      { id: "ani", label: "Ani", icon: "auto_awesome" },
      { id: "accounts", label: "Accounts", icon: "account_circle" },
      { id: "appearance", label: "Appearance", icon: "palette" },
      { id: "license", label: "License", icon: "workspace_premium" },
      { id: "about", label: "About", icon: "info" },
    ],
  },
];

export const SETTINGS_PAGES: PageInfo[] = SETTINGS_GROUPS.flatMap((g) => g.pages);

export const isSettingsPage = (v: unknown): v is SettingsPage => SETTINGS_PAGES.some((p) => p.id === v);

export interface SettingEntry {
  /** Matches `data-setting` on the row, for the jump and highlight. */
  id: string;
  page: SettingsPage;
  label: string;
  hint: string;
  /** Other words people might type. */
  words: string;
}

export const SETTINGS_INDEX: SettingEntry[] = [
  { id: "quality", page: "playback", label: "Preferred quality", hint: "720p, 1080p or 4K first", words: "resolution hd uhd" },
  { id: "auto4k", page: "playback", label: "Pick 4K automatically", hint: "Use 4K without asking", words: "uhd data" },
  { id: "codec", page: "playback", label: "Video codec", hint: "H.264, HEVC or AV1", words: "h265 gpu decode" },
  { id: "audio-lang", page: "playback", label: "Audio language", hint: "The language Velmora looks for first", words: "dub language" },
  { id: "sub-lang", page: "playback", label: "Subtitle language", hint: "Subtitles Velmora picks first", words: "subtitles captions cc" },
  { id: "anime-audio", page: "playback", label: "Anime audio", hint: "Sub or dub", words: "japanese english dub sub" },

  { id: "dialogue", page: "sound", label: "Dialogue boost", hint: "Clearer voices", words: "speech voice center" },
  { id: "night", page: "sound", label: "Night mode", hint: "Quieter loud parts", words: "compressor dynamic range" },
  { id: "normalize", page: "sound", label: "Even out volume", hint: "Steady level between scenes", words: "loudness normalize leveler" },
  { id: "boost", page: "sound", label: "Volume up to 200%", hint: "For quiet videos", words: "loud amplify" },

  { id: "torrents", page: "sources", label: "Torrents", hint: "YTS, EZTV and Nyaa", words: "p2p vpn" },
  { id: "french", page: "sources", label: "French sources", hint: "VF and VOSTFR", words: "france language" },
  { id: "verify", page: "sources", label: "Check links first", hint: "Drop dead links", words: "dead broken verify" },
  { id: "adult", page: "sources", label: "Show 18+ titles", hint: "Adult titles", words: "nsfw mature adult" },
  { id: "providers", page: "sources", label: "Source order", hint: "Which sites are tried first", words: "provider priority hianime moviebox" },

  { id: "folder", page: "downloads", label: "Download folder", hint: "Where downloads are saved", words: "path location disk" },
  { id: "concurrent", page: "downloads", label: "Downloads at once", hint: "How many at the same time", words: "parallel simultaneous" },
  { id: "seeding", page: "downloads", label: "Keep seeding finished torrents", hint: "Upload after downloading", words: "torrent upload" },

  { id: "music-account", page: "music", label: "YouTube Music account", hint: "Sign in or out", words: "google sign in login youtube" },
  { id: "autoplay", page: "music", label: "Autoplay", hint: "Similar songs when the queue ends", words: "radio continue similar" },
  { id: "eq", page: "music", label: "Equalizer", hint: "10 bands, presets and your own curves", words: "eq bass treble preset parametric preamp curve" },
  { id: "music-normalize", page: "music", label: "Volume normalization", hint: "Even out loud and quiet songs", words: "loudness replaygain level leveler" },
  { id: "output-device", page: "music", label: "Output device", hint: "Headphones, speakers or a DAC", words: "speaker headphones dac audio device output" },
  { id: "exclusive", page: "music", label: "Exclusive mode", hint: "Bit-perfect, past the Windows mixer", words: "wasapi bit perfect audiophile exclusive" },
  { id: "crossfade", page: "music", label: "Crossfade", hint: "Fade songs into each other, or gapless", words: "gapless fade transition mix dj" },
  { id: "music-quality", page: "music", label: "Audio quality", hint: "Low, Normal or High", words: "bitrate data saver kbps opus aac" },
  { id: "precache", page: "music", label: "Save the next songs ahead", hint: "Next songs start at once", words: "cache preload buffer offline" },
  { id: "speed", page: "music", label: "Speed", hint: "0.5x to 2x", words: "tempo faster slower playback rate" },
  { id: "pitch", page: "music", label: "Keep pitch", hint: "Natural voices at any speed", words: "chipmunk tempo nightcore" },
  { id: "music-format", page: "music", label: "Download as", hint: "M4A, MP3, Opus or Velmora only", words: "format mp3 m4a aac opus export tags cover" },
  { id: "ytdlp", page: "music", label: "yt-dlp", hint: "Update if songs stop playing", words: "update songs not playing" },

  { id: "music-cache", page: "music-storage", label: "Saved pages", hint: "Home, library and albums kept on this PC", words: "cache offline clear" },
  { id: "music-saved", page: "music-storage", label: "Saved songs", hint: "Songs kept ahead of time", words: "cache precache clear space" },
  { id: "music-history", page: "music-storage", label: "Search and play history", hint: "Recent searches and songs played", words: "history clear recent" },

  { id: "ani-use", page: "ani", label: "Use Ani", hint: "Velmora's assistant", words: "assistant ai chat gemini" },
  { id: "ani-key", page: "ani", label: "Gemini key", hint: "Your own API key", words: "api key google" },
  { id: "ani-learn", page: "ani", label: "Learn from what I watch", hint: "Personal picks", words: "personalize recommendations" },
  { id: "ani-personalize", page: "ani", label: "Personalize", hint: "What Ani knows about you", words: "privacy data" },
  { id: "ani-models", page: "ani", label: "Chat and research models", hint: "Which Gemini models Ani uses", words: "model flash pro" },

  { id: "anilist", page: "accounts", label: "AniList", hint: "Keep your anime list in step", words: "anime sync account" },
  { id: "trakt", page: "accounts", label: "Trakt", hint: "Add what you finish to your history", words: "sync account movies" },

  { id: "accent", page: "appearance", label: "Accent colour", hint: "The app's colour", words: "color theme palette" },

  { id: "license", page: "license", label: "Max license", hint: "Your key and the ads", words: "max key ads premium" },

  { id: "version", page: "about", label: "Version and updates", hint: "Check for an update", words: "update version about" },
];

/** Settings matching a search, best first. */
export function findSettings(query: string): SettingEntry[] {
  const words = query.toLowerCase().split(/\s+/).filter(Boolean);
  if (!words.length) return [];
  const pageName = (p: SettingsPage) => SETTINGS_PAGES.find((x) => x.id === p)?.label ?? "";
  return SETTINGS_INDEX.map((e) => {
    const label = e.label.toLowerCase();
    const all = `${label} ${e.hint} ${e.words} ${pageName(e.page)}`.toLowerCase();
    if (!words.every((w) => all.includes(w))) return null;
    // Labels that start with the search rank first, then labels that contain it.
    const score = words.reduce((n, w) => n + (label.startsWith(w) ? 3 : label.includes(w) ? 2 : 1), 0);
    return { e, score };
  })
    .filter((x): x is { e: SettingEntry; score: number } => !!x)
    .sort((a, b) => b.score - a.score)
    .map((x) => x.e);
}
