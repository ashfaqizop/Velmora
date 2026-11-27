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

export const UPDATED = "2026-11-27";

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
    status: "building",
    summary:
      "The frame of the app. Velmora and Music become two modes with a switch in the top bar, the bottom bar is Home, Browse, Theatre and Search, tablets get a side rail, and both looks arrive: Liquid Glass and Material You.",
    result: "Sliding between movies and music.",
    pieces: ["Mode switch", "Bottom bar and side rail", "Liquid Glass", "Material You", "Mini player"],
  },
  {
    n: 4,
    title: "Accounts and onboarding",
    icon: "account_circle",
    status: "building",
    summary:
      "Sign in with your Velmora account, YouTube Music sign-in for Music, and a first run that sets things up. My List, watch progress and settings sync with desktop.",
    result: "Your desktop library on your phone.",
    pieces: ["Velmora sign-in", "Sync with desktop", "Max restore", "Onboarding"],
  },
  {
    n: 5,
    title: "Home, Browse, title pages, search",
    icon: "movie",
    status: "planned",
    summary:
      "Trending rows, Browse tabs for Movies, Series, Anime, Live TV and Sports, title pages with seasons and episodes, My List, and one search across everything.",
    result: "Finding something to watch.",
    pieces: ["Home rows", "Browse tabs", "Title pages", "One search"],
  },
  {
    n: 6,
    title: "Video player",
    icon: "play_circle",
    status: "planned",
    summary:
      "ExoPlayer on the engine's sources, with a source picker, subtitles, audio tracks, quality, picture-in-picture, continue watching and torrent streaming.",
    result: "Pressing play.",
    pieces: ["Source picker", "Subtitles and audio", "Picture-in-picture", "Continue watching"],
  },
  {
    n: 7,
    title: "Downloads",
    icon: "download",
    status: "planned",
    summary:
      "Downloads saved encrypted, so only Velmora plays them, with a Wi-Fi only rule and pages for downloaded movies and downloaded music.",
    result: "Watching offline.",
    pieces: ["Encrypted .vlm files", "Background downloads", "Wi-Fi rule"],
  },
  {
    n: 8,
    title: "Live TV, Sports, Theatre",
    icon: "theaters",
    status: "planned",
    summary: "Live channels, matches and where to watch them, and Theatre: the lineup, booking, the lobby, chat and the synced show.",
    result: "Watching together.",
    pieces: ["Live TV", "Sports", "Theatre with chat"],
  },
  {
    n: 9,
    title: "Social and the rest",
    icon: "forum",
    status: "planned",
    summary:
      "Velmora Social, links that open in the app, announcements, privacy settings, and Transfer from Echo Music for people moving over.",
    result: "Everything desktop has.",
    pieces: ["Velmora Social", "App links", "Transfer from Echo Music"],
  },
  {
    n: 10,
    title: "UI/UX I: look and motion",
    icon: "palette",
    status: "planned",
    summary:
      "The design language. Expressive shapes and motion everywhere, a poster that grows into its title page, the mode switch finished, colors taken from each poster, haptics, and skeletons where there used to be spinners.",
    result: "It feels like Velmora, not a port.",
    pieces: ["Expressive motion", "Shared transitions", "Poster colors", "Skeleton loading"],
  },
  {
    n: 11,
    title: "UI/UX II: every screen size",
    icon: "devices_fold",
    status: "planned",
    summary:
      "Phone, tablet and foldable layouts designed on purpose rather than stretched, a landscape player with its own controls, Android 8 to 11 looking as finished as 16, and accessibility: TalkBack, large text, contrast.",
    result: "Right at every size.",
    pieces: ["Tablet layouts", "Foldables", "Landscape player", "Accessibility"],
  },
  {
    n: 12,
    title: "UI/UX III: polish with testers",
    icon: "auto_awesome",
    status: "planned",
    summary:
      "Every screen walked through on the test devices and fixed: the small interactions, startup time and smooth scrolling, the wording, and the first run, refined from where testers get stuck.",
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
