//! Discord Rich Presence.
//!
//! The pages report what's on screen (`discord_set`), Settings sends the
//! user's choices (`discord_configure`), and one background task turns the
//! two into an activity and keeps Discord in step.
//!
//! Phase 1 talks to the Discord app over its local pipe (see `ipc`). The
//! sender only cares about "the activity we want now", so a second transport
//! can be added beside it without touching the pages.

mod ipc;
mod model;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::watch;

pub use model::Presence;

/// Velmora's Discord application. Public (it's in every Rich Presence
/// payload); `VELMORA_DISCORD_APP_ID` at build time overrides it.
const APP_ID: Option<&str> = match option_env!("VELMORA_DISCORD_APP_ID") {
    Some(id) => Some(id),
    None => Some("1533360080928378930"),
};
/// How often to look for the Discord app while it isn't running.
const RETRY: Duration = Duration::from_secs(15);
/// Discord allows about 5 updates per 20 s; stay well under it.
const MIN_GAP: Duration = Duration::from_secs(4);
/// A progress bar whose start moved by less than this is the same playback,
/// not a seek, so it isn't worth an update.
const DRIFT_MS: i64 = 3_000;

/// Which part of the app a presence comes from. Earlier wins.
const SOURCES: [&str; 4] = ["player", "music", "match", "browse"];

fn app_id() -> Option<&'static str> {
    APP_ID.map(str::trim).filter(|s| !s.is_empty())
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    pub enabled: bool,
    pub movies: bool,
    pub series: bool,
    pub anime: bool,
    pub music: bool,
    pub sports: bool,
    pub live_tv: bool,
    pub browsing: bool,
    /// Off: friends only see "Watching on Velmora", no title or picture.
    pub show_details: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: false,
            movies: true,
            series: true,
            anime: true,
            music: true,
            sports: true,
            live_tv: true,
            browsing: false,
            show_details: true,
        }
    }
}

impl Config {
    fn allows(&self, p: &Presence) -> bool {
        match p {
            Presence::Idle => false,
            Presence::Video { media, .. } => match media.as_str() {
                "movie" => self.movies,
                "anime" => self.anime,
                _ => self.series,
            },
            Presence::Music { .. } => self.music,
            Presence::Match { .. } => self.sports,
            Presence::LiveTv { .. } => self.live_tv,
            Presence::Browsing { .. } => self.browsing,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// "unavailable" (no App ID in this build), "off", "waiting" (Discord app
    /// not running), or "connected".
    pub state: &'static str,
}

struct Inner {
    config: Config,
    slots: HashMap<&'static str, Presence>,
    /// The last activity handed to the sender, for the drift check.
    last: Option<Value>,
}

pub struct Discord {
    inner: Mutex<Inner>,
    /// `None` means "clear"; the outer Option is "feature off, disconnect".
    want: watch::Sender<Option<Option<Value>>>,
    status: Mutex<Status>,
}

impl Discord {
    pub fn new() -> Arc<Self> {
        let (want, _) = watch::channel(None);
        Arc::new(Self {
            inner: Mutex::new(Inner { config: Config::default(), slots: HashMap::new(), last: None }),
            want,
            status: Mutex::new(Status { state: if app_id().is_some() { "off" } else { "unavailable" } }),
        })
    }

    /// Work out the activity from the slots and settings, and hand it to the
    /// sender if it changed.
    fn recompute(&self) {
        let mut inner = self.inner.lock().unwrap();
        if !inner.config.enabled {
            inner.last = None;
            self.want.send_replace(None);
            return;
        }
        let config = inner.config.clone();
        let picked = SOURCES.iter().find_map(|s| inner.slots.get(s).filter(|p| config.allows(p)));
        let next = picked.and_then(|p| {
            if config.show_details {
                // Over the local pipe Discord takes picture URLs as they are.
                p.activity(&|url| url.filter(|u| u.starts_with("https://") && u.len() <= 256).map(str::to_string))
            } else {
                p.activity_hidden()
            }
        });
        let next = steady(inner.last.as_ref(), next);
        if inner.last != next {
            inner.last = next.clone();
            self.want.send_replace(Some(next));
        } else if self.want.borrow().is_none() {
            // Just switched on with nothing changed: still tell the sender.
            self.want.send_replace(Some(next));
        }
    }

    fn set_status(&self, app: &AppHandle, state: &'static str) {
        let mut s = self.status.lock().unwrap();
        if s.state != state {
            s.state = state;
            let _ = app.emit("discord-status", s.clone());
        }
    }
}

/// Keep the old timestamps when a playing item only drifted, so a report every
/// few seconds doesn't become an update every few seconds. A real seek, pause,
/// or new item still goes through.
fn steady(old: Option<&Value>, new: Option<Value>) -> Option<Value> {
    let Some(old) = old else { return new };
    let Some(n) = new.as_ref() else { return None };
    let without = |v: &Value| {
        let mut v = v.clone();
        if let Some(o) = v.as_object_mut() {
            o.remove("timestamps");
        }
        v
    };
    if without(old) != without(n) {
        return new;
    }
    let at = |v: &Value, k: &str| v.get("timestamps").and_then(|t| t.get(k)).and_then(Value::as_i64);
    let close = |k: &str| match (at(old, k), at(n, k)) {
        (Some(a), Some(b)) => (a - b).abs() < DRIFT_MS,
        (None, None) => true,
        _ => false,
    };
    if close("start") && close("end") {
        Some(old.clone())
    } else {
        new
    }
}

/// Start the background sender. Call once at setup.
pub fn start(app: AppHandle, discord: Arc<Discord>) {
    let Some(id) = app_id() else { return };
    let rx = discord.want.subscribe();
    tauri::async_runtime::spawn(run(app, discord, rx, id));
}

async fn run(app: AppHandle, d: Arc<Discord>, mut rx: watch::Receiver<Option<Option<Value>>>, id: &'static str) {
    loop {
        // Off: hold no connection, wait to be switched on.
        if rx.borrow_and_update().is_none() {
            d.set_status(&app, "off");
            if rx.changed().await.is_err() {
                return;
            }
            continue;
        }
        let mut pipe = match ipc::Ipc::connect(id).await {
            Ok(p) => p,
            Err(_) => {
                d.set_status(&app, "waiting");
                // Try again later, or sooner if something changes.
                tokio::select! {
                    _ = tokio::time::sleep(RETRY) => {}
                    r = rx.changed() => {
                        if r.is_err() {
                            return;
                        }
                    }
                }
                continue;
            }
        };
        d.set_status(&app, "connected");

        let mut sent: Option<Option<Value>> = None;
        let mut last_at: Option<Instant> = None;
        loop {
            let want = rx.borrow_and_update().clone();
            let Some(activity) = want else {
                // Switched off: clear and drop the pipe (Discord clears on close too).
                let _ = pipe.set_activity(None).await;
                break;
            };
            if sent.as_ref() != Some(&activity) {
                if let Some(t) = last_at {
                    let gap = t.elapsed();
                    if gap < MIN_GAP {
                        // Wait out the gap, then take whatever is newest.
                        tokio::time::sleep(MIN_GAP - gap).await;
                        continue;
                    }
                }
                if pipe.set_activity(activity.as_ref()).await.is_err() {
                    // Discord quit. Go back to looking for it.
                    break;
                }
                last_at = Some(Instant::now());
                sent = Some(activity);
            }
            if rx.changed().await.is_err() {
                return;
            }
        }
    }
}

// ── Commands ────────────────────────────────────────────────────────

#[tauri::command]
pub fn discord_configure(discord: State<'_, Arc<Discord>>, config: Config) {
    discord.inner.lock().unwrap().config = config;
    discord.recompute();
}

/// Report what a part of the app is showing; `None` clears that part.
#[tauri::command]
pub fn discord_set(discord: State<'_, Arc<Discord>>, source: String, presence: Option<Presence>) {
    let Some(key) = SOURCES.iter().find(|s| **s == source) else { return };
    {
        let mut inner = discord.inner.lock().unwrap();
        match presence {
            Some(p) if p != Presence::Idle => {
                inner.slots.insert(key, p);
            }
            _ => {
                inner.slots.remove(key);
            }
        }
    }
    discord.recompute();
}

#[tauri::command]
pub fn discord_status(discord: State<'_, Arc<Discord>>) -> Status {
    discord.status.lock().unwrap().clone()
}
