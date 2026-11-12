// SPDX-License-Identifier: GPL-3.0-only
// Ported in part from YTubic (https://github.com/NUber-dev/YTubic), Copyright George Shyshov.
//! Windows media controls (SMTC) for Music: the tile in the volume flyout
//! and Quick Settings, the lock screen, and the keyboard and headset media
//! keys.
//!
//! The controls are made here, bound to the main window, so Windows shows
//! them as Velmora's own (name and icon). mpv's built-in `media-controls` is
//! off for Music so there's never a second tile.
//!
//! souvlaki's controls are COM objects: not `Send`, and they must be used on
//! the thread that owns the window. They live in a main-thread thread-local,
//! and the commands below hop onto the main thread to reach them.

use serde::Deserialize;
use tauri::AppHandle;

/// What the player sends about the current song.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NowPlaying {
    title: String,
    artist: String,
    album: String,
    /// Cover URL; empty for none.
    cover: String,
    duration: f64,
    position: f64,
    paused: bool,
}

#[cfg(desktop)]
mod imp {
    use std::cell::RefCell;
    use std::time::Duration;

    use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig};
    use tauri::{AppHandle, Emitter, Manager};

    use super::NowPlaying;

    thread_local! {
        static CONTROLS: RefCell<Option<MediaControls>> = const { RefCell::new(None) };
        // Metadata last sent. Sending it again re-uploads the cover (COM work on
        // the UI thread, a dropped frame), so the regular position updates skip
        // it when nothing changed.
        static LAST: RefCell<Option<String>> = const { RefCell::new(None) };
    }

    /// Make the controls. Must run on the main thread, after the main window exists.
    pub fn init(app: &AppHandle) {
        #[cfg(windows)]
        let hwnd = app.get_webview_window("main").and_then(|w| w.hwnd().ok()).map(|h| h.0 as *mut std::ffi::c_void);
        #[cfg(not(windows))]
        let hwnd = None;

        let config = PlatformConfig { dbus_name: "velmora", display_name: "Velmora", hwnd };
        let mut controls = match MediaControls::new(config) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("media controls unavailable: {e:?}");
                return;
            }
        };
        let handle = app.clone();
        let attached = controls.attach(move |event: MediaControlEvent| {
            let send = |v: serde_json::Value| {
                let _ = handle.emit("media-control", v);
            };
            match event {
                MediaControlEvent::Play => send(serde_json::json!({ "action": "play" })),
                MediaControlEvent::Pause => send(serde_json::json!({ "action": "pause" })),
                MediaControlEvent::Toggle => send(serde_json::json!({ "action": "toggle" })),
                MediaControlEvent::Next => send(serde_json::json!({ "action": "next" })),
                MediaControlEvent::Previous => send(serde_json::json!({ "action": "previous" })),
                MediaControlEvent::Stop => send(serde_json::json!({ "action": "stop" })),
                MediaControlEvent::SetPosition(MediaPosition(d)) => {
                    send(serde_json::json!({ "action": "seek", "position": d.as_secs_f64() }))
                }
                _ => {}
            }
        });
        if let Err(e) = attached {
            tracing::warn!("media controls couldn't attach: {e:?}");
            return;
        }
        CONTROLS.with(|c| *c.borrow_mut() = Some(controls));
    }

    pub fn apply(app: &AppHandle, now: NowPlaying) {
        let NowPlaying { title, artist, album, cover, duration, position, paused } = now;
        CONTROLS.with(|cell| {
            let mut cell = cell.borrow_mut();
            let Some(controls) = cell.as_mut() else { return };
            let sig = format!("{title}\u{1}{artist}\u{1}{album}\u{1}{cover}\u{1}{duration:.0}");
            let changed = LAST.with(|m| {
                let mut m = m.borrow_mut();
                let fresh = m.as_deref() != Some(sig.as_str());
                if fresh {
                    *m = Some(sig);
                }
                fresh
            });
            if changed {
                // The window title is what the taskbar preview and Alt+Tab show.
                set_title(app, &if artist.is_empty() { title.clone() } else { format!("{title} - {artist}") });
                let _ = controls.set_metadata(MediaMetadata {
                    title: Some(&title),
                    artist: (!artist.is_empty()).then_some(artist.as_str()),
                    album: (!album.is_empty()).then_some(album.as_str()),
                    cover_url: (!cover.is_empty()).then_some(cover.as_str()),
                    duration: (duration > 0.0).then(|| Duration::from_secs_f64(duration)),
                });
            }
            let progress = Some(MediaPosition(Duration::from_secs_f64(position.max(0.0))));
            let _ = controls.set_playback(if paused { MediaPlayback::Paused { progress } } else { MediaPlayback::Playing { progress } });
        });
    }

    pub fn clear(app: &AppHandle) {
        LAST.with(|m| *m.borrow_mut() = None);
        set_title(app, "Velmora");
        CONTROLS.with(|cell| {
            if let Some(controls) = cell.borrow_mut().as_mut() {
                let _ = controls.set_playback(MediaPlayback::Stopped);
            }
        });
    }

    fn set_title(app: &AppHandle, title: &str) {
        if let Some(w) = app.get_webview_window("main") {
            let _ = w.set_title(title);
        }
    }
}

/// Make the controls (desktop). Call from `setup`, after the main window.
pub fn init(app: &AppHandle) {
    #[cfg(desktop)]
    imp::init(app);
    #[cfg(mobile)]
    let _ = app;
}

/// The current song and whether it's playing.
#[tauri::command]
pub fn media_update(app: AppHandle, now: NowPlaying) {
    #[cfg(desktop)]
    {
        let handle = app.clone();
        let _ = app.run_on_main_thread(move || imp::apply(&handle, now));
    }
    #[cfg(mobile)]
    let _ = (app, now);
}

/// Nothing is playing (Music closed, or a movie took over).
#[tauri::command]
pub fn media_clear(app: AppHandle) {
    #[cfg(desktop)]
    {
        let handle = app.clone();
        let _ = app.run_on_main_thread(move || imp::clear(&handle));
    }
    #[cfg(mobile)]
    let _ = app;
}
