//! HDHub4u — ported from the custom HDHub API client (v3) with the E2E fixes:
//! * categories scoped to `.page-meta` (was picking up the nav menu)
//! * ids are mirror-independent slugs (search permalinks point at old mirrors)
//! * episode / pack structure parsed from the page's heading flow
//! * stream links (hdstream4u) resolved, junk HubCloud servers dropped

use std::collections::BTreeMap;
use std::sync::{Arc, LazyLock};

use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use serde::Deserialize;

use super::{Ctx, Provider};
use crate::cache::ttl;
use crate::error::{Error, Result};
use crate::model::*;
use crate::release::ReleaseTags;
use crate::resolvers::{self, HostKind, ResolveCtx};
use crate::util::{absolutize, b64_decode_str, collapse_ws, text_of, year_from};

pub struct HdHub {
    ctx: Arc<Ctx>,
}

macro_rules! sel {
    ($name:ident, $css:expr) => {
        static $name: LazyLock<Selector> = LazyLock::new(|| Selector::parse($css).expect("selector"));
    };
}

sel!(POSTER_IMG, "img.nx-poster");
sel!(H1, "h1");
sel!(META_CAT, ".page-meta a[href*='/category/']");
sel!(IMG, "img[src]");
sel!(IMDB, "a[href*='imdb.com/title/']");
sel!(IFRAME, "iframe[src]");
sel!(FLOW, ".page-body h2, .page-body h3, .page-body h4, .page-body h5, .page-body a[href]");
sel!(PAGE_NUM, ".page-numbers");

static EPISODE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\bep(?:isode)?[\s.\-]*0*(\d{1,4})\b").expect("regex"));
static SEASON: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\bseason\s*0*(\d{1,2})\b|\bS0*(\d{1,2})\b").expect("regex"));

const MIRROR_KEY: &str = "hdhub:mirror";

impl HdHub {
    pub fn new(ctx: Arc<Ctx>) -> Self {
        Self { ctx }
    }

    /// Current mirror. Resolution order (all remote-configurable):
    /// 1. `registry.hdhub.force_mirror` — emergency override
    /// 2. cached result of a previous resolution (24 h)
    /// 3. live host-config CDN (`registry.hdhub.host_cdn`)
    /// 4. first entry of `registry.mirrors.hd_hub`
    pub async fn mirror(&self) -> Result<String> {
        let reg = self.ctx.registry();
        if let Some(forced) = reg.hdhub.force_mirror.as_deref().filter(|m| m.starts_with("http")) {
            return Ok(forced.trim_end_matches('/').to_string());
        }
        if let Some(m) = self.ctx.cache.get::<String>(MIRROR_KEY).await {
            return Ok(m);
        }
        let mirror = match self.live_mirror().await {
            Some(m) => m,
            None => self.ctx.base(ProviderId::HdHub)?,
        };
        self.ctx.cache.put(MIRROR_KEY, &mirror, ttl::MIRROR).await;
        Ok(mirror)
    }

    /// Ask HDHub's host-config CDN which mirror is live (`{"c": base64(url)}`).
    pub async fn live_mirror(&self) -> Option<String> {
        let reg = self.ctx.registry();
        let stamp = chrono::Utc::now().format("1a%Y%m%d%H%M").to_string();
        let url = format!("{}?v={stamp}", reg.hdhub.host_cdn);
        match self.ctx.http.get_json::<serde_json::Value>(&url, &[]).await {
            Ok(v) => v["c"]
                .as_str()
                .and_then(b64_decode_str)
                .map(|c| c.split('?').next().unwrap_or_default().trim_end_matches('/').to_string())
                .filter(|m| m.starts_with("http")),
            Err(e) => {
                tracing::warn!("hdhub host-config failed: {e}");
                None
            }
        }
    }

    async fn fetch_from(&self, base: &str, path: &str) -> Result<String> {
        let url = format!("{base}/{path}");
        self.ctx.http.get_text(&url, &[("Referer", &format!("{base}/"))]).await
    }

    /// Fetch a site path, failing over across mirrors when the current one
    /// is dead (domain rotated mid-session). The working mirror is cached.
    async fn get_page(&self, path: &str) -> Result<String> {
        let path = path.trim_start_matches('/');
        let first = self.mirror().await?;
        let err = match self.fetch_from(&first, path).await {
            Ok(html) => return Ok(html),
            Err(e) if is_mirror_failure(&e) => e,
            Err(e) => return Err(e),
        };
        tracing::warn!(mirror = %first, "hdhub mirror failed ({err}); failing over");
        self.ctx.cache.invalidate(MIRROR_KEY).await;

        let reg = self.ctx.registry();
        let mut candidates: Vec<String> = Vec::new();
        if let Some(live) = self.live_mirror().await {
            candidates.push(live);
        }
        for m in reg.mirrors(ProviderId::HdHub) {
            let m = m.trim_end_matches('/').to_string();
            if !candidates.contains(&m) {
                candidates.push(m);
            }
        }
        for base in candidates.into_iter().filter(|c| *c != first) {
            if let Ok(html) = self.fetch_from(&base, path).await {
                self.ctx.cache.put(MIRROR_KEY, &base, ttl::MIRROR).await;
                return Ok(html);
            }
        }
        Err(err)
    }

    pub async fn listing(&self, category: &str, page: u32) -> Result<(Vec<TitleSummary>, u32)> {
        let path = match (category.is_empty(), page <= 1) {
            (true, true) => String::new(),
            (true, false) => format!("page/{page}/"),
            (false, true) => format!("category/{category}/"),
            (false, false) => format!("category/{category}/page/{page}/"),
        };
        let html = self.get_page(&path).await?;
        Ok(parse_listing(&html, page))
    }
}

#[derive(Debug, Deserialize)]
struct TsResponse {
    #[serde(default)]
    hits: Vec<TsHit>,
}

#[derive(Debug, Deserialize)]
struct TsHit {
    document: TsDoc,
}

#[derive(Debug, Deserialize)]
struct TsDoc {
    post_title: String,
    permalink: String,
    #[serde(default)]
    post_thumbnail: Option<String>,
    #[serde(default)]
    category: Vec<String>,
    #[serde(default)]
    imdb_id: Option<String>,
    #[serde(default)]
    stars: Vec<String>,
}

#[async_trait]
impl Provider for HdHub {
    fn id(&self) -> ProviderId {
        ProviderId::HdHub
    }

    fn kinds(&self) -> &'static [MediaKind] {
        &[MediaKind::Movie, MediaKind::Series]
    }

    async fn search(&self, query: &str) -> Result<Vec<TitleSummary>> {
        let reg = self.ctx.registry();
        let base = self.mirror().await?;
        let url = format!(
            "{}/collections/{}/documents/search",
            reg.hdhub.search_url.trim_end_matches('/'),
            reg.hdhub.search_collection
        );
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let params = [
            ("q", query),
            ("query_by", "post_title,category,stars,director,imdb_id"),
            ("query_by_weights", "4,2,2,2,4"),
            ("sort_by", "_text_match:desc,sort_by_date:desc"),
            ("limit", "24"),
            ("highlight_fields", "none"),
            ("use_cache", "true"),
            ("page", "1"),
            ("analytics_tag", today.as_str()),
        ];
        let resp: TsResponse =
            self.ctx.http.get_json_query(&url, &params, &[("Referer", &format!("{base}/"))]).await?;
        Ok(resp.hits.into_iter().map(|h| doc_to_summary(h.document)).collect())
    }

    async fn detail(&self, id: &str) -> Result<TitleDetail> {
        let key = format!("hdhub:detail:{id}");
        if let Some(d) = self.ctx.cache.get::<TitleDetail>(&key).await {
            return Ok(d);
        }
        let html = self.get_page(&format!("{}/", id.trim_matches('/'))).await?;
        let detail = parse_detail(&html, id, &self.ctx.registry().host_aliases)?;
        self.ctx.cache.put(&key, &detail, ttl::DETAIL).await;
        Ok(detail)
    }

    async fn sources(&self, detail: &TitleDetail, target: Target, _mode: AudioMode) -> Result<Vec<Source>> {
        let have_web = self.ctx.web.is_some();
        let reg = self.ctx.registry();
        let aliases = &reg.host_aliases;
        let wanted: Vec<&HostedLink> = detail
            .links
            .iter()
            .filter(|l| match target {
                Target::Movie => l.episode.is_none() && !l.is_pack,
                Target::Episode { episode, .. } => l.episode == Some(episode),
                Target::Season { .. } => l.is_pack,
            })
            .filter(|l| match HostKind::detect_with(&l.url, aliases) {
                Some(HostKind::HubStream) => false,
                Some(HostKind::GreenMotors) => have_web,
                Some(_) => true,
                None => false,
            })
            .collect();
        // Movies list the season-less links; if a series has no per-episode
        // links at all, fall back to packs so the user still gets something.
        let wanted = if wanted.is_empty() && matches!(target, Target::Episode { .. }) {
            detail.links.iter().filter(|l| l.is_pack && HostKind::detect_with(&l.url, aliases).is_some()).collect()
        } else {
            wanted
        };

        let rctx = ResolveCtx { http: &self.ctx.http, provider: ProviderId::HdHub, web: self.ctx.web.as_deref(), aliases };
        let rctx = &rctx;
        // Build the futures up front: a borrowing closure inside the stream trips
        // rustc's higher-ranked lifetime check in Send futures (rust-lang#64552).
        let futs: Vec<_> = wanted.into_iter().map(|l| resolvers::resolve(rctx, l)).collect();
        let results: Vec<Result<Vec<Source>>> = stream::iter(futs).buffer_unordered(6).collect().await;

        let mut out = Vec::new();
        let mut last_err = None;
        for r in results {
            match r {
                Ok(s) => out.extend(s),
                Err(e) => {
                    tracing::debug!("hdhub link failed: {e}");
                    last_err = Some(e);
                }
            }
        }
        if out.is_empty() {
            if let Some(e) = last_err {
                return Err(e);
            }
        }
        Ok(out)
    }

    async fn rails(&self) -> Result<Vec<Rail>> {
        let reg = self.ctx.registry();
        let futs = reg.hdhub.rails.iter().map(|(id, title, cat)| async move {
            let key = format!("hdhub:rail:{id}");
            if let Some(items) = self.ctx.cache.get::<Vec<TitleSummary>>(&key).await {
                return Ok::<_, Error>(Rail { id: id.clone(), title: title.clone(), items });
            }
            let (items, _) = self.listing(cat, 1).await?;
            self.ctx.cache.put(&key, &items, ttl::RAIL).await;
            Ok(Rail { id: id.clone(), title: title.clone(), items })
        });
        let rails: Vec<Rail> = futures::future::join_all(futs).await.into_iter().filter_map(|r| r.ok()).collect();
        if rails.is_empty() {
            return Err(Error::provider(ProviderId::HdHub, "all rails failed"));
        }
        Ok(rails)
    }
}

/// Errors that mean "this mirror is gone" rather than "this page is missing".
fn is_mirror_failure(e: &Error) -> bool {
    match e {
        Error::Network { .. } | Error::Blocked { .. } => true,
        Error::Http { status, .. } => matches!(status, 403 | 410 | 451 | 500..=599),
        _ => false,
    }
}

/// Slug from any mirror's permalink: `https://new2.hdhub4u.free/foo-bar/` → `foo-bar`.
pub fn slug_of(url: &str) -> String {
    url::Url::parse(url)
        .map(|u| u.path().trim_matches('/').to_string())
        .unwrap_or_else(|_| url.trim_matches('/').to_string())
}

/// `"Avengers: Endgame (2019) BluRay [Hindi (DD5.1) & English] 1080p ..."` → `"Avengers: Endgame"`.
pub fn clean_title(raw: &str) -> String {
    static CUT: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)\s*(\(\s*(19|20)\d{2}|\[|\|\s*|\b(season|s\d{1,2}\b|web-?dl|webrip|bluray|hdrip|hdtc|hdcam|hdts|ds4k|4k|\d{3,4}p|full movie|hindi|dual audio)\b)").expect("regex")
    });
    let decoded = crate::util::unescape_html(raw);
    let cut = CUT.find(&decoded).map(|m| &decoded[..m.start()]).unwrap_or(&decoded);
    let out = cut.trim().trim_end_matches([':', '-', '–', '(']).trim().to_string();
    if out.is_empty() {
        collapse_ws(&decoded)
    } else {
        out
    }
}

fn kind_from(title: &str, categories: &[String]) -> MediaKind {
    let t = title.to_ascii_lowercase();
    let series = t.contains("season") || t.contains("series") || t.contains("episode")
        || categories.iter().any(|c| c.to_ascii_lowercase().replace(['-', ' '], "") == "webseries");
    if series {
        MediaKind::Series
    } else {
        MediaKind::Movie
    }
}

fn badges_for(title: &str, categories: &[String]) -> Vec<String> {
    let mut b: Vec<String> = ReleaseTags::parse(title).badges().into_iter().filter(|x| x != "Pack").take(3).collect();
    if categories.iter().any(|c| c.trim() == "18+" || c.eq_ignore_ascii_case("adult")) {
        b.push("18+".into());
    }
    b
}

fn doc_to_summary(d: TsDoc) -> TitleSummary {
    let name = clean_title(&d.post_title);
    let kind = kind_from(&d.post_title, &d.category);
    let mut s = TitleSummary::new(ProviderId::HdHub, slug_of(&d.permalink), name, kind);
    s.year = year_from(&d.post_title);
    s.poster = d.post_thumbnail.map(|p| tmdb_size(&p, "w500"));
    s.ids.imdb = d.imdb_id.as_deref().and_then(ExternalIds::normalize_imdb);
    s.badges = badges_for(&d.post_title, &d.category);
    if !d.stars.is_empty() {
        s.overview = Some(format!("Starring {}", d.stars.join(", ")));
    }
    s
}

/// Rewrite a TMDB image URL to another size bucket.
pub fn tmdb_size(url: &str, size: &str) -> String {
    static SZ: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"/t/p/(w\d+|h\d+|original)/").expect("regex"));
    if url.contains("image.tmdb.org") {
        SZ.replace(url, format!("/t/p/{size}/").as_str()).into_owned()
    } else {
        url.to_string()
    }
}

pub fn parse_listing(html: &str, page: u32) -> (Vec<TitleSummary>, u32) {
    let doc = Html::parse_document(html);
    let mut items = Vec::new();
    for img in doc.select(&POSTER_IMG) {
        let Some(a) = img.ancestors().filter_map(ElementRef::wrap).find(|e| e.value().name() == "a") else { continue };
        let Some(href) = a.value().attr("href") else { continue };
        let raw = img.value().attr("alt").unwrap_or_default();
        let mut s = TitleSummary::new(ProviderId::HdHub, slug_of(href), clean_title(raw), kind_from(raw, &[]));
        s.year = year_from(raw);
        s.poster = img.value().attr("src").map(|p| tmdb_size(p, "w500"));
        s.badges = badges_for(raw, &[]);
        items.push(s);
    }
    let total = doc
        .select(&PAGE_NUM)
        .filter_map(|e| text_of(e).replace(',', "").parse::<u32>().ok())
        .max()
        .unwrap_or(page)
        .max(page);
    (items, total)
}

pub fn parse_detail(html: &str, slug: &str, aliases: &BTreeMap<String, String>) -> Result<TitleDetail> {
    let doc = Html::parse_document(html);
    let raw_title = doc.select(&H1).next().map(text_of).ok_or_else(|| Error::parse("hdhub detail", "no <h1>"))?;

    let categories: Vec<String> = doc
        .select(&META_CAT)
        .map(|a| text_of(a).chars().filter(|c| !('\u{e000}'..='\u{f8ff}').contains(c)).collect::<String>().trim().to_string())
        .filter(|c| !c.is_empty())
        .collect();

    let kind = kind_from(&raw_title, &categories);
    let mut summary = TitleSummary::new(ProviderId::HdHub, slug.trim_matches('/'), clean_title(&raw_title), kind);
    summary.year = year_from(&raw_title);
    summary.badges = badges_for(&raw_title, &categories);

    let mut screenshots = Vec::new();
    for img in doc.select(&IMG) {
        let src = img.value().attr("src").unwrap_or_default();
        if summary.poster.is_none() && src.contains("image.tmdb.org") {
            summary.poster = Some(tmdb_size(src, "w500"));
            summary.backdrop = Some(tmdb_size(src, "original"));
        } else if src.contains("catimages") {
            static TH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\.th\.(jpe?g|png|webp)$").expect("regex"));
            let full = TH.replace(src, ".$1").into_owned();
            if !screenshots.contains(&full) {
                screenshots.push(full);
            }
        }
    }

    if let Some(a) = doc.select(&IMDB).next() {
        summary.ids.imdb = a.value().attr("href").and_then(|h| {
            h.split("/title/").nth(1).and_then(|r| r.split('/').next()).and_then(ExternalIds::normalize_imdb)
        });
        let rating = text_of(a);
        summary.rating = rating.split('/').next().and_then(|r| r.trim().parse::<f32>().ok());
    }

    let trailer = doc
        .select(&IFRAME)
        .filter_map(|f| f.value().attr("src"))
        .find(|s| s.contains("youtube.com/embed"))
        .map(|s| absolutize("https://www.youtube.com/", s));

    let season = SEASON
        .captures(&raw_title)
        .and_then(|c| c.get(1).or_else(|| c.get(2)))
        .and_then(|m| m.as_str().parse::<u32>().ok())
        .or(kind.is_episodic().then_some(1));

    let links = parse_links(&doc, season, aliases);

    let mut seasons = Vec::new();
    if let Some(sn) = season.filter(|_| kind.is_episodic()) {
        let mut eps: Vec<u32> = links.iter().filter_map(|l| l.episode).collect();
        eps.sort_unstable();
        eps.dedup();
        seasons.push(Season {
            number: sn,
            name: None,
            episodes: eps
                .into_iter()
                .map(|n| Episode { season: sn, number: n, title: None, overview: None, air_date: None, thumbnail: None, key: None })
                .collect(),
        });
    }

    let mut d = TitleDetail::from_summary(summary);
    d.genres = categories;
    d.trailer = trailer;
    d.screenshots = screenshots;
    d.seasons = seasons;
    d.links = links;
    Ok(d)
}

/// Walk the post body in document order. `EPiSODE N` headings set the current
/// episode; quality headings label the generic "Drive | Instant | WATCH" anchors.
fn parse_links(doc: &Html, season: Option<u32>, aliases: &BTreeMap<String, String>) -> Vec<HostedLink> {
    let mut out: Vec<HostedLink> = Vec::new();
    let mut episode: Option<u32> = None;
    let mut heading = String::new();

    for el in doc.select(&FLOW) {
        let name = el.value().name();
        if name != "a" {
            let text = text_of(el);
            if let Some(c) = EPISODE.captures(&text) {
                // "EPiSODE 1" alone → new episode block; "Episode 3 – 720p" → both
                episode = c[1].parse().ok();
                let rest = EPISODE.replace(&text, "").trim().trim_start_matches(['–', '-', ':']).trim().to_string();
                if !rest.is_empty() {
                    heading = rest;
                }
            } else if !text.is_empty() {
                heading = text.split(['–', '|']).next().unwrap_or(&text).trim().to_string();
            }
            continue;
        }

        let href = el.value().attr("href").unwrap_or_default();
        let Some(kind) = HostKind::detect_with(href, aliases) else { continue };
        let anchor = text_of(el);
        let generic = anchor.len() < 14 && ReleaseTags::parse(&anchor).resolution.is_none();
        let label = if generic && !heading.is_empty() { format!("{heading} · {anchor}") } else { anchor.clone() };
        let tags = ReleaseTags::parse(&label);
        let is_pack = tags.is_pack;
        if out.iter().any(|l| l.url == href) {
            continue;
        }
        out.push(HostedLink {
            host: kind.label().to_string(),
            url: href.to_string(),
            episode: if is_pack { None } else { episode },
            season,
            is_pack,
            is_stream_page: matches!(kind, HostKind::HdStream4u | HostKind::HubStream | HostKind::HbPlay),
            tags,
            label,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SERIES: &str = r#"<html><body class="postid-198779"><h1>Akka (Season 1) WEB-DL [Hindi DD5.1] 4K 1080p 720p &amp; 480p [x264/ESubs] | NF Series</h1>
      <div class="page-meta"><a href="/category/crime/">&#xe2c7;Crime</a><a href="/category/web-series/">&#xe2c7;WEB-Series</a></div>
      <nav><a href="/category/adult/">18+</a></nav>
      <main class="page-body">
        <img src="https://image.tmdb.org/t/p/w342/cF0.jpg">
        <img src="https://catimages.co/i/abc.th.jpg">
        <h2>: DOWNLOAD LINKS :</h2>
        <h3><a href="https://greenmotors.club/?id=enc">720p x264 [1.6GB]</a></h3>
        <h4><a href="https://hubdrive.bar/file/4237020442">1080p MULTi WEB-DL PACK [10.3GB]</a></h4>
        <h2>: Single Episode x264 Links :</h2>
        <h4>EPiSODE 1</h4>
        <h4>720p – <span><a href="https://hubdrive.bar/file/1">Drive</a></span> | <span><a href="https://hubcdn.club/file/A">Instant</a></span> | <span><a href="https://hubstream.art/#x">WATCH</a></span></h4>
        <h4>1080p – <a href="https://hubdrive.bar/file/2">Drive</a> | <a href="https://hubcdn.club/file/B">Instant</a></h4>
        <h4>EPiSODE 2</h4>
        <h4>720p – <a href="https://hubdrive.bar/file/3">Drive</a></h4>
      </main></body></html>"#;

    #[test]
    fn series_flow() {
        let d = parse_detail(SERIES, "akka-season-1-hindi-webrip-full-series", &BTreeMap::new()).unwrap();
        assert_eq!(d.summary.name, "Akka");
        assert_eq!(d.summary.kind, MediaKind::Series);
        assert_eq!(d.genres, vec!["Crime", "WEB-Series"], "nav menu categories must be ignored");
        assert!(!d.summary.badges.contains(&"18+".to_string()));
        assert_eq!(d.summary.poster.as_deref(), Some("https://image.tmdb.org/t/p/w500/cF0.jpg"));
        assert_eq!(d.screenshots, vec!["https://catimages.co/i/abc.jpg"]);

        let pack = d.links.iter().find(|l| l.url.ends_with("4237020442")).unwrap();
        assert!(pack.is_pack && pack.episode.is_none());

        let e1: Vec<_> = d.links.iter().filter(|l| l.episode == Some(1)).collect();
        assert_eq!(e1.len(), 5);
        assert_eq!(e1[0].label, "720p · Drive");
        assert_eq!(e1[0].tags.resolution, Some(720));
        assert_eq!(e1[3].label, "1080p · Drive");
        assert_eq!(d.links.iter().filter(|l| l.episode == Some(2)).count(), 1);
        assert_eq!(d.seasons[0].episodes.len(), 2);
        assert_eq!(d.links.iter().find(|l| l.url.contains("greenmotors")).unwrap().episode, None);
    }

    #[test]
    fn movie_flow() {
        let html = r#"<h1>Jurassic World: Rebirth (2025) DS4K WEB-DL [Hindi (DD5.1) &amp; English] 1080p</h1>
          <a href="https://www.imdb.com/title/tt31036941/">6.3/10</a>
          <iframe src="https://www.youtube.com/embed/qTacXD5J9PA"></iframe>
          <div class="page-body"><h3><a href="https://hubcdn.wiki/file/f">480p⚡[500MB]</a></h3>
          <h4>WATCH | PLAYER-2 <a href="https://hdstream4u.com/file/t1">WATCH</a></h4></div>"#;
        let d = parse_detail(html, "jurassic", &BTreeMap::new()).unwrap();
        assert_eq!(d.summary.name, "Jurassic World: Rebirth");
        assert_eq!(d.summary.year, Some(2025));
        assert_eq!(d.summary.kind, MediaKind::Movie);
        assert_eq!(d.summary.ids.imdb.as_deref(), Some("tt31036941"));
        assert_eq!(d.summary.rating, Some(6.3));
        assert_eq!(d.trailer.as_deref(), Some("https://www.youtube.com/embed/qTacXD5J9PA"));
        assert_eq!(d.links.len(), 2);
        assert!(d.links.iter().all(|l| l.episode.is_none()));
        assert!(d.links[1].is_stream_page);
        assert!(d.seasons.is_empty());
    }

    #[test]
    fn titles_and_slugs() {
        assert_eq!(clean_title("Avengers: Endgame (2019) BluRay [Hindi (DD5.1) & English] 1080p"), "Avengers: Endgame");
        assert_eq!(clean_title("Hostel Haadsa (Season 1) WEB-DL [Hindi DD2.0]"), "Hostel Haadsa");
        assert_eq!(clean_title("Bigg Boss (Season 20) WEB-DL [Hindi 2.0]"), "Bigg Boss");
        assert_eq!(slug_of("https://new2.hdhub4u.free/avengers-endgame-2019/"), "avengers-endgame-2019");
        assert_eq!(tmdb_size("https://image.tmdb.org/t/p/w200/x.jpg", "w500"), "https://image.tmdb.org/t/p/w500/x.jpg");
    }

    #[test]
    fn listing() {
        let html = r#"<a href="https://new10.hdhub4u.cl/akka-season-1/"><img class="nx-poster" alt="Akka (Season 1) WEB-DL" src="https://image.tmdb.org/t/p/w342/a.jpg"></a>
                      <a class="page-numbers">2</a><a class="page-numbers">970</a>"#;
        let (items, total) = parse_listing(html, 1);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].origin.id, "akka-season-1");
        assert_eq!(items[0].kind, MediaKind::Series);
        assert_eq!(total, 970);
    }
}
