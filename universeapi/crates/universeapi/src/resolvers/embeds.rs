//! Generic resolver for French-site video players (uqload, fsvid, vidzy,
//! ansembed/vidmoly, VOE and its rotating mirrors, sibnet, sendvid, and the
//! filemoon family). It works by page content rather than by host name, so
//! a new mirror domain needs no code change:
//!
//! 1. follow JavaScript redirects (VOE's `/e/` stubs hop across domains)
//! 2. VOE's 2026 JSON config: ROT13, strip markers, base64, shift -3,
//!    reverse, base64 → JSON with the HLS/MP4 URL
//! 3. host quirks for sibnet and sendvid
//! 4. unpack every P.A.C.K.E.R block and look for HLS/MP4 URLs, skipping
//!    decoy playlists (`/troll/`)

use std::sync::LazyLock;

use regex::Regex;

use crate::error::{Error, Result};
use crate::model::{ProviderId, Source, StreamFormat};
use crate::net::Http;
use crate::util::{b64_decode, host_of, origin_of, unpack_packer_all};

static JS_REDIRECT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"window\.location\.(?:href|replace)\s*[=(]\s*['"](https?://[^'"]+)['"]"#).expect("regex"));
static JSON_CONFIG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?s)<script[^>]*type=["']application/json["'][^>]*>(.*?)</script>"#).expect("regex"));
static MEDIA_URL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"https?://[^"'\s\\<>]+?\.(?:m3u8|mp4|txt)(?:\?[^"'\s\\<>]*)?"#).expect("regex"));
static SIBNET: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"src\s*:\s*["'](/v/[^"']+\.mp4)["']"#).expect("regex"));
static SOURCE_TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<source[^>]+src=["'](https?://[^"']+)["']|property=["']og:video["'][^>]+content=["'](https?://[^"']+)["']"#).expect("regex"));

/// Resolve a player embed URL into playable sources.
pub async fn resolve(http: &Http, provider: ProviderId, url: &str, label: &str, referer: Option<&str>) -> Result<Vec<Source>> {
    let mut page_url = url.to_string();
    let mut referer = referer.map(str::to_string).unwrap_or_else(|| origin_of(url));
    let mut html = http.get_text(&page_url, &[("Referer", &referer)]).await?;

    for _ in 0..3 {
        let next = JS_REDIRECT.captures(crate::net::prefix(&html, 6000)).map(|c| c[1].to_string());
        match next {
            Some(n) if n != page_url => {
                referer = page_url.clone();
                page_url = n;
                html = http.get_text(&page_url, &[("Referer", &referer)]).await?;
            }
            _ => break,
        }
    }

    let host = host_of(&page_url);
    let found = voe_config(&html)
        .or_else(|| host.contains("sibnet").then(|| SIBNET.captures(&html).map(|c| format!("https://video.sibnet.ru{}", &c[1]))).flatten())
        .or_else(|| {
            host.contains("sendvid")
                .then(|| SOURCE_TAG.captures(&html).and_then(|c| c.get(1).or_else(|| c.get(2)).map(|m| m.as_str().to_string())))
                .flatten()
        })
        .or_else(|| best_media_url(&html));

    let media = found.ok_or_else(|| Error::parse("embed player", format!("no stream found on {host}")))?;
    let mut s = Source::new(provider, host_label(&host), label, media).with_referer(origin_of(&page_url));
    if s.format == StreamFormat::Unknown {
        s.format = if s.url.contains(".m3u8") || s.url.contains("master.txt") { StreamFormat::Hls } else { StreamFormat::Mp4 };
    }
    Ok(vec![s])
}

fn host_label(host: &str) -> String {
    let name = host.rsplit('.').nth(1).unwrap_or(host);
    let mut c = name.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

/// Best HLS/MP4 URL across the raw page and every unpacked script.
pub fn best_media_url(html: &str) -> Option<String> {
    let mut hay = unpack_packer_all(html).join("\n");
    hay.push('\n');
    hay.push_str(html);
    let mut cands: Vec<String> = MEDIA_URL
        .find_iter(&hay)
        .map(|m| m.as_str().to_string())
        .filter(|u| !is_decoy(u))
        .collect();
    cands.dedup();
    let rank = |u: &str| {
        if u.contains("master.m3u8") || u.contains("master.txt") {
            0
        } else if u.contains(".m3u8") {
            1
        } else {
            2
        }
    };
    cands.sort_by_key(|u| rank(u));
    cands.into_iter().next()
}

fn is_decoy(u: &str) -> bool {
    let l = u.to_ascii_lowercase();
    l.contains("/troll/") || l.contains("/ads/") || l.contains("preroll") || l.contains("/thumbs/") || l.ends_with("/sample.mp4")
}

/// Decode VOE's obfuscated JSON player config and return its stream URL.
pub fn voe_config(html: &str) -> Option<String> {
    let blob = JSON_CONFIG.captures(html)?.get(1)?.as_str().trim().to_string();
    let json = voe_decode(&blob)?;
    let v: serde_json::Value = serde_json::from_str(&json).ok()?;
    let item = v.as_array().and_then(|a| a.first()).unwrap_or(&v);
    let from_list = |key: &str| {
        item.get(key)?
            .as_array()?
            .iter()
            .filter_map(|x| x.get("file").and_then(|f| f.as_str()))
            .find(|f| f.starts_with("http"))
            .map(str::to_string)
    };
    from_list("sources")
        .or_else(|| ["source", "file", "url", "src"].iter().find_map(|k| item.get(*k)?.as_str().filter(|f| f.starts_with("http")).map(str::to_string)))
        .or_else(|| from_list("fallback"))
}

pub fn voe_decode(blob: &str) -> Option<String> {
    let mut s: String = blob
        .chars()
        .map(|c| match c {
            'a'..='z' => (((c as u8 - b'a' + 13) % 26) + b'a') as char,
            'A'..='Z' => (((c as u8 - b'A' + 13) % 26) + b'A') as char,
            _ => c,
        })
        .collect();
    for marker in ["@$", "^^", "~@", "%?", "*~", "!!", "#&"] {
        s = s.replace(marker, "_");
    }
    s.retain(|c| c != '_');
    let step1 = b64_decode(&s)?;
    let shifted: String = step1.iter().map(|b| char::from(b.wrapping_sub(3))).collect();
    let reversed: String = shifted.chars().rev().collect();
    let step2 = b64_decode(&reversed)?;
    String::from_utf8(step2).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;

    #[test]
    fn voe_roundtrip() {
        // Build a blob the way VOE does and make sure we invert every layer.
        let json = r#"{"source":"https://cdn.voe.example/master.m3u8","fallback":[{"file":"https://cdn/x.mp4"}]}"#;
        let b = base64::engine::general_purpose::STANDARD;
        let inner = b.encode(json);
        let reversed: String = inner.chars().rev().collect();
        let shifted: Vec<u8> = reversed.bytes().map(|x| x + 3).collect();
        let outer = b.encode(shifted);
        let marked = format!("{}@${}", &outer[..4], &outer[4..]);
        let rot: String = marked
            .chars()
            .map(|c| match c {
                'a'..='z' => (((c as u8 - b'a' + 13) % 26) + b'a') as char,
                'A'..='Z' => (((c as u8 - b'A' + 13) % 26) + b'A') as char,
                _ => c,
            })
            .collect();
        let html = format!(r#"<script type="application/json">{rot}</script>"#);
        assert_eq!(voe_config(&html).as_deref(), Some("https://cdn.voe.example/master.m3u8"));
    }

    #[test]
    fn picks_master_and_skips_decoys() {
        let html = r#"<script>var a="https://s1.fsvid.lol/troll/master.m3u8"; var b="https://cdn.x/v/index-v1.m3u8"; var c="https://u14.vidzy.cc/hls2/08/x/master.m3u8?t=1";</script>"#;
        assert_eq!(best_media_url(html).as_deref(), Some("https://u14.vidzy.cc/hls2/08/x/master.m3u8?t=1"));
    }

    #[test]
    fn packed_player() {
        let html = r"<script>eval(function(p,a,c,k,e,d){while(c--)if(k[c])p=p.replace(new RegExp('\\b'+c.toString(a)+'\\b','g'),k[c]);return p}('0({1:\'2://3.4/5.6\'})',7,7,'setup|file|https|cdn|example|master|m3u8'.split('|'),0,{}))</script>";
        assert_eq!(best_media_url(html).as_deref(), Some("https://cdn.example/master.m3u8"));
    }

    #[test]
    fn misc() {
        assert_eq!(host_label("ansembed.net"), "Ansembed");
    }
}
