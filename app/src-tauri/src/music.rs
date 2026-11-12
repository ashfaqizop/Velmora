// SPDX-License-Identifier: GPL-3.0-only
// Ported in part from YTubic (https://github.com/NUber-dev/YTubic), Copyright George Shyshov.
//! Music: Google sign-in for YouTube Music, the cookie jar on disk, the
//! session keeper, and the commands the Music pages call.
//!
//! Sign-in only matters for Music; the rest of Velmora never needs it.
//!
//! - Sign-in opens Google's own page in a window with its own WebView2
//!   profile. Once YouTube's cookies appear there, they're read out and handed
//!   to the engine. Velmora never sees the password.
//! - The jar is saved encrypted with Windows DPAPI, readable only by this
//!   Windows user on this PC.
//! - Google expires cookies read out of a browser after about two hours, so a
//!   hidden "keeper" window on the same profile reloads YouTube Music every 20
//!   minutes and the fresh cookies replace the jar. Cookies Google rotates in
//!   ordinary answers are saved as they arrive (see the engine's music::auth).
//!
//! The approach is YTubic's (GPL-3.0-only); this file is a smaller rewrite of
//! the sign-in and keeper parts of its lib.rs.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
#[cfg(desktop)]
use tauri::webview::PageLoadEvent;
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};
#[cfg(desktop)]
use tauri::{WebviewUrl, WebviewWindowBuilder};
use universeapi::music::auth::Cookie;
use universeapi::music::{
    Account, Album, Artist, AudioQuality, FeedPage, LibraryKind, Lyrics, MusicItem, Playlist, Rating, Resolved, SearchFilter,
    SearchResults, StreamMode, TrackPage, YtDlpStatus,
};

use crate::AppState;

const LOGIN_LABEL: &str = "music-login";
const KEEPER_LABEL: &str = "music-keeper";
const SERVICE_LOGIN_URL: &str =
    "https://accounts.google.com/ServiceLogin?service=youtube&continue=https%3A%2F%2Fmusic.youtube.com%2F";
const MUSIC_URL: &str = "https://music.youtube.com/";
/// The browser Google sees. It must match the real engine (WebView2 is
/// Chrome), or Google refuses the sign-in as an insecure browser.
#[cfg(desktop)]
const LOGIN_UA: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36";
/// Shared by the sign-in and keeper windows (WebView2 needs identical args on
/// one profile). They keep both windows off the media keys and stop the
/// hidden keeper from ever playing sound.
#[cfg(desktop)]
const WEBVIEW_ARGS: &str =
    "--disable-features=HardwareMediaKeyHandling,MediaSessionService --autoplay-policy=user-gesture-required";
const KEEP_EVERY: Duration = Duration::from_secs(20 * 60);
const TICK: Duration = Duration::from_millis(1500);

mod secure {
    //! The jar on disk: DPAPI on Windows, so only this Windows user on this
    //! PC can read it. Elsewhere it's a plain file in the app data folder.

    #[cfg(windows)]
    const ENTROPY: &[u8] = b"velmora/music-cookies v1";

    #[cfg(windows)]
    pub fn encrypt(plain: &[u8]) -> Result<Vec<u8>, String> {
        use std::ptr;
        use windows_sys::Win32::Foundation::LocalFree;
        use windows_sys::Win32::Security::Cryptography::{CryptProtectData, CRYPT_INTEGER_BLOB};
        // SAFETY: the input blobs point at live slices for the whole call, and
        // the output buffer is copied out before LocalFree releases it.
        unsafe {
            let input = CRYPT_INTEGER_BLOB { cbData: plain.len() as u32, pbData: plain.as_ptr() as *mut u8 };
            let entropy = CRYPT_INTEGER_BLOB { cbData: ENTROPY.len() as u32, pbData: ENTROPY.as_ptr() as *mut u8 };
            let mut out: CRYPT_INTEGER_BLOB = std::mem::zeroed();
            let ok = CryptProtectData(&input, ptr::null(), &entropy, ptr::null_mut(), ptr::null(), 0, &mut out);
            if ok == 0 {
                return Err("couldn't encrypt the YouTube Music session".into());
            }
            let data = std::slice::from_raw_parts(out.pbData, out.cbData as usize).to_vec();
            LocalFree(out.pbData as _);
            Ok(data)
        }
    }

    #[cfg(windows)]
    pub fn decrypt(blob: &[u8]) -> Result<Vec<u8>, String> {
        use std::ptr;
        use windows_sys::Win32::Foundation::LocalFree;
        use windows_sys::Win32::Security::Cryptography::{CryptUnprotectData, CRYPT_INTEGER_BLOB};
        // SAFETY: as in `encrypt`.
        unsafe {
            let input = CRYPT_INTEGER_BLOB { cbData: blob.len() as u32, pbData: blob.as_ptr() as *mut u8 };
            let entropy = CRYPT_INTEGER_BLOB { cbData: ENTROPY.len() as u32, pbData: ENTROPY.as_ptr() as *mut u8 };
            let mut out: CRYPT_INTEGER_BLOB = std::mem::zeroed();
            let ok = CryptUnprotectData(&input, ptr::null_mut(), &entropy, ptr::null_mut(), ptr::null(), 0, &mut out);
            if ok == 0 {
                return Err("couldn't read the saved YouTube Music session".into());
            }
            let data = std::slice::from_raw_parts(out.pbData, out.cbData as usize).to_vec();
            LocalFree(out.pbData as _);
            Ok(data)
        }
    }

    #[cfg(not(windows))]
    pub fn encrypt(plain: &[u8]) -> Result<Vec<u8>, String> {
        Ok(plain.to_vec())
    }

    #[cfg(not(windows))]
    pub fn decrypt(blob: &[u8]) -> Result<Vec<u8>, String> {
        Ok(blob.to_vec())
    }
}

pub struct MusicStore {
    dir: PathBuf,
    account: Mutex<Option<Account>>,
    /// Google ended the session; the user has to sign in again.
    expired: AtomicBool,
    keeper_running: AtomicBool,
    signing_in: AtomicBool,
}

/// Page loads seen by the keeper window, so a refresh can wait for its reload.
#[cfg_attr(mobile, allow(dead_code))]
static KEEPER_LOADS: AtomicU64 = AtomicU64::new(0);

impl MusicStore {
    fn jar_file(&self) -> PathBuf {
        self.dir.join("cookies.enc")
    }

    fn profile_dir(&self) -> PathBuf {
        self.dir.join("profile")
    }

    fn visitor_file(&self) -> PathBuf {
        self.dir.join("visitor.txt")
    }
}

fn save_jar(file: &Path, cookies: &[Cookie]) {
    let result = serde_json::to_vec(cookies)
        .map_err(|e| e.to_string())
        .and_then(|plain| secure::encrypt(&plain))
        .and_then(|blob| {
            if let Some(dir) = file.parent() {
                std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            }
            // Write then rename, so a crash never leaves half a jar.
            let tmp = file.with_extension("enc.tmp");
            std::fs::write(&tmp, blob).map_err(|e| e.to_string())?;
            std::fs::rename(&tmp, file).map_err(|e| e.to_string())
        });
    if let Err(e) = result {
        tracing::warn!("saving the YouTube Music session failed: {e}");
    }
}

fn load_jar(file: &Path) -> Option<Vec<Cookie>> {
    let blob = std::fs::read(file).ok()?;
    let plain = secure::decrypt(&blob).ok()?;
    serde_json::from_slice(&plain).ok()
}

/// Called once at startup: restore the saved session and start the keeper.
pub fn setup(app: &AppHandle, state: &AppState, data_dir: &Path) -> MusicStore {
    let store = MusicStore {
        dir: data_dir.join("music"),
        account: Mutex::new(None),
        expired: AtomicBool::new(false),
        keeper_running: AtomicBool::new(false),
        signing_in: AtomicBool::new(false),
    };
    let music = state.engine.music();
    if let Ok(v) = std::fs::read_to_string(store.visitor_file()) {
        music.set_visitor_data(Some(v.trim().to_string()).filter(|v| !v.is_empty()));
    }
    if let Some(cookies) = load_jar(&store.jar_file()) {
        music.set_cookies(cookies);
    }
    let jar_file = store.jar_file();
    music.set_on_rotate(Some(Arc::new(move |cookies: &[Cookie]| save_jar(&jar_file, cookies))));
    if music.signed_in() {
        start_keeper(app.clone());
    }
    store
}

fn convert(list: &[tauri::webview::Cookie<'static>]) -> Vec<Cookie> {
    list.iter()
        .map(|c| Cookie {
            name: c.name().to_string(),
            value: c.value().to_string(),
            domain: c.domain().unwrap_or("").to_string(),
            path: c.path().unwrap_or("/").to_string(),
            expires: c.expires_datetime().map(|d| d.unix_timestamp()),
            secure: c.secure().unwrap_or(false),
            http_only: c.http_only().unwrap_or(false),
        })
        .collect()
}

fn on_domain(c: &tauri::webview::Cookie<'static>, domain: &str) -> bool {
    c.domain().is_some_and(|d| d.trim_start_matches('.').ends_with(domain))
}

fn has_youtube_auth(list: &[tauri::webview::Cookie<'static>]) -> bool {
    list.iter().any(|c| matches!(c.name(), "__Secure-1PSID" | "SAPISID") && on_domain(c, "youtube.com"))
}

fn has_google_auth(list: &[tauri::webview::Cookie<'static>]) -> bool {
    list.iter().any(|c| matches!(c.name(), "SAPISID" | "SID" | "__Secure-1PSID") && on_domain(c, "google.com"))
}

/// LOGIN_INFO comes last in YouTube's sign-in handshake.
fn has_login_info(list: &[tauri::webview::Cookie<'static>]) -> bool {
    list.iter().any(|c| c.name() == "LOGIN_INFO" && on_domain(c, "youtube.com"))
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MusicStatus {
    signed_in: bool,
    /// Google ended the session; sign in again.
    expired: bool,
    signing_in: bool,
    account: Option<Account>,
}

fn status(state: &AppState, store: &MusicStore) -> MusicStatus {
    MusicStatus {
        signed_in: state.engine.music().signed_in(),
        expired: store.expired.load(Ordering::Relaxed),
        signing_in: store.signing_in.load(Ordering::Relaxed),
        account: store.account.lock().unwrap().clone(),
    }
}

fn emit_status(app: &AppHandle) {
    let (Some(state), Some(store)) = (app.try_state::<AppState>(), app.try_state::<MusicStore>()) else { return };
    let _ = app.emit("music-status", status(&state, &store));
}

#[tauri::command]
pub fn music_status(state: State<'_, AppState>, store: State<'_, MusicStore>) -> MusicStatus {
    status(&state, &store)
}

/// The signed-in account's name and photo.
#[tauri::command]
pub async fn music_account(
    app: AppHandle,
    state: State<'_, AppState>,
    store: State<'_, MusicStore>,
) -> Result<Account, String> {
    match state.engine.music().account().await {
        Ok(a) => {
            *store.account.lock().unwrap() = Some(a.clone());
            Ok(a)
        }
        Err(e) => {
            // A 401 or an empty account menu means Google ended the session.
            if matches!(e, universeapi::Error::Http { status: 401 | 403, .. } | universeapi::Error::Config(_)) {
                store.expired.store(true, Ordering::Relaxed);
                emit_status(&app);
            }
            Err(e.to_string())
        }
    }
}

/// Open Google's sign-in page. The result arrives as a `music-status` event.
#[tauri::command]
pub async fn music_sign_in(app: AppHandle, store: State<'_, MusicStore>) -> Result<(), String> {
    if let Some(w) = app.get_webview_window(LOGIN_LABEL) {
        let _ = w.set_focus();
        return Ok(());
    }
    // The keeper holds the same profile; WebView2 wants one owner at a time
    // when the login window starts fresh.
    if let Some(k) = app.get_webview_window(KEEPER_LABEL) {
        let _ = k.close();
    }
    let win = login_window(&app, &store)?;
    store.signing_in.store(true, Ordering::Relaxed);
    emit_status(&app);

    let app2 = app.clone();
    tauri::async_runtime::spawn(async move {
        let done = watch_sign_in(&app2, win).await;
        if let Some(store) = app2.try_state::<MusicStore>() {
            store.signing_in.store(false, Ordering::Relaxed);
            if done {
                store.expired.store(false, Ordering::Relaxed);
            }
        }
        if done {
            start_keeper(app2.clone());
        }
        emit_status(&app2);
    });
    Ok(())
}

/// Poll the sign-in window until YouTube's cookies are there or it closes.
async fn watch_sign_in(app: &AppHandle, _win: WebviewWindow) -> bool {
    let mut nudged = false;
    let mut grace = 0u8;
    loop {
        tokio::time::sleep(TICK).await;
        let Some(win) = app.get_webview_window(LOGIN_LABEL) else { return false };
        let Ok(cookies) = win.cookies() else { continue };
        if !has_youtube_auth(&cookies) {
            // Google sometimes parks the window on a Google page and never
            // follows the continue link. Replaying the sign-in URL lets
            // Google hand the session to youtube.com itself.
            let host = win.url().ok().and_then(|u| u.host_str().map(str::to_string)).unwrap_or_default();
            let in_flow = host == "accounts.google.com" || host.ends_with("youtube.com");
            if !nudged && has_google_auth(&cookies) && !in_flow {
                if let Ok(u) = SERVICE_LOGIN_URL.parse::<tauri::Url>() {
                    let _ = win.navigate(u);
                }
                nudged = true;
            }
            continue;
        }
        // Give YouTube's handshake a few ticks to finish so the jar looks
        // like the browser session Google issued.
        if !has_login_info(&cookies) && grace < 4 {
            grace += 1;
            continue;
        }
        let jar = convert(&cookies);
        let Some(state) = app.try_state::<AppState>() else { return false };
        let music = state.engine.music();
        music.set_cookies(jar);
        if let Some(store) = app.try_state::<MusicStore>() {
            save_jar(&store.jar_file(), &music.cookies());
        }
        let _ = win.close();
        return music.signed_in();
    }
}

fn start_keeper(app: AppHandle) {
    let Some(store) = app.try_state::<MusicStore>() else {
        // Setup runs before the store is managed; try again shortly.
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(Duration::from_secs(5)).await;
            start_keeper(app);
        });
        return;
    };
    if store.keeper_running.swap(true, Ordering::SeqCst) {
        return;
    }
    tauri::async_runtime::spawn(async move {
        // Let launch settle first; the saved jar works in the meantime.
        tokio::time::sleep(Duration::from_secs(20)).await;
        loop {
            let signed_in = app.try_state::<AppState>().is_some_and(|s| s.engine.music().signed_in());
            if !signed_in {
                break;
            }
            match refresh(&app).await {
                Ok(true) => {}
                Ok(false) => {
                    if let Some(store) = app.try_state::<MusicStore>() {
                        store.expired.store(true, Ordering::Relaxed);
                    }
                    emit_status(&app);
                    break;
                }
                Err(e) => tracing::debug!("music keeper: {e}"),
            }
            tokio::time::sleep(KEEP_EVERY).await;
        }
        if let Some(store) = app.try_state::<MusicStore>() {
            store.keeper_running.store(false, Ordering::SeqCst);
        }
    });
}

/// Reload YouTube Music in the hidden keeper window and take its fresh
/// cookies. `Ok(false)` means Google signed the profile out.
async fn refresh(app: &AppHandle) -> Result<bool, String> {
    if app.get_webview_window(LOGIN_LABEL).is_some() {
        return Ok(true);
    }
    let store = app.try_state::<MusicStore>().ok_or("no music store")?;
    let before = KEEPER_LOADS.load(Ordering::Relaxed);
    let win = match app.get_webview_window(KEEPER_LABEL) {
        Some(w) => {
            let _ = w.navigate(MUSIC_URL.parse::<tauri::Url>().map_err(|e| e.to_string())?);
            w
        }
        None => keeper_window(app, &store)?,
    };

    let mut loaded = false;
    let mut signed_out_ticks = 0u8;
    for _ in 0..24 {
        tokio::time::sleep(TICK).await;
        loaded |= KEEPER_LOADS.load(Ordering::Relaxed) > before;
        if !loaded {
            continue;
        }
        let Ok(cookies) = win.cookies() else { continue };
        if has_youtube_auth(&cookies) {
            let state = app.try_state::<AppState>().ok_or("no app state")?;
            let music = state.engine.music();
            music.set_cookies(convert(&cookies));
            save_jar(&store.jar_file(), &music.cookies());
            if let Some(v) = music.visitor_data() {
                let _ = std::fs::write(store.visitor_file(), v);
            }
            return Ok(true);
        }
        signed_out_ticks += 1;
        if signed_out_ticks >= 4 {
            return Ok(false);
        }
    }
    Err("the keeper page didn't load".into())
}

/// The sign-in window: Google's own page on its own WebView2 profile.
#[cfg(desktop)]
fn login_window(app: &AppHandle, store: &MusicStore) -> Result<WebviewWindow, String> {
    let url = SERVICE_LOGIN_URL.parse::<tauri::Url>().map_err(|e| e.to_string())?;
    WebviewWindowBuilder::new(app, LOGIN_LABEL, WebviewUrl::External(url))
        .title("Sign in to YouTube Music")
        .inner_size(500.0, 720.0)
        .min_inner_size(420.0, 560.0)
        .center()
        .data_directory(store.profile_dir())
        .user_agent(LOGIN_UA)
        .additional_browser_args(WEBVIEW_ARGS)
        // The real host in the title, so a redirect somewhere odd is visible.
        .on_page_load(|w, payload| {
            let host = payload.url().host_str().unwrap_or("?").to_string();
            let _ = w.set_title(&format!("Sign in · {host}"));
        })
        .build()
        .map_err(|e| e.to_string())
}

/// The hidden keeper window on the sign-in profile.
#[cfg(desktop)]
fn keeper_window(app: &AppHandle, store: &MusicStore) -> Result<WebviewWindow, String> {
    let url = MUSIC_URL.parse::<tauri::Url>().map_err(|e| e.to_string())?;
    WebviewWindowBuilder::new(app, KEEPER_LABEL, WebviewUrl::External(url))
        .title("YouTube Music session")
        .visible(false)
        .skip_taskbar(true)
        .focused(false)
        .inner_size(800.0, 600.0)
        .data_directory(store.profile_dir())
        .user_agent(LOGIN_UA)
        .additional_browser_args(WEBVIEW_ARGS)
        .on_page_load(|_, payload| {
            if payload.event() == PageLoadEvent::Finished {
                KEEPER_LOADS.fetch_add(1, Ordering::Relaxed);
            }
        })
        .build()
        .map_err(|e| e.to_string())
}

// Android has a single window. Sign-in there moves to a native sheet (next step).
#[cfg(mobile)]
fn login_window(_app: &AppHandle, _store: &MusicStore) -> Result<WebviewWindow, String> {
    Err("Music sign-in isn't ready on Android yet".into())
}

#[cfg(mobile)]
fn keeper_window(_app: &AppHandle, _store: &MusicStore) -> Result<WebviewWindow, String> {
    Err("the Music keeper isn't ready on Android yet".into())
}

/// Forget the session: the jar, the profile and the account.
#[tauri::command]
pub async fn music_sign_out(app: AppHandle, state: State<'_, AppState>, store: State<'_, MusicStore>) -> Result<(), String> {
    for label in [KEEPER_LABEL, LOGIN_LABEL] {
        if let Some(w) = app.get_webview_window(label) {
            let _ = w.close();
        }
    }
    state.engine.music().clear_cookies();
    *store.account.lock().unwrap() = None;
    store.expired.store(false, Ordering::Relaxed);
    let _ = std::fs::remove_file(store.jar_file());
    // WebView2 lets go of the profile folder a moment after the window closes.
    let profile = store.profile_dir();
    tauri::async_runtime::spawn(async move {
        for _ in 0..5 {
            tokio::time::sleep(Duration::from_secs(1)).await;
            if std::fs::remove_dir_all(&profile).is_ok() || !profile.exists() {
                break;
            }
        }
    });
    emit_status(&app);
    Ok(())
}

// ── Engine calls ────────────────────────────────────────────────────

fn err(e: universeapi::Error) -> String {
    e.to_string()
}

#[tauri::command]
pub async fn music_home(state: State<'_, AppState>, next: Option<String>) -> Result<FeedPage, String> {
    state.engine.music().home(next.as_deref()).await.map_err(err)
}

#[tauri::command]
pub async fn music_feed(
    state: State<'_, AppState>,
    browse_id: String,
    params: Option<String>,
    next: Option<String>,
) -> Result<FeedPage, String> {
    state.engine.music().feed(&browse_id, params.as_deref(), next.as_deref()).await.map_err(err)
}

#[tauri::command]
pub async fn music_search(state: State<'_, AppState>, query: String, filter: SearchFilter) -> Result<SearchResults, String> {
    state.engine.music().search(&query, filter).await.map_err(err)
}

#[tauri::command]
pub async fn music_album(state: State<'_, AppState>, id: String) -> Result<Album, String> {
    state.engine.music().album(&id).await.map_err(err)
}

#[tauri::command]
pub async fn music_artist(state: State<'_, AppState>, id: String) -> Result<Artist, String> {
    state.engine.music().artist(&id).await.map_err(err)
}

#[tauri::command]
pub async fn music_playlist(state: State<'_, AppState>, id: String) -> Result<Playlist, String> {
    state.engine.music().playlist(&id).await.map_err(err)
}

#[tauri::command]
pub async fn music_playlist_more(state: State<'_, AppState>, token: String) -> Result<TrackPage, String> {
    state.engine.music().playlist_more(&token).await.map_err(err)
}

#[tauri::command]
pub async fn music_queue(
    state: State<'_, AppState>,
    video_id: Option<String>,
    playlist_id: Option<String>,
    params: Option<String>,
) -> Result<TrackPage, String> {
    state.engine.music().queue(video_id.as_deref(), playlist_id.as_deref(), params.as_deref()).await.map_err(err)
}

#[tauri::command]
pub async fn music_radio(state: State<'_, AppState>, video_id: String) -> Result<TrackPage, String> {
    state.engine.music().radio(&video_id).await.map_err(err)
}

#[tauri::command]
pub async fn music_queue_more(state: State<'_, AppState>, token: String) -> Result<TrackPage, String> {
    state.engine.music().queue_more(&token).await.map_err(err)
}

#[tauri::command]
pub async fn music_lyrics(
    state: State<'_, AppState>,
    video_id: String,
    title: String,
    artist: String,
    duration: Option<u32>,
) -> Result<Option<Lyrics>, String> {
    state.engine.music().lyrics(&video_id, &title, &artist, duration).await.map_err(err)
}

#[tauri::command]
pub async fn music_liked(state: State<'_, AppState>, limit: Option<usize>) -> Result<Playlist, String> {
    state.engine.music().liked_songs(limit.unwrap_or(2000)).await.map_err(err)
}

#[tauri::command]
pub async fn music_library(state: State<'_, AppState>, kind: LibraryKind) -> Result<Vec<MusicItem>, String> {
    state.engine.music().library(kind).await.map_err(err)
}

#[tauri::command]
pub async fn music_own_playlists(state: State<'_, AppState>) -> Result<Vec<MusicItem>, String> {
    state.engine.music().own_playlists().await.map_err(err)
}

#[tauri::command]
pub async fn music_rate(state: State<'_, AppState>, video_id: String, rating: Rating) -> Result<(), String> {
    state.engine.music().rate(&video_id, rating).await.map_err(err)
}

#[tauri::command]
pub async fn music_playlist_add(
    state: State<'_, AppState>,
    playlist_id: String,
    video_ids: Vec<String>,
) -> Result<(), String> {
    state.engine.music().playlist_add(&playlist_id, &video_ids).await.map_err(err)
}

#[tauri::command]
pub async fn music_playlist_remove(
    state: State<'_, AppState>,
    playlist_id: String,
    video_id: String,
    set_video_id: String,
) -> Result<(), String> {
    state.engine.music().playlist_remove(&playlist_id, &video_id, &set_video_id).await.map_err(err)
}

#[tauri::command]
pub async fn music_playlist_create(
    state: State<'_, AppState>,
    title: String,
    video_ids: Vec<String>,
) -> Result<String, String> {
    state.engine.music().playlist_create(&title, &video_ids).await.map_err(err)
}

#[tauri::command]
pub async fn music_subscribe(state: State<'_, AppState>, channel_id: String, on: bool) -> Result<(), String> {
    state.engine.music().subscribe(&channel_id, on).await.map_err(err)
}

/// Stream URLs for the player, resolved without the user's cookies.
#[tauri::command]
pub async fn music_stream(
    state: State<'_, AppState>,
    video_id: String,
    video: bool,
    max_height: Option<u32>,
    quality: Option<AudioQuality>,
) -> Result<Resolved, String> {
    let mode = if video { StreamMode::Video } else { StreamMode::Audio };
    state
        .engine
        .music()
        .resolve_with(&video_id, mode, max_height.unwrap_or(1080), quality.unwrap_or_default())
        .await
        .map_err(err)
}

#[tauri::command]
pub async fn music_ytdlp_status(state: State<'_, AppState>) -> Result<YtDlpStatus, String> {
    Ok(state.engine.music().ytdlp().status().await)
}

/// Install yt-dlp if missing, otherwise update it now.
#[tauri::command]
pub async fn music_ytdlp_update(state: State<'_, AppState>) -> Result<YtDlpStatus, String> {
    let y = state.engine.music().ytdlp();
    if y.installed() {
        y.update().await.map_err(err)?;
    } else {
        y.ensure().await.map_err(err)?;
    }
    Ok(y.status().await)
}

// ── Downloads ───────────────────────────────────────────────────────
//
// Songs download like videos: through the engine's queue, saved as
// Velmora-only .vlm files, in a "Music" folder inside the download folder.
// A small index keeps each song's details (cover, artists, album) so the
// Downloaded tab looks like the rest of Music offline.

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct IndexEntry {
    file_title: String,
    item: MusicItem,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineSong {
    item: MusicItem,
    path: String,
    size: u64,
}

fn index_file(store: &MusicStore) -> PathBuf {
    store.dir.join("downloads.json")
}

fn read_index(store: &MusicStore) -> Vec<IndexEntry> {
    std::fs::read(index_file(store)).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
}

fn write_index(store: &MusicStore, list: &[IndexEntry]) {
    if let Ok(text) = serde_json::to_vec(list) {
        let _ = std::fs::create_dir_all(&store.dir);
        let _ = std::fs::write(index_file(store), text);
    }
}

fn music_dir(state: &AppState) -> PathBuf {
    state.engine.download_dir().join("Music")
}

/// "Daft Punk - Get Lucky", safe for a file name.
fn file_title(item: &MusicItem) -> String {
    let by = item.artist_names();
    let raw = if by.is_empty() { item.title.clone() } else { format!("{by} - {}", item.title) };
    raw.chars().map(|c| if r#"<>:"/\|?*"#.contains(c) || c.is_control() { ' ' } else { c }).collect::<String>().trim().to_string()
}

fn norm(s: &str) -> String {
    s.to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect()
}

/// Download a song for offline listening.
#[tauri::command]
pub async fn music_download(
    state: State<'_, AppState>,
    store: State<'_, MusicStore>,
    item: MusicItem,
) -> Result<u64, String> {
    use universeapi::model::{ProviderId, Source, StreamFormat};
    let resolved = state.engine.music().resolve(&item.id, StreamMode::Audio, 0).await.map_err(err)?;
    let title = file_title(&item);
    let mut source = Source::new(ProviderId::YoutubeMusic, "YouTube Music", title.clone(), resolved.audio_url.clone());
    source.headers = resolved.headers.iter().cloned().collect();
    source.format = match resolved.ext.as_deref() {
        Some("m4a") | Some("mp4") => StreamFormat::Mp4,
        _ => StreamFormat::Webm,
    };
    source.size_bytes = resolved.filesize;
    let request = universeapi::download::DownloadRequest {
        source,
        title: title.clone(),
        dest_dir: Some(music_dir(&state)),
        episode: None,
    };
    let id = state.engine.downloads().enqueue(request).await;
    let mut index = read_index(&store);
    index.retain(|e| e.item.id != item.id);
    index.push(IndexEntry { file_title: title, item });
    write_index(&store, &index);
    Ok(id)
}

/// Songs saved on this PC, with their details when the index has them.
#[tauri::command]
pub fn music_downloads(state: State<'_, AppState>, store: State<'_, MusicStore>) -> Vec<OfflineSong> {
    let index = read_index(&store);
    let Ok(entries) = std::fs::read_dir(music_dir(&state)) else { return Vec::new() };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_string();
        if !path.is_file() || name.ends_with(".part") || name.contains(".part.") {
            continue;
        }
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
        // "Title (2)" is a second copy of "Title".
        let base = stem.rsplit_once(" (").filter(|(_, n)| n.trim_end_matches(')').parse::<u32>().is_ok()).map_or(stem, |(b, _)| b);
        let Some(e) = index.iter().find(|e| norm(&e.file_title) == norm(base)) else { continue };
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        out.push(OfflineSong { item: e.item.clone(), path: path.display().to_string(), size });
    }
    out
}

/// Delete a downloaded song.
#[tauri::command]
pub fn music_download_delete(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let file = PathBuf::from(&path);
    // Only files inside the Music download folder.
    let dir = music_dir(&state);
    let inside = file.parent().and_then(|p| p.canonicalize().ok()) == dir.canonicalize().ok();
    if !inside {
        return Err("That file isn't a downloaded song.".into());
    }
    std::fs::remove_file(&file).map_err(|e| e.to_string())
}
