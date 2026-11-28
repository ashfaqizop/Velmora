//! Downloading an HLS stream without ffmpeg.
//!
//! Android has no ffmpeg sidecar, and desktop can lose it. Most HLS hosts
//! serve one variant with audio muxed into MPEG-TS (or fMP4) segments, and
//! those segments laid end to end are a playable file. This reads the master
//! playlist, takes the best variant, and joins its segments in order,
//! decrypting AES-128 ones on the way. What it can't do it says plainly:
//! live streams, audio kept in its own playlist, SAMPLE-AES, byte ranges.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockDecryptMut, KeyIvInit};
use futures::StreamExt;
use reqwest::Client;
use tokio::io::AsyncWriteExt;
use url::Url;

use crate::error::{Error, Result};

type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

/// Segments fetched at once; written in order.
const PARALLEL: usize = 4;
const TRIES: usize = 3;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Key {
    pub url: String,
    pub iv: Option<[u8; 16]>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Segment {
    pub url: String,
    pub key: Option<Key>,
    pub seq: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Plan {
    pub segments: Vec<Segment>,
    /// fMP4 init segment (`#EXT-X-MAP`); its presence makes the file an MP4.
    pub init: Option<String>,
}

impl Plan {
    /// The container the joined segments make.
    pub fn extension(&self) -> &'static str {
        if self.init.is_some() {
            "mp4"
        } else {
            "ts"
        }
    }
}

fn bad(detail: impl Into<String>) -> Error {
    Error::Parse { what: "HLS", detail: detail.into() }
}

fn unsupported(why: &str) -> Error {
    Error::Unsupported(format!("this stream can't be saved: {why}. It still plays as a stream"))
}

/// `KEY=VALUE,KEY="quoted, value"` into a map.
pub(crate) fn attrs(s: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let mut rest = s;
    while !rest.is_empty() {
        let Some(eq) = rest.find('=') else { break };
        let key = rest[..eq].trim().to_ascii_uppercase();
        rest = &rest[eq + 1..];
        let value;
        if let Some(stripped) = rest.strip_prefix('"') {
            let end = stripped.find('"').unwrap_or(stripped.len());
            value = stripped[..end].to_string();
            rest = stripped.get(end + 1..).unwrap_or("");
            rest = rest.strip_prefix(',').unwrap_or(rest);
        } else {
            let end = rest.find(',').unwrap_or(rest.len());
            value = rest[..end].trim().to_string();
            rest = rest.get(end + 1..).unwrap_or("");
        }
        out.insert(key, value);
    }
    out
}

fn join(base: &Url, uri: &str) -> Result<String> {
    base.join(uri.trim()).map(|u| u.to_string()).map_err(|e| bad(format!("HLS link {uri:?}: {e}")))
}

fn parse_iv(s: &str) -> Option<[u8; 16]> {
    let hex = s.trim().trim_start_matches("0x").trim_start_matches("0X");
    if hex.len() != 32 {
        return None;
    }
    let mut out = [0u8; 16];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(out)
}

/// From a master playlist, the best variant's URL. `None` when `text` is
/// already a media playlist.
pub(crate) fn pick_variant(text: &str, base: &Url) -> Result<Option<String>> {
    let mut audio_groups: HashMap<String, bool> = HashMap::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("#EXT-X-MEDIA:") {
            let a = attrs(rest);
            if a.get("TYPE").map(String::as_str) == Some("AUDIO") {
                let has_uri = a.contains_key("URI");
                let group = a.get("GROUP-ID").cloned().unwrap_or_default();
                let e = audio_groups.entry(group).or_insert(false);
                *e |= has_uri;
            }
        }
    }
    let mut best: Option<(u64, String, Option<String>)> = None;
    let mut lines = text.lines().peekable();
    while let Some(line) = lines.next() {
        let Some(rest) = line.strip_prefix("#EXT-X-STREAM-INF:") else { continue };
        let a = attrs(rest);
        let bandwidth = a.get("AVERAGE-BANDWIDTH").or_else(|| a.get("BANDWIDTH")).and_then(|b| b.parse().ok()).unwrap_or(0);
        let uri = loop {
            match lines.next() {
                Some(l) if l.trim().is_empty() || l.starts_with('#') => continue,
                Some(l) => break Some(l.trim().to_string()),
                None => break None,
            }
        };
        let Some(uri) = uri else { break };
        if best.as_ref().is_none_or(|(b, _, _)| bandwidth > *b) {
            best = Some((bandwidth, uri, a.get("AUDIO").cloned()));
        }
    }
    let Some((_, uri, audio)) = best else { return Ok(None) };
    if audio.is_some_and(|g| audio_groups.get(&g).copied().unwrap_or(false)) {
        return Err(unsupported("its audio comes in a separate track"));
    }
    Ok(Some(join(base, &uri)?))
}

/// The segments of a media playlist, with their keys.
pub(crate) fn parse_media(text: &str, base: &Url) -> Result<Plan> {
    let mut seq: u64 = 0;
    let mut key: Option<Key> = None;
    let mut init = None;
    let mut segments = Vec::new();
    let mut ended = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(v) = line.strip_prefix("#EXT-X-MEDIA-SEQUENCE:") {
            seq = v.trim().parse().unwrap_or(0);
        } else if let Some(v) = line.strip_prefix("#EXT-X-KEY:") {
            let a = attrs(v);
            match a.get("METHOD").map(String::as_str) {
                Some("NONE") | None => key = None,
                Some("AES-128") => {
                    let uri = a.get("URI").ok_or_else(|| bad("HLS key without a URI"))?;
                    key = Some(Key { url: join(base, uri)?, iv: a.get("IV").and_then(|iv| parse_iv(iv)) });
                }
                Some(_) => return Err(unsupported("it uses SAMPLE-AES encryption")),
            }
        } else if let Some(v) = line.strip_prefix("#EXT-X-MAP:") {
            let a = attrs(v);
            if a.contains_key("BYTERANGE") {
                return Err(unsupported("it uses byte ranges"));
            }
            if let Some(uri) = a.get("URI") {
                init = Some(join(base, uri)?);
            }
        } else if line.starts_with("#EXT-X-BYTERANGE") {
            return Err(unsupported("it uses byte ranges"));
        } else if line.starts_with("#EXT-X-ENDLIST") {
            ended = true;
        } else if !line.starts_with('#') {
            segments.push(Segment { url: join(base, line)?, key: key.clone(), seq });
            seq += 1;
        }
    }
    if !ended {
        return Err(unsupported("it's live"));
    }
    if segments.is_empty() {
        return Err(bad("the HLS playlist has no segments"));
    }
    Ok(Plan { segments, init })
}

async fn get(http: &Client, url: &str, headers: &BTreeMap<String, String>) -> Result<Vec<u8>> {
    let mut last = None;
    for attempt in 0..TRIES {
        let mut rb = http.get(url).timeout(std::time::Duration::from_secs(90));
        for (k, v) in headers {
            rb = rb.header(k.as_str(), v.as_str());
        }
        match rb.send().await {
            Ok(resp) if resp.status().is_success() => match resp.bytes().await {
                Ok(b) => return Ok(b.to_vec()),
                Err(e) => last = Some(Error::Network { url: url.to_string(), source: e }),
            },
            Ok(resp) => {
                let status = resp.status().as_u16();
                last = Some(Error::Http { status, url: url.to_string() });
                // A 4xx won't get better on a retry.
                if (400..500).contains(&status) && status != 429 {
                    break;
                }
            }
            Err(e) => last = Some(Error::Network { url: url.to_string(), source: e }),
        }
        tokio::time::sleep(std::time::Duration::from_millis(600 * (attempt as u64 + 1))).await;
    }
    Err(last.unwrap_or_else(|| Error::Transport(format!("couldn't fetch {url}"))))
}

/// Read the master (if any) and media playlists at `url`.
pub(crate) async fn plan(http: &Client, url: &str, headers: &BTreeMap<String, String>) -> Result<Plan> {
    let base = Url::parse(url).map_err(|e| bad(format!("HLS URL: {e}")))?;
    let text = String::from_utf8_lossy(&get(http, url, headers).await?).into_owned();
    if !text.trim_start().starts_with("#EXTM3U") {
        return Err(bad("that isn't an HLS playlist"));
    }
    match pick_variant(&text, &base)? {
        Some(variant) => {
            let vbase = Url::parse(&variant).map_err(|e| bad(format!("HLS variant URL: {e}")))?;
            let media = String::from_utf8_lossy(&get(http, &variant, headers).await?).into_owned();
            parse_media(&media, &vbase)
        }
        None => parse_media(&text, &base),
    }
}

fn decrypt(key: &[u8; 16], iv: &[u8; 16], data: &mut Vec<u8>) -> Result<()> {
    let n = Aes128CbcDec::new(key.into(), iv.into())
        .decrypt_padded_mut::<Pkcs7>(data)
        .map_err(|_| bad("an HLS segment didn't decrypt"))?
        .len();
    data.truncate(n);
    Ok(())
}

/// What a resumed download picks up from: segments written and bytes on disk.
fn read_mark(path: &Path) -> Option<(usize, u64)> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut it = text.split_whitespace();
    Some((it.next()?.parse().ok()?, it.next()?.parse().ok()?))
}

/// Join the plan's segments into `out` (written as `out.part`, with a
/// `.part.idx` mark so a pause or a dropped connection resumes where it
/// stopped). `progress(bytes, segments_done)` is called after each segment;
/// `interrupted()` is checked between them.
pub(crate) async fn download(
    http: &Client,
    headers: &BTreeMap<String, String>,
    plan: &Plan,
    out: &Path,
    mut progress: impl FnMut(u64, usize),
    interrupted: impl Fn() -> bool,
) -> Result<PathBuf> {
    let part = PathBuf::from(format!("{}.part", out.display()));
    let mark = PathBuf::from(format!("{}.part.idx", out.display()));
    let (mut done, mut written) = match (read_mark(&mark), tokio::fs::metadata(&part).await) {
        (Some((n, len)), Ok(meta)) if meta.len() >= len && n <= plan.segments.len() => (n, len),
        _ => (0, 0),
    };

    let mut file = tokio::fs::OpenOptions::new().create(true).write(true).truncate(false).open(&part).await.map_err(fs)?;
    file.set_len(written).await.map_err(fs)?;
    let mut file = tokio::io::BufWriter::new(file);
    {
        use tokio::io::AsyncSeekExt;
        file.seek(std::io::SeekFrom::Start(written)).await.map_err(fs)?;
    }
    if written == 0 {
        if let Some(init) = &plan.init {
            let bytes = get(http, init, headers).await?;
            file.write_all(&bytes).await.map_err(fs)?;
            written += bytes.len() as u64;
        }
    }

    let mut keys: HashMap<String, [u8; 16]> = HashMap::new();
    for seg in &plan.segments[done..] {
        if let Some(k) = &seg.key {
            if !keys.contains_key(&k.url) {
                let raw = get(http, &k.url, headers).await?;
                let key: [u8; 16] = raw.get(..16).and_then(|b| b.try_into().ok()).ok_or_else(|| bad("an HLS key isn't 16 bytes"))?;
                keys.insert(k.url.clone(), key);
            }
        }
    }

    let mut fetched = futures::stream::iter(plan.segments[done..].iter().cloned())
        .map(|seg| {
            let keys = &keys;
            async move {
                let mut data = get(http, &seg.url, headers).await?;
                if let Some(k) = &seg.key {
                    let iv = k.iv.unwrap_or_else(|| {
                        let mut iv = [0u8; 16];
                        iv[8..].copy_from_slice(&seg.seq.to_be_bytes());
                        iv
                    });
                    decrypt(&keys[&k.url], &iv, &mut data)?;
                }
                Ok::<_, Error>(data)
            }
        })
        .buffered(PARALLEL);

    while let Some(next) = fetched.next().await {
        if interrupted() {
            file.flush().await.map_err(fs)?;
            return Err(Error::Transport("download interrupted".into()));
        }
        let data = next?;
        file.write_all(&data).await.map_err(fs)?;
        written += data.len() as u64;
        done += 1;
        // The mark only moves once the bytes are on disk.
        if done % 8 == 0 || done == plan.segments.len() {
            file.flush().await.map_err(fs)?;
            let _ = tokio::fs::write(&mark, format!("{done} {written}")).await;
        }
        progress(written, done);
    }
    file.flush().await.map_err(fs)?;
    drop(file);
    tokio::fs::rename(&part, out).await.map_err(fs)?;
    let _ = tokio::fs::remove_file(&mark).await;
    Ok(out.to_path_buf())
}

fn fs(e: std::io::Error) -> Error {
    Error::Transport(format!("file system: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Url {
        Url::parse("https://cdn.example/show/master.m3u8").unwrap()
    }

    #[test]
    fn attributes_with_quotes() {
        let a = attrs(r#"METHOD=AES-128,URI="https://k.x/key?a=1,b=2",IV=0x000102030405060708090A0B0C0D0E0F"#);
        assert_eq!(a["METHOD"], "AES-128");
        assert_eq!(a["URI"], "https://k.x/key?a=1,b=2");
        assert_eq!(parse_iv(&a["IV"]).unwrap()[15], 0x0F);
    }

    #[test]
    fn best_variant_wins() {
        let m = "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=800000,RESOLUTION=640x360\nlow/index.m3u8\n#EXT-X-STREAM-INF:BANDWIDTH=5000000,RESOLUTION=1920x1080\nhigh/index.m3u8\n";
        assert_eq!(pick_variant(m, &base()).unwrap().as_deref(), Some("https://cdn.example/show/high/index.m3u8"));
    }

    #[test]
    fn separate_audio_is_refused() {
        let m = "#EXTM3U\n#EXT-X-MEDIA:TYPE=AUDIO,GROUP-ID=\"aud\",URI=\"audio.m3u8\"\n#EXT-X-STREAM-INF:BANDWIDTH=1,AUDIO=\"aud\"\nv.m3u8\n";
        assert!(pick_variant(m, &base()).is_err());
    }

    #[test]
    fn media_playlist_with_keys() {
        let m = "#EXTM3U\n#EXT-X-MEDIA-SEQUENCE:7\n#EXT-X-KEY:METHOD=AES-128,URI=\"k.bin\"\n#EXTINF:4,\na.ts\n#EXT-X-KEY:METHOD=NONE\n#EXTINF:4,\nhttps://o.cdn/b.ts\n#EXT-X-ENDLIST\n";
        let p = parse_media(m, &base()).unwrap();
        assert_eq!(p.segments.len(), 2);
        assert_eq!(p.segments[0].seq, 7);
        assert_eq!(p.segments[0].key.as_ref().unwrap().url, "https://cdn.example/show/k.bin");
        assert!(p.segments[1].key.is_none());
        assert_eq!(p.extension(), "ts");
    }

    #[test]
    fn live_is_refused() {
        assert!(parse_media("#EXTM3U\n#EXTINF:4,\na.ts\n", &base()).is_err());
    }
}
