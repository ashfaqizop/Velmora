//! Live TV from the public iptv-org playlist (`channels`, `streams`,
//! `blocklist`, `countries` and `logos` JSON files).
//!
//! The library keeps what the playlist itself says is fine to list: channels on
//! its blocklist (copyright and adult entries), closed channels and adult
//! categories are dropped, and so is any channel with no stream. On top of that
//! the registry carries a remote hide list (`tv.hidden_channels`,
//! `tv.hidden_hosts`) so a channel can be pulled without an app release.
//!
//! Streams die and move all the time. [`Tv::check`] probes a channel's streams
//! so the app can show which ones work. The program guide (EPG) is not part of
//! this module yet.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use futures::future::join_all;
use serde::{Deserialize, Serialize};

use crate::cache::ttl;
use crate::error::{Error, Result};
use crate::model::{ProviderId, Source};
use crate::net::Http;
use crate::providers::Ctx;

/// One way to watch a channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TvStream {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    /// Header the stream insists on, sent as `Referer`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    /// Only plays in some countries.
    pub geo_blocked: bool,
    /// Not on the air around the clock.
    pub not_24_7: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TvChannel {
    pub id: String,
    pub name: String,
    /// ISO 3166-1 alpha-2, uppercase.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    pub categories: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
    /// Best first: reachable-everywhere before geo-blocked, then by quality.
    pub streams: Vec<TvStream>,
}

impl TvChannel {
    /// A source for the media proxy and the player, with the headers the
    /// stream asks for.
    pub fn source(&self, stream: usize) -> Option<Source> {
        let s = self.streams.get(stream)?;
        let mut src = Source::new(ProviderId::IptvOrg, "iptv-org", self.name.clone(), s.url.clone());
        if let Some(r) = &s.referrer {
            src = src.with_referer(r.clone());
        }
        if let Some(ua) = &s.user_agent {
            src = src.with_header("User-Agent", ua.clone());
        }
        Some(src)
    }
}

/// Everything the playlist lists that passed the filters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TvLibrary {
    pub channels: Vec<TvChannel>,
    /// ISO code to country name.
    pub countries: BTreeMap<String, String>,
    /// Channels left out because the playlist's blocklist names them.
    pub blocked: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TvQuery {
    /// ISO country code.
    pub country: Option<String>,
    /// A category id such as `sports` or `news`.
    pub category: Option<String>,
    /// Matches channel and network names.
    pub text: Option<String>,
    pub offset: usize,
    /// 0 means 50; capped at 500.
    pub limit: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TvPage {
    pub total: usize,
    pub items: Vec<TvChannel>,
}

/// A country or category with how many channels it has.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Facet {
    pub id: String,
    pub name: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamHealth {
    pub url: String,
    pub ok: bool,
    /// HTTP status, 0 when nothing answered.
    pub status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

pub struct Tv {
    ctx: Arc<Ctx>,
    lib: tokio::sync::Mutex<Option<(Instant, Arc<TvLibrary>)>>,
}

impl Tv {
    pub fn new(ctx: Arc<Ctx>) -> Self {
        Self { ctx, lib: tokio::sync::Mutex::new(None) }
    }

    /// The channel library. Loaded on first use and kept for six hours; if a
    /// refresh fails the last good copy keeps working.
    pub async fn library(&self) -> Result<Arc<TvLibrary>> {
        let mut slot = self.lib.lock().await;
        if let Some((at, lib)) = slot.as_ref() {
            if at.elapsed() < ttl::TV_LIBRARY {
                return Ok(lib.clone());
            }
        }
        match self.fetch().await {
            Ok(lib) => {
                let lib = Arc::new(lib);
                *slot = Some((Instant::now(), lib.clone()));
                Ok(lib)
            }
            Err(e) => match slot.as_ref() {
                Some((_, old)) => {
                    tracing::warn!("iptv-org refresh failed, keeping the last library: {e}");
                    Ok(old.clone())
                }
                None => Err(e),
            },
        }
    }

    async fn fetch(&self) -> Result<TvLibrary> {
        let api = self.ctx.registry().tv.api_url.trim_end_matches('/').to_string();
        let http = &self.ctx.http;
        let url = |name: &str| format!("{api}/{name}.json");
        let (u_channels, u_streams, u_blocklist, u_countries, u_logos) =
            (url("channels"), url("streams"), url("blocklist"), url("countries"), url("logos"));

        let core = async {
            futures::try_join!(
                http.get_json::<Vec<RawChannel>>(&u_channels, &[]),
                http.get_json::<Vec<RawStream>>(&u_streams, &[]),
                http.get_json::<Vec<RawBlock>>(&u_blocklist, &[]),
                http.get_json::<Vec<RawCountry>>(&u_countries, &[]),
            )
        };
        let (core, logos) = futures::join!(core, http.get_json::<Vec<RawLogo>>(&u_logos, &[]));
        let (channels, streams, blocklist, countries) = core?;
        // Logos are a nicety; a missing file shouldn't lose the whole library.
        let logos = logos.unwrap_or_else(|e| {
            tracing::warn!("iptv-org logos: {e}");
            Vec::new()
        });
        Ok(build(channels, streams, blocklist, countries, logos))
    }

    /// Channels matching a query, honoring the remote hide list.
    pub async fn channels(&self, q: &TvQuery) -> Result<TvPage> {
        let lib = self.library().await?;
        let hidden = Hidden::from_registry(&self.ctx);
        Ok(query(&lib, &hidden, q))
    }

    /// One channel by id.
    pub async fn channel(&self, id: &str) -> Result<Option<TvChannel>> {
        let lib = self.library().await?;
        let hidden = Hidden::from_registry(&self.ctx);
        Ok(lib.channels.iter().find(|c| c.id.eq_ignore_ascii_case(id) && hidden.allows(c)).map(|c| hidden.strip(c)))
    }

    /// Countries that have channels, most channels first.
    pub async fn countries(&self) -> Result<Vec<Facet>> {
        let lib = self.library().await?;
        let hidden = Hidden::from_registry(&self.ctx);
        Ok(facets(&lib, &hidden, |c| c.country.iter().cloned().collect()))
    }

    /// Categories that have channels, most channels first.
    pub async fn categories(&self) -> Result<Vec<Facet>> {
        let lib = self.library().await?;
        let hidden = Hidden::from_registry(&self.ctx);
        Ok(facets(&lib, &hidden, |c| c.categories.clone()))
    }

    /// Probes a channel's best streams with a tiny ranged request each.
    /// Results are kept for 15 minutes.
    pub async fn check(&self, channel_id: &str) -> Result<Vec<StreamHealth>> {
        let ch = self.channel(channel_id).await?.ok_or_else(|| Error::NotFound(format!("TV channel {channel_id}")))?;
        let key = format!("tv:health:{}", ch.id);
        if let Some(hit) = self.ctx.cache.get::<Vec<StreamHealth>>(&key).await {
            return Ok(hit);
        }
        let http = &self.ctx.http;
        let out = join_all(ch.streams.iter().take(6).map(|s| probe_stream(http, s))).await;
        self.ctx.cache.put(&key, &out, ttl::TV_HEALTH).await;
        Ok(out)
    }
}

async fn probe_stream(http: &Http, s: &TvStream) -> StreamHealth {
    let mut headers: Vec<(String, String)> = Vec::new();
    if let Some(r) = &s.referrer {
        headers.push(("Referer".into(), r.clone()));
    }
    if let Some(ua) = &s.user_agent {
        headers.push(("User-Agent".into(), ua.clone()));
    }
    match http.probe(&s.url, &headers).await {
        Ok(p) if p.is_media => StreamHealth { url: s.url.clone(), ok: true, status: p.status, detail: None },
        Ok(p) => StreamHealth { url: s.url.clone(), ok: false, status: p.status, detail: Some("not a video stream".into()) },
        Err(e) => StreamHealth { url: s.url.clone(), ok: false, status: 0, detail: Some(e.to_string()) },
    }
}

// ── The playlist's raw files ────────────────────────────────────────────

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RawChannel {
    id: String,
    name: String,
    country: Option<String>,
    network: Option<String>,
    #[serde(deserialize_with = "crate::util::null_default")]
    categories: Vec<String>,
    is_nsfw: bool,
    closed: Option<String>,
    website: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RawStream {
    channel: Option<String>,
    url: String,
    quality: Option<String>,
    #[serde(deserialize_with = "crate::util::null_default")]
    labels: Vec<String>,
    user_agent: Option<String>,
    referrer: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RawBlock {
    channel: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RawCountry {
    code: String,
    name: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct RawLogo {
    channel: Option<String>,
    url: String,
    in_use: bool,
}

/// Height in lines from "1080p" or "576i"; 0 when unknown.
fn quality_rank(q: Option<&str>) -> u32 {
    q.map(|q| q.chars().take_while(char::is_ascii_digit).collect::<String>()).and_then(|d| d.parse().ok()).unwrap_or(0)
}

fn nonempty(s: Option<String>) -> Option<String> {
    s.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// Joins the files into the library, applying the playlist's own filters.
fn build(
    channels: Vec<RawChannel>,
    streams: Vec<RawStream>,
    blocklist: Vec<RawBlock>,
    countries: Vec<RawCountry>,
    logos: Vec<RawLogo>,
) -> TvLibrary {
    let blocked: HashSet<String> = blocklist.into_iter().filter_map(|b| b.channel).collect();

    let mut by_channel: HashMap<String, Vec<TvStream>> = HashMap::new();
    for s in streams {
        let Some(channel) = s.channel else { continue };
        if !(s.url.starts_with("http://") || s.url.starts_with("https://")) {
            continue;
        }
        by_channel.entry(channel).or_default().push(TvStream {
            url: s.url,
            quality: nonempty(s.quality),
            referrer: nonempty(s.referrer),
            user_agent: nonempty(s.user_agent),
            geo_blocked: s.labels.iter().any(|l| l == "Geo-blocked"),
            not_24_7: s.labels.iter().any(|l| l == "Not 24/7"),
        });
    }

    // A logo the playlist marks as in use wins over an older one.
    let mut logo_of: HashMap<String, String> = HashMap::new();
    for l in logos {
        let Some(channel) = l.channel else { continue };
        if !l.url.starts_with("http") {
            continue;
        }
        if l.in_use {
            logo_of.insert(channel, l.url);
        } else {
            logo_of.entry(channel).or_insert(l.url);
        }
    }

    let mut blocked_count = 0;
    let mut out = Vec::new();
    for c in channels {
        if blocked.contains(&c.id) {
            blocked_count += 1;
            continue;
        }
        if c.closed.is_some() || c.is_nsfw || c.categories.iter().any(|x| x == "xxx") {
            continue;
        }
        let Some(mut streams) = by_channel.remove(&c.id) else { continue };
        let mut seen = HashSet::new();
        streams.retain(|s| seen.insert(s.url.clone()));
        streams.sort_by_key(|s| (s.geo_blocked, std::cmp::Reverse(quality_rank(s.quality.as_deref()))));
        out.push(TvChannel {
            logo: logo_of.remove(&c.id),
            id: c.id,
            name: c.name,
            country: nonempty(c.country).map(|c| c.to_uppercase()),
            network: nonempty(c.network),
            categories: c.categories,
            website: nonempty(c.website),
            streams,
        });
    }
    out.sort_by_cached_key(|c| c.name.to_lowercase());

    TvLibrary { channels: out, countries: countries.into_iter().map(|c| (c.code.to_uppercase(), c.name)).collect(), blocked: blocked_count }
}

// ── Queries ─────────────────────────────────────────────────────────────

/// The registry's remote hide list.
struct Hidden {
    ids: HashSet<String>,
    hosts: Vec<String>,
}

impl Hidden {
    fn from_registry(ctx: &Ctx) -> Self {
        let reg = ctx.registry();
        Self {
            ids: reg.tv.hidden_channels.iter().map(|c| c.trim().to_lowercase()).collect(),
            hosts: reg.tv.hidden_hosts.iter().map(|h| h.trim().to_lowercase()).filter(|h| !h.is_empty()).collect(),
        }
    }

    fn host_hidden(&self, url: &str) -> bool {
        if self.hosts.is_empty() {
            return false;
        }
        let host = crate::util::host_of(url).to_lowercase();
        self.hosts.iter().any(|h| host == *h || host.ends_with(&format!(".{h}")))
    }

    /// Visible: not hidden, and at least one stream left.
    fn allows(&self, c: &TvChannel) -> bool {
        !self.ids.contains(&c.id.to_lowercase()) && c.streams.iter().any(|s| !self.host_hidden(&s.url))
    }

    /// The channel without its hidden-host streams.
    fn strip(&self, c: &TvChannel) -> TvChannel {
        let mut out = c.clone();
        out.streams.retain(|s| !self.host_hidden(&s.url));
        out
    }
}

fn query(lib: &TvLibrary, hidden: &Hidden, q: &TvQuery) -> TvPage {
    let text = q.text.as_deref().map(|t| t.trim().to_lowercase()).filter(|t| !t.is_empty());
    let country = q.country.as_deref().map(|c| c.trim().to_uppercase()).filter(|c| !c.is_empty());
    let category = q.category.as_deref().map(|c| c.trim().to_lowercase()).filter(|c| !c.is_empty());

    let mut hits: Vec<(u8, &TvChannel)> = lib
        .channels
        .iter()
        .filter(|c| hidden.allows(c))
        .filter(|c| country.as_deref().is_none_or(|k| c.country.as_deref() == Some(k)))
        .filter(|c| category.as_deref().is_none_or(|k| c.categories.iter().any(|x| x == k)))
        .filter_map(|c| match &text {
            None => Some((0, c)),
            Some(t) => {
                let name = c.name.to_lowercase();
                if name.starts_with(t.as_str()) {
                    Some((0, c))
                } else if name.contains(t.as_str()) {
                    Some((1, c))
                } else if c.network.as_deref().is_some_and(|n| n.to_lowercase().contains(t.as_str())) {
                    Some((2, c))
                } else {
                    None
                }
            }
        })
        .collect();
    hits.sort_by_cached_key(|(rank, c)| (*rank, c.name.to_lowercase()));

    let limit = if q.limit == 0 { 50 } else { q.limit.min(500) };
    TvPage {
        total: hits.len(),
        items: hits.into_iter().skip(q.offset).take(limit).map(|(_, c)| hidden.strip(c)).collect(),
    }
}

fn facets(lib: &TvLibrary, hidden: &Hidden, keys: impl Fn(&TvChannel) -> Vec<String>) -> Vec<Facet> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for c in lib.channels.iter().filter(|c| hidden.allows(c)) {
        for k in keys(c) {
            *counts.entry(k).or_default() += 1;
        }
    }
    let mut out: Vec<Facet> = counts
        .into_iter()
        .map(|(id, count)| {
            let name = lib.countries.get(&id).cloned().unwrap_or_else(|| title_case(&id));
            Facet { id, name, count }
        })
        .collect();
    out.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));
    out
}

fn title_case(id: &str) -> String {
    let mut chars = id.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ch(id: &str, name: &str, country: &str, categories: &[&str]) -> RawChannel {
        RawChannel {
            id: id.into(),
            name: name.into(),
            country: Some(country.into()),
            categories: categories.iter().map(|c| c.to_string()).collect(),
            ..Default::default()
        }
    }

    fn st(channel: &str, url: &str, quality: &str, labels: &[&str]) -> RawStream {
        RawStream {
            channel: Some(channel.into()),
            url: url.into(),
            quality: Some(quality.into()),
            labels: labels.iter().map(|l| l.to_string()).collect(),
            ..Default::default()
        }
    }

    fn lib() -> TvLibrary {
        let mut closed = ch("Closed.us", "Closed One", "US", &["general"]);
        closed.closed = Some("2020-01-01".into());
        let mut adult = ch("Adult.us", "Adult One", "US", &["general"]);
        adult.is_nsfw = true;
        let channels = vec![
            ch("NewsOne.us", "News One", "us", &["news"]),
            ch("SportsHub.us", "Sports Hub", "US", &["sports"]),
            ch("Blocked.us", "Blocked One", "US", &["sports"]),
            ch("NoStream.us", "No Stream", "US", &["news"]),
            ch("Kids.xx", "Kids", "XX", &["xxx"]),
            ch("Dubbed.in", "Hindi Sports", "IN", &["sports"]),
            closed,
            adult,
        ];
        let streams = vec![
            st("SportsHub.us", "https://a.example/low.m3u8", "480p", &[]),
            st("SportsHub.us", "https://a.example/geo.m3u8", "1080p", &["Geo-blocked"]),
            st("SportsHub.us", "https://b.example/hd.m3u8", "720p", &["Not 24/7"]),
            st("SportsHub.us", "https://b.example/hd.m3u8", "720p", &[]),
            st("SportsHub.us", "rtmp://c.example/live", "720p", &[]),
            st("NewsOne.us", "https://news.example/live.m3u8", "1080p", &[]),
            st("Blocked.us", "https://x.example/live.m3u8", "1080p", &[]),
            st("Closed.us", "https://x.example/c.m3u8", "1080p", &[]),
            st("Adult.us", "https://x.example/a.m3u8", "1080p", &[]),
            st("Kids.xx", "https://x.example/k.m3u8", "1080p", &[]),
            st("Dubbed.in", "https://in.example/s.m3u8", "720p", &[]),
            RawStream { channel: None, url: "https://orphan.example/x.m3u8".into(), ..Default::default() },
        ];
        let blocklist = vec![RawBlock { channel: Some("Blocked.us".into()) }, RawBlock { channel: None }];
        let countries = vec![
            RawCountry { code: "US".into(), name: "United States".into() },
            RawCountry { code: "IN".into(), name: "India".into() },
        ];
        let logos = vec![
            RawLogo { channel: Some("SportsHub.us".into()), url: "https://img.example/old.png".into(), in_use: false },
            RawLogo { channel: Some("SportsHub.us".into()), url: "https://img.example/new.png".into(), in_use: true },
            RawLogo { channel: Some("NewsOne.us".into()), url: "ftp://nope".into(), in_use: true },
        ];
        build(channels, streams, blocklist, countries, logos)
    }

    fn none_hidden() -> Hidden {
        Hidden { ids: HashSet::new(), hosts: Vec::new() }
    }

    #[test]
    fn library_keeps_only_what_the_playlist_allows() {
        let l = lib();
        let names: Vec<_> = l.channels.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["Hindi Sports", "News One", "Sports Hub"], "blocked, closed, adult and stream-less channels are dropped");
        assert_eq!(l.blocked, 1);
        assert_eq!(l.countries.get("IN").map(String::as_str), Some("India"));
    }

    #[test]
    fn streams_are_cleaned_and_ordered() {
        let l = lib();
        let hub = l.channels.iter().find(|c| c.id == "SportsHub.us").unwrap();
        let urls: Vec<_> = hub.streams.iter().map(|s| s.url.as_str()).collect();
        // No rtmp, no duplicate; reachable everywhere first, best quality first; geo-blocked last.
        assert_eq!(urls, ["https://b.example/hd.m3u8", "https://a.example/low.m3u8", "https://a.example/geo.m3u8"]);
        assert!(hub.streams[2].geo_blocked);
        assert_eq!(hub.logo.as_deref(), Some("https://img.example/new.png"), "the logo in use wins");
        let news = l.channels.iter().find(|c| c.id == "NewsOne.us").unwrap();
        assert_eq!(news.country.as_deref(), Some("US"), "country codes are uppercased");
        assert_eq!(news.logo, None, "a non-http logo is ignored");
    }

    #[test]
    fn quality_ranks_by_height() {
        assert_eq!(quality_rank(Some("1080p")), 1080);
        assert_eq!(quality_rank(Some("576i")), 576);
        assert_eq!(quality_rank(Some("4K")), 4);
        assert_eq!(quality_rank(None), 0);
    }

    #[test]
    fn query_filters_ranks_and_pages() {
        let l = lib();
        let h = none_hidden();
        let q = |text: &str| TvQuery { text: Some(text.into()), ..Default::default() };

        assert_eq!(query(&l, &h, &TvQuery::default()).total, 3);
        let us = query(&l, &h, &TvQuery { country: Some("us".into()), ..Default::default() });
        assert_eq!(us.total, 2);
        let sports = query(&l, &h, &TvQuery { category: Some("Sports".into()), ..Default::default() });
        assert_eq!(sports.total, 2);

        // A name that starts with the text beats one that only contains it.
        let r = query(&l, &h, &q("sports"));
        assert_eq!(r.items[0].name, "Sports Hub");
        assert_eq!(r.items[1].name, "Hindi Sports");

        let page = query(&l, &h, &TvQuery { limit: 1, offset: 1, ..Default::default() });
        assert_eq!((page.total, page.items.len()), (3, 1));
    }

    #[test]
    fn remote_hide_list_removes_channels_and_hosts() {
        let l = lib();
        let hide_channel = Hidden { ids: ["sportshub.us".to_string()].into_iter().collect(), hosts: Vec::new() };
        assert_eq!(query(&l, &hide_channel, &TvQuery::default()).total, 2);

        // Hiding a host strips those streams; a channel left with none disappears.
        let hide_host = Hidden { ids: HashSet::new(), hosts: vec!["b.example".into(), "news.example".into()] };
        let r = query(&l, &hide_host, &TvQuery::default());
        assert_eq!(r.total, 2, "News One only streamed from the hidden host");
        let hub = r.items.iter().find(|c| c.id == "SportsHub.us").unwrap();
        assert!(hub.streams.iter().all(|s| !s.url.contains("b.example")));
        assert!(hide_host.host_hidden("https://cdn.b.example/x.m3u8"), "subdomains are hidden too");
        assert!(!hide_host.host_hidden("https://notb.example/x.m3u8"));
    }

    #[test]
    fn facets_count_and_name() {
        let l = lib();
        let countries = facets(&l, &none_hidden(), |c| c.country.iter().cloned().collect());
        assert_eq!(countries[0], Facet { id: "US".into(), name: "United States".into(), count: 2 });
        let categories = facets(&l, &none_hidden(), |c| c.categories.clone());
        assert_eq!(categories[0], Facet { id: "sports".into(), name: "Sports".into(), count: 2 });
    }

    #[test]
    fn source_carries_the_headers_the_stream_needs() {
        let mut l = lib();
        let hub = l.channels.iter_mut().find(|c| c.id == "SportsHub.us").unwrap();
        hub.streams[0].referrer = Some("https://player.example/".into());
        hub.streams[0].user_agent = Some("TestAgent/1.0".into());
        let src = hub.source(0).unwrap();
        assert_eq!(src.provider, ProviderId::IptvOrg);
        assert_eq!(src.label, "Sports Hub");
        assert_eq!(src.headers.get("Referer").map(String::as_str), Some("https://player.example/"));
        assert_eq!(src.headers.get("User-Agent").map(String::as_str), Some("TestAgent/1.0"));
        assert!(hub.source(99).is_none());
    }
}
