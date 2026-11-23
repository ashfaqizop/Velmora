//! Velmora accounts: the browser signs you in, this catches the result.
//!
//! Signing in happens on the auth site (Google, Discord or an emailed code,
//! then a profile the first time). The app only does three things:
//!
//! 1. `account_begin` makes a secret (PKCE), listens on a local port, and opens
//!    the browser at the auth site with the port and a hash of the secret. When
//!    the site is done it sends the browser to `http://127.0.0.1:<port>/callback`
//!    with a one-time code, which is traded at the site's API for the session.
//! 2. `account_redeem` does the same trade with a short code typed in by hand,
//!    for when a firewall blocks the local port.
//! 3. `account_vault_*` keeps the session's refresh token in Windows Credential
//!    Manager (a file elsewhere), so the next launch is already signed in.
//!
//! Nothing here keeps the access token. The page's script talks to Supabase.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
use base64::Engine as _;
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, State};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Notify;

use crate::AppState;

/// Where sign-in pages and the handoff API live.
const SITE: &str = "https://auth.getvelmora.dpdns.org";
const TIMEOUT: Duration = Duration::from_secs(15);
/// How long to wait for the person to finish in the browser.
const WAIT: Duration = Duration::from_secs(600);

mod vault {
    //! Credential Manager on Windows, a file elsewhere (Android later). Only
    //! the refresh token goes in: Credential Manager holds about 2.5 KB a value.
    use std::path::Path;

    #[cfg(windows)]
    fn entry() -> Option<keyring::Entry> {
        keyring::Entry::new("Velmora", "account").ok()
    }

    #[cfg(windows)]
    pub fn read(_dir: &Path) -> Option<String> {
        entry()?.get_password().ok()
    }

    #[cfg(windows)]
    pub fn write(_dir: &Path, text: &str) -> Result<(), String> {
        entry().ok_or("Credential Manager isn't available.")?.set_password(text).map_err(|e| e.to_string())
    }

    #[cfg(windows)]
    pub fn clear(_dir: &Path) {
        if let Some(e) = entry() {
            let _ = e.delete_credential();
        }
    }

    #[cfg(not(windows))]
    pub fn read(dir: &Path) -> Option<String> {
        std::fs::read_to_string(dir.join("account.token")).ok()
    }

    #[cfg(not(windows))]
    pub fn write(dir: &Path, text: &str) -> Result<(), String> {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        std::fs::write(dir.join("account.token"), text).map_err(|e| e.to_string())
    }

    #[cfg(not(windows))]
    pub fn clear(dir: &Path) {
        let _ = std::fs::remove_file(dir.join("account.token"));
    }
}

/// A sign-in waiting for the browser.
struct Attempt {
    verifier: String,
    cancel: Arc<Notify>,
}

pub struct AccountStore {
    data_dir: PathBuf,
    attempt: Mutex<Option<Attempt>>,
}

impl AccountStore {
    pub fn new(data_dir: &Path) -> Self {
        Self { data_dir: data_dir.to_path_buf(), attempt: Mutex::new(None) }
    }

    fn take_attempt(&self) -> Option<Attempt> {
        self.attempt.lock().unwrap().take()
    }
}

/// What the page needs to continue the session.
#[derive(Serialize)]
pub struct Session {
    access_token: String,
    refresh_token: String,
}

fn random_b64(bytes: usize) -> String {
    let mut buf = vec![0u8; bytes];
    let _ = getrandom::getrandom(&mut buf);
    B64.encode(buf)
}

/// The one-time code from `GET /callback?code=...&state=...`, if this is that
/// request and it carries our state.
fn parse_callback(target: &str, state: &str) -> Option<String> {
    let (path, query) = target.split_once('?')?;
    if path != "/callback" {
        return None;
    }
    let (mut code, mut got_state) = (None, None);
    for pair in query.split('&') {
        match pair.split_once('=') {
            Some(("code", v)) => code = Some(v),
            Some(("state", v)) => got_state = Some(v),
            _ => {}
        }
    }
    let code = code?;
    let ok = code.len() == 43 && code.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_');
    (ok && got_state == Some(state)).then(|| code.to_string())
}

async fn respond(sock: &mut TcpStream, status: &str, location: Option<&str>) {
    let mut head = format!("HTTP/1.1 {status}\r\nContent-Length: 0\r\nConnection: close\r\nCache-Control: no-store\r\n");
    if let Some(l) = location {
        head.push_str(&format!("Location: {l}\r\n"));
    }
    head.push_str("\r\n");
    let _ = sock.write_all(head.as_bytes()).await;
    let _ = sock.shutdown().await;
}

/// Accepts connections until the browser arrives with the right code.
async fn wait_for_callback(listener: &TcpListener, state: &str) -> Result<String, String> {
    loop {
        let (mut sock, _) = listener.accept().await.map_err(|e| format!("Sign-in stopped: {e}"))?;
        let mut buf = vec![0u8; 8192];
        let mut len = 0usize;
        let read = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let n = sock.read(&mut buf[len..]).await.unwrap_or(0);
                if n == 0 {
                    break;
                }
                len += n;
                if len == buf.len() || buf[..len].windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
            }
        })
        .await;
        if read.is_err() {
            continue;
        }
        let head = String::from_utf8_lossy(&buf[..len]).to_string();
        let target = head.lines().next().and_then(|l| l.split_whitespace().nth(1)).unwrap_or("").to_string();
        match parse_callback(&target, state) {
            Some(code) => {
                respond(&mut sock, "302 Found", Some(&format!("{SITE}/auth/?done=1"))).await;
                return Ok(code);
            }
            // A favicon request, or something else poking the port.
            None => respond(&mut sock, "404 Not Found", None).await,
        }
    }
}

/// Trade a one-time code (the long one or the typed short one) for the session.
async fn exchange(state: &AppState, code: &str, verifier: &str) -> Result<Session, String> {
    let resp = state
        .engine
        .ctx()
        .http
        .client()
        .post(format!("{SITE}/api/auth/exchange"))
        .header("Content-Type", "application/json")
        .body(json!({ "code": code, "verifier": verifier }).to_string())
        .timeout(TIMEOUT)
        .send()
        .await
        .map_err(|_| "Couldn't reach Velmora's server. Check your connection and try again.".to_string())?;
    let body: Value = serde_json::from_str(&resp.text().await.unwrap_or_default()).unwrap_or(Value::Null);
    if body["ok"] == true {
        if let (Some(a), Some(r)) = (body["access_token"].as_str(), body["refresh_token"].as_str()) {
            return Ok(Session { access_token: a.to_string(), refresh_token: r.to_string() });
        }
    }
    Err(body["message"].as_str().unwrap_or("Sign-in couldn't finish. Try again.").to_string())
}

/// Open the browser to sign in, and wait for it to come back. Resolves to the
/// session, or to nothing if the attempt was cancelled or finished by a typed code.
#[tauri::command]
pub async fn account_begin(
    app: AppHandle,
    state: State<'_, AppState>,
    acct: State<'_, AccountStore>,
) -> Result<Option<Session>, String> {
    use tauri_plugin_opener::OpenerExt;

    let verifier = random_b64(32);
    let challenge = B64.encode(Sha256::digest(verifier.as_bytes()));
    let csrf = random_b64(24);

    let listener = TcpListener::bind(("127.0.0.1", 0)).await.map_err(|e| format!("Couldn't start sign-in: {e}"))?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();

    let cancel = Arc::new(Notify::new());
    let old = acct.attempt.lock().unwrap().replace(Attempt { verifier: verifier.clone(), cancel: cancel.clone() });
    if let Some(old) = old {
        old.cancel.notify_one();
    }

    let url = format!("{SITE}/auth/?port={port}&state={csrf}&challenge={challenge}");
    if let Err(e) = app.opener().open_url(url, None::<&str>) {
        acct.take_attempt();
        return Err(format!("Couldn't open your browser: {e}"));
    }

    let outcome = tokio::select! {
        r = wait_for_callback(&listener, &csrf) => Some(r),
        _ = cancel.notified() => None,
        _ = tokio::time::sleep(WAIT) => Some(Err("Sign-in timed out. Try again.".to_string())),
    };

    // Only clear the attempt if it is still this one (a newer one may have replaced it).
    {
        let mut slot = acct.attempt.lock().unwrap();
        if slot.as_ref().map(|a| Arc::ptr_eq(&a.cancel, &cancel)).unwrap_or(false) {
            *slot = None;
        }
    }

    match outcome {
        None => Ok(None),
        Some(Err(e)) => Err(e),
        Some(Ok(code)) => exchange(&state, &code, &verifier).await.map(Some),
    }
}

/// Finish the waiting sign-in with the short code the website shows.
#[tauri::command]
pub async fn account_redeem(state: State<'_, AppState>, acct: State<'_, AccountStore>, code: String) -> Result<Session, String> {
    let (verifier, cancel) = {
        let slot = acct.attempt.lock().unwrap();
        match slot.as_ref() {
            Some(a) => (a.verifier.clone(), a.cancel.clone()),
            None => return Err("Start signing in first, then enter the code.".into()),
        }
    };
    let code: String = code.chars().filter(|c| !c.is_whitespace()).collect();
    let session = exchange(&state, &code, &verifier).await?;
    // The browser route is no longer needed.
    cancel.notify_one();
    Ok(session)
}

/// Stop waiting for the browser.
#[tauri::command]
pub fn account_cancel(acct: State<'_, AccountStore>) {
    if let Some(a) = acct.take_attempt() {
        a.cancel.notify_one();
    }
}

#[tauri::command]
pub fn account_vault_read(acct: State<'_, AccountStore>) -> Option<String> {
    vault::read(&acct.data_dir)
}

#[tauri::command]
pub fn account_vault_write(acct: State<'_, AccountStore>, text: String) -> Result<(), String> {
    vault::write(&acct.data_dir, &text)
}

#[tauri::command]
pub fn account_vault_clear(acct: State<'_, AccountStore>) {
    vault::clear(&acct.data_dir);
}

#[cfg(test)]
mod tests {
    use super::*;

    const CODE: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQ";

    #[test]
    fn callback_needs_path_code_and_state() {
        assert_eq!(CODE.len(), 43);
        assert_eq!(parse_callback(&format!("/callback?code={CODE}&state=xyz"), "xyz"), Some(CODE.to_string()));
        assert_eq!(parse_callback(&format!("/callback?state=xyz&code={CODE}"), "xyz"), Some(CODE.to_string()));
        assert!(parse_callback(&format!("/callback?code={CODE}&state=nope"), "xyz").is_none());
        assert!(parse_callback(&format!("/other?code={CODE}&state=xyz"), "xyz").is_none());
        assert!(parse_callback("/callback?code=short&state=xyz", "xyz").is_none());
        assert!(parse_callback("/favicon.ico", "xyz").is_none());
    }

    #[test]
    fn secrets_are_url_safe() {
        let v = random_b64(32);
        assert_eq!(v.len(), 43);
        assert!(v.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'));
        assert_eq!(B64.encode(Sha256::digest(v.as_bytes())).len(), 43);
    }
}
