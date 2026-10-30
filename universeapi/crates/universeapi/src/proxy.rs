//! Media proxy core for the in-app player.
//!
//! A WebView `<video>`/hls.js player can't send the Referer, Cookie or
//! User-Agent headers most sources require. The app registers a [`Source`]
//! and gets back a local path; its custom URI scheme (Tauri) or a loopback
//! server hands every request to [`MediaProxy::handle`].
//!
//! Proxied paths keep the upstream URL's shape:
//! `/p/{token}/{scheme}/{host[:port]}/{path}?{query}`, so relative segment
//! URIs inside HLS/DASH manifests resolve correctly with no rewriting. Only
//! absolute URLs in manifests (other hosts, keys, variants) are rewritten.
//! Torrent files are served from `/t/{token}` with byte-range seeking.

use std::collections::{BTreeMap, HashMap};
use std::sync::{LazyLock, RwLock};

use futures::stream::BoxStream;
use futures::StreamExt;
use regex::Regex;

use crate::error::{Error, Result};
use crate::model::Source;
use crate::net::Http;

pub enum ProxyBody {
    Bytes(Vec<u8>),
    Stream(BoxStream<'static, std::io::Result<Vec<u8>>>),
}

pub struct ProxyResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: ProxyBody,
}

enum Entry {
    Http { headers: BTreeMap<String, String> },
    #[cfg(feature = "torrent")]
    Torrent { job: std::sync::Arc<crate::torrent::TorrentJob> },
}

pub struct MediaProxy {
    http: Http,
    entries: RwLock<HashMap<String, Entry>>,
    #[cfg(feature = "torrent")]
    torrents: crate::download::TorrentSlot,
}

fn token() -> String {
    format!("{:016x}", fastrand::u64(..))
}

/// `https://cdn.x:8443/a/b.m3u8?t=1` → `/p/{token}/https/cdn.x:8443/a/b.m3u8?t=1`
pub fn proxify(token: &str, absolute: &str) -> Option<String> {
    let u = url::Url::parse(absolute).ok()?;
    if !matches!(u.scheme(), "http" | "https") {
        return None;
    }
    let host = match u.port() {
        Some(p) => format!("{}:{p}", u.host_str()?),
        None => u.host_str()?.to_string(),
    };
    let q = u.query().map(|q| format!("?{q}")).unwrap_or_default();
    Some(format!("/p/{token}/{}/{host}{}{q}", u.scheme(), u.path()))
}

/// Inverse of [`proxify`]: `(token, upstream url)`.
pub fn unproxify(path_and_query: &str) -> Option<(String, String)> {
    let rest = path_and_query.strip_prefix("/p/")?;
    let mut parts = rest.splitn(3, '/');
    let token = parts.next()?.to_string();
    let scheme = parts.next().filter(|s| *s == "http" || *s == "https")?;
    let tail = parts.next()?;
    Some((token, format!("{scheme}://{tail}")))
}

static URI_ATTR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"URI="([^"]+)""#).expect("regex"));
static ABS_URL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"https?://[^"'<>\s]+"#).expect("regex"));

/// Rewrite absolute URIs in an HLS playlist; relative ones already resolve under the proxy path.
pub fn rewrite_hls(text: &str, token: &str) -> String {
    let mut out = String::with_capacity(text.len() + 256);
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() {
            out.push('\n');
            continue;
        }
        if t.starts_with('#') {
            let replaced = URI_ATTR.replace_all(line, |c: &regex::Captures<'_>| match proxify(token, &c[1]) {
                Some(p) => format!("URI=\"{p}\""),
                None => c[0].to_string(),
            });
            out.push_str(&replaced);
        } else if t.starts_with("http://") || t.starts_with("https://") {
            out.push_str(&proxify(token, t).unwrap_or_else(|| t.to_string()));
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

/// Rewrite absolute URLs in a DASH manifest (BaseURL, SegmentTemplate...).
pub fn rewrite_dash(text: &str, token: &str) -> String {
    ABS_URL
        .replace_all(text, |c: &regex::Captures<'_>| {
            let raw = c[0].replace("&amp;", "&");
            // leave XML namespace / schema URIs alone
            if raw.contains("w3.org") || raw.contains("urn:") || raw.contains("dashif.org") || raw.contains("mpeg.org") {
                return c[0].to_string();
            }
            proxify(token, &raw).map(|p| p.replace('&', "&amp;")).unwrap_or_else(|| c[0].to_string())
        })
        .into_owned()
}

/// `bytes=START-END` → (start, inclusive end).
pub fn parse_range(h: &str, len: u64) -> Option<(u64, u64)> {
    let spec = h.trim().strip_prefix("bytes=")?.split(',').next()?.trim();
    let (a, b) = spec.split_once('-')?;
    let last = len.checked_sub(1)?;
    let (start, end) = match (a.trim(), b.trim()) {
        ("", suffix) => {
            let n: u64 = suffix.parse().ok()?;
            (len.saturating_sub(n), last)
        }
        (s, "") => (s.parse().ok()?, last),
        (s, e) => (s.parse().ok()?, e.parse::<u64>().ok()?.min(last)),
    };
    (start <= end).then_some((start, end))
}

impl MediaProxy {
    pub fn new(http: Http, #[cfg(feature = "torrent")] torrents: crate::download::TorrentSlot) -> Self {
        Self {
            http,
            entries: RwLock::new(HashMap::new()),
            #[cfg(feature = "torrent")]
            torrents,
        }
    }

    /// Register a source; returns the proxied path to give the player.
    pub fn register(&self, src: &Source) -> Result<String> {
        let t = token();
        let path = proxify(&t, &src.url).ok_or_else(|| Error::Unsupported(format!("cannot proxy {}", src.url)))?;
        if let Ok(mut e) = self.entries.write() {
            e.insert(t, Entry::Http { headers: src.headers.clone() });
        }
        Ok(path)
    }

    /// Register a torrent file for streaming while it downloads.
    #[cfg(feature = "torrent")]
    pub fn register_torrent(&self, job: std::sync::Arc<crate::torrent::TorrentJob>) -> String {
        let t = token();
        if let Ok(mut e) = self.entries.write() {
            e.insert(t.clone(), Entry::Torrent { job });
        }
        format!("/t/{t}")
    }

    pub fn unregister(&self, path: &str) {
        let tok = path.trim_start_matches("/p/").trim_start_matches("/t/").split('/').next().unwrap_or_default().to_string();
        if let Ok(mut e) = self.entries.write() {
            e.remove(&tok);
        }
    }

    /// Serve one request from the player.
    pub async fn handle(&self, path_and_query: &str, range: Option<&str>) -> Result<ProxyResponse> {
        #[cfg(feature = "torrent")]
        if let Some(tok) = path_and_query.strip_prefix("/t/") {
            return self.handle_torrent(tok.split(['/', '?']).next().unwrap_or(tok), range).await;
        }
        let (tok, upstream) = unproxify(path_and_query).ok_or_else(|| Error::NotFound(format!("bad proxy path {path_and_query}")))?;
        let headers = match self.entries.read().ok().and_then(|e| match e.get(&tok) {
            Some(Entry::Http { headers }) => Some(headers.clone()),
            _ => None,
        }) {
            Some(h) => h,
            None => return Err(Error::NotFound("expired proxy token".into())),
        };

        let mut rb = self.http.client().get(&upstream);
        for (k, v) in &headers {
            rb = rb.header(k.as_str(), v.as_str());
        }
        if let Some(r) = range {
            rb = rb.header("Range", r);
        }
        let resp = rb.send().await.map_err(|e| Error::Network { url: upstream.clone(), source: e })?;
        let status = resp.status().as_u16();
        let ctype = resp.headers().get("content-type").and_then(|v| v.to_str().ok()).unwrap_or("").to_ascii_lowercase();
        let path_lower = upstream.split('?').next().unwrap_or("").to_ascii_lowercase();

        let is_hls = ctype.contains("mpegurl") || path_lower.ends_with(".m3u8") || path_lower.ends_with("master.txt");
        let is_dash = ctype.contains("dash+xml") || path_lower.ends_with(".mpd");
        if (is_hls || is_dash) && (200..300).contains(&status) {
            let text = resp.text().await.map_err(|e| Error::Network { url: upstream.clone(), source: e })?;
            let (body, mime) = if is_dash || text.trim_start().starts_with("<?xml") || text.contains("<MPD") {
                (rewrite_dash(&text, &tok), "application/dash+xml")
            } else {
                (rewrite_hls(&text, &tok), "application/vnd.apple.mpegurl")
            };
            return Ok(ProxyResponse {
                status,
                headers: vec![("Content-Type".into(), mime.into()), ("Cache-Control".into(), "no-cache".into())],
                body: ProxyBody::Bytes(body.into_bytes()),
            });
        }

        let mut out_headers = vec![("Accept-Ranges".to_string(), "bytes".to_string())];
        for name in ["content-type", "content-length", "content-range"] {
            if let Some(v) = resp.headers().get(name).and_then(|v| v.to_str().ok()) {
                out_headers.push((name.to_string(), v.to_string()));
            }
        }
        let stream = resp.bytes_stream().map(|r| r.map(|b| b.to_vec()).map_err(std::io::Error::other)).boxed();
        Ok(ProxyResponse { status, headers: out_headers, body: ProxyBody::Stream(stream) })
    }

    #[cfg(feature = "torrent")]
    async fn handle_torrent(&self, tok: &str, range: Option<&str>) -> Result<ProxyResponse> {
        use tokio::io::{AsyncReadExt, AsyncSeekExt};
        let job = match self.entries.read().ok().and_then(|e| match e.get(tok) {
            Some(Entry::Torrent { job }) => Some(job.clone()),
            _ => None,
        }) {
            Some(x) => x,
            None => return Err(Error::NotFound("expired torrent token".into())),
        };
        let file = job.selected_file().cloned().ok_or_else(|| Error::Torrent("no file selected".into()))?;
        let len = file.len;
        let (start, end, status) = match range.and_then(|r| parse_range(r, len)) {
            Some((s, e)) => (s, e, 206),
            None => (0, len.saturating_sub(1), 200),
        };
        let te = self.torrents.get().ok_or_else(|| Error::Torrent("torrent session not started".into()))?;
        let mut reader = te.stream(&job).await?;
        reader.seek(std::io::SeekFrom::Start(start)).await.map_err(|e| Error::Torrent(e.to_string()))?;
        let remaining = end + 1 - start;
        let stream = futures::stream::unfold((reader, remaining), |(mut r, left)| async move {
            if left == 0 {
                return None;
            }
            let mut buf = vec![0u8; left.min(256 * 1024) as usize];
            match r.read(&mut buf).await {
                Ok(0) => None,
                Ok(n) => {
                    buf.truncate(n);
                    Some((Ok(buf), (r, left - n as u64)))
                }
                Err(e) => Some((Err(e), (r, 0))),
            }
        })
        .boxed();
        let mime = match crate::torrent::file_format(&file.name) {
            crate::model::StreamFormat::Mp4 => "video/mp4",
            crate::model::StreamFormat::Webm => "video/webm",
            _ => "video/x-matroska",
        };
        let mut headers = vec![
            ("Content-Type".to_string(), mime.to_string()),
            ("Accept-Ranges".to_string(), "bytes".to_string()),
            ("Content-Length".to_string(), (end + 1 - start).to_string()),
        ];
        if status == 206 {
            headers.push(("Content-Range".to_string(), format!("bytes {start}-{end}/{len}")));
        }
        Ok(ProxyResponse { status, headers, body: ProxyBody::Stream(stream) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxify_roundtrip() {
        let p = proxify("tok", "https://cdn.x:8443/hls/a/master.m3u8?t=1&s=2").unwrap();
        assert_eq!(p, "/p/tok/https/cdn.x:8443/hls/a/master.m3u8?t=1&s=2");
        assert_eq!(unproxify(&p), Some(("tok".into(), "https://cdn.x:8443/hls/a/master.m3u8?t=1&s=2".into())));
        assert_eq!(unproxify("/p/tok/ftp/x"), None);
        assert_eq!(proxify("t", "magnet:?xt=1"), None);
    }

    #[test]
    fn hls_rewrite_only_absolute() {
        let m3u8 = "#EXTM3U\n#EXT-X-KEY:METHOD=AES-128,URI=\"https://keys.x/k.bin\"\n#EXT-X-MEDIA:TYPE=AUDIO,URI=\"audio/index.m3u8\"\n#EXTINF:4,\nseg-1.ts\nhttps://other.cdn/seg-2.ts\n";
        let out = rewrite_hls(m3u8, "tok");
        assert!(out.contains("URI=\"/p/tok/https/keys.x/k.bin\""));
        assert!(out.contains("URI=\"audio/index.m3u8\""), "relative stays relative");
        assert!(out.contains("\nseg-1.ts\n"));
        assert!(out.contains("\n/p/tok/https/other.cdn/seg-2.ts\n"));
    }

    #[test]
    fn dash_rewrite() {
        let mpd = r#"<MPD xmlns="urn:mpeg:dash:schema:mpd:2011"><BaseURL>https://cdn.x/dash/a/</BaseURL><SegmentTemplate media="https://cdn.x/s/$Number$.m4s?a=1&amp;b=2"/></MPD>"#;
        let out = rewrite_dash(mpd, "tok");
        assert!(out.contains("<BaseURL>/p/tok/https/cdn.x/dash/a/</BaseURL>"));
        assert!(out.contains("media=\"/p/tok/https/cdn.x/s/$Number$.m4s?a=1&amp;b=2\""));
        assert!(out.contains("urn:mpeg:dash:schema:mpd:2011"));
    }

    #[test]
    fn ranges() {
        assert_eq!(parse_range("bytes=0-", 100), Some((0, 99)));
        assert_eq!(parse_range("bytes=10-19", 100), Some((10, 19)));
        assert_eq!(parse_range("bytes=-10", 100), Some((90, 99)));
        assert_eq!(parse_range("bytes=50-500", 100), Some((50, 99)));
        assert_eq!(parse_range("bytes=200-", 100), None);
    }
}
