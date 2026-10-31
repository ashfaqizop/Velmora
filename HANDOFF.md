# Velmora handoff

Last updated 2026-10-31, while v0.1.5 was going through CI. Read this before starting v0.2.0 work.

## Where things stand

- The **UniverseAPI engine** (`universeapi/`, Rust) is at v1.0.0 plus app-driven changes. These include the download folder preference, vault files and the download library.
- The **Velmora app** (`app/`, Tauri 2 + Svelte 5 + @m3e/web) shipped `app-v0.1.0`, and v0.1.5 was pushed as commits `39aeb71` and `3a44a0e`.
- v0.1.5 is released (`app-v0.1.5`, with notes).
- **v0.2.0 is Ani Intelligence**, an in-app Gemini assistant (BYOK). The spec, tool list and the 8-step build plan are in `docs/ani-intelligence.md`. Work goes two steps per commit and push; never tag a release without the user's explicit go-ahead.

### What v0.1.5 changed (so you don't redo it)

- **Sliders:**
  - `app/src/lib/slider.ts` is a Svelte action that takes over pointer dragging for every `m3e-slider`.
  - m3e-slider caches its geometry and drew the thumb from stale numbers, so the first press "flew" 100 px.
  - Use `use:slider` on any new slider. Seek bars pass `{ onstart, onend }` to commit on release.
- **Download folder:** set in Settings → Downloads and in the fourth onboarding step (`app/src/components/DownloadFolder.svelte`).
  - Stored as `Preferences.download_dir`.
  - The engine switches live (`DownloadManager::set_dir`).
  - `set_download_dir` checks the folder is writable.
- **Vault downloads:** new downloads are `.vlm` files.
  - The format is in `universeapi/src/vault.rs`: header `VELMORA1` + nonce + original extension, then ChaCha20 ciphertext.
  - The key is `vault.key` in the app local data dir, made on first launch (`stream::load_key`).
  - The loopback server serves `/v/{token}` decrypted, with Range support. Tokens come from the `open_local` command.
  - Plain files from 0.1.0 still play.
- **Downloads page:**
  - lists everything on disk under "On this PC" (`DownloadManager::library`)
  - re-downloads become "Name (2)" (`claim_target`), and the dialog warns when you already have the title (`existing`)
  - cancel deletes partials, and orphans are cleaned at launch
  - "Clear finished" keeps failed jobs
- **Player:** `aid` and `sid` are set as strings. Numbers made mpv fail with "error accessing property".

## Rules the user set (follow every one)

- **No cargo on this PC.** No `cargo build/check/test/clippy` locally; Rust compiles only in GitHub CI. Running `rustfmt` on a file is allowed, but don't use it to reformat files: its local version disagrees with the committed style, and CI's rustfmt step is non-blocking anyway.
- **Commit identity:**
  - Author `siliconcode-dev <siliconxcode@gmail.com>` (repo-local config).
  - The only trailer is `Co-Authored-By: ashfaqizop <ashfaqizop@gmail.com>`. No Claude trailer.
  - Push through `gh` as `ashfaqizop`.
- **Commit, push and release only when the user says so.** Batch related fixes into one push. A small follow-up commit for a red build is fine.
- **Writing style:** plain "humanizer" voice in replies, commits, docs and UI copy. No em dashes, no "not X but Y", no bold-label lists, no closing offers. The user invokes `/humanizer` for docs.
- **@m3e/web everywhere**, used properly: real components, not lookalikes. Fonts are Instrument Serif (headings) and Plus Jakarta Sans (text). The seed colour is `#6015EA`, vibrant, dark.
- **Credit UniverseAPI** wherever sources show up: the "Powered by UniverseAPI" mark is on the splash, the sidebar card, the search status card, About and the player snackbar. Never name the source sites in marketing.
- **Ask before running a dev server or a browser** (headless or not). The user said yes once, for the slider repro.
- **Never commit:**
  - `ani-cli/`, `MovieBox-TUI/`, `freeflix-cli/`, `custom api/`
  - `video-work/`, `/ffmpeg.exe`, `/ffprobe.exe`
  - `.env` files, secrets, `app/slider-test/`
  - `HANDOFF-slider.md`, `TEST-REPORT.md`, `test-shots/` (local-only scratch, excluded in `.git/info/exclude`)

## Repo map

```text
Velmora/
├── app/                               the desktop app
│   ├── src/                           Svelte 5 frontend
│   │   ├── main.ts, app.css           fonts, theme, global helpers (.on-shape, .display)
│   │   ├── lib/api.ts                 typed invoke() wrappers for every command
│   │   ├── lib/types.ts               mirrors of engine/app Rust types
│   │   ├── lib/state.svelte.ts        nav, feed, library, downloads, settings, network stores
│   │   ├── lib/slider.ts              the m3e-slider drag fix (use:slider)
│   │   ├── lib/m3e.ts                 every @m3e/web module the app registers
│   │   ├── components/                Splash, TopBar, Sidebar, Hero, Rail, Poster, Onboarding, DownloadFolder, ...
│   │   └── pages/                     Home, Search, Title, Player, Downloads, Settings, Grid
│   └── src-tauri/
│       ├── src/lib.rs                 setup: engine, vault key, loopback server, plugins, command list
│       ├── src/commands.rs            every #[tauri::command]
│       ├── src/stream.rs              loopback HTTP server for mpv (/p, /t via engine proxy; /v vault files)
│       ├── src/runtime.rs             first-launch download of mpv DLLs + ffmpeg (release runtime-v1)
│       ├── src/prefs.rs, sync.rs      preferences file; AniList (implicit) and Trakt (PKCE) sign-in
│       ├── tauri.conf.json            window, CSP, updater pubkey + endpoint
│       └── capabilities/              default.json, desktop.json
├── universeapi/                       the engine (Rust workspace; see universeapi/README.md)
├── .github/workflows/app.yml          frontend check, Windows build; on app-v* tags, signed release
├── .github/workflows/universeapi.yml  engine tests, clippy, Android checks, live smoke test
├── docs/universe-api-engine.md
└── README.md, CONTRIBUTING.md, CODE_OF_CONDUCT.md, SECURITY.md, LICENSE (GPL-3.0-or-later)
```

## How things fit together

- **Startup:** `Splash.svelte` checks for an update (tauri-plugin-updater), installs the runtime on first launch, refreshes the provider registry, then shows the shell.
- **The window:** borderless and transparent. It starts hidden, then maximizes and shows on first paint.
- **Playback:** libmpv is embedded through `tauri-plugin-libmpv` (wid mode).
  - While the player is open, the page turns transparent (`html.player-active`) so mpv shows through.
  - Streams go through the loopback server on `127.0.0.1:<random port>`.
  - If a source fails, the player moves to the next one at the same timestamp.
- **Updates:** a tag `app-vX.Y.Z` makes `app.yml` build, sign and publish the NSIS installer, `.sig` and `latest.json`. Installed apps pick it up on launch.
  - The updater private key is the repo secret `TAURI_SIGNING_PRIVATE_KEY`.
  - A local copy was in an old session scratchpad. The user was asked to back it up.
- **Build-time secrets** (GitHub secrets, injected by `app.yml`): `TMDB_READ_TOKEN`, `TMDB_API_KEY`, `ANILIST_CLIENT_ID`, `TRAKT_CLIENT_ID`. They're never written to the repo.

## Releasing

1. Bump the version in `app/package.json`, `app/package-lock.json` (two places) and `app/src-tauri/Cargo.toml`. `tauri.conf.json` reads it from package.json.
2. Commit, push, and wait for both workflows to go green (`gh run list`, `gh run watch <id> --exit-status`).
3. Tag and push: `git tag -a app-vX.Y.Z -m "Velmora vX.Y.Z" && git push origin app-vX.Y.Z`.
4. When the tag run finishes, check `gh release view app-vX.Y.Z` lists the installer, `.sig` and `latest.json`. Then replace the placeholder notes:
   `gh release edit app-vX.Y.Z --notes-file notes.md`
   Write the notes in the humanizer voice, the way the v0.1.0 and v0.1.5 notes are written.

## Testing without cargo

- **Frontend:** `cd app && npm run check` (svelte-check). It must stay at 0 errors and 0 warnings.
- **Rust:** compiles only in CI. Read your own code for clippy lints before pushing; CI runs `clippy -D warnings`. In v0.1.5 it caught `unnecessary_sort_by`.
- **Engine tests:** run in CI on Ubuntu and Windows. New engine code needs unit tests. For examples, see the tests at the bottom of `download.rs` and `vault.rs`.
- **The real app:** the user installs the CI build and reports bugs one at a time, often with screenshots.
- **Computer use:** a Claude desktop session can drive the installed app (there's a Velmora shortcut on the desktop). It was flaky on the first try. To use one as a tester, write a brief like `HANDOFF-slider.md`.
- **Slider repro kit (local only):**
  - `app/slider-test/` (serve with `npx vite slider-test --port 5179`)
  - a Chrome DevTools-protocol driver that was in the old session scratchpad (`drag.mjs`). Recreate it if needed: launch Chrome with `--remote-debugging-port`, send `Input.dispatchMouseEvent`, read the thumb's `getBoundingClientRect`.

## Gotchas

- **Bash heredocs collapse backslashes.** `"\r\n"` written through a heredoc came out as real line breaks inside a Rust string. Use the Write/Edit tools for anything with escapes, or write a Python script file with Write and run it.
- **Line endings are mixed.** `core.autocrlf=true`, and some files are CRLF in the working tree. When patching with Python, read with `newline=""`, normalise to LF for matching, then write back in the original style.
- **The editor can write its buffer over a file you changed from the shell.** Re-check if an edit seems to vanish.
- **A new push to `main` cancels the running CI on `main`.** If you need a run's result, wait before pushing again.
- **Svelte 5 delegates `pointerdown`, `input`, `change` and similar events to the root.** Calling `stopPropagation` in a capture listener on an element stops delegated Svelte handlers on it too. That's why the slider action takes `onstart` and `onend` callbacks.
- **m3e quirks:**
  - `m3e-slider` has `min-inline-size: 200px` unless `--m3e-slider-min-width` is set.
  - `m3e-card` paints its own container colour, so art goes inside the slotted content.
  - Content slotted into `m3e-shape` isn't centred; use the `.on-shape` grid helper and give the icon `z-index: 1`.
  - Set `--m3e-shape-container-color` on the shape itself.
  - The engine omits empty Vecs in JSON (`skip_serializing_if`). The frontend normalises them in `api.detail`.
- **All public engine futures must stay `Send`.** A compile-time test in `engine.rs` enforces it.

## Open loose ends

- **Seeding:** with vault files on, a finished torrent is sealed and its plain file deleted, so seeding can't continue. This was already shaky, because finished torrent files were moved out of the torrent folder. Decide with the user whether seeding should keep a plain copy in `torrents/`.
- **Changing the download folder** doesn't move existing files. "On this PC" lists only the current folder.
- **The vault key isn't backed up.** If app data is wiped, old `.vlm` files are lost. An export or backup option could be a v0.2.0 item.
- **`HANDOFF-slider.md` / `TEST-REPORT.md`:** a desktop-app tester session may have written a report about the 0.1.0 build. It's out of date now that v0.1.5 fixes those bugs, but worth a skim.
- **Android:** postponed. The engine's `impersonate` feature still fails BoringSSL bindgen for the NDK (informational CI job).
- **Engine known limits:**
  - MovieBox series need a paid account, so don't bypass it.
  - Coflix blocks GitHub runners.
  - The hubstream.art player and hianime's second server aren't resolved.
  - Anime-Sama films and OVAs are skipped.
  - HDHub and MovieBox titles have thinner detail pages than TMDB or AniList ones.
- **Things the user was asked to do (confirm they did):**
  - back up the updater signing key
  - enable private vulnerability reporting
  - regenerate the AniList client secret that was pasted in chat
  - set the Trakt redirect URI to `https://localhost/velmora/trakt-callback`
- **The first commit** `c6526e6` still carries an old email and a Claude trailer. Fixing it needs a force-push the user hasn't approved.

## Launch video

A 63 s launch video was made in Remotion at `video-work/launch-vid/velmora-launch/` (gitignored), and the user posted it to Instagram. The storyboard and the user's 12 decisions are in `video-work/launch-vid/storyboard.md`. Studio: `npx remotion studio` in that folder. Render: `npx remotion render VelmoraLaunch out/velmora-launch.mp4 --codec=h264 --crf=16`.
