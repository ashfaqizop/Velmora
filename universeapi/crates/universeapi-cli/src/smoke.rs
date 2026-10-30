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

    check(&mut r, "moviebox: series episode sources", false, async {
        let hits = mb.search("breaking bad").await?;
        let show = hits.iter().find(|h| h.kind.is_episodic()).ok_or_else(|| anyhow::anyhow!("no series hit"))?;
        let d = mb.detail(&show.origin.id).await?;
        ensure(!d.seasons.is_empty(), "no seasons parsed")?;
        let mut src = mb.sources(&d, Target::Episode { season: 1, episode: 1 }, AudioMode::Sub).await?;
        engine.verify(&mut src).await;
        let ok = src.iter().filter(|s| s.verified == Some(true)).count();
        ensure(ok > 0, format!("0 verified of {}", src.len()))?;
        anyhow::Ok(format!("{} — {} seasons, S1E1 {ok} verified", d.summary.name, d.seasons.len()))
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
