//! Coflix (French films and series on a WordPress theme). Ported from
//! freeflix-cli. Movie and episode pages point at a player aggregator
//! (`cfServers` JSON or a legacy iframe) whose `<li onclick="showVideo('b64')">`
//! entries hold base64 player URLs, which go through the shared embed resolver.

use std::sync::{Arc, LazyLock};

use async_trait::async_trait;
use regex::Regex;
use scraper::{Html, Selector};

use super::{Ctx, Provider};
use crate::cache::ttl;
use crate::error::{Error, Result};
use crate::model::*;
use crate::resolvers::embeds;
use crate::util::{absolutize, b64_decode_str, text_of, year_from};

pub struct Coflix {
    ctx: Arc<Ctx>,
}

macro_rules! sel {
    ($name:ident, $css:expr) => {
        static $name: LazyLock<Selector> = LazyLock::new(|| Selector::parse($css).expect("selector"));
    };
}

sel!(A_HREF, "a[href]");
sel!(IMG, "img");
sel!(H1, "h1");
sel!(IFRAME, "iframe[src]");
sel!(SEASON_BTN, "button[data-season]");
sel!(EP_TITLE, ".cf-episode-title");
sel!(PLAYER_LI, "li[onclick]");
sel!(SPAN, "span");
sel!(OG_DESC, "meta[property=\"og:description\"], meta[name=\"description\"]");

static CARD_HREF: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"/(film|serie)/[^/]+/?$").expect("regex"));
static CF_SERVERS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)var\s+cfServers\s*=\s*(\[.*?\])\s*;").expect("regex"));
static ONCLICK_HREF: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"href=['"]([^'"]+)['"]"#).expect("regex"));
static SHOW_VIDEO: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"showVideo\('([^']+)'").expect("regex"));
static SAISON: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)saison\s*(\d{1,3})").expect("regex"));

/// Players that are dead or need a browser.
const DEAD: &[&str] = &["waaw", "netu", "hqq", "trakx"];
const MAX_PLAYERS: usize = 5;

fn https(src: &str) -> String {
    if let Some(rest) = src.strip_prefix("//") {
        format!("https://{rest}")
    } else {
        src.to_string()
    }
}

pub fn parse_search(html: &str, base: &str) -> Vec<TitleSummary> {
    let doc = Html::parse_document(html);
    let mut out: Vec<TitleSummary> = Vec::new();
    for a in doc.select(&A_HREF) {
        let Some(href) = a.value().attr("href").filter(|h| CARD_HREF.is_match(h)) else { continue };
        let Some(img) = a.select(&IMG).next() else { continue };
        let name = img.value().attr("alt").map(str::trim).unwrap_or_default();
        if name.is_empty() {
            continue;
        }
        let url = absolutize(&format!("{base}/"), href);
        let path = url::Url::parse(&url).map(|u| u.path().to_string()).unwrap_or(url);
        if out.iter().any(|s| s.origin.id == path) {
            continue;
        }
        let kind = if path.contains("/serie/") { MediaKind::Series } else { MediaKind::Movie };
        let mut s = TitleSummary::new(ProviderId::Coflix, path, name, kind);
        s.poster = img.value().attr("src").or(img.value().attr("data-src")).map(https);
        s.badges = vec!["VF".into()];
        out.push(s);
    }
    out
}

/// Player aggregator URL from a movie/episode page.
pub fn players_url(html: &str, base: &str) -> Option<String> {
    let from_servers = CF_SERVERS.captures(html).and_then(|c| {
        let v: serde_json::Value = serde_json::from_str(&c[1]).ok()?;
        v.as_array()?.iter().find_map(|e| e.get("embed_url")?.as_str().map(str::to_string))
    });
    from_servers
        .or_else(|| {
            let doc = Html::parse_document(html);
            doc.select(&IFRAME).next().and_then(|f| f.value().attr("src")).map(str::to_string)
        })
        .map(|u| absolutize(&format!("{base}/"), &https(&u)))
}

/// `<li onclick="showVideo('<b64>')"><span>Name / ...</span>` → (name, url).
pub fn parse_players(html: &str) -> Vec<(String, String)> {
    let doc = Html::parse_document(html);
    doc.select(&PLAYER_LI)
        .filter_map(|li| {
            let raw = SHOW_VIDEO.captures(li.value().attr("onclick")?)?.get(1)?.as_str().to_string();
            let url = b64_decode_str(&raw).filter(|u| u.starts_with("http"))?;
            let name = li.select(&SPAN).next().map(text_of).unwrap_or_default();
            let name = name.split(" /").next().unwrap_or(&name).trim().to_string();
            Some((name, url))
        })
        .filter(|(_, u)| !DEAD.iter().any(|d| u.contains(d)))
        .collect()
}

/// Seasons → (number, [(episode number, episode url)]) from a series page.
pub fn parse_series(html: &str) -> Vec<(u32, Vec<(u32, String)>)> {
    let doc = Html::parse_document(html);
    let mut out = Vec::new();
    let panels: Vec<(String, u32)> = doc
        .select(&SEASON_BTN)
        .enumerate()
        .filter_map(|(i, b)| {
            let key = b.value().attr("data-season")?.to_string();
            let n = SAISON.captures(&text_of(b)).and_then(|c| c[1].parse().ok()).unwrap_or(i as u32 + 1);
            Some((key, n))
        })
        .collect();
    let panel_list: Vec<(Option<String>, u32)> =
        if panels.is_empty() { vec![(None, 1)] } else { panels.into_iter().map(|(k, n)| (Some(k), n)).collect() };
    for (key, number) in panel_list {
        let css = match &key {
            Some(k) if k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') => {
                format!(".cf-episodes-panel[data-panel='{k}'] .cf-episode-item")
            }
            Some(_) => continue,
            None => ".cf-episode-item".to_string(),
        };
        let Ok(sel) = Selector::parse(&css) else { continue };
        let mut eps: Vec<(u32, String)> = Vec::new();
        for (i, item) in doc.select(&sel).enumerate() {
            let Some(url) = item.value().attr("onclick").and_then(|o| ONCLICK_HREF.captures(o)).map(|c| c[1].to_string()) else { continue };
            let title = item.select(&EP_TITLE).next().map(text_of).unwrap_or_default();
            let n = crate::providers::dramachi::episode_number(&title).unwrap_or(i as u32 + 1);
            if !eps.iter().any(|(_, u)| *u == url) {
                eps.push((n, url));
            }
        }
        if !eps.is_empty() {
            out.push((number, eps));
        }
    }
    out
}

impl Coflix {
    pub fn new(ctx: Arc<Ctx>) -> Self {
        Self { ctx }
    }

    fn base(&self) -> Result<String> {
        self.ctx.base(ProviderId::Coflix)
    }

    async fn page(&self, url: &str) -> Result<String> {
        let base = self.base()?;
        let full = if url.starts_with("http") { url.to_string() } else { format!("{base}{url}") };
        self.ctx.http.get_text(&full, &[("Referer", &format!("{base}/"))]).await
    }
}

#[async_trait]
impl Provider for Coflix {
    fn id(&self) -> ProviderId {
        ProviderId::Coflix
    }

    fn kinds(&self) -> &'static [MediaKind] {
        &[MediaKind::Movie, MediaKind::Series]
    }

    async fn search(&self, query: &str) -> Result<Vec<TitleSummary>> {
        let base = self.base()?;
        let url = url::Url::parse_with_params(&format!("{base}/"), &[("s", query)]).map_err(|e| Error::Config(e.to_string()))?;
        Ok(parse_search(&self.page(url.as_str()).await?, &base))
    }

    async fn detail(&self, id: &str) -> Result<TitleDetail> {
        let key = format!("coflix:detail:{id}");
        if let Some(d) = self.ctx.cache.get::<TitleDetail>(&key).await {
            return Ok(d);
        }
        let html = self.page(id).await?;
        let kind = if id.contains("/serie/") { MediaKind::Series } else { MediaKind::Movie };
        let (name, poster, overview) = {
            let doc = Html::parse_document(&html);
            let name = doc.select(&H1).next().map(text_of).filter(|t| !t.is_empty()).unwrap_or_else(|| id.trim_matches('/').rsplit('/').next().unwrap_or(id).replace('-', " "));
            let poster = doc
                .select(&IMG)
                .filter_map(|i| i.value().attr("src"))
                .find(|s| s.contains("image.tmdb.org/t/p/"))
                .map(|s| crate::providers::hdhub::tmdb_size(&https(s), "w500"));
            let overview = doc.select(&OG_DESC).next().and_then(|m| m.value().attr("content")).map(str::to_string).filter(|x| !x.is_empty());
            (name, poster, overview)
        };
        let mut s = TitleSummary::new(ProviderId::Coflix, id, name.clone(), kind);
        s.year = year_from(&name);
        s.poster = poster;
        s.overview = overview;
        let mut d = TitleDetail::from_summary(s);
        if kind == MediaKind::Series {
            for (number, eps) in parse_series(&html) {
                d.seasons.push(Season {
                    number,
                    name: None,
                    episodes: eps
                        .into_iter()
                        .map(|(n, url)| Episode { season: number, number: n, title: None, overview: None, air_date: None, thumbnail: None, key: Some(url) })
                        .collect(),
                });
            }
        }
        self.ctx.cache.put(&key, &d, ttl::DETAIL).await;
        Ok(d)
    }

    async fn sources(&self, detail: &TitleDetail, target: Target, _mode: AudioMode) -> Result<Vec<Source>> {
        let page_url = match target {
            Target::Movie => detail.summary.origin.id.clone(),
            Target::Episode { season, episode } => detail
                .seasons
                .iter()
                .filter(|s| s.number == season)
                .flat_map(|s| &s.episodes)
                .find(|e| e.number == episode)
                .and_then(|e| e.key.clone())
                .ok_or_else(|| Error::NotFound(format!("coflix S{season}E{episode}")))?,
            Target::Season { .. } => return Ok(Vec::new()),
        };
        let base = self.base()?;
        let html = self.page(&page_url).await?;
        let agg = players_url(&html, &base).ok_or_else(|| Error::parse("coflix page", "no player aggregator"))?;
        let list = self.ctx.http.get_text(&agg, &[("Referer", &format!("{base}/"))]).await?;
        let players: Vec<(String, String)> = parse_players(&list).into_iter().take(MAX_PLAYERS).collect();
        if players.is_empty() {
            return Err(Error::NotFound("coflix lists no usable player".into()));
        }
        let results = futures::future::join_all(
            players.iter().map(|(name, url)| embeds::resolve(&self.ctx.http, ProviderId::Coflix, url, name, Some(agg.as_str()))),
        )
        .await;
        let mut out = Vec::new();
        let mut last_err = None;
        for r in results {
            match r {
                Ok(list) => {
                    for mut s in list {
                        // Coflix serves French dubs unless the player says otherwise.
                        s.tags.french = s.tags.french.or(Some(crate::release::FrenchVariant::Vf));
                        if s.tags.languages.is_empty() {
                            s.tags.languages = vec!["fr".into()];
                        }
                        out.push(s);
                    }
                }
                Err(e) => last_err = Some(e),
            }
        }
        if out.is_empty() {
            return Err(last_err.unwrap_or_else(|| Error::NotFound("no coflix player resolved".into())));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;

    #[test]
    fn search_cards() {
        let html = r#"<a href="https://coflix.ac/film/dune/"><img alt="Dune" src="//image.tmdb.org/t/p/w342/n.jpg"></a>
            <a href="https://coflix.ac/film/dune/">Dune text link</a>
            <a href="/serie/dune-prophecy/"><img alt="Dune : Prophecy" src="//image.tmdb.org/t/p/w342/f.jpg"></a>
            <a href="https://coflix.ac/genre/sf/"><img alt="SF"></a>"#;
        let r = parse_search(html, "https://coflix.ac");
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].origin.id, "/film/dune/");
        assert_eq!(r[0].poster.as_deref(), Some("https://image.tmdb.org/t/p/w342/n.jpg"));
        assert_eq!(r[1].kind, MediaKind::Series);
    }

    #[test]
    fn aggregator_and_players() {
        let page = r#"<script>var cfServers = [{"nombre":"Lecteurvideo","embed_url":"https:\/\/lecteurvideo.com\/embed.php?id=34359"}];</script>"#;
        assert_eq!(players_url(page, "https://coflix.ac").as_deref(), Some("https://lecteurvideo.com/embed.php?id=34359"));
        let b = base64::engine::general_purpose::STANDARD;
        let list = format!(
            r#"<li onclick="showVideo('{}')"><span>VoeSx Vidéo 11 / pubs</span></li><li onclick="showVideo('{}')"><span>Netu</span></li>"#,
            b.encode("https://voe.sx/e/jzdwwmlffkom"),
            b.encode("https://waaw.to/f/x")
        );
        assert_eq!(parse_players(&list), vec![("VoeSx Vidéo 11".to_string(), "https://voe.sx/e/jzdwwmlffkom".to_string())]);
    }

    #[test]
    fn series_panels() {
        let html = r#"<button data-season="0">Saison 1 <span class="cf-server-tab-lang">VF</span></button><button data-season="1">Saison 2</button>
            <div class="cf-episodes-panel" data-panel="0"><div class="cf-episode-item" onclick="window.location.href='https://coflix.ac/episode/x-1x1/'"><span class="cf-episode-title">Episode 1</span></div>
            <div class="cf-episode-item" onclick="window.location.href='https://coflix.ac/episode/x-1x2/'"><span class="cf-episode-title">Episode 2</span></div></div>
            <div class="cf-episodes-panel" data-panel="1"><div class="cf-episode-item" onclick="window.location.href='https://coflix.ac/episode/x-2x1/'"><span class="cf-episode-title">Episode 1</span></div></div>"#;
        let s = parse_series(html);
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].0, 1);
        assert_eq!(s[0].1, vec![(1, "https://coflix.ac/episode/x-1x1/".to_string()), (2, "https://coflix.ac/episode/x-1x2/".to_string())]);
        assert_eq!(s[1].0, 2);
    }
}
