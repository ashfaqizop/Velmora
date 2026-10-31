# Contributing to Velmora

Thanks for helping. This page covers how the repo is put together, what CI checks, and what a pull request needs to get merged.

## Before you start

- For a bug, search the [issues](https://github.com/ashfaqizop/Velmora/issues) first, then open one with the bug report template.
- For anything bigger than a small fix (a new provider, a new screen, a change to how sources are ranked), open an issue first so we can agree on the approach before you spend time on it.
- Security problems go through [SECURITY.md](SECURITY.md), never a public issue.

## How the code is organised

| Folder | What lives there |
| --- | --- |
| `app/src` | Svelte 5 + TypeScript UI. Every control is an [@m3e/web](https://github.com/matraic/m3e) component. |
| `app/src-tauri` | Tauri 2 shell: Tauri commands, the loopback stream server for mpv, first-launch runtime download, preferences, AniList/Trakt sync. |
| `universeapi/crates/universeapi` | The engine: providers, link resolvers, metadata (TMDB, Cinemeta, AniList), torrents, downloads, media proxy. |
| `universeapi/crates/universeapi-cli` | Developer CLI and the live smoke test CI runs every day. |

The app talks to the engine only through `universeapi::Engine`. If the UI needs something new, add it to the engine first and expose it through a command in `app/src-tauri/src/commands.rs`.

## Setting up

You need Node 22, stable Rust and, on Windows, [NASM](https://www.nasm.us/).

```sh
cd app && npm ci && npm run tauri dev
```

On first run the app downloads mpv and ffmpeg into its data folder, the same as a normal install. Builds without the repo secrets use Cinemeta for metadata and show AniList and Trakt as not set up; everything else works.

For engine work, the CLI is the quickest loop:

```sh
cd universeapi
cargo run -p universeapi-cli -- search "dune"
cargo run -p universeapi-cli -- sources "dune"
cargo run -p universeapi-cli -- smoke          # live checks against the real sites
```

## What CI checks

Every pull request runs these, and all of them must pass:

- `app`: `npm run check` (svelte-check, no errors or warnings) and `npm run build`, then a full Windows build of the app.
- `universeapi`: `cargo test --workspace` on Linux and Windows, `cargo clippy --workspace --all-targets -- -D warnings`, and an Android arm64 check.

`cargo fmt` uses the settings in `universeapi/rustfmt.toml` (max width 120). Run it before you push.

The live smoke test runs on a schedule because the sites it calls change without notice. A smoke failure on your PR is worth reading, but it isn't always your change.

## Pull requests

- Keep each PR to one change. A refactor and a feature are two PRs.
- Write the commit subject as `type(scope): what changed`, for example `fix(app): seek bar no longer jumps on release` or `feat(universeapi): add <provider>`. Types: `feat`, `fix`, `docs`, `refactor`, `test`, `ci`, `chore`.
- Say in the description what you tested and how. For UI changes, add a screenshot or a short clip.
- Public engine futures must stay `Send` (Tauri commands require it). A compile-time test in `engine.rs` catches this; if it fails, build the futures into a `Vec` before streaming them.

## Adding or fixing a provider

1. Add the provider under `universeapi/crates/universeapi/src/providers/` and implement the `Provider` trait.
2. Register it in `Engine::new` and give it a place in the default provider order in `config.rs`.
3. Add unit tests that parse saved responses, and a check in `universeapi-cli/src/smoke.rs`.
4. Mirror domains that move often belong in the registry. Add defaults in `Registry::default()` and document the key in `universeapi/registry.example.json`.

If a site only moved to a new domain, no code change is needed. Open a PR against `registry.json` on the [`config`](https://github.com/ashfaqizop/Velmora/tree/config) branch instead; every app picks it up on its next launch.

## Rules for this repo

- Never commit secrets: API keys, tokens, signing keys or `.env` files. CI gets credentials from repository secrets, and builds without them must still work.
- Don't commit third-party source trees or binaries (mpv, ffmpeg, other projects' code). Runtime binaries ship through the `runtime-v*` releases with their licenses noted.
- Fonts and assets must have a license that allows redistribution in a GPL app.
- Velmora reads public pages and hosts nothing. Contributions that bypass paywalls, DRM or login walls won't be merged.

## License

By contributing you agree that your work is released under the [GPL-3.0-or-later](LICENSE), the same license as the rest of Velmora.
