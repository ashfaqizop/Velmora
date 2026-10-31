<p align="center">
  <img src="logo.png" alt="Velmora" width="360" />
</p>

<p align="center">
  Movies, series and anime from many sources, searched and played in one desktop app.
</p>

<p align="center">
  <a href="https://github.com/ashfaqizop/Velmora/actions/workflows/app.yml"><img alt="App build" src="https://github.com/ashfaqizop/Velmora/actions/workflows/app.yml/badge.svg" /></a>
  <a href="https://github.com/ashfaqizop/Velmora/actions/workflows/universeapi.yml"><img alt="Engine build" src="https://github.com/ashfaqizop/Velmora/actions/workflows/universeapi.yml/badge.svg" /></a>
  <a href="LICENSE"><img alt="License: GPL-3.0-or-later" src="https://img.shields.io/badge/license-GPL--3.0--or--later-blue" /></a>
</p>

Velmora is a Windows app (Android is planned) that looks a title up across several streaming and download sites at once, ranks the links it finds, and plays the best one in a built-in mpv player. Everything runs on your machine. There is no account, no server of ours and no tracking.

The part that finds and resolves links is a separate Rust library, the [UniverseAPI engine](universeapi/README.md), which lives in this repo and can be used on its own.

## What it does

- Search every source at once. Results stream in as each site answers, and the same title from different sites shows up once.
- Play inside the app with libmpv, so MKV, HEVC, AC3 and 4K files play without extra codecs. If a link fails, the next one starts at the same spot.
- Pick up where you stopped. Continue Watching remembers the exact second, and the end of an episode offers the next one.
- Download movies and episodes for offline viewing, with pause, resume and a queue.
- Stream torrents while they download. Seeding is off unless you turn it on.
- Browse full title pages: cast and crew with photos, trailer, gallery, age rating, release details, links to IMDb, TMDB, AniList and MyAnimeList, and similar titles.
- Sync what you finish to AniList (anime) and Trakt (movies and series).
- Choose your preferred quality, codec, audio and subtitle languages, provider order and accent colour.
- Update in place. Velmora checks for a new version on every launch and installs it itself.

The UI uses [@m3e/web](https://github.com/matraic/m3e), a set of Material 3 Expressive web components.

## Install

Velmora has no public release yet. When there is one, the installer will be on the [Releases](https://github.com/ashfaqizop/Velmora/releases) page, and every later update installs from inside the app.

The installer is small on purpose. On first launch Velmora downloads the video player and download tools (about 80 MB) from the [`runtime-v1`](https://github.com/ashfaqizop/Velmora/releases/tag/runtime-v1) release and checks each file's SHA-256 before using it.

## Repository layout

```
Velmora/
├── app/                         the desktop app
│   ├── src/                     Svelte 5 + TypeScript UI on @m3e/web
│   └── src-tauri/               Tauri 2 shell: engine commands, mpv, stream server, sync
├── universeapi/                 the UniverseAPI engine (Rust workspace)
│   ├── crates/universeapi       providers, resolvers, metadata, torrents, downloads, proxy
│   └── crates/universeapi-cli   developer CLI and the live smoke test
├── docs/                        design notes
└── .github/workflows/           CI for the app and the engine
```

Mirror domains, kill switches and host aliases live in `registry.json` on the [`config`](https://github.com/ashfaqizop/Velmora/tree/config) branch. The app reads it on launch, so a moved site is fixed with a commit to that branch instead of a new release.

## Building

GitHub Actions builds and tests everything. Each push to `main` produces a Windows installer as a workflow artifact, and an `app-v*` tag publishes a signed release that the in-app updater picks up.

To work on it locally you need Node 22, a stable Rust toolchain and, on Windows, [NASM](https://www.nasm.us/) (the engine's Chrome-fingerprint fallback uses BoringSSL).

```sh
cd app
npm ci
npm run check        # svelte-check
npm run tauri dev    # run the app
```

The engine on its own:

```sh
cd universeapi
cargo test --workspace
cargo run -p universeapi-cli -- search "frieren"
```

Builds made without the repo secrets still run. They use Cinemeta instead of TMDB for metadata and show AniList and Trakt as not set up.

## Content and responsibility

Velmora hosts nothing. It reads public pages from third-party sites, the same pages your browser would open, and plays what they link to. Whether watching or downloading a given title is legal depends on where you live and on the source, and that is your responsibility. Torrent peers can see your IP address while a torrent runs; use a VPN if that matters to you.

If you hold the rights to something a source offers, contact that site. Velmora keeps no copy and has no way to remove it.

## Contributing

Bug reports, fixes and new providers are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) first, and please follow the [Code of Conduct](CODE_OF_CONDUCT.md). Security issues go through [SECURITY.md](SECURITY.md), not public issues.

## License

Velmora and the UniverseAPI engine are free software under the [GNU General Public License v3.0 or later](LICENSE).

Parts that ship with or alongside the app keep their own licenses:

| Component | License |
| --- | --- |
| [@m3e/web](https://github.com/matraic/m3e) | MIT |
| [tauri-plugin-libmpv](https://github.com/nini22P/tauri-plugin-libmpv) | MPL-2.0 |
| libmpv ([mpv-winbuild](https://github.com/zhongfly/mpv-winbuild), LGPL build) | LGPL-2.1-or-later |
| ffmpeg ([gyan.dev](https://www.gyan.dev/ffmpeg/builds/) essentials build) | GPL-3.0 |
| Instrument Serif, Plus Jakarta Sans | SIL Open Font License 1.1 |
| Material Symbols | Apache-2.0 |

The UniverseAPI engine draws on ideas from [ani-cli](https://github.com/pystardust/ani-cli), MovieBox-TUI and freeflix-cli. No code from them is included in this repository.
