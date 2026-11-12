//! Velmora desktop/mobile shell: owns the UniverseAPI engine and exposes it
//! to the web UI as Tauri commands.

mod ads;
mod ani;
mod commands;
mod discord;
mod license;
mod music;
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
/// Sports keys, injected the same way. Without them the engine uses TheSportsDB's
/// free test key and skips football-data.org.
const SPORTSDB_KEY: Option<&str> = option_env!("VELMORA_SPORTSDB_KEY");
const FOOTBALL_DATA_TOKEN: Option<&str> = option_env!("VELMORA_FOOTBALL_DATA_TOKEN");

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
    /// Vault key holder and the vault files handed to the player.
    pub sealed: Arc<stream::Sealed>,
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
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init());
    #[cfg(desktop)]
    {
        builder = builder
            .plugin(tauri_plugin_updater::Builder::new().build())
            .plugin(tauri_plugin_libmpv::init());
    }

    builder
        .setup(|app| {
            create_main_window(app)?;
            let paths = app.path();
            let config_dir = paths.app_config_dir()?;
            let prefs_file = prefs::path(&config_dir);
            let sync = sync::SyncStore::load(&config_dir);
            let runtime_dir = runtime::dir(&paths.app_local_data_dir()?);

            let engine = Arc::new(build_engine(app, prefs::load(&prefs_file))?);
            if runtime::is_ready(&runtime_dir) {
                runtime::activate(&runtime_dir, &engine);
            }
            // Downloads are stored as vault files, playable only with this install's key.
            let key = stream::load_key(&paths.app_local_data_dir()?.join("vault.key")).map_err(std::io::Error::other)?;
            let vault = Arc::new(universeapi::vault::Vault::new(key));
            engine.downloads().set_vault(Some(vault.clone()));
            let sealed = Arc::new(stream::Sealed::new(vault));
            let stream_port = tauri::async_runtime::block_on(stream::start(engine.clone(), sealed.clone()))?;
            // Downloads don't survive a restart, so any .part left in the folder is dead weight.
            let downloads = engine.downloads().clone();
            tauri::async_runtime::spawn(async move { downloads.remove_orphaned_parts().await });
            app.manage(ani::AniStore::load(&paths.app_local_data_dir()?));
            app.manage(license::LicenseStore::load(&paths.app_local_data_dir()?));
            let discord = discord::Discord::new();
            discord::start(app.handle().clone(), discord.clone());
            app.manage(discord);
            let state = AppState {
                engine,
                stream_port,
                sessions: Mutex::new(HashMap::new()),
                runtime_dir,
                prefs_file,
                sync,
                sealed,
            };
            // YouTube Music: restore the saved session (Music needs sign-in; nothing else does).
            let music_store = music::setup(app.handle(), &state, &paths.app_local_data_dir()?);
            app.manage(state);
            app.manage(music_store);

            // The window starts hidden so the transparent frame never flashes;
            // the UI shows it on first paint. Fallback if the UI fails to load.
            #[cfg(desktop)]
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
        // Closing the main window quits Velmora. Without this the hidden Music
        // keeper window keeps the process (and mpv) alive with nothing on screen.
        .on_window_event(|window, event| {
            if window.label() == "main" && matches!(event, tauri::WindowEvent::Destroyed) {
                window.app_handle().exit(0);
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::refresh_config,
            commands::home,
            commands::search_stream,
            commands::search_narrow,
            commands::sports_day,
            commands::sports_live,
            commands::sports_upcoming,
            commands::sports_results,
            commands::sports_lookup,
            commands::sports_watch,
            commands::tv_channels,
            commands::tv_channel,
            commands::tv_countries,
            commands::tv_categories,
            commands::tv_check,
            commands::tv_sources,
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
            commands::downloads_library,
            commands::download_existing,
            commands::download_folder,
            commands::open_download_folder,
            commands::set_download_dir,
            commands::open_local,
            commands::default_download_folder,
            commands::sync_status,
            commands::sync_disconnect,
            commands::anilist_login,
            commands::trakt_login,
            commands::sync_watched,
            commands::open_external,
            ani::ani_key_status,
            ani::ani_set_key,
            ani::ani_models,
            ani::ani_generate,
            ani::ani_stop,
            ani::tmdb_raw,
            ani::tmdb_list,
            ani::anilist_find,
            ani::anilist_raw,
            license::license_status,
            license::license_activate,
            license::license_refresh,
            license::license_remove,
            ads::ads_config,
            ads::ads_report,
            music::music_status,
            music::music_account,
            music::music_sign_in,
            music::music_sign_out,
            music::music_home,
            music::music_feed,
            music::music_search,
            music::music_album,
            music::music_artist,
            music::music_playlist,
            music::music_playlist_more,
            music::music_queue,
            music::music_radio,
            music::music_queue_more,
            music::music_lyrics,
            music::music_liked,
            music::music_library,
            music::music_own_playlists,
            music::music_rate,
            music::music_playlist_add,
            music::music_playlist_remove,
            music::music_playlist_create,
            music::music_subscribe,
            music::music_stream,
            music::music_ytdlp_status,
            music::music_ytdlp_update,
            music::music_download,
            music::music_downloads,
            music::music_download_delete,
            discord::discord_configure,
            discord::discord_set,
            discord::discord_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Velmora");
}

/// The main window is made here rather than from the config so that links an
/// ad (or a trailer) tries to open in a new window go to the default browser
/// instead of a bare webview window.
fn create_main_window(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|w| w.label == "main")
        .cloned()
        .ok_or("the main window is missing from tauri.conf.json")?;
    #[allow(unused_mut)]
    let mut builder = tauri::WebviewWindowBuilder::from_config(app.handle(), &config)?;
    #[cfg(desktop)]
    {
        let handle = app.handle().clone();
        builder = builder.on_new_window(move |url, _features| {
            if matches!(url.scheme(), "https" | "http") {
                use tauri_plugin_opener::OpenerExt;
                let _ = handle.opener().open_url(url.as_str(), None::<&str>);
            }
            tauri::webview::NewWindowResponse::Deny
        });
    }
    builder.build()?;
    Ok(())
}

fn build_engine(app: &App, preferences: Preferences) -> Result<Engine, Box<dyn std::error::Error>> {
    let mut registry = Registry::default();
    registry.tmdb_read_token = TMDB_READ_TOKEN.filter(|s| !s.is_empty()).map(str::to_string);
    registry.tmdb_api_key = TMDB_API_KEY.filter(|s| !s.is_empty()).map(str::to_string);
    registry.thesportsdb_key = SPORTSDB_KEY.filter(|s| !s.is_empty()).map(str::to_string);
    registry.football_data_token = FOOTBALL_DATA_TOKEN.filter(|s| !s.is_empty()).map(str::to_string);
    // UNIVERSEAPI_* env vars still win, which helps when debugging locally.
    let registry = registry.with_env_overrides();

    let paths = app.path();
    let cache_dir = paths.app_cache_dir().ok().map(|d| d.join("engine"));
    let download_dir: Option<PathBuf> = paths
        .video_dir()
        .map(|d| d.join("Velmora"))
        .or_else(|_| paths.app_data_dir().map(|d| d.join("downloads")))
        .ok();

    // yt-dlp for Music lives next to the other first-launch components.
    let tools_dir = paths.app_local_data_dir().ok().map(|d| d.join("tools"));
    let opts = EngineOptions { preferences, registry, cache_dir, download_dir, tools_dir, ..Default::default() };
    // The engine's download manager spawns tasks, so build it inside the runtime.
    let engine = tauri::async_runtime::block_on(async move { Engine::new(opts) })?;
    Ok(engine)
}
