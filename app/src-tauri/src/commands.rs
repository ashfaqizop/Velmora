//! Commands the web UI calls through `invoke`.

use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use serde::Serialize;
use tauri::ipc::Channel;
use tauri::{AppHandle, State};
use universeapi::model::{AudioMode, ProviderRef, Rail, Source, Target, TitleDetail, TitleSummary};
use universeapi::download::{DownloadRequest, DownloadStatus};
use universeapi::torrent::TorrentProgress;
use universeapi::{Preferences, SearchBatch, SourcesReport};

use crate::{prefs, runtime, sync, AppState, StreamSession};

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

// ── First-launch components ────────────────────────────────────────────

#[tauri::command]
pub fn runtime_ready(state: State<'_, AppState>) -> bool {
    runtime::is_ready(&state.runtime_dir)
}

#[tauri::command]
pub async fn install_runtime(state: State<'_, AppState>, on_progress: Channel<runtime::Progress>) -> Result<(), String> {
    let engine = state.engine.clone();
    let dir = state.runtime_dir.clone();
    runtime::install(&engine, &dir, &on_progress).await
}

// ── Preferences ────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_preferences(state: State<'_, AppState>) -> Preferences {
    (*state.engine.preferences()).clone()
}

#[tauri::command]
pub fn set_preferences(state: State<'_, AppState>, preferences: Preferences) -> Result<(), String> {
    prefs::save(&state.prefs_file, &preferences)?;
    state.engine.set_preferences(preferences);
    Ok(())
}

// ── Downloads ──────────────────────────────────────────────────────────

#[tauri::command]
pub async fn download_start(
    state: State<'_, AppState>,
    source: Source,
    title: String,
    episode: Option<(u32, u32)>,
) -> Result<u64, String> {
    let downloads = state.engine.downloads().clone();
    Ok(downloads.enqueue(DownloadRequest { source, title, dest_dir: None, episode }).await)
}

#[tauri::command]
pub async fn downloads_list(state: State<'_, AppState>) -> Result<Vec<DownloadStatus>, String> {
    let downloads = state.engine.downloads().clone();
    Ok(downloads.list().await)
}

#[tauri::command]
pub async fn download_pause(state: State<'_, AppState>, id: u64) -> Result<(), String> {
    let downloads = state.engine.downloads().clone();
    downloads.pause(id).await;
    Ok(())
}

#[tauri::command]
pub async fn download_resume(state: State<'_, AppState>, id: u64) -> Result<(), String> {
    let downloads = state.engine.downloads().clone();
    downloads.resume(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn download_cancel(state: State<'_, AppState>, id: u64) -> Result<(), String> {
    let downloads = state.engine.downloads().clone();
    downloads.cancel(id).await;
    Ok(())
}

#[tauri::command]
pub async fn downloads_clear(state: State<'_, AppState>) -> Result<(), String> {
    let downloads = state.engine.downloads().clone();
    downloads.clear_finished().await;
    Ok(())
}

/// Show a finished download in the file manager. Only paths inside the
/// download folder are accepted.
#[tauri::command]
pub fn reveal_file(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let file = std::path::Path::new(&path).canonicalize().map_err(|e| e.to_string())?;
    let root = state.engine.download_dir().canonicalize().map_err(|e| e.to_string())?;
    if !file.starts_with(&root) {
        return Err("not a Velmora download".into());
    }
    #[cfg(windows)]
    {
        std::process::Command::new("explorer").arg(format!("/select,{}", file.display())).spawn().map_err(|e| e.to_string())?;
    }
    #[cfg(not(windows))]
    {
        let folder = file.parent().unwrap_or(&root);
        std::process::Command::new("xdg-open").arg(folder).spawn().map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ── AniList / Trakt ────────────────────────────────────────────────────

#[tauri::command]
pub fn sync_status(state: State<'_, AppState>) -> sync::SyncStatus {
    state.sync.status()
}

#[tauri::command]
pub fn sync_disconnect(app: AppHandle, state: State<'_, AppState>, service: String) {
    state.sync.disconnect(&service);
    let _ = tauri::Emitter::emit(&app, "sync-changed", ());
}

#[tauri::command]
pub fn anilist_login(app: AppHandle) -> Result<(), String> {
    sync::anilist_login(&app)
}

#[tauri::command]
pub fn trakt_login(app: AppHandle) -> Result<(), String> {
    sync::trakt_login(&app)
}

/// Mark a title or episode watched on every connected service.
#[tauri::command]
pub async fn sync_watched(app: AppHandle, title: TitleSummary, target: Target) -> Result<Vec<String>, String> {
    Ok(sync::watched(&app, &title, target).await.into_iter().map(str::to_string).collect())
}

/// Open an https link in the default browser (title pages: IMDb, TMDB, homepage).
#[tauri::command]
pub fn open_external(app: AppHandle, url: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let parsed = tauri::Url::parse(&url).map_err(|e| e.to_string())?;
    if parsed.scheme() != "https" {
        return Err("only https links can be opened".into());
    }
    app.opener().open_url(parsed.as_str(), None::<&str>).map_err(|e| e.to_string())
}
