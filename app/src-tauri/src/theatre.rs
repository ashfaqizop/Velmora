//! Theatre: the app helps check that drafted screenings can be played. It asks
//! the site which titles need checking and reports, anonymously, whether it
//! found sources for each (see site/api/theatre/[action].js). The lineup,
//! tickets and chat go straight to Supabase from the frontend.

use std::time::Duration;

use serde_json::{json, Value};
use tauri::State;

use crate::license::SITE;
use crate::AppState;

const TIMEOUT: Duration = Duration::from_secs(15);

/// Titles waiting on source checks: `{ titles: [{ k: "movie/603", name, year }] }`.
#[tauri::command]
pub async fn theatre_probes(state: State<'_, AppState>) -> Result<Value, String> {
    let resp = state
        .engine
        .ctx()
        .http
        .client()
        .get(format!("{SITE}/api/theatre/probes"))
        .timeout(TIMEOUT)
        .send()
        .await
        .map_err(|_| "offline".to_string())?;
    if !resp.status().is_success() {
        return Err("unavailable".into());
    }
    let text = resp.text().await.map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

/// Report whether this PC found sources for a title. No ID, no account.
#[tauri::command]
pub async fn theatre_probe(state: State<'_, AppState>, k: String, found: bool) -> Result<bool, String> {
    let resp = state
        .engine
        .ctx()
        .http
        .client()
        .post(format!("{SITE}/api/theatre/probe"))
        .header("Content-Type", "application/json")
        .body(json!({ "k": k, "found": found }).to_string())
        .timeout(TIMEOUT)
        .send()
        .await
        .map_err(|_| "offline".to_string())?;
    Ok(resp.status().is_success())
}
