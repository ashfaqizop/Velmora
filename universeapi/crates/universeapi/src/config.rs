//! Engine configuration.
//!
//! * [`Preferences`] — user-facing choices (quality, languages, provider order...).
//! * [`Registry`] — remotely-updatable data (mirror domains, API keys, toggles).
//!   The app downloads `registry.json` from the repo's `config` branch on launch,
//!   so a dead domain is fixed with a commit instead of an app release.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::model::{MediaKind, ProviderId};
use crate::release::VideoCodec;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    /// Preferred vertical resolution for auto-selection.
    pub preferred_resolution: u16,
    /// Never auto-select above this resolution (4K is manual-only by default).
    pub max_auto_resolution: u16,
    pub preferred_codec: VideoCodec,
    /// Ordered audio language preference (ISO 639-1).
    pub audio_languages: Vec<String>,
    /// Ordered subtitle language preference (ISO 639-1).
    pub subtitle_languages: Vec<String>,
    pub show_adult: bool,
    pub include_torrents: bool,
    pub include_french: bool,
    /// Provider priority per content kind; earlier = preferred.
    pub provider_order: BTreeMap<MediaKind, Vec<ProviderId>>,
    pub disabled_providers: Vec<ProviderId>,
    /// Probe resolved links before returning them (drops ad/decoy pages).
    pub verify_sources: bool,
    pub max_concurrent_downloads: usize,
    /// TMDB/Cinemeta region; `None` = global.
    pub region: Option<String>,
}

impl Default for Preferences {
    fn default() -> Self {
        use ProviderId::*;
        let mut provider_order = BTreeMap::new();
        let movie_tv = vec![MovieBox, HdHub, FourKHdHub, FrenchStream, Coflix, Yts, Eztv];
        provider_order.insert(MediaKind::Movie, movie_tv.clone());
        provider_order.insert(MediaKind::Series, movie_tv);
        provider_order.insert(MediaKind::Anime, vec![ZokoAnime, HiAnime, AnimeSama, Nyaa]);
        provider_order.insert(MediaKind::AsianDrama, vec![Dramachi, MovieBox]);
        Self {
            preferred_resolution: 1080,
            max_auto_resolution: 1080,
            preferred_codec: VideoCodec::H264,
            audio_languages: vec!["en".into()],
            subtitle_languages: vec!["en".into()],
            show_adult: false,
            include_torrents: true,
            include_french: true,
            provider_order,
            disabled_providers: Vec::new(),
            verify_sources: true,
            max_concurrent_downloads: 3,
            region: None,
        }
    }
}

impl Preferences {
    pub fn is_enabled(&self, p: ProviderId) -> bool {
        if self.disabled_providers.contains(&p) {
            return false;
        }
        if p.is_torrent() && !self.include_torrents {
            return false;
        }
        if matches!(p, ProviderId::AnimeSama | ProviderId::FrenchStream | ProviderId::Coflix) && !self.include_french {
            return false;
        }
        true
    }

    /// 0 = most preferred. Providers missing from the list rank after listed ones.
    pub fn provider_rank(&self, kind: MediaKind, p: ProviderId) -> usize {
        self.provider_order
            .get(&kind)
            .and_then(|order| order.iter().position(|x| *x == p))
            .unwrap_or(usize::MAX / 2)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Registry {
    /// Schema version; the engine refuses newer major versions.
    pub version: u32,
    /// Base URLs per provider. First entry is tried first; others are fallbacks.
    pub mirrors: BTreeMap<ProviderId, Vec<String>>,
    /// HDHub rotates domains constantly — everything about it is remote.
    pub hdhub: HdHubRemote,
    /// Hoster rebrands: maps a new domain *name label* to a known resolver
    /// family, e.g. `{"hubkloud": "hubcloud"}`. TLD changes need no entry.
    pub host_aliases: BTreeMap<String, String>,
    pub tmdb_api_key: Option<String>,
    pub tmdb_read_token: Option<String>,
    pub cinemeta_url: String,
    pub anilist_url: String,
    /// Remote kill-switches for misbehaving providers.
    pub disabled: Vec<ProviderId>,
    /// Chrome-like User-Agent used for scraping.
    pub user_agent: String,
}

pub const REGISTRY_VERSION: u32 = 1;

impl Default for Registry {
    fn default() -> Self {
        use ProviderId::*;
        let mut mirrors = BTreeMap::new();
        let mut m = |p: ProviderId, urls: &[&str]| {
            mirrors.insert(p, urls.iter().map(|s| s.to_string()).collect::<Vec<_>>());
        };
        m(HdHub, &["https://new10.hdhub4u.cl"]);
        m(HiAnime, &["https://hianime.at"]);
        m(ZokoAnime, &["https://zokoanime.video"]);
        m(
            MovieBox,
            &[
                "https://api6.aoneroom.com",
                "https://api5.aoneroom.com",
                "https://api4.aoneroom.com",
                "https://api4sg.aoneroom.com",
                "https://api3.aoneroom.com",
                "https://api6sg.aoneroom.com",
                "https://api.inmoviebox.com",
            ],
        );
        m(FourKHdHub, &["https://4khdhub.one"]);
        m(AnimeSama, &["https://anime-sama.pw"]);
        m(FrenchStream, &["https://french-stream.net"]);
        m(Coflix, &["https://coflix.ac"]);
        m(Yts, &["https://yts.mx"]);
        m(Eztv, &["https://eztvx.to"]);
        m(Nyaa, &["https://nyaa.si"]);
        Self {
            version: REGISTRY_VERSION,
            mirrors,
            hdhub: HdHubRemote::default(),
            host_aliases: BTreeMap::new(),
            tmdb_api_key: None,
            tmdb_read_token: None,
            cinemeta_url: "https://v3-cinemeta.strem.io".into(),
            anilist_url: "https://graphql.anilist.co".into(),
            disabled: Vec::new(),
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36".into(),
        }
    }
}

/// Remote-tunable HDHub settings. Mirror resolution order:
/// `force_mirror` → live host-config CDN → `mirrors[hd_hub]` list (failover).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HdHubRemote {
    /// Emergency override: skip auto-resolution and use exactly this mirror.
    pub force_mirror: Option<String>,
    /// Endpoint HDHub publishes its live mirror on (`{"c": base64(url)}`).
    pub host_cdn: String,
    /// Typesense search server (domain-independent from the site).
    pub search_url: String,
    pub search_collection: String,
    /// Home rails: (rail id, title, category slug; "" = latest).
    pub rails: Vec<(String, String, String)>,
}

impl Default for HdHubRemote {
    fn default() -> Self {
        Self {
            force_mirror: None,
            host_cdn: "https://h4.suncdn.org/host/".into(),
            search_url: "https://search.pingora.fyi".into(),
            search_collection: "post".into(),
            rails: vec![
                ("hdhub_latest".into(), "Latest on HDHub".into(), String::new()),
                ("hdhub_bollywood".into(), "Bollywood".into(), "bollywood-movies".into()),
                ("hdhub_south".into(), "South Hindi Dubbed".into(), "south-hindi-movies".into()),
            ],
        }
    }
}

impl Registry {
    /// Parse a remote registry, layering it over the built-in defaults so a
    /// partial file (e.g. only one mirror changed) is valid.
    pub fn from_json_over_defaults(json: &str) -> crate::Result<Self> {
        let remote: serde_json::Value =
            serde_json::from_str(json).map_err(|e| crate::Error::Config(format!("registry.json: {e}")))?;
        let mut base = serde_json::to_value(Self::default()).expect("registry serializes");
        merge_json(&mut base, remote);
        let reg: Self = serde_json::from_value(base).map_err(|e| crate::Error::Config(format!("registry.json: {e}")))?;
        if reg.version > REGISTRY_VERSION {
            return Err(crate::Error::Config(format!(
                "registry version {} is newer than supported {REGISTRY_VERSION}",
                reg.version
            )));
        }
        Ok(reg)
    }

    /// Apply secrets from the environment (CI secrets / app build-time
    /// injection) so keys never have to live in the public repo.
    pub fn with_env_overrides(mut self) -> Self {
        let env = |k: &str| std::env::var(k).ok().filter(|v| !v.trim().is_empty());
        if let Some(t) = env("UNIVERSEAPI_TMDB_READ_TOKEN") {
            self.tmdb_read_token = Some(t);
        }
        if let Some(k) = env("UNIVERSEAPI_TMDB_API_KEY") {
            self.tmdb_api_key = Some(k);
        }
        if let Some(m) = env("UNIVERSEAPI_HDHUB_MIRROR") {
            self.hdhub.force_mirror = Some(m);
        }
        self
    }

    pub fn mirror(&self, p: ProviderId) -> Option<&str> {
        self.mirrors.get(&p).and_then(|v| v.first()).map(String::as_str)
    }

    pub fn mirrors(&self, p: ProviderId) -> &[String] {
        self.mirrors.get(&p).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn has_tmdb(&self) -> bool {
        self.tmdb_api_key.as_deref().is_some_and(|k| !k.is_empty())
            || self.tmdb_read_token.as_deref().is_some_and(|k| !k.is_empty())
    }
}

/// Deep-merge `patch` into `base` (objects merge, everything else replaces).
fn merge_json(base: &mut serde_json::Value, patch: serde_json::Value) {
    match (base, patch) {
        (serde_json::Value::Object(b), serde_json::Value::Object(p)) => {
            for (k, v) in p {
                merge_json(b.entry(k).or_insert(serde_json::Value::Null), v);
            }
        }
        (b, p) => *b = p,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_registry_layers_over_defaults() {
        let reg = Registry::from_json_over_defaults(r#"{"mirrors":{"hd_hub":["https://new11.hdhub4u.limo"]}}"#).unwrap();
        assert_eq!(reg.mirror(ProviderId::HdHub), Some("https://new11.hdhub4u.limo"));
        assert_eq!(reg.mirror(ProviderId::HiAnime), Some("https://hianime.at"), "untouched mirrors keep defaults");
    }

    #[test]
    fn hdhub_remote_overrides() {
        let reg = Registry::from_json_over_defaults(
            r#"{"hdhub":{"force_mirror":"https://new12.hdhub4u.xyz","search_url":"https://s.new.fyi"},"host_aliases":{"hubkloud":"hubcloud"}}"#,
        )
        .unwrap();
        assert_eq!(reg.hdhub.force_mirror.as_deref(), Some("https://new12.hdhub4u.xyz"));
        assert_eq!(reg.hdhub.search_url, "https://s.new.fyi");
        assert_eq!(reg.hdhub.host_cdn, "https://h4.suncdn.org/host/", "unspecified fields keep defaults");
        assert_eq!(reg.hdhub.rails.len(), 3);
        assert_eq!(reg.host_aliases.get("hubkloud").map(String::as_str), Some("hubcloud"));
    }

    #[test]
    fn newer_registry_rejected() {
        assert!(Registry::from_json_over_defaults(r#"{"version":99}"#).is_err());
    }

    #[test]
    fn provider_ranking_follows_decisions() {
        let p = Preferences::default();
        assert!(p.provider_rank(MediaKind::Movie, ProviderId::MovieBox) < p.provider_rank(MediaKind::Movie, ProviderId::HdHub));
        assert!(p.provider_rank(MediaKind::Anime, ProviderId::ZokoAnime) < p.provider_rank(MediaKind::Anime, ProviderId::Nyaa));
        assert!(p.provider_rank(MediaKind::AsianDrama, ProviderId::Dramachi) < p.provider_rank(MediaKind::AsianDrama, ProviderId::MovieBox));
    }
}
