//! Ads for the Public tier: the remote rules and the per-slot view counts.
//!
//! The rules live on the site (`/ads.json`) so they can be toned down or
//! switched off without an app update. The UI holds the defaults and applies
//! them; this side only fetches, because the webview's CSP keeps it off the
//! network. View counts are anonymous totals per slot, nothing else.

use std::collections::HashMap;
use std::time::Duration;

use serde_json::{json, Value};
use tauri::State;

use crate::license::{LicenseStore, SITE};
use crate::AppState;

const TIMEOUT: Duration = Duration::from_secs(10);

/// The current ad rules from the site, or null if it can't be reached.
#[tauri::command]
pub async fn ads_config(state: State<'_, AppState>) -> Result<Value, String> {
    let resp = state.engine.ctx().http.client().get(format!("{SITE}/ads.json")).timeout(TIMEOUT).send().await;
    let Ok(resp) = resp else { return Ok(Value::Null) };
    if !resp.status().is_success() {
        return Ok(Value::Null);
    }
    let body: Value = serde_json::from_str(&resp.text().await.unwrap_or_default()).unwrap_or(Value::Null);
    Ok(if body.is_object() { body } else { Value::Null })
}

/// Send how many ads each slot showed. Max installs send nothing.
#[tauri::command]
pub async fn ads_report(
    state: State<'_, AppState>,
    lic: State<'_, LicenseStore>,
    counts: HashMap<String, u32>,
) -> Result<(), String> {
    if lic.status().tier() == "max" || counts.is_empty() {
        return Ok(());
    }
    let _ = state
        .engine
        .ctx()
        .http
        .client()
        .post(format!("{SITE}/api/ads/stats"))
        .header("Content-Type", "application/json")
        .body(json!({ "counts": counts }).to_string())
        .timeout(TIMEOUT)
        .send()
        .await;
    Ok(())
}
