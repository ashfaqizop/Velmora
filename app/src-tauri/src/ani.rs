//! Ani, the in-app assistant: the user's own Gemini key (BYOK), the streaming
//! call to Gemini, and the read-only lookups Ani's tools need beyond the
//! regular commands. The key stays on this side; the web UI never sees it.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use futures::StreamExt;
use serde::Serialize;
use serde_json::Value;
use tauri::ipc::Channel;
use tauri::State;
use universeapi::model::TitleSummary;

use crate::AppState;

const GEMINI: &str = "https://generativelanguage.googleapis.com/v1beta";
/// A long answer with thinking can run well past the engine client's 25 s default.
const GENERATE_TIMEOUT: Duration = Duration::from_secs(300);
const CHECK_TIMEOUT: Duration = Duration::from_secs(15);
/// Waits before retrying an overloaded or rate-limited request.
const RETRY_DELAYS: [Duration; 2] = [Duration::from_millis(1500), Duration::from_millis(4000)];

fn is_transient(status: u16) -> bool {
    matches!(status, 429 | 500 | 502 | 503 | 504)
}

pub struct AniStore {
    key_file: PathBuf,
    key: Mutex<Option<String>>,
    /// Requests the UI asked to stop; checked between streamed chunks.
    stopped: Mutex<HashSet<String>>,
}

impl AniStore {
    pub fn load(data_dir: &Path) -> Self {
        let key_file = data_dir.join("ani.key");
        let key = std::fs::read_to_string(&key_file).ok().map(|k| k.trim().to_string()).filter(|k| !k.is_empty());
        Self { key_file, key: Mutex::new(key), stopped: Mutex::new(HashSet::new()) }
    }

    fn key(&self) -> Option<String> {
        self.key.lock().unwrap().clone()
    }

    fn status(&self) -> KeyStatus {
        let key = self.key();
        KeyStatus { set: key.is_some(), hint: key.map(|k| hint(&k)) }
    }

    fn take_stop(&self, id: &str) -> bool {
        self.stopped.lock().unwrap().remove(id)
    }
}

#[derive(Serialize)]
pub struct KeyStatus {
    set: bool,
    /// The last four characters, so the user can tell which key is saved.
    hint: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    id: String,
    display_name: String,
}

fn hint(key: &str) -> String {
    let tail: String = key.chars().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect();
    format!("…{tail}")
}

fn valid_key(key: &str) -> bool {
    (20..200).contains(&key.len()) && key.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// Model ids go into the URL path, so only plain ids are accepted.
fn valid_model(model: &str) -> bool {
    model.starts_with("gemini-")
        && model.len() < 64
        && model.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '.'))
}

/// Turn a Gemini error body into one sentence for the chat.
fn gemini_error(status: u16, body: &str) -> String {
    let v: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    let message = v["error"]["message"].as_str().unwrap_or("").to_string();
    let reason = body.contains("API_KEY_INVALID") || body.contains("API key not valid");
    match status {
        400 if reason => "Gemini didn't accept this API key. Check it in Settings → Ani.".into(),
        401 | 403 => "Gemini refused the request. The key may be wrong or not allowed to use this model.".into(),
        404 => "Gemini doesn't know that model. Pick another one in Settings → Ani.".into(),
        429 => "Gemini's rate limit for this key was reached. Wait a minute and try again.".into(),
        500..=599 if !message.is_empty() => format!("Gemini is having trouble right now ({}). Try again in a moment.", message.trim_end_matches('.')),
        500..=599 => "Gemini is having trouble right now. Try again in a moment.".into(),
        _ if !message.is_empty() => format!("Gemini: {message}"),
        _ => format!("Gemini answered with HTTP {status}."),
    }
}

/// Pull complete server-sent events off the front of `buf` and return their
/// `data:` payloads. `buf` holds bytes with `\r` already removed.
fn take_events(buf: &mut Vec<u8>) -> Vec<String> {
    let mut out = Vec::new();
    while let Some(end) = buf.windows(2).position(|w| w == b"\n\n") {
        let event: Vec<u8> = buf.drain(..end + 2).collect();
        let text = String::from_utf8_lossy(&event);
        let data: Vec<&str> = text.lines().filter_map(|l| l.strip_prefix("data:")).map(str::trim_start).collect();
        if !data.is_empty() {
            out.push(data.join("\n"));
        }
    }
    out
}

async fn list_models(state: &AppState, key: &str) -> Result<Vec<ModelInfo>, String> {
    let resp = state
        .engine
        .ctx()
        .http
        .client()
        .get(format!("{GEMINI}/models"))
        .query(&[("pageSize", "200")])
        .header("x-goog-api-key", key)
        .timeout(CHECK_TIMEOUT)
        .send()
        .await
        .map_err(|e| format!("Couldn't reach Gemini: {e}"))?;
    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();
    if !(200..300).contains(&status) {
        return Err(gemini_error(status, &body));
    }
    let v: Value = serde_json::from_str(&body).map_err(|e| format!("Gemini: {e}"))?;
    let models = v["models"].as_array().cloned().unwrap_or_default();
    Ok(models
        .iter()
        .filter(|m| {
            m["supportedGenerationMethods"].as_array().is_some_and(|a| a.iter().any(|x| x == "generateContent"))
        })
        .filter_map(|m| {
            let id = m["name"].as_str()?.strip_prefix("models/")?.to_string();
            valid_model(&id).then(|| ModelInfo { display_name: m["displayName"].as_str().unwrap_or(&id).to_string(), id })
        })
        .collect())
}

#[tauri::command]
pub fn ani_key_status(ani: State<'_, AniStore>) -> KeyStatus {
    ani.status()
}

/// Save a key after checking it with Gemini, or remove the saved one (`None`).
#[tauri::command]
pub async fn ani_set_key(state: State<'_, AppState>, ani: State<'_, AniStore>, key: Option<String>) -> Result<KeyStatus, String> {
    match key.map(|k| k.trim().to_string()).filter(|k| !k.is_empty()) {
        None => {
            *ani.key.lock().unwrap() = None;
            if ani.key_file.exists() {
                std::fs::remove_file(&ani.key_file).map_err(|e| e.to_string())?;
            }
        }
        Some(key) => {
            if !valid_key(&key) {
                return Err("That doesn't look like a Gemini API key. Copy it again from Google AI Studio, without spaces.".into());
            }
            list_models(&state, &key).await?;
            if let Some(parent) = ani.key_file.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            std::fs::write(&ani.key_file, &key).map_err(|e| e.to_string())?;
            *ani.key.lock().unwrap() = Some(key);
        }
    }
    Ok(ani.status())
}

/// Gemini models this key can chat with, for the model picker.
#[tauri::command]
pub async fn ani_models(state: State<'_, AppState>, ani: State<'_, AniStore>) -> Result<Vec<ModelInfo>, String> {
    let key = ani.key().ok_or("Add a Gemini API key first.")?;
    list_models(&state, &key).await
}

/// Stream one `streamGenerateContent` call. Each Gemini chunk is sent to
/// `on_event` as JSON; the command returns when the stream ends or is stopped.
#[tauri::command]
pub async fn ani_generate(
    state: State<'_, AppState>,
    ani: State<'_, AniStore>,
    id: String,
    model: String,
    body: Value,
    on_event: Channel<Value>,
) -> Result<(), String> {
    let key = ani.key().ok_or("Ani needs a Gemini API key. Add one in Settings → Ani.")?;
    if !valid_model(&model) {
        return Err(format!("Unknown model \"{model}\"."));
    }
    // Overloaded and rate-limited answers are usually gone a few seconds
    // later, so try again before giving up. Nothing has streamed yet here.
    let mut attempt = 0;
    let resp = loop {
        let resp = state
            .engine
            .ctx()
            .http
            .client()
            .post(format!("{GEMINI}/models/{model}:streamGenerateContent"))
            .query(&[("alt", "sse")])
            .header("x-goog-api-key", &key)
            .timeout(GENERATE_TIMEOUT)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Couldn't reach Gemini: {e}"))?;
        let status = resp.status().as_u16();
        if (200..300).contains(&status) {
            break resp;
        }
        let text = resp.text().await.unwrap_or_default();
        if attempt < RETRY_DELAYS.len() && is_transient(status) && !ani.take_stop(&id) {
            tokio::time::sleep(RETRY_DELAYS[attempt]).await;
            attempt += 1;
            continue;
        }
        ani.take_stop(&id);
        return Err(gemini_error(status, &text));
    };

    let mut stream = resp.bytes_stream();
    let mut buf: Vec<u8> = Vec::new();
    let result = loop {
        if ani.take_stop(&id) {
            break Ok(());
        }
        let Some(chunk) = stream.next().await else { break Ok(()) };
        let chunk = match chunk {
            Ok(c) => c,
            Err(e) => break Err(format!("The connection to Gemini dropped: {e}")),
        };
        buf.extend(chunk.iter().copied().filter(|b| *b != b'\r'));
        for data in take_events(&mut buf) {
            let Ok(event) = serde_json::from_str::<Value>(&data) else { continue };
            if on_event.send(event).is_err() {
                return Ok(());
            }
        }
    };
    ani.take_stop(&id);
    result
}

#[tauri::command]
pub fn ani_stop(ani: State<'_, AniStore>, id: String) {
    ani.stopped.lock().unwrap().insert(id);
}

// ── Lookups for Ani's tools ────────────────────────────────────────────

fn pairs(query: HashMap<String, String>) -> Vec<(String, String)> {
    query.into_iter().collect()
}

/// Raw JSON from a read-only TMDB endpoint (people, collections, genres, ...).
#[tauri::command]
pub async fn tmdb_raw(state: State<'_, AppState>, path: String, query: HashMap<String, String>) -> Result<Value, String> {
    state.engine.tmdb().raw(&path, &pairs(query)).await.map_err(|e| e.to_string())
}

/// Titles from a TMDB list endpoint, as the summaries the rest of the app uses.
#[tauri::command]
pub async fn tmdb_list(
    state: State<'_, AppState>,
    path: String,
    query: HashMap<String, String>,
    field: Option<String>,
    kind: Option<String>,
) -> Result<Vec<TitleSummary>, String> {
    let field = field.unwrap_or_else(|| "results".into());
    let kind = kind.unwrap_or_else(|| "movie".into());
    state.engine.tmdb().list(&path, &pairs(query), &field, &kind).await.map_err(|e| e.to_string())
}

/// Anime from AniList filters (search, genres, sort, year, season, status, format).
#[tauri::command]
pub async fn anilist_find(state: State<'_, AppState>, filters: Value) -> Result<Vec<TitleSummary>, String> {
    state.engine.anilist().find(filters).await.map_err(|e| e.to_string())
}

/// A read-only AniList GraphQL query (airing schedules, staff, relations).
#[tauri::command]
pub async fn anilist_raw(state: State<'_, AppState>, query: String, variables: Value) -> Result<Value, String> {
    state.engine.anilist().raw(&query, variables).await.map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_split_across_chunks() {
        let mut buf = b"data: {\"a\":1}\n\ndata: {\"b\"".to_vec();
        assert_eq!(take_events(&mut buf), ["{\"a\":1}"]);
        buf.extend_from_slice(b":2}\n\n");
        assert_eq!(take_events(&mut buf), ["{\"b\":2}"]);
        assert!(buf.is_empty());
    }

    #[test]
    fn model_ids() {
        assert!(valid_model("gemini-3.5-flash"));
        assert!(valid_model("gemini-3.1-flash-lite"));
        assert!(!valid_model("gemini-3.5-flash:generateContent?x"));
        assert!(!valid_model("../files"));
    }

    #[test]
    fn transient_statuses() {
        assert!(is_transient(503) && is_transient(429));
        assert!(!is_transient(400) && !is_transient(404));
        let e = gemini_error(503, r#"{"error":{"message":"The model is overloaded."}}"#);
        assert_eq!(e, "Gemini is having trouble right now (The model is overloaded). Try again in a moment.");
    }

    #[test]
    fn key_hint() {
        assert_eq!(hint("AIzaSyExample1234"), "…1234");
        assert!(valid_key("AIzaSyD-abcdefghijklmnopqrstuvwxyz012"));
        assert!(!valid_key("AIza key with spaces and more text"));
    }
}
