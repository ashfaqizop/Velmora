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
| 3 | Shell and navigation | built green, waiting for device testing |
| 4 | Accounts and onboarding | building |
| 5 | Home, Browse, title pages, search | |
| 6 | Video player | |
| 7 | Downloads | |
| 8 | Live TV, Sports, Theatre | |
| 9 | Social and the rest | |
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
  secret, opens the auth site with , and the
  site returns through  ( for
  test builds), or the short code typed by hand. The code is traded at
  . Code: .
- The refresh token is kept encrypted with an Android Keystore key
  (, outside auto-backup). A saved sign-in opens the app offline;
  the server checks it in the background.
- After a sign-in (): the account's synced settings (desktop's
  ) go into the engine's saved preferences, and the download key is
  backed up to  (), or restored from it on a
  fresh install that has no downloads yet. Max comes from .
- My List and watch progress sync arrive with their screens (phases 5 and 6);
  the server side is already desktop's .
- Onboarding (): welcome, sign in (required), look, languages
  (pushed to the account's settings), done. The sign-in step returns whenever
  the account is signed out. Music mode shows a YouTube Music sign-in gate
  () until Echo's login has a signed-in cookie.

## 5. Home, Browse, title pages, search
Home rows (including Live TV and Sports), Browse tabs (Movies, Series, Anime,
Live TV, Sports), title pages with seasons and episodes, My List, unified
search with the mode-switch shortcut.

## 6. Video player
ExoPlayer on engine sources, source picker, subtitles, audio tracks, quality,
picture-in-picture, background audio, mobile data rules, continue watching,
torrent streaming.

## 7. Downloads
`.vlm` downloads in `Movies/Velmora` through a decrypting data source, a
download service with the Wi-Fi rule, the movie and music downloads pages.

## 8. Live TV, Sports, Theatre
Channels player, matches and where to watch, Theatre lineup, booking, lobby,
chat, synced show, Max perks, start-event report.

## 9. Social and the rest
Social in a WebView, App Links (verification file on the site), deep links,
announcements, anonymous stats and privacy settings, Velmora Find / Connect /
Wrapped renames, Transfer from Echo Music (old and new backups), settings pass.

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
