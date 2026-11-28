# Velmora for Android: build plan

Thirteen phases: nine that build the app, three for UI and UX, then the release. Decisions behind them are in [android.md](android.md). Every
phase ends with a debug APK from CI, tested on the user's three devices
(Android 9 phone, Android 14 phone, Android 16 tablet) before the next one
starts. Nothing is built locally: no Gradle, Android Studio or cargo on the
developer PC.

| # | Phase | Status |
|---|---|---|
| 1 | Import and rebrand | done, tested on devices |
| 2 | Rust engine on Android | done, tested on device (search, sources, direct and torrent playback) |
| 3 | Shell and navigation | done, tested on device |
| 4 | Accounts and onboarding | done, tested on device (offline start moved to phase 5) |
| 5 | Home, Browse, title pages, search | done, tested on device (offline start fixed) |
| 6 | Video player | built, waiting for device testing |
| 7 | Downloads | built green, waiting for device testing |
| 8 | Live TV, Sports, Theatre | built green, waiting for device testing |
| 9 | Social and the rest | building |
| 10 | UI/UX I: look and motion | |
| 11 | UI/UX II: every screen size | |
| 12 | UI/UX III: polish with testers | |
| 13 | Updates and release | |

## 1. Import and rebrand
Echo Music copied into `android/` without its git history. Renamed to
`com.velmora.stepbyte` everywhere (application id, namespace, every Kotlin
package) and called Velmora. Icons from Velmora's own (adaptive, round,
Android 13 monochrome). Removed: Firebase, Google Drive backup, Listen
Together, Echo's translations, Discord/Patreon/donate links. Echo's URLs point
to Velmora's. Our CI workflow: arm64-v8a + armeabi-v7a universal APK, signed
from the `ANDROID_*` secrets, debug APK on every push to `main` touching
`android/`. Result: a working music app called Velmora.

## 2. Rust engine on Android
A UniFFI wrapper crate around UniverseAPI, built with `cargo-ndk` in CI, and a
Kotlin `:engine` module. The torrent engine and the `.vlm` vault compiled for
Android. Result: a hidden test screen that searches and resolves a stream
through Rust, proving the bridge on Android 9, 14 and 16 before anything is
built on it.

How it is built (2026-11-27):

- `android/engine/rust/ffi` is the UniFFI wrapper (crate `velmora-engine`,
  library `libvelmora_engine.so`). It is its own Cargo workspace with a path
  dependency on UniverseAPI, so the engine's own CI is untouched. Calls take
  and return JSON in the shapes the desktop frontend gets from its Tauri
  commands. Long calls report progress through an `EventSink` callback.
- The engine runs on its own Tokio runtime; a cancelled coroutine stops
  waiting and the engine finishes and caches the work, as on desktop.
- `stream.rs` is the desktop stream server ported over: ExoPlayer plays
  `http://127.0.0.1:{port}/p|t|v|f/...` (proxied hosts, torrents while they
  download, vault files decrypted on the fly, plain files).
- The vault key lives in `noBackupFilesDir/vault/key.bin`; downloads go to the
  app's external Movies folder for now (phase 7 decides the final place).
- Engine logs go to logcat: `adb logcat -s VelmoraEngine`.
- `:engine` (Kotlin) wraps the generated bindings in a typed coroutine API,
  `Universe`. Titles and sources keep the engine's exact JSON so nothing is
  lost when they go back across.
- CI builds the library for arm64-v8a and armeabi-v7a with cargo-ndk, then
  generates the Kotlin bindings from it, then runs Gradle. TMDB keys are built
  in from the repo secrets, the same as the desktop release.
- The engine lab: About, tap the logo seven times. It starts the engine, runs
  the self-test (stream server, vault round trip, torrent session, TLS), runs
  a live search, finds sources with per-provider progress, and plays one.
- Not on Android yet: the hidden-WebView solver for JavaScript hosts and the
  browser-fingerprint fallback (BoringSSL does not build for Android). Hosts
  that need them fail over to other sources.

## 3. Shell and navigation
The Velmora/Music mode system, each mode with its own back stack. Top bar
(wordmark, avatar, downloads, switch), the switch panel and its slide
animation, bottom bar Home | Browse | Theatre | Search, tablet side rail. The
two looks (Liquid Glass, Material You), Velmora fonts and colors, Android 8 to
11 fallbacks, splash. The music mini-player across modes.

## 4. Accounts and onboarding
Velmora sign-in through `auth.` with the return to the app, required YouTube
Music sign-in for Music, onboarding (welcome, sign in, personalize, pick the
look), sync (My List, progress, shared settings), Max restore, vault key backup.

How it is built (2026-11-27):

- Sign-in is desktop's flow with a different way back: the app makes a PKCE
  secret, opens the auth site with `?android=velmora&state=&challenge=`, and the
  site returns through `velmora://auth/callback?code=&state=` (`velmora-debug` for
  test builds), or the short code typed by hand. The code is traded at
  `/api/auth/exchange`. Code: `velmora/account/VelmoraAccount.kt`.
- The refresh token is kept encrypted with an Android Keystore key
  (`SecureStore`, outside auto-backup). A saved sign-in opens the app offline;
  the server checks it in the background.
- After a sign-in (`AccountSync`): the account's synced settings (desktop's
  `PREF_KEYS`) go into the engine's saved preferences, and the download key is
  backed up to `vault_keys` (`supabase/vault.sql`), or restored from it on a
  fresh install that has no downloads yet. Max comes from `/api/social/max`.
- My List and watch progress sync arrive with their screens (phases 5 and 6);
  the server side is already desktop's `supabase/sync.sql`.
- Onboarding (`Onboarding.kt`): welcome, sign in (required), look, languages
  (pushed to the account's settings), done. The sign-in step returns whenever
  the account is signed out. Music mode shows a YouTube Music sign-in gate
  (`YouTubeMusicGate.kt`) until Echo's login has a signed-in cookie.

## 5. Home, Browse, title pages, search
Home rows (including Live TV and Sports), Browse tabs (Movies, Series, Anime,
Live TV, Sports), title pages with seasons and episodes, My List, unified
search with the mode-switch shortcut.

Carried over from phase 4 testing: with no network the app stayed on the
sign-in / Get started screen for good, and on a slow network that screen
flashed for a few seconds before going away. Once someone has signed in, the
app must open straight into the library offline or on a slow connection. The
saved session decides what shows at launch; the server check runs in the
background and only signs out when the server really rejects the session,
never because it couldn't be reached.

How it is built (2026-11-27):

- Offline start: `VelmoraAccount.init` builds the signed-in state from what
  is saved (profile, or at least the user id) before any network call. Only
  a 400/401 from Supabase that names the refresh token signs out; timeouts,
  proxies and server errors keep the saved session, and a network callback
  checks again when the phone is back online. `OnboardingGate` reads
  "onboarding finished" from DataStore before its first frame; the old gate
  started from the flow's default (`false`), which is what showed
  "Get started".
- Home (`velmora/screens/VelmoraScreens.kt`): the spotlight is a
  `HorizontalPager` with parallax art, stretching page dots and auto-advance;
  then collection chips, Continue watching, My List, Live now (scores
  refreshed each minute) and every engine row with See all. `HomeFeed` saves
  the last good rows to `files/velmora/home.json`, so Home has posters
  offline and says so.
- Browse (`BrowseScreen.kt`): tabs in a pager. Movies and Series are TMDB
  `/discover` (or `/trending`) with genre chips and Popular / Trending / Top
  rated / New, paged as you scroll. Anime is AniList with genres and
  Popular / Trending / This season / Top rated / Airing. Live TV lists
  iptv-org channels by category and name (they play in phase 8). Sports shows
  live, upcoming by day and recent results. Catalogs live for the whole run,
  so a title page and back keeps the scroll and the list.
- Title page (`TitleScreen.kt`): the title travels in the route as the
  engine's JSON, so the poster's data shows at once and survives process
  death; the engine's detail fills in seasons and episodes, cast with
  photos, gallery, details and More like this. Play waits for phase 6.
- My List and progress (`velmora/library/VelmoraLibrary.kt`): saved on the
  phone first, synced with desktop's `list_items` / `progress` tables and
  `sync_push_*` functions, newest edit wins, removals as deleted rows.
- The bridge gained `tmdb_list`, `tmdb_raw`, `tmdb_titles`, `anilist_find`,
  `tv_channels`, `tv_categories`, `tv_countries`, `sports_live`,
  `sports_upcoming` and `sports_results`.

## 6. Video player
ExoPlayer on engine sources, source picker, subtitles, audio tracks, quality,
picture-in-picture, background audio, mobile data rules, continue watching,
torrent streaming.

How it is built (2026-11-28), in `velmora/player/`:

- `VideoSession` owns the one ExoPlayer for the process and everything about
  a playback: what to play (`targetFor`: the saved episode, else the first),
  the lookup, picking a source, falling over, the saved spot and the next
  episode. `VideoPlayerActivity` only draws it, so rotating, PiP and the
  background service never restart the video.
- `SourceFinder` is desktop's finder: one shared lookup per title, target and
  sub/dub, started by the title page as soon as the detail has the ids
  (`lookupTitle` merges them in), reused for 10 minutes. The player joins
  the running lookup. While it runs, each site shows as a chip; "Play the
  best so far" skips the slow ones.
- Picking: best ranked first, skipping tried ones. On mobile data, torrents
  ask first (or are skipped, or allowed: `PlayerPrefs.MobileData`) and
  "Save data" puts 720p sources first. A source that never plays is
  forgotten (`forget_sources`) and the next starts; one that drops mid-film
  is reopened once at the same spot.
- The player: Media3 1.8 with Jellyfin's FFmpeg audio decoder (AC3, E-AC3,
  DTS, TrueHD on phones without them), HLS and DASH, long HTTP timeouts and a
  bigger buffer for torrents, audio focus (music pauses), the account's audio
  and subtitle languages as the first choice. External subtitles from the
  source are added as side-loaded tracks.
- Controls in Compose over a bare `PlayerView`: double-tap skip that adds up,
  drag to seek with a preview, brightness left and volume right, two-finger
  pinch to zoom, lock, fit/zoom/stretch, rotate. Sheets for quality,
  subtitles and their size, audio, speed, sources, episodes and sub/dub.
- PiP: auto-enter on Android 12+, on leaving before that, with back,
  play/pause and forward actions. `VideoService` (a `MediaSessionService`)
  gives lock-screen controls and, with "Keep playing in the background" on,
  sound with the screen off.
- Progress: saved every 15 s, on pause and on close into `VelmoraLibrary`,
  which syncs it with desktop. The next episode shows for the last 40 s and
  plays after an 8 s countdown when the episode ends.

## 7. Downloads
`.vlm` downloads in `Movies/Velmora` through a decrypting data source, a
download service with the Wi-Fi rule, the movie and music downloads pages.

How it is built (2026-11-28):

- The engine's download manager does the transfer and seals each file into
  a `.vlm` vault file with the install's key (the bridge now hands the
  stream server's vault to `downloads().set_vault`). The key is backed up to
  the account since phase 4, so a reinstall still opens old downloads.
- HLS without ffmpeg: Android has no ffmpeg sidecar, so the engine gained
  `hls.rs`, a native joiner. It takes the best variant, fetches segments four
  at a time and writes them in order, decrypts AES-128 ones (RustCrypto `aes`
  and `cbc`), and resumes from a `.part.idx` mark after a pause or a dropped
  connection. It refuses what it can't do (live, audio in its own playlist,
  SAMPLE-AES, byte ranges) in plain words. Desktop uses it too when its
  ffmpeg is missing. DASH still needs ffmpeg, so those sources only stream.
- `velmora/downloads/VelmoraDownloads.kt` keeps the list in
  `files/velmora/downloads.json`, so a download outlives the app: after a
  restart it goes back to the engine with the same source and continues from
  its `.part` file. It picks the source (best ranked at or under the chosen
  quality, direct files before torrents, no DASH), falls over to the next
  source twice, checks free space with `StatFs`, and follows the Wi-Fi rule:
  on mobile data, running jobs pause and new ones wait, then carry on when
  Wi-Fi is back. Whole seasons queue one record per episode.
- `DownloadService` is a `dataSync` foreground service with one progress
  notification (and "Pause all"); it always reaches the foreground before
  stopping, so Android never kills the app for a late `startForeground`.
  A finished download posts its own notification that opens Downloads.
- Offline play: `VideoSession` checks `VelmoraDownloads.localFile` first and
  plays the file through `open_local` (vault files decrypt on the fly from
  `/v/`), with subtitles saved beside it, no lookup and no network.
- UI: the title page has a download button whose icon shows the state (ring
  with progress, paused, done) and a sheet for quality, Wi-Fi only, this
  episode or the whole season; episode rows have one-tap download buttons.
  The Downloads page shows Velmora's share of the storage, active downloads
  with wavy progress, speed and time left, then what's on the phone: movies,
  and shows grouped with their episodes, with how much was watched.

## 8. Live TV, Sports, Theatre
Channels player, matches and where to watch, Theatre lineup, booking, lobby,
chat, synced show, Max perks, start-event report.

How it is built (2026-11-28):

- Live TV plays in the same player: `VideoSession.playChannel` makes the
  channel's streams (bridge `tv_sources`) the sources, starts at the live
  edge, and moves to the next stream when one drops or ends. The controls
  show LIVE (and Go live when the stream has drifted back) instead of a seek
  bar, plus a favourite heart. `player/LiveTv.kt` keeps favourites, the
  channels watched lately and the country, all on the phone. Browse's Live
  TV tab gained a country picker, Favourites and Watched lately rows, and a
  heart on every card.
- Sports: every match card opens `MatchScreen`: teams, the score refreshed
  while it's on (`sports_lookup`), kickoff, venue, and Where to watch for
  the chosen country (`sports_watch`); a free channel Velmora has plays at
  once, the rest open the broadcaster.
- Theatre (`velmora/theatre/`), the same tables and functions as desktop
  and the web (`supabase/theatre.sql`):
  - `Theatre` keeps the lineup, your tickets, people you follow going, the
    server clock (`theatre_now`, best of three), book / cancel / enter with
    the server's refusals in plain words, notifications when a held
    screening is cancelled or moved or a waitlist seat frees up, and
    WorkManager reminders when the lobby opens and 5 minutes before.
  - The anonymous source checks of the spec (`Probes`): a few drafted
    titles, on Wi-Fi, while nothing plays, each once.
  - `Realtime` is a small Supabase Realtime client on OkHttp's WebSocket
    (Phoenix protocol: join with the access token for private channels,
    heartbeats, rejoin with backoff, broadcast, presence). `TheatreRoom`
    joins `theatre:<id>` (presence, reactions) and `theatre-chat:<id>`
    (listen only; `theatre_say` sends), keeps 200 messages in memory,
    mentions, mute, chat mode changes, and quiet mode beyond the room's size.
  - The player: `playTheatre` loads the film in the lobby and holds it at
    0:00 (ExoPlayer buffers ahead), then everyone starts together at the
    server's start time; late joiners start at the live point. It follows
    live (more than 6 s off snaps back); a pause falls behind, shown as
    "2:14 behind · Back to live"; seeks can't pass live; a failed source
    resumes at live, and with none it keeps looking once a minute while you
    stay in the lobby with the chat.
  - Screens: the Theatre tab (featured live or next screening, lineup by
    day, My tickets, seats / open / sold out, Tonight's pick, people you
    follow going, the one right button), Home's Tonight in Theatre row, a
    chip on title pages, the lobby (drifting backdrop, countdown, Ready with
    minutes loaded ahead, who's here, chat), the last five seconds on
    changing shapes, chat over the film / beside it / hidden, floating
    reactions, and the end card ("You were there", chat until it closes).
  - Links: `velmora://theatre/<id>` (Velmora Social's Open in Velmora) and
    the reminders open the screening.

## 9. Social and the rest
Social in a WebView, App Links (verification file on the site), deep links,
announcements, anonymous stats and privacy settings, Velmora Find / Connect /
Wrapped renames, Transfer from Echo Music (old and new backups), settings pass.

How it is built (2026-11-28):

- Velmora Social (`velmora/social/SocialScreen.kt`): the website in a WebView
  with a native dark bar (back through the page's history, reload, share, open
  in browser), a loading screen instead of a white flash, an offline screen,
  and file picking for profile pictures. Signing in: the app never shares its
  refresh token (Supabase rotates them, two holders sign each other out).
  `SocialSession` asks `/api/auth/handoff` with `mint: true` for a second
  session of the same account (made on the server from a one-time sign-in
  link that's never emailed), and the page redeems the code with the PKCE
  secret from the address fragment (`site/src/lib/session.ts`). The page then
  keeps its own session; if it ever asks to sign in again the app hands over
  again. Signing out of the app clears the WebView's storage.
- Links (`VelmoraLinks`): `velmora://title|music|profile|theatre/...`,
  `getvelmora.dpdns.org/open/?to=...` and Social pages (`/@name`, `/title/`,
  `/list/`, `/theatre`, `/trending`, `/leaderboards`). The https ones are
  verified App Links (`site/public/.well-known/assetlinks.json`, release and
  debug packages, one signing key). Titles open the title page, songs play in
  Music mode, profiles and Social pages open the Social screen, screenings
  open Theatre. The website's sign-in now always returns to `/social/`, so a
  return address never looks like a profile link to Android.
- Announcements (`velmora/notices/`): the same Supabase rows as desktop,
  every 10 minutes, kept for offline, maintenance first and not dismissable,
  shown as a card at the top of Home.
- Privacy (`velmora/privacy/`): `UsageStats` counts like desktop (titles,
  sites, player `exo`, quality, songs, hours) and sends finished days to
  `/api/stats/report`; a one-time card on Home says so. `ActivityPoster`
  posts watching / finished / added to My List to Social (`post_activity`),
  queued offline, never 18+, only with the account's sharing on.
  `WatchReports` turns the player's tick into "a minute in" and "95%".
- Settings: a Velmora group at the top of Settings (Account and Social,
  Watching, Downloads, Theatre, Privacy, Transfer from Echo Music), then Music
  and App. Velmora's pages are in `velmora/settings/`.
- Transfer from Echo Music (`EchoTransfer`): reads Echo's backup zip (or a
  bare `song.db`), shows what's in it (songs, liked, playlists, albums,
  artists, plays), refuses a newer database, then copies the database and
  merges the settings file entry by entry (Echo's music settings in,
  Velmora's own keys kept, Echo package paths moved), and restarts.
- Renames: song recognition is Velmora Find. The Theatre lobby's Leave button
  has its own row (it sat on the "Lobby open" badge in landscape).

## 10. UI/UX I: look and motion
Phases 3 to 9 make every screen work; these three make them feel like
Velmora. This one is the design language: Material 3 Expressive shapes and
motion throughout, shared-element transitions (poster to title page, mini
player to full player), the Velmora/Music switch animation finished, Liquid
Glass tuned so both looks are equally good, Velmora's fonts and type scale,
per-title color taken from the poster, haptics, and loading, empty and error
states with skeletons instead of spinners.

## 11. UI/UX II: every screen size
Phone, tablet and foldable layouts designed on purpose rather than
stretched: list-detail panes, a two-column title page, a landscape player
with its own controls, the side rail on tablets. Edge to edge with gesture
navigation, Android 8 to 11 looking as finished as 16, keyboard and mouse on
tablets, and accessibility: TalkBack labels, large font scales, contrast and
touch targets.

## 12. UI/UX III: polish with testers
Every screen walked through on the three test devices and fixed:
micro-interactions, the details nobody lists, jank and startup time
(baseline profiles), a copy pass in plain language, the onboarding refined
from what testers stumble on, and the screenshots for the website.

## 13. Updates and release
In-app updater (background download, install prompt), one `app-v` tag builds
both apps into one Release, the website offers the APK, a last speed check on
all three devices, release.
