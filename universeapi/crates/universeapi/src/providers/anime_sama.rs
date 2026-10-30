//! Anime-Sama (French anime, VOSTFR and VF). Ported from freeflix-cli.
//!
//! The registry holds a *portal* domain; its "btn-primary" link leads to the
//! current mirror, which is cached for a day. Each season keeps its episodes
//! in `<season>/<lang>/episodes.js` as `var epsN = [...]` arrays, one array
//! per player host. Player pages go through [`crate::resolvers::embeds`].

use std::sync::{Arc, LazyLock};

use async_trait::async_trait;
use regex::Regex;
use scraper::{Html, Selector};

use super::{Ctx, Provider};
use crate::cache::ttl;
use crate::error::{Error, Result};
use crate::model::*;
use crate::release::FrenchVariant;
use crate::resolvers::embeds;
use crate::util::{absolutize, text_of};

const ORIGIN_KEY: &str = "animesama:origin";

pub struct AnimeSama {
    ctx: Arc<Ctx>,
}

macro_rules! sel {
    ($name:ident, $css:expr) => {
        static $name: LazyLock<Selector> = LazyLock::new(|| Selector::parse($css).expect("selector"));
    };
}

sel!(PORTAL_BTN, "a.btn-primary[href]");
sel!(CARDS, "#list_catalog > div");
sel!(SCAN_BADGE, ".as-badge-scan");
sel!(ANIME_BADGE, ".as-badge-anime");
sel!(INFO, "p.info-value");
sel!(LINK, "a[href]");
sel!(TITLE, "h3, h1, h2");
sel!(IMG, "img");
sel!(H1, "h1, h4#titreOeuvre");
sel!(OG_IMAGE, "meta[property=\"og:image\"]");
sel!(OG_TITLE, "meta[property=\"og:title\"]");
sel!(OG_DESC, "meta[property=\"og:description\"], meta[name=\"description\"]");

static PANEL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"panneauAnime\("([^"]*)",\s*"([^"]*)"\)"#).expect("regex"));
static BLOCK_COMMENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)/\*.*?\*/").expect("regex"));
static LINE_COMMENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^\s*//[^\n]*").expect("regex"));
static EPS_ARRAY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)var\s+eps(\d+)\s*=\s*\[(.*?)\];").expect("regex"));
static QUOTED_URL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"['"](https?://[^'"]+)['"]"#).expect("regex"));
static SAISON: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)saison\s*(\d{1,3})").expect("regex"));
static LANG_SUFFIX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"/(vostfr|vf|vf1|vf2|va|vj|vkr|vcn|vqc)/?$").expect("regex"));

impl AnimeSama {
    pub fn new(ctx: Arc<Ctx>) -> Self {
        Self { ctx }
    }

    /// Current mirror, resolved through the portal and cached for a day.
    pub async fn origin(&self) -> Result<String> {
        if let Some(o) = self.ctx.cache.get::<String>(ORIGIN_KEY).await {
            return Ok(o);
        }
        let portal = self.ctx.base(ProviderId::AnimeSama)?;
        let html = self.ctx.http.get_text(&portal, &[]).await?;
        let next = {
            let doc = Html::parse_document(&html);
            doc.select(&PORTAL_BTN).next().and_then(|a| a.value().attr("href")).map(|h| absolutize(&portal, h))
        };
        let origin = match next {
            Some(n) => match self.ctx.http.get(&n, &[]).await {
                Ok(resp) => crate::util::origin_of(resp.url().as_str()).trim_end_matches('/').to_string(),
                Err(_) => n.trim_end_matches('/').to_string(),
            },
            None => portal,
        };
        self.ctx.cache.put(ORIGIN_KEY, &origin, ttl::MIRROR).await;
        Ok(origin)
    }

    async fn episodes_js(&self, season_base: &str, lang: &str) -> Result<Vec<Vec<String>>> {
        let origin = self.origin().await?;
        let url = format!("{season_base}/{lang}/episodes.js");
        let js = self.ctx.http.get_text(&url, &[("Referer", &format!("{origin}/")), ("Origin", origin.as_str())]).await?;
        Ok(parse_episodes_js(&js))
    }
}

/// `var eps1 = ['url', ...]; var eps2 = [...]` → players (outer) × episodes (inner).
pub fn parse_episodes_js(js: &str) -> Vec<Vec<String>> {
    let mut players: Vec<(u32, Vec<String>)> = EPS_ARRAY
        .captures_iter(js)
        .filter_map(|c| {
            let n = c[1].parse().ok()?;
            let urls: Vec<String> = QUOTED_URL.captures_iter(&c[2]).map(|u| u[1].to_string()).collect();
            Some((n, urls))
        })
        .filter(|(_, u)| !u.is_empty())
        .collect();
    players.sort_by_key(|(n, _)| *n);
    players.into_iter().map(|(_, u)| u).collect()
}

pub fn parse_search(html: &str, origin: &str) -> Vec<TitleSummary> {
    let doc = Html::parse_document(html);
    let mut out = Vec::new();
    for card in doc.select(&CARDS) {
        if card.select(&SCAN_BADGE).next().is_some() && card.select(&ANIME_BADGE).next().is_none() {
            continue;
        }
        if card.select(&INFO).any(|p| text_of(p) == "Scans") {
            continue;
        }
        let Some(href) = card.select(&LINK).next().and_then(|a| a.value().attr("href")) else { continue };
        let url = absolutize(&format!("{origin}/"), href);
        let path = url::Url::parse(&url).map(|u| u.path().trim_end_matches('/').to_string()).unwrap_or(url);
        let name = card
            .select(&TITLE)
            .next()
            .map(text_of)
            .filter(|t| !t.is_empty())
            .or_else(|| card.value().attr("data-nom").map(str::to_string))
            .unwrap_or_default();
        if name.is_empty() || path.is_empty() {
            continue;
        }
        let mut s = TitleSummary::new(ProviderId::AnimeSama, path, name, MediaKind::Anime);
        s.poster = card
            .select(&IMG)
            .next()
            .and_then(|i| i.value().attr("src").or(i.value().attr("data-src")))
            .or(card.value().attr("data-img"))
            .map(|p| absolutize(&format!("{origin}/"), p));
        s.badges = vec!["VOSTFR".into(), "VF".into()];
        out.push(s);
    }
    out
}

/// Season panels declared in page JS, with commented-out template seasons removed.
pub fn parse_seasons(html: &str) -> Vec<(String, String)> {
    let script = BLOCK_COMMENT.replace_all(html, "");
    let script = LINE_COMMENT.replace_all(&script, "");
    let mut out: Vec<(String, String)> = Vec::new();
    for c in PANEL.captures_iter(&script) {
        let (name, path) = (c[1].trim().to_string(), c[2].trim().to_string());
        if name.is_empty() || path.is_empty() || name.eq_ignore_ascii_case("nom") || path.eq_ignore_ascii_case("url") {
            continue;
        }
        if !out.iter().any(|(_, p)| *p == path) {
            out.push((name, path));
        }
    }
    out
}

#[async_trait]
impl Provider for AnimeSama {
    fn id(&self) -> ProviderId {
        ProviderId::AnimeSama
    }

    fn kinds(&self) -> &'static [MediaKind] {
        &[MediaKind::Anime]
    }

    async fn search(&self, query: &str) -> Result<Vec<TitleSummary>> {
        let origin = self.origin().await?;
        let url = url::Url::parse_with_params(&format!("{origin}/catalogue/"), &[("search", query)]).map_err(|e| Error::Config(e.to_string()))?;
        let html = self.ctx.http.get_text(url.as_str(), &[("Referer", &format!("{origin}/"))]).await?;
        Ok(parse_search(&html, &origin))
    }

    async fn detail(&self, id: &str) -> Result<TitleDetail> {
        let key = format!("animesama:detail:{id}");
        if let Some(d) = self.ctx.cache.get::<TitleDetail>(&key).await {
            return Ok(d);
        }
        let origin = self.origin().await?;
        let page = format!("{origin}{id}");
        let html = self.ctx.http.get_text(&page, &[("Referer", &format!("{origin}/"))]).await?;

        let (name, poster, overview) = {
            let doc = Html::parse_document(&html);
            let name = doc
                .select(&H1)
                .map(text_of)
                .find(|t| !t.is_empty())
                .or_else(|| doc.select(&OG_TITLE).next().and_then(|m| m.value().attr("content")).map(str::to_string))
                .unwrap_or_else(|| id.rsplit('/').next().unwrap_or(id).replace('-', " "));
            let poster = doc.select(&OG_IMAGE).next().and_then(|m| m.value().attr("content")).map(|p| absolutize(&page, p));
            let overview = doc.select(&OG_DESC).next().and_then(|m| m.value().attr("content")).map(str::to_string).filter(|d| !d.is_empty());
            (name, poster, overview)
        };
        let mut s = TitleSummary::new(ProviderId::AnimeSama, id, name, MediaKind::Anime);
        s.poster = poster;
        s.overview = overview;
        let mut d = TitleDetail::from_summary(s);

        // Numbered seasons only ("Saison 2"); films/OAV panels are skipped for now.
        let numbered: Vec<(u32, String)> = parse_seasons(&html)
            .into_iter()
            .filter_map(|(name, path)| Some((SAISON.captures(&name)?[1].parse().ok()?, path)))
            .collect();
        let counts = futures::future::join_all(numbered.iter().map(|(_, path)| {
            let base = format!("{page}/{}", LANG_SUFFIX.replace(path.trim_matches('/'), ""));
            async move {
                match self.episodes_js(&base, "vostfr").await {
                    Ok(p) if !p.is_empty() => p.iter().map(Vec::len).max().unwrap_or(0),
                    _ => self.episodes_js(&base, "vf").await.map(|p| p.iter().map(Vec::len).max().unwrap_or(0)).unwrap_or(0),
                }
            }
        }))
        .await;
        for ((number, path), count) in numbered.into_iter().zip(counts) {
            if count == 0 {
                continue;
            }
            d.seasons.push(Season {
                number,
                name: Some(path),
                episodes: (1..=count as u32)
                    .map(|n| Episode { season: number, number: n, title: None, overview: None, air_date: None, thumbnail: None, key: None })
                    .collect(),
            });
        }
        d.seasons.sort_by_key(|s| s.number);
        self.ctx.cache.put(&key, &d, ttl::DETAIL).await;
        Ok(d)
    }

    async fn sources(&self, detail: &TitleDetail, target: Target, mode: AudioMode) -> Result<Vec<Source>> {
        let (season, episode) = match target {
            Target::Episode { season, episode } => (season, episode),
            Target::Movie => (1, 1),
            Target::Season { .. } => return Ok(Vec::new()),
        };
        let path = detail
            .seasons
            .iter()
            .find(|s| s.number == season)
            .and_then(|s| s.name.clone())
            .ok_or_else(|| Error::NotFound(format!("anime-sama season {season}")))?;
        let origin = self.origin().await?;
        let base = format!("{origin}{}/{}", detail.summary.origin.id, LANG_SUFFIX.replace(path.trim_matches('/'), ""));
        let langs: [(&str, FrenchVariant); 2] = match mode {
            AudioMode::Sub => [("vostfr", FrenchVariant::Vostfr), ("vf", FrenchVariant::Vf)],
            AudioMode::Dub => [("vf", FrenchVariant::Vf), ("vostfr", FrenchVariant::Vostfr)],
        };

        let referer = format!("{origin}/");
        let mut jobs = Vec::new();
        for (lang, variant) in langs {
            let Ok(players) = self.episodes_js(&base, lang).await else { continue };
            for (i, urls) in players.iter().enumerate().take(3) {
                if let Some(u) = urls.get((episode as usize).saturating_sub(1)).cloned() {
                    let label = format!("Episode {episode} · {} · Lecteur {}", lang.to_uppercase(), i + 1);
                    jobs.push((u, label, variant));
                }
            }
        }
        if jobs.is_empty() {
            return Err(Error::NotFound(format!("anime-sama has no players for S{season}E{episode}")));
        }
        let results = futures::future::join_all(jobs.iter().map(|(u, label, _)| embeds::resolve(&self.ctx.http, ProviderId::AnimeSama, u, label, Some(&referer)))).await;

        let mut out = Vec::new();
        let mut last_err = None;
        for ((_, _, variant), r) in jobs.iter().zip(results) {
            match r {
                Ok(list) => {
                    for mut s in list {
                        s.tags.french = Some(*variant);
                        if *variant == FrenchVariant::Vf {
                            s.tags.languages = vec!["fr".into()];
                        } else {
                            s.tags.languages = vec!["ja".into()];
                            s.subtitles.push(Track { lang: Some("fr".into()), label: "Français (incrustés)".into(), url: None, default: true });
                        }
                        out.push(s);
                    }
                }
                Err(e) => last_err = Some(e),
            }
        }
        if out.is_empty() {
            return Err(last_err.unwrap_or_else(|| Error::NotFound("no anime-sama player resolved".into())));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn episodes_js_players() {
        let js = "var eps2 = ['https://b/1', 'https://b/2'];\nvar eps1 = [\n'https://a/1',\n'https://a/2',\n'https://a/3'];";
        let p = parse_episodes_js(js);
        assert_eq!(p.len(), 2);
        assert_eq!(p[0], vec!["https://a/1", "https://a/2", "https://a/3"], "sorted by player number");
        assert_eq!(p[1].len(), 2);
    }

    #[test]
    fn seasons_skip_templates_and_comments() {
        let html = r#"<script>panneauAnime("nom", "url");
            panneauAnime("Saison 1", "saison1/vostfr");
            /* panneauAnime("Saison 2", "saison2/vostfr"); */
            // panneauAnime("Saison 3", "saison3/vostfr");
            panneauAnime("Film", "film/vostfr");</script>"#;
        assert_eq!(parse_seasons(html), vec![("Saison 1".to_string(), "saison1/vostfr".to_string()), ("Film".to_string(), "film/vostfr".to_string())]);
        assert_eq!(LANG_SUFFIX.replace("saison1/vostfr", ""), "saison1");
    }

    #[test]
    fn search_cards() {
        let html = r#"<div id="list_catalog">
            <div data-nom="Frieren"><a href="https://anime-sama.to/catalogue/frieren"><img src="https://img/f.jpg"><h3>Frieren</h3></a></div>
            <div><span class="as-badge-scan"></span><a href="/catalogue/manga-only"><h3>Manga Only</h3></a></div></div>"#;
        let r = parse_search(html, "https://anime-sama.to");
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].origin.id, "/catalogue/frieren");
        assert_eq!(r[0].poster.as_deref(), Some("https://img/f.jpg"));
    }
}
