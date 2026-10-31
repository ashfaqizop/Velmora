//! Commands the web UI calls through `invoke`.

use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use serde::Serialize;
use tauri::ipc::Channel;
use tauri::{AppHandle, State};
use universeapi::model::{AudioMode, ProviderRef, Rail, Source, Target, TitleDetail, TitleSummary};
use universeapi::torrent::TorrentProgress;
use universeapi::{SearchBatch, SourcesReport};

use crate::{AppState, StreamSession};

const CONFIG_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    version: String,
    platform: &'static str,
    tmdb: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigStatus {
    /// True when the remote registry was fetched and applied.
    updated: bool,
    /// Why the refresh failed; the engine keeps its current registry.
    message: Option<String>,
}

#[tauri::command]
pub fn app_info(app: AppHandle, state: State<'_, AppState>) -> AppInfo {
    AppInfo {
        version: app.package_info().version.to_string(),
        platform: std::env::consts::OS,
        tmdb: state.engine.registry().has_tmdb(),
    }
}

/// Pull the remote registry (mirrors, kill-switches, host aliases).
#[tauri::command]
pub async fn refresh_config(state: State<'_, AppState>) -> Result<ConfigStatus, String> {
    let engine = state.engine.clone();
    let status = match tokio::time::timeout(CONFIG_TIMEOUT, engine.refresh_registry_default()).await {
        Ok(Ok(())) => ConfigStatus { updated: true, message: None },
        Ok(Err(e)) => {
            tracing::warn!("config refresh failed: {e}");
            ConfigStatus { updated: false, message: Some(e.to_string()) }
        }
        Err(_) => ConfigStatus { updated: false, message: Some("timed out".into()) },
    };
    Ok(status)
}

#[tauri::command]
pub async fn home(state: State<'_, AppState>) -> Result<Vec<Rail>, String> {
    let engine = state.engine.clone();
    Ok(engine.home().await)
}

/// Streams one batch per provider over `on_batch` as each one answers.
#[tauri::command]
pub async fn search_stream(
    state: State<'_, AppState>,
    query: String,
    on_batch: Channel<SearchBatch>,
) -> Result<(), String> {
    let engine = state.engine.clone();
    let mut batches = Box::pin(engine.search_stream(&query));
    while let Some(batch) = batches.next().await {
        // The UI dropped the channel (a newer query started): stop early.
        if on_batch.send(batch).is_err() {
            break;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn detail(state: State<'_, AppState>, origin: ProviderRef) -> Result<TitleDetail, String> {
    let engine = state.engine.clone();
    engine.detail(&origin).await.map_err(|e| e.to_string())
}

/// Ranked, probed sources for a movie or one episode.
#[tauri::command]
pub async fn sources(
    state: State<'_, AppState>,
    title: TitleSummary,
    target: Target,
    mode: AudioMode,
) -> Result<SourcesReport, String> {
    let engine = state.engine.clone();
    Ok(engine.sources(&title, target, mode).await)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleTrack {
    label: String,
    lang: Option<String>,
    url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenedStream {
    /// Handle for `close_stream` / `torrent_status`.
    id: String,
    /// Loopback URL to hand to mpv.
    url: String,
    torrent: bool,
    subtitles: Vec<SubtitleTrack>,
}

/// Register a source with the media proxy and return a URL mpv can play.
/// Torrents are added to the session first (metadata can take a while).
#[tauri::command]
pub async fn open_stream(
    state: State<'_, AppState>,
    source: Source,
    episode: Option<(u32, u32)>,
) -> Result<OpenedStream, String> {
    let engine = state.engine.clone();
    let (path, torrent) = if source.is_torrent() {
        let magnet = source.torrent.as_ref().map(|t| t.magnet.clone()).unwrap_or_else(|| source.url.clone());
        let te = engine.torrents().await.map_err(|e| e.to_string())?;
        let job = Arc::new(te.add(&magnet, episode).await.map_err(|e| e.to_string())?);
        (engine.proxy().register_torrent(job.clone()), Some(job))
    } else {
        (engine.proxy().register(&source).map_err(|e| e.to_string())?, None)
    };

    let id = path.clone();
    let url = format!("http://127.0.0.1:{}{}", state.stream_port, path);
    let subtitles = source
        .subtitles
        .iter()
        .filter_map(|t| Some(SubtitleTrack { label: t.label.clone(), lang: t.lang.clone(), url: t.url.clone()? }))
        .collect();
    let is_torrent = torrent.is_some();
    if let Ok(mut sessions) = state.sessions.lock() {
        sessions.insert(id.clone(), StreamSession { path, torrent });
    }
    Ok(OpenedStream { id, url, torrent: is_torrent, subtitles })
}

/// Forget a stream; a streaming torrent is removed along with its pieces.
#[tauri::command]
pub async fn close_stream(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let session = state.sessions.lock().ok().and_then(|mut s| s.remove(&id));
    let Some(session) = session else { return Ok(()) };
    let engine = state.engine.clone();
    engine.proxy().unregister(&session.path);
    if let Some(job) = session.torrent {
        let te = engine.torrents().await.map_err(|e| e.to_string())?;
        // The proxy dropped its handle above, so this is normally the last one.
        match Arc::try_unwrap(job) {
            Ok(job) => te.remove(job, true).await.map_err(|e| e.to_string())?,
            Err(job) => te.pause(&job).await.map_err(|e| e.to_string())?,
        }
    }
    Ok(())
}

/// Download progress of a streaming torrent (for the buffering overlay).
#[tauri::command]
pub async fn torrent_status(state: State<'_, AppState>, id: String) -> Result<Option<TorrentProgress>, String> {
    let job = state.sessions.lock().ok().and_then(|s| s.get(&id).and_then(|x| x.torrent.clone()));
    let Some(job) = job else { return Ok(None) };
    let engine = state.engine.clone();
    let te = engine.torrents().await.map_err(|e| e.to_string())?;
    Ok(Some(te.progress(&job)))
}
