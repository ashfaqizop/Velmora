# Handoff: where Velmora stands (2026-11-28)

For the next session, so work carries on without re-reading history. Short
versions of the decisions live in the memory folder
(`~/.claude/projects/g--1AA-Velmora/memory/`, index in `MEMORY.md`).

## House rules

- **Nothing compiles locally except the web frontends.** No cargo, rustfmt,
  Gradle or Android Studio on this PC. Rust and Android build only on GitHub
  Actions (`app.yml` desktop, `universeapi.yml` engine, `android.yml`
  Android). Locally: `npx svelte-check` and `npm run build` in `app/` and `site/`.
- **Commit and push freely**, unless the user says "commit, don't push" for a
  phase (they did for phase 8, then said push). Never tag or publish a release
  unless the user says so.
- Commit author `siliconcode-dev`, with only this trailer:
  `Co-Authored-By: ashfaqizop <ashfaqizop@gmail.com>`.
- Plain-language writing everywhere (humanizer style, no AI tells).
- Never write a secret into a file, commit or memory. Secrets live in Vercel,
  Supabase or GitHub Actions secrets. Public values (Supabase URL and
  publishable key, Turnstile site key, Discord app ID) are fine in the repo.
- UI: site and desktop use `@m3e/web`; Android keeps Echo's Material 3
  Expressive look (the user asked to keep Echo's design language).
- **Each phase starts by updating the site** (`site/src/lib/android.ts`:
  phase `status`/`date`, `UPDATED`) and the table in
  `docs/android_build_plan.md`, then the code, then "How it is built" notes.
- Install on the phone only when the user says so. Download APKs only when
  asked (the user often says "download it" first, then "install it").

## Editing without a compiler (learned the hard way)

- Multi-line edits with backticks, `$`, `${}` or quotes: write a `.cjs` script
  in the scratchpad and run it. Inline `node -e` in bash eats backticks and
  breaks on apostrophes (`'` in a comment ended a single-quoted `node -e`).
- Every edit script uses exact-match replacements that throw on a miss.
- Never put a stray `cat > file` (no heredoc) in a command: it waits on stdin
  forever.
- **Before writing a "new" file, check it doesn't exist.** Echo's strings live
  in `res/values/velmora_strings.xml` (1,100 strings); new ones go in
  `velmora_shell_strings.xml`.
- Check every `R.drawable`/`R.string` and every Material icon name exists.
  `Icons.AutoMirrored.Rounded.ChevronRight` doesn't (use `KeyboardArrowRight`);
  that one name failed phase 6's first build.
- Kotlin traps seen: smart casts on properties from another module (engine
  models) don't work; `"a" + if (x) "b" else "" to true` parses wrong (wrap
  in parentheses); a private composable named `Row` shadows Compose's `Row`;
  `pointerInput` gesture detectors eat each other (pinch must only react to
  two fingers); a service started with `startForegroundService` must call
  `startForeground` even when it stops at once.
- Pushing anything under `android/` cancels the running Android CI build
  (concurrency group). Batch Android pushes.

## Stack

- `app/`: Tauri 2 + Svelte 5 desktop app (Windows). Latest release **0.6.6**.
- `universeapi/`: Rust engine. GPL-3.0.
- `site/`: Svelte + Vite site on Vercel (Hobby plan: 11 of 12 functions used),
  Upstash Redis, Supabase. Hosts: `getvelmora.dpdns.org`, `auth.`, `social.`, `admin.`.
- `supabase/*.sql`: all database code, run by the user in the SQL editor (safe to re-run).
- `android/`: Velmora for Android, built on Echo Music.

## Android (active work)

Read `docs/android.md` (decisions) and `docs/android_build_plan.md`
(**13 phases**, each with "How it is built" notes).

| Phase | State |
|---|---|
| 1 Import and rebrand | done |
| 2 Rust engine | done, tested |
| 3 Shell and navigation | done, tested |
| 4 Accounts and onboarding | done, tested |
| 5 Home, Browse, title pages, search | done, tested (offline start fixed) |
| 6 Video player | built green, on the phone, **waiting for the user's test** |
| 7 Downloads | built green, on the phone, **waiting for the user's test** |
| 8 Live TV, Sports, Theatre | built green first try (run 46121989284, `c850db2`), installed, **waiting for the user's test** |
| 9 Social and the rest | next |

Where the code is (`android/app/src/main/kotlin/com/velmora/stepbyte/velmora/`):

- `account/`: sign-in, Keystore session, sync of settings and vault key,
  onboarding gate. Offline start: the saved session decides the first screen;
  only Supabase rejecting the refresh token signs out.
- `library/VelmoraLibrary.kt`: My List and watch progress, local first, synced
  with desktop's `list_items` / `progress` tables.
- `shell/`: modes, top bar, switch panel, `HomeFeed` (cached to disk).
- `screens/`: Home, Browse (Movies, Series, Anime, Live TV, Sports tabs),
  title page, collection, My List, search, match page. `Common.kt` has routes
  (`VRoutes`) and shared cards.
- `player/`: `VideoSession` (the one ExoPlayer: sources, fallback, spot,
  next episode, live channels, Theatre clock), `VideoPlayerActivity` (PiP),
  `VideoPlayerScreen` (gestures, controls), `PlayerSheets`, `SourceFinder`
  (lookups ahead, shared with the title page), `VideoService` (media
  session), `LiveTv` (favourites, recents, region), `TheatreLayer` (lobby,
  countdown, chat, reactions, end card).
- `downloads/`: `VelmoraDownloads` (saved list, source picking, Wi-Fi rule,
  free space, seasons), `DownloadService` (foreground notification),
  Downloads page and title-page controls.
- `theatre/`: `Theatre` (lineup, tickets, clock, book/cancel/enter,
  WorkManager reminders), `Realtime` (Supabase Realtime client on OkHttp),
  `TheatreRoom` (presence, chat, reactions), `TheatreScreen` (the tab).

Engine changes for Android: librqbit DHT state in `torrents/.dht.json`; the
native HLS downloader `universeapi/crates/universeapi/src/hls.rs` (new deps
`aes`, `cbc`), used on Android and when desktop's ffmpeg is missing. The
bridge (`android/engine/rust/ffi/src/lib.rs`) now covers search, sources,
streams, TMDB/AniList lists, Live TV, Sports, downloads and local files.
Media3 is 1.8.0 (to match Jellyfin's FFmpeg audio decoder, `1.8.0+1`).

Next steps:

1. Collect the user's test findings on phases 6, 7 and 8; fix them in one
   batch; then mark those phases done (plan table and `android.ts`).
2. Phase 9: Social in a WebView, App Links (verification file on the site),
   deep links, announcements, anonymous stats and privacy settings, renames,
   Transfer from Echo Music, settings pass.
3. Not built from the Theatre spec yet: the rate-and-review card and "Move my
   ticket". The Theatre tab needs a lineup on the server to show anything.

Leftovers to clean up later: Listen Together internals (~4,000 lines,
unused); `SearchableSettings.kt` still lists Echo's social links; deep links
into Music routes while in Velmora mode don't switch mode; avatars show colour
+ initial, not the animal drawings; desktop vault key backup not built.

## Testing on the phone

- adb: `E:\Android\platform-tools\adb.exe`. Two phones have been used:
  `83CNW19C11006655` (the Huawei, Android 10) and `A9SVVB4423000743` (used
  for phases 7 and 8; it needed the USB debugging prompt accepted once).
- APKs: the scratchpad script `fetch-only.sh <run-id>` downloads the arm64
  artifact in 16 parallel pieces (about 30 s; one piece at a time is very
  slow from here) without installing. `fetch-apk.sh` also installs. Then
  `adb install -r <apk>`. The phone's Bluetooth folder is `/sdcard/bluetooth/`.
- Screenshots: `adb exec-out screencap -p > file.png`. Drive with
  `adb shell input tap/swipe/text/keyevent`. In Git Bash use
  `MSYS_NO_PATHCONV=1` for device paths.
- Debug package: `com.velmora.stepbyte.debug`. Logs:
  `adb logcat -s VelmoraEngine VelmoraAccount AccountSync VelmoraPlayer VelmoraDownloads VelmoraTheatre VelmoraRealtime`.

## Waiting on the user

- Test phases 6, 7 and 8 (the build on the phone is `c850db2`).
- Supabase is up to date (activity, admin, theatre, vault all run 2026-11-27).
- Max online: `node tools/license/velmora-license.mjs secret signing`, paste
  into Vercel as `LICENSE_SIGNING_KEY`, redeploy, import
  `%USERPROFILE%\.velmora-license\issued.json` on the admin Max page.
- Search Console: resubmit `sitemap.xml` (includes `/android/`).
- Mailtrap domain for email codes. Theatre two-account test.
- Optional: a `SPORTSDB_KEY` GitHub secret for richer sports data.

## Not yet released

Desktop changes since 0.6.6 (announcement banner, Theatre start-event report,
CSP allowing the site, the HLS fallback without ffmpeg) ship with the next
desktop release.

## Known problems

- Engine rustfmt job fails (~200 diffs); informational.
- "Android check (impersonate)" engine job always fails (BoringSSL); informational.
- Maven Central sometimes rate-limits CI (HTTP 429): just re-run the job.

## Useful facts

- Supabase URL and publishable key: `app/src/lib/account.svelte.ts`,
  `site/src/auth/config.ts`, `android/.../velmora/account/Supa.kt`.
- Site icons are a subset: add names to `ICONS` in `site/vite.config.ts`.
  New content pages go in `PAGES` there and in `src/pages/registry.ts`, then
  `node scripts/make-pages.mjs`; add them to `public/sitemap.xml`.
- The user's Instagram: @ashfaqizop and @stepbyte.studios.
- Git Bash rewrites `/CN=...`-style arguments; use `MSYS_NO_PATHCONV=1`.
- Many repo files are CRLF; normalize before exact-match edits.
