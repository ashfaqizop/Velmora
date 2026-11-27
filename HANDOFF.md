# Handoff: where Velmora stands (2026-11-27)

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
- UI uses `@m3e/web` (site on 2.13.7, desktop app on 2.10.3). Admin pages use
  the full width with 12px gaps.
- The user eyeballs UI themselves; screenshot only to debug.
- Slow laptop: don't leave background processes running.

## Stack

- `app/`: Tauri 2 + Svelte 5 desktop app (Windows). Latest release **0.6.6** (`app-v0.6.6`).
- `universeapi/`: Rust engine (sources, resolving, metadata, torrents, `.vlm` vault). GPL-3.0.
- `site/`: Svelte + Vite multi-page site on Vercel, functions in `site/api/`
  (Hobby plan: **11 of 12 functions used**), Upstash Redis, Supabase.
  Hosts: `getvelmora.dpdns.org`, `auth.`, `social.`, `admin.` (rules in `site/vercel.json`).
- `supabase/*.sql`: all database code; the user runs files in the SQL editor
  (all safe to re-run).
- `android/`: Velmora for Android, built on Echo Music (see below).

## Recently done (all pushed)

- **Admin site, all 3 phases** (`docs/admin.md`): overview, people, reports,
  reviews, names, audit; Theatre ops; Max licences online, announcements,
  app ads (`/ads.json` now served from Redis `cfg:ads`), remote config
  (`registry.json` on the `config` branch via `CONFIG_GITHUB_TOKEN`),
  analytics, curation (featured people, staff picks, leaderboard hiding),
  Theatre Tuning (`cfg:lineup` knobs read by `api/_lib/lineup.js`) and Insights.
- **Site**: 8 content pages + 6 comparison pages (prerendered), SEO basics,
  SmartScreen animation on `/docs/install/`, Android copy now says "in
  production development, drops soon".
- **Android phase 1** (below).

## Android (active work)

Read `docs/android.md` (40 decisions) and `docs/android_build_plan.md` (10 phases) first.

- Echo Music (Kotlin/Compose, GPL-3.0) imported into `android/`, renamed to
  `com.velmora.stepbyte` everywhere (packages, JNI, C++ namespaces). Version
  0.6.6 / code 6006 (code = major*1e6 + minor*1e3 + patch, matches desktop).
- Removed: Firebase, Echo's translations, Echo's services (canvas server,
  320 kbps extension, share domain), donation/social links, 14 alternate icons
  and the picker, Android TV entry, Listen Together's UI and networking.
  **Listen Together's internals (~4,000 lines) are still in the code, unused:
  delete them in phase 3** when the player and menus are reworked.
- Echo's updater is off (`UPDATES_ENABLED = false` in
  `velmora/updater/velmoraupdater.kt`) until phase 10.
- Icons: fox launcher PNGs from `app/src-tauri/icons/android`, plus a flat fox
  glyph (`drawable/ic_velmora_glyph.xml`, also notification, QS tile,
  `ic_launcher_nobg`).
- About screen credits Echo Music; `android/README.md` too. Keep the credit.
- CI: `.github/workflows/android.yml` builds `:app:assembleUniversalGmsDebug`
  on pushes touching `android/`, signed with the release key, artifact
  `velmora-android-debug` (about 135 MB debug).
- Secrets in GitHub: `ANDROID_KEYSTORE_BASE64`, `ANDROID_KEY_ALIAS`,
  `ANDROID_KEY_PASSWORD`, `ANDROID_STORE_PASSWORD` (backup copy only in
  `%USERPROFILE%\.velmora-android\`), `LASTFM_API_KEY`, `LASTFM_SECRET`.
  Discord app ID `1533360080928378930` is in `app/build.gradle.kts`
  (redirect and Public Client set by the user).
- **Status:** phase 1 built green (run 45983475036). Waiting for the user to
  test on Android 9, Android 14 and an Android 16 tablet. Next is **phase 2**:
  UniverseAPI through UniFFI + cargo-ndk in CI, a Kotlin `:engine` module and a
  hidden test screen.
- Editing Kotlin without a compiler: scripts in the session scratchpad cut
  whole blocks by brace matching. After any cut, scan for empty wrappers like
  `add(\n)` (one slipped through once). Check resources references too:
  AAPT fails on a missing `@mipmap`/`@drawable`.

## Waiting on the user

- Re-run in Supabase, in order: `supabase/activity.sql`, `supabase/admin.sql`,
  `supabase/theatre.sql` (admin phase 3a/3b tables: announcements, featured,
  leaderboard_hidden, admin_growth, theatre_tickets.via, admin_theatre_stats).
- Max online: `node tools/license/velmora-license.mjs secret signing` (tool is
  local, gitignored), paste into Vercel as `LICENSE_SIGNING_KEY`, redeploy,
  then import `%USERPROFILE%\.velmora-license\issued.json` on the admin Max page.
- `CONFIG_GITHUB_TOKEN` is set (user did it on 2026-11-27).
- Search Console: resubmit `sitemap.xml`. Mailtrap domain for email codes.
  Theatre two-account test.
- Android phase 1 device testing (see above).

## Not yet released

The desktop app changes since 0.6.6 (announcement banner, Theatre start-event
report, CSP allowing the site) ship with the next desktop release. The admin
failure-rate and app-vs-Social numbers fill in after that.

## Known problems

- Engine rustfmt job fails (~200 diffs since 0.5.0); a `workflow_dispatch`
  job that runs `cargo fmt` and commits was proposed, not built.
- "Android check (impersonate)" engine job always fails (BoringSSL); informational.

## Useful facts

- Supabase publishable key and URL: `app/src/lib/account.svelte.ts`,
  `site/src/auth/config.ts`, `site/api/_lib/config.js`.
- Site Material Symbols are a subset: add new names to `ICONS` in
  `site/vite.config.ts` (check with the Google Fonts css2 URL). New pages go
  into `PAGES` there and into `src/pages/registry.ts`; run
  `node scripts/make-pages.mjs` after editing the registry.
- Content pages are prerendered (`npm run build` = vite build + `scripts/prerender.mjs`).
- Instagram posters: HTML at `<scratchpad>/posters/` (1080x1350, rendered with
  headless Chrome: `chrome.exe --headless=new --screenshot --window-size=1080,1350
  "file:///.../android.html?s=N"`). Final Android set exported to
  `C:\Users\Azam\Pictures\velmora-android\1..5-velmora-android.png`. The
  scratchpad can be cleared; the exports are the durable copies.
- User's Instagram: @ashfaqizop and @stepbyte.studios.
- Git Bash rewrites `/CN=...`-style arguments into paths; use `MSYS_NO_PATHCONV=1`.
- Many repo files are CRLF; normalize (`\r\n` → `\n`) before exact-match edits.
