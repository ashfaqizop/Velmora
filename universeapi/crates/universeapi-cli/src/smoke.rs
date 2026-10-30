//! Live health checks run daily in CI. A failing *critical* check exits 1 and
//! the workflow opens an issue, so a site change is caught before users see it.

use std::future::Future;
use std::time::Instant;

use serde::Serialize;
use universeapi::model::{AudioMode, ExternalIds, MediaKind, ProviderId, ProviderRef, Target, TitleSummary};
use universeapi::providers::hdhub::HdHub;
use universeapi::Engine;

#[derive(Debug, Serialize)]
pub struct Check {
    pub name: String,
    pub critical: bool,
    pub ok: bool,
    pub millis: u128,
    pub detail: String,
}

#[derive(Debug, Default, Serialize)]
pub struct SmokeReport {
    pub checks: Vec<Check>,
}

impl SmokeReport {
    pub fn critical_failures(&self) -> usize {
        self.checks.iter().filter(|c| c.critical && !c.ok).count()
    }
}

async fn check<Fut>(report: &mut SmokeReport, name: &str, critical: bool, fut: Fut)
where
    Fut: Future<Output = anyhow::Result<String>>,
{
    let t = Instant::now();
    let (ok, detail) = match fut.await {
        Ok(d) => (true, d),
        Err(e) => (false, format!("{e:#}")),
    };
    report.checks.push(Check { name: name.into(), critical, ok, millis: t.elapsed().as_millis(), detail });
}

fn ensure(cond: bool, msg: impl Into<String>) -> anyhow::Result<()> {
    if cond {
        Ok(())
    } else {
        Err(anyhow::anyhow!(msg.into()))
    }
}

pub async fn run(engine: &Engine) -> SmokeReport {
    let mut r = SmokeReport::default();

    // The published config branch must always parse with this engine build;
    // a bad edit there fails the daily run before users launch the app.
    check(&mut r, "registry: published config parses", true, async {
        let text = engine.ctx().http.get_text(universeapi::config::REGISTRY_URL, &[]).await?;
        let reg = universeapi::Registry::from_json_over_defaults(&text)?;
        let hd = reg.mirror(ProviderId::HdHub).unwrap_or_default().to_string();
        anyhow::Ok(format!("version {}, {} mirror lists, HDHub {hd}", reg.version, reg.mirrors.len()))
    })
    .await;
    let hd = HdHub::new(engine.ctx().clone());

    // ── HDHub (custom API) ─────────────────────────────────────────────
    check(&mut r, "hdhub: live mirror", true, async {
        let m = hd.mirror().await?;
        ensure(m.starts_with("http"), "no mirror")?;
        anyhow::Ok(m)
    })
    .await;

    check(&mut r, "hdhub: typesense search", true, async {
        let hits = engine.provider(ProviderId::HdHub).unwrap().search("avengers").await?;
        ensure(!hits.is_empty(), "0 hits")?;
        anyhow::Ok(format!("{} hits, first: {}", hits.len(), hits[0].name))
    })
    .await;

    let mut latest: Vec<TitleSummary> = Vec::new();
    check(&mut r, "hdhub: latest listing", true, async {
        let (items, pages) = hd.listing("", 1).await?;
        ensure(items.len() >= 10, format!("only {} items", items.len()))?;
        latest = items.clone();
        anyhow::Ok(format!("{} items, {pages} pages", items.len()))
    })
    .await;

    let movie = latest.iter().find(|t| t.kind == MediaKind::Movie).cloned();
    let series = latest.iter().find(|t| t.kind == MediaKind::Series).cloned();

    if let Some(m) = movie.clone() {
        check(&mut r, "hdhub: movie detail + links", true, async {
            let d = engine.detail(&m.origin).await?;
            ensure(!d.links.is_empty(), "no download links parsed")?;
            anyhow::Ok(format!("{} — {} links, genres {:?}", d.summary.name, d.links.len(), d.genres))
        })
        .await;
        check(&mut r, "hdhub: movie sources verified", true, async {
            let rep = engine.sources(&m, Target::Movie, AudioMode::Sub).await;
            let ok = rep.sources.iter().filter(|s| s.verified == Some(true)).count();
            ensure(ok > 0, format!("0 verified of {} (errors: {:?})", rep.sources.len(), rep.errors))?;
            let hosts: Vec<_> = rep.sources.iter().map(|s| s.host.as_str()).collect();
            anyhow::Ok(format!("{ok} verified; hosts {hosts:?}"))
        })
        .await;
    }
    if let Some(s) = series.clone() {
        check(&mut r, "hdhub: series episode sources", false, async {
            let d = engine.detail(&s.origin).await?;
            let ep = d.seasons.first().and_then(|se| se.episodes.first()).map(|e| (e.season, e.number));
            let target = ep.map(|(se, e)| Target::Episode { season: se, episode: e }).unwrap_or(Target::Season { season: 1 });
            let rep = engine.sources(&s, target, AudioMode::Sub).await;
            let ok = rep.sources.iter().filter(|x| x.verified == Some(true)).count();
            ensure(ok > 0, format!("0 verified for {:?} of {}", target, d.summary.name))?;
            anyhow::Ok(format!("{} {:?}: {ok} verified", d.summary.name, target))
        })
        .await;
    }

    // ── MovieBox ───────────────────────────────────────────────────────
    let mb = engine.provider(ProviderId::MovieBox).unwrap();
    let mut mb_movie: Option<TitleSummary> = None;
    check(&mut r, "moviebox: signed login + search", true, async {
        let hits = mb.search("the batman").await?;
        ensure(!hits.is_empty(), "0 hits")?;
        mb_movie = hits.iter().find(|h| h.kind == MediaKind::Movie).cloned();
        anyhow::Ok(format!("{} hits, first: {} ({:?})", hits.len(), hits[0].name, hits[0].year))
    })
    .await;

    if let Some(m) = mb_movie.clone() {
        check(&mut r, "moviebox: movie sources verified", true, async {
            let d = mb.detail(&m.origin.id).await?;
            let mut src = mb.sources(&d, Target::Movie, AudioMode::Sub).await?;
            engine.verify(&mut src).await;
            let ok: Vec<_> = src.iter().filter(|s| s.verified == Some(true)).collect();
            ensure(!ok.is_empty(), format!("0 verified of {}", src.len()))?;
            let subs = src.iter().map(|s| s.subtitles.len()).max().unwrap_or(0);
            let kinds: Vec<String> = ok.iter().map(|s| format!("{:?}/{:?}", s.format, s.tags.resolution)).collect();
            anyhow::Ok(format!("{} — {} verified {:?}, {subs} subtitle tracks", d.summary.name, ok.len(), kinds))
        })
        .await;
    }

    check(&mut r, "moviebox: series episode (files or members-only gate)", true, async {
        let hits = mb.search("breaking bad").await?;
        let show = hits.iter().find(|h| h.kind.is_episodic()).ok_or_else(|| anyhow::anyhow!("no series hit"))?;
        let d = mb.detail(&show.origin.id).await?;
        ensure(!d.seasons.is_empty(), "no seasons parsed")?;
        match mb.sources(&d, Target::Episode { season: 1, episode: 1 }, AudioMode::Sub).await {
            Ok(mut src) => {
                engine.verify(&mut src).await;
                let ok = src.iter().filter(|s| s.verified == Some(true)).count();
                ensure(ok > 0, format!("0 verified of {}", src.len()))?;
                anyhow::Ok(format!("{} — {} seasons, S1E1 {ok} verified", d.summary.name, d.seasons.len()))
            }
            // Members-only series are expected; the engine falls through to other providers.
            Err(universeapi::Error::NotFound(m)) if m == universeapi::providers::moviebox::MEMBERS_ONLY => {
                anyhow::Ok(format!("{} — members-only on MovieBox, handled", d.summary.name))
            }
            Err(e) => Err(e.into()),
        }
    })
    .await;

    check(&mut r, "engine: series episode across providers", true, async {
        let hits = engine.search("breaking bad").await;
        let show = hits.iter().find(|h| h.kind.is_episodic()).ok_or_else(|| anyhow::anyhow!("no series hit"))?;
        let rep = engine.sources(show, Target::Episode { season: 1, episode: 1 }, AudioMode::Sub).await;
        let best = rep.sources.first().ok_or_else(|| anyhow::anyhow!("no sources at all: {:?}", rep.errors))?;
        let providers: std::collections::BTreeSet<String> = rep.sources.iter().map(|s| s.provider.to_string()).collect();
        anyhow::Ok(format!("{} sources from {providers:?}; best {} {} {:?}", rep.sources.len(), best.provider, best.host, best.tags.resolution))
    })
    .await;

    // ── 4KHDHub + GreenMotors (no browser) ─────────────────────────────
    let fk = engine.provider(ProviderId::FourKHdHub).unwrap();
    let mut fk_movie: Option<TitleSummary> = None;
    check(&mut r, "4khdhub: search + detail links", true, async {
        let hits = fk.search("dune").await?;
        let m = hits.iter().find(|h| h.kind == MediaKind::Movie).ok_or_else(|| anyhow::anyhow!("no movie hit"))?;
        let d = fk.detail(&m.origin.id).await?;
        ensure(!d.links.is_empty(), "no download links parsed")?;
        fk_movie = Some(m.clone());
        anyhow::Ok(format!("{} ({:?}) — {} links", d.summary.name, d.summary.year, d.links.len()))
    })
    .await;

    if let Some(m) = fk_movie.clone() {
        check(&mut r, "greenmotors: gate decoded over HTTP", true, async {
            let d = fk.detail(&m.origin.id).await?;
            let gate = d
                .links
                .iter()
                .find(|l| l.host == "GreenMotors")
                .ok_or_else(|| anyhow::anyhow!("no GreenMotors link on page"))?;
            let html = engine.ctx().http.get_text(&gate.url, &[]).await?;
            let target = universeapi::resolvers::greenmotors_decode(&html).ok_or_else(|| anyhow::anyhow!("gate not decodable"))?;
            anyhow::Ok(target)
        })
        .await;

        check(&mut r, "4khdhub: movie sources verified", false, async {
            let d = fk.detail(&m.origin.id).await?;
            let mut src = fk.sources(&d, Target::Movie, AudioMode::Sub).await?;
            engine.verify(&mut src).await;
            let ok = src.iter().filter(|s| s.verified == Some(true)).count();
            ensure(ok > 0, format!("0 verified of {}", src.len()))?;
            let hosts: Vec<_> = src.iter().filter(|s| s.verified == Some(true)).map(|s| s.host.as_str()).collect();
            anyhow::Ok(format!("{ok} verified; {hosts:?}"))
        })
        .await;
    }

    // ── Dramachi ───────────────────────────────────────────────────────
    check(&mut r, "dramachi: search → episode → CDN file", false, async {
        let p = engine.provider(ProviderId::Dramachi).unwrap();
        let hits = p.search("queen of tears").await?;
        let t = hits.first().ok_or_else(|| anyhow::anyhow!("0 hits"))?;
        let d = p.detail(&t.origin.id).await?;
        let eps = d.seasons.first().map(|s| s.episodes.len()).unwrap_or(0);
        ensure(eps > 0, "no episodes")?;
        let mut src = p.sources(&d, Target::Episode { season: 1, episode: 1 }, AudioMode::Sub).await?;
        engine.verify(&mut src).await;
        let ok = src.iter().filter(|s| s.verified == Some(true)).count();
        ensure(ok > 0, format!("0 verified of {}", src.len()))?;
        anyhow::Ok(format!("{} — {eps} eps, S1E1 {ok} verified ({})", d.summary.name, src.iter().map(|s| s.label.as_str()).collect::<Vec<_>>().join(" | ")))
    })
    .await;

    // ── French sources ─────────────────────────────────────────────────
    check(&mut r, "anime-sama: search → season → VOSTFR player", false, async {
        let p = engine.provider(ProviderId::AnimeSama).unwrap();
        let hits = p.search("frieren").await?;
        let t = hits.first().ok_or_else(|| anyhow::anyhow!("0 hits"))?;
        let d = p.detail(&t.origin.id).await?;
        ensure(!d.seasons.is_empty(), "no seasons")?;
        let mut src = p.sources(&d, Target::Episode { season: 1, episode: 1 }, AudioMode::Sub).await?;
        engine.verify(&mut src).await;
        let ok: Vec<_> = src.iter().filter(|s| s.verified == Some(true)).map(|s| s.host.clone()).collect();
        ensure(!ok.is_empty(), format!("0 verified of {}", src.len()))?;
        anyhow::Ok(format!("{} — {} seasons; verified {ok:?}", d.summary.name, d.seasons.len()))
    })
    .await;

    check(&mut r, "french-stream: movie players", false, async {
        let p = engine.provider(ProviderId::FrenchStream).unwrap();
        let hits = p.search("dune").await?;
        let m = hits.iter().find(|h| h.kind == MediaKind::Movie).ok_or_else(|| anyhow::anyhow!("no movie hit"))?;
        let d = p.detail(&m.origin.id).await?;
        let mut src = p.sources(&d, Target::Movie, AudioMode::Dub).await?;
        engine.verify(&mut src).await;
        let ok: Vec<_> = src.iter().filter(|s| s.verified == Some(true)).map(|s| s.label.clone()).collect();
        ensure(!ok.is_empty(), format!("0 verified of {}", src.len()))?;
        anyhow::Ok(format!("{} ({:?}) — verified {ok:?}", d.summary.name, m.year))
    })
    .await;

    check(&mut r, "coflix: movie players", false, async {
        let p = engine.provider(ProviderId::Coflix).unwrap();
        let hits = p.search("dune").await?;
        let m = hits.iter().find(|h| h.kind == MediaKind::Movie).ok_or_else(|| anyhow::anyhow!("no movie hit"))?;
        let d = p.detail(&m.origin.id).await?;
        let mut src = p.sources(&d, Target::Movie, AudioMode::Dub).await?;
        engine.verify(&mut src).await;
        let ok: Vec<_> = src.iter().filter(|s| s.verified == Some(true)).map(|s| s.host.clone()).collect();
        ensure(!ok.is_empty(), format!("0 verified of {}", src.len()))?;
        anyhow::Ok(format!("{} — verified {ok:?}", d.summary.name))
    })
    .await;

    // ── Torrents ───────────────────────────────────────────────────────
    let torrent_check = |provider: ProviderId, ids: ExternalIds, kind: MediaKind, target: Target| async move {
        let p = engine.provider(provider).unwrap();
        let s = p.sources_by_ids(&ids, kind, target, AudioMode::Sub).await?.unwrap_or_default();
        let best = s.iter().filter_map(|x| x.torrent.as_ref().and_then(|t| t.seeders)).max().unwrap_or(0);
        ensure(!s.is_empty() && best > 0, format!("{} torrents, best seeders {best}", s.len()))?;
        anyhow::Ok(format!("{} torrents, best seeders {best}", s.len()))
    };
    check(
        &mut r,
        "yts: Dune (tt1160419) by IMDb id",
        false,
        torrent_check(ProviderId::Yts, ExternalIds { imdb: Some("tt1160419".into()), ..Default::default() }, MediaKind::Movie, Target::Movie),
    )
    .await;
    check(
        &mut r,
        "eztv: Breaking Bad S01E01 by IMDb id",
        false,
        torrent_check(
            ProviderId::Eztv,
            ExternalIds { imdb: Some("tt0903747".into()), ..Default::default() },
            MediaKind::Series,
            Target::Episode { season: 1, episode: 1 },
        ),
    )
    .await;
    check(&mut r, "nyaa: Frieren episode 3", false, async {
        let p = engine.provider(ProviderId::Nyaa).unwrap();
        let d = p.detail("Sousou no Frieren").await?;
        let s = p.sources(&d, Target::Episode { season: 1, episode: 3 }, AudioMode::Sub).await?;
        anyhow::Ok(format!("{} torrents; top: {}", s.len(), s.first().map(|x| x.label.as_str()).unwrap_or("")))
    })
    .await;

    // ── Torrent engine (librqbit): legal, well-seeded Ubuntu ISO ────────
    check(&mut r, "torrent engine: metadata + stream first 256 KiB", false, async {
        use tokio::io::AsyncReadExt;
        let page = engine.ctx().http.get_text("https://releases.ubuntu.com/24.04/", &[]).await?;
        let name = page
            .split('"')
            .find(|s| s.ends_with("-live-server-amd64.iso.torrent"))
            .ok_or_else(|| anyhow::anyhow!("no .torrent link on releases page"))?;
        let url = format!("https://releases.ubuntu.com/24.04/{name}");
        let te = engine.torrents().await?;
        let job = te.add(&url, None).await?;
        let file = job.selected_file().map(|f| (f.name.clone(), f.len)).unwrap_or_default();
        let mut reader = te.stream(&job).await?;
        let mut buf = vec![0u8; 256 * 1024];
        tokio::time::timeout(std::time::Duration::from_secs(120), reader.read_exact(&mut buf))
            .await
            .map_err(|_| anyhow::anyhow!("no data within 120 s"))??;
        let p = te.progress(&job);
        te.remove(job, true).await?;
        anyhow::Ok(format!("{} ({} MB) streamed 256 KiB; {} bytes fetched", file.0, file.1 / 1_000_000, p.downloaded))
    })
    .await;

    // ── Download manager ───────────────────────────────────────────────
    check(&mut r, "downloads: direct file, then HLS remux + cancel", false, async {
        use universeapi::download::{DownloadRequest, DownloadState};
        use universeapi::model::Source;
        let dm = engine.downloads().clone();
        let wait = |id: u64, done: fn(&universeapi::download::DownloadStatus) -> bool, secs: u64| {
            let dm = dm.clone();
            async move {
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(secs);
                loop {
                    let st = dm.list().await.into_iter().find(|s| s.id == id).ok_or_else(|| anyhow::anyhow!("job vanished"))?;
                    if done(&st) {
                        return anyhow::Ok(st);
                    }
                    if let DownloadState::Failed { error } = &st.state {
                        anyhow::bail!("job failed: {error}");
                    }
                    ensure(std::time::Instant::now() < deadline, format!("timed out in state {:?}", st.state))?;
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
            }
        };

        let mut file = Source::new(ProviderId::HdHub, "test", "SHA256SUMS", "https://releases.ubuntu.com/24.04/SHA256SUMS");
        file.filename = Some("SHA256SUMS.txt".into());
        let id = dm.enqueue(DownloadRequest { source: file, title: "ubuntu-sums".into(), dest_dir: None, episode: None }).await;
        let st = wait(id, |s| matches!(s.state, DownloadState::Completed { .. }), 60).await?;
        let DownloadState::Completed { path } = &st.state else { unreachable!() };
        let bytes = std::fs::metadata(path)?.len();
        ensure(bytes > 0, "empty file")?;

        let hls = Source::new(ProviderId::HdHub, "test", "HLS test", "https://test-streams.mux.dev/x36xhzz/x36xhzz.m3u8");
        let id2 = dm.enqueue(DownloadRequest { source: hls, title: "hls-test".into(), dest_dir: None, episode: None }).await;
        let st2 = wait(id2, |s| s.downloaded > 2_000_000, 90).await?;
        dm.cancel(id2).await;
        wait(id2, |s| s.state == DownloadState::Cancelled, 30).await?;
        anyhow::Ok(format!("direct {bytes} B ok; ffmpeg remuxed {} KB before cancel", st2.downloaded / 1000))
    })
    .await;

    // ── Media proxy: master → variant → segment, headers injected ──────
    check(&mut r, "proxy: HLS master, variant and segment", false, async {
        use universeapi::proxy::ProxyBody;
        let ids = ExternalIds { mal: Some(52991), ..Default::default() };
        let p = engine.provider(ProviderId::ZokoAnime).unwrap();
        let src = p
            .sources_by_ids(&ids, MediaKind::Anime, Target::Episode { season: 1, episode: 1 }, AudioMode::Sub)
            .await?
            .unwrap_or_default()
            .into_iter()
            .find(|s| s.format == universeapi::model::StreamFormat::Hls)
            .ok_or_else(|| anyhow::anyhow!("no HLS source"))?;
        let proxy = engine.proxy();
        let master = proxy.register(&src)?;

        // resolve a playlist entry the way a player does, relative to the proxied URL
        let resolve = |base: &str, entry: &str| -> String {
            if entry.starts_with('/') {
                return entry.to_string();
            }
            let path = base.split('?').next().unwrap_or(base);
            format!("{}/{entry}", &path[..path.rfind('/').unwrap_or(0)])
        };
        let text = |b: ProxyBody| match b {
            ProxyBody::Bytes(v) => String::from_utf8_lossy(&v).into_owned(),
            ProxyBody::Stream(_) => String::new(),
        };
        let first_uri = |t: &str| t.lines().map(str::trim).find(|l| !l.is_empty() && !l.starts_with('#')).map(str::to_string);

        let m = proxy.handle(&master, None).await?;
        ensure(m.status == 200, format!("master status {}", m.status))?;
        let mt = text(m.body);
        ensure(mt.starts_with("#EXTM3U"), "master is not a playlist")?;
        let variant = resolve(&master, &first_uri(&mt).ok_or_else(|| anyhow::anyhow!("empty master"))?);
        let v = proxy.handle(&variant, None).await?;
        let vt = text(v.body);
        ensure(vt.starts_with("#EXTM3U"), format!("variant {} is not a playlist", v.status))?;
        let seg = resolve(&variant, &first_uri(&vt).ok_or_else(|| anyhow::anyhow!("empty variant"))?);
        let s = proxy.handle(&seg, Some("bytes=0-65535")).await?;
        let got = match s.body {
            ProxyBody::Stream(mut st) => {
                use futures::StreamExt;
                let mut n = 0usize;
                while let Some(chunk) = st.next().await {
                    n += chunk?.len();
                    if n >= 4096 {
                        break;
                    }
                }
                n
            }
            ProxyBody::Bytes(b) => b.len(),
        };
        ensure((200..300).contains(&s.status) && got > 0, format!("segment status {} with {got} bytes", s.status))?;
        anyhow::Ok(format!("master → variant → segment ok ({} → {got} bytes)", s.status))
    })
    .await;

    // ── Anime ──────────────────────────────────────────────────────────
    check(&mut r, "zokoanime: MAL fast path (Frieren ep1)", true, async {
        let ids = ExternalIds { mal: Some(52991), ..Default::default() };
        let p = engine.provider(ProviderId::ZokoAnime).unwrap();
        let mut s = p
            .sources_by_ids(&ids, MediaKind::Anime, Target::Episode { season: 1, episode: 1 }, AudioMode::Sub)
            .await?
            .unwrap_or_default();
        engine.verify(&mut s).await;
        let hls = s.iter().find(|x| x.format == universeapi::model::StreamFormat::Hls);
        ensure(hls.is_some_and(|h| h.verified == Some(true)), format!("no verified HLS in {} sources", s.len()))?;
        anyhow::Ok(format!("{} sources, {} subtitle tracks", s.len(), hls.map(|h| h.subtitles.len()).unwrap_or(0)))
    })
    .await;

    check(&mut r, "hianime: search → detail → episode", false, async {
        let p = engine.provider(ProviderId::HiAnime).unwrap();
        let hits = p.search("frieren").await?;
        let first = hits.first().ok_or_else(|| anyhow::anyhow!("0 hits"))?;
        let d = p.detail(&first.origin.id).await?;
        let eps = d.seasons.first().map(|s| s.episodes.len()).unwrap_or(0);
        ensure(eps > 0, "no episodes")?;
        ensure(d.summary.ids.mal.is_some(), "MAL id not recovered")?;
        let src = p.sources(&d, Target::Episode { season: 1, episode: 1 }, AudioMode::Sub).await?;
        ensure(!src.is_empty(), "no sources")?;
        anyhow::Ok(format!("{} — {eps} eps, mal {:?}", d.summary.name, d.summary.ids.mal))
    })
    .await;

    // ── Metadata ───────────────────────────────────────────────────────
    check(&mut r, "anilist: trending + detail", true, async {
        let a = engine.provider(ProviderId::Anilist).unwrap();
        let rails = a.rails().await?;
        let items = rails.first().map(|r| r.items.len()).unwrap_or(0);
        ensure(items > 0, "empty trending")?;
        let d = engine.detail(&ProviderRef { provider: ProviderId::Anilist, id: "154587".into() }).await?;
        ensure(d.summary.ids.mal == Some(52991), "Frieren MAL id mismatch")?;
        anyhow::Ok(format!("{} rails, trending {items}", rails.len()))
    })
    .await;

    check(&mut r, "cinemeta: trending + series detail", true, async {
        let c = engine.provider(ProviderId::Cinemeta).unwrap();
        let rails = c.rails().await?;
        ensure(rails.iter().all(|r| !r.items.is_empty()), "empty rail")?;
        let d = engine.detail(&ProviderRef { provider: ProviderId::Cinemeta, id: "series/tt0903747".into() }).await?;
        ensure(d.seasons.len() >= 5, format!("Breaking Bad seasons: {}", d.seasons.len()))?;
        anyhow::Ok(format!("{} rails; BB {} seasons", rails.len(), d.seasons.len()))
    })
    .await;

    let has_tmdb = engine.registry().has_tmdb();
    check(&mut r, "tmdb: trending + tv detail", has_tmdb, async {
        ensure(has_tmdb, "no TMDB key in environment (set UNIVERSEAPI_TMDB_READ_TOKEN)")?;
        let t = engine.provider(ProviderId::Tmdb).unwrap();
        let rails = t.rails().await?;
        let d = t.detail("tv/1396").await?;
        ensure(d.summary.ids.imdb.as_deref() == Some("tt0903747"), "imdb id via external_ids missing")?;
        anyhow::Ok(format!("{} rails; {} seasons", rails.len(), d.seasons.len()))
    })
    .await;

    // ── Engine end-to-end ──────────────────────────────────────────────
    check(&mut r, "engine: home feed", true, async {
        let rails = engine.home().await;
        ensure(rails.len() >= 4, format!("only {} rails", rails.len()))?;
        anyhow::Ok(rails.iter().map(|r| format!("{}:{}", r.id, r.items.len())).collect::<Vec<_>>().join(" "))
    })
    .await;

    check(&mut r, "engine: merged search + anime sources", true, async {
        let hits = engine.search("frieren").await;
        let t = hits
            .iter()
            .find(|h| h.kind == MediaKind::Anime && h.ids.mal.is_some())
            .ok_or_else(|| anyhow::anyhow!("no anime hit with MAL id"))?;
        let rep = engine.sources(t, Target::Episode { season: 1, episode: 1 }, AudioMode::Sub).await;
        ensure(rep.sources.first().is_some_and(|s| s.verified == Some(true)), format!("best source unverified: {:?}", rep.errors))?;
        anyhow::Ok(format!("{} on {} providers; best {} {}", t.name, 1 + t.also_on.len(), rep.sources[0].provider, rep.sources[0].host))
    })
    .await;

    r
}

pub fn print(r: &SmokeReport) {
    println!("UniverseAPI smoke test\n");
    for c in &r.checks {
        let mark = match (c.ok, c.critical) {
            (true, _) => "PASS",
            (false, true) => "FAIL",
            (false, false) => "WARN",
        };
        println!("{mark}  {:>6} ms  {}", c.millis, c.name);
        println!("               {}", c.detail);
    }
    let pass = r.checks.iter().filter(|c| c.ok).count();
    println!("\n{pass}/{} passed, {} critical failure(s)", r.checks.len(), r.critical_failures());
}
