// SPDX-License-Identifier: GPL-3.0-or-later
//! Music downloads as ordinary audio files: MP3, M4A or Opus, with the title,
//! artist, album, track number and cover inside, so any player shows them
//! properly. (The older "Velmora only" vault download stays in music.rs.)
//!
//! Each song: the best audio is fetched (or taken from the songs saved ahead,
//! if it's there), the cover is fetched and cut square, then the ffmpeg that
//! Velmora installs on first launch converts and tags it. MP3 and M4A carry
//! the cover as an attached picture; Opus (Ogg) can't, so it goes in as a
//! METADATA_BLOCK_PICTURE comment, the way Ogg players read covers. Two songs
//! convert at a time; progress goes to the page as `music-export` events.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use base64::Engine as _;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::Semaphore;
use universeapi::music::{AudioQuality, MusicItem, StreamMode};

use crate::music::{file_title, music_dir, read_index, write_index, IndexEntry, MusicStore};
use crate::musiccache::{fetch, MusicCache};
use crate::AppState;

const CONVERT_TIMEOUT: Duration = Duration::from_secs(5 * 60);

pub struct MusicExports {
    slots: Semaphore,
    jobs: Mutex<HashMap<String, Job>>,
}

impl MusicExports {
    pub fn new() -> Self {
        Self { slots: Semaphore::new(2), jobs: Mutex::new(HashMap::new()) }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    video_id: String,
    title: String,
    artist: String,
    format: String,
    /// "waiting", "downloading", "converting", "done" or "failed".
    stage: &'static str,
    /// 0..1 while downloading.
    progress: f32,
    error: Option<String>,
}

fn update(app: &AppHandle, video_id: &str, f: impl FnOnce(&mut Job)) {
    let Some(ex) = app.try_state::<MusicExports>() else { return };
    let job = {
        let mut jobs = ex.jobs.lock().unwrap();
        let Some(job) = jobs.get_mut(video_id) else { return };
        f(job);
        job.clone()
    };
    let _ = app.emit("music-export", job);
}

/// Start downloading songs in a format. `album` and `tracks` come from an
/// album page, so the files get the album name and track numbers.
#[tauri::command]
pub async fn music_export(
    app: AppHandle,
    exports: State<'_, MusicExports>,
    items: Vec<MusicItem>,
    format: String,
    album: Option<String>,
    tracks: Option<Vec<u32>>,
) -> Result<usize, String> {
    if !matches!(format.as_str(), "mp3" | "m4a" | "opus") {
        return Err(format!("unknown format {format}"));
    }
    let mut started = 0;
    for (i, item) in items.into_iter().enumerate() {
        if !item.playable() {
            continue;
        }
        {
            let mut jobs = exports.jobs.lock().unwrap();
            // Already on its way.
            if jobs.get(&item.id).is_some_and(|j| !matches!(j.stage, "done" | "failed")) {
                continue;
            }
            jobs.insert(
                item.id.clone(),
                Job {
                    video_id: item.id.clone(),
                    title: item.title.clone(),
                    artist: item.artist_names(),
                    format: format.clone(),
                    stage: "waiting",
                    progress: 0.0,
                    error: None,
                },
            );
        }
        update(&app, &item.id, |_| {});
        let track = tracks.as_ref().and_then(|t| t.get(i).copied());
        let (app2, format2, album2) = (app.clone(), format.clone(), album.clone());
        tauri::async_runtime::spawn(async move {
            let id = item.id.clone();
            let result = run(&app2, item, &format2, album2, track).await;
            update(&app2, &id, |j| match result {
                Ok(()) => {
                    j.stage = "done";
                    j.progress = 1.0;
                }
                Err(e) => {
                    tracing::warn!("music download {id}: {e}");
                    j.stage = "failed";
                    j.error = Some(e);
                }
            });
        });
        started += 1;
    }
    Ok(started)
}

/// The downloads in this session, for a page that opens mid-way.
#[tauri::command]
pub fn music_exports(exports: State<'_, MusicExports>) -> Vec<Job> {
    exports.jobs.lock().unwrap().values().cloned().collect()
}

/// Forget finished and failed downloads.
#[tauri::command]
pub fn music_exports_clear(exports: State<'_, MusicExports>) {
    exports.jobs.lock().unwrap().retain(|_, j| !matches!(j.stage, "done" | "failed"));
}

async fn run(app: &AppHandle, item: MusicItem, format: &str, album: Option<String>, track: Option<u32>) -> Result<(), String> {
    let exports = app.state::<MusicExports>();
    let _slot = exports.slots.acquire().await.map_err(|e| e.to_string())?;
    let state = app.state::<AppState>();
    let ffmpeg = state.runtime_dir.join(if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" });
    if !ffmpeg.is_file() {
        return Err("Velmora's ffmpeg isn't installed yet. Restart Velmora to finish setting up.".into());
    }
    let out_dir = music_dir(&state);
    let tmp = out_dir.join(".velmora-tmp");
    tokio::fs::create_dir_all(&tmp).await.map_err(|e| e.to_string())?;
    let mut scratch = Scratch(Vec::new());

    // 1. The audio: a song saved ahead at High, or a fresh download.
    update(app, &item.id, |j| j.stage = "downloading");
    let cache = app.state::<MusicCache>();
    let (audio, src_codec) = match cache.find(&item.id).filter(|p| p.to_string_lossy().contains(".high.")) {
        Some(p) => {
            let codec = if p.extension().is_some_and(|e| e == "m4a" || e == "mp4") { "aac" } else { "opus" };
            (p, codec)
        }
        None => {
            let r = state
                .engine
                .music()
                .resolve_with(&item.id, StreamMode::Audio, 0, AudioQuality::High)
                .await
                .map_err(|e| e.to_string())?;
            let ext = r.ext.as_deref().filter(|e| e.chars().all(|c| c.is_ascii_alphanumeric()) && e.len() <= 5).unwrap_or("webm");
            let part = tmp.join(format!("{}.{ext}", item.id));
            scratch.0.push(part.clone());
            let (app2, id) = (app.clone(), item.id.clone());
            let progress = move |have: u64, total: Option<u64>| {
                if let Some(t) = total.filter(|t| *t > 0) {
                    let p = (have as f32 / t as f32).min(1.0);
                    update(&app2, &id, |j| j.progress = p);
                }
            };
            fetch(&state, &r, &part, &progress).await?;
            let aac = r.audio_codec.as_deref().is_some_and(|c| c.starts_with("mp4a")) || matches!(ext, "m4a" | "mp4");
            (part, if aac { "aac" } else { "opus" })
        }
    };

    // 2. The cover, cut square. A song without one still downloads.
    update(app, &item.id, |j| j.stage = "converting");
    let cover = cover(&state, &ffmpeg, &item, &tmp, &mut scratch).await;

    // 3. Convert and tag.
    let artist = item.artist_names();
    let album = album.or_else(|| item.album.clone()).unwrap_or_default();
    let mut tags: Vec<(&str, String)> = vec![("title", item.title.clone())];
    if !artist.is_empty() {
        tags.push(("artist", artist.clone()));
    }
    if !album.is_empty() {
        tags.push(("album", album.clone()));
        if let Some(first) = item.artists.first() {
            tags.push(("album_artist", first.name.clone()));
        }
    }
    if let Some(n) = track {
        tags.push(("track", n.to_string()));
    }
    tags.push(("comment", format!("https://music.youtube.com/watch?v={}", item.id)));
    if format == "opus" {
        if let Some(c) = &cover {
            if let Ok(bytes) = tokio::fs::read(c).await {
                tags.push(("METADATA_BLOCK_PICTURE", picture_block(&bytes)));
            }
        }
    }
    let meta = tmp.join(format!("{}.meta.txt", item.id));
    scratch.0.push(meta.clone());
    tokio::fs::write(&meta, ffmetadata(&tags)).await.map_err(|e| e.to_string())?;

    let ext = format;
    let out = tmp.join(format!("{}.out.{ext}", item.id));
    scratch.0.push(out.clone());
    let mut args: Vec<String> = vec!["-hide_banner".into(), "-loglevel".into(), "error".into(), "-y".into()];
    args.extend(["-i".into(), audio.to_string_lossy().into_owned()]);
    let with_cover = format != "opus" && cover.is_some();
    if with_cover {
        args.extend(["-i".into(), cover.as_ref().unwrap().to_string_lossy().into_owned()]);
    }
    let meta_input = if with_cover { 2 } else { 1 };
    args.extend(["-i".into(), meta.to_string_lossy().into_owned()]);
    args.extend(["-map".into(), "0:a:0".into()]);
    if with_cover {
        args.extend(["-map".into(), "1:v:0".into()]);
    }
    args.extend(["-map_metadata".into(), meta_input.to_string(), "-map_chapters".into(), "-1".into()]);
    match format {
        "mp3" => {
            // LAME V0: transparent, about 245 kbps.
            args.extend(["-c:a", "libmp3lame", "-q:a", "0", "-id3v2_version", "3"].map(String::from));
            if with_cover {
                args.extend(["-c:v", "copy", "-metadata:s:v", "title=Album cover", "-metadata:s:v", "comment=Cover (front)"].map(String::from));
            }
            args.extend(["-f", "mp3"].map(String::from));
        }
        "m4a" => {
            // AAC passes through as it is; Opus becomes AAC at 256 kbps.
            if src_codec == "aac" {
                args.extend(["-c:a", "copy"].map(String::from));
            } else {
                args.extend(["-c:a", "aac", "-b:a", "256k"].map(String::from));
            }
            if with_cover {
                args.extend(["-c:v", "copy", "-disposition:v:0", "attached_pic"].map(String::from));
            }
            args.extend(["-movflags", "+faststart", "-f", "mp4"].map(String::from));
        }
        _ => {
            // Opus passes through untouched; AAC becomes Opus at 160 kbps.
            if src_codec == "opus" {
                args.extend(["-c:a", "copy"].map(String::from));
            } else {
                args.extend(["-c:a", "libopus", "-b:a", "160k"].map(String::from));
            }
            args.extend(["-vn", "-f", "opus"].map(String::from));
        }
    }
    args.push(out.to_string_lossy().into_owned());
    ffmpeg_run(&ffmpeg, &args).await?;

    // 4. Into the Music folder under "Artist - Title", next to the other downloads.
    let title = file_title(&item);
    let dest = free_name(&out_dir, &title, ext);
    tokio::fs::rename(&out, &dest).await.map_err(|e| e.to_string())?;
    let store = app.state::<MusicStore>();
    let mut index = read_index(&store);
    index.retain(|e| e.item.id != item.id || e.file_title != title);
    index.push(IndexEntry { file_title: title, item });
    write_index(&store, &index);
    Ok(())
}

/// Temporary files this download made; gone when it ends, however it ends.
/// (A song taken from the saved-ahead cache isn't one of them.)
struct Scratch(Vec<PathBuf>);
impl Drop for Scratch {
    fn drop(&mut self) {
        for p in &self.0 {
            let _ = std::fs::remove_file(p);
        }
    }
}

/// Fetch the biggest cover and cut it square (a video's thumbnail is 16:9),
/// at most 1000 px, as JPEG.
async fn cover(state: &AppState, ffmpeg: &Path, item: &MusicItem, tmp: &Path, scratch: &mut Scratch) -> Option<PathBuf> {
    let best = item.thumbnails.iter().max_by_key(|t| t.width)?;
    let url = bigger(&best.url);
    let client = state.engine.ctx().http.client().clone();
    let bytes = client.get(&url).timeout(Duration::from_secs(30)).send().await.ok()?.bytes().await.ok()?;
    if bytes.len() < 512 {
        return None;
    }
    let raw = tmp.join(format!("{}.cover.src", item.id));
    let jpg = tmp.join(format!("{}.cover.jpg", item.id));
    scratch.0.push(raw.clone());
    scratch.0.push(jpg.clone());
    tokio::fs::write(&raw, &bytes).await.ok()?;
    let mut args: Vec<String> = ["-hide_banner", "-loglevel", "error", "-y", "-i"].map(String::from).to_vec();
    args.push(raw.to_string_lossy().into_owned());
    args.extend(["-vf", "crop='min(iw,ih)':'min(iw,ih)',scale='min(1000,iw)':-2", "-frames:v", "1", "-q:v", "2"].map(String::from));
    args.push(jpg.to_string_lossy().into_owned());
    ffmpeg_run(ffmpeg, &args).await.ok()?;
    jpg.is_file().then_some(jpg)
}

/// YouTube Music covers take their size from the URL ("=w120-h120-…").
fn bigger(url: &str) -> String {
    if url.contains("googleusercontent.com") || url.contains("ggpht.com") {
        if let Some(i) = url.rfind("=w") {
            return format!("{}=w1200-h1200-l90-rj", &url[..i]);
        }
    }
    url.to_string()
}

/// The FLAC picture block Ogg players read covers from, base64.
fn picture_block(jpeg: &[u8]) -> String {
    let mime = b"image/jpeg";
    let mut b = Vec::with_capacity(jpeg.len() + 64);
    b.extend_from_slice(&3u32.to_be_bytes()); // front cover
    b.extend_from_slice(&(mime.len() as u32).to_be_bytes());
    b.extend_from_slice(mime);
    b.extend_from_slice(&0u32.to_be_bytes()); // no description
    for v in [0u32, 0, 24, 0] {
        // width, height (unknown is allowed), depth, colours
        b.extend_from_slice(&v.to_be_bytes());
    }
    b.extend_from_slice(&(jpeg.len() as u32).to_be_bytes());
    b.extend_from_slice(jpeg);
    base64::engine::general_purpose::STANDARD.encode(b)
}

/// ffmpeg's metadata file. A file rather than -metadata flags, because a
/// cover as text is far longer than a Windows command line allows.
fn ffmetadata(tags: &[(&str, String)]) -> String {
    let esc = |v: &str| {
        v.chars()
            .flat_map(|c| match c {
                '=' | ';' | '#' | '\\' | '\n' => vec!['\\', c],
                '\r' => vec![],
                _ => vec![c],
            })
            .collect::<String>()
    };
    let mut out = String::from(";FFMETADATA1\n");
    for (k, v) in tags {
        out.push_str(&format!("{}={}\n", esc(k), esc(v)));
    }
    out
}

/// "Artist - Title.mp3", or "Artist - Title (2).mp3" when that's taken.
fn free_name(dir: &Path, title: &str, ext: &str) -> PathBuf {
    let first = dir.join(format!("{title}.{ext}"));
    if !first.exists() {
        return first;
    }
    (2..1000).map(|n| dir.join(format!("{title} ({n}).{ext}"))).find(|p| !p.exists()).unwrap_or(first)
}

async fn ffmpeg_run(ffmpeg: &Path, args: &[String]) -> Result<(), String> {
    let mut cmd = tokio::process::Command::new(ffmpeg);
    cmd.args(args).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::piped()).kill_on_drop(true);
    #[cfg(windows)]
    cmd.creation_flags(0x0800_0000); // no console window
    let out = tokio::time::timeout(CONVERT_TIMEOUT, cmd.output())
        .await
        .map_err(|_| "converting took too long".to_string())?
        .map_err(|e| format!("couldn't run ffmpeg: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&out.stderr);
        Err(format!("ffmpeg: {}", err.trim().lines().last().unwrap_or("failed")))
    }
}
