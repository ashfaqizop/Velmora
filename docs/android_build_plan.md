# Velmora for Android: build plan

Ten phases. Decisions behind them are in [android.md](android.md). Every
phase ends with a debug APK from CI, tested on the user's three devices
(Android 9 phone, Android 14 phone, Android 16 tablet) before the next one
starts. Nothing is built locally: no Gradle, Android Studio or cargo on the
developer PC.

| # | Phase | Status |
|---|---|---|
| 1 | Import and rebrand | built, waiting for device testing |
| 2 | Rust engine on Android | |
| 3 | Shell and navigation | |
| 4 | Accounts and onboarding | |
| 5 | Home, Browse, title pages, search | |
| 6 | Video player | |
| 7 | Downloads | |
| 8 | Live TV, Sports, Theatre | |
| 9 | Social and the rest | |
| 10 | Updates and release | |

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

## 10. Updates and release
In-app updater (background download, install prompt), one `app-v` tag builds
both apps into one Release, the website offers the APK, a speed and polish
pass on all three devices, release.
