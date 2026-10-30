//! Commands the web UI calls through `invoke`.

use std::time::Duration;

use futures::StreamExt;
use serde::Serialize;
use tauri::ipc::Channel;
use tauri::{AppHandle, State};
use universeapi::model::{ProviderRef, Rail, TitleDetail};
use universeapi::SearchBatch;

use crate::AppState;

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
