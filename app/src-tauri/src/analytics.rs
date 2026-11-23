//! Anonymous usage statistics: the app's daily batch of counts goes to the
//! site, and the most-watched titles come back for the "Popular on Velmora"
//! row. Nothing here carries an ID or an account (see site/api/_lib/stats.js).

use std::time::Duration;

use serde_json::Value;
use tauri::State;

use crate::license::SITE;
use crate::AppState;

const TIMEOUT: Duration = Duration::from_secs(15);

/// Send one finished day of counts. True when the batch is dealt with and can be
/// forgotten: accepted, or refused as unusable (too old, malformed). False when
/// it should be tried again later (offline, busy).
#[tauri::command]
pub async fn analytics_report(state: State<'_, AppState>, batch: Value) -> Result<bool, String> {
    let resp = state
        .engine
        .ctx()
        .http
        .client()
        .post(format!("{SITE}/api/stats/report"))
        .header("Content-Type", "application/json")
        .body(batch.to_string())
        .timeout(TIMEOUT)
        .send()
        .await;
    Ok(match resp {
        Ok(r) => matches!(r.status().as_u16(), 200 | 400),
        Err(_) => false,
    })
}

/// The most-watched titles this week: `{ people, titles: [{ k: "movie/603", people }] }`.
#[tauri::command]
pub async fn analytics_trending(state: State<'_, AppState>) -> Result<Value, String> {
    let resp = state
        .engine
        .ctx()
        .http
        .client()
        .get(format!("{SITE}/api/stats/trending?days=7&limit=24"))
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
