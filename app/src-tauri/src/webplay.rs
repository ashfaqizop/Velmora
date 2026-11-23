// SPDX-License-Identifier: GPL-3.0-or-later
//! Support for when mpv can't run on a PC.
//!
//! The built-in player is WebView2's own media engine. It plays from the
//! loopback server (`stream.rs`), which adds the headers a stream needs and
//! allows CORS, so Web Audio can run the EQ on it.
//!
//! mpv also writes a log here for each start, so a failed start can say why.

use std::collections::BTreeMap;
use std::path::PathBuf;

use tauri::{AppHandle, Manager, State};

use crate::AppState;

/// A remote stream for the built-in player: a loopback URL that fetches it
/// with `headers`.
#[tauri::command]
pub fn web_media_url(state: State<'_, AppState>, url: String, headers: Vec<(String, String)>) -> Result<String, String> {
    if url.starts_with("http://127.0.0.1:") {
        return Ok(url);
    }
    let headers: BTreeMap<String, String> = headers.into_iter().collect();
    let path = state.engine.proxy().register_url(&url, headers).map_err(|e| e.to_string())?;
    Ok(format!("http://127.0.0.1:{}{}", state.stream_port, path))
}

/// A file on this PC for the built-in player. Only media files.
#[tauri::command]
pub fn web_file_url(state: State<'_, AppState>, path: String) -> Result<String, String> {
    let file = PathBuf::from(&path);
    if !file.is_file() {
        return Err("This file is no longer on disk.".into());
    }
    let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    if !matches!(ext.as_str(), "webm" | "weba" | "m4a" | "mp4" | "mp3" | "opus" | "ogg" | "oga" | "flac" | "wav" | "mkv") {
        return Err("That isn't a media file.".into());
    }
    let token = state.sealed.register_plain(file)?;
    Ok(format!("http://127.0.0.1:{}/f/{token}", state.stream_port))
}

fn log_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?.join("logs");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// Where mpv writes its log for one way of starting (`profile`).
#[tauri::command]
pub fn player_log_path(app: AppHandle, profile: String) -> Result<String, String> {
    let name: String = profile.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-').take(32).collect();
    Ok(log_dir(&app)?.join(format!("mpv-{name}.log")).to_string_lossy().into_owned())
}

/// The end of that log: the last lines, and every error line before them.
#[tauri::command]
pub fn player_log(app: AppHandle, profile: String) -> Result<String, String> {
    let path = PathBuf::from(player_log_path(app, profile)?);
    let Ok(bytes) = std::fs::read(&path) else {
        return Ok(String::new());
    };
    let text = String::from_utf8_lossy(&bytes);
    let lines: Vec<&str> = text.lines().collect();
    let tail_from = lines.len().saturating_sub(40);
    let mut out: Vec<&str> = lines[..tail_from]
        .iter()
        .filter(|l| {
            let l = l.to_ascii_lowercase();
            l.contains("[e]") || l.contains("[f]") || l.contains("error") || l.contains("fail")
        })
        .copied()
        .collect();
    if out.len() > 40 {
        out.drain(..out.len() - 40);
    }
    out.extend_from_slice(&lines[tail_from..]);
    let mut s = out.join("\n");
    if s.len() > 12_000 {
        let cut = s.len() - 12_000;
        let at = (cut..s.len()).find(|&i| s.is_char_boundary(i)).unwrap_or(s.len());
        s = s[at..].to_string();
    }
    Ok(s)
}
