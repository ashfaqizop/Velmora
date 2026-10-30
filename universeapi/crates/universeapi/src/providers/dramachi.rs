//! Dramachi (nodeobjects API): Asian dramas and films. Ported from
//! MovieBox-TUI. Each season has "versions" that are really audio tracks
//! (Original, Dub, Hindi Dub); the engine fetches the original plus the
//! versions matching the user's audio preferences.

use std::collections::BTreeMap;
use std::sync::{Arc, LazyLock};

use async_trait::async_trait;
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::{Ctx, Provider};
use crate::cache::ttl;
use crate::error::{Error, Result};
use crate::model::*;
use crate::release::parse_size;
use crate::util::year_from;

const THUMB_CDN: &str = "https://static.nodeobjects.com/thumbnail/";

pub struct Dramachi {
    ctx: Arc<Ctx>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct SearchResp {
    #[serde(deserialize_with = "crate::util::null_default")]
    data: Vec<Item>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct Item {
    id: String,
    title: String,
    thumb: Option<String>,
    year: Option<String>,
    category: Option<String>,
    content: Option<String>,
    storyline: Option<String>,
    director: Option<String>,
    genres: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct TitleResp {
    #[serde(deserialize_with = "crate::util::null_default")]
    album: Vec<Item>,
    #[serde(deserialize_with = "crate::util::null_default")]
    cast: Vec<Cast>,
    #[serde(deserialize_with = "crate::util::null_default")]
    seasons: BTreeMap<String, SeasonGroup>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct Cast {
    name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct SeasonGroup {
    #[serde(deserialize_with = "crate::util::null_default")]
    versions: Vec<Version>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct Version {
    version_name: String,
    rip: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct EpList {
    #[serde(deserialize_with = "crate::util::null_default")]
    episode_list: Vec<EpItem>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct EpItem {
    f_title: String,
    fid: String,
    disk: String,
    quality: Option<String>,
    size: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct FileResp {
    #[serde(rename = "fileInfo", deserialize_with = "crate::util::null_default")]
    file_info: Vec<FileInfo>,
    #[serde(rename = "hostInfo")]
    host_info: Option<HostInfo>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct FileInfo {
    url: Option<String>,
    filename: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct HostInfo {
    host: Option<String>,
}

fn kind_of(it: &Item) -> MediaKind {
    let content = it.content.as_deref().unwrap_or("").to_ascii_lowercase();
    let category = it.category.as_deref().unwrap_or("").to_ascii_lowercase();
    if content.starts_with("movie") {
        MediaKind::Movie
    } else if category.contains("anime") {
        MediaKind::Anime
    } else {
        MediaKind::AsianDrama
    }
}

fn summary(it: &Item) -> TitleSummary {
    let mut s = TitleSummary::new(ProviderId::Dramachi, it.id.clone(), it.title.clone(), kind_of(it));
    s.year = it.year.as_deref().and_then(year_from);
    s.poster = it.thumb.as_deref().map(str::trim).filter(|t| !t.is_empty()).map(|t| format!("{THUMB_CDN}{t}"));
    s.overview = it.storyline.clone().filter(|x| !x.trim().is_empty());
    if let Some(c) = it.category.as_deref().filter(|c| !c.is_empty()) {
        // "k drama" -> "K-Drama"
        let mut words = c.split_whitespace().map(|w| {
            let mut ch = w.chars();
            ch.next().map(|f| f.to_uppercase().collect::<String>() + ch.as_str()).unwrap_or_default()
        });
        let label = match (words.next(), words.next()) {
            (Some(a), Some(b)) if a.len() == 1 => format!("{a}-{b}"),
            (Some(a), Some(b)) => format!("{a} {b}"),
            (Some(a), None) => a,
            _ => String::new(),
        };
        if !label.is_empty() {
            s.badges.push(label);
        }
    }
    s
}

/// "Season 01" / "Season 2 HI DUB" → 1 / 2.
fn season_number(key: &str) -> Option<u32> {
    static N: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)season\s*0*(\d{1,3})").expect("regex"));
    N.captures(key).and_then(|c| c[1].parse().ok())
}

/// Episode number from `"Queen of Tears S01E16"`, `"Ep 7"` or a trailing `"002"`.
pub fn episode_number(f_title: &str) -> Option<u32> {
    static EP: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)(?:\bS\d{1,3}E|\bEp(?:isode)?[ .\-_]*)0*(\d{1,4})").expect("regex"));
    if let Some(c) = EP.captures(f_title) {
        return c[1].parse().ok();
    }
    f_title.split_whitespace().rev().find_map(|t| t.parse::<u32>().ok().filter(|n| *n < 5000))
}

/// Audio language a Dramachi version represents (`None` = original audio).
fn version_lang(name: &str) -> Option<String> {
    let l = name.to_ascii_lowercase();
    if l.contains("original") || l.is_empty() {
        None
    } else if let Some(code) = crate::release::ReleaseTags::parse(name).languages.into_iter().next() {
        Some(code)
    } else if l.contains("hi ") || l.ends_with(" hi") {
        Some("hi".into())
    } else {
        // plain "Dub" is the English dub
        Some("en".into())
    }
}

impl Dramachi {
    pub fn new(ctx: Arc<Ctx>) -> Self {
        Self { ctx }
    }

    fn api(&self, query: &str) -> Result<String> {
        Ok(format!("{}/?{query}", self.ctx.base(ProviderId::Dramachi)?))
    }

    async fn title(&self, id: &str) -> Result<TitleResp> {
        let key = format!("dramachi:title:{id}");
        if let Some(t) = self.ctx.cache.get::<TitleResp>(&key).await {
            return Ok(t);
        }
        let url = self.api(&format!("interface=title_v2&id={}", enc(id)))?;
        let t: TitleResp = self.ctx.http.get_json(&url, &[]).await?;
        self.ctx.cache.put(&key, &t, ttl::DETAIL).await;
        Ok(t)
    }

    async fn episodes(&self, id: &str, rip: &str) -> Result<Vec<EpItem>> {
        let url = self.api(&format!("interface=eplist&season={}&id={}", enc(rip), enc(id)))?;
        let list: EpList = self.ctx.http.get_json(&url, &[]).await?;
        Ok(list.episode_list)
    }

    async fn file(&self, ep: &EpItem) -> Result<Source> {
        let url = self.api(&format!("interface=getFile&fid={}&findex={}", enc(&ep.fid), enc(&ep.disk)))?;
        let f: FileResp = self.ctx.http.get_json(&url, &[]).await?;
        let host = f.host_info.and_then(|h| h.host).filter(|h| !h.is_empty()).ok_or_else(|| Error::NotFound("dramachi file host".into()))?;
        let info = f.file_info.into_iter().next().ok_or_else(|| Error::NotFound("dramachi file".into()))?;
        let path = info.url.ok_or_else(|| Error::parse("dramachi getFile", "no url"))?;
        let stream = format!("https://{}/cdn/{}", host.trim_end_matches('/'), path.trim_start_matches('/'));
        let label = format!("{} {}", ep.quality.as_deref().unwrap_or(""), ep.f_title).trim().to_string();
        let mut s = Source::new(ProviderId::Dramachi, "Dramachi CDN", label, stream);
        s.filename = info.filename;
        s.size_bytes = ep.size.as_deref().and_then(parse_size);
        s.retag();
        Ok(s)
    }
}

fn enc(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

#[async_trait]
impl Provider for Dramachi {
    fn id(&self) -> ProviderId {
        ProviderId::Dramachi
    }

    fn kinds(&self) -> &'static [MediaKind] {
        &[MediaKind::AsianDrama, MediaKind::Movie, MediaKind::Anime]
    }

    async fn search(&self, query: &str) -> Result<Vec<TitleSummary>> {
        let q = query.trim();
        if q.is_empty() {
            return Ok(Vec::new());
        }
        let url = self.api(&format!("interface=search&q={}&filter=all&page=1", enc(q)))?;
        let r: SearchResp = self.ctx.http.get_json(&url, &[]).await?;
        Ok(r.data.iter().filter(|i| !i.id.is_empty()).map(summary).collect())
    }

    async fn detail(&self, id: &str) -> Result<TitleDetail> {
        let key = format!("dramachi:detail:{id}");
        if let Some(d) = self.ctx.cache.get::<TitleDetail>(&key).await {
            return Ok(d);
        }
        let t = self.title(id).await?;
        let album = t.album.first().ok_or_else(|| Error::NotFound(format!("dramachi {id}")))?;
        let mut d = TitleDetail::from_summary(summary(album));
        d.genres = album.genres.as_deref().unwrap_or("").split(',').map(|g| g.trim().to_string()).filter(|g| !g.is_empty()).collect();
        d.directors = album.director.clone().filter(|x| !x.trim().is_empty()).into_iter().collect();
        d.cast = t.cast.iter().map(|c| c.name.trim().to_string()).filter(|n| !n.is_empty()).collect();

        if d.summary.kind != MediaKind::Movie {
            // Episode lists come from the first (original) version of each season.
            let seasons: Vec<(u32, String)> = t
                .seasons
                .iter()
                .filter_map(|(k, g)| Some((season_number(k).unwrap_or(1), g.versions.first()?.rip.clone())))
                .collect();
            let lists = futures::future::join_all(seasons.iter().map(|(_, rip)| self.episodes(id, rip))).await;
            for ((number, _), list) in seasons.iter().zip(lists) {
                let mut eps: Vec<Episode> = Vec::new();
                for (i, e) in list.unwrap_or_default().iter().enumerate() {
                    let n = episode_number(&e.f_title).unwrap_or(i as u32 + 1);
                    if !eps.iter().any(|x| x.number == n) {
                        eps.push(Episode { season: *number, number: n, title: None, overview: None, air_date: None, thumbnail: None, key: None });
                    }
                }
                eps.sort_by_key(|e| e.number);
                if !eps.is_empty() {
                    d.seasons.push(Season { number: *number, name: None, episodes: eps });
                }
            }
            d.seasons.sort_by_key(|s| s.number);
        }
        self.ctx.cache.put(&key, &d, ttl::DETAIL).await;
        Ok(d)
    }

    async fn sources(&self, detail: &TitleDetail, target: Target, _mode: AudioMode) -> Result<Vec<Source>> {
        let id = detail.summary.origin.id.as_str();
        let (season, episode) = match target {
            Target::Movie => (0, 1),
            Target::Episode { season, episode } => (season, episode),
            Target::Season { .. } => return Ok(Vec::new()),
        };
        let t = self.title(id).await?;
        let group = t
            .seasons
            .iter()
            .find(|(k, _)| season == 0 || season_number(k) == Some(season))
            .map(|(_, g)| g)
            .ok_or_else(|| Error::NotFound(format!("dramachi season {season}")))?;

        // Original audio + versions in the user's preferred languages (max 3).
        let prefs = self.ctx.prefs();
        let versions: Vec<(&Version, Option<String>)> = group
            .versions
            .iter()
            .map(|v| (v, version_lang(&v.version_name)))
            .filter(|(_, lang)| lang.as_ref().is_none_or(|l| prefs.audio_languages.contains(l)))
            .take(3)
            .collect();

        let mut out = Vec::new();
        let mut last_err = None;
        for (v, lang) in versions {
            let list = match self.episodes(id, &v.rip).await {
                Ok(l) => l,
                Err(e) => {
                    last_err = Some(e);
                    continue;
                }
            };
            let pick = list
                .iter()
                .find(|e| episode_number(&e.f_title) == Some(episode))
                .or_else(|| list.get(episode.saturating_sub(1) as usize).filter(|_| episode_number(&list[0].f_title).is_none()));
            let Some(ep) = pick.filter(|e| !e.fid.is_empty() && !e.disk.is_empty()) else { continue };
            match self.file(ep).await {
                Ok(mut s) => {
                    s.label = format!("{} · {}", s.label, v.version_name);
                    if let Some(l) = lang {
                        if !s.tags.languages.contains(&l) {
                            s.tags.languages.insert(0, l);
                        }
                    }
                    out.push(s);
                }
                Err(e) => last_err = Some(e),
            }
        }
        if out.is_empty() {
            return Err(last_err.unwrap_or_else(|| Error::NotFound(format!("dramachi has no file for {target:?}"))));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn episode_numbers() {
        assert_eq!(episode_number("Queen of Tears S01E16"), Some(16));
        assert_eq!(episode_number("Squid Game S02E07 DUB"), Some(7));
        assert_eq!(episode_number("Parasite 2019 002"), Some(2));
        assert_eq!(episode_number("Some Show Ep 4"), Some(4));
        assert_eq!(season_number("Season 01 HI DUB"), Some(1));
    }

    #[test]
    fn versions_map_to_languages() {
        assert_eq!(version_lang("Original"), None);
        assert_eq!(version_lang("Hindi Dub"), Some("hi".into()));
        assert_eq!(version_lang("Dub"), Some("en".into()));
    }

    #[test]
    fn search_mapping() {
        let r: SearchResp = serde_json::from_str(
            r#"{"data":[{"id":"61779","title":"Queen of Tears","thumb":"q.jpg","year":"2024","category":"k drama","content":"drama"},
                       {"id":"9","title":"Parasite","year":"2019","content":"movies","category":null}]}"#,
        )
        .unwrap();
        let s: Vec<_> = r.data.iter().map(summary).collect();
        assert_eq!(s[0].kind, MediaKind::AsianDrama);
        assert_eq!(s[0].badges, vec!["K-Drama"]);
        assert_eq!(s[0].poster.as_deref(), Some("https://static.nodeobjects.com/thumbnail/q.jpg"));
        assert_eq!(s[1].kind, MediaKind::Movie);
        let empty: SearchResp = serde_json::from_str(r#"{"data":null}"#).unwrap();
        assert!(empty.data.is_empty());
    }
}
