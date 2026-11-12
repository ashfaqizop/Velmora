// SPDX-License-Identifier: GPL-3.0-or-later
//! Songs saved ahead of time, so the next ones start at once.
//!
//! The player asks for the next few songs in the queue; each is resolved with
//! yt-dlp and downloaded to `<app local data>/music/cache` as
//! `<videoId>.<quality>.<ext>`. googlevideo slows a whole-file download to
//! playback speed, so the file comes in 8 MB ranges, which it serves at full
//! speed. The folder has a size cap; the songs played longest ago go first.
//!
//! These files are a cache, not downloads: Velmora may delete them at any
//! time, and they never show in the library.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use serde::Serialize;
use tauri::State;
use tokio::io::AsyncWriteExt;
use universeapi::music::{AudioQuality, StreamMode};

use crate::AppState;

/// One request's worth; googlevideo throttles larger ones.
const CHUNK: u64 = 8 * 1024 * 1024;
/// A song bigger than this isn't worth caching (a long mix).
const MAX_FILE: u64 = 80 * 1024 * 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

pub struct MusicCache {
    dir: PathBuf,
    /// Songs being fetched right now.
    busy: Mutex<HashSet<String>>,
}

impl MusicCache {
    pub fn new(dir: PathBuf) -> Self {
        let _ = std::fs::create_dir_all(&dir);
        // A fetch cut off by a quit leaves a .part; it's useless now.
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for e in entries.flatten() {
                if e.path().extension().is_some_and(|x| x == "part") {
                    let _ = std::fs::remove_file(e.path());
                }
            }
        }
        Self { dir, busy: Mutex::new(HashSet::new()) }
    }

    /// The cached file for a song, at any quality, best first.
    fn find(&self, video_id: &str) -> Option<PathBuf> {
        let prefix = format!("{video_id}.");
        let mut found: Vec<PathBuf> = std::fs::read_dir(&self.dir)
            .ok()?
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                name.starts_with(&prefix) && !name.ends_with(".part")
            })
            .collect();
        let rank = |p: &PathBuf| {
            let name = p.to_string_lossy();
            if name.contains(".high.") {
                0
            } else if name.contains(".normal.") {
                1
            } else {
                2
            }
        };
        found.sort_by_key(rank);
        found.into_iter().next()
    }

    /// Every finished file with its size and last use, oldest use first.
    fn files(&self) -> Vec<(PathBuf, u64, SystemTime)> {
        let mut list: Vec<(PathBuf, u64, SystemTime)> = std::fs::read_dir(&self.dir)
            .map(|rd| {
                rd.flatten()
                    .filter_map(|e| {
                        let p = e.path();
                        if p.extension().is_some_and(|x| x == "part") {
                            return None;
                        }
                        let m = e.metadata().ok()?;
                        m.is_file().then(|| (p, m.len(), m.modified().unwrap_or(SystemTime::UNIX_EPOCH)))
                    })
                    .collect()
            })
            .unwrap_or_default();
        list.sort_by_key(|(_, _, t)| *t);
        list
    }

    /// Delete the songs used longest ago until the folder fits in `cap`.
    fn trim(&self, cap: u64, keep: &Path) {
        let files = self.files();
        let mut total: u64 = files.iter().map(|(_, n, _)| n).sum();
        for (path, len, _) in files {
            if total <= cap {
                break;
            }
            if path == keep {
                continue;
            }
            if std::fs::remove_file(&path).is_ok() {
                total = total.saturating_sub(len);
            }
        }
    }
}

/// Mark a file as just used, so the size cap keeps it longest.
fn touch(path: &Path) {
    if let Ok(f) = std::fs::File::options().append(true).open(path) {
        let _ = f.set_modified(SystemTime::now());
    }
}

/// Drops the song from the busy set however the fetch ends.
struct Busy<'a>(&'a Mutex<HashSet<String>>, String);
impl Drop for Busy<'_> {
    fn drop(&mut self) {
        if let Ok(mut set) = self.0.lock() {
            set.remove(&self.1);
        }
    }
}

/// The cached copy of a song, if there is one. Playing it counts as a use.
#[tauri::command]
pub async fn music_cached(cache: State<'_, MusicCache>, video_id: String) -> Result<Option<String>, String> {
    Ok(cache.find(&video_id).map(|p| {
        touch(&p);
        p.to_string_lossy().into_owned()
    }))
}

/// Save a song ahead of time. Returns its path; does nothing if it's already
/// there or being fetched. `cap_mb` is the folder's size cap.
#[tauri::command]
pub async fn music_precache(
    state: State<'_, AppState>,
    cache: State<'_, MusicCache>,
    video_id: String,
    quality: AudioQuality,
    cap_mb: u64,
) -> Result<Option<String>, String> {
    if let Some(p) = cache.find(&video_id) {
        return Ok(Some(p.to_string_lossy().into_owned()));
    }
    {
        let mut busy = cache.busy.lock().map_err(|_| "cache is busy".to_string())?;
        if !busy.insert(video_id.clone()) {
            return Ok(None);
        }
    }
    let _busy = Busy(&cache.busy, video_id.clone());

    let r = state
        .engine
        .music()
        .resolve_with(&video_id, StreamMode::Audio, 0, quality)
        .await
        .map_err(|e| e.to_string())?;
    if r.filesize.is_some_and(|n| n > MAX_FILE) {
        return Ok(None);
    }
    let q = match quality {
        AudioQuality::Low => "low",
        AudioQuality::Normal => "normal",
        AudioQuality::High => "high",
    };
    let ext = r.ext.as_deref().filter(|e| e.chars().all(|c| c.is_ascii_alphanumeric()) && e.len() <= 5).unwrap_or("webm");
    let path = cache.dir.join(format!("{video_id}.{q}.{ext}"));
    let part = cache.dir.join(format!("{video_id}.{q}.{ext}.part"));

    let client = state.engine.ctx().http.client().clone();
    let mut file = tokio::fs::File::create(&part).await.map_err(|e| e.to_string())?;
    let mut have: u64 = 0;
    let result: Result<(), String> = async {
        loop {
            let end = have + CHUNK - 1;
            let mut req = client.get(&r.audio_url).timeout(REQUEST_TIMEOUT).header("Range", format!("bytes={have}-{end}"));
            for (k, v) in &r.headers {
                req = req.header(k.as_str(), v.as_str());
            }
            let resp = req.send().await.map_err(|e| e.to_string())?;
            let status = resp.status().as_u16();
            if status == 416 {
                break; // Asked past the end: done.
            }
            if !(200..300).contains(&status) {
                return Err(format!("googlevideo answered {status}"));
            }
            let whole = status == 200; // The range was ignored: this is everything.
            let body = resp.bytes().await.map_err(|e| e.to_string())?;
            file.write_all(&body).await.map_err(|e| e.to_string())?;
            have += body.len() as u64;
            if have > MAX_FILE {
                return Err("too big to cache".into());
            }
            if whole || (body.len() as u64) < CHUNK {
                break;
            }
        }
        file.flush().await.map_err(|e| e.to_string())
    }
    .await;
    drop(file);
    if let Err(e) = result {
        let _ = tokio::fs::remove_file(&part).await;
        return Err(e);
    }
    if have == 0 {
        let _ = tokio::fs::remove_file(&part).await;
        return Err("nothing came back".into());
    }
    tokio::fs::rename(&part, &path).await.map_err(|e| e.to_string())?;
    cache.trim(cap_mb.max(50) * 1024 * 1024, &path);
    Ok(Some(path.to_string_lossy().into_owned()))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheSize {
    songs: usize,
    bytes: u64,
}

#[tauri::command]
pub async fn music_precache_size(cache: State<'_, MusicCache>) -> Result<CacheSize, String> {
    let files = cache.files();
    Ok(CacheSize { songs: files.len(), bytes: files.iter().map(|(_, n, _)| n).sum() })
}

/// Empty the folder. A file mpv has open stays until it lets go.
#[tauri::command]
pub async fn music_precache_clear(cache: State<'_, MusicCache>) -> Result<(), String> {
    for (path, _, _) in cache.files() {
        let _ = std::fs::remove_file(path);
    }
    Ok(())
}
