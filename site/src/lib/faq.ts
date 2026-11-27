// Every FAQ answer, in one place: the homepage shows the ones marked `home`,
// the /faq page shows all of them, and both feed the page's FAQ structured data.

export type FaqGroup = "basics" | "account" | "install" | "legal";

export const FAQ_GROUPS: { id: FaqGroup; label: string; icon: string }[] = [
  { id: "basics", label: "The basics", icon: "movie" },
  { id: "account", label: "Account and privacy", icon: "account_circle" },
  { id: "install", label: "Install and updates", icon: "download" },
  { id: "legal", label: "Content and the law", icon: "gavel" },
];

export interface Faq {
  q: string;
  a: string;
  group: FaqGroup;
  /** Also shown in the homepage FAQ. */
  home?: boolean;
}

export const FAQ: Faq[] = [
  {
    q: "Is Velmora free?",
    a: "Yes. The full app is free and open source under GPL-3.0, with no subscription. Velmora Public shows a few quiet ads, never while a video plays, and they help pay for its development. This website shows ads too.",
    group: "basics",
    home: true,
  },
  {
    q: "What is Velmora Max?",
    a: "The same app with no ads at all. Max keys are given out by the developer, not sold. Each key works on one PC at a time: enter it in Settings, under License, and remove it there to move it to another PC. If you sign in to a Velmora account, the key is linked to it, and signing in on a new PC brings Max along (the old PC drops back to Public).",
    group: "basics",
    home: true,
  },
  {
    q: "Why does Windows warn me about the installer?",
    a: "Velmora isn't code-signed yet, because signing certificates are expensive. Windows SmartScreen warns about any unsigned app it hasn't seen often. The source is public and GitHub Actions builds every installer from it. Click More info, then Run anyway.",
    group: "install",
    home: true,
  },
  {
    q: "Does Velmora host any movies or shows?",
    a: "No. Velmora doesn't host, upload or store any content. It searches third-party sites the way a browser does and plays what they serve. You're responsible for what you watch and for following the law where you live.",
    group: "legal",
    home: true,
  },
  {
    q: "Do I need an account?",
    a: "Yes, from Velmora 0.6.0: a free account, made once in your browser with Google, Discord or an emailed code. It gives you a username and profile, keeps your list, progress and settings in step across PCs, and brings Max with you to a new one. Only that first sign-in needs the internet; after it, Velmora opens offline too. AniList and Trakt can still be connected separately.",
    group: "account",
    home: true,
  },
  {
    q: "What does Velmora collect?",
    a: "Your account stores your email, username, display name and picture, bio and links, and your list, progress and settings so they sync. Separately, the app sends once a day anonymous totals (which sites and players get used, which titles get watched) with no ID and no account attached; one switch in Settings, under Privacy, turns that off. The privacy policy has the details, and the public numbers are on the stats page.",
    group: "account",
    home: true,
  },
  {
    q: "What is Ani, and does it cost anything?",
    a: "Ani is an optional assistant inside Velmora. It uses Google Gemini with your own API key, which you can get free from Google AI Studio. Your key and the taste profile Ani builds stay on your PC.",
    group: "basics",
    home: true,
  },
  {
    q: "Is there a Mac, Linux, iPhone or Android version?",
    a: "Android is in production development and drops soon: a full build for phones and tablets on Android 8 and up, with Material You or Liquid Glass. iPhone, iPad and Mac (native SwiftUI apps with Liquid Glass) and Linux builds come after that. Today Velmora runs on Windows.",
    group: "install",
    home: true,
  },
  {
    q: "How do updates work?",
    a: "Velmora checks for new versions when it starts and installs them itself. Every update is signed, and the app checks the signature before installing.",
    group: "install",
    home: true,
  },
  {
    q: "Something doesn't work. Where do I report it?",
    a: "Open an issue on GitHub with what you tried and what happened. Screenshots help a lot.",
    group: "install",
    home: true,
  },
  {
    q: "Do I need administrator rights to install it?",
    a: "No. Velmora installs for your own Windows user account only, so it doesn't ask for administrator rights. Windows may still show its SmartScreen warning, as described above.",
    group: "install",
  },
  {
    q: "Does Velmora work offline?",
    a: "After your first sign-in, Velmora opens without the internet, and you can watch episodes you downloaded. Searching and playing online sources needs a connection.",
    group: "basics",
  },
  {
    q: "What is Theatre?",
    a: "Scheduled screenings that everyone on Velmora watches at the same moment, with a live chat that is never saved. You can book a seat, and the lobby and the chat open 30 minutes before the start.",
    group: "basics",
  },
  {
    q: "What is Velmora Social?",
    a: "A website, social.getvelmora.dpdns.org, with profiles, follows, a feed of what the people you follow are watching, ratings, reviews and lists. There are no direct messages. Sharing what you watch is on by default, and you can turn it off in Settings, under Privacy.",
    group: "account",
  },
  {
    q: "Can I delete my account and my data?",
    a: "Yes. In Velmora, open Settings, then Velmora account. You can copy everything Velmora holds about you, or delete the account and all of it. Deleting can't be undone.",
    group: "account",
  },
];
