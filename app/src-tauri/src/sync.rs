//! Watch-progress sync: AniList for anime, Trakt for movies and series.
//!
//! Client ids come from repo secrets at build time; a build without them
//! simply reports the service as unavailable. Sign-in tokens are kept in
//! `sync.json` in the app config folder.
//!
//! * AniList uses the implicit grant. Sign-in runs in a small WebView window
//!   and the token is read from the redirect's URL fragment.
//! * Trakt uses the device flow: the UI shows a short code, the user enters it
//!   at trakt.tv/activate, and the UI polls [`trakt_poll`] until it's done.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use universeapi::model::{Target, TitleSummary};
use universeapi::Engine;

use crate::AppState;

const ANILIST_CLIENT_ID: Option<&str> = option_env!("VELMORA_ANILIST_CLIENT_ID");
const TRAKT_CLIENT_ID: Option<&str> = option_env!("VELMORA_TRAKT_CLIENT_ID");
const TRAKT_CLIENT_SECRET: Option<&str> = option_env!("VELMORA_TRAKT_CLIENT_SECRET");

const ANILIST_GRAPHQL: &str = "https://graphql.anilist.co";
const TRAKT_API: &str = "https://api.trakt.tv";
const TRAKT_REDIRECT: &str = "urn:ietf:wg:oauth:2.0:oob";
const LOGIN_WINDOW: &str = "anilist-login";

fn set(v: Option<&'static str>) -> Option<&'static str> {
    v.filter(|s| !s.trim().is_empty())
}

fn trakt_keys() -> Option<(&'static str, &'static str)> {
    Some((set(TRAKT_CLIENT_ID)?, set(TRAKT_CLIENT_SECRET)?))
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct Accounts {
    anilist: Option<AniList>,
    trakt: Option<Trakt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AniList {
    token: String,
    user: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Trakt {
    access: String,
    refresh: String,
    /// Unix seconds.
    expires_at: u64,
    user: String,
}

pub struct Sync {
    file: PathBuf,
    accounts: Mutex<Accounts>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    anilist_available: bool,
    anilist_user: Option<String>,
    trakt_available: bool,
    trakt_user: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCode {
    device_code: String,
    user_code: String,
    verification_url: String,
    expires_in: u64,
    interval: u64,
}

impl Sync {
    pub fn load(config_dir: &Path) -> Self {
        let file = config_dir.join("sync.json");
        let accounts = std::fs::read_to_string(&file).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
        Self { file, accounts: Mutex::new(accounts) }
    }

    fn snapshot(&self) -> Accounts {
        self.accounts.lock().map(|a| a.clone()).unwrap_or_default()
    }

    fn update(&self, f: impl FnOnce(&mut Accounts)) {
        let Ok(mut accounts) = self.accounts.lock() else { return };
        f(&mut accounts);
        if let Some(parent) = self.file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(text) = serde_json::to_string_pretty(&*accounts) {
            if let Err(e) = std::fs::write(&self.file, text) {
                tracing::warn!("saving sync accounts failed: {e}");
            }
        }
    }

    pub fn status(&self) -> SyncStatus {
        let a = self.snapshot();
        SyncStatus {
            anilist_available: set(ANILIST_CLIENT_ID).is_some(),
            anilist_user: a.anilist.map(|x| x.user),
            trakt_available: trakt_keys().is_some(),
            trakt_user: a.trakt.map(|x| x.user),
        }
    }

    pub fn disconnect(&self, service: &str) {
        self.update(|a| match service {
            "anilist" => a.anilist = None,
            "trakt" => a.trakt = None,
            _ => {}
        });
    }
}

fn changed(app: &AppHandle) {
    let _ = app.emit("sync-changed", ());
}

// ── AniList ────────────────────────────────────────────────────────────

/// Open the AniList sign-in window; the token is caught on redirect.
pub fn anilist_login(app: &AppHandle) -> Result<(), String> {
    let id = set(ANILIST_CLIENT_ID).ok_or("AniList isn't set up in this build")?;
    if let Some(w) = app.get_webview_window(LOGIN_WINDOW) {
        let _ = w.set_focus();
        return Ok(());
    }
    let url = format!("https://anilist.co/api/v2/oauth/authorize?client_id={id}&response_type=token");
    let handle = app.clone();
    WebviewWindowBuilder::new(app, LOGIN_WINDOW, WebviewUrl::External(url.parse().map_err(|e| format!("{e}"))?))
        .title("Connect AniList")
        .inner_size(520.0, 760.0)
        .center()
        .on_navigation(move |url| {
            let token = url
                .fragment()
                .and_then(|f| f.split('&').find_map(|kv| kv.strip_prefix("access_token=")))
                .map(str::to_string);
            let Some(token) = token else { return true };
            let h = handle.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = finish_anilist(&h, token).await {
                    tracing::warn!("AniList sign-in failed: {e}");
                    let _ = h.emit("sync-error", e);
                }
                if let Some(w) = h.get_webview_window(LOGIN_WINDOW) {
                    let _ = w.close();
                }
            });
            false
        })
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}

async fn finish_anilist(app: &AppHandle, token: String) -> Result<(), String> {
    let state = app.state::<AppState>();
    let engine = state.engine.clone();
    let data = anilist_gql(&engine, &token, "query { Viewer { name } }", json!({})).await?;
    let user = data["Viewer"]["name"].as_str().unwrap_or("AniList").to_string();
    state.sync.update(|a| a.anilist = Some(AniList { token, user }));
    changed(app);
    Ok(())
}

async fn anilist_gql(engine: &Engine, token: &str, query: &str, variables: Value) -> Result<Value, String> {
    let resp = engine
        .ctx()
        .http
        .client()
        .post(ANILIST_GRAPHQL)
        .bearer_auth(token)
        .json(&json!({ "query": query, "variables": variables }))
        .send()
        .await
        .map_err(|e| format!("AniList: {e}"))?;
    let status = resp.status();
    let body: Value = resp.json().await.map_err(|e| format!("AniList: {e}"))?;
    if !status.is_success() || body.get("errors").is_some() {
        let msg = body["errors"][0]["message"].as_str().unwrap_or(status.as_str()).to_string();
        return Err(format!("AniList: {msg}"));
    }
    Ok(body["data"].clone())
}

// ── Trakt ──────────────────────────────────────────────────────────────

fn trakt_post(engine: &Engine, path: &str, client_id: &str, token: Option<&str>, body: Value) -> impl std::future::Future<Output = Result<(u16, Value), String>> {
    let mut req = engine
        .ctx()
        .http
        .client()
        .post(format!("{TRAKT_API}{path}"))
        .header("trakt-api-version", "2")
        .header("trakt-api-key", client_id)
        .json(&body);
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    async move {
        let resp = req.send().await.map_err(|e| format!("Trakt: {e}"))?;
        let status = resp.status().as_u16();
        let body = resp.json::<Value>().await.unwrap_or(Value::Null);
        Ok((status, body))
    }
}

pub async fn trakt_start(engine: &Engine) -> Result<DeviceCode, String> {
    let (id, _) = trakt_keys().ok_or("Trakt isn't set up in this build")?;
    let (status, body) = trakt_post(engine, "/oauth/device/code", id, None, json!({ "client_id": id })).await?;
    if status != 200 {
        return Err(format!("Trakt answered {status}"));
    }
    Ok(DeviceCode {
        device_code: body["device_code"].as_str().unwrap_or_default().to_string(),
        user_code: body["user_code"].as_str().unwrap_or_default().to_string(),
        verification_url: body["verification_url"].as_str().unwrap_or("https://trakt.tv/activate").to_string(),
        expires_in: body["expires_in"].as_u64().unwrap_or(600),
        interval: body["interval"].as_u64().unwrap_or(5),
    })
}

/// One poll of the device flow: `Ok(true)` once signed in, `Ok(false)` while waiting.
pub async fn trakt_poll(app: &AppHandle, device_code: &str) -> Result<bool, String> {
    let (id, secret) = trakt_keys().ok_or("Trakt isn't set up in this build")?;
    let state = app.state::<AppState>();
    let engine = state.engine.clone();
    let body = json!({ "code": device_code, "client_id": id, "client_secret": secret });
    let (status, tokens) = trakt_post(&engine, "/oauth/device/token", id, None, body).await?;
    match status {
        200 => {}
        400 | 429 => return Ok(false), // not approved yet / polling too fast
        404 => return Err("That code isn't valid any more".into()),
        409 => return Err("That code was already used".into()),
        410 => return Err("The code expired, start again".into()),
        418 => return Err("Sign-in was declined".into()),
        s => return Err(format!("Trakt answered {s}")),
    }
    let access = tokens["access_token"].as_str().unwrap_or_default().to_string();
    let refresh = tokens["refresh_token"].as_str().unwrap_or_default().to_string();
    let expires_at = tokens["created_at"].as_u64().unwrap_or_else(now) + tokens["expires_in"].as_u64().unwrap_or(86_400);

    let user = engine
        .ctx()
        .http
        .client()
        .get(format!("{TRAKT_API}/users/settings"))
        .header("trakt-api-version", "2")
        .header("trakt-api-key", id)
        .bearer_auth(&access)
        .send()
        .await
        .ok();
    let user = match user {
        Some(r) => r.json::<Value>().await.ok().and_then(|v| v["user"]["username"].as_str().map(str::to_string)),
        None => None,
    }
    .unwrap_or_else(|| "Trakt".into());

    state.sync.update(|a| a.trakt = Some(Trakt { access, refresh, expires_at, user }));
    changed(app);
    Ok(true)
}

/// A valid Trakt access token, refreshed first when it is about to expire.
async fn trakt_token(app: &AppHandle, engine: &Engine) -> Option<String> {
    let (id, secret) = trakt_keys()?;
    let state = app.state::<AppState>();
    let account = state.sync.snapshot().trakt?;
    if now() + 3600 < account.expires_at {
        return Some(account.access);
    }
    let body = json!({
        "refresh_token": account.refresh,
        "client_id": id,
        "client_secret": secret,
        "redirect_uri": TRAKT_REDIRECT,
        "grant_type": "refresh_token",
    });
    match trakt_post(engine, "/oauth/token", id, None, body).await {
        Ok((200, t)) => {
            let access = t["access_token"].as_str()?.to_string();
            let refresh = t["refresh_token"].as_str().unwrap_or(&account.refresh).to_string();
            let expires_at = t["created_at"].as_u64().unwrap_or_else(now) + t["expires_in"].as_u64().unwrap_or(86_400);
            let user = account.user.clone();
            state.sync.update(|a| a.trakt = Some(Trakt { access: access.clone(), refresh, expires_at, user }));
            Some(access)
        }
        Ok((s, _)) => {
            tracing::warn!("Trakt token refresh answered {s}; signing out");
            state.sync.update(|a| a.trakt = None);
            changed(app);
            None
        }
        Err(e) => {
            tracing::warn!("Trakt token refresh failed: {e}");
            None
        }
    }
}

// ── Marking things watched ─────────────────────────────────────────────

/// Called by the player once a title or episode is mostly watched.
/// Returns the names of the services that were updated.
pub async fn watched(app: &AppHandle, title: &TitleSummary, target: Target) -> Vec<&'static str> {
    let state = app.state::<AppState>();
    let engine = state.engine.clone();
    let accounts = state.sync.snapshot();
    let mut done = Vec::new();

    if let (Some(acc), Some(media_id)) = (accounts.anilist.as_ref(), title.ids.anilist) {
        let (progress, status) = match target {
            Target::Episode { episode, .. } => (episode, "CURRENT"),
            _ => (1, "COMPLETED"),
        };
        let mutation = "mutation ($id: Int, $progress: Int, $status: MediaListStatus) { \
                        SaveMediaListEntry(mediaId: $id, progress: $progress, status: $status) { id } }";
        let vars = json!({ "id": media_id, "progress": progress, "status": status });
        match anilist_gql(&engine, &acc.token, mutation, vars).await {
            Ok(_) => done.push("AniList"),
            Err(e) => tracing::warn!("AniList update failed: {e}"),
        }
    }

    let has_ids = title.ids.imdb.is_some() || title.ids.tmdb.is_some();
    if accounts.trakt.is_some() && has_ids {
        if let (Some(token), Some((id, _))) = (trakt_token(app, &engine).await, trakt_keys()) {
            let mut ids = json!({});
            if let Some(imdb) = &title.ids.imdb {
                ids["imdb"] = json!(imdb);
            }
            if let Some(tmdb) = title.ids.tmdb {
                ids["tmdb"] = json!(tmdb);
            }
            let body = match target {
                Target::Episode { season, episode } => json!({
                    "shows": [{ "ids": ids, "seasons": [{ "number": season, "episodes": [{ "number": episode }] }] }]
                }),
                _ => json!({ "movies": [{ "ids": ids }] }),
            };
            match trakt_post(&engine, "/sync/history", id, Some(&token), body).await {
                Ok((200 | 201, _)) => done.push("Trakt"),
                Ok((s, _)) => tracing::warn!("Trakt history answered {s}"),
                Err(e) => tracing::warn!("Trakt history failed: {e}"),
            }
        }
    }
    done
}
