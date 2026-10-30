//! 4KHDHub: high-bitrate 4K/1080p releases. Ported from MovieBox-TUI's
//! parser. Every download link sits behind a GreenMotors gate, which the
//! shared resolver decodes over plain HTTP before following HubCloud/HubDrive.

use std::collections::BTreeMap;
use std::sync::{Arc, LazyLock};

use async_trait::async_trait;
use futures::stream::{self, StreamExt};
use regex::Regex;
use scraper::{ElementRef, Html, Selector};

use super::{Ctx, Provider};
use crate::cache::ttl;
use crate::error::{Error, Result};
use crate::model::*;
use crate::release::ReleaseTags;
use crate::resolvers::{self, HostKind, ResolveCtx};
use crate::util::{text_of, year_from};

pub struct FourKHdHub {
    ctx: Arc<Ctx>,
}

macro_rules! sel {
    ($name:ident, $css:expr) => {
        static $name: LazyLock<Selector> = LazyLock::new(|| Selector::parse($css).expect("selector"));
    };
}

sel!(CARD, "a.movie-card");
sel!(CARD_TITLE, ".movie-card-title");
sel!(CARD_META, ".movie-card-meta");
sel!(IMG, "img");
sel!(H1, "h1");
sel!(DESC, ".content-section p.mt-4");
sel!(META_DESC, "meta[name=\"description\"]");
sel!(OG_IMAGE, "meta[property=\"og:image\"]");
sel!(IMDB, ".imdb-score");
sel!(GENRE, ".badge-outline a");
sel!(META_ITEM, ".metadata-item");
sel!(META_LABEL, ".metadata-label");
sel!(META_VALUE, ".metadata-value");
sel!(FILE_ITEM, ".download-item");
sel!(FILE_TITLE, ".file-title");
sel!(EP_ITEM, "#episodes .episode-download-item");
sel!(EP_TITLE, ".episode-file-title");
sel!(SIZE, ".badge-size, .badge");
sel!(LINK, "a[href]");

static SXEY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\bS(\d{1,2})(?:[ .]?E(\d{1,4}))?\b").expect("regex"));

const GENRES: &[&str] = &[
    "action", "adventure", "animation", "comedy", "crime", "documentary", "drama", "family", "fantasy", "history",
    "horror", "music", "mystery", "romance", "science fiction", "sci-fi", "thriller", "war", "western",
];

impl FourKHdHub {
    pub fn new(ctx: Arc<Ctx>) -> Self {
        Self { ctx }
    }

    fn base(&self) -> Result<String> {
        self.ctx.base(ProviderId::FourKHdHub)
    }

    async fn page(&self, path: &str) -> Result<String> {
        let base = self.base()?;
        let url = if path.starts_with("http") { path.to_string() } else { format!("{base}/{}", path.trim_start_matches('/')) };
        self.ctx.http.get_text(&url, &[("Referer", &format!("{base}/"))]).await
    }
}

/// (season, episode) from a scene name; `episode` is `None` for season packs.
pub fn season_episode(name: &str) -> Option<(u32, Option<u32>)> {
    let c = SXEY.captures(name)?;
    let season = c.get(1)?.as_str().parse().ok()?;
    let episode = c.get(2).and_then(|e| e.as_str().parse().ok());
    Some((season, episode))
}

fn metadata(doc: &Html, label: &str) -> Option<String> {
    doc.select(&META_ITEM).find_map(|item| {
        let l = item.select(&META_LABEL).next().map(text_of)?;
        if l.trim_end_matches(':').eq_ignore_ascii_case(label) {
            item.select(&META_VALUE).next().map(text_of).filter(|v| !v.is_empty())
        } else {
            None
        }
    })
}

fn strip_year(title: &str) -> String {
    static Y: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s*\((19|20)\d{2}\)\s*$").expect("regex"));
    Y.replace(title.trim(), "").trim().to_string()
}

pub fn parse_search(html: &str) -> Vec<TitleSummary> {
    let doc = Html::parse_document(html);
    let mut out = Vec::new();
    for card in doc.select(&CARD) {
        let Some(href) = card.value().attr("href") else { continue };
        let path = url::Url::parse(href).map(|u| u.path().to_string()).unwrap_or_else(|_| href.to_string());
        let Some(name) = card.select(&CARD_TITLE).next().map(text_of).filter(|t| !t.is_empty()) else { continue };
        let meta = card.select(&CARD_META).next().map(text_of).unwrap_or_default();
        let kind = if path.contains("-series-") { MediaKind::Series } else { MediaKind::Movie };
        let mut s = TitleSummary::new(ProviderId::FourKHdHub, path, name, kind);
        s.year = year_from(&meta);
        s.poster = card.select(&IMG).next().and_then(|i| i.value().attr("src")).map(str::to_string);
        if let Some(seasons) = meta.split('•').nth(1).map(str::trim).filter(|x| !x.is_empty()) {
            s.badges.push(seasons.to_string());
        }
        out.push(s);
    }
    out
}

pub fn parse_detail(html: &str, id: &str, aliases: &BTreeMap<String, String>) -> Result<TitleDetail> {
    let doc = Html::parse_document(html);
    let raw = doc
        .select(&H1)
        .map(text_of)
        .find(|t| !t.is_empty())
        .ok_or_else(|| Error::parse("4khdhub detail", "no title"))?;
    let kind = if id.contains("-series-") { MediaKind::Series } else { MediaKind::Movie };
    let mut s = TitleSummary::new(ProviderId::FourKHdHub, id, strip_year(&raw), kind);
    s.year = metadata(&doc, "Release")
        .or_else(|| metadata(&doc, "Last Air"))
        .as_deref()
        .and_then(year_from)
        .or_else(|| year_from(&raw));
    s.poster = doc.select(&OG_IMAGE).next().and_then(|m| m.value().attr("content")).map(str::to_string);
    s.rating = doc.select(&IMDB).next().map(text_of).and_then(|r| r.parse().ok());
    s.overview = doc
        .select(&DESC)
        .map(text_of)
        .find(|t| !t.is_empty())
        .or_else(|| doc.select(&META_DESC).next().and_then(|m| m.value().attr("content")).map(str::to_string));

    // Page-level audio languages apply to files that don't name their own.
    let page_langs = metadata(&doc, "Audios").map(|a| ReleaseTags::parse(&a).languages).unwrap_or_default();
    if page_langs.len() >= 2 {
        s.badges.push("Dual Audio".into());
    }

    let mut d = TitleDetail::from_summary(s);
    d.genres = doc.select(&GENRE).map(text_of).filter(|g| GENRES.contains(&g.to_ascii_lowercase().as_str())).collect();
    d.directors = metadata(&doc, "Director").into_iter().collect();
    d.cast = metadata(&doc, "Stars")
        .map(|s| s.split(',').map(|x| x.split('(').next().unwrap_or(x).trim().to_string()).filter(|x| !x.is_empty()).collect())
        .unwrap_or_default();

    let mut links = Vec::new();
    collect_links(&doc, &FILE_ITEM, &FILE_TITLE, &page_langs, aliases, &mut links);
    collect_links(&doc, &EP_ITEM, &EP_TITLE, &page_langs, aliases, &mut links);

    let mut seasons: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for l in &links {
        if let (Some(sn), Some(ep)) = (l.season, l.episode) {
            let eps = seasons.entry(sn).or_default();
            if !eps.contains(&ep) {
                eps.push(ep);
            }
        }
    }
    d.seasons = seasons
        .into_iter()
        .map(|(number, mut eps)| {
            eps.sort_unstable();
            Season {
                number,
                name: None,
                episodes: eps
                    .into_iter()
                    .map(|n| Episode { season: number, number: n, title: None, overview: None, air_date: None, thumbnail: None, key: None })
                    .collect(),
            }
        })
        .collect();
    d.links = links;
    Ok(d)
}

fn collect_links(
    doc: &Html,
    item_sel: &Selector,
    title_sel: &Selector,
    page_langs: &[String],
    aliases: &BTreeMap<String, String>,
    out: &mut Vec<HostedLink>,
) {
    for item in doc.select(item_sel) {
        let Some(file) = item.select(title_sel).next().map(text_of).filter(|t| !t.is_empty()) else { continue };
        let size = item.select(&SIZE).map(text_of).find(|t| crate::release::parse_size(t).is_some()).unwrap_or_default();
        let mut tags = ReleaseTags::parse(&format!("{file} {size}"));
        if tags.languages.is_empty() {
            tags.languages = page_langs.to_vec();
            tags.multi_audio |= page_langs.len() >= 2;
        }
        let se = season_episode(&file);
        let is_pack = tags.is_pack || matches!(se, Some((_, None))) || StreamFormat::from_url_or_name(&file) == StreamFormat::Archive;
        tags.is_pack = is_pack;
        for a in item.select(&LINK) {
            push_link(a, &file, &tags, se, is_pack, aliases, out);
        }
    }
}

fn push_link(
    a: ElementRef<'_>,
    file: &str,
    tags: &ReleaseTags,
    se: Option<(u32, Option<u32>)>,
    is_pack: bool,
    aliases: &BTreeMap<String, String>,
    out: &mut Vec<HostedLink>,
) {
    let Some(href) = a.value().attr("href").filter(|h| h.starts_with("https://")) else { return };
    let Some(kind) = HostKind::detect_with(href, aliases) else { return };
    if out.iter().any(|l| l.url == href) {
        return;
    }
    let via = text_of(a).trim_start_matches("Download").trim().to_string();
    out.push(HostedLink {
        label: if via.is_empty() { file.to_string() } else { format!("{file} · {via}") },
        url: href.to_string(),
        host: kind.label().to_string(),
        tags: tags.clone(),
        season: se.map(|(s, _)| s),
        episode: se.and_then(|(_, e)| e),
        is_pack,
        is_stream_page: false,
    });
}

#[async_trait]
impl Provider for FourKHdHub {
    fn id(&self) -> ProviderId {
        ProviderId::FourKHdHub
    }

    fn kinds(&self) -> &'static [MediaKind] {
        &[MediaKind::Movie, MediaKind::Series]
    }

    async fn search(&self, query: &str) -> Result<Vec<TitleSummary>> {
        let base = self.base()?;
        let url = url::Url::parse_with_params(&format!("{base}/"), &[("s", query)]).map_err(|e| Error::Config(e.to_string()))?;
        Ok(parse_search(&self.page(url.as_str()).await?))
    }

    async fn detail(&self, id: &str) -> Result<TitleDetail> {
        let key = format!("4khdhub:detail:{id}");
        if let Some(d) = self.ctx.cache.get::<TitleDetail>(&key).await {
            return Ok(d);
        }
        let html = self.page(id).await?;
        let d = parse_detail(&html, id, &self.ctx.registry().host_aliases)?;
        self.ctx.cache.put(&key, &d, ttl::DETAIL).await;
        Ok(d)
    }

    async fn sources(&self, detail: &TitleDetail, target: Target, _mode: AudioMode) -> Result<Vec<Source>> {
        let wanted: Vec<&HostedLink> = detail
            .links
            .iter()
            .filter(|l| match target {
                Target::Movie => !l.is_pack && l.episode.is_none(),
                Target::Episode { season, episode } => l.season == Some(season) && l.episode == Some(episode),
                Target::Season { season } => l.is_pack && l.season == Some(season),
            })
            .collect();
        if wanted.is_empty() {
            return Err(Error::NotFound(format!("4KHDHub has no files for {target:?}")));
        }
        let reg = self.ctx.registry();
        let rctx = ResolveCtx {
            http: &self.ctx.http,
            provider: ProviderId::FourKHdHub,
            web: self.ctx.web.as_deref(),
            aliases: &reg.host_aliases,
        };
        let rctx = &rctx;
        // Futures built up front so the stream holds no borrowing closure (rust-lang#64552).
        let futs: Vec<_> = wanted.into_iter().map(|l| resolvers::resolve(rctx, l)).collect();
        let results: Vec<Result<Vec<Source>>> = stream::iter(futs).buffer_unordered(6).collect().await;

        let mut out = Vec::new();
        let mut last_err = None;
        for r in results {
            match r {
                Ok(s) => out.extend(s),
                Err(e) => last_err = Some(e),
            }
        }
        if out.is_empty() {
            return Err(last_err.unwrap_or_else(|| Error::NotFound("no 4KHDHub mirror resolved".into())));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_numbers() {
        assert_eq!(season_episode("The.Boys.S05E01.Fifteen.Inches.2160p"), Some((5, Some(1))));
        assert_eq!(season_episode("The.Boys.S05.2160p.AMZN.WEB-DL"), Some((5, None)));
        assert_eq!(season_episode("Dune - Part One (2021) 2160p"), None);
    }

    #[test]
    fn search_cards() {
        let html = r#"<a class="movie-card" href="/dune-prophecy-series-1799/"><img src="https://image.tmdb.org/t/p/w500/x.jpg">
            <div class="movie-card-title">Dune: Prophecy</div><div class="movie-card-meta">2024 • S01</div></a>
            <a class="movie-card" href="/dune-movie-195/"><div class="movie-card-title">Dune</div><div class="movie-card-meta">2021</div></a>"#;
        let r = parse_search(html);
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].origin.id, "/dune-prophecy-series-1799/");
        assert_eq!(r[0].kind, MediaKind::Series);
        assert_eq!(r[0].badges, vec!["S01"]);
        assert_eq!(r[1].year, Some(2021));
    }

    #[test]
    fn detail_movie_and_series() {
        let movie = r#"<h1>Dune (2021)</h1><meta property="og:image" content="https://image.tmdb.org/p.jpg">
            <span class="imdb-score">8.0</span><span class="badge-outline"><a>Sci-Fi</a></span><span class="badge-outline"><a>4K</a></span>
            <div class="metadata-item"><span class="metadata-label">Release:</span><span class="metadata-value">2021-09-15</span></div>
            <div class="metadata-item"><span class="metadata-label">Audios:</span><span class="metadata-value">Hindi | English</span></div>
            <div class="download-item"><div class="file-title">Dune - Part One (2021) 1080p BluRay REMUX AVC</div><span class="badge-size">29.57 GB</span>
              <a href="https://greenmotors.club/?id=A">Download HubCloud</a><a href="https://greenmotors.club/?id=B">Download HubDrive</a></div>"#;
        let d = parse_detail(movie, "/dune-movie-195/", &BTreeMap::new()).unwrap();
        assert_eq!(d.summary.name, "Dune");
        assert_eq!(d.summary.year, Some(2021));
        assert_eq!(d.summary.rating, Some(8.0));
        assert_eq!(d.genres, vec!["Sci-Fi"]);
        assert_eq!(d.links.len(), 2);
        assert_eq!(d.links[0].label, "Dune - Part One (2021) 1080p BluRay REMUX AVC · HubCloud");
        assert_eq!(d.links[0].tags.languages, vec!["hi", "en"], "page audios fill in");
        assert!(d.links[0].tags.size_bytes.unwrap() > 29 * 1024 * 1024 * 1024);
        assert!(!d.links[0].is_pack && d.links[0].episode.is_none());

        let series = r#"<h1>The Boys</h1>
            <div class="download-item"><div class="file-title">The.Boys.S05.1080p.AMZN.WEB-DL.H.264</div><span class="badge">37.25 GB</span><a href="https://greenmotors.club/?id=P">Download HubCloud</a></div>
            <div id="episodes"><div class="episode-download-item"><div class="episode-file-title">The.Boys.S05E01.1080p.WEB-DL</div><a href="https://greenmotors.club/?id=E1">Download HubCloud</a></div>
            <div class="episode-download-item"><div class="episode-file-title">The.Boys.S05E02.1080p.WEB-DL</div><a href="https://greenmotors.club/?id=E2">Download HubCloud</a></div></div>"#;
        let d = parse_detail(series, "/the-boys-series-605/", &BTreeMap::new()).unwrap();
        assert_eq!(d.summary.kind, MediaKind::Series);
        let pack = d.links.iter().find(|l| l.url.ends_with("=P")).unwrap();
        assert!(pack.is_pack && pack.season == Some(5) && pack.episode.is_none());
        assert_eq!(d.seasons.len(), 1);
        assert_eq!(d.seasons[0].episodes.len(), 2);
    }
}
