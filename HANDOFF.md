# Handoff: where Velmora stands (2026-11-27, evening)

For the next session, so work carries on without re-reading history. Short
versions of the decisions live in the memory folder
(`~/.claude/projects/g--1AA-Velmora/memory/`, index in `MEMORY.md`).

## House rules

- **Nothing compiles locally except the web frontends.** No cargo, rustfmt,
  Gradle or Android Studio on this PC. Rust and Android build only on GitHub
  Actions (`app.yml` desktop, `universeapi.yml` engine, `android.yml`
  Android). Locally: `npx svelte-check` and `npm run build` in `app/` and `site/`.
- **Commit and push freely.** Never tag or publish a release unless the user says so.
- Commit author `siliconcode-dev`, with only this trailer:
  `Co-Authored-By: ashfaqizop <ashfaqizop@gmail.com>`.
- Plain-language writing everywhere (humanizer style, no AI tells).
- Never write a secret into a file, commit or memory. Secrets live in Vercel,
  Supabase or GitHub Actions secrets. Public values (Supabase URL and
  publishable key, Turnstile site key, Discord app ID) are fine in the repo.
- UI: site and desktop use `@m3e/web`; Android keeps Echo's Material 3
  Expressive look (the user asked to keep Echo's design language).
- The user eyeballs UI themselves; screenshot only to debug.
- Slow laptop and slow internet: don't leave background processes running,
  don't download big artifacts unless asked (the user often installs APKs
  themselves).

## Editing without a compiler (learned the hard way)

- Multi-line edits with backticks, `$`, `${}` or quotes: write a `.cjs` script
  in the scratchpad and run it. Inline `node -e` in bash eats backticks
  (happened twice to docs/comments this session).
- Every edit script uses exact-match replacements that throw on a miss.
- **Before writing a "new" file, check it doesn't exist.** The rebrand renamed
  Echo's strings to `res/values/velmora_strings.xml`; overwriting it wiped 1,100
  strings (caught and restored in c7c8fc5). New shell strings live in
  `velmora_shell_strings.xml`.
- After cutting Kotlin blocks, check brace balance (scratchpad `kt.cjs`,
  `matchBrace`) and scan for empty wrappers. Check every `R.drawable`/`R.string`
  referenced exists: AAPT fails on missing resources.
- Kotlin traps hit this session: `return` inside an expression-bodied function,
  calling `pointerInput` without a receiver, a local val shadowed inside
  `apply {}`, `remember` lambdas capturing stale state.
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

Read `docs/android.md` (40 decisions) and `docs/android_build_plan.md`
(**13 phases**: 1-9 build the app, 10-12 UI/UX only, 13 updates and release;
each phase has "How it is built" notes).

| Phase | State |
|---|---|
| 1 Import and rebrand | done |
| 2 Rust engine | **done, tested on the phone** (search, sources, direct + torrent playback) |
| 3 Shell and navigation | built green (run 45999722889), **not yet tested on the phone** |
| 4 Accounts and onboarding | pushed 705adb1, **CI run 46002246898 was in progress** |

What exists:

- **Engine bridge** (`android/engine/`): `rust/ffi` is a UniFFI crate wrapping
  UniverseAPI (own Cargo workspace, path dependency on the engine), JSON across
  the bridge in desktop's shapes, plus the desktop stream server ported
  (`stream.rs`, ExoPlayer plays `http://127.0.0.1:{port}/p|t|v|f/...`). Kotlin
  `Universe` gives typed coroutine calls and saved engine preferences
  (`updatePreferences`). CI builds arm64 + armv7 with cargo-ndk, generates the
  Kotlin bindings, then Gradle. Engine logs: `adb logcat -s VelmoraEngine`.
  Engine change for Android: librqbit's DHT state goes in `torrents/.dht.json`.
- **Engine lab** (hidden): About, tap the logo 7 times.
- **Shell** (`velmora/shell/`): two modes as nested nav graphs (`velmora`,
  `music`) with saved back stacks (`AppMode.kt`), Velmora tabs on Echo's
  floating toolbar, wordmark top bar (long-press switches), avatar/Downloads/
  switch pill (`ShellTopBar.kt`), the drag-to-switch panel with circular
  reveal (`ModeSwitch.kt`), Velmora quick panel, Look picker (Appearance),
  Velmora blue + Plus Jakarta Sans defaults, Instrument Serif, night splash,
  side rail on tablets. Velmora Home shows engine rows (`HomeFeed`), Search
  uses the engine; Browse/Theatre/Downloads are placeholders.
- **Accounts** (`velmora/account/`): PKCE sign-in via the auth site, which now
  accepts `?android=velmora|velmora-debug` and returns to
  `<scheme>://auth/callback` (or the short code). Refresh token encrypted with
  an Android Keystore key. After sign-in: account settings into the engine,
  vault key backup/restore (`supabase/vault.sql`, **already run by the user**),
  Max via `/api/social/max`. Onboarding gate (welcome, sign in, look,
  languages, done) and a YouTube Music gate for Music mode.
- CI artifacts per run: `velmora-android-debug` (universal, ~147 MB) and
  `velmora-android-debug-arm64` (~123 MB, the one the user installs).
- Site: `/android/` progress page; data in `site/src/lib/android.ts` (change a
  phase's `status`/`date` when it moves; phases 3 and 4 currently "building").
  The home pill says "Developing · Velmora for Android · Check progress".

Next steps:

1. Check CI run 46002246898 (phase 4). Fix any compile errors.
2. The user installs the arm64 APK; test phases 3 and 4 on the phone over adb:
   onboarding and sign-in (the browser returns via `velmora-debug://`), the
   tabs, the switch panel and reveal, the Look picker, the YouTube Music gate,
   sign out. Then mark 3 and 4 done in `docs/android_build_plan.md` and
   `site/src/lib/android.ts`.
3. Phase 5 (Home, Browse, title pages, search, My List and its sync).

Leftovers to clean up later: Listen Together internals (~4,000 lines, unused,
delete in a later phase); `SearchableSettings.kt` still lists Echo's
Instagram/X/coffee/Patreon/UPI/Discord entries; deep links into Music routes
while in Velmora mode don't switch mode; avatars show colour + initial, not
the animal drawings; desktop vault key backup not built yet.

## Testing on the phone

- The user's test phone: **Huawei STK-L21, Android 10, 64-bit** (the docs say
  "Android 9"; it's actually 10). The Android 14 phone and Android 16 tablet
  only for phases 3, 10 and 11.
- adb: `E:\Android\platform-tools\adb.exe` (USB debugging authorized). In Git
  Bash use `MSYS_NO_PATHCONV=1` for device paths. Screenshots:
  `adb exec-out screencap -p > file.png` (1080x2340; displayed images are
  scaled by about 1.17). Drive with `adb shell input tap/swipe/text/keyevent`.
  One back press closes the keyboard; a second leaves the screen.
- Debug package: `com.velmora.stepbyte.debug`; app files under
  `/sdcard/Android/data/com.velmora.stepbyte.debug/files/`.

## Waiting on the user

- Test the phase 3/4 builds (above).
- Re-run in Supabase if not done: `activity.sql`, `admin.sql`, `theatre.sql`.
  (`vault.sql` is done.)
- Max online: `node tools/license/velmora-license.mjs secret signing`, paste
  into Vercel as `LICENSE_SIGNING_KEY`, redeploy, import
  `%USERPROFILE%\.velmora-license\issued.json` on the admin Max page.
- Search Console: resubmit `sitemap.xml` (now includes `/android/`).
- Mailtrap domain for email codes. Theatre two-account test.
- Optional: a `SPORTSDB_KEY` GitHub secret (sports on Android, phase 8).

## Not yet released

Desktop changes since 0.6.6 (announcement banner, Theatre start-event report,
CSP allowing the site) ship with the next desktop release.

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
- Instagram posters: the final set is in `C:\Users\Azam\Pictures\velmora-android\`.
- The user's Instagram: @ashfaqizop and @stepbyte.studios.
- Git Bash rewrites `/CN=...`-style arguments; use `MSYS_NO_PATHCONV=1`.
- Many repo files are CRLF; normalize before exact-match edits.
