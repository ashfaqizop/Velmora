// SPDX-License-Identifier: GPL-3.0-or-later
//! Movies and shows on the built-in player, for PCs where mpv can't run.
//!
//! WebView2 plays H.264, VP9 and AV1 video with AAC, MP3, Opus or FLAC
//! audio from an MP4. ffmpeg turns whatever a source sends into that, on the
//! fly: streams it can already play are only repackaged (almost no CPU), the
//! rest (HEVC, 10-bit H.264, AC3, DTS...) is converted. The result is a
//! fragmented MP4 served from the loopback server at `/w/{token}`, read by a
//! `<video>` element as it comes.
//!
//! A session starts at one point in the film. Seeking past what's loaded
//! asks for a new session from there. Text subtitles come out of the same
//! ffmpeg as WebVTT files, one per track, that grow as the film is read;
//! picture subtitles (Blu-ray, DVD) are drawn onto the picture instead.
//!
//! Timing: every output keeps the film's own clock (`-copyts`), shifted so
//! the session's first frame is 0. When the video is only repackaged it
//! can't start mid-GOP, so the session really starts at the keyframe before
//! the asked-for spot; that keyframe's time is looked up first, and it's
//! the offset the page adds to the `<video>`'s clock. Video and subtitles
//! share that one shift, so they stay in step.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Mutex;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::AppState;

const PROBE_TIMEOUT: Duration = Duration::from_secs(45);
const SUB_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct VideoInfo {
    codec: String,
    width: u32,
    height: u32,
    pix_fmt: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AudioInfo {
    /// Among the audio streams, from 0.
    index: u32,
    codec: String,
    channels: String,
    lang: Option<String>,
    title: Option<String>,
    default: bool,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubInfo {
    /// Among the subtitle streams, from 0.
    index: u32,
    codec: String,
    lang: Option<String>,
    title: Option<String>,
    default: bool,
    /// Text (becomes WebVTT) rather than pictures (drawn onto the video).
    text: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Probe {
    /// Seconds; none for live streams.
    duration: Option<f64>,
    video: Option<VideoInfo>,
    audio: Vec<AudioInfo>,
    subs: Vec<SubInfo>,
}

/// How one session is made, chosen by the page from the probe.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    /// Seconds into the film.
    start: f64,
    /// Audio stream (among audio streams); none for the first.
    audio: Option<u32>,
    /// Repackage the video as it is.
    copy_video: bool,
    /// Repackage the audio as it is.
    copy_audio: bool,
    /// When converting, no taller than this.
    max_height: u32,
    /// A picture subtitle stream to draw onto the video (forces converting).
    burn_sub: Option<u32>,
    /// Text subtitle streams to write out as WebVTT.
    text_subs: Vec<u32>,
    /// A live stream: no seeking.
    live: bool,
}

struct Job {
    args: Vec<String>,
}

/// Sessions by token, and where their subtitles go.
pub struct VideoJobs {
    ffmpeg: PathBuf,
    dir: PathBuf,
    jobs: Mutex<HashMap<String, Job>>,
}

impl VideoJobs {
    pub fn new(ffmpeg: PathBuf, dir: PathBuf) -> Self {
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::create_dir_all(&dir);
        Self { ffmpeg, dir, jobs: Mutex::new(HashMap::new()) }
    }

    fn args(&self, token: &str) -> Option<Vec<String>> {
        self.jobs.lock().ok()?.get(token).map(|j| j.args.clone())
    }

    fn sub_file(&self, token: &str, index: u32) -> PathBuf {
        self.dir.join(format!("{token}-{index}.vtt"))
    }
}

fn token() -> Result<String, String> {
    let mut raw = [0u8; 12];
    getrandom::getrandom(&mut raw).map_err(|e| e.to_string())?;
    Ok(raw.iter().map(|b| format!("{b:02x}")).collect())
}

fn command(ffmpeg: &Path) -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new(ffmpeg);
    cmd.stdin(Stdio::null()).kill_on_drop(true);
    #[cfg(windows)]
    cmd.creation_flags(0x0800_0000); // no console window
    cmd
}

fn ffmpeg_ready(jobs: &VideoJobs) -> Result<(), String> {
    if jobs.ffmpeg.is_file() {
        Ok(())
    } else {
        Err("The built-in player needs ffmpeg, which hasn't finished downloading.".into())
    }
}

/// What's in a stream or file: length, video, audio and subtitle tracks.
#[tauri::command]
pub async fn web_video_probe(state: State<'_, AppState>, input: String) -> Result<Probe, String> {
    let jobs = &state.sealed.video;
    ffmpeg_ready(jobs)?;
    let mut cmd = command(&jobs.ffmpeg);
    cmd.args(["-hide_banner", "-nostdin", "-i", &input]).stdout(Stdio::null()).stderr(Stdio::piped());
    let out = tokio::time::timeout(PROBE_TIMEOUT, cmd.output())
        .await
        .map_err(|_| "The stream took too long to open.".to_string())?
        .map_err(|e| format!("couldn't run ffmpeg: {e}"))?;
    let text = String::from_utf8_lossy(&out.stderr);
    parse_probe(&text).ok_or_else(|| {
        let last = text.trim().lines().last().unwrap_or("no answer").trim().to_string();
        format!("ffmpeg couldn't read the stream: {last}")
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Opened {
    token: String,
    url: String,
    /// The film's time at the session's first frame.
    offset: f64,
}

/// Start a session; returns its token, the URL the `<video>` reads, and
/// where in the film it starts.
#[tauri::command]
pub async fn web_video_open(state: State<'_, AppState>, input: String, plan: Plan) -> Result<Opened, String> {
    let jobs = &state.sealed.video;
    ffmpeg_ready(jobs)?;
    let token = token()?;
    let seeking = !plan.live && plan.start > 0.5;
    let offset = if !seeking {
        0.0
    } else if plan.copy_video && plan.burn_sub.is_none() {
        keyframe_before(&jobs.ffmpeg, &input, plan.start).await.unwrap_or(plan.start)
    } else {
        // Converting decodes up to the exact spot.
        plan.start
    };
    let args = build_args(jobs, &token, &input, &plan, offset);
    if let Ok(mut map) = jobs.jobs.lock() {
        // A film's worth of seeks; nothing older is still playing.
        if map.len() > 64 {
            map.clear();
        }
        map.insert(token.clone(), Job { args });
    }
    let url = format!("http://127.0.0.1:{}/w/{token}", state.stream_port);
    Ok(Opened { token, url, offset })
}

/// Where a repackaged session from `start` really begins: the time of the
/// first video packet ffmpeg reads after seeking there.
async fn keyframe_before(ffmpeg: &Path, input: &str, start: f64) -> Option<f64> {
    let mut cmd = command(ffmpeg);
    cmd.args(["-hide_banner", "-nostdin", "-loglevel", "error", "-copyts", "-start_at_zero", "-ss", &format!("{start:.3}"), "-i", input])
        .args(["-map", "0:v:0", "-c", "copy", "-frames:v", "1", "-f", "framemd5", "pipe:1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let out = tokio::time::timeout(Duration::from_secs(20), cmd.output()).await.ok()?.ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    // "#tb 0: 1/1000" then "0, 595300, 595300, 42, 81234, <md5>"
    let (num, den) = text.lines().find_map(|l| {
        let tb = l.strip_prefix("#tb 0:")?.trim();
        let (n, d) = tb.split_once('/')?;
        Some((n.trim().parse::<f64>().ok()?, d.trim().parse::<f64>().ok()?))
    })?;
    let pts: f64 = text.lines().find(|l| !l.starts_with('#') && !l.trim().is_empty())?.split(',').nth(2)?.trim().parse().ok()?;
    let t = pts * num / den;
    (t.is_finite() && t >= 0.0 && t <= start + 1.0).then_some(t)
}

/// The WebVTT written so far for one text subtitle track of a session.
#[tauri::command]
pub async fn web_video_subs(state: State<'_, AppState>, token: String, index: u32) -> Result<String, String> {
    if !token.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("bad session".into());
    }
    let path = state.sealed.video.sub_file(&token, index);
    Ok(tokio::fs::read_to_string(&path).await.unwrap_or_default())
}

/// A whole subtitle file (or one track of a file) as WebVTT: sidecar files
/// and subtitles a source links to.
#[tauri::command]
pub async fn web_subtitle(state: State<'_, AppState>, input: String, index: Option<u32>) -> Result<String, String> {
    let jobs = &state.sealed.video;
    ffmpeg_ready(jobs)?;
    let mut cmd = command(&jobs.ffmpeg);
    cmd.args(["-hide_banner", "-nostdin", "-loglevel", "error", "-i", &input]);
    if let Some(i) = index {
        cmd.args(["-map", &format!("0:s:{i}")]);
    }
    cmd.args(["-f", "webvtt", "pipe:1"]).stdout(Stdio::piped()).stderr(Stdio::piped());
    let out = tokio::time::timeout(SUB_TIMEOUT, cmd.output())
        .await
        .map_err(|_| "The subtitles took too long.".to_string())?
        .map_err(|e| format!("couldn't run ffmpeg: {e}"))?;
    if out.stdout.is_empty() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(format!("Couldn't read the subtitles: {}", err.trim().lines().last().unwrap_or("nothing came back")));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn build_args(jobs: &VideoJobs, token: &str, input: &str, plan: &Plan, offset: f64) -> Vec<String> {
    let mut a: Vec<String> = ["-hide_banner", "-nostdin", "-loglevel", "error"].iter().map(|s| s.to_string()).collect();
    if input.starts_with("http") {
        a.extend(["-reconnect", "1", "-reconnect_on_network_error", "1", "-reconnect_delay_max", "5"].map(String::from));
    }
    let seeking = !plan.live && plan.start > 0.5;
    // Film time on every output, shifted so this session starts at 0.
    let shift = format!("{:.3}", -offset);
    if seeking {
        a.extend(["-copyts".into(), "-start_at_zero".into(), "-ss".into(), format!("{:.3}", plan.start)]);
    }
    a.extend(["-i".into(), input.to_string()]);

    let burn = plan.burn_sub;
    let convert = !plan.copy_video || burn.is_some();
    let scale = format!("scale=-2:'min(ih,{})'", plan.max_height.clamp(360, 2160));
    match burn {
        Some(s) => {
            a.extend(["-filter_complex".into(), format!("[0:v:0][0:s:{s}]overlay,{scale},format=yuv420p[v]"), "-map".into(), "[v]".into()]);
        }
        None => {
            a.extend(["-map".into(), "0:v:0?".into()]);
            if convert {
                a.extend(["-vf".into(), format!("{scale},format=yuv420p")]);
            }
        }
    }
    a.extend(["-map".into(), format!("0:a:{}?", plan.audio.unwrap_or(0))]);
    if convert {
        let preset = if plan.max_height > 720 { "superfast" } else { "veryfast" };
        a.extend(
            ["-c:v", "libx264", "-preset", preset, "-tune", "zerolatency", "-crf", "22", "-profile:v", "high", "-g", "48", "-sc_threshold", "0"]
                .map(String::from),
        );
    } else {
        a.extend(["-c:v".into(), "copy".into()]);
    }
    if plan.copy_audio {
        a.extend(["-c:a".into(), "copy".into()]);
    } else {
        a.extend(["-c:a", "aac", "-b:a", "192k", "-ac", "2"].map(String::from));
    }
    if seeking {
        a.extend(["-output_ts_offset".into(), shift.clone()]);
    }
    a.extend(
        ["-sn", "-dn", "-avoid_negative_ts", "make_zero", "-f", "mp4", "-movflags", "frag_keyframe+empty_moov+default_base_moof", "-frag_duration", "1000000", "pipe:1"]
            .map(String::from),
    );
    // Text subtitles, each to its own file, written as the film is read.
    for &i in plan.text_subs.iter().take(24) {
        let file = jobs.sub_file(token, i);
        a.extend(["-map".into(), format!("0:s:{i}"), "-c:s".into(), "webvtt".into(), "-flush_packets".into(), "1".into()]);
        if seeking {
            a.extend(["-output_ts_offset".into(), shift.clone()]);
        }
        a.extend(["-f".into(), "webvtt".into()]);
        a.push(file.to_string_lossy().into_owned());
    }
    a
}

/// Serve `/w/{token}`: run ffmpeg and send its MP4 as it comes. The film
/// stops being made when the player lets go of the connection.
pub async fn serve(sock: &mut TcpStream, jobs: &VideoJobs, token: &str, range: Option<&str>, head_only: bool) -> std::io::Result<()> {
    let token = token.split(['/', '?']).next().unwrap_or_default();
    let Some(args) = jobs.args(token) else {
        return crate::stream::respond_text(sock, 404, "unknown session").await;
    };
    // The MP4 is made as it's read, so there's no middle to jump to.
    if range.is_some_and(|r| !matches!(r.trim(), "bytes=0-" | "bytes=0-0")) {
        return crate::stream::respond_text(sock, 416, "not seekable").await;
    }
    let head = format!(
        "HTTP/1.1 200 OK\r\n{}Content-Type: video/mp4\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        crate::stream::CORS
    );
    sock.write_all(head.as_bytes()).await?;
    if head_only {
        return sock.shutdown().await;
    }
    let mut cmd = command(&jobs.ffmpeg);
    cmd.args(&args).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn()?;
    let mut stdout = child.stdout.take().ok_or_else(|| std::io::Error::other("no ffmpeg output"))?;
    // Keep ffmpeg's complaints for the log, without letting the pipe fill up.
    if let Some(mut err) = child.stderr.take() {
        tokio::spawn(async move {
            let mut buf = Vec::new();
            let _ = tokio::io::AsyncReadExt::read_to_end(&mut err, &mut buf).await;
            let text = String::from_utf8_lossy(&buf);
            if !text.trim().is_empty() {
                tracing::warn!("built-in player ffmpeg: {}", text.trim().lines().last().unwrap_or(""));
            }
        });
    }
    let copied = tokio::io::copy(&mut stdout, sock).await;
    let _ = child.kill().await;
    copied?;
    sock.shutdown().await
}

// ── Reading ffmpeg's description of the input ─────────────────────────

fn parse_probe(text: &str) -> Option<Probe> {
    if !text.contains("Input #0") {
        return None;
    }
    let mut probe = Probe { duration: None, video: None, audio: Vec::new(), subs: Vec::new() };
    let lines: Vec<&str> = text.lines().collect();
    for (n, line) in lines.iter().enumerate() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("Duration: ") {
            probe.duration = parse_duration(rest.split(',').next().unwrap_or(""));
            continue;
        }
        let Some(rest) = t.strip_prefix("Stream #") else { continue };
        // "0:1[0x2](eng): Audio: aac (LC), 48000 Hz, stereo, fltp (default)"
        let Some((head, body)) = rest.split_once(": ") else { continue };
        let lang = head.find('(').and_then(|i| head[i + 1..].split(')').next()).map(str::to_string).filter(|l| l != "und");
        let Some((kind, desc)) = body.split_once(": ") else { continue };
        let codec = desc.split([' ', ',']).next().unwrap_or("").to_string();
        let default = desc.contains("(default)");
        let title = stream_title(&lines[n + 1..]);
        match kind {
            "Video" if probe.video.is_none() => {
                // Cover art in a music file isn't a video track to play.
                if desc.contains("(attached pic)") {
                    continue;
                }
                let (width, height) = resolution(desc).unwrap_or((0, 0));
                let pix_fmt = desc
                    .split(", ")
                    .nth(1)
                    .map(|s| s.split('(').next().unwrap_or("").trim().to_string())
                    .unwrap_or_default();
                probe.video = Some(VideoInfo { codec, width, height, pix_fmt });
            }
            "Audio" => {
                let channels = desc.split(", ").nth(2).unwrap_or("").trim().to_string();
                let index = probe.audio.len() as u32;
                probe.audio.push(AudioInfo { index, codec, channels, lang, title, default });
            }
            "Subtitle" => {
                let text_sub = matches!(codec.as_str(), "subrip" | "srt" | "ass" | "ssa" | "webvtt" | "mov_text" | "text");
                let index = probe.subs.len() as u32;
                probe.subs.push(SubInfo { index, codec, lang, title, default, text: text_sub });
            }
            _ => {}
        }
    }
    Some(probe)
}

/// "title : English" in the Metadata block right under a stream line.
fn stream_title(after: &[&str]) -> Option<String> {
    for l in after {
        let t = l.trim();
        if t.starts_with("Stream #") || t.starts_with("Input #") || t.starts_with("Output #") {
            break;
        }
        if let Some((k, v)) = t.split_once(':') {
            if k.trim().eq_ignore_ascii_case("title") {
                let v = v.trim();
                if !v.is_empty() {
                    return Some(v.chars().take(80).collect());
                }
            }
        }
    }
    None
}

fn parse_duration(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.starts_with("N/A") {
        return None;
    }
    let mut parts = s.split(':');
    let h: f64 = parts.next()?.parse().ok()?;
    let m: f64 = parts.next()?.parse().ok()?;
    let sec: f64 = parts.next()?.parse().ok()?;
    let total = h * 3600.0 + m * 60.0 + sec;
    (total > 0.0).then_some(total)
}

/// The first "1920x1080" in a stream description.
fn resolution(desc: &str) -> Option<(u32, u32)> {
    for word in desc.split([' ', ',']) {
        if let Some((w, h)) = word.split_once('x') {
            if let (Ok(w), Ok(h)) = (w.parse::<u32>(), h.parse::<u32>()) {
                if w >= 16 && h >= 16 {
                    return Some((w, h));
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "Input #0, matroska,webm, from 'http://127.0.0.1:5000/t/abc':
  Metadata:
    title           : Show
  Duration: 00:23:40.04, start: 0.000000, bitrate: 2337 kb/s
  Stream #0:0(jpn): Video: hevc (Main 10), yuv420p10le(tv, bt709), 1920x1080, SAR 1:1 DAR 16:9, 23.98 fps, 23.98 tbr, 1k tbn (default)
  Stream #0:1[0x2](jpn): Audio: eac3, 48000 Hz, 5.1(side), fltp (default)
    Metadata:
      title           : Japanese
  Stream #0:2(eng): Audio: aac (LC), 48000 Hz, stereo, fltp
  Stream #0:3(eng): Subtitle: ass (default)
    Metadata:
      title           : Full Subs
  Stream #0:4(eng): Subtitle: hdmv_pgs_subtitle
At least one output file must be specified";

    #[test]
    fn reads_ffmpeg_output() {
        let p = parse_probe(SAMPLE).expect("probe");
        assert!((p.duration.unwrap() - 1420.04).abs() < 0.01);
        let v = p.video.unwrap();
        assert_eq!((v.codec.as_str(), v.width, v.height, v.pix_fmt.as_str()), ("hevc", 1920, 1080, "yuv420p10le"));
        assert_eq!(p.audio.len(), 2);
        assert_eq!(p.audio[0].codec, "eac3");
        assert_eq!(p.audio[0].channels, "5.1(side)");
        assert_eq!(p.audio[0].title.as_deref(), Some("Japanese"));
        assert_eq!(p.audio[1].lang.as_deref(), Some("eng"));
        assert_eq!(p.subs.len(), 2);
        assert!(p.subs[0].text && p.subs[0].default);
        assert_eq!(p.subs[0].title.as_deref(), Some("Full Subs"));
        assert!(!p.subs[1].text);
    }

    #[test]
    fn live_has_no_duration() {
        let p = parse_probe("Input #0, hls, from 'x':\n  Duration: N/A, start: 1.0, bitrate: N/A\n  Stream #0:0: Video: h264 (High), yuv420p(tv), 1280x720").unwrap();
        assert!(p.duration.is_none());
        assert_eq!(p.video.unwrap().height, 720);
    }
}
