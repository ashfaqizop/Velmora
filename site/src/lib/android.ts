// Velmora for Android: what the app has, how to install it and what the next
// updates bring. Used by /android/ and the home page. The build plan, phase by
// phase, is in docs/android_build_plan.md.

/** SHA-256 of the signing certificate every APK carries. */
export const SIGNING_SHA256 = "D7:55:CF:03:86:08:B0:5A:D2:4C:20:2E:8D:6B:6A:A6:D8:58:2B:E4:21:E7:58:92:06:0D:CD:EB:A2:AB:A6:E2";

export const FEATURES = [
  { icon: "search", shape: "soft-burst", color: "#8fb4ff", title: "One search, every source", text: "Movies, series, anime and Asian drama from many sites at once, with the best link picked for you." },
  { icon: "play_circle", shape: "12-sided-cookie", color: "#3271fe", title: "A real player", text: "Source picker, subtitles, audio tracks, quality, gestures, picture-in-picture, and cinema sound on every phone." },
  { icon: "download", shape: "pill", color: "#3ddc84", title: "Offline downloads", text: "Whole seasons, kept encrypted, with a Wi-Fi only rule. They keep going in the background." },
  { icon: "theaters", shape: "flower", color: "#ff7a59", title: "Theatre", text: "Everyone watches the same film at the same moment, with a lobby, a countdown and live chat that's never saved." },
  { icon: "sports_soccer", shape: "4-leaf-clover", color: "#ffb74d", title: "Live TV and sports", text: "Live channels that switch stream when one drops, match pages, and where to watch in your country." },
  { icon: "library_music", shape: "puffy-diamond", color: "#f8e8c8", title: "Music", text: "A whole YouTube Music app built in, with lyrics, downloads and Velmora Find. Bring your library over from Echo Music." },
  { icon: "groups", shape: "9-sided-cookie", color: "#c9c1ff", title: "Velmora Social", text: "Your profile, friends, lists and reviews, inside the app and already signed in." },
  { icon: "sync", shape: "sunny", color: "#8ff0b8", title: "Synced with desktop", text: "One account: My List and watch progress follow you between your PC and your phone." },
];

export const STEPS = [
  { title: "Download the APK", text: "Pick the 64-bit file. Nearly every phone from the last several years is 64-bit; the universal file is for older ones." },
  { title: "Allow the install", text: "Open the file. Android asks you to let your browser or file app install apps: turn it on for this once." },
  { title: "Install", text: "If Play Protect says it hasn't seen the app, choose Install anyway. Velmora isn't on the Play Store, so this is expected." },
  { title: "Sign in", text: "Your free Velmora account, the same one as desktop: Google, Discord or an emailed code. After that, Velmora opens offline too." },
];

export const ROADMAP = [
  { icon: "palette", when: "Update 1", title: "Look and layout", text: "One design language across every screen, colors taken from each poster, and tablet layouts made on purpose." },
  { icon: "animation", when: "Update 2", title: "Transitions and motion", text: "Posters that grow into title pages, a mini player that opens into the full one, springs everywhere. Nothing jumps." },
  { icon: "auto_awesome", when: "Update 3", title: "Polish", text: "The details testers find: smoothness, startup time, accessibility and wording." },
];

export const FAQ = [
  { q: "Is it free?", a: "Yes, free and open source like desktop Velmora. The Android app has no ads at all." },
  { q: "Which phones and tablets?", a: "Android 8 and newer, phones and tablets, on 64-bit and 32-bit ARM chips." },
  { q: "Is it on the Play Store?", a: "No. You get it here as an APK, and it updates itself from inside the app." },
  { q: "Does it have everything desktop has?", a: "Almost. Search, the player, downloads, live TV, sports, Theatre, Social and Music are all in. Ani, the assistant, is desktop only." },
  { q: "Is it finished?", a: "It works and it's tested, and the design isn't done yet. The next updates are about how it looks and moves." },
];
