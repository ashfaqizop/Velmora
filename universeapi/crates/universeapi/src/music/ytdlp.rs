// SPDX-License-Identifier: GPL-3.0-only
// Ported in part from YTubic (https://github.com/NUber-dev/YTubic), Copyright George Shyshov.
//! A managed copy of yt-dlp for music playback.
//!
//! The approach follows YTubic (GPL-3.0-only, github.com/NUber-dev/YTubic,
//! `src-tauri/src/ytdlp.rs`): the official release is downloaded on first
//! use and updated on its own every 72 hours, because YouTube breaks
//! extractors often and yt-dlp ships fixes within days.
//!
//! Streams are always resolved anonymously. YTubic found that YouTube treats a
//! signed-in yt-dlp request as a bot and strips the real formats, so the
//! user's cookies never go near yt-dlp, and playback can't put the account at
//! risk.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::error::{Error, Result};

#[cfg(windows)]
const BINARY: &str = "yt-dlp.exe";
#[cfg(not(windows))]
const BINARY: &str = "yt-dlp";

#[cfg(windows)]
const DOWNLOAD_URL: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe";
#[cfg(target_os = "macos")]
const DOWNLOAD_URL: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos";
#[cfg(all(unix, not(target_os = "macos")))]
const DOWNLOAD_URL: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux";

const UPDATE_EVERY: Duration = Duration::from_secs(72 * 3600);
const RESOLVE_TIMEOUT: Duration = Duration::from_secs(45);

/// What the player needs to play one track.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resolved {
    pub video_id: String,
    pub title: String,
    pub duration: Option<f64>,
    /// The audio stream.
    pub audio_url: String,
    /// The video stream, when video was asked for and exists. Play it with
    /// `audio_url` as the external audio track.
    pub video_url: Option<String>,
    pub video_height: Option<u32>,
    /// Headers googlevideo expects with these URLs.
    pub headers: Vec<(String, String)>,
    pub audio_codec: Option<String>,
    /// Audio bitrate in kbps.
    pub audio_bitrate: Option<f64>,
    pub ext: Option<String>,
    pub filesize: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamMode {
    Audio,
    Video,
}

/// How much audio to fetch. YouTube serves Opus around 160 kbps at best for
/// most accounts (256 kbps AAC needs Premium).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioQuality {
    /// About 50 to 70 kbps: data saver.
    Low,
    /// About 128 kbps.
    Normal,
    /// The best there is.
    #[default]
    High,
}

impl AudioQuality {
    /// yt-dlp's format filter for the audio part. Each falls back to any
    /// audio so a song never fails just for want of the exact bitrate.
    fn audio_format(self) -> &'static str {
        match self {
            AudioQuality::Low => "bestaudio[abr<=72][protocol^=http]/worstaudio[protocol^=http]/bestaudio",
            AudioQuality::Normal => "bestaudio[abr<=136][protocol^=http]/bestaudio[protocol^=http]/bestaudio",
            AudioQuality::High => "bestaudio[protocol^=http]/bestaudio",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YtDlpStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub path: String,
}

pub struct YtDlp {
    dir: PathBuf,
    http: reqwest::Client,
    /// One install or update at a time.
    lock: Mutex<()>,
}

/// A yt-dlp command that doesn't flash a console window on Windows.
fn command(program: &Path) -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new(program);
    cmd.kill_on_drop(true);
    #[cfg(windows)]
    cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    cmd
}

/// YouTube video IDs are 11 characters of [A-Za-z0-9_-].
pub fn valid_video_id(id: &str) -> bool {
    id.len() == 11 && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

impl YtDlp {
    pub fn new(dir: PathBuf, http: reqwest::Client) -> Self {
        Self { dir, http, lock: Mutex::new(()) }
    }

    pub fn path(&self) -> PathBuf {
        self.dir.join(BINARY)
    }

    fn stamp(&self) -> PathBuf {
        self.dir.join("yt-dlp.checked")
    }

    pub fn installed(&self) -> bool {
        self.path().is_file()
    }

    /// Make sure yt-dlp is there: download it on first use, and start a
    /// self-update in the background when the last one is over 72 hours old.
    pub async fn ensure(self: &std::sync::Arc<Self>) -> Result<PathBuf> {
        if !self.installed() {
            self.download().await?;
        } else if self.update_due() {
            let me = self.clone();
            tokio::spawn(async move {
                if let Err(e) = me.update().await {
                    tracing::warn!("yt-dlp update failed: {e}");
                }
            });
        }
        Ok(self.path())
    }

    fn update_due(&self) -> bool {
        let checked = std::fs::metadata(self.stamp()).and_then(|m| m.modified()).ok();
        checked.and_then(|t| SystemTime::now().duration_since(t).ok()).is_none_or(|age| age >= UPDATE_EVERY)
    }

    fn touch_stamp(&self) {
        let _ = std::fs::write(self.stamp(), chrono::Utc::now().to_rfc3339());
    }

    async fn download(&self) -> Result<()> {
        let _guard = self.lock.lock().await;
        if self.installed() {
            return Ok(());
        }
        tokio::fs::create_dir_all(&self.dir).await.map_err(|e| Error::Transport(format!("yt-dlp folder: {e}")))?;
        tracing::info!("downloading yt-dlp");
        let resp = self
            .http
            .get(DOWNLOAD_URL)
            .timeout(Duration::from_secs(300))
            .send()
            .await
            .map_err(|e| Error::Network { url: DOWNLOAD_URL.into(), source: e })?;
        if !resp.status().is_success() {
            return Err(Error::Http { status: resp.status().as_u16(), url: DOWNLOAD_URL.into() });
        }
        let bytes = resp.bytes().await.map_err(|e| Error::Network { url: DOWNLOAD_URL.into(), source: e })?;
        if bytes.len() < 1_000_000 {
            return Err(Error::Transport("the yt-dlp download was cut short".into()));
        }
        let part = self.dir.join(format!("{BINARY}.part"));
        tokio::fs::write(&part, &bytes).await.map_err(|e| Error::Transport(format!("save yt-dlp: {e}")))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&part, std::fs::Permissions::from_mode(0o755));
        }
        tokio::fs::rename(&part, self.path()).await.map_err(|e| Error::Transport(format!("install yt-dlp: {e}")))?;
        self.touch_stamp();
        Ok(())
    }

    /// Run yt-dlp's own updater. Returns the version after it.
    pub async fn update(&self) -> Result<String> {
        let guard = self.lock.lock().await;
        let out = command(&self.path())
            .args(["-U", "--no-warnings"])
            .output()
            .await
            .map_err(|e| Error::Transport(format!("run yt-dlp: {e}")))?;
        self.touch_stamp();
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            return Err(Error::Transport(format!("yt-dlp update: {}", crate::net::prefix(err.trim(), 300))));
        }
        drop(guard);
        self.version().await
    }

    pub async fn version(&self) -> Result<String> {
        let out = command(&self.path())
            .arg("--version")
            .output()
            .await
            .map_err(|e| Error::Transport(format!("run yt-dlp: {e}")))?;
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    pub async fn status(&self) -> YtDlpStatus {
        let installed = self.installed();
        let version = if installed { self.version().await.ok().filter(|v| !v.is_empty()) } else { None };
        YtDlpStatus { installed, version, path: self.path().display().to_string() }
    }

    /// Resolve a video ID to playable stream URLs.
    pub async fn resolve(
        self: &std::sync::Arc<Self>,
        video_id: &str,
        mode: StreamMode,
        max_height: u32,
    ) -> Result<Resolved> {
        self.resolve_with(video_id, mode, max_height, AudioQuality::High).await
    }

    /// `resolve` at a chosen audio quality.
    pub async fn resolve_with(
        self: &std::sync::Arc<Self>,
        video_id: &str,
        mode: StreamMode,
        max_height: u32,
        quality: AudioQuality,
    ) -> Result<Resolved> {
        if !valid_video_id(video_id) {
            return Err(Error::NotFound(format!("not a YouTube video ID: {video_id}")));
        }
        let program = self.ensure().await?;
        let format = match mode {
            StreamMode::Audio => quality.audio_format().to_string(),
            StreamMode::Video => {
                // The first audio alternative, joined to the video.
                let audio = quality.audio_format().split('/').next().unwrap_or("bestaudio");
                format!(
                    "bestvideo[height<={max_height}][protocol^=http]+{audio}/bestvideo[height<={max_height}][protocol^=http]+bestaudio[protocol^=http]/best[height<={max_height}][protocol^=http]/best"
                )
            }
        };
        let url = format!("https://www.youtube.com/watch?v={video_id}");
        let run = command(&program).args(["-j", "-f", &format, "--no-playlist", "--no-warnings", &url]).output();
        let out = tokio::time::timeout(RESOLVE_TIMEOUT, run)
            .await
            .map_err(|_| Error::Transport("yt-dlp took too long".into()))?
            .map_err(|e| Error::Transport(format!("run yt-dlp: {e}")))?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            return Err(Error::Transport(format!("yt-dlp: {}", crate::net::prefix(err.trim(), 400))));
        }
        let json: Value =
            serde_json::from_slice(&out.stdout).map_err(|e| Error::parse("yt-dlp output", e.to_string()))?;
        parse_resolved(video_id, &json)
    }
}

fn headers_of(v: &Value) -> Vec<(String, String)> {
    v.get("http_headers")
        .and_then(Value::as_object)
        .map(|m| m.iter().filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string()))).collect())
        .unwrap_or_default()
}

fn is_audio_only(f: &Value) -> bool {
    f.get("vcodec").and_then(Value::as_str).is_some_and(|v| v == "none")
}

/// Read yt-dlp's `-j` output. With a merged format the streams are in
/// `requested_formats`; otherwise the top level is the one stream.
pub fn parse_resolved(video_id: &str, json: &Value) -> Result<Resolved> {
    let formats = json.get("requested_formats").and_then(Value::as_array).cloned().unwrap_or_default();
    let (audio, video) = if formats.is_empty() {
        let single = json.clone();
        if is_audio_only(&single) {
            (single, None)
        } else {
            // A progressive file carries both: play it as the video, and as audio when video is off.
            (single.clone(), Some(single))
        }
    } else {
        let audio = formats.iter().find(|f| is_audio_only(f)).cloned().or_else(|| formats.last().cloned());
        let video = formats.iter().find(|f| !is_audio_only(f)).cloned();
        (audio.ok_or_else(|| Error::parse("yt-dlp output", "no audio format"))?, video)
    };
    let audio_url =
        audio.get("url").and_then(Value::as_str).ok_or_else(|| Error::parse("yt-dlp output", "no stream URL"))?;
    let mut headers = headers_of(&audio);
    if headers.is_empty() {
        headers = headers_of(json);
    }
    Ok(Resolved {
        video_id: video_id.to_string(),
        title: json.get("title").and_then(Value::as_str).unwrap_or_default().to_string(),
        duration: json.get("duration").and_then(Value::as_f64),
        audio_url: audio_url.to_string(),
        video_url: video.as_ref().and_then(|v| v.get("url")).and_then(Value::as_str).map(str::to_string),
        video_height: video.as_ref().and_then(|v| v.get("height")).and_then(Value::as_u64).map(|h| h as u32),
        headers,
        audio_codec: audio.get("acodec").and_then(Value::as_str).map(str::to_string),
        audio_bitrate: audio.get("abr").and_then(Value::as_f64),
        ext: audio.get("ext").and_then(Value::as_str).map(str::to_string),
        filesize: audio.get("filesize").or_else(|| audio.get("filesize_approx")).and_then(Value::as_u64),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn video_ids() {
        assert!(valid_video_id("4NRXx6U8ABQ"));
        assert!(!valid_video_id("4NRXx6U8AB"));
        assert!(!valid_video_id("4NRXx6U8AB;"));
    }

    #[test]
    fn parses_audio_only() {
        let j = json!({ "title": "T", "duration": 200.0, "url": "https://a", "vcodec": "none", "acodec": "opus", "abr": 160.0, "ext": "webm", "http_headers": { "User-Agent": "UA" } });
        let r = parse_resolved("4NRXx6U8ABQ", &j).unwrap();
        assert_eq!(r.audio_url, "https://a");
        assert!(r.video_url.is_none());
        assert_eq!(r.headers, vec![("User-Agent".to_string(), "UA".to_string())]);
    }

    #[test]
    fn parses_merged() {
        let j = json!({ "title": "T", "requested_formats": [
            { "url": "https://v", "vcodec": "avc1", "acodec": "none", "height": 1080 },
            { "url": "https://a", "vcodec": "none", "acodec": "opus" }
        ] });
        let r = parse_resolved("4NRXx6U8ABQ", &j).unwrap();
        assert_eq!(r.audio_url, "https://a");
        assert_eq!(r.video_url.as_deref(), Some("https://v"));
        assert_eq!(r.video_height, Some(1080));
    }
}
