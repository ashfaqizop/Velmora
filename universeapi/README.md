<p align="center"><img src="../universeapi-logo.png" alt="UniverseAPI" width="200"></p>

# UniverseAPI Engine

The single backend behind **Velmora**: catalog, metadata, trending, streaming and
downloading in one Rust crate. It unifies **ani-cli**, **MovieBox-TUI**,
**freeflix-cli** and the **HDHub custom API**.

```
universeapi/
├── crates/universeapi       # the engine (library, no UI, Windows + Android)
└── crates/universeapi-cli   # dev/CI front-end: search, sources, daily live smoke test
```

> Builds, tests and releases run **only in GitHub Actions**
> ([workflow](../.github/workflows/universeapi.yml)), never on a dev machine.

## What's in it (v1.0)

| Layer | Contents |
|---|---|
| Metadata | **TMDB** (when a key is configured) · **Cinemeta** (no key, fallback) · **AniList** (anime trending, airing today, MAL ids) |
| Providers | **MovieBox** (signed aoneroom API: DASH + direct MP4, captions, language dubs, placeholder recovery) · **4KHDHub** (4K/1080p releases, season packs) · **HDHub** (custom API: Typesense search, listings, posts, episodes, packs) · **Dramachi** (Asian dramas, dub versions) · **Anime-Sama** (VOSTFR/VF anime) · **French-Stream** (VF/VFQ/VOSTFR films and series) · **Coflix** (French films and series) · **YTS**, **EZTV**, **Nyaa** (magnet sources by IMDb id or episode, ranked below direct links) · **ZokoAnime** (direct by MAL id) · **HiAnime** (ani-cli port) |
| Resolvers | HubCDN · HubDrive · HubCloud (junk-filtered) · HBLinks · HBPlay · GreenMotors (gate decoded over plain HTTP, WebView fallback) · HDStream4U (P.A.C.K.E.R → HLS) · PixelDrain · French player embeds (uqload, fsvid, vidzy, ansembed/vidmoly, VOE mirrors, sibnet, sendvid, filemoon family) · ZokoAnime (XOR blob → HLS + subs) |
| Engine | merged search (progressive stream), `search_smart` (the titles that exactly match the query, plus everything ranked best first; see `relevance.rs`), home rails, cross-provider source gathering, **1 KB range-probe verification** (drops ad/decoy pages), preference-driven ranking |
| Torrents | librqbit session (feature `torrent`, on by default): picks the episode inside season packs, streams while downloading with a seekable reader, pauses when done unless seeding is turned on |
| Downloads | queue sized by `max_concurrent_downloads` (changeable live), pause/resume/cancel, HTTP resume from `.part` files, HLS/DASH remuxed to MKV by the ffmpeg sidecar with every track, torrents through the shared session, season zips extracted, subtitles saved beside the video |
| Player proxy | `engine.proxy()`: register a source, hand the returned path to the player; requests carry the source's Referer/Cookie/User-Agent, absolute URLs in HLS/DASH manifests are rewritten (relative ones resolve on their own), torrents stream from `/t/{token}` with byte ranges |
| Sports | **TheSportsDB** (many sports) and **football-data.org** (football, token) merged into one match model: live, upcoming and results, cached per day (live 60 s, upcoming 10 min, results 1 h). Both free tiers run a few minutes late, so a day says `delayed: true`. "Where to watch" lists official broadcasters from hand-written registry rules and TheSportsDB's channel listing |
| Live TV | The public **iptv-org** playlist joined into one channel library: its own blocklist (copyright and adult entries), closed channels and channels without a stream are dropped, plus a remote hide list in the registry. Streams are ordered best first and can be probed |
| Infra | retry + jittered backoff, Cloudflare detection, Chrome-fingerprint fallback (`impersonate` feature, `wreq` on BoringSSL) used only after the normal client is refused, two-level TTL cache (memory + disk), release-name parser |

The app fetches the published registry with `engine.refresh_registry_default()`
during its launch splash (see "Remote configuration" below).

## Usage

```rust
use universeapi::{Engine, EngineOptions, Registry, model::{Target, AudioMode}};

let engine = Engine::new(EngineOptions {
    registry: Registry::default().with_env_overrides(),
    cache_dir: Some(app_cache_dir),
    ..Default::default()
})?;
engine.refresh_registry_default().await?; // config branch registry.json

let rails = engine.home().await;
let hits = engine.search("frieren").await;
let report = engine.sources(&hits[0], Target::Episode { season: 1, episode: 1 }, AudioMode::Sub).await;
```

## Remote configuration (domain rotation)

HDHub and friends rotate domains constantly. Nothing domain-specific is
hard-wired: the app fetches `registry.json` from the repo's `config` branch on
every launch and layers it over built-in defaults, so **a dead domain is fixed
with a one-line commit, not an app release**. See
[`registry.example.json`](registry.example.json).

HDHub mirror resolution order:

1. `hdhub.force_mirror`, an emergency override
2. the cached mirror from the last successful resolution (24 h)
3. the live host-config CDN (`hdhub.host_cdn`)
4. the `mirrors.hd_hub` list

If the current mirror dies mid-session, the engine re-resolves and fails over automatically.

Hoster renames such as `hubcloud` → `hubkloud` are handled by `host_aliases`;
TLD-only rotations (`hubcloud.ist` → `hubcloud.foo`) need no change at all.

**Secrets never live in this repo.** TMDB credentials come from the environment:

| Variable | Used by |
|---|---|
| `UNIVERSEAPI_TMDB_READ_TOKEN` | TMDB v4 bearer token (preferred) |
| `UNIVERSEAPI_TMDB_API_KEY` | TMDB v3 key |
| `UNIVERSEAPI_HDHUB_MIRROR` | force an HDHub mirror (debugging) |

In CI these come from the `TMDB_READ_TOKEN` / `TMDB_API_KEY` repository secrets.

## Sports and Live TV

```rust
let sports = engine.sports();
let day = sports.today().await?;            // SportsDay: matches, sources, errors, delayed
let live = sports.live().await?;            // matches in play now
let soon = sports.upcoming(2).await?;       // next two days, soonest first
let done = sports.results(2).await?;        // newest first
let m = sports.lookup("tsdb:2420125").await?;
let options = engine.sports_watch(&m, "US").await; // official broadcasters

let tv = engine.tv();
let page = tv.channels(&TvQuery { country: Some("US".into()), category: Some("news".into()), ..Default::default() }).await?;
let health = tv.check(&page.items[0].id).await?;   // which streams answer
let source = page.items[0].source(0);              // for engine.proxy().register(..)
```

Keys come from the environment, like TMDB: `UNIVERSEAPI_SPORTSDB_KEY` (without one the free public test key is used, which is thin) and `UNIVERSEAPI_FOOTBALL_DATA_TOKEN`. In CI they are the `SPORTSDB_KEY` and `FOOTBALL_DATA_TOKEN` repository secrets. Both free tiers are small and shared by everyone who uses the same key, so for release builds point `sports.thesportsdb_url` and `sports.football_data_url` in the registry at a caching proxy that holds the keys; the engine then needs no key at all.

**Where to watch is about rights, not streams.** Options come from hand-written `sports.watch` rules (league, country, broadcaster, optional page) and from TheSportsDB's channel listing. Only a rule that names an `iptv_channel` (a free-to-air or official free channel someone checked by hand) can carry a Play button, and the engine drops it if that channel isn't in the TV library. A channel name from the listing is never looked up in the TV library, so a paid channel can't get a Play button by name matching.

**Live TV hide list.** `tv.hidden_channels` (iptv-org ids) and `tv.hidden_hosts` (hostnames, subdomains included) remove channels and streams for everyone on next launch, with no app release, for when a rights holder asks. See [`registry.example.json`](registry.example.json).

Not here yet: the program guide (EPG). The playlist's `guides.json` only points at grabber sites; programme data needs the iptv-org EPG project.

## CLI

```
universeapi home
universeapi search "dune"
universeapi sources "frieren" --episode 3
universeapi sources "akka" --season 1 --pack
universeapi sports live            # also: upcoming, results, day 2026-11-09, watch tsdb:123 --country US
universeapi tv channels --country US --category sports --q espn
universeapi tv check ExampleChannel.us
universeapi smoke --report smoke.json
```

`smoke` runs daily in CI against the live sites. If a critical check fails, the
workflow opens (or updates) a `smoke-failure` issue.

## License

GPL-3.0-or-later.
