//! Unified domain model. Every provider maps its own data into these types so
//! the engine can merge, rank and hand results to the app without caring where
//! they came from.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::release::ReleaseTags;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderId {
    // metadata
    Cinemeta,
    Tmdb,
    Anilist,
    // streaming / download sources
    HdHub,
    HiAnime,
    ZokoAnime,
    MovieBox,
    FourKHdHub,
    Dramachi,
    AnimeSama,
    FrenchStream,
    Coflix,
    Yts,
    Eztv,
    Nyaa,
}

impl ProviderId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cinemeta => "cinemeta",
            Self::Tmdb => "tmdb",
            Self::Anilist => "anilist",
            Self::HdHub => "hd_hub",
            Self::HiAnime => "hi_anime",
            Self::ZokoAnime => "zoko_anime",
            Self::MovieBox => "movie_box",
            Self::FourKHdHub => "four_k_hd_hub",
            Self::Dramachi => "dramachi",
            Self::AnimeSama => "anime_sama",
            Self::FrenchStream => "french_stream",
            Self::Coflix => "coflix",
            Self::Yts => "yts",
            Self::Eztv => "eztv",
            Self::Nyaa => "nyaa",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Cinemeta => "Cinemeta",
            Self::Tmdb => "TMDB",
            Self::Anilist => "AniList",
            Self::HdHub => "HDHub",
            Self::HiAnime => "HiAnime",
            Self::ZokoAnime => "ZokoAnime",
            Self::MovieBox => "MovieBox",
            Self::FourKHdHub => "4KHDHub",
            Self::Dramachi => "Dramachi",
            Self::AnimeSama => "Anime-Sama",
            Self::FrenchStream => "French-Stream",
            Self::Coflix => "Coflix",
            Self::Yts => "YTS",
            Self::Eztv => "EZTV",
            Self::Nyaa => "Nyaa",
        }
    }

    pub fn is_torrent(self) -> bool {
        matches!(self, Self::Yts | Self::Eztv | Self::Nyaa)
    }
}

impl fmt::Display for ProviderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.display_name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaKind {
    Movie,
    Series,
    Anime,
    AsianDrama,
}

impl MediaKind {
    pub fn is_episodic(self) -> bool {
        !matches!(self, Self::Movie)
    }
}

/// Cross-provider identifiers used to merge the same title from many sources.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExternalIds {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imdb: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tmdb: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mal: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anilist: Option<u64>,
}

impl ExternalIds {
    pub fn is_empty(&self) -> bool {
        self.imdb.is_none() && self.tmdb.is_none() && self.mal.is_none() && self.anilist.is_none()
    }

    /// Normalises an IMDb id to the `tt1234567` form.
    pub fn normalize_imdb(raw: &str) -> Option<String> {
        let digits: String = raw.trim().trim_start_matches("tt").chars().filter(char::is_ascii_digit).collect();
        (!digits.is_empty()).then(|| format!("tt{digits:0>7}"))
    }

    /// Fill any missing ids from `other`.
    pub fn absorb(&mut self, other: &ExternalIds) {
        if self.imdb.is_none() {
            self.imdb.clone_from(&other.imdb);
        }
        self.tmdb = self.tmdb.or(other.tmdb);
        self.mal = self.mal.or(other.mal);
        self.anilist = self.anilist.or(other.anilist);
    }

    pub fn shares_any(&self, other: &ExternalIds) -> bool {
        (self.imdb.is_some() && self.imdb == other.imdb)
            || (self.tmdb.is_some() && self.tmdb == other.tmdb)
            || (self.mal.is_some() && self.mal == other.mal)
            || (self.anilist.is_some() && self.anilist == other.anilist)
    }
}

/// A reference to one provider's copy of a title.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProviderRef {
    pub provider: ProviderId,
    /// Provider-scoped id (slug, numeric id, URL — whatever the provider needs).
    pub id: String,
}

/// Lightweight item used in search results and home rails.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TitleSummary {
    pub origin: ProviderRef,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_name: Option<String>,
    pub kind: MediaKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backdrop: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<f32>,
    #[serde(default)]
    pub ids: ExternalIds,
    /// Short labels shown as chips: "VF", "VOSTFR", "Dual Audio", "4K", "Torrent"...
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub badges: Vec<String>,
    /// Every provider known to carry this title (filled by the engine's merger).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub also_on: Vec<ProviderRef>,
}

impl TitleSummary {
    pub fn new(provider: ProviderId, id: impl Into<String>, name: impl Into<String>, kind: MediaKind) -> Self {
        Self {
            origin: ProviderRef { provider, id: id.into() },
            name: name.into(),
            original_name: None,
            kind,
            year: None,
            poster: None,
            backdrop: None,
            overview: None,
            rating: None,
            ids: ExternalIds::default(),
            badges: Vec::new(),
            also_on: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Episode {
    pub season: u32,
    pub number: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub air_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    /// Provider-specific episode key, when the provider needs one to fetch sources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Season {
    pub number: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub episodes: Vec<Episode>,
}

/// An unresolved link found on a provider page (a hoster page, not yet a playable URL).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostedLink {
    pub label: String,
    pub url: String,
    pub host: String,
    pub tags: ReleaseTags,
    /// `None` for movies and whole-season packs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episode: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub season: Option<u32>,
    pub is_pack: bool,
    /// Link opens a web player rather than a file.
    pub is_stream_page: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TitleDetail {
    pub summary: TitleSummary,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub genres: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cast: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub directors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_minutes: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailer: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub screenshots: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub seasons: Vec<Season>,
    /// Provider-page links that still need a resolver pass (HDHub style pages).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<HostedLink>,
}

impl TitleDetail {
    pub fn from_summary(summary: TitleSummary) -> Self {
        Self {
            summary,
            genres: Vec::new(),
            cast: Vec::new(),
            directors: Vec::new(),
            runtime_minutes: None,
            trailer: None,
            screenshots: Vec::new(),
            seasons: Vec::new(),
            links: Vec::new(),
        }
    }
}

/// What the user wants to watch / download.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Target {
    Movie,
    Episode { season: u32, episode: u32 },
    /// A whole season (downloads of season packs).
    Season { season: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioMode {
    Sub,
    Dub,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamFormat {
    Hls,
    Dash,
    Mp4,
    Mkv,
    Webm,
    /// zip/rar season pack — download only.
    Archive,
    Torrent,
    Unknown,
}

impl StreamFormat {
    pub fn from_url_or_name(s: &str) -> Self {
        let lower = s.to_ascii_lowercase();
        let path = lower.split(['?', '#']).next().unwrap_or(&lower);
        if path.ends_with(".m3u8") || path.contains(".m3u8") || path.contains("/hls") {
            Self::Hls
        } else if path.ends_with(".mpd") {
            Self::Dash
        } else if path.ends_with(".mp4") || path.ends_with(".m4v") {
            Self::Mp4
        } else if path.ends_with(".mkv") {
            Self::Mkv
        } else if path.ends_with(".webm") {
            Self::Webm
        } else if path.ends_with(".zip") || path.ends_with(".rar") || path.ends_with(".7z") {
            Self::Archive
        } else if lower.starts_with("magnet:") || path.ends_with(".torrent") {
            Self::Torrent
        } else {
            Self::Unknown
        }
    }

    pub fn is_streamable(self) -> bool {
        !matches!(self, Self::Archive)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Track {
    /// BCP-47-ish language code when known ("en", "hi", "fr").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default)]
    pub default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TorrentInfo {
    pub magnet: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seeders: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leechers: Option<u32>,
    /// Index of the wanted file inside a multi-file torrent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_index: Option<usize>,
}

/// A playable and/or downloadable media source — the final output of the engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Source {
    pub provider: ProviderId,
    /// Which hoster/resolver produced the final URL ("HubCDN", "ZokoAnime", ...).
    pub host: String,
    pub label: String,
    pub url: String,
    pub format: StreamFormat,
    /// Headers that must be sent with every request for this media (Referer, Cookie, ...).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, String>,
    pub tags: ReleaseTags,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subtitles: Vec<Track>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub audio: Vec<Track>,
    /// Unix seconds after which the signed URL stops working.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub torrent: Option<TorrentInfo>,
    /// Season pack covering the whole season rather than one episode.
    #[serde(default)]
    pub is_pack: bool,
    /// Set by the verifier: `Some(true)` = probed and serves media bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
    /// Ranking score computed by the engine (higher is better).
    #[serde(default)]
    pub score: i32,
}

impl Source {
    pub fn new(provider: ProviderId, host: impl Into<String>, label: impl Into<String>, url: impl Into<String>) -> Self {
        let url = url.into();
        let label = label.into();
        let format = StreamFormat::from_url_or_name(&url);
        Self {
            provider,
            host: host.into(),
            tags: ReleaseTags::parse(&label),
            label,
            format,
            url,
            headers: BTreeMap::new(),
            filename: None,
            size_bytes: None,
            subtitles: Vec::new(),
            audio: Vec::new(),
            expires_at: None,
            torrent: None,
            is_pack: false,
            verified: None,
            score: 0,
        }
    }

    pub fn with_header(mut self, key: &str, value: impl Into<String>) -> Self {
        self.headers.insert(key.to_string(), value.into());
        self
    }

    pub fn with_referer(self, referer: impl Into<String>) -> Self {
        self.with_header("Referer", referer)
    }

    pub fn is_torrent(&self) -> bool {
        self.torrent.is_some() || self.format == StreamFormat::Torrent
    }

    /// Re-derive tags from everything we know (label + filename).
    pub fn retag(&mut self) {
        let mut text = self.label.clone();
        if let Some(name) = &self.filename {
            text.push(' ');
            text.push_str(name);
        }
        let mut tags = ReleaseTags::parse(&text);
        tags.merge_missing(&self.tags);
        if tags.size_bytes.is_none() {
            tags.size_bytes = self.size_bytes;
        }
        self.tags = tags;
        if self.format == StreamFormat::Unknown {
            if let Some(name) = &self.filename {
                self.format = StreamFormat::from_url_or_name(name);
            }
        }
    }
}

/// A row on the home screen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rail {
    pub id: String,
    pub title: String,
    pub items: Vec<TitleSummary>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imdb_normalization() {
        assert_eq!(ExternalIds::normalize_imdb("4154796").as_deref(), Some("tt4154796"));
        assert_eq!(ExternalIds::normalize_imdb("tt0111161").as_deref(), Some("tt0111161"));
        assert_eq!(ExternalIds::normalize_imdb("12345").as_deref(), Some("tt0012345"));
        assert_eq!(ExternalIds::normalize_imdb(""), None);
    }

    #[test]
    fn format_detection() {
        assert_eq!(StreamFormat::from_url_or_name("https://x/master.m3u8?t=1"), StreamFormat::Hls);
        assert_eq!(StreamFormat::from_url_or_name("Movie.2025.1080p.mkv"), StreamFormat::Mkv);
        assert_eq!(StreamFormat::from_url_or_name("Show.S01.PACK.zip"), StreamFormat::Archive);
        assert_eq!(StreamFormat::from_url_or_name("magnet:?xt=urn:btih:abc"), StreamFormat::Torrent);
        assert_eq!(StreamFormat::from_url_or_name("https://r2.dev/744585abec"), StreamFormat::Unknown);
    }

    #[test]
    fn ids_merge() {
        let mut a = ExternalIds { imdb: Some("tt1".into()), ..Default::default() };
        let b = ExternalIds { imdb: Some("tt2".into()), mal: Some(5), ..Default::default() };
        a.absorb(&b);
        assert_eq!(a.imdb.as_deref(), Some("tt1"));
        assert_eq!(a.mal, Some(5));
        assert!(a.shares_any(&ExternalIds { mal: Some(5), ..Default::default() }));
    }
}
