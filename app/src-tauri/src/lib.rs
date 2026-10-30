//! Velmora desktop/mobile shell: owns the UniverseAPI engine and exposes it
//! to the web UI as Tauri commands.

mod commands;

use std::path::PathBuf;
use std::sync::Arc;

use tauri::{App, Manager};
use universeapi::config::Registry;
use universeapi::{Engine, EngineOptions};

/// TMDB credentials, injected from repo secrets by the release workflow.
/// Local and CI check builds have none and use Cinemeta instead.
const TMDB_READ_TOKEN: Option<&str> = option_env!("VELMORA_TMDB_READ_TOKEN");
const TMDB_API_KEY: Option<&str> = option_env!("VELMORA_TMDB_API_KEY");

pub struct AppState {
    pub engine: Arc<Engine>,
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
        builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    }

    builder
        .setup(|app| {
            let engine = build_engine(app)?;
            app.manage(AppState { engine: Arc::new(engine) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::refresh_config,
            commands::home,
            commands::search_stream,
            commands::detail,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Velmora");
}

fn build_engine(app: &App) -> Result<Engine, Box<dyn std::error::Error>> {
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

    let opts = EngineOptions { registry, cache_dir, download_dir, ..Default::default() };
    // The engine's download manager spawns tasks, so build it inside the runtime.
    let engine = tauri::async_runtime::block_on(async move { Engine::new(opts) })?;
    Ok(engine)
}
