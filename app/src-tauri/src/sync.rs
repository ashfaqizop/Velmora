//! Watch-progress sync: AniList for anime, Trakt for movies and series.
//!
//! Client ids come from repo secrets at build time; a build without them
//! simply reports the service as unavailable. Sign-in tokens are kept in
//! `sync.json` in the app config folder. Neither service needs a client
//! secret, which a desktop app couldn't keep secret anyway.
//!
//! Both sign-ins run in a small WebView window that we watch:
//! * AniList (implicit grant): the token arrives in the redirect's fragment.
//! * Trakt (authorization code + PKCE): the code arrives in the redirect's
//!   query and is exchanged together with the PKCE verifier.
//!
//! The redirect never loads; navigation is cancelled as soon as we see it.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, Manager, Url, WebviewUrl, WebviewWindowBuilder};
use universeapi::model::{Target, TitleSummary};
use universeapi::Engine;

use crate::AppState;

const ANILIST_CLIENT_ID: Option<&str> = option_env!("VELMORA_ANILIST_CLIENT_ID");
const TRAKT_CLIENT_ID: Option<&str> = option_env!("VELMORA_TRAKT_CLIENT_ID");

const ANILIST_GRAPHQL: &str = "https://graphql.anilist.co";
const TRAKT_API: &str = "https://api.trakt.tv";
/// Must match the redirect URI registered for the Trakt app.
const TRAKT_REDIRECT: &str = "https://localhost/velmora/trakt-callback";
const ANILIST_WINDOW: &str = "anilist-login";
const TRAKT_WINDOW: &str = "trakt-login";

fn set(v: Option<&'static str>) -> Option<&'static str> {
    v.filter(|s| !s.trim().is_empty())
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// URL-safe random string for PKCE verifiers and the OAuth `state`.
fn random_token(bytes: usize) -> Result<String, String> {
    let mut buf = vec![0u8; bytes];
    getrandom::getrandom(&mut buf).map_err(|e| e.to_string())?;
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(buf))
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

/// A Trakt sign-in in progress: the PKCE verifier and the state we sent.
struct Pending {
    verifier: String,
    state: String,
}

pub struct Sync {
    file: PathBuf,
    accounts: Mutex<Accounts>,
    pending: Mutex<Option<Pending>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    anilist_available: bool,
    anilist_user: Option<String>,
    trakt_available: bool,
    trakt_user: Option<String>,
}

impl Sync {
    pub fn load(config_dir: &Path) -> Self {
        let file = config_dir.join("sync.json");
        let accounts = std::fs::read_to_string(&file).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
        Self { file, accounts: Mutex::new(accounts), pending: Mutex::new(None) }
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
            trakt_available: set(TRAKT_CLIENT_ID).is_some(),
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

fn report(app: &AppHandle, err: String) {
    tracing::warn!("sign-in failed: {err}");
    let _ = app.emit("sync-error", err);
}

/// Open a sign-in window and hand the first URL starting with `redirect`
/// to `on_redirect`; the window closes after that.
fn sign_in_window<F, Fut>(app: &AppHandle, label: &str, title: &str, url: Url, redirect: &'static str, on_redirect: F) -> Result<(), String>
where
    F: Fn(AppHandle, Url) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<(), String>> + Send + 'static,
{
    if let Some(w) = app.get_webview_window(label) {
        let _ = w.set_focus();
        return Ok(());
    }
    let handle = app.clone();
    let label_owned = label.to_string();
    WebviewWindowBuilder::new(app, label, WebviewUrl::External(url))
        .title(title)
        .inner_size(520.0, 760.0)
        .center()
        .on_navigation(move |nav| {
            if !nav.as_str().starts_with(redirect) {
                return true;
            }
            let h = handle.clone();
            let label = label_owned.clone();
            let fut = on_redirect(h.clone(), nav.clone());
            tauri::async_runtime::spawn(async move {
                if let Err(e) = fut.await {
                    report(&h, e);
                }
                if let Some(w) = h.get_webview_window(&label) {
                    let _ = w.close();
                }
            });
            false
        })
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ── AniList ────────────────────────────────────────────────────────────

const ANILIST_REDIRECT: &str = "https://anilist.co/api/v2/oauth/pin";

pub fn anilist_login(app: &AppHandle) -> Result<(), String> {
    let id = set(ANILIST_CLIENT_ID).ok_or("AniList isn't set up in this build")?;
    let url = Url::parse_with_params("https://anilist.co/api/v2/oauth/authorize", &[("client_id", id), ("response_type", "token")])
        .map_err(|e| e.to_string())?;
    sign_in_window(app, ANILIST_WINDOW, "Connect AniList", url, ANILIST_REDIRECT, |app, url| async move {
        let token = url
            .fragment()
            .and_then(|f| f.split('&').find_map(|kv| kv.strip_prefix("access_token=")))
            .map(str::to_string)
            .ok_or("AniList didn't return a token")?;
        finish_anilist(&app, token).await
    })
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

async fn trakt_post(engine: &Engine, path: &str, token: Option<&str>, body: Value) -> Result<(u16, Value), String> {
    let id = set(TRAKT_CLIENT_ID).ok_or("Trakt isn't set up in this build")?;
    let mut req = engine
        .ctx()
        .http
        .client()
        .post(format!("{TRAKT_API}{path}"))
        .header("trakt-api-version", "2")
        .header("trakt-api-key", id)
        .json(&body);
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    let resp = req.send().await.map_err(|e| format!("Trakt: {e}"))?;
    let status = resp.status().as_u16();
    let body = resp.json::<Value>().await.unwrap_or(Value::Null);
    Ok((status, body))
}

/// Start Trakt sign-in (authorization code with PKCE).
pub fn trakt_login(app: &AppHandle) -> Result<(), String> {
    let id = set(TRAKT_CLIENT_ID).ok_or("Trakt isn't set up in this build")?;
    let verifier = random_token(48)?;
    let state = random_token(16)?;
    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let url = Url::parse_with_params(
        "https://trakt.tv/oauth/authorize",
        &[
            ("response_type", "code"),
            ("client_id", id),
            ("redirect_uri", TRAKT_REDIRECT),
            ("state", state.as_str()),
            ("code_challenge", challenge.as_str()),
            ("code_challenge_method", "S256"),
        ],
    )
    .map_err(|e| e.to_string())?;

    let app_state = app.state::<AppState>();
    if let Ok(mut p) = app_state.sync.pending.lock() {
        *p = Some(Pending { verifier, state });
    }
    sign_in_window(app, TRAKT_WINDOW, "Connect Trakt", url, TRAKT_REDIRECT, |app, url| async move {
        let query: std::collections::HashMap<String, String> = url.query_pairs().into_owned().collect();
        if let Some(err) = query.get("error") {
            return Err(format!("Trakt: {err}"));
        }
        let code = query.get("code").cloned().ok_or("Trakt didn't return a code")?;
        finish_trakt(&app, code, query.get("state").cloned().unwrap_or_default()).await
    })
}

async fn finish_trakt(app: &AppHandle, code: String, returned_state: String) -> Result<(), String> {
    let state = app.state::<AppState>();
    let pending = state.sync.pending.lock().ok().and_then(|mut p| p.take()).ok_or("No Trakt sign-in in progress")?;
    if pending.state != returned_state {
        return Err("Trakt sign-in didn't match, try again".into());
    }
    let id = set(TRAKT_CLIENT_ID).ok_or("Trakt isn't set up in this build")?;
    let engine = state.engine.clone();
    let body = json!({
        "code": code,
        "client_id": id,
        "redirect_uri": TRAKT_REDIRECT,
        "grant_type": "authorization_code",
        "code_verifier": pending.verifier,
    });
    let (status, tokens) = trakt_post(&engine, "/oauth/token", None, body).await?;
    if status != 200 {
        let msg = tokens["error_description"].as_str().or(tokens["error"].as_str()).unwrap_or("sign-in failed");
        return Err(format!("Trakt: {msg} ({status})"));
    }
    let (access, refresh, expires_at) = read_tokens(&tokens, None).ok_or("Trakt returned no token")?;
    let user = trakt_username(&engine, &access).await.unwrap_or_else(|| "Trakt".into());
    state.sync.update(|a| a.trakt = Some(Trakt { access, refresh, expires_at, user }));
    changed(app);
    Ok(())
}

fn read_tokens(t: &Value, old_refresh: Option<&str>) -> Option<(String, String, u64)> {
    let access = t["access_token"].as_str()?.to_string();
    let refresh = t["refresh_token"].as_str().or(old_refresh).unwrap_or_default().to_string();
    let expires_at = t["created_at"].as_u64().unwrap_or_else(now) + t["expires_in"].as_u64().unwrap_or(86_400);
    Some((access, refresh, expires_at))
}

async fn trakt_username(engine: &Engine, access: &str) -> Option<String> {
    let id = set(TRAKT_CLIENT_ID)?;
    let resp = engine
        .ctx()
        .http
        .client()
        .get(format!("{TRAKT_API}/users/settings"))
        .header("trakt-api-version", "2")
        .header("trakt-api-key", id)
        .bearer_auth(access)
        .send()
        .await
        .ok()?;
    let v: Value = resp.json().await.ok()?;
    v["user"]["username"].as_str().map(str::to_string)
}

/// A valid Trakt access token, refreshed first when it is about to expire.
async fn trakt_token(app: &AppHandle, engine: &Engine) -> Option<String> {
    let id = set(TRAKT_CLIENT_ID)?;
    let state = app.state::<AppState>();
    let account = state.sync.snapshot().trakt?;
    if now() + 3600 < account.expires_at {
        return Some(account.access);
    }
    let body = json!({
        "refresh_token": account.refresh,
        "client_id": id,
        "redirect_uri": TRAKT_REDIRECT,
        "grant_type": "refresh_token",
    });
    match trakt_post(engine, "/oauth/token", None, body).await {
        Ok((200, t)) => {
            let (access, refresh, expires_at) = read_tokens(&t, Some(&account.refresh))?;
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
        if let Some(token) = trakt_token(app, &engine).await {
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
            match trakt_post(&engine, "/sync/history", Some(&token), body).await {
                Ok((200 | 201, _)) => done.push("Trakt"),
                Ok((s, _)) => tracing::warn!("Trakt history answered {s}"),
                Err(e) => tracing::warn!("Trakt history failed: {e}"),
            }
        }
    }
    done
}
