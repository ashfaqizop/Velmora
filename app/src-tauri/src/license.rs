//! Velmora Max: the license key, its one-PC activation, and the launch
//! counter the ad rules use.
//!
//! A Max key is `VLM1.<payload>.<signature>` (base64url), signed with Ed25519
//! by a key that exists only on the developer's PC. Activating it sends the
//! key and a SHA-256 of this PC's ID to the site, which binds the key to this
//! PC and answers with an activation token signed by the server key. Both
//! signatures are checked here with public keys only.
//!
//! The key and token live in Windows Credential Manager. The token lasts 30
//! days and is refreshed about once a week, so Max survives a month offline.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
use base64::Engine as _;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tauri::State;

use crate::AppState;

/// Checks Max keys. Same value as `site/api/_lib/keys.js`.
const SIGNING_PUBLIC_KEY: &str = "FnlbeKNPEBeONzf_j6oj74xGFKFTWl1qAyIA9QJNS5c";
/// Checks activation tokens from the site.
const SERVER_PUBLIC_KEY: &str = "yiN3UCjCmt1qBkAC90Elq0yQ8JBnJ6lO7UxtCRBxgGs";
pub const SITE: &str = "https://getvelmora.dpdns.org";
/// Ask the server again once the token is this old.
const REFRESH_AFTER: u64 = 7 * 86_400;
const TIMEOUT: Duration = Duration::from_secs(15);

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn verify(public: &str, payload: &[u8], sig: &[u8]) -> bool {
    let Ok(pk) = B64.decode(public) else { return false };
    let Ok(pk) = <[u8; 32]>::try_from(pk) else { return false };
    let Ok(vk) = VerifyingKey::from_bytes(&pk) else { return false };
    let Ok(sig) = Signature::from_slice(sig) else { return false };
    vk.verify(payload, &sig).is_ok()
}

#[derive(Deserialize, Clone, Debug)]
struct KeyPayload {
    v: u32,
    id: String,
    #[serde(default)]
    n: String,
}

/// A Max key's contents, if its signature checks out.
fn parse_key(key: &str) -> Option<KeyPayload> {
    let mut parts = key.trim().split('.');
    let (Some("VLM1"), Some(p), Some(s), None) = (parts.next(), parts.next(), parts.next(), parts.next()) else {
        return None;
    };
    let payload = B64.decode(p).ok()?;
    let sig = B64.decode(s).ok()?;
    if !verify(SIGNING_PUBLIC_KEY, &payload, &sig) {
        return None;
    }
    let k: KeyPayload = serde_json::from_slice(&payload).ok()?;
    (k.v == 1 && k.id.starts_with("MX-")).then_some(k)
}

#[derive(Deserialize, Debug)]
struct TokenPayload {
    v: u32,
    id: String,
    d: String,
    iat: u64,
    exp: u64,
}

fn parse_token(token: &str) -> Option<TokenPayload> {
    let (p, s) = token.split_once('.')?;
    let payload = B64.decode(p).ok()?;
    let sig = B64.decode(s).ok()?;
    if !verify(SERVER_PUBLIC_KEY, &payload, &sig) {
        return None;
    }
    let t: TokenPayload = serde_json::from_slice(&payload).ok()?;
    (t.v == 1).then_some(t)
}

/// What's kept in Credential Manager.
#[derive(Serialize, Deserialize, Clone, Default)]
struct Stored {
    key: String,
    #[serde(default)]
    token: Option<String>,
    /// The server's last refusal ("revoked", "other_device"), if any.
    #[serde(default)]
    problem: Option<String>,
}

mod vault {
    //! Credential Manager on Windows, a file elsewhere (Android later).
    use std::path::Path;

    #[cfg(windows)]
    fn entry() -> Option<keyring::Entry> {
        keyring::Entry::new("Velmora", "max-license").ok()
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
        std::fs::read_to_string(dir.join("license.json")).ok()
    }

    #[cfg(not(windows))]
    pub fn write(dir: &Path, text: &str) -> Result<(), String> {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        std::fs::write(dir.join("license.json"), text).map_err(|e| e.to_string())
    }

    #[cfg(not(windows))]
    pub fn clear(dir: &Path) {
        let _ = std::fs::remove_file(dir.join("license.json"));
    }
}

/// This PC's ID as the server sees it: a hash, never the ID itself.
fn device_id(data_dir: &Path) -> String {
    let raw = machine_guid().unwrap_or_else(|| fallback_id(data_dir));
    let mut h = Sha256::new();
    h.update(b"velmora-device-v1:");
    h.update(raw.trim().to_lowercase().as_bytes());
    format!("{:x}", h.finalize())
}

/// Windows' own machine ID. It changes when Windows is reinstalled, which is
/// when a key should need releasing anyway.
#[cfg(windows)]
fn machine_guid() -> Option<String> {
    use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_64KEY};
    let key = winreg::RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(r"SOFTWARE\Microsoft\Cryptography", KEY_READ | KEY_WOW64_64KEY)
        .ok()?;
    key.get_value::<String, _>("MachineGuid").ok().filter(|s| !s.trim().is_empty())
}

#[cfg(not(windows))]
fn machine_guid() -> Option<String> {
    std::fs::read_to_string("/etc/machine-id").ok().filter(|s| !s.trim().is_empty())
}

/// A random ID kept in the data folder, for systems without a machine ID.
fn fallback_id(data_dir: &Path) -> String {
    let file = data_dir.join("device.id");
    if let Ok(id) = std::fs::read_to_string(&file) {
        if !id.trim().is_empty() {
            return id;
        }
    }
    let mut bytes = [0u8; 16];
    let _ = getrandom::getrandom(&mut bytes);
    let id = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
    let _ = std::fs::create_dir_all(data_dir);
    let _ = std::fs::write(&file, &id);
    id
}

#[derive(Serialize, Deserialize, Default)]
struct Usage {
    launches: u32,
    first: u64,
}

/// Counts this launch. The ad rules skip the first one.
fn count_launch(data_dir: &Path) -> u32 {
    let file = data_dir.join("usage.json");
    let mut usage: Usage = std::fs::read_to_string(&file).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
    usage.launches = usage.launches.saturating_add(1);
    if usage.first == 0 {
        usage.first = now();
    }
    let _ = std::fs::create_dir_all(data_dir);
    if let Ok(text) = serde_json::to_string(&usage) {
        let _ = std::fs::write(&file, text);
    }
    usage.launches
}

pub struct LicenseStore {
    data_dir: PathBuf,
    device: String,
    launches: u32,
    stored: Mutex<Option<Stored>>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LicenseStatus {
    /// "public" or "max".
    tier: &'static str,
    /// none, active, pending, expired, revoked, other_device.
    state: &'static str,
    id: Option<String>,
    name: Option<String>,
    /// When the current activation runs out if the server can't be reached.
    valid_until: Option<u64>,
    /// Launches of the app on this PC, this one included.
    launches: u32,
}

impl LicenseStatus {
    pub fn tier(&self) -> &'static str {
        self.tier
    }
}

impl LicenseStore {
    pub fn load(data_dir: &Path) -> Self {
        let stored = vault::read(data_dir).and_then(|t| serde_json::from_str::<Stored>(&t).ok());
        Self {
            device: device_id(data_dir),
            launches: count_launch(data_dir),
            data_dir: data_dir.to_path_buf(),
            stored: Mutex::new(stored),
        }
    }

    fn get(&self) -> Option<Stored> {
        self.stored.lock().unwrap().clone()
    }

    fn put(&self, stored: Option<Stored>) -> Result<(), String> {
        match &stored {
            Some(s) => vault::write(&self.data_dir, &serde_json::to_string(s).map_err(|e| e.to_string())?)?,
            None => vault::clear(&self.data_dir),
        }
        *self.stored.lock().unwrap() = stored;
        Ok(())
    }

    /// A token that proves this key is active on this PC right now.
    fn valid_token(&self, key: &KeyPayload, token: Option<&str>) -> Option<TokenPayload> {
        let t = parse_token(token?)?;
        (t.id == key.id && t.d == self.device && t.exp > now()).then_some(t)
    }

    pub fn status(&self) -> LicenseStatus {
        let public = |state, key: Option<&KeyPayload>| LicenseStatus {
            tier: "public",
            state,
            id: key.map(|k| k.id.clone()),
            name: key.map(|k| k.n.clone()),
            valid_until: None,
            launches: self.launches,
        };
        let Some(stored) = self.get() else { return public("none", None) };
        let Some(key) = parse_key(&stored.key) else { return public("none", None) };
        if let Some(t) = self.valid_token(&key, stored.token.as_deref()) {
            return LicenseStatus {
                tier: "max",
                state: "active",
                id: Some(key.id),
                name: Some(key.n),
                valid_until: Some(t.exp),
                launches: self.launches,
            };
        }
        let state = match stored.problem.as_deref() {
            Some("revoked") => "revoked",
            Some("other_device") => "other_device",
            _ if stored.token.is_some() => "expired",
            _ => "pending",
        };
        public(state, Some(&key))
    }

    /// True when the token is missing or old enough to ask the server again.
    fn due(&self) -> bool {
        let Some(stored) = self.get() else { return false };
        let Some(key) = parse_key(&stored.key) else { return false };
        match self.valid_token(&key, stored.token.as_deref()) {
            Some(t) => now().saturating_sub(t.iat) >= REFRESH_AFTER,
            None => stored.problem.is_none() || stored.token.is_some(),
        }
    }
}

enum Answer {
    Ok(String),
    Refused { code: String, message: String },
    Offline,
}

async fn ask(state: &AppState, action: &str, key: &str, device: &str) -> Answer {
    let resp = state
        .engine
        .ctx()
        .http
        .client()
        .post(format!("{SITE}/api/license/{action}"))
        .header("Content-Type", "application/json")
        .body(json!({ "key": key, "device": device }).to_string())
        .timeout(TIMEOUT)
        .send()
        .await;
    let Ok(resp) = resp else { return Answer::Offline };
    let status = resp.status().as_u16();
    let body: Value = serde_json::from_str(&resp.text().await.unwrap_or_default()).unwrap_or(Value::Null);
    if body["ok"] == true {
        return Answer::Ok(body["token"].as_str().unwrap_or_default().to_string());
    }
    let code = body["error"].as_str().unwrap_or_default().to_string();
    // Server trouble and rate limits aren't a verdict on the key.
    if status >= 500 || status == 429 || code.is_empty() {
        return Answer::Offline;
    }
    let message = body["message"].as_str().unwrap_or("The license server refused this key.").to_string();
    Answer::Refused { code, message }
}

#[tauri::command]
pub fn license_status(lic: State<'_, LicenseStore>) -> LicenseStatus {
    lic.status()
}

/// Enter a Max key: check it here, then activate it on this PC.
#[tauri::command]
pub async fn license_activate(
    state: State<'_, AppState>,
    lic: State<'_, LicenseStore>,
    key: String,
) -> Result<LicenseStatus, String> {
    let key = key.split_whitespace().collect::<String>();
    if parse_key(&key).is_none() {
        return Err("That isn't a Velmora Max key. Check that you copied all of it.".into());
    }
    match ask(&state, "activate", &key, &lic.device).await {
        Answer::Ok(token) if parse_token(&token).is_some() => {
            lic.put(Some(Stored { key, token: Some(token), problem: None }))?;
            Ok(lic.status())
        }
        Answer::Ok(_) => Err("The license server gave an answer Velmora couldn't check. Try again later.".into()),
        Answer::Refused { message, .. } => Err(message),
        Answer::Offline => Err("Couldn't reach the license server. Check your connection and try again.".into()),
    }
}

/// Renew the activation when it's due. Called at startup; quiet on failure.
#[tauri::command]
pub async fn license_refresh(state: State<'_, AppState>, lic: State<'_, LicenseStore>) -> Result<LicenseStatus, String> {
    if !lic.due() {
        return Ok(lic.status());
    }
    let Some(mut stored) = lic.get() else { return Ok(lic.status()) };
    match ask(&state, "activate", &stored.key, &lic.device).await {
        Answer::Ok(token) if parse_token(&token).is_some() => {
            stored.token = Some(token);
            stored.problem = None;
            lic.put(Some(stored))?;
        }
        Answer::Refused { code, .. } if matches!(code.as_str(), "revoked" | "other_device" | "bad_key") => {
            stored.token = None;
            stored.problem = Some(if code == "bad_key" { "revoked".into() } else { code });
            lic.put(Some(stored))?;
        }
        // Offline or a hiccup: keep the token until it runs out.
        _ => {}
    }
    Ok(lic.status())
}

/// Remove the key from this PC and free it for another one.
#[tauri::command]
pub async fn license_remove(state: State<'_, AppState>, lic: State<'_, LicenseStore>) -> Result<LicenseStatus, String> {
    if let Some(stored) = lic.get() {
        // Best effort: if the server can't be reached, the developer can release it.
        let _ = ask(&state, "release", &stored.key, &lic.device).await;
    }
    lic.put(None)?;
    Ok(lic.status())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_junk_keys() {
        assert!(parse_key("").is_none());
        assert!(parse_key("VLM1.abc.def").is_none());
        assert!(parse_key("VLM2.abc.def").is_none());
        assert!(parse_key("VLM1.a.b.c").is_none());
        assert!(parse_token("abc.def").is_none());
        assert!(parse_token("nodot").is_none());
    }

    #[test]
    fn public_keys_decode() {
        for k in [SIGNING_PUBLIC_KEY, SERVER_PUBLIC_KEY] {
            let raw = B64.decode(k).unwrap();
            let raw = <[u8; 32]>::try_from(raw).unwrap();
            assert!(VerifyingKey::from_bytes(&raw).is_ok());
        }
    }

    #[test]
    fn device_id_is_a_sha256() {
        let dir = std::env::temp_dir().join("velmora-license-test");
        let id = device_id(&dir);
        assert_eq!(id.len(), 64);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(id, device_id(&dir));
    }
}
