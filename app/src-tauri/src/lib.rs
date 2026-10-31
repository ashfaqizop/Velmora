//! Velmora desktop/mobile shell: owns the UniverseAPI engine and exposes it
//! to the web UI as Tauri commands.

mod commands;
mod prefs;
mod runtime;
mod stream;
mod sync;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tauri::{App, Manager};
use universeapi::config::Registry;
use universeapi::{Engine, EngineOptions, Preferences};

/// TMDB credentials, injected from repo secrets by the release workflow.
/// Local and CI check builds have none and use Cinemeta instead.
const TMDB_READ_TOKEN: Option<&str> = option_env!("VELMORA_TMDB_READ_TOKEN");
const TMDB_API_KEY: Option<&str> = option_env!("VELMORA_TMDB_API_KEY");

pub struct AppState {
    pub engine: Arc<Engine>,
    /// Port of the loopback server mpv reads from (see `stream.rs`).
    pub stream_port: u16,
    /// Streams currently handed to the player, by id.
    pub sessions: Mutex<HashMap<String, StreamSession>>,
    /// Folder holding the first-launch components (see `runtime.rs`).
    pub runtime_dir: PathBuf,
    /// Where preferences are saved.
    pub prefs_file: PathBuf,
    /// AniList / Trakt accounts.
    pub sync: sync::SyncStore,
}

pub struct StreamSession {
    /// Proxy path registered for this stream.
    pub path: String,
    pub torrent: Option<Arc<universeapi::torrent::TorrentJob>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,universeapi=info".into()),
        )
        .try_init();

    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default().plugin(tauri_plugin_process::init());
    #[cfg(desktop)]
    {
        builder = builder
            .plugin(tauri_plugin_updater::Builder::new().build())
            .plugin(tauri_plugin_libmpv::init());
    }

    builder
        .setup(|app| {
            let paths = app.path();
            let config_dir = paths.app_config_dir()?;
            let prefs_file = prefs::path(&config_dir);
            let sync = sync::SyncStore::load(&config_dir);
            let runtime_dir = runtime::dir(&paths.app_local_data_dir()?);

            let engine = Arc::new(build_engine(app, prefs::load(&prefs_file))?);
            if runtime::is_ready(&runtime_dir) {
                runtime::activate(&runtime_dir, &engine);
            }
            let stream_port = tauri::async_runtime::block_on(stream::start(engine.clone()))?;
            app.manage(AppState {
                engine,
                stream_port,
                sessions: Mutex::new(HashMap::new()),
                runtime_dir,
                prefs_file,
                sync,
            });

            // The window starts hidden so the transparent frame never flashes;
            // the UI shows it on first paint. Fallback if the UI fails to load.
            if let Some(window) = app.get_webview_window("main") {
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    if !window.is_visible().unwrap_or(true) {
                        let _ = window.maximize();
                        let _ = window.show();
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::refresh_config,
            commands::home,
            commands::search_stream,
            commands::detail,
            commands::sources,
            commands::open_stream,
            commands::close_stream,
            commands::torrent_status,
            commands::runtime_ready,
            commands::install_runtime,
            commands::get_preferences,
            commands::set_preferences,
            commands::download_start,
            commands::downloads_list,
            commands::download_pause,
            commands::download_resume,
            commands::download_cancel,
            commands::downloads_clear,
            commands::reveal_file,
            commands::sync_status,
            commands::sync_disconnect,
            commands::anilist_login,
            commands::trakt_login,
            commands::sync_watched,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Velmora");
}

fn build_engine(app: &App, preferences: Preferences) -> Result<Engine, Box<dyn std::error::Error>> {
    let mut registry = Registry::default();
    registry.tmdb_read_token = TMDB_READ_TOKEN.filter(|s| !s.is_empty()).map(str::to_string);
    registry.tmdb_api_key = TMDB_API_KEY.filter(|s| !s.is_empty()).map(str::to_string);
    // UNIVERSEAPI_* env vars still win, which helps when debugging locally.
    let registry = registry.with_env_overrides();

    let paths = app.path();
    let cache_dir = paths.app_cache_dir().ok().map(|d| d.join("engine"));
    let download_dir: Option<PathBuf> = paths
        .video_dir()
        .map(|d| d.join("Velmora"))
        .or_else(|_| paths.app_data_dir().map(|d| d.join("downloads")))
        .ok();

    let opts = EngineOptions { preferences, registry, cache_dir, download_dir, ..Default::default() };
    // The engine's download manager spawns tasks, so build it inside the runtime.
    let engine = tauri::async_runtime::block_on(async move { Engine::new(opts) })?;
    Ok(engine)
}
