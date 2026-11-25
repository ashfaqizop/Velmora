// Facts for the comparison pages. Every claim about another app comes from
// that app's own site or documentation (listed in `sources`), checked on
// CHECKED. Apps change: re-check before editing a row, and update CHECKED.
//
// Rule for this file: say only what the sources say. If we couldn't confirm
// something about another app, leave it out instead of guessing.

import emby from "../assets/brands/emby.svg";
import jellyfin from "../assets/brands/jellyfin.svg";
import kodi from "../assets/brands/kodi.svg";
import plex from "../assets/brands/plex.svg";
import stremio from "../assets/brands/stremio.svg";
import tubi from "../assets/brands/tubi.svg";
import vlc from "../assets/brands/vlcmediaplayer.svg";
import velmoraMark from "../assets/velmora-mark-small.webp";

export const CHECKED = "25 November 2026";
export const CHECKED_ISO = "2026-11-25";

export interface Source {
  label: string;
  url: string;
}
export interface Row {
  label: string;
  velmora: string;
  other: string;
}
export interface Qa {
  q: string;
  a: string;
}

export interface VsData {
  slug: string;
  logo: string;
  name: string;
  /** Short line under the title. */
  tagline: string;
  lead: string;
  shape: string;
  color: string;
  rows: Row[];
  alike: string[];
  pickOther: string[];
  pickVelmora: string[];
  limits: string[];
  faq: Qa[];
  sources: Source[];
  related: { href: string; icon: string; title: string; text: string }[];
}

/** Velmora's own mark, for the comparison heroes and cards. */
export const VELMORA_LOGO = velmoraMark;

// What's true of Velmora, in one place, so every page agrees.
export const VELMORA = {
  price: "Free. Velmora Public shows a few quiet ads, never while a video plays. Velmora Max has none.",
  platforms: "Windows 10 and 11 today. Android, iPhone, iPad, Mac and Linux are coming soon.",
  license: "Open source, GPL-3.0.",
  sources: "Built in. Velmora searches third-party sites for you, so there is nothing to find, install or trust first.",
  hosting: "No. Velmora hosts, uploads and stores no content.",
  extras: "Theatre (scheduled screenings everyone watches together), Velmora Social, YouTube Music, sports and Live TV, the Ani assistant, AniList and Trakt sync, downloads.",
};

export const LIMITS = [
  "Velmora runs on Windows only for now.",
  "The installer isn't code-signed yet, so Windows shows a SmartScreen warning.",
  "You need a free account, made once in your browser.",
  "What you can watch depends on third-party sites, so it varies and some links won't play.",
];


export const STREMIO: VsData = {
  slug: "velmora-vs-stremio",
  logo: stremio,
  name: "Stremio",
  tagline: "Two free apps that find video through other people's sources.",
  lead: "Both apps leave the content to third parties. The difference is who picks the sources: with Stremio you install add-ons, with Velmora they're built in. Here's how they compare, and where Stremio is the better choice.",
  shape: "soft-burst",
  color: "#8fb4ff",
  rows: [
    { label: "Price", velmora: VELMORA.price, other: "Free. Stremio says its current features stay free. There's an optional paid Supporter plan that helps fund development." },
    { label: "Where it runs", velmora: VELMORA.platforms, other: "Windows, macOS, Linux, Android, iOS, Android TV, Apple TV, several smart TV brands, and a web version." },
    { label: "Where the video comes from", velmora: VELMORA.sources, other: "Community add-ons you install. Most of Stremio's content comes from add-ons made and hosted by independent developers." },
    { label: "Does it host content?", velmora: VELMORA.hosting, other: "No. Stremio says it doesn't host, supply or stream any movies, series or copyrighted material." },
    { label: "Open source", velmora: VELMORA.license, other: "In part. The core is MIT, the web app GPL-2.0 and the desktop shell GPL-3.0." },
    { label: "Beyond movies and series", velmora: VELMORA.extras, other: "Movies, series and TV channels, with more depending on the add-ons you install." },
  ],
  alike: [
    "Both are free to use.",
    "Neither hosts the video: both rely on third-party sources, so what's available changes.",
    "Both publish their source code.",
    "Both leave it to you to follow the law where you live.",
  ],
  pickOther: [
    "You want it on your phone, your TV, a Mac or Linux, not only a Windows PC.",
    "You like choosing and installing your own add-ons.",
    "You want a version that runs in a browser.",
  ],
  pickVelmora: [
    "You're on Windows and want search to work out of the box, with no add-ons to set up.",
    "You want a built-in 4K player, with MKV, HEVC and AV1 and a fallback when mpv can't run.",
    "You'd use Theatre, Velmora Social, music or the Ani assistant alongside your watching.",
  ],
  limits: LIMITS,
  faq: [
    { q: "Is Velmora like Stremio?", a: "They solve a similar problem: one free app to find and play movies, series and anime from sources you don't have to hunt for yourself. Stremio finds video through add-ons you install. Velmora has its sources built in. Stremio runs on many more devices." },
    { q: "Does Velmora need add-ons?", a: "No. Velmora searches its built-in sources and ranks the links it finds. There is nothing to install or configure first." },
    { q: "Which is better for Windows?", a: "Both run on Windows. If you only use a Windows PC and want a built-in player and search that works at once, Velmora is built for that. If you also want an app on your phone or TV, Stremio is the one that runs there today." },
    { q: "Are Stremio and Velmora legal?", a: "Neither hosts video. Both find it on third-party sites and leave you responsible for what you watch and for following the law where you live." },
  ],
  sources: [
    { label: "Stremio: plans and what it is (not an IPTV provider)", url: "https://www.stremio.com/plans" },
    { label: "Stremio: downloads for every platform", url: "https://www.stremio.com/downloads" },
    { label: "The Stremio Blog: Stremio Supporters", url: "https://blog.stremio.com/stremio-supporters-a-way-to-sustain-our-development/" },
    { label: "Stremio core, MIT license (GitHub)", url: "https://github.com/Stremio/stremio-core" },
    { label: "Stremio web, GPL-2.0 (GitHub)", url: "https://github.com/Stremio/stremio-web" },
    { label: "Stremio desktop shell, GPL-3.0 (GitHub)", url: "https://github.com/Stremio/stremio-shell" },
  ],
  related: [
    { href: "/stremio-alternatives/", icon: "compare_arrows", title: "Stremio alternatives", text: "Seven other apps, grouped by what you want." },
    { href: "/velmora-vs-kodi/", icon: "balance", title: "Velmora vs Kodi", text: "A different kind of app: a media center you set up." },
    { href: "/download/", icon: "download", title: "Download Velmora", text: "Free for Windows 10 and 11." },
  ],
};

export const KODI: VsData = {
  slug: "velmora-vs-kodi",
  logo: kodi,
  name: "Kodi",
  tagline: "A volunteer-run media center against an app that works out of the box.",
  lead: "Kodi is a free, open-source media center you shape yourself, and it runs almost anywhere. Velmora is narrower: a Windows app that searches and plays at once. Here's how they differ, and when Kodi is the better pick.",
  shape: "12-sided-cookie",
  color: "#3ddc84",
  rows: [
    { label: "Price", velmora: VELMORA.price, other: "Free. Kodi is developed by a non-profit and, the project says, only by volunteers in their spare time." },
    { label: "Where it runs", velmora: VELMORA.platforms, other: "Windows, macOS, Linux, Android, iOS and tvOS, BSD, and more, including webOS and Raspberry Pi." },
    { label: "What you get", velmora: "An app that searches third-party sources and plays the best link in a built-in player.", other: "A media center. Kodi doesn't supply any media: you add your own files, or set it up to reach third-party services." },
    { label: "Add-ons", velmora: "Not needed. Sources are built in.", other: "Extensions written mostly by third-party developers and published to an official repository. The Kodi project distances itself from add-ons that enable unauthorised access to content." },
    { label: "Open source", velmora: VELMORA.license, other: "Yes, GPL-2.0 or later." },
    { label: "Setup", velmora: "Install, sign in once, search.", other: "You build it up yourself: libraries, skins and add-ons." },
  ],
  alike: [
    "Both are free.",
    "Both are open source.",
    "Neither hosts video for you: Kodi supplies no media, and Velmora hosts none.",
    "Both leave it to you to follow the law where you live.",
  ],
  pickOther: [
    "You have your own library of files and want to organise and play it well.",
    "You want to customise how it looks and works, with skins and add-ons.",
    "You need it on a Mac, Linux, a phone, a Raspberry Pi or a TV box.",
    "You'd rather run something from a non-profit, volunteer-run project.",
  ],
  pickVelmora: [
    "You want to search a title and press Play, with no setup.",
    "You're on a Windows PC and want a built-in 4K player.",
    "You'd use Theatre, Velmora Social or music alongside your watching.",
  ],
  limits: LIMITS,
  faq: [
    { q: "Is Velmora a Kodi alternative?", a: "For some people. If you used Kodi mainly to find and play movies and series from online sources, Velmora does that without add-ons. If you used Kodi for your own library, skins and a media-center setup, Kodi is the better tool." },
    { q: "Does Velmora use Kodi add-ons?", a: "No. Velmora has its own built-in sources and doesn't run Kodi add-ons." },
    { q: "Which runs on more devices?", a: "Kodi, by a long way. It runs on Windows, macOS, Linux, Android, iOS, tvOS and more. Velmora is Windows only for now." },
    { q: "Does Kodi provide movies?", a: "No. The Kodi project says it doesn't provide any media. You supply your own content or set it up to use third-party services." },
  ],
  sources: [
    { label: "Kodi: About", url: "https://kodi.tv/about/" },
    { label: "Official Kodi Wiki: Kodi", url: "https://kodi.wiki/view/Kodi" },
    { label: "Official Kodi Wiki: Forks", url: "https://kodi.wiki/view/Forks" },
    { label: "Team Kodi on GitHub", url: "https://github.com/xbmc" },
  ],
  related: [
    { href: "/kodi-alternatives/", icon: "compare_arrows", title: "Kodi alternatives", text: "Seven other apps, grouped by what you want." },
    { href: "/velmora-vs-plex/", icon: "balance", title: "Velmora vs Plex", text: "Plex is a server for your own library." },
    { href: "/download/", icon: "download", title: "Download Velmora", text: "Free for Windows 10 and 11." },
  ],
};

export const PLEX: VsData = {
  slug: "velmora-vs-plex",
  logo: plex,
  name: "Plex",
  tagline: "A server for your own library, against a search-and-play app.",
  lead: "Plex mostly does something different: it runs a media server for the files you own and streams them to your devices, and it also offers a free catalog of ad-supported movies and shows. Velmora searches third-party sources and plays them. Here's how they compare.",
  shape: "puffy-diamond",
  color: "#ffb74d",
  rows: [
    { label: "Price", velmora: VELMORA.price, other: "Free to download, set up a server and stream on your home network. Streaming your own library from outside the home needs a Remote Watch Pass or a Plex Pass. Prices are on plex.tv." },
    { label: "What it is", velmora: "An app that searches third-party sources and plays the best link in a built-in player.", other: "A media server and app for your own library, plus a free, ad-supported catalog of movies and shows that needs no subscription." },
    { label: "Where it runs", velmora: VELMORA.platforms, other: "Apps on many devices. The server runs on Windows, macOS, Linux, many NAS devices and Docker." },
    { label: "Ads", velmora: "A few quiet ads in Public, never while a video plays. None in Max.", other: "The free catalog has ads, and neither Plex Pass nor Remote Watch Pass removes them. Free titles stream online only." },
    { label: "Where the video comes from", velmora: VELMORA.sources, other: "Your own files, and Plex's own licensed free catalog." },
    { label: "Source code", velmora: VELMORA.license, other: "Proprietary. Plex says it's licensed for personal use." },
  ],
  alike: [
    "You can start for free.",
    "Both play video in a polished app, with no technical setup for basic use.",
    "Both have accounts that follow you between devices.",
  ],
  pickOther: [
    "You own a collection of movies, shows or music and want it on every device you have.",
    "You want free, ad-supported movies and shows from a licensed catalog.",
    "You need apps on TVs, phones and consoles, not only a PC.",
  ],
  pickVelmora: [
    "You don't have a library to organise. You want to search for a title and watch it.",
    "You want open-source software and no subscription anywhere.",
    "You're on Windows and want one app for movies, series, anime and music.",
  ],
  limits: LIMITS,
  faq: [
    { q: "Is Velmora a Plex alternative?", a: "Only partly. Plex is mainly a server for your own files, which Velmora isn't. If you want a free place to search and watch titles you don't own, Velmora is closer to that. If you have your own library, Plex or an open-source server like Jellyfin fits better." },
    { q: "Is Plex free?", a: "The software is free, and streaming on your home network is free. Remote streaming of your own library needs a paid plan. Plex's free movie and show catalog is free with ads." },
    { q: "Does Velmora serve my own files?", a: "No. Velmora isn't a media server." },
  ],
  sources: [
    { label: "Plex: plans", url: "https://www.plex.tv/plans/" },
    { label: "Plex support: free vs paid", url: "https://support.plex.tv/articles/202526943-plex-free-vs-paid/" },
    { label: "Plex support: Remote Watch Pass overview", url: "https://support.plex.tv/articles/remote-watch-pass-overview/" },
    { label: "Plex support: free movies and shows FAQ", url: "https://support.plex.tv/articles/frequently-asked-questions-vod/" },
    { label: "Plex support: Media Server requirements", url: "https://support.plex.tv/articles/200375666-plex-media-server-requirements/" },
  ],
  related: [
    { href: "/best-free-movie-apps-for-windows/", icon: "movie", title: "Best free movie apps for Windows", text: "Free catalogs, search apps and players, compared." },
    { href: "/velmora-vs-kodi/", icon: "balance", title: "Velmora vs Kodi", text: "A media center you set up yourself." },
    { href: "/download/", icon: "download", title: "Download Velmora", text: "Free for Windows 10 and 11." },
  ],
};

// ── Alternatives ──────────────────────────────────────────────────────────

export type Kind = "search" | "library" | "catalog" | "player";

export interface App {
  name: string;
  logo?: string;
  kind: Kind;
  color: string;
  shape: string;
  url: string;
  windows: string;
  price: string;
  source: string;
  what: string;
  bestFor: string;
  watchOut: string;
}

export const KINDS: Record<Kind, { label: string; icon: string; blurb: string }> = {
  search: { label: "Finds video for you", icon: "travel_explore", blurb: "Apps that search third-party sources. What's available varies, and you're responsible for what you watch." },
  library: { label: "Your own library", icon: "video_library", blurb: "Media centers and servers for the files you own." },
  catalog: { label: "Free licensed catalogs", icon: "tv", blurb: "Free, ad-supported movies and shows, served by the companies that license them." },
  player: { label: "Just a player", icon: "play_circle", blurb: "Plays files and streams. It doesn't find anything for you." },
};

export const APPS: Record<string, App> = {
  velmora: {
    name: "Velmora",
    logo: velmoraMark,
    kind: "search",
    color: "#3271fe",
    shape: "12-sided-cookie",
    url: "/download/",
    windows: "Yes: Windows 10 and 11. Other platforms are coming soon.",
    price: "Free. Public has a few quiet ads, Max has none.",
    source: "Open source (GPL-3.0)",
    what: "One search across built-in third-party sources, the best link picked for you, and a built-in 4K player. Also Theatre, Velmora Social and music.",
    bestFor: "Windows users who want to search a title and press Play, with no add-ons to set up.",
    watchOut: "Windows only for now, an unsigned installer, and a free account. What plays depends on third-party sites.",
  },
  stremio: {
    name: "Stremio",
    logo: stremio,
    kind: "search",
    color: "#8fb4ff",
    shape: "soft-burst",
    url: "https://www.stremio.com/downloads",
    windows: "Yes. Also macOS, Linux, Android, iOS, TVs and a web version.",
    price: "Free, with an optional paid Supporter plan.",
    source: "Partly open source (MIT core, GPL web app and shell)",
    what: "A media center that gets its content from community add-ons you install. Stremio says it hosts no content itself.",
    bestFor: "People who want the same app on a PC, phone and TV, and don't mind picking add-ons.",
    watchOut: "What you can watch depends entirely on the add-ons you choose to install, and those are made by independent developers.",
  },
  kodi: {
    name: "Kodi",
    logo: kodi,
    kind: "library",
    color: "#3ddc84",
    shape: "4-leaf-clover",
    url: "https://kodi.tv/",
    windows: "Yes. Also macOS, Linux, Android, iOS, tvOS and more.",
    price: "Free, made by a non-profit.",
    source: "Open source (GPL-2.0 or later)",
    what: "A media center for your own library, with skins and add-ons. Kodi supplies no media of its own.",
    bestFor: "People with their own files who want to customise how everything looks and works.",
    watchOut: "It's a toolkit, so there's setup. Add-ons that enable unauthorised access aren't endorsed by the project.",
  },
  plex: {
    name: "Plex",
    logo: plex,
    kind: "library",
    color: "#ffb74d",
    shape: "puffy-diamond",
    url: "https://www.plex.tv/plans/",
    windows: "Yes, as an app and as a server.",
    price: "Free for your home network and the free catalog. Remote streaming of your library is paid.",
    source: "Proprietary",
    what: "A media server and app for your own library, plus a free, ad-supported catalog of movies and shows.",
    bestFor: "People who own media and want it on every device, or who want free ad-supported movies from a licensed catalog.",
    watchOut: "Streaming your own library from outside the house needs a paid plan, and the free catalog has ads.",
  },
  jellyfin: {
    name: "Jellyfin",
    logo: jellyfin,
    kind: "library",
    color: "#a78bfa",
    shape: "8-leaf-clover",
    url: "https://jellyfin.org/",
    windows: "Yes. Clients for Windows, macOS, Linux, Android, iOS, TVs and the web.",
    price: "Free. The project says there are no premium tiers or hidden fees.",
    source: "Open source (GPL)",
    what: "A free media server for your own library. The project says it has no tracking, phone-home behaviour or central servers.",
    bestFor: "People with their own library who want a free, open-source server and don't need paid features.",
    watchOut: "You run the server yourself. It organises your files and doesn't find content for you.",
  },
  emby: {
    name: "Emby",
    logo: emby,
    kind: "library",
    color: "#52b54b",
    shape: "6-sided-cookie",
    url: "https://emby.media/premiere.html",
    windows: "Yes. A server plus client apps.",
    price: "Free to start. Features like hardware transcoding, DVR and offline media need Emby Premiere.",
    source: "Commercial product",
    what: "A personal media server for your own video, music and photo libraries, with a free tier and a paid Premiere tier.",
    bestFor: "People with their own library who want a polished server and will pay for the extras.",
    watchOut: "Emby's own feature matrix puts full playback on some clients, including Windows, behind Premiere.",
  },
  vlc: {
    name: "VLC",
    logo: vlc,
    kind: "player",
    color: "#ff8a65",
    shape: "sunny",
    url: "https://www.videolan.org/vlc/",
    windows: "Yes. Also macOS, Linux, Android and iOS.",
    price: "Free, made by the non-profit VideoLAN.",
    source: "Open source (GPL-2.0)",
    what: "A free player for most media files, DVDs and streaming protocols, with no codec packs needed.",
    bestFor: "Playing files you already have.",
    watchOut: "It's a player. It doesn't search for or recommend anything.",
  },
  tubi: {
    name: "Tubi",
    logo: tubi,
    kind: "catalog",
    color: "#ffd54f",
    shape: "very-sunny",
    url: "https://tubitv.com/",
    windows: "Yes, in a web browser.",
    price: "Free with ads. No subscription or card.",
    source: "Commercial service",
    what: "A free, ad-supported streaming service with a large catalog of movies and TV.",
    bestFor: "Free, licensed movies and TV with nothing to install.",
    watchOut: "Ads, and a catalog you can't change. Availability depends on your country.",
  },
  pluto: {
    name: "Pluto TV",
    kind: "catalog",
    color: "#f48fb1",
    shape: "9-sided-cookie",
    url: "https://pluto.tv/",
    windows: "Yes: a free Windows app, or the website.",
    price: "Free with ads.",
    source: "Commercial service",
    what: "Free live TV channels and on-demand movies and shows, paid for by advertising.",
    bestFor: "Channel surfing and free on-demand movies without a subscription.",
    watchOut: "Ads, and what's on varies. Channel counts change.",
  },
};

export interface AltData {
  slug: string;
  title: string;
  accent: string;
  crumb: string;
  lead: string;
  intro: string;
  /** Groups in the order shown: which apps, with a note. */
  groups: { kind: Kind; apps: string[]; note: string }[];
  faq: Qa[];
  sources: Source[];
  related: { href: string; icon: string; title: string; text: string }[];
}

const COMMON_SOURCES: Source[] = [
  { label: "Stremio: plans", url: "https://www.stremio.com/plans" },
  { label: "Kodi: About", url: "https://kodi.tv/about/" },
  { label: "Plex support: free vs paid", url: "https://support.plex.tv/articles/202526943-plex-free-vs-paid/" },
  { label: "Jellyfin", url: "https://jellyfin.org/" },
  { label: "Emby Premiere feature matrix", url: "https://emby.media/support/articles/Premiere-Feature-Matrix.html" },
  { label: "VideoLAN: VLC", url: "https://www.videolan.org/vlc/" },
  { label: "Tubi help center: supported devices", url: "https://tubitv.com/help-center/About-Tubi/articles/4409907124763" },
  { label: "Pluto TV: where to watch", url: "https://pluto.tv/free/where-to-watch" },
];

export const STREMIO_ALTS: AltData = {
  slug: "stremio-alternatives",
  title: "Stremio",
  accent: "alternatives",
  crumb: "Stremio alternatives",
  lead: "Seven apps people use instead of Stremio, grouped by what you actually want from one. We make Velmora, so it's on the list. We've said what each app is, who it suits, and what to watch out for.",
  intro: "Stremio finds movies and series through add-ons. Depending on what you liked about it, you may want something that finds video for you, a server for your own files, or a free licensed catalog.",
  groups: [
    { kind: "search", apps: ["velmora"], note: "The closest in spirit: an app that finds video on third-party sites. Velmora has its sources built in, so there are no add-ons to choose." },
    { kind: "library", apps: ["kodi", "plex", "jellyfin", "emby"], note: "If you own your media, a library app or server organises it and plays it on your devices." },
    { kind: "catalog", apps: ["tubi", "pluto"], note: "If you want free movies and shows that are licensed, these are paid for by ads." },
    { kind: "player", apps: ["vlc"], note: "If you only need to play files you already have." },
  ],
  faq: [
    { q: "What is the best free alternative to Stremio?", a: "It depends on what you want. For an app that finds video for you, Velmora is closest. For your own library, Kodi, Plex and Jellyfin. For free, licensed movies and TV, Tubi and Pluto TV." },
    { q: "Is there a Stremio alternative with no add-ons?", a: "Velmora has its sources built in, so there are no add-ons to find or install. It runs on Windows only for now." },
    { q: "Are these alternatives legal?", a: "Tubi, Pluto TV and Plex's free catalog serve licensed content with ads. Kodi, Jellyfin, Emby, Plex and VLC play what you give them. Apps that find video on third-party sites, like Stremio and Velmora, leave you responsible for what you watch and for following the law where you live." },
  ],
  sources: COMMON_SOURCES,
  related: [
    { href: "/velmora-vs-stremio/", icon: "balance", title: "Velmora vs Stremio", text: "A side-by-side of the two." },
    { href: "/kodi-alternatives/", icon: "compare_arrows", title: "Kodi alternatives", text: "If it's Kodi you're replacing." },
    { href: "/best-free-movie-apps-for-windows/", icon: "movie", title: "Best free movie apps for Windows", text: "All the options in one place." },
  ],
};

export const KODI_ALTS: AltData = {
  slug: "kodi-alternatives",
  title: "Kodi",
  accent: "alternatives",
  crumb: "Kodi alternatives",
  lead: "Seven apps people use instead of Kodi, grouped by what you want from one. We make Velmora, so it's on the list. Each entry says who it suits and what to watch out for.",
  intro: "Kodi is a free media center you build up yourself. If it's more setup than you want, or you only need part of what it does, one of these may fit better.",
  groups: [
    { kind: "library", apps: ["plex", "jellyfin", "emby"], note: "If you used Kodi for your own library, these servers organise it and stream it to your devices, with less to configure." },
    { kind: "search", apps: ["velmora", "stremio"], note: "If you used Kodi with add-ons to find movies and series, these two do that: Stremio with add-ons you install, Velmora with sources built in." },
    { kind: "catalog", apps: ["tubi", "pluto"], note: "If you want free, licensed movies and shows with nothing to set up." },
    { kind: "player", apps: ["vlc"], note: "If you only need to play files." },
  ],
  faq: [
    { q: "What is a simple alternative to Kodi?", a: "If you want something with less setup: Plex or Jellyfin for your own library, Velmora or Stremio for finding video, VLC for playing files." },
    { q: "What if I want to stay with something Kodi-based?", a: "The official Kodi wiki lists GPL-compliant forks, including LibreELEC, a complete media center software suite for embedded systems and computers." },
    { q: "Is there an open-source alternative to Kodi?", a: "Yes. Jellyfin, VLC and Velmora are open source, and Stremio is partly open source." },
  ],
  sources: [...COMMON_SOURCES, { label: "Official Kodi Wiki: Forks", url: "https://kodi.wiki/view/Forks" }],
  related: [
    { href: "/velmora-vs-kodi/", icon: "balance", title: "Velmora vs Kodi", text: "A side-by-side of the two." },
    { href: "/stremio-alternatives/", icon: "compare_arrows", title: "Stremio alternatives", text: "If it's Stremio you're replacing." },
    { href: "/best-free-movie-apps-for-windows/", icon: "movie", title: "Best free movie apps for Windows", text: "All the options in one place." },
  ],
};

export const BEST_FREE: AltData = {
  slug: "best-free-movie-apps-for-windows",
  title: "Best free movie apps",
  accent: "for Windows",
  crumb: "Best free movie apps for Windows",
  lead: "Nine free ways to watch movies on a Windows PC, grouped by how they work. We make Velmora, so we've included it and said where it falls short. Facts below come from each app's own site.",
  intro: "There isn't one best app, because they do different jobs. Pick the group that matches what you want, then read what to watch out for.",
  groups: [
    { kind: "catalog", apps: ["tubi", "pluto", "plex"], note: "Free movies and shows from licensed catalogs, paid for by ads. Nothing to install for Tubi (it runs in a browser). Pluto TV has a Windows app and a website. Plex's free catalog is separate from its media server." },
    { kind: "search", apps: ["velmora", "stremio"], note: "Apps that find movies, series and anime on third-party sites. There's more to watch, and you're responsible for what you watch and for following the law where you live." },
    { kind: "library", apps: ["kodi", "jellyfin"], note: "Free, open-source ways to organise and play files you already own." },
    { kind: "player", apps: ["vlc"], note: "The one to have for playing any file." },
  ],
  faq: [
    { q: "What is the best free movie app for Windows?", a: "For free, licensed movies and TV with ads, Tubi and Pluto TV. For finding movies, series and anime across third-party sources, Velmora or Stremio. For your own files, Kodi, Jellyfin or VLC." },
    { q: "Are free movie apps legal?", a: "Tubi, Pluto TV and Plex's free catalog serve licensed content paid for by ads. Apps that find video on third-party sites leave you responsible for what you watch and for following the law where you live." },
    { q: "Do I need to pay for any of these?", a: "No. All nine are free to use. Some have optional paid plans, such as Plex, Emby and Stremio's Supporter plan." },
  ],
  sources: COMMON_SOURCES,
  related: [
    { href: "/stremio-alternatives/", icon: "compare_arrows", title: "Stremio alternatives", text: "If it's Stremio you're replacing." },
    { href: "/kodi-alternatives/", icon: "compare_arrows", title: "Kodi alternatives", text: "If it's Kodi you're replacing." },
    { href: "/free-netflix-alternative/", icon: "tv", title: "A free Netflix alternative", text: "What Velmora is, and isn't." },
  ],
};
