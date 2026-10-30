//! Torrent sources: YTS (movies), EZTV (TV episodes) and Nyaa (anime).
//!
//! They only add sources to titles found elsewhere, so they never show up
//! in search results. YTS and EZTV look up by IMDb id (exact, no fuzzy
//! matching); Nyaa searches its RSS feed for "<series> <episode>". Every
//! result is a magnet [`Source`] for the torrent engine, ranked below
//! direct links.

use std::sync::{Arc, LazyLock};

use async_trait::async_trait;
use regex::Regex;
use serde::Deserialize;

use super::{Ctx, Provider};
use crate::error::{Error, Result};
use crate::model::*;
use crate::release::parse_size;

/// Public trackers appended to every magnet so peers are found without DHT warm-up.
const TRACKERS: &[&str] = &[
    "udp://tracker.opentrackr.org:1337/announce",
    "udp://open.stealth.si:80/announce",
    "udp://tracker.torrent.eu.org:451/announce",
    "udp://exodus.desync.com:6969/announce",
    "udp://tracker.openbittorrent.com:6969/announce",
    "udp://open.demonii.com:1337/announce",
];

pub fn magnet(hash: &str, name: &str) -> String {
    let mut m = format!("magnet:?xt=urn:btih:{}&dn={}", hash.to_ascii_lowercase(), enc(name));
    for t in TRACKERS {
        m.push_str("&tr=");
        m.push_str(&enc(t));
    }
    m
}

fn enc(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

fn torrent_source(provider: ProviderId, label: &str, hash: &str, seeds: Option<u32>, peers: Option<u32>, size: Option<u64>) -> Source {
    let link = magnet(hash, label);
    let mut s = Source::new(provider, provider.display_name(), label, link.clone());
    s.format = StreamFormat::Torrent;
    s.size_bytes = size;
    s.torrent = Some(TorrentInfo { magnet: link, info_hash: Some(hash.to_ascii_lowercase()), seeders: seeds, leechers: peers, file_index: None });
    s.retag();
    s
}

/// Placeholder title for providers that are addressed by name, not catalogue id.
fn pseudo(provider: ProviderId, query: &str, kind: MediaKind) -> TitleSummary {
    TitleSummary::new(provider, query, query, kind)
}

fn digits(imdb: &str) -> String {
    imdb.trim_start_matches("tt").to_string()
}

fn num<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Option<u64>, D::Error> {
    let v = Option::<serde_json::Value>::deserialize(d)?;
    Ok(v.and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.trim().parse().ok()))))
}

// ── YTS ──────────────────────────────────────────────────────────────────

pub struct Yts {
    ctx: Arc<Ctx>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct YtsResp {
    data: YtsData,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct YtsData {
    #[serde(deserialize_with = "crate::util::null_default")]
    movies: Vec<YtsMovie>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct YtsMovie {
    title: String,
    year: Option<u16>,
    imdb_code: Option<String>,
    #[serde(deserialize_with = "crate::util::null_default")]
    torrents: Vec<YtsTorrent>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct YtsTorrent {
    hash: String,
    quality: String,
    #[serde(rename = "type")]
    kind: String,
    video_codec: Option<String>,
    bit_depth: Option<String>,
    audio_channels: Option<String>,
    #[serde(deserialize_with = "num")]
    seeds: Option<u64>,
    #[serde(deserialize_with = "num")]
    peers: Option<u64>,
    #[serde(deserialize_with = "num")]
    size_bytes: Option<u64>,
}

impl Yts {
    pub fn new(ctx: Arc<Ctx>) -> Self {
        Self { ctx }
    }

    async fn query(&self, term: &str) -> Result<Vec<YtsMovie>> {
        let reg = self.ctx.registry();
        let mut last = None;
        for base in reg.mirrors(ProviderId::Yts) {
            let url = format!("{}/api/v2/list_movies.json?query_term={}&limit=20", base.trim_end_matches('/'), enc(term));
            match self.ctx.http.get_json::<YtsResp>(&url, &[]).await {
                Ok(r) => return Ok(r.data.movies),
                Err(e) => last = Some(e),
            }
        }
        Err(last.unwrap_or_else(|| Error::Config("no YTS mirror configured".into())))
    }
}

fn yts_sources(m: &YtsMovie) -> Vec<Source> {
    m.torrents
        .iter()
        .filter(|t| !t.hash.is_empty())
        .map(|t| {
            let label = format!(
                "{} ({}) {} {} {} {} {}",
                m.title,
                m.year.map(|y| y.to_string()).unwrap_or_default(),
                t.quality,
                t.kind.to_uppercase(),
                t.video_codec.as_deref().unwrap_or(""),
                t.bit_depth.as_deref().map(|b| format!("{b}bit")).unwrap_or_default(),
                t.audio_channels.as_deref().map(|a| format!("AAC {a}")).unwrap_or_default()
            );
            let label = label.split_whitespace().collect::<Vec<_>>().join(" ");
            torrent_source(ProviderId::Yts, &label, &t.hash, t.seeds.map(|x| x as u32), t.peers.map(|x| x as u32), t.size_bytes)
        })
        .collect()
}

#[async_trait]
impl Provider for Yts {
    fn id(&self) -> ProviderId {
        ProviderId::Yts
    }

    fn kinds(&self) -> &'static [MediaKind] {
        &[MediaKind::Movie]
    }

    async fn search(&self, query: &str) -> Result<Vec<TitleSummary>> {
        Ok(self
            .query(query)
            .await?
            .into_iter()
            .filter_map(|m| {
                let imdb = m.imdb_code.clone()?;
                let mut s = TitleSummary::new(ProviderId::Yts, imdb.clone(), m.title, MediaKind::Movie);
                s.year = m.year;
                s.ids.imdb = Some(imdb);
                Some(s)
            })
            .collect())
    }

    async fn detail(&self, id: &str) -> Result<TitleDetail> {
        let mut s = pseudo(ProviderId::Yts, id, MediaKind::Movie);
        s.ids.imdb = ExternalIds::normalize_imdb(id);
        Ok(TitleDetail::from_summary(s))
    }

    async fn sources(&self, detail: &TitleDetail, target: Target, mode: AudioMode) -> Result<Vec<Source>> {
        self.sources_by_ids(&detail.summary.ids, MediaKind::Movie, target, mode)
            .await?
            .ok_or_else(|| Error::NotFound("YTS needs an IMDb id".into()))
    }

    async fn sources_by_ids(&self, ids: &ExternalIds, kind: MediaKind, target: Target, _: AudioMode) -> Result<Option<Vec<Source>>> {
        let Some(imdb) = ids.imdb.as_deref() else { return Ok(None) };
        if kind != MediaKind::Movie || target != Target::Movie {
            return Ok(None);
        }
        let movies = self.query(imdb).await?;
        let out: Vec<Source> = movies.iter().filter(|m| m.imdb_code.as_deref() == Some(imdb)).flat_map(yts_sources).collect();
        if out.is_empty() {
            return Err(Error::NotFound(format!("YTS has no torrents for {imdb}")));
        }
        Ok(Some(out))
    }
}

// ── EZTV ─────────────────────────────────────────────────────────────────

pub struct Eztv {
    ctx: Arc<Ctx>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct EztvResp {
    #[serde(deserialize_with = "num")]
    torrents_count: Option<u64>,
    #[serde(deserialize_with = "crate::util::null_default")]
    torrents: Vec<EztvTorrent>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct EztvTorrent {
    hash: String,
    title: String,
    #[serde(deserialize_with = "num")]
    season: Option<u64>,
    #[serde(deserialize_with = "num")]
    episode: Option<u64>,
    #[serde(deserialize_with = "num")]
    seeds: Option<u64>,
    #[serde(deserialize_with = "num")]
    peers: Option<u64>,
    #[serde(deserialize_with = "num")]
    size_bytes: Option<u64>,
}

impl Eztv {
    pub fn new(ctx: Arc<Ctx>) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl Provider for Eztv {
    fn id(&self) -> ProviderId {
        ProviderId::Eztv
    }

    fn kinds(&self) -> &'static [MediaKind] {
        &[MediaKind::Series, MediaKind::AsianDrama]
    }

    async fn search(&self, _query: &str) -> Result<Vec<TitleSummary>> {
        Ok(Vec::new())
    }

    async fn detail(&self, id: &str) -> Result<TitleDetail> {
        Err(Error::Unsupported(format!("EZTV is IMDb-addressed ({id})")))
    }

    async fn sources(&self, detail: &TitleDetail, target: Target, mode: AudioMode) -> Result<Vec<Source>> {
        self.sources_by_ids(&detail.summary.ids, detail.summary.kind, target, mode)
            .await?
            .ok_or_else(|| Error::NotFound("EZTV needs an IMDb id".into()))
    }

    async fn sources_by_ids(&self, ids: &ExternalIds, kind: MediaKind, target: Target, _: AudioMode) -> Result<Option<Vec<Source>>> {
        let Some(imdb) = ids.imdb.as_deref() else { return Ok(None) };
        if !kind.is_episodic() {
            return Ok(None);
        }
        let (season, episode) = match target {
            Target::Episode { season, episode } => (season as u64, Some(episode as u64)),
            Target::Season { season } => (season as u64, None),
            Target::Movie => return Ok(None),
        };
        let reg = self.ctx.registry();
        let mirrors = reg.mirrors(ProviderId::Eztv);
        let mut out = Vec::new();
        // Results are newest first, 100 per page; long-running shows need a few pages.
        for page in 1..=5u32 {
            let mut resp = None;
            let mut last = None;
            for base in mirrors {
                let url = format!("{}/api/get-torrents?imdb_id={}&limit=100&page={page}", base.trim_end_matches('/'), digits(imdb));
                match self.ctx.http.get_json::<EztvResp>(&url, &[]).await {
                    Ok(r) => {
                        resp = Some(r);
                        break;
                    }
                    Err(e) => last = Some(e),
                }
            }
            let r = match (resp, last) {
                (Some(r), _) => r,
                (None, Some(e)) => return Err(e),
                (None, None) => return Err(Error::Config("no EZTV mirror configured".into())),
            };
            for t in &r.torrents {
                let wanted = t.season == Some(season)
                    && match episode {
                        Some(e) => t.episode == Some(e),
                        None => t.episode == Some(0) || t.episode.is_none(),
                    };
                if wanted && !t.hash.is_empty() {
                    let mut s = torrent_source(ProviderId::Eztv, &t.title, &t.hash, t.seeds.map(|x| x as u32), t.peers.map(|x| x as u32), t.size_bytes);
                    s.is_pack = episode.is_none();
                    out.push(s);
                }
            }
            let total = r.torrents_count.unwrap_or(0);
            if r.torrents.is_empty() || u64::from(page) * 100 >= total {
                break;
            }
        }
        if out.is_empty() {
            return Err(Error::NotFound(format!("EZTV has no torrents for {imdb} {target:?}")));
        }
        Ok(Some(out))
    }
}

// ── Nyaa ─────────────────────────────────────────────────────────────────

pub struct Nyaa {
    ctx: Arc<Ctx>,
}

static ITEM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)<item>(.*?)</item>").expect("regex"));

fn tag<'a>(item: &'a str, name: &str) -> Option<&'a str> {
    let open = format!("<{name}>");
    let start = item.find(&open)? + open.len();
    let end = item[start..].find(&format!("</{name}>"))? + start;
    Some(item[start..end].trim())
}

#[derive(Debug, Clone, PartialEq)]
pub struct NyaaItem {
    pub title: String,
    pub hash: String,
    pub seeders: u32,
    pub leechers: u32,
    pub size: Option<u64>,
}

pub fn parse_nyaa_rss(xml: &str) -> Vec<NyaaItem> {
    ITEM.captures_iter(xml)
        .filter_map(|c| {
            let it = c.get(1)?.as_str();
            Some(NyaaItem {
                title: crate::util::unescape_html(tag(it, "title")?),
                hash: tag(it, "nyaa:infoHash")?.to_ascii_lowercase(),
                seeders: tag(it, "nyaa:seeders").and_then(|s| s.parse().ok()).unwrap_or(0),
                leechers: tag(it, "nyaa:leechers").and_then(|s| s.parse().ok()).unwrap_or(0),
                size: tag(it, "nyaa:size").and_then(parse_size),
            })
        })
        .collect()
}

/// Does a fansub release title name this episode (and not a batch range)?
pub fn names_episode(title: &str, episode: u32) -> bool {
    let re = Regex::new(&format!(r"(?i)(?:\s-\s|\dE|\bE|\bEP\.?\s?|\s#|\[)0*{episode}(?:v\d)?(?:\s|\]|\)|\.|$|\[)")).expect("regex");
    let batch = Regex::new(r"(?i)\b\d{1,4}\s*[-~]\s*\d{1,4}\b|\bbatch\b|\bcomplete\b").expect("regex");
    re.is_match(title) && !batch.is_match(title)
}

/// Season a fansub title names ("S2", "Season 2", "2nd Season"); `None` = first/unspecified.
pub fn season_of(title: &str) -> Option<u32> {
    static SEASON: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)\bS(\d{1,2})(?:E\d+|\b)|\bseason\s*(\d{1,2})\b|\b(\d{1,2})(?:st|nd|rd|th)\s+season\b").expect("regex")
    });
    let c = SEASON.captures(title)?;
    c.get(1).or_else(|| c.get(2)).or_else(|| c.get(3)).and_then(|m| m.as_str().parse().ok())
}

impl Nyaa {
    pub fn new(ctx: Arc<Ctx>) -> Self {
        Self { ctx }
    }

    async fn rss(&self, query: &str) -> Result<Vec<NyaaItem>> {
        let base = self.ctx.base(ProviderId::Nyaa)?;
        // c=1_2: anime, English-translated; s=seeders: best-seeded first.
        let url = format!("{base}/?page=rss&q={}&c=1_2&f=0&s=seeders&o=desc", enc(query));
        Ok(parse_nyaa_rss(&self.ctx.http.get_text(&url, &[]).await?))
    }
}

#[async_trait]
impl Provider for Nyaa {
    fn id(&self) -> ProviderId {
        ProviderId::Nyaa
    }

    fn kinds(&self) -> &'static [MediaKind] {
        &[MediaKind::Anime]
    }

    /// Nyaa has no title catalogue: echo the query so the engine can address it by name.
    async fn search(&self, query: &str) -> Result<Vec<TitleSummary>> {
        Ok(vec![pseudo(ProviderId::Nyaa, query, MediaKind::Anime)])
    }

    async fn detail(&self, id: &str) -> Result<TitleDetail> {
        Ok(TitleDetail::from_summary(pseudo(ProviderId::Nyaa, id, MediaKind::Anime)))
    }

    async fn sources(&self, detail: &TitleDetail, target: Target, mode: AudioMode) -> Result<Vec<Source>> {
        let name = detail.summary.origin.id.as_str();
        let (query, filter, season): (String, Option<u32>, u32) = match target {
            Target::Episode { season, episode } => (format!("{name} {episode:02}"), Some(episode), season.max(1)),
            Target::Movie => (name.to_string(), None, 1),
            Target::Season { season } => (format!("{name} batch"), None, season.max(1)),
        };
        let query = if mode == AudioMode::Dub { format!("{query} dual audio") } else { query };
        let items = self.rss(&query).await?;
        let out: Vec<Source> = items
            .iter()
            .filter(|i| i.seeders > 0)
            .filter(|i| filter.is_none_or(|e| names_episode(&i.title, e)))
            .filter(|i| season_of(&i.title).unwrap_or(1) == season)
            .take(8)
            .map(|i| {
                let mut s = torrent_source(ProviderId::Nyaa, &i.title, &i.hash, Some(i.seeders), Some(i.leechers), i.size);
                s.is_pack = matches!(target, Target::Season { .. });
                s
            })
            .collect();
        if out.is_empty() {
            return Err(Error::NotFound(format!("Nyaa has nothing seeded for {query}")));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magnets() {
        let m = magnet("ABCDEF0123", "Dune (2021) 1080p");
        assert!(m.starts_with("magnet:?xt=urn:btih:abcdef0123&dn=Dune+%282021%29+1080p&tr=udp%3A%2F%2Ftracker.opentrackr.org"));
    }

    #[test]
    fn yts_mapping() {
        let r: YtsResp = serde_json::from_str(
            r#"{"data":{"movies":[{"title":"Dune","year":2021,"imdb_code":"tt1160419","torrents":[
                {"hash":"AB12","quality":"1080p","type":"bluray","video_codec":"x264","bit_depth":"8","audio_channels":"5.1","seeds":120,"peers":30,"size_bytes":2400000000},
                {"hash":"CD34","quality":"2160p","type":"web","video_codec":"x265","seeds":"15","peers":"2","size_bytes":5000000000}]}]}}"#,
        )
        .unwrap();
        let s = yts_sources(&r.data.movies[0]);
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].format, StreamFormat::Torrent);
        assert_eq!(s[0].tags.resolution, Some(1080));
        assert_eq!(s[0].torrent.as_ref().unwrap().seeders, Some(120));
        assert_eq!(s[1].torrent.as_ref().unwrap().seeders, Some(15), "string counts parse");
        assert_eq!(s[1].tags.resolution, Some(2160));
        let empty: YtsResp = serde_json::from_str(r#"{"data":{"movie_count":0}}"#).unwrap();
        assert!(empty.data.movies.is_empty());
    }

    #[test]
    fn nyaa_rss() {
        let xml = r#"<rss><channel><item><title>[SubsPlease] Sousou no Frieren - 03 (1080p) [ABCD1234].mkv</title>
            <nyaa:seeders>812</nyaa:seeders><nyaa:leechers>12</nyaa:leechers><nyaa:infoHash>FFEE00</nyaa:infoHash><nyaa:size>1.4 GiB</nyaa:size></item>
            <item><title>[Group] Frieren 01-28 [Batch]</title><nyaa:seeders>90</nyaa:seeders><nyaa:infoHash>aa</nyaa:infoHash></item></channel></rss>"#;
        let items = parse_nyaa_rss(xml);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].hash, "ffee00");
        assert_eq!(items[0].seeders, 812);
        assert!(items[0].size.unwrap() > 1_000_000_000);
        assert!(names_episode(&items[0].title, 3));
        assert!(!names_episode(&items[0].title, 4));
        assert!(!names_episode(&items[1].title, 3), "batch ranges are not single episodes");
        assert!(names_episode("[Erai-raws] Show - 12v2 [1080p]", 12));
        assert!(names_episode("Show S01E05 1080p WEB", 5));
        assert_eq!(season_of("[Raze] Sousou no Frieren S2 - 03 x265"), Some(2));
        assert_eq!(season_of("[SubsPlease] Sousou no Frieren - 03 (1080p)"), None);
        assert_eq!(season_of("Frieren 2nd Season - 03"), Some(2));
        assert_eq!(season_of("Show S01E05 1080p"), Some(1));
    }
}
