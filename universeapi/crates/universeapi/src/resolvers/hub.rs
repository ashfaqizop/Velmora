//! The "Hub" hoster family used by HDHub / 4KHDHub:
//! HubCDN, HubDrive, HubCloud, HBLinks, HBPlay, GreenMotors.

use std::sync::LazyLock;
use std::time::Duration;

use regex::Regex;
use scraper::{Html, Selector};

use super::{presigned_expiry, HostKind, ResolveCtx};
use crate::error::{Error, Result};
use crate::model::{HostedLink, Source, StreamFormat};
use crate::release::parse_size;
use crate::util::{absolutize, b64_decode_str, host_of, origin_of, text_of};

static A_HREF: LazyLock<Selector> = LazyLock::new(|| Selector::parse("a[href]").expect("selector"));

/// `hubcdn.*/file/{id}` → page JS `var reurl = ".../?r=<b64>"`, where the
/// base64 decodes to `.../dl/?link=<direct url>`. No timer, no browser.
pub async fn hubcdn(ctx: &ResolveCtx<'_>, link: &HostedLink) -> Result<Vec<Source>> {
    static REURL: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"var\s+reurl\s*=\s*["']([^"']+)["']"#).expect("regex"));
    let resp = ctx.http.get(&link.url, &[]).await?;
    let final_url = resp.url().to_string();
    let html = resp.text().await.map_err(|e| Error::Network { url: link.url.clone(), source: e })?;

    let mut candidates = Vec::new();
    if let Some(c) = REURL.captures(&html) {
        candidates.push(c[1].to_string());
    }
    candidates.push(final_url);

    for cand in candidates {
        if let Some(direct) = hubcdn_extract(&cand) {
            let mut s = Source::new(ctx.provider, "HubCDN", &link.label, direct);
            s.expires_at = presigned_expiry(&s.url);
            return Ok(vec![s]);
        }
    }
    Err(Error::parse("hubcdn page", "reurl / link= parameter not found"))
}

fn hubcdn_extract(url: &str) -> Option<String> {
    // Either `...?r=<b64>` (ad-gateway) or already `.../dl/?link=<url>`.
    if let Some(idx) = url.find("link=") {
        let l = &url[idx + 5..];
        if l.starts_with("http") {
            return Some(l.to_string());
        }
    }
    let parsed = url::Url::parse(url).ok()?;
    let r = parsed.query_pairs().find(|(k, _)| k == "r")?.1.into_owned();
    let decoded = b64_decode_str(&r)?;
    let l = decoded.split_once("link=")?.1;
    l.starts_with("http").then(|| l.to_string())
}

/// `hubdrive.*/file/{id}` → `POST /ajax.php?ajax=direct-download` (needs the
/// `xla=s4t` cookie the page normally sets). Returns R2 URL + filename + size.
pub async fn hubdrive(ctx: &ResolveCtx<'_>, link: &HostedLink) -> Result<Vec<Source>> {
    let file_id = link.url.trim_end_matches('/').rsplit('/').next().unwrap_or_default();
    let origin = origin_of(&link.url);
    let api = format!("{origin}ajax.php?ajax=direct-download");
    let body = ctx
        .http
        .post_form_text(
            &api,
            &[("id", file_id)],
            &[
                ("Cookie", "xla=s4t"),
                ("X-Requested-With", "XMLHttpRequest"),
                ("Referer", &link.url),
                ("Origin", origin.trim_end_matches('/')),
            ],
        )
        .await?;
    let v: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| Error::parse("hubdrive json", e.to_string()))?;
    let code_ok = match &v["code"] {
        serde_json::Value::String(s) => s == "200",
        serde_json::Value::Number(n) => n.as_u64() == Some(200),
        _ => false,
    };
    if !code_ok {
        return Err(Error::parse("hubdrive", format!("api refused: {}", crate::net::prefix(&body, 200))));
    }
    let d = &v["data"];
    let url = d["gd"].as_str().filter(|u| u.starts_with("http")).ok_or_else(|| Error::parse("hubdrive", "no gd url"))?;
    let mut s = Source::new(ctx.provider, "HubDrive", &link.label, url);
    s.filename = d["n"].as_str().map(str::to_string);
    s.size_bytes = match &d["s"] {
        serde_json::Value::String(x) => x.parse().ok(),
        serde_json::Value::Number(n) => n.as_u64(),
        _ => None,
    };
    s.retag();
    Ok(vec![s])
}

/// `hubcloud.*/drive/{id}` → token page (`.../hubcloud.php?...`) → server list.
/// Only real file servers are kept; site chrome, VPN ads and the `gpdl`
/// "10Gbps" ad-redirect are filtered (the verifier drops any survivors).
pub async fn hubcloud(ctx: &ResolveCtx<'_>, link: &HostedLink) -> Result<Vec<Source>> {
    let page = ctx.http.get_text(&link.url, &[("Referer", &origin_of(&link.url))]).await?;
    let gen_url = {
        let doc = Html::parse_document(&page);
        let found = doc
            .select(&A_HREF)
            .filter_map(|a| a.value().attr("href"))
            .find(|h| h.contains("hubcloud.php"))
            .map(|h| absolutize(&link.url, h));
        found.or_else(|| {
            // Some variants inject the URL from JS: `var url = '...hubcloud.php?...'`
            static JS_URL: LazyLock<Regex> =
                LazyLock::new(|| Regex::new(r#"['"](https?://[^'"]+hubcloud\.php\?[^'"]+)['"]"#).expect("regex"));
            JS_URL.captures(&page).map(|c| c[1].to_string())
        })
    }
    .ok_or_else(|| Error::parse("hubcloud page", "token link not found"))?;

    let html = ctx.http.get_text(&gen_url, &[("Referer", &link.url)]).await?;
    let doc = Html::parse_document(&html);

    let filename = Selector::parse(".card-header")
        .ok()
        .and_then(|sel| doc.select(&sel).next().map(text_of))
        .filter(|t| t.contains('.'));
    let size = {
        static SIZE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"(?i)file\s*size[^0-9]{0,40}([\d.,]+\s*[gmk]i?b)").expect("regex"));
        SIZE.captures(&html).and_then(|c| parse_size(&c[1]))
    };

    static KEEP_LABEL: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?i)download|server|fsl|pixel|buzz|watch").expect("regex"));
    const DENY_HOSTS: &[&str] = &["hdhub4u", "one.one.one.one", "t.me", "telegram", "google.", "gamerxyt", "tinyurl"];

    let mut out = Vec::new();
    for a in doc.select(&A_HREF) {
        let href = a.value().attr("href").unwrap_or_default();
        if !href.starts_with("http") {
            continue;
        }
        let label = text_of(a);
        let host = host_of(href);
        if DENY_HOSTS.iter().any(|d| host.contains(d)) || host.starts_with("gpdl.") || !KEEP_LABEL.is_match(&label) {
            continue;
        }
        if host.contains("hubcloud") && (href.contains("/drive/") || href.contains("/tg/")) {
            continue;
        }

        let (server, url) = if host.contains("r2.cloudflarestorage.com") || host.ends_with(".r2.dev") {
            ("HubCloud FSL", href.to_string())
        } else if host.contains("pixeldrain") {
            ("PixelDrain", pixeldrain_direct(href))
        } else if HostKind::detect(href) == Some(HostKind::HbPlay) {
            match hbplay_decode(href) {
                Some(u) => ("HubCloud FSL", u),
                None => continue,
            }
        } else if host.contains("fuckingfast") || host.contains("buzz") {
            ("BuzzCDN", href.to_string())
        } else {
            ("HubCloud", href.to_string())
        };

        if out.iter().any(|s: &Source| s.url == url) {
            continue;
        }
        let mut s = Source::new(ctx.provider, server, &link.label, url);
        s.label = format!("{} · {}", link.label, label);
        s.filename.clone_from(&filename);
        s.size_bytes = size;
        s.expires_at = presigned_expiry(&s.url);
        s.retag();
        if s.format == StreamFormat::Unknown {
            if let Some(f) = &filename {
                s.format = StreamFormat::from_url_or_name(f);
            }
        }
        out.push(s);
    }
    if out.is_empty() {
        return Err(Error::parse("hubcloud", "no download servers on token page"));
    }
    Ok(out)
}

/// `hblinks.*/archives/{id}` — aggregator page listing Hub-family links.
pub async fn hblinks(ctx: &ResolveCtx<'_>, link: &HostedLink) -> Result<Vec<HostedLink>> {
    let html = ctx.http.get_text(&link.url, &[("Referer", &origin_of(&link.url))]).await?;
    Ok(hblinks_parse(&html, link, ctx.aliases))
}

fn hblinks_parse(html: &str, parent: &HostedLink, aliases: &std::collections::BTreeMap<String, String>) -> Vec<HostedLink> {
    let doc = Html::parse_document(html);
    let mut out: Vec<HostedLink> = Vec::new();
    for a in doc.select(&A_HREF) {
        let href = a.value().attr("href").unwrap_or_default();
        let Some(kind) = HostKind::detect_with(href, aliases) else { continue };
        if !matches!(kind, HostKind::HubCloud | HostKind::HubDrive | HostKind::HubCdn) || out.iter().any(|l| l.url == href) {
            continue;
        }
        out.push(HostedLink {
            label: parent.label.clone(),
            url: href.to_string(),
            host: kind.label().to_string(),
            tags: parent.tags.clone(),
            episode: parent.episode,
            season: parent.season,
            is_pack: parent.is_pack,
            is_stream_page: false,
        });
    }
    // Prefer the instant hosts: HubDrive/HubCDN resolve in one request.
    out.sort_by_key(|l| match HostKind::detect_with(&l.url, aliases) {
        Some(HostKind::HubDrive) => 0,
        Some(HostKind::HubCdn) => 1,
        _ => 2,
    });
    out
}

/// GreenMotors gate (`greenmotors.*/?id=...`). The "AES + 10 s timer" page is
/// a front: the first response stores the target in localStorage as
/// `base64(base64(rot13(base64(json))))`, and the JSON's `o` field is the
/// base64 of the real HubCloud/HubDrive/HBLinks URL. The timer (`w`) is
/// client-side only, so plain HTTP is enough. The WebView solver remains a
/// fallback in case the scheme changes.
pub async fn greenmotors(ctx: &ResolveCtx<'_>, link: &HostedLink) -> Result<Vec<HostedLink>> {
    let html = ctx.http.get_text(&link.url, &[("Referer", &origin_of(&link.url))]).await;
    if let Some(target) = html.as_deref().ok().and_then(greenmotors_decode) {
        return Ok(vec![child_link(link, &target, ctx.aliases)]);
    }
    let web = ctx.web.ok_or_else(|| Error::parse("greenmotors", "gate page has no decodable target"))?;
    let target = web
        .click_through(&link.url, "#verify_btn", Duration::from_secs(13), "hblinks")
        .await?
        .ok_or_else(|| Error::parse("greenmotors", "timer page did not open an hblinks tab"))?;
    let html = ctx.http.get_text(&target, &[]).await?;
    Ok(hblinks_parse(&html, link, ctx.aliases))
}

/// Decode the target URL from a GreenMotors gate page.
pub fn greenmotors_decode(html: &str) -> Option<String> {
    static STORED: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"\bs\(\s*['"]o['"]\s*,\s*['"]([A-Za-z0-9+/=_-]+)['"]"#).expect("regex"));
    let blob = STORED.captures(html)?.get(1)?.as_str().to_string();
    let layer2 = b64_decode_str(&b64_decode_str(&blob)?)?;
    let json_text = b64_decode_str(&rot13(&layer2))?;
    let v: serde_json::Value = serde_json::from_str(&json_text).ok()?;
    let target = b64_decode_str(v.get("o")?.as_str()?)?;
    target.starts_with("http").then_some(target)
}

fn rot13(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' => (((c as u8 - b'a' + 13) % 26) + b'a') as char,
            'A'..='Z' => (((c as u8 - b'A' + 13) % 26) + b'A') as char,
            _ => c,
        })
        .collect()
}

fn child_link(parent: &HostedLink, url: &str, aliases: &std::collections::BTreeMap<String, String>) -> HostedLink {
    HostedLink {
        label: parent.label.clone(),
        url: url.to_string(),
        host: HostKind::detect_with(url, aliases).map(|k| k.label().to_string()).unwrap_or_else(|| host_of(url)),
        tags: parent.tags.clone(),
        episode: parent.episode,
        season: parent.season,
        is_pack: parent.is_pack,
        is_stream_page: false,
    }
}

/// `hbplay.pages.dev/?u=<b64 url>` — the "Watch Online" button just wraps the
/// direct R2 URL in base64.
pub fn hbplay(ctx: &ResolveCtx<'_>, link: &HostedLink) -> Result<Vec<Source>> {
    let url = hbplay_decode(&link.url).ok_or_else(|| Error::parse("hbplay", "no u= parameter"))?;
    let mut s = Source::new(ctx.provider, "HubCloud FSL", &link.label, url);
    s.expires_at = presigned_expiry(&s.url);
    Ok(vec![s])
}

fn hbplay_decode(url: &str) -> Option<String> {
    let parsed = url::Url::parse(url).ok()?;
    let u = parsed.query_pairs().find(|(k, _)| k == "u")?.1.into_owned();
    b64_decode_str(&u).filter(|d| d.starts_with("http"))
}

pub(super) fn pixeldrain_direct(url: &str) -> String {
    // pixeldrain.com/u/{id} → API download endpoint (no captcha page)
    match url.split("/u/").nth(1) {
        Some(id) => {
            let id = id.split(['?', '/', '#']).next().unwrap_or(id);
            let host = host_of(url);
            format!("https://{host}/api/file/{id}?download")
        }
        None => url.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::release::ReleaseTags;

    #[test]
    fn hubcdn_r_param() {
        // r = base64("https://hubcdn.club/dl/?link=https://pub-x.r2.dev/abc")
        let r = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            "https://hubcdn.club/dl/?link=https://pub-x.r2.dev/abc",
        );
        assert_eq!(
            hubcdn_extract(&format!("https://inventoryidea.com/?r={r}")).as_deref(),
            Some("https://pub-x.r2.dev/abc")
        );
        assert_eq!(
            hubcdn_extract("https://hubcdn.club/dl/?link=https://video-downloads.googleusercontent.com/x=y").as_deref(),
            Some("https://video-downloads.googleusercontent.com/x=y"),
            "link values containing '=' are kept intact"
        );
    }

    #[test]
    fn hbplay_unwrap() {
        let u = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, "https://abc.r2.cloudflarestorage.com/hub/f.mkv");
        assert_eq!(
            hbplay_decode(&format!("https://hbplay.pages.dev/?u={u}")).as_deref(),
            Some("https://abc.r2.cloudflarestorage.com/hub/f.mkv")
        );
    }

    #[test]
    fn greenmotors_gate_decodes_without_browser() {
        // Real gate page captured 2026-10-30 (4KHDHub "Dune" link).
        let html = r#"<script>function s(e,t,i){}s('o','Y214WE0xWjNZbXRhVUdwMmIxQldObFo2ZFRCeFZVOXRRbmxxYVV0UU9XRndla2w1YjNveGFYRlVPV3h3YkRWM1RERnFhVzVVT1dkTlNtdDFiM3BGZVhCNWFtbFdkbXAyYjJ4V05sWjZVMVpJZDA5M1JsSXdNa2RWZURWdk1rVkxSbnBqZGtWdGVHdEtlRm94Y0ZSYWJVaExUVzVHVW1OcVRWUXhTWEY1WW14Q1ZXTkJTVkpKTTBkTFkzZGFVMk5XY0ZRNVFVVXlhMmxXWVRBOQ==',180*1000);</script>"#;
        assert_eq!(greenmotors_decode(html).as_deref(), Some("https://hubcloud.ist/drive/go3110374dzh0ih"));
        assert_eq!(greenmotors_decode("<html>no blob</html>"), None);
        assert_eq!(rot13("Uryyb"), "Hello");
    }

    #[test]
    fn pixeldrain() {
        assert_eq!(pixeldrain_direct("https://pixeldrain.com/u/AbC123"), "https://pixeldrain.com/api/file/AbC123?download");
    }

    #[test]
    fn hblinks_prefers_instant_hosts() {
        let parent = HostedLink {
            label: "1080p".into(),
            url: "https://hblinks.lol/archives/1".into(),
            host: "HBLinks".into(),
            tags: ReleaseTags::default(),
            episode: Some(2),
            season: Some(1),
            is_pack: false,
            is_stream_page: false,
        };
        let html = r#"<a href="https://hubcloud.ist/drive/a">HubCloud</a><a href="https://t.me/x">tg</a>
                      <a href="https://hubdrive.bar/file/9">HubDrive</a><a href="https://hubdrive.bar/file/9">dup</a>"#;
        let links = hblinks_parse(html, &parent, &Default::default());
        assert_eq!(links.len(), 2);
        assert!(links[0].url.contains("hubdrive"));
        assert_eq!(links[0].episode, Some(2));
    }
}
