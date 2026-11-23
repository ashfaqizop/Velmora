<p align="center">
  <img src="docs/velmora-logo.png" alt="" width="140" />
</p>

<h1 align="center">Velmora</h1>

<p align="center">
  Movies, series, anime and music from many sources, searched and played in one desktop app.
</p>

<p align="center">
  <a href="https://github.com/ashfaqizop/Velmora/actions/workflows/app.yml"><img alt="App build" src="https://github.com/ashfaqizop/Velmora/actions/workflows/app.yml/badge.svg" /></a>
  <a href="https://github.com/ashfaqizop/Velmora/actions/workflows/universeapi.yml"><img alt="Engine build" src="https://github.com/ashfaqizop/Velmora/actions/workflows/universeapi.yml/badge.svg" /></a>
  <a href="LICENSE"><img alt="License: GPL-3.0-or-later" src="https://img.shields.io/badge/license-GPL--3.0--or--later-blue" /></a>
</p>

Velmora is a Windows app (Android is planned) that looks a title up across several streaming and download sites at once, ranks the links it finds, and plays the best one in a built-in mpv player. It also brings your YouTube Music into the same app. Everything runs on your machine. There is no account, no server of ours and no tracking. If you turn on Ani, your messages go to Google's Gemini API with your own key.

The part that finds and resolves links is a separate Rust library, the [UniverseAPI engine](universeapi/README.md), which lives in this repo and can be used on its own.

## What it does

- Search every source at once. Results stream in as each site answers, and the same title from different sites shows up once.
- Play inside the app with libmpv, so MKV, HEVC, AC3 and 4K files play without extra codecs. If a link fails, the next one starts at the same spot.
- Play on older PCs too. When mpv won't start, Velmora tries simpler graphics modes down to plain software and remembers the one that works. If none does, a built-in player takes over: ffmpeg repackages or converts the stream on the fly and the app's own window plays it, with subtitles, audio tracks and seeking. Settings > Playback > Player shows which one is in use and can copy a report to send.
- Music: YouTube Music in Velmora (sign in with Google), with synced lyrics, music videos and an immersive player. Albums play gapless, a 10-band parametric EQ and even volume between songs shape the sound, the next songs are saved ahead so they start at once, and you can pick the output device or take it exclusively.
- Download songs, albums and playlists as MP3, M4A or Opus with their tags and cover, or keep them inside Velmora. Windows media keys, the volume flyout and the lock screen control Music, and Home grows rows from what you play: mixes from your top artists, On repeat and Forgotten favorites.
- Sports on Home and a Sports page: live scores, fixtures and results, with official "where to watch" for each match, and a Live TV page of free channels from a public playlist.
- Sound options in the player and in Settings: Dialogue boost, Night mode, Even out volume, and volume up to 200%.
- Pick up where you stopped. Continue Watching remembers the exact second, and the end of an episode offers the next one.
- Download movies and episodes for offline viewing, with pause, resume and a queue, to a folder you choose. Downloads are stored in Velmora's own format and play only inside the app.
- Stream torrents while they download. Seeding is off unless you turn it on.
- Browse full title pages: cast and crew with photos, trailer, gallery, age rating, release details, links to IMDb, TMDB, AniList and MyAnimeList, and similar titles.
- Sync what you finish to AniList (anime) and Trakt (movies and series).
- Ask Ani, the built-in assistant, for recommendations, facts about a title, watch orders or the best download link. It looks things up with Velmora's own tools, learns your taste from what you watch, and answers with cards you can play or download from. Ani is optional and runs on your own Gemini API key.
- Choose your preferred quality, codec, audio and subtitle languages, provider order and accent colour. Settings is split into pages you can search.
- Move like a phone app. Pages slide in the direction you go, a poster grows into its title page and shrinks back on Back, the mini player opens into the full player, and Back returns you to where you were scrolled. Motion runs on Material 3 Expressive springs, and Settings > Appearance > Motion can reduce it to fades or turn it off (it follows Windows' animation setting by default). The mouse's side buttons and Alt+Left/Right go back and forward.
- Update in place. Velmora checks for a new version on every launch and installs it itself.

The UI uses [@m3e/web](https://github.com/matraic/m3e), a set of Material 3 Expressive web components.

## Install

Download `Velmora_x.y.z_x64-setup.exe` from the [latest release](https://github.com/ashfaqizop/Velmora/releases/latest) and run it. It installs for your user only, so it doesn't need admin rights. After that, every update installs from inside the app when it starts.

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
├── site/                        the download website (Svelte + Vite, deployed on Vercel)
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
| Instrument Serif, Bricolage Grotesque | SIL Open Font License 1.1 |
| Material Symbols | Apache-2.0 |
| [humanizer](https://github.com/blader/humanizer) skill, bundled for Ani | MIT |
| Noto Sans (lyrics and titles in other scripts) | SIL Open Font License 1.1 |

The Windows media controls are ported in part from [YTubic](https://github.com/NUber-dev/YTubic) (GPL-3.0).

The UniverseAPI engine draws on ideas from [ani-cli](https://github.com/pystardust/ani-cli), MovieBox-TUI and freeflix-cli. No code from them is included in this repository.
