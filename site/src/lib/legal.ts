// The privacy policy and terms, in plain language. Update `UPDATED` with any change.

export const UPDATED = "2026-11-11";

export interface LegalSection {
  heading: string;
  paragraphs: string[];
}

export interface LegalDoc {
  title: string;
  intro: string;
  sections: LegalSection[];
}

export const PRIVACY: LegalDoc = {
  title: "Privacy policy",
  intro:
    "Velmora has no accounts and keeps what you watch on your PC. This page lists everything that does leave it, who receives it, and why. It covers the Velmora app for Windows, the license service and this website.",
  sections: [
    {
      heading: "What stays on your PC",
      paragraphs: [
        "Your watch history, Continue Watching, My List, settings, downloads, the songs you play and search for in Music, and the taste profile Ani builds are stored on your PC only. Velmora has no server that collects them, and nobody else can see them.",
      ],
    },
    {
      heading: "Searching and playing",
      paragraphs: [
        "When you search or play something, Velmora contacts third-party sites and services directly from your PC, the way a browser does: the video sources, TMDB, AniList and Cinemeta for title details, TheSportsDB and football-data.org for scores, and the public iptv-org playlist for Live TV. Each of them sees your IP address and the request, under its own privacy policy.",
        "Updates are checked on GitHub when Velmora starts, so GitHub sees your IP address too.",
      ],
    },
    {
      heading: "Ani",
      paragraphs: [
        "Ani is optional. If you turn it on, your messages and the results Ani looks up are sent to Google Gemini with your own API key, under Google's terms for that key. Your key stays on your PC.",
      ],
    },
    {
      heading: "Music",
      paragraphs: [
        "Music is optional and is the only part of Velmora that needs a Google account. You sign in on Google's own page, in a separate window; Velmora never sees your password. Velmora keeps the YouTube cookies that window receives, encrypted with Windows so only your Windows account on this PC can read them, and uses them to talk to YouTube Music directly from your PC. They never reach us.",
        "With them, Velmora shows your YouTube Music home, library and playlists, and sends your likes and playlist changes to your account. A hidden window refreshes the session every 20 minutes so Google doesn't sign you out.",
        "Songs play through yt-dlp, a separate open-source program Velmora downloads and keeps updated. It fetches each song from YouTube without your cookies. Lyrics come from YouTube Music and LRCLIB. To even out volume, Velmora asks YouTube Music for each song's loudness level, signed in as you when you are; you can turn that off in Settings, under Music.",
        "Signing out in Settings, under Music, deletes the cookies and the sign-in window's data from this PC.",
      ],
    },
    {
      heading: "AniList and Trakt",
      paragraphs: [
        "If you connect AniList or Trakt, Velmora stores the sign-in on your PC and tells that service what you finished watching. Nothing is sent until you connect.",
      ],
    },
    {
      heading: "Ads in Velmora Public",
      paragraphs: [
        "Velmora Public shows ads from Adsterra. Each ad is a page from this website, shown inside a sealed frame in the app. The frame can't read anything in Velmora, so the ad network never learns what you search or watch.",
        "Like any ad network, Adsterra and its advertisers receive your IP address and basic device and browser details when an ad loads, and may use cookies inside that frame, under Adsterra's privacy policy. Clicking an ad opens it in your own browser.",
        "Velmora sends us one thing about ads: anonymous counts of how many ads each spot in the app showed each day. These counts carry no ID and nothing about you.",
        "Velmora Max shows no ads, loads nothing from Adsterra and sends no counts.",
      ],
    },
    {
      heading: "Max licenses",
      paragraphs: [
        "When you activate a Max key, Velmora sends the key and a one-way hash of your PC's Windows ID to our license service. The key contains its license ID and the name the developer gave it. The hash can't be turned back into the ID.",
        "We store the license ID, the name, the hash, and when the key was activated and last checked, so that each key works on one PC at a time. Velmora checks again about once a week. When you remove the key in Settings, the link to your PC is deleted.",
      ],
    },
    {
      heading: "This website",
      paragraphs: [
        "The website is hosted on Vercel, which keeps standard server logs, including IP addresses. Pages show ads from Adsterra, including banners, a social bar and a pop-under once per visit. Adsterra may use cookies and receives your IP address and browser details.",
        "Download counts come from GitHub, and fonts from Google Fonts. Your browser contacts both directly.",
        "In the app, fonts for Chinese, Japanese and Korean text load from jsDelivr the first time such text is on screen.",
        "To limit abuse, the license and ad-count services keep a short counter per IP address that expires within two minutes.",
      ],
    },
    {
      heading: "What we don't do",
      paragraphs: [
        "We don't sell data, build profiles, or run analytics on how you use Velmora. Apart from Max licenses and the anonymous ad counts above, Velmora has nothing about you on our side.",
      ],
    },
    {
      heading: "Children",
      paragraphs: ["Velmora isn't meant for children under 13."],
    },
    {
      heading: "Changes and questions",
      paragraphs: [
        "If this policy changes, the new version is posted here with a new date. Questions go to the issue tracker on GitHub.",
      ],
    },
  ],
};

export const TERMS: LegalDoc = {
  title: "Terms of use",
  intro:
    "These terms cover the Velmora builds published on GitHub, the Max license service and this website. The source code itself is licensed under GPL-3.0, and nothing here limits your rights under that license.",
  sections: [
    {
      heading: "Using Velmora",
      paragraphs: [
        "Velmora is free. You can install it, use it and share it. The GPL-3.0 license lets you read, change and redistribute the source code under its conditions.",
      ],
    },
    {
      heading: "Content",
      paragraphs: [
        "Velmora doesn't host, upload or store any movies, shows, channels or streams. It searches third-party sites and plays what they serve, the way a browser does. You're responsible for what you watch and download, and for following the law where you live.",
        "\"Where to watch\" for sports lists official broadcasters only. Live TV channels come from a public playlist that Velmora doesn't control. Some may not play.",
      ],
    },
    {
      heading: "Music and YouTube",
      paragraphs: [
        "Velmora isn't affiliated with Google or YouTube. Music uses your own YouTube account, and YouTube's terms apply to how you use it. You're responsible for that use, including downloading songs.",
      ],
    },
    {
      heading: "Velmora Public and ads",
      paragraphs: [
        "Velmora Public is the full app with ads from a third-party network. The ads are not endorsements, and we don't control what each advertiser shows. If you see an ad that looks harmful or misleading, report it on GitHub.",
      ],
    },
    {
      heading: "Velmora Max",
      paragraphs: [
        "A Max key turns ads off. Keys are given by the developer at their discretion. They are not sold, and they are not for sale or resale.",
        "A key is for the person it was given to and works on one PC at a time. A key that is shared, sold or abused can be turned off. Because Max is free, there is no refund or guarantee, and Max may stop working if the license service shuts down.",
      ],
    },
    {
      heading: "No warranty",
      paragraphs: [
        "Velmora is provided as is, without warranty of any kind, as the GPL-3.0 license states. To the extent the law allows, the developers aren't liable for any damage or loss from using it, including problems caused by third-party sites, streams or ads.",
      ],
    },
    {
      heading: "Changes",
      paragraphs: ["If these terms change, the new version is posted here with a new date. Using Velmora after that means you accept it."],
    },
  ],
};
