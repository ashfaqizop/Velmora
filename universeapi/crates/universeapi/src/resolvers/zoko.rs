//! ZokoAnime embed (the server behind hianime / ani-cli).
//!
//! The embed is addressable directly by **MyAnimeList id**:
//! `https://zokoanime.video/stream/mal/{mal}/{episode}/{sub|dub}`.
//! Its page ships `window.__P = base64(json XOR "otaku-embed-v1")` with the
//! HLS master, subtitles and a download path. Using the MAL id from AniList
//! skips hianime search/episode/server scraping entirely (1 request vs 4).

use std::sync::LazyLock;

use regex::Regex;
use serde::Deserialize;

use crate::error::{Error, Result};
use crate::model::{AudioMode, ProviderId, Source, StreamFormat, Track};
use crate::net::Http;
use crate::util::{b64_decode, origin_of, xor};

const KEY: &[u8] = b"otaku-embed-v1";

#[derive(Debug, Deserialize)]
pub struct EmbedConfig {
    pub src: Option<String>,
    #[serde(default, deserialize_with = "crate::util::null_default")]
    pub subtitles: Vec<EmbedSub>,
    pub download_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EmbedSub {
    pub lang: Option<String>,
    pub label: Option<String>,
    #[serde(default)]
    pub default: bool,
    pub src: String,
}

pub fn embed_url(base: &str, mal_id: u64, episode: u32, mode: AudioMode) -> String {
    let mode = match mode {
        AudioMode::Sub => "sub",
        AudioMode::Dub => "dub",
    };
    format!("{}/stream/mal/{mal_id}/{episode}/{mode}", base.trim_end_matches('/'))
}

/// Decode the `window.__P` blob from an embed page.
pub fn decode_page(html: &str) -> Result<EmbedConfig> {
    static BLOB: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"window\.__P\s*=\s*"([^"]+)""#).expect("regex"));
    let blob = BLOB.captures(html).ok_or_else(|| Error::parse("zokoanime", "window.__P not found"))?;
    let raw = b64_decode(&blob[1]).ok_or_else(|| Error::parse("zokoanime", "blob is not base64"))?;
    let json = xor(&raw, KEY);
    serde_json::from_slice(&json).map_err(|e| Error::parse("zokoanime blob", e.to_string()))
}

/// Resolve an embed URL (as found on hianime, or built from a MAL id).
pub async fn resolve_embed(http: &Http, provider: ProviderId, embed: &str, label: &str) -> Result<Vec<Source>> {
    let html = http.get_text(embed, &[("Referer", &origin_of(embed))]).await?;
    let cfg = decode_page(&html)?;
    let referer = origin_of(embed);
    let mut out = Vec::new();

    if let Some(src) = cfg.src.filter(|s| s.starts_with("http")) {
        let mut s = Source::new(provider, "ZokoAnime", label, src).with_referer(&referer);
        s.format = StreamFormat::Hls;
        s.subtitles = cfg
            .subtitles
            .iter()
            .map(|t| Track {
                lang: t.lang.clone(),
                label: t.label.clone().or_else(|| t.lang.clone()).unwrap_or_else(|| "Subtitles".into()),
                url: Some(t.src.clone()),
                default: t.default,
            })
            .collect();
        out.push(s);
    }
    if let Some(dl) = cfg.download_url.filter(|d| !d.is_empty()) {
        let url = crate::util::absolutize(embed, &dl);
        let mut s = Source::new(provider, "ZokoAnime", format!("{label} · Download"), url).with_referer(&referer);
        s.format = StreamFormat::Mp4;
        out.push(s);
    }
    if out.is_empty() {
        return Err(Error::NotFound(format!("no stream in embed {embed}")));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;

    #[test]
    fn decodes_blob() {
        let json = br#"{"src":"https://hls.x/master.m3u8","subtitles":[{"lang":"en","label":"English","default":true,"src":"https://hls.x/en.vtt"}],"download_url":"/download/mal/52991/1/sub"}"#;
        let blob = base64::engine::general_purpose::STANDARD.encode(xor(json, KEY));
        let html = format!(r#"<script>window.__P="{blob}";</script>"#);
        let cfg = decode_page(&html).unwrap();
        assert_eq!(cfg.src.as_deref(), Some("https://hls.x/master.m3u8"));
        assert_eq!(cfg.subtitles[0].lang.as_deref(), Some("en"));
        assert_eq!(cfg.download_url.as_deref(), Some("/download/mal/52991/1/sub"));
    }

    #[test]
    fn builds_mal_embed() {
        assert_eq!(
            embed_url("https://zokoanime.video/", 52991, 1, AudioMode::Dub),
            "https://zokoanime.video/stream/mal/52991/1/dub"
        );
    }
}
