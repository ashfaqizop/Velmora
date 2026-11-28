// Where Velmora for Android stands, for /android/ and the home page pill.
// Mirrors docs/android_build_plan.md: when a phase moves, change its `status`
// (and `date` when it's done) and bump UPDATED.

export type PhaseStatus = "done" | "testing" | "building" | "planned";

export interface Phase {
  n: number;
  title: string;
  icon: string;
  status: PhaseStatus;
  /** Day it was finished, YYYY-MM-DD. */
  date?: string;
  summary: string;
  /** What you can do once it's in. */
  result: string;
  pieces: string[];
}

export interface PhaseGroup {
  name: string;
  blurb: string;
  from: number;
  to: number;
}

export const UPDATED = "2026-11-28";

export const DEVICES = [
  { icon: "phone_android", label: "Android 9 phone" },
  { icon: "phone_android", label: "Android 14 phone" },
  { icon: "tablet_android", label: "Android 16 tablet" },
];

export const GROUPS: PhaseGroup[] = [
  { name: "Build the app", blurb: "Everything desktop Velmora does, working on Android.", from: 1, to: 9 },
  { name: "Make it feel like Velmora", blurb: "Three phases spent only on design, motion and polish.", from: 10, to: 12 },
  { name: "Ship it", blurb: "Updates, the APK, and the release.", from: 13, to: 13 },
];

export const PHASES: Phase[] = [
  {
    n: 1,
    title: "Import and rebrand",
    icon: "badge",
    status: "done",
    date: "2026-11-27",
    summary:
      "Echo Music, the open-source YouTube Music app, came in as the base and was renamed to Velmora from the package name down. Firebase, Google Drive backup and Listen Together came out. Velmora's icons and links went in.",
    result: "A working music app called Velmora, built and signed in the cloud.",
    pieces: ["com.velmora.stepbyte", "Fox icon, themed on Android 13+", "Signed builds from CI", "No Firebase"],
  },
  {
    n: 2,
    title: "Rust engine on Android",
    icon: "memory",
    status: "done",
    date: "2026-11-27",
    summary:
      "UniverseAPI, the Rust engine behind desktop Velmora, compiled for Android and called from Kotlin. Search, sources, torrents and the encrypted download vault come with it, plus a small local server that feeds the video player.",
    result: "A hidden engine lab that searches, finds sources and plays one, on all three test devices.",
    pieces: ["UniFFI bridge", "arm64 and armv7", "Torrent streaming", "Encrypted vault", "Engine lab"],
  },
  {
    n: 3,
    title: "Shell and navigation",
    icon: "dashboard",
    status: "done",
    date: "2026-11-27",
    summary:
      "The frame of the app. Velmora and Music become two modes with a switch in the top bar, the bottom bar is Home, Browse, Theatre and Search, tablets get a side rail, and both looks arrive: Liquid Glass and Material You.",
    result: "Sliding between movies and music.",
    pieces: ["Mode switch", "Bottom bar and side rail", "Liquid Glass", "Material You", "Mini player"],
  },
  {
    n: 4,
    title: "Accounts and onboarding",
    icon: "account_circle",
    status: "done",
    date: "2026-11-27",
    summary:
      "Sign in with your Velmora account, YouTube Music sign-in for Music, and a first run that sets things up. My List, watch progress and settings sync with desktop.",
    result: "Your desktop library on your phone.",
    pieces: ["Velmora sign-in", "Sync with desktop", "Max restore", "Onboarding"],
  },
  {
    n: 5,
    title: "Home, Browse, title pages, search",
    icon: "movie",
    status: "done",
    date: "2026-11-28",
    summary:
      "Trending rows, Browse tabs for Movies, Series, Anime, Live TV and Sports, title pages with seasons and episodes, My List, and one search across everything. It also fixes what phase 4 testing found offline: once you've signed in, the app opens straight to your library with no network, and a slow connection never flashes the sign-in screen.",
    result: "Finding something to watch, online or not.",
    pieces: ["Home rows", "Browse tabs", "Title pages", "My List", "One search", "Offline start"],
  },
  {
    n: 6,
    title: "Video player",
    icon: "play_circle",
    status: "testing",
    summary:
      "ExoPlayer on the engine's sources, with a source picker, subtitles, audio tracks, quality, picture-in-picture, continue watching and torrent streaming. Cinema audio (AC3, E-AC3, DTS) plays on every phone through a built-in decoder, and your spot syncs with desktop.",
    result: "Pressing play.",
    pieces: ["Source picker", "Subtitles and audio", "Picture-in-picture", "Continue watching", "Gestures", "Mobile data rule"],
  },
  {
    n: 7,
    title: "Downloads",
    icon: "download",
    status: "testing",
    summary:
      "Downloads saved encrypted, so only Velmora plays them, with a Wi-Fi only rule and pages for downloaded movies and downloaded music. They keep going in the background with a progress notification, pause and resume, and play offline from the title page and Continue watching.",
    result: "Watching offline.",
    pieces: ["Encrypted .vlm files", "Background downloads", "Wi-Fi rule", "Whole seasons", "Offline play"],
  },
  {
    n: 8,
    title: "Live TV, Sports, Theatre",
    icon: "theaters",
    status: "testing",
    summary:
      "Live channels with favourites and a stream that switches by itself when one drops, match pages with where to watch in your country, and Theatre: the lineup, booking and reminders, the lobby with its countdown, the synced show, live chat that's never saved, and reactions.",
    result: "Watching together.",
    pieces: ["Live TV", "Sports", "Where to watch", "Theatre lobby", "Synced show", "Live chat"],
  },
  {
    n: 9,
    title: "Social and the rest",
    icon: "forum",
    status: "building",
    summary:
      "Velmora Social inside the app, already signed in. Links to titles, songs, profiles and screenings open straight in the app. Announcements from the team, anonymous usage counts you can see and switch off, what you watch shared on your profile only if you want, Transfer from Echo Music, and Velmora's own settings in one place.",
    result: "Everything desktop has.",
    pieces: ["Velmora Social", "App links", "Announcements", "Privacy", "Transfer from Echo Music", "Settings"],
  },
  {
    n: 10,
    title: "UI/UX I: look and layout",
    icon: "palette",
    status: "planned",
    summary:
      "The design language: Expressive shapes, Velmora's fonts, both looks equally good, colors taken from each poster, haptics, and skeletons where there used to be spinners. Phone and tablet layouts made on purpose, with a side rail and a two-column title page on tablets and a landscape player with its own controls.",
    result: "It feels like Velmora, not a port.",
    pieces: ["Expressive shapes", "Poster colors", "Skeleton loading", "Tablet layouts", "Landscape player"],
  },
  {
    n: 11,
    title: "UI/UX II: transitions and motion",
    icon: "animation",
    status: "planned",
    summary:
      "Nothing jumps. A poster grows into its title page, a card opens into the player, the mini player into the full one, back gestures preview where they go, the mode switch morphs one world into the other, lists arrive in a ripple, and every press and sheet moves on springs. Reduced motion respected.",
    result: "One continuous app.",
    pieces: ["Shared elements", "Predictive back", "Mode switch", "Springs everywhere", "Player motion"],
  },
  {
    n: 12,
    title: "UI/UX III: polish with testers",
    icon: "auto_awesome",
    status: "planned",
    summary:
      "Every screen walked through on the test devices and fixed: the small interactions, startup time and smooth scrolling, accessibility, the wording, and the first run, refined from where testers get stuck.",
    result: "The details done.",
    pieces: ["Device walkthroughs", "Startup and smoothness", "Wording pass"],
  },
  {
    n: 13,
    title: "Updates and release",
    icon: "rocket_launch",
    status: "planned",
    summary:
      "An in-app updater that downloads in the background and asks before it installs, one release for desktop and Android, and the APK right here on this website.",
    result: "Velmora for Android, out.",
    pieces: ["In-app updates", "APK on the website", "Release"],
  },
];

export const STATUS_LABEL: Record<PhaseStatus, string> = {
  done: "Done",
  testing: "Testing on devices",
  building: "Building now",
  planned: "Planned",
};

/** The phase being worked on: the first one that isn't done. */
export function currentPhase(): Phase {
  return PHASES.find((p) => p.status !== "done") ?? PHASES[PHASES.length - 1];
}

/** Rough share of the work finished: done phases count fully, the one in progress partly. */
export function progress(): number {
  const weight: Record<PhaseStatus, number> = { done: 1, testing: 0.8, building: 0.4, planned: 0 };
  return PHASES.reduce((sum, p) => sum + weight[p.status], 0) / PHASES.length;
}

export function shortDate(iso: string): string {
  return new Date(`${iso}T12:00:00Z`).toLocaleDateString("en-GB", { day: "numeric", month: "short", year: "numeric", timeZone: "UTC" });
}
