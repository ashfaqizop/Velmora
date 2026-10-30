# Velmora handoff

Last updated 2026-10-30, after tagging `universeapi-v1.0.0`.

## Where things stand

The UniverseAPI Engine is done: all ten roadmap steps are built, pushed and green in CI. The tag `universeapi-v1.0.0` points at commit `206b14b`. The final run passed 30 of 31 live checks; the one warning is Coflix refusing GitHub's datacenter IPs.

Nothing for the Tauri app exists yet. That is the next phase.

## Repo

- GitHub: https://github.com/ashfaqizop/Velmora (public). Push as `ashfaqizop` through `gh`.
- The first account, `siliconcode-dev`, is flagged by GitHub: its profile and repos return 404 and Actions is disabled. Don't host or run CI there.
- Branches: `main` (code) and `config` (only `registry.json` plus a README; the app reads it on launch).

```
Velmora/
├── universeapi/                 the engine (Rust workspace)
│   ├── crates/universeapi       library: providers, resolvers, metadata, torrent, downloads, proxy
│   ├── crates/universeapi-cli   dev/CI CLI, `smoke` = live provider checks
│   ├── registry.example.json
│   └── README.md                engine overview and usage
├── docs/universe-api-engine.md  source analysis, design, v1.0 summary
├── .github/workflows/universeapi.yml
├── icon.png, logo.png           app logos
├── universeapi-logo.png         engine logo
└── ani-cli/ MovieBox-TUI/ freeflix-cli/ "custom api/"   gitignored references, never commit
```

## Rules the user set

- **No cargo on the dev PC.** No `cargo build/check/test/fmt` locally. Write code, push, read CI.
- **Commit identity:** author `siliconcode-dev <siliconxcode@gmail.com>` (repo-local git config). Trailer `Co-Authored-By: ashfaqizop <ashfaqizop@gmail.com>`, with no Claude trailer.
- **Batch work:** keep moving through steps and push two or more steps per commit. Small fix commits for a red build are fine.
- **Writing style:** plain "humanizer" style in replies, commits and docs. No em dashes, no "not X but Y", no bold-label lists, no closing offers like "Want me to...?".
- **Never commit** the four reference folders or any secret. The repo is public.

## Decisions already made

- License GPL-3.0-or-later.
- Windows first, Android second (Tauri 2). Engine code must stay Android-buildable.
- Metadata: TMDB for movies/TV (key in the repo secrets `TMDB_READ_TOKEN` / `TMDB_API_KEY`, read from `UNIVERSEAPI_TMDB_*` env vars), Cinemeta as no-key fallback, AniList for anime.
- Provider priority: movies/TV MovieBox → HDHub → 4KHDHub → French → torrents; anime ZokoAnime/hianime → Anime-Sama → Nyaa; Asian drama Dramachi → MovieBox. All reorderable in `Preferences`.
- Quality, codec, 4K auto-pick, audio/subtitle languages and the 18+ toggle are all configurable (defaults 1080p, H.264, 4K manual-only, 18+ hidden). Onboarding will set them later.
- Torrents: stream while downloading, seeding off by default (`seed_after_download`), one-time VPN notice in the app.
- Sync: local, plus optional AniList and Trakt login later. No own backend.
- In-app player only (no external mpv/VLC button). Bundle ffmpeg as a sidecar for downloads.
- IPTV and Live TV were dropped entirely. Home feed is global.
- OTA updates only: one install, then in-app updates through GitHub Releases, with an instant splash that checks for updates on every launch.

## Engine at a glance

Public entry point: `universeapi::Engine` (`engine.rs`).

- `home()`: rails in display order (trending movies/series, trending anime, airing today, HDHub latest, K-Drama, Bollywood, South dubs). The app adds Continue Watching itself.
- `search(q)` / `search_stream(q)`: merged results / one batch per provider as it answers.
- `detail(&ProviderRef)`, then `sources(&title, Target, AudioMode)` → `SourcesReport` with links already probed and ranked, plus per-provider failures.
- `downloads()`: queue with pause/resume/cancel and live progress (`subscribe`). Call `set_ffmpeg(path)` with the sidecar path.
- `proxy()`: `register(&source)` returns a path for the player; the app's custom URI scheme should forward every request to `handle(path, range)`. Torrents use `register_torrent`.
- `torrents()`: the librqbit session (feature `torrent`, on by default).
- `refresh_registry_default()`: call from the launch splash.
- `EngineOptions` takes `cache_dir`, `download_dir`, a `WebSolver` (hidden WebView, only a fallback now) and a `BlockedFallback`.

Features: `torrent` (default on), `impersonate` (wreq Chrome fingerprint on BoringSSL; the CLI enables it, the library default leaves it off).

All public engine futures must stay `Send` for Tauri commands. A compile-time test in `engine.rs` enforces it.

## CI and verification

Workflow `.github/workflows/universeapi.yml` runs on pushes that touch `universeapi/**`, daily at 03:17 UTC, and on manual dispatch:

- Test on ubuntu and windows, Lint (clippy `-D warnings`; rustfmt is non-blocking), Android arm64 check.
- Android `impersonate` check: informational, still failing on BoringSSL bindgen for the NDK.
- Live smoke test (`universeapi smoke`): about 31 checks against the real sites. A critical failure exits 1, and the scheduled run opens or updates a `smoke-failure` issue.

Useful commands:

```
gh run list --limit 3
gh run watch <id> --exit-status
gh run view <id> --log-failed
gh run view <id> --log | grep "Live smoke test"     # PASS/WARN/FAIL lines
gh run download <id> -n smoke-report
```

## Known issues and loose ends

- MovieBox series are members-only (`vipInfo.requireMemberType`). The engine reports this as unavailable and other providers serve episodes. Don't try to bypass the paywall.
- Coflix returns 403 to GitHub runners even through the Chrome fallback. It works from residential IPs.
- hubstream.art (HDHub "PLAYER-2") is an encrypted SPA and isn't resolved.
- hianime's second server (megaplay, "HD-1") isn't used. Only ZokoAnime is decoded.
- Anime-Sama film/OAV panels are skipped; only numbered seasons are mapped.
- Android build of `impersonate` fails in `btls-sys` bindgen ("Unversioned target triples are not supported"). The global `BINDGEN_EXTRA_CLANG_ARGS` fix went into the last run and still failed. Next idea: pass `--target` through cargo-ndk's clang args or pin the NDK.
- The first commit `c6526e6` still has the old email and a Claude trailer. Fixing it needs a force-push, which the user hasn't approved.

## Next phase: the Tauri app

1. Scaffold the Tauri 2 app (`src-tauri/` plus a web frontend) using the engine as a path dependency. Builds only in CI.
2. Launch splash: shows instantly, fetches the registry, checks for an OTA update, then opens the main window.
3. OTA: tauri-plugin-updater with signed GitHub Releases. Generate the updater keypair and store the private key as a repo secret (the user approved this earlier). Add a release workflow that injects the TMDB secrets at build time.
4. Custom URI scheme (for example `velmora-stream://`) wired to `engine.proxy().handle`.
5. UI on @m3e/web with expressive components, using the logos in the repo root. The user mentioned inspiration images; ask for them before designing.

Decisions to ask the user about first:

- Player: libmpv embedded in the window (plays MKV/HEVC/AC3, recommended) versus the WebView player only (many 4K/HEVC/Dolby files won't play).
- Frontend framework: Svelte 5 + Vite + TS was recommended; React, Lit or Solid are the alternatives.

## Gotchas learned this session

- Bash heredocs on this machine collapse `\\` and a Python `'''` string turns `\b` into a backspace byte. Write files with the Write/Edit tools, or use raw strings. Scan for control bytes before committing:
  `python -c "import pathlib;print([(str(p),c) for p in pathlib.Path('universeapi').rglob('*.rs') for c in range(32) if c not in (9,10,13) and bytes([c]) in p.read_bytes()])"`
- The editor (VS Code) can write its buffer back over a file edited from the shell. Re-check a file if an edit seems to vanish.
- `python -I` (used for scripts that read downloaded data) drops the script folder from `sys.path`; load helpers by absolute path.
- A borrowing closure inside `stream::iter(..).map(..).buffer_unordered()` breaks `Send` in `async_trait` methods (rust-lang#64552). Build a `Vec` of futures first, then stream it.
- A new push cancels the in-progress CI run on `main` (workflow concurrency). Check results before pushing again if you need them.
- The scratch Python venv with `requests` and `bs4` for live probing is in the session scratchpad and won't carry over. Recreate it with `python -m venv` if needed.
