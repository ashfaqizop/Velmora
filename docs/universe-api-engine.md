# UniverseAPI Engine — Source Analysis & Unified Design

Status: **v1.0.0 built** · 2026-10-30. The engine lives in [`universeapi/`](../universeapi/README.md);
this document keeps the original source analysis and design for reference.

The UniverseAPI Engine is Velmora's single backend for catalog, metadata, streaming and
downloading. It unifies four sources into one Rust crate that runs inside the Tauri core.
The source repos (`ani-cli/`, `MovieBox-TUI/`, `freeflix-cli/`, `custom api/`) are local,
gitignored references. They are never committed; only the engine is.

---

## 1. Source analysis

### 1.1 ani-cli (POSIX shell, 683 lines, GPL-3.0)

| Aspect | Finding |
|---|---|
| Upstream | `hianime.at` (switched from allanime) |
| Search | `GET /search?keyword=` → HTML, `h3.film-name > a` → `slug-<id>` + title |
| Episodes | `GET /api/theme/episode/list/{id}` → `data-number` / `data-id` |
| Servers | `GET /api/theme/episode/servers?episodeId=` → only the **ZokoAnime** server is decodable; `data-hash` is the base64 embed URL |
| Stream | Embed page carries `window.__P` = `base64(json XOR "otaku-embed-v1")` → `{src: master.m3u8, subtitles[]}` |
| Extras | MAL id in embed URL (ani-skip), separate HLS audio track, Referer = embed origin, sub/dub modes |
| Live check | ✅ search reachable, no Cloudflare challenge (2026-10-30) |

**Takeaway:** a tiny, high-value anime provider. Port the ~60 lines of scraping logic; the XOR
deobfuscation is trivial in Rust.

### 1.2 MovieBox-TUI (Rust, ~51.6k lines, MIT/Apache-2.0)

| Aspect | Finding |
|---|---|
| Architecture | `Provider` + `ReleaseProvider` traits, `service.rs` facade, tokio + reqwest (rustls) |
| Providers | MovieBox (aoneroom API, HMAC-signed `x-tr-signature`, 7-host failover pool), **4KHDHub** (+ HubCloud resolver), Dramachi, CircleFTP/DhakaFlix (BDIX), Stremio addons, IPTV/M3U |
| Infra worth reusing | Loopback proxy with DASH manifest rewriting + header injection, chunked range-resume downloader, MessagePack disk cache, Hickory DNS fallback, SHA-256 verified updater |
| Models | `CatalogItem`, `MediaDetails`, `Season/Episode`, `Release`, `SourceMirror`, `PlaybackSource{url, headers, subtitle}` |

**Takeaway:** this is the **skeleton** of the engine. It's already Rust and permissively licensed,
and its trait design matches what Velmora needs. TUI code (~60%) is dropped.

### 1.3 freeflix-cli (Python, GPL-3.0)

| Aspect | Finding |
|---|---|
| Providers | Anime-Sama, GoldenAnime, French-Manga, GoldenMS, French-Stream, Coflix, YTS, EZTV, Nyaa, HiAnime |
| Language focus | Mostly **French** (VF/VOSTFR); YTS/EZTV/Nyaa are torrent sources |
| Infra worth reusing | **Remote mirror registry** (`source_portal.jsonc`, 24h origin cache), resilient scraper w/ retries, AniList sync, OpenSubtitles fallback, P.A.C.K.E.R deobfuscator, decoy-stream detection, SSRF-safe proxy, resume tracker |

**Takeaway:** mine it for **patterns** (remote mirror config, resilience, AniList, subtitle
fallback, decoy detection) more than for providers. French and torrent providers are optional modules.

### 1.4 Custom API — HDHub4u client (Python, 1345 lines)

#### E2E results (2026-10-30, live mirror `new10.hdhub4u.cl`)

| # | Endpoint | Result | Latency |
|---|---|---|---|
| 1 | `get_host_config` / `resolve_mirror` (suncdn) | ✅ matches hardcoded mirror | 0.8s |
| 2 | `get_featured_movies` | ✅ 30 items | 0.4s |
| 3 | `search_movies` (Typesense, open) | ✅ 13 hits / 14,548 docs, includes `imdb_id` | 0.6s |
| 4 | `get_latest_releases` p1/p2 | ✅ 70/page, 970 pages | 0.2s |
| 5 | `get_category_page` | ✅ 15/page | <0.1s |
| 6 | `get_tag_page` | ✅ | <0.1s |
| 7 | `get_movie_detail` (movie + series) | ✅ title, poster, trailer, screenshots, links, reactions | 0.2s |
| 10–11 | sitemaps | ✅ 20 sub-sitemaps, 1000 posts each | 0.5s |
| 12 | `resolve_hubcdn` (.club + .wiki) | ✅ direct R2 / Google CDN, **range-probe 206** | 1.5s |
| 13 | `resolve_hubdrive` | ✅ R2 + filename + size, **range-probe 206** | 0.8s |
| 14 | `resolve_hubcloud` | ⚠️ partial: see bugs | 2.4s |
| 15 | `resolve_greenmotors` | ❌ needs Playwright (not installed) | — |
| 9 | `submit_reaction` | ⏭ skipped (writes to the live site) | — |

#### Bugs found

1. **`categories` is wrong.** `a[href*='/category/']` matches the site's nav menu, so every post gets
   "BollyWood, 18+, TALKs…". Fix: scope to `.page-meta a[href*='/category/']`
   (gives the correct `Crime, Drama, Netflix, WEB-Series`).
2. **HubCloud returns junk as "direct".** For one file, the returned links break down as:
   - `HDhub4u.ms` and `one.one.one.one` (VPN ad): site chrome, not downloads.
   - `gpdl.hubcloud.ist` "10Gbps": redirects to an ad page (`cryptonewz.online`, HTML).
   - `*.workers.dev`: 403.
   - Only **FSLv2 (R2 signed)** actually serves bytes.

   Fix: allow-list hosts and verify each link with a range probe (content-type is not `text/html`).
3. **`hbplay` "Watch Online" isn't unwrapped.** Its `u=` param is base64 of the R2 URL, so it can be
   decoded directly instead of being typed `stream`.
4. **No stream resolvers.** `stream_links` are returned raw.
   - `hdstream4u.com/file/{id}` is **solved here**: P.A.C.K.E.R unpack → signed HLS master
     (multi-audio, Hindi default) + English VTT. Verified 200 `application/vnd.apple.mpegurl`.
   - `hubstream.art/#{id}` is an encrypted SPA (1 MB bundle, AES). Needs reverse-engineering;
     deferred.
5. **`4khdhub.one` is unhandled.** MovieBox-TUI already ships a 4KHDHub provider and HubCloud resolver.
6. Minor:
   - `lstrip("www.")` strips characters, not the prefix.
   - Featured `year` disagrees with `caption`.
   - Permalinks in search use old mirrors (`new2.hdhub4u.free`) and must be rebased onto the live mirror.
   - `search_all` stops by `len >= found` and can request one extra page.

---

## 2. UniverseAPI Engine design

### 2.1 Shape

```
src-tauri/
└── crates/universe-engine/          # pure library, no Tauri dependency → unit-testable in CI
    ├── model.rs                     # unified domain model (below)
    ├── engine.rs                    # facade: fan-out, merge, dedupe, cache, rank
    ├── registry.rs                  # remote provider/mirror config (OTA-updatable JSON)
    ├── net/                         # reqwest client pool, DNS fallback, retry/backoff, UA rotation
    ├── cache/                       # MessagePack disk cache + in-memory LRU, per-endpoint TTL
    ├── providers/                   # catalog + metadata sources
    │   ├── hdhub/                   # ← custom API (Typesense search, listings, detail)
    │   ├── hianime/                 # ← ani-cli
    │   ├── moviebox/                # ← MovieBox-TUI (signed API)
    │   ├── fourkhdhub/              # ← MovieBox-TUI
    │   └── addons/                  # ← MovieBox-TUI (Stremio)   [optional: dramachi, anime-sama, nyaa]
    ├── resolvers/                   # host → direct/stream URL, shared by ALL providers
    │   ├── hubcdn.rs  hubdrive.rs  hubcloud.rs  hblinks.rs  hbplay.rs
    │   ├── greenmotors.rs           # via hidden Tauri WebView (trait-injected), no Playwright
    │   ├── hdstream4u.rs            # P.A.C.K.E.R unpack → HLS
    │   └── zokoanime.rs             # XOR "otaku-embed-v1" → HLS
    ├── stream/                      # media proxy (Referer/header injection, HLS/DASH rewrite)
    └── download/                    # segmented range-resume downloader + HLS remux queue
```

### 2.2 Unified model

```text
Title        { uid, kind: Movie|Series|Anime, name, year, ids{imdb,tmdb,mal,anilist},
               poster, backdrop, overview, genres, rating, trailer, origins: [ProviderRef] }
Season       { number, episodes: [Episode] }
Episode      { number, title, air_date, thumb, origins: [ProviderRef] }
Source       { provider, kind: Hls|Dash|Mp4|Archive, url, headers, quality, size,
               audio: [Track], subtitles: [Track], expires_at, verified: bool }
```

- **Cross-provider merge:** titles dedupe on `imdb_id` (HDHub Typesense exposes it), then
  `tmdb`/`mal`, then normalized title+year. One Velmora title lists every provider that has it.
- **Two resolution intents** (from MovieBox-TUI's `ResolutionIntent`): `Stream` prefers HLS/DASH
  sources that play in-app. `Download` prefers direct files with known size (HubDrive gives the filename and bytes).

### 2.3 Optimizations over the sources

| Area | Optimization |
|---|---|
| Domain rotation | Mirrors/selectors live in a **signed remote `registry.json`** (freeflix pattern + suncdn auto-resolve), fetched during the launch splash and cached. A dead domain gets fixed by a config push, with no app update. |
| Latency | Fan-out search across providers concurrently, stream results to the UI as each provider finishes (Tauri events), and race failover hosts (MovieBox pool). |
| Resolver quality | Every resolved link gets a **1 KB range probe** before it's shown. `text/html` means ad/decoy and the link is dropped. Results are ranked by verified, then speed, then expiry. |
| Expiry | R2 signed URLs (5–8h) carry `expires_at`; the engine re-resolves transparently on 403. |
| In-app playback | WebView `<video>` can't set `Referer`. A **Tauri custom URI scheme** (`velmora-stream://`) proxies media with injected headers and rewrites HLS/DASH manifests. This replaces MovieBox's TCP loopback: no open port and no firewall prompt. |
| GreenMotors | Uses a hidden Tauri `WebviewWindow` to run the AES/timer page instead of shipping Chromium/Playwright (saves ~150 MB). |
| Caching | Per-endpoint TTL (search 10 min, detail 6h, resolved links until `expires_at`), stale-while-revalidate for home rails. |
| Resilience | Retry with jittered backoff on 403/429/5xx, Cloudflare-challenge detection, a per-provider circuit breaker with health exposed to a Settings → Sources screen. |

### 2.4 Licensing note

ani-cli and freeflix-cli are **GPL-3.0**. Porting their code into Velmora would make a distributed
Velmora binary GPL-3.0 as well. MovieBox-TUI (MIT/Apache) and the custom API (yours) carry no
such constraint. Option A: accept GPL-3.0 for Velmora. Option B: clean-room reimplement the
ani-cli/freeflix *protocols* (the endpoints and the XOR key are facts, not code). **Decision needed.**

### 2.5 Build constraint

No cargo builds or packaging on the dev PC. The engine crate is written locally; compile, test
(`cargo test -p universe-engine`), bundle, sign, and release all run in **GitHub Actions**.

---

## 3. What was built (v1.0.0)

| Step | Delivered |
|---|---|
| Core | unified model, release-name parser, ranking, HTTP layer with retries and a 1 KB link probe, two-level cache, remote registry |
| HDHub | custom API port with the E2E bugs fixed, mirror failover, every domain remote-configurable |
| Anime | hianime (ani-cli port), ZokoAnime by MAL id, Anime-Sama, Nyaa |
| Movies and TV | MovieBox (signed API; series are members-only and handled), 4KHDHub, French-Stream, Coflix, YTS, EZTV |
| Asian dramas | Dramachi with dub versions, TMDB K-Drama rail |
| Resolvers | Hub family, GreenMotors decoded over plain HTTP (no browser), hdstream4u, French player embeds including VOE |
| Metadata | TMDB (key from repo secrets), Cinemeta fallback, AniList trending and airing |
| Torrents | librqbit session: episode picking inside packs, streaming while downloading, seeding off by default |
| Network | Chrome-fingerprint fallback (`impersonate`, wreq on BoringSSL) after the normal client is refused |
| Downloads | queue with live concurrency, resume, ffmpeg remux for HLS/DASH, torrent jobs, zip extraction, subtitles |
| Player proxy | header injection, HLS/DASH manifest rewriting, torrent byte-range streaming |
| Remote config | `registry.json` on the `config` branch, checked by the daily smoke test |

Every push builds and tests on Windows, Linux and Android (arm64), and a
daily scheduled run checks all providers against the live sites.

Known limits at v1.0: MovieBox series need a paid membership; Coflix and
some torrent APIs refuse datacenter IPs (fine from home connections);
hubstream.art's player is still encrypted; the Android build of the
Chrome fallback is not green yet.

## 4. Original proposed phases

1. **Engine core**: model, net, cache, registry, plus the HDHub provider and its 7 resolvers (with the bugs above fixed). Includes a CI workflow running `cargo check` and `cargo test`.
2. **Providers**: hianime (ani-cli port), MovieBox + 4KHDHub (adapted from MovieBox-TUI), cross-provider merge.
3. **Tauri shell**: instant splash, then OTA check (tauri-plugin-updater + signed GitHub Releases), then the main window.
4. **UI**: @m3e/web expressive design system from the inspiration images and the logos.
5. **Stream proxy + in-app player + download manager.**
