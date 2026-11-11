// SPDX-License-Identifier: GPL-3.0-only
// Ported in part from YTubic (https://github.com/NUber-dev/YTubic), Copyright George Shyshov.
//! Lyrics: YouTube Music's own (synced when it has them), then LRCLIB.
//!
//! The YouTube Music path follows YTubic (GPL-3.0-only,
//! github.com/NUber-dev/YTubic, `src/lib/lyrics/ytmusic.ts`): the Android
//! Music client gets timed lyrics where the web client only gets plain text.
//! Both calls are anonymous.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::parse::{collect_key, s, text};
use crate::error::{Error, Result};

const API: &str = "https://music.youtube.com/youtubei/v1";
const ANDROID_MUSIC_VERSION: &str = "7.21.50";
const LRCLIB: &str = "https://lrclib.net/api";
const TIMEOUT: Duration = Duration::from_secs(12);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricLine {
    /// Seconds from the start; `None` for unsynced lyrics.
    pub time: Option<f64>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lyrics {
    /// "YouTube Music" or "LRCLIB".
    pub source: String,
    pub synced: bool,
    pub lines: Vec<LyricLine>,
}

fn plain(source: &str, body: &str) -> Lyrics {
    Lyrics {
        source: source.into(),
        synced: false,
        lines: body.lines().map(|l| LyricLine { time: None, text: l.trim().to_string() }).collect(),
    }
}

async fn android_music(http: &reqwest::Client, endpoint: &str, mut body: Value) -> Result<Value> {
    let url = format!("{API}/{endpoint}?prettyPrint=false");
    body["context"] = json!({ "client": {
        "clientName": "ANDROID_MUSIC",
        "clientVersion": ANDROID_MUSIC_VERSION,
        "androidSdkVersion": 34,
        "hl": "en",
        "gl": "US",
    } });
    let resp = http
        .post(&url)
        .header("Content-Type", "application/json")
        .header("X-YouTube-Client-Name", "21")
        .header("X-YouTube-Client-Version", ANDROID_MUSIC_VERSION)
        .header("Origin", "https://music.youtube.com")
        .timeout(TIMEOUT)
        .body(body.to_string())
        .send()
        .await
        .map_err(|e| Error::Network { url: url.clone(), source: e })?;
    if !resp.status().is_success() {
        return Err(Error::Http { status: resp.status().as_u16(), url });
    }
    resp.json().await.map_err(|e| Error::parse("lyrics response", e.to_string()))
}

/// YouTube Music's lyrics for a video, if it has any.
pub async fn youtube_music(http: &reqwest::Client, video_id: &str) -> Result<Option<Lyrics>> {
    let next = android_music(http, "next", json!({ "videoId": video_id })).await?;
    let mut browse_id = None;
    for tab in collect_key(&next, "tabRenderer") {
        let Some(ep) = tab.pointer("/endpoint/browseEndpoint") else { continue };
        let page = s(ep, "/browseEndpointContextSupportedConfigs/browseEndpointContextMusicConfig/pageType");
        if page != Some("MUSIC_PAGE_TYPE_TRACK_LYRICS") {
            continue;
        }
        if tab.get("unselectable").and_then(Value::as_bool) == Some(true) {
            return Ok(None);
        }
        browse_id = s(ep, "/browseId").map(str::to_string);
        break;
    }
    let Some(browse_id) = browse_id else { return Ok(None) };
    let page = android_music(http, "browse", json!({ "browseId": browse_id })).await?;
    Ok(parse_youtube_lyrics(&page))
}

pub fn parse_youtube_lyrics(page: &Value) -> Option<Lyrics> {
    for block in collect_key(page, "timedLyricsData") {
        let Some(rows) = block.as_array().filter(|r| !r.is_empty()) else { continue };
        let lines: Vec<LyricLine> = rows
            .iter()
            .filter_map(|row| {
                let start = row.pointer("/cueRange/startTimeMilliseconds")?;
                let ms = start.as_f64().or_else(|| start.as_str().and_then(|s| s.parse().ok()))?;
                Some(LyricLine { time: Some(ms / 1000.0), text: s(row, "/lyricLine").unwrap_or("").trim().to_string() })
            })
            .collect();
        if !lines.is_empty() {
            return Some(Lyrics { source: "YouTube Music".into(), synced: true, lines });
        }
    }
    for shelf in collect_key(page, "musicDescriptionShelfRenderer") {
        let body = text(shelf.get("description"));
        if !body.trim().is_empty() {
            return Some(plain("YouTube Music", body.trim()));
        }
    }
    None
}

/// `[mm:ss.xx] line` LRC text into timed lines.
pub fn parse_lrc(lrc: &str) -> Vec<LyricLine> {
    let mut out = Vec::new();
    for line in lrc.lines() {
        let mut rest = line.trim();
        let mut times = Vec::new();
        while let Some(stripped) = rest.strip_prefix('[') {
            let Some((stamp, after)) = stripped.split_once(']') else { break };
            let Some((m, sec)) = stamp.split_once(':') else { break };
            let (Ok(m), Ok(sec)) = (m.trim().parse::<f64>(), sec.trim().parse::<f64>()) else { break };
            times.push(m * 60.0 + sec);
            rest = after;
        }
        for t in times {
            out.push(LyricLine { time: Some(t), text: rest.trim().to_string() });
        }
    }
    out.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
    out
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LrclibRecord {
    #[serde(default)]
    track_name: String,
    #[serde(default)]
    duration: Option<f64>,
    #[serde(default)]
    plain_lyrics: Option<String>,
    #[serde(default)]
    synced_lyrics: Option<String>,
    #[serde(default)]
    instrumental: bool,
}

fn from_record(r: &LrclibRecord) -> Option<Lyrics> {
    if r.instrumental {
        return None;
    }
    if let Some(synced) = r.synced_lyrics.as_deref().filter(|s| !s.trim().is_empty()) {
        let lines = parse_lrc(synced);
        if !lines.is_empty() {
            return Some(Lyrics { source: "LRCLIB".into(), synced: true, lines });
        }
    }
    r.plain_lyrics.as_deref().filter(|s| !s.trim().is_empty()).map(|p| plain("LRCLIB", p.trim()))
}

fn norm(s: &str) -> String {
    s.to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect()
}

/// LRCLIB: an exact lookup first, then a search scored by title and length.
pub async fn lrclib(
    http: &reqwest::Client,
    title: &str,
    artist: &str,
    duration: Option<u32>,
) -> Result<Option<Lyrics>> {
    if title.trim().is_empty() {
        return Ok(None);
    }
    let client_header = "Velmora (https://github.com/ashfaqizop/Velmora)";
    if !artist.is_empty() {
        let mut q = vec![("track_name", title.to_string()), ("artist_name", artist.to_string())];
        if let Some(d) = duration {
            q.push(("duration", d.to_string()));
        }
        let resp = http
            .get(format!("{LRCLIB}/get"))
            .query(&q)
            .header("Lrclib-Client", client_header)
            .timeout(TIMEOUT)
            .send()
            .await;
        if let Ok(r) = resp {
            if r.status().is_success() {
                if let Ok(rec) = r.json::<LrclibRecord>().await {
                    if let Some(l) = from_record(&rec) {
                        return Ok(Some(l));
                    }
                }
            }
        }
    }
    let q = format!("{title} {artist}");
    let url = format!("{LRCLIB}/search");
    let resp = http
        .get(&url)
        .query(&[("q", q.trim())])
        .header("Lrclib-Client", client_header)
        .timeout(TIMEOUT)
        .send()
        .await
        .map_err(|e| Error::Network { url: url.clone(), source: e })?;
    if !resp.status().is_success() {
        return Err(Error::Http { status: resp.status().as_u16(), url });
    }
    let records: Vec<LrclibRecord> = resp.json().await.map_err(|e| Error::parse("LRCLIB search", e.to_string()))?;
    let want = norm(title);
    let best = records
        .iter()
        .filter(|r| {
            let have = norm(&r.track_name);
            !want.is_empty() && (have.contains(&want) || want.contains(&have))
        })
        .filter(|r| match (duration, r.duration) {
            (Some(d), Some(rd)) => (rd - f64::from(d)).abs() <= 4.0,
            _ => true,
        })
        // Synced lyrics first.
        .max_by_key(|r| r.synced_lyrics.as_deref().is_some_and(|s| !s.is_empty()));
    Ok(best.and_then(from_record))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lrc_lines() {
        let l = parse_lrc("[00:12.50] Hello\n[01:02.00][01:30.00] Again\n[ar:Someone]\n");
        assert_eq!(l.len(), 3);
        assert_eq!(l[0], LyricLine { time: Some(12.5), text: "Hello".into() });
        assert_eq!(l[2].time, Some(90.0));
    }

    #[test]
    fn youtube_timed() {
        let page = json!({ "x": { "timedLyricsData": [
            { "lyricLine": "One", "cueRange": { "startTimeMilliseconds": "1500" } },
            { "lyricLine": "Two", "cueRange": { "startTimeMilliseconds": 3000 } }
        ] } });
        let l = parse_youtube_lyrics(&page).unwrap();
        assert!(l.synced);
        assert_eq!(l.lines[1].time, Some(3.0));
    }
}
