// The changelog, written for the website. It's kept here rather than read
// from GitHub so each release can have its own story, highlights and look.
// Add a new release at the top; set `soon` until it's actually out.

export type ChangeKind = "new" | "improved" | "fixed";

export interface Change {
  kind: ChangeKind;
  title: string;
  text: string;
}

export interface Highlight {
  icon: string;
  title: string;
  text: string;
}

export interface ReleaseNote {
  version: string;
  /** ISO date it shipped (or is planned for). */
  date: string;
  /** A short name for the release. */
  name: string;
  summary: string;
  /** Accent for this release's card and timeline node. */
  color: string;
  /** m3e-shape name for the timeline node. */
  shape: string;
  /** Built but not released yet. */
  soon?: boolean;
  highlights?: Highlight[];
  changes: Change[];
}

export const KIND_LABEL: Record<ChangeKind, string> = { new: "New", improved: "Improved", fixed: "Fixed" };
export const KIND_ICON: Record<ChangeKind, string> = { new: "add_circle", improved: "trending_up", fixed: "build" };

export const RELEASES: ReleaseNote[] = [
  {
    version: "0.3.5",
    date: "2026-11-11",
    name: "Music",
    summary: "YouTube Music inside Velmora, with synced lyrics and a player that moves with the song. Velmora also comes two ways now: Public, with a few quiet ads, and Max, with none.",
    color: "#7b6cff",
    shape: "12-sided-cookie",
    highlights: [
      { icon: "library_music", title: "Music", text: "Sign in with Google once and your YouTube Music is here: mixes, playlists, likes and your library." },
      { icon: "lyrics", title: "Synced lyrics", text: "Big lines that follow the song. Click one to jump there." },
      { icon: "verified", title: "Velmora Public", text: "Everything stays free. A few quiet ads help pay for development, never while a video plays." },
      { icon: "workspace_premium", title: "Velmora Max", text: "No ads at all. Paste a Max key in Settings, under License." },
      { icon: "lock", title: "Ads in a sealed frame", text: "They can't see what you search or watch, and clicks open in your browser." },
    ],
    changes: [
      { kind: "new", title: "Music mode", text: "Music in the sidebar switches Velmora into a music app: Home, Explore, Library and your playlists." },
      { kind: "new", title: "A player that floats", text: "A small player follows you across the app. Open it for the cover, lyrics, the queue and the music video." },
      { kind: "new", title: "Your library, synced", text: "Likes and playlist edits go straight to your YouTube Music account." },
      { kind: "new", title: "Songs offline", text: "Download songs and albums as Velmora-only files." },
      { kind: "new", title: "Ani knows music", text: "Six more tools and two more cards: songs like one you love, charts, artists and albums." },
      { kind: "new", title: "License in Settings", text: "Activate, change or remove a Max key. A key works on one PC at a time." },
      { kind: "new", title: "Quiet ads in Public", text: "None on your first launch or in the first two minutes, and one on screen at a time." },
      { kind: "improved", title: "A simpler sidebar", text: "The rotating tips are gone." },
      { kind: "improved", title: "A side panel that folds", text: "The panel on Home is closed until you open it." },
      { kind: "new", title: "Privacy policy and terms", text: "Both are on the website, in plain language." },
    ],
  },
  {
    version: "0.3.0",
    date: "2026-11-09",
    name: "Sports and Live TV",
    summary: "Live scores and fixtures on Home, a Live TV page with free channels, Ani that knows both, and a search that finds the title you typed.",
    color: "#ff5a5a",
    shape: "sunny",
    highlights: [
      { icon: "sports_score", title: "Sports on Home", text: "Live now, upcoming and results in one rail, with a page for every match." },
      { icon: "tv", title: "Where to watch", text: "Official broadcasters in your country, with a Play button for free channels that carry the match." },
      { icon: "live_tv", title: "Live TV", text: "Browse channels by country and category, keep favorites, and watch in the built-in player." },
      { icon: "wand_stars", title: "Ani knows sports", text: "Ask who plays tonight, what the score was, or where a match is on." },
    ],
    changes: [
      { kind: "new", title: "Sports page and match pages", text: "Scores can run a few minutes behind, and the page says so." },
      { kind: "new", title: "Live TV", text: "Channels come from a public playlist. Velmora hosts none of them, and some may not play." },
      { kind: "new", title: "Ani: 8 more tools and 3 more cards", text: "Matches, where to watch and channels, all from real data." },
      { kind: "improved", title: "Search", text: "If a result is exactly the title you typed, only that shows, with a button for the full list." },
      { kind: "improved", title: "A new font", text: "Bricolage Grotesque for text, with Instrument Serif for headings as before." },
    ],
  },
  {
    version: "0.2.2",
    date: "2026-11-01",
    name: "Better sound",
    summary: "Four sound options for the player: clearer dialogue, a night mode, even volume between sources, and up to 200% volume.",
    color: "#ffb74d",
    shape: "soft-burst",
    highlights: [
      { icon: "record_voice_over", title: "Dialogue boost", text: "Raises the center channel on 5.1 and 7.1 audio, and lifts speech on stereo." },
      { icon: "bedtime", title: "Night mode", text: "Quiets explosions and lifts whispers, so you stop reaching for the volume." },
      { icon: "equalizer", title: "Even out volume", text: "Smooths the jump between loud scenes, quiet scenes and different sources." },
      { icon: "volume_up", title: "Up to 200%", text: "For quiet videos and laptop speakers. The bar turns amber past 100." },
    ],
    changes: [
      { kind: "new", title: "Sound menu in the player", text: "A new sound button next to the audio track button. Changes apply while the video plays." },
      { kind: "new", title: "Sound settings", text: "The same four switches in Settings, under Sound. Your choice carries over to every video." },
      { kind: "improved", title: "Sidebar tips", text: "New tips about the sound options." },
    ],
  },
  {
    version: "0.2.1",
    date: "2026-10-31",
    name: "Search, fixed",
    summary: "A quick fix: search works again.",
    color: "#2dd4bf",
    shape: "4-sided-cookie",
    changes: [
      {
        kind: "fixed",
        title: "Search never finished",
        text: "In 0.2.0, searching from the top bar stayed on \"0 sources answered\". Recording the search for Ani kept restarting it. Results arrive as each source answers again.",
      },
    ],
  },
  {
    version: "0.2.0",
    date: "2026-10-31",
    name: "Meet Ani",
    summary: "Velmora's own assistant. Ask it what to watch and it answers with titles you can play, save or download.",
    color: "#7b6cff",
    shape: "very-sunny",
    highlights: [
      { icon: "travel_explore", title: "Looks things up first", text: "37 lookup tools across every source, TMDB and AniList. You watch it work, live." },
      { icon: "view_carousel", title: "Answers with cards", text: "Posters with Play, My List and Download buttons, ranked rows, comparisons, watch orders and more." },
      { icon: "person_heart", title: "Knows your taste", text: "Learns from what you search, watch and download. Everything stays on your PC." },
      { icon: "back_hand", title: "You stay in charge", text: "Ani never plays, downloads or changes a setting on its own." },
    ],
    changes: [
      { kind: "new", title: "Ani", text: "In the menu on the left. Optional, and runs on Google Gemini with your own free key." },
      { kind: "new", title: "Personalize", text: "See and remove what Ani has learned in Settings, under Ani. Tell it \"I prefer dubs\" once and it remembers." },
      { kind: "improved", title: "Cleaner source names", text: "The player and download dialog show plain names instead of odd server nicknames." },
      { kind: "improved", title: "More sidebar tips", text: "Ten new tips about Ani." },
      { kind: "fixed", title: "Powered by UniverseAPI", text: "The mark no longer loses its asterisk when another copy of it is hidden." },
    ],
  },
  {
    version: "0.1.5",
    date: "2026-10-31",
    name: "Your folder, your files",
    summary: "Sliders that follow your mouse, a download folder you pick, and downloads that play only in Velmora.",
    color: "#ff5a5a",
    shape: "8-leaf-clover",
    highlights: [
      { icon: "folder_open", title: "Pick your download folder", text: "In Settings or during setup, with an Open folder button on the Downloads page." },
      { icon: "lock", title: "Private downloads", text: "New downloads are encrypted for this install and play only in Velmora, with seeking as before." },
    ],
    changes: [
      { kind: "new", title: "Download folder", text: "Choose where downloads go, in Settings under Downloads or during first-time setup." },
      { kind: "new", title: "Velmora-only files", text: "New downloads are saved as .vlm files. Files from 0.1.0 still play." },
      { kind: "fixed", title: "Sliders jumped", text: "Seek, volume and Settings sliders now stay under the pointer." },
      { kind: "fixed", title: "Switching subtitles or audio", text: "Works again, and a failed switch gives a plain message." },
      { kind: "fixed", title: "Downloads vanished after a restart", text: "Finished downloads show under \"On this PC\", ready to play." },
      { kind: "fixed", title: "Downloading twice overwrote the first copy", text: "The second one saves as \"Name (2)\", and the dialog tells you that you already have it." },
      { kind: "fixed", title: "Cancelled downloads left big files", text: "Cancelling now removes the partial file, and old leftovers are cleaned up at launch." },
      { kind: "fixed", title: "\"Clear finished\" removed failed downloads", text: "Failed ones stay, so you can retry from where they stopped." },
    ],
  },
  {
    version: "0.1.0",
    date: "2026-10-31",
    name: "Hello, Velmora",
    summary: "The first release: movies, series and anime from many sources, searched and played in one Windows app.",
    color: "#c38bff",
    shape: "12-sided-cookie",
    highlights: [
      { icon: "travel_explore", title: "Search everything", text: "Every source at once, with results arriving as each site answers." },
      { icon: "play_circle", title: "Built-in player", text: "MKV, HEVC, AC3 and 4K. If a link fails, the next one starts at the same spot." },
      { icon: "history", title: "Continue Watching", text: "Resume to the second, with up next at the end of each episode." },
      { icon: "sync", title: "AniList and Trakt", text: "What you finish is marked as watched for you." },
    ],
    changes: [
      { kind: "new", title: "Downloads", text: "Pause, resume and offline playback." },
      { kind: "new", title: "Torrents", text: "Stream while they download, with seeding off by default." },
      { kind: "new", title: "Title pages", text: "Cast and crew, trailer, gallery, ratings, release details and similar titles." },
      { kind: "new", title: "Settings", text: "Quality, codec, languages, source order and accent color, plus a short setup on first launch." },
      { kind: "new", title: "Self-updating", text: "New versions install themselves when the app starts." },
    ],
  },
];

/** What's being worked on, shown above the releases. */
export const NEXT = {
  title: "Velmora for Android",
  text: "The same app on your phone and tablet: search, the player, downloads and Ani. Work on the Android build has started.",
};

export const longDate = (iso: string) =>
  new Date(`${iso}T12:00:00`).toLocaleDateString(undefined, { day: "numeric", month: "long", year: "numeric" });
