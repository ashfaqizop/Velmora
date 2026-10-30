//! French-Stream (French movies and series, VF / VFQ / VOSTFR). Ported from
//! freeflix-cli. Search is a POST to the DLE ajax endpoint; movies list their
//! players in `film_api.php`, series seasons in `ep-data.php`. Series cards
//! carry their season list inline, so the title id stores it directly.

use std::collections::BTreeMap;
use std::sync::{Arc, LazyLock};

use async_trait::async_trait;
use regex::Regex;
use scraper::{Html, Selector};
use serde_json::Value;

use super::{Ctx, Provider};
use crate::cache::ttl;
use crate::error::{Error, Result};
use crate::model::*;
use crate::release::FrenchVariant;
use crate::resolvers::embeds;
use crate::util::{absolutize, text_of, year_from};

pub struct FrenchStream {
    ctx: Arc<Ctx>,
}

macro_rules! sel {
    ($name:ident, $css:expr) => {
        static $name: LazyLock<Selector> = LazyLock::new(|| Selector::parse($css).expect("selector"));
    };
}

sel!(ITEM, "div.search-item");
sel!(ITEM_TITLE, ".search-title");
sel!(IMG, "img");
sel!(OG_TITLE, "meta[property=\"og:title\"]");
sel!(OG_DESC, "meta[property=\"og:description\"], meta[name=\"description\"]");
sel!(THUMB, "img.dvd-thumbnail");
sel!(GENRE_LI, "ul#s-list li");
sel!(A, "a");

static HREF: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"location\.href='([^']+)'").expect("regex"));
static YEAR_SUFFIX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s*\((19|20)\d{2}\)\s*$").expect("regex"));
static SERIE_SUFFIX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\s*-\s*s[ée]rie\s*$").expect("regex"));

/// Max player pages resolved per request (each is 1-3 HTTP calls).
const MAX_PLAYERS: usize = 6;

impl FrenchStream {
    pub fn new(ctx: Arc<Ctx>) -> Self {
        Self { ctx }
    }

    fn base(&self) -> Result<String> {
        self.ctx.base(ProviderId::FrenchStream)
    }

    async fn json(&self, path: &str) -> Result<Value> {
        let base = self.base()?;
        self.ctx.http.get_json(&format!("{base}{path}"), &[("Referer", &format!("{base}/"))]).await
    }
}

/// Numeric DLE id: `/15109316-dune.html` → `15109316`.
fn numeric_id(path: &str) -> String {
    path.trim_matches('/').rsplit('/').next().unwrap_or(path).split('-').next().unwrap_or_default().to_string()
}

fn clean_name(raw: &str) -> String {
    let t = SERIE_SUFFIX.replace(raw.trim(), "");
    YEAR_SUFFIX.replace(&t, "").trim().to_string()
}

fn variant(lang: &str) -> (Option<FrenchVariant>, Vec<String>) {
    match lang.to_ascii_lowercase().as_str() {
        "vf" | "vff" => (Some(FrenchVariant::Vf), vec!["fr".into()]),
        "vfq" => (Some(FrenchVariant::Vfq), vec!["fr".into()]),
        "vostfr" => (Some(FrenchVariant::Vostfr), Vec::new()),
        _ => (None, Vec::new()),
    }
}

fn usable(link: &str) -> bool {
    let l = link.to_ascii_lowercase();
    l.starts_with("http") && !["trakx", "netu", "chamber_go", "/moon2/"].iter().any(|d| l.contains(d))
}

pub fn parse_search(html: &str, base: &str) -> Vec<TitleSummary> {
    let doc = Html::parse_fragment(html);
    let mut out = Vec::new();
    for it in doc.select(&ITEM) {
        let Some(raw) = it.select(&ITEM_TITLE).next().map(text_of).filter(|t| !t.is_empty()) else { continue };
        let poster = it.select(&IMG).next().and_then(|i| i.value().attr("src")).map(|p| absolutize(&format!("{base}/"), p));
        let (id, kind) = if let Some(seasons) = it.value().attr("data-seasons") {
            let parsed: Vec<Value> = serde_json::from_str(seasons).unwrap_or_default();
            let compact: Vec<(u64, String)> = parsed
                .iter()
                .filter_map(|s| Some((s.get("n")?.as_u64()?, s.get("l")?.as_str()?.to_string())))
                .collect();
            if compact.is_empty() {
                continue;
            }
            (format!("series:{}", serde_json::to_string(&compact).unwrap_or_default()), MediaKind::Series)
        } else if let Some(c) = it.value().attr("onclick").and_then(|o| HREF.captures(o)) {
            (c[1].to_string(), MediaKind::Movie)
        } else {
            continue;
        };
        let mut s = TitleSummary::new(ProviderId::FrenchStream, id, clean_name(&raw), kind);
        s.year = year_from(&raw);
        s.poster = poster;
        s.badges = vec!["VF".into(), "VOSTFR".into()];
        out.push(s);
    }
    out
}

/// `{"vf": {"1": {"premium": url, ...}, ...}, "vostfr": {...}}` → episode → [(lang, player, url)].
fn episode_players(data: &Value) -> BTreeMap<u32, Vec<(String, String, String)>> {
    let mut out: BTreeMap<u32, Vec<(String, String, String)>> = BTreeMap::new();
    let Some(obj) = data.as_object() else { return out };
    for (lang, eps) in obj {
        if lang == "info" {
            continue;
        }
        for (num, players) in eps.as_object().into_iter().flatten() {
            let Ok(n) = num.parse::<u32>() else { continue };
            for (player, link) in players.as_object().into_iter().flatten() {
                if let Some(l) = link.as_str().filter(|l| usable(l)) {
                    out.entry(n).or_default().push((lang.clone(), player.clone(), l.to_string()));
                }
            }
        }
    }
    out
}

/// `{"players": {"premium": {"vff": url, ...}, ...}}` → [(lang, player, url)].
fn movie_players(data: &Value) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for (player, langs) in data.get("players").and_then(Value::as_object).into_iter().flatten() {
        for (lang, link) in langs.as_object().into_iter().flatten() {
            if let Some(l) = link.as_str().filter(|l| usable(l)) {
                if !out.iter().any(|(_, _, u): &(String, String, String)| u == l) {
                    out.push((lang.clone(), player.clone(), l.to_string()));
                }
            }
        }
    }
    out
}

/// Order players by the requested audio and cap the count.
fn choose(mut players: Vec<(String, String, String)>, mode: AudioMode) -> Vec<(String, String, String)> {
    let rank = |lang: &str| {
        let l = lang.to_ascii_lowercase();
        let dub = matches!(l.as_str(), "vf" | "vff" | "vfq");
        let sub = l == "vostfr";
        match mode {
            AudioMode::Dub if dub => 0,
            AudioMode::Sub if sub => 0,
            _ if dub || sub => 1,
            _ => 2,
        }
    };
    players.sort_by_key(|(lang, _, _)| rank(lang));
    players.truncate(MAX_PLAYERS);
    players
}

#[async_trait]
impl Provider for FrenchStream {
    fn id(&self) -> ProviderId {
        ProviderId::FrenchStream
    }

    fn kinds(&self) -> &'static [MediaKind] {
        &[MediaKind::Movie, MediaKind::Series]
    }

    async fn search(&self, query: &str) -> Result<Vec<TitleSummary>> {
        let base = self.base()?;
        let html = self
            .ctx
            .http
            .post_form_text(&format!("{base}/engine/ajax/search.php"), &[("query", query), ("page", "1")], &[("Referer", &format!("{base}/"))])
            .await?;
        Ok(parse_search(&html, &base))
    }

    async fn detail(&self, id: &str) -> Result<TitleDetail> {
        let key = format!("frenchstream:detail:{id}");
        if let Some(d) = self.ctx.cache.get::<TitleDetail>(&key).await {
            return Ok(d);
        }
        let d = if let Some(json) = id.strip_prefix("series:") {
            let seasons: Vec<(u64, String)> = serde_json::from_str(json).map_err(|e| Error::parse("french-stream id", e.to_string()))?;
            let datas = futures::future::join_all(seasons.iter().map(|(_, l)| self.json_owned(format!("/ep-data.php?id={}", numeric_id(l))))).await;
            let title = datas
                .iter()
                .filter_map(|r| r.as_ref().ok())
                .find_map(|v| v.pointer("/info/title").and_then(Value::as_str).map(str::to_string))
                .unwrap_or_else(|| "French-Stream".into());
            let mut d = TitleDetail::from_summary(TitleSummary::new(ProviderId::FrenchStream, id, clean_name(&title), MediaKind::Series));
            for ((n, l), data) in seasons.iter().zip(datas) {
                let Ok(v) = data else { continue };
                let eps: Vec<u32> = episode_players(&v).into_keys().collect();
                if eps.is_empty() {
                    continue;
                }
                let number = *n as u32;
                d.seasons.push(Season {
                    number,
                    name: Some(numeric_id(l)),
                    episodes: eps
                        .into_iter()
                        .map(|e| Episode { season: number, number: e, title: None, overview: None, air_date: None, thumbnail: None, key: None })
                        .collect(),
                });
            }
            d
        } else {
            let base = self.base()?;
            let html = self.ctx.http.get_text(&format!("{base}{id}"), &[("Referer", &format!("{base}/"))]).await?;
            let doc = Html::parse_document(&html);
            let name = doc
                .select(&OG_TITLE)
                .next()
                .and_then(|m| m.value().attr("content"))
                .map(str::to_string)
                .unwrap_or_else(|| id.trim_matches('/').replace(['-', '/'], " "));
            let mut s = TitleSummary::new(ProviderId::FrenchStream, id, clean_name(&name), MediaKind::Movie);
            s.year = year_from(&name);
            s.poster = doc.select(&THUMB).next().and_then(|i| i.value().attr("src")).map(|p| absolutize(&format!("{base}/"), p));
            s.overview = doc.select(&OG_DESC).next().and_then(|m| m.value().attr("content")).map(str::to_string).filter(|x| !x.is_empty());
            let mut d = TitleDetail::from_summary(s);
            d.genres = doc.select(&GENRE_LI).nth(1).map(|li| li.select(&A).map(text_of).filter(|g| !g.is_empty()).collect()).unwrap_or_default();
            d
        };
        self.ctx.cache.put(&key, &d, ttl::DETAIL).await;
        Ok(d)
    }

    async fn sources(&self, detail: &TitleDetail, target: Target, mode: AudioMode) -> Result<Vec<Source>> {
        let id = detail.summary.origin.id.as_str();
        let players = match target {
            Target::Movie if !id.starts_with("series:") => {
                movie_players(&self.json(&format!("/engine/ajax/film_api.php?id={}", numeric_id(id))).await?)
            }
            Target::Episode { season, episode } => {
                let sid = detail
                    .seasons
                    .iter()
                    .find(|s| s.number == season)
                    .and_then(|s| s.name.clone())
                    .ok_or_else(|| Error::NotFound(format!("french-stream season {season}")))?;
                let v = self.json(&format!("/ep-data.php?id={sid}")).await?;
                episode_players(&v).remove(&episode).unwrap_or_default()
            }
            _ => return Ok(Vec::new()),
        };
        let chosen = choose(players, mode);
        if chosen.is_empty() {
            return Err(Error::NotFound("french-stream lists no usable player".into()));
        }
        let base = self.base()?;
        let referer = format!("{base}/");
        let results = futures::future::join_all(chosen.iter().map(|(lang, player, url)| {
            let label = format!("{} · {}", lang.to_uppercase(), player);
            let referer = referer.clone();
            async move { embeds::resolve(&self.ctx.http, ProviderId::FrenchStream, url, &label, Some(&referer)).await }
        }))
        .await;

        let mut out = Vec::new();
        let mut last_err = None;
        for ((lang, _, _), r) in chosen.iter().zip(results) {
            match r {
                Ok(list) => {
                    let (fv, langs) = variant(lang);
                    for mut s in list {
                        s.tags.french = fv;
                        if !langs.is_empty() {
                            s.tags.languages.clone_from(&langs);
                        }
                        out.push(s);
                    }
                }
                Err(e) => last_err = Some(e),
            }
        }
        if out.is_empty() {
            return Err(last_err.unwrap_or_else(|| Error::NotFound("no french-stream player resolved".into())));
        }
        Ok(out)
    }
}

impl FrenchStream {
    async fn json_owned(&self, path: String) -> Result<Value> {
        self.json(&path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_cards() {
        let html = r#"<div class="search-item search-series" data-seasons='[{"n":1,"l":"\/15109823-sophie-cross-saison-1.html","p":"x"},{"n":2,"l":"\/15114353-sophie-cross-saison-2.html","p":"y"}]' onclick="showSeasons(this)"><div class="search-poster"><img src="https://image.tmdb.org/t/p/w400/a.jpg"></div><div class="search-title">Sophie Cross - série</div></div>
            <div class="search-item" onclick="location.href='/15109316-dune.html'"><img src="/img/d.jpg"><div class="search-title">Dune (2021)</div></div>"#;
        let r = parse_search(html, "https://french-stream.net");
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].name, "Sophie Cross");
        assert_eq!(r[0].kind, MediaKind::Series);
        assert!(r[0].origin.id.starts_with("series:[[1,"));
        assert_eq!(r[1].name, "Dune");
        assert_eq!(r[1].year, Some(2021));
        assert_eq!(r[1].origin.id, "/15109316-dune.html");
        assert_eq!(r[1].poster.as_deref(), Some("https://french-stream.net/img/d.jpg"));
        assert_eq!(numeric_id("/15109316-dune.html"), "15109316");
    }

    #[test]
    fn players() {
        let movie: Value = serde_json::from_str(r#"{"players":{"premium":{"default":"https://fsvid.lol/embed-a.html","vff":"https://fsvid.lol/embed-b.html"},"netu":{"vff":"https://trakx.lol/moon2/x"}}}"#).unwrap();
        let m = movie_players(&movie);
        assert_eq!(m.len(), 2, "dead netu/trakx host dropped");
        let chosen = choose(m, AudioMode::Dub);
        assert_eq!(chosen[0].0, "vff");

        let series: Value = serde_json::from_str(r#"{"vf":{"1":{"premium":"https://fsvid.lol/e1.html","netu":"https://trakx.lol/x"},"2":{"uqload":"https://uqload.vc/e2.html"}},"vostfr":{"1":{"vidzy":"https://vidzy.cc/e1.html"}},"vo":[],"info":{"title":"X"}}"#).unwrap();
        let eps = episode_players(&series);
        assert_eq!(eps.keys().copied().collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(eps[&1].len(), 2);
        assert_eq!(variant("vostfr").0, Some(FrenchVariant::Vostfr));
    }
}
