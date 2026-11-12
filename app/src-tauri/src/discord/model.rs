//! What Velmora is doing right now, and how it turns into a Discord activity.
//!
//! One `Presence` is the single source of truth. Both connection modes (the
//! local Discord-app pipe, and later the direct connection) build the activity
//! from the same `activity()` here, so they always show the same thing.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Discord's activity types. We only ever use Watching and Listening, plus
/// Playing for the optional "browsing" status.
#[derive(Clone, Copy)]
enum Kind {
    Playing = 0,
    Listening = 2,
    Watching = 3,
}

/// A row from the app: the whole of what we might show. The frontend sends one
/// of these on every change; `None` means "nothing playing".
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase", tag = "kind")]
pub enum Presence {
    /// Nothing is playing and browsing status is off.
    #[default]
    Idle,
    /// A movie, series episode or anime episode in the video player.
    Video {
        /// The work's name ("Dune: Part Two", "The Bear").
        title: String,
        /// "movie", "series", "anime", "asian_drama".
        media: String,
        /// "S2 · E5 · Pasta", "Episode 12 · Sub", or the year for a movie.
        line2: String,
        /// Poster or episode still URL.
        art: Option<String>,
        /// Hover text over the poster.
        art_text: Option<String>,
        /// Seconds into the video and the full length; drives the bar.
        position: f64,
        duration: f64,
        paused: bool,
        /// A link for the "details" line, when we have one (TMDB, AniList).
        link: Option<String>,
    },
    /// A song or music video.
    Music {
        title: String,
        artist: String,
        album: Option<String>,
        art: Option<String>,
        position: f64,
        duration: f64,
        paused: bool,
        /// Whether this is a music video rather than a song.
        video: bool,
        link: Option<String>,
    },
    /// A live match you are watching.
    Match {
        /// "Real Madrid vs Bayern".
        title: String,
        /// "2 - 1 · 71'".
        line2: String,
        /// Competition name, shown on hover.
        league: Option<String>,
        /// A crest URL.
        art: Option<String>,
        /// Kickoff in Unix seconds, for the match clock.
        start: Option<i64>,
    },
    /// A live TV channel.
    LiveTv {
        channel: String,
        /// "Live TV · France".
        line2: String,
        art: Option<String>,
        /// When this channel started playing, in Unix seconds.
        start: i64,
    },
    /// Nothing playing, but the app is open and browsing status is on.
    Browsing {
        /// "Browsing movies", "Exploring music".
        line: String,
        /// When the app opened / went idle, in Unix seconds.
        start: i64,
    },
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// Discord limits each visible string to 128 bytes. Cut on a char boundary and
/// add an ellipsis so a long title never breaks the payload.
fn clip(s: &str) -> String {
    const MAX: usize = 128;
    if s.len() <= MAX {
        return s.to_string();
    }
    let mut end = MAX - 1;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &s[..end])
}

/// Discord needs details/state to be at least 2 chars when present.
fn pad(s: &str) -> String {
    let s = s.trim();
    if s.len() < 2 {
        format!("{s} ")
    } else {
        clip(s)
    }
}

/// Build `start`/`end` timestamps from a position so Discord draws and advances
/// the progress bar itself. A paused item gets no timestamps, so the bar stops.
fn timestamps(position: f64, duration: f64, paused: bool) -> Option<Value> {
    if paused || duration <= 0.0 {
        return None;
    }
    let start = now() - position as i64;
    let end = start + duration as i64;
    Some(json!({ "start": start * 1000, "end": end * 1000 }))
}

/// Build the small badge (the corner icon over the big picture).
fn badge(image: &str, text: &str) -> Value {
    json!({ "small_image": image, "small_text": text })
}

impl Presence {
    /// The activity object Discord expects, or `None` to clear the status.
    ///
    /// `art_url` converts a picture URL into something Discord will show. Over
    /// the local pipe that's the URL unchanged; the direct mode swaps in a
    /// proxied asset key. Returning `None` from it falls back to the logo.
    pub fn activity(&self, art_url: &dyn Fn(Option<&str>) -> Option<String>) -> Option<Value> {
        match self {
            Presence::Idle => None,
            Presence::Video { title, media, line2, art, art_text, position, duration, paused, link } => {
                let mut assets = json!({});
                merge_art(&mut assets, art_url(art.as_deref()), art_text.as_deref().unwrap_or(title));
                let (badge_img, badge_txt) = if *paused { ("pause", "Paused") } else { ("play", media_word(media)) };
                assets.as_object_mut().unwrap().extend(badge(badge_img, badge_txt).as_object().unwrap().clone());
                Some(base(
                    Kind::Watching,
                    title,
                    line2,
                    media_word(media),
                    assets,
                    timestamps(*position, *duration, *paused),
                    link.as_deref(),
                ))
            }
            Presence::Music { title, artist, album, art, position, duration, paused, video, link } => {
                let mut assets = json!({});
                merge_art(&mut assets, art_url(art.as_deref()), album.as_deref().unwrap_or(artist));
                let (badge_img, badge_txt) = if *paused { ("pause", "Paused") } else { ("play", "Playing") };
                assets.as_object_mut().unwrap().extend(badge(badge_img, badge_txt).as_object().unwrap().clone());
                let state = if *video { format!("{artist} · Video") } else { artist.clone() };
                Some(base(
                    Kind::Listening,
                    title,
                    &state,
                    "YouTube Music",
                    assets,
                    timestamps(*position, *duration, *paused),
                    link.as_deref(),
                ))
            }
            Presence::Match { title, line2, league, art, start } => {
                let mut assets = json!({});
                merge_art(&mut assets, art_url(art.as_deref()), league.as_deref().unwrap_or("Live"));
                assets.as_object_mut().unwrap().extend(badge("live", "Live").as_object().unwrap().clone());
                let ts = start.map(|s| json!({ "start": s * 1000 }));
                Some(base(Kind::Watching, title, line2, league.as_deref().unwrap_or("Sports"), assets, ts, None))
            }
            Presence::LiveTv { channel, line2, art, start } => {
                let mut assets = json!({});
                merge_art(&mut assets, art_url(art.as_deref()), channel);
                assets.as_object_mut().unwrap().extend(badge("live", "Live").as_object().unwrap().clone());
                Some(base(Kind::Watching, channel, line2, "Live TV", assets, Some(json!({ "start": start * 1000 })), None))
            }
            Presence::Browsing { line, start } => {
                let assets = json!({ "large_image": "velmora", "large_text": "Velmora" });
                Some(base(Kind::Playing, "Velmora", line, "Velmora", assets, Some(json!({ "start": start * 1000 })), None))
            }
        }
    }

    /// The same, but with no title or picture: "Watching something on Velmora".
    /// Used when the user keeps the feature on but hides the details.
    pub fn activity_hidden(&self) -> Option<Value> {
        let (kind, verb) = match self {
            Presence::Idle => return None,
            Presence::Music { .. } => (Kind::Listening, "Listening on Velmora"),
            Presence::Browsing { .. } => (Kind::Playing, "Using Velmora"),
            _ => (Kind::Watching, "Watching on Velmora"),
        };
        let assets = json!({ "large_image": "velmora", "large_text": "Velmora" });
        Some(base(kind, verb, "", "Velmora", assets, None, None))
    }
}

/// The shared shape of every activity. `details` is line 1, `state` is line 2.
fn base(kind: Kind, details: &str, state: &str, name: &str, assets: Value, ts: Option<Value>, link: Option<&str>) -> Value {
    let mut v = json!({
        "type": kind as u8,
        "name": name,
        "details": pad(details),
        "assets": assets,
        "instance": false,
    });
    let obj = v.as_object_mut().unwrap();
    if !state.trim().is_empty() {
        obj.insert("state".into(), json!(pad(state)));
    }
    if let Some(ts) = ts {
        obj.insert("timestamps".into(), ts);
    }
    // "Get Velmora" always; the content link when we have one.
    let mut buttons = vec![json!({ "label": "Get Velmora", "url": "https://getvelmora.dpdns.org" })];
    if let Some(url) = link.filter(|u| u.starts_with("https://")) {
        buttons.insert(0, json!({ "label": "More info", "url": url }));
    }
    obj.insert("buttons".into(), json!(buttons));
    v
}

fn merge_art(assets: &mut Value, url: Option<String>, text: &str) {
    let obj = assets.as_object_mut().unwrap();
    obj.insert("large_image".into(), json!(url.unwrap_or_else(|| "velmora".into())));
    obj.insert("large_text".into(), json!(clip(text)));
}

fn media_word(media: &str) -> &'static str {
    match media {
        "movie" => "Movie",
        "anime" => "Anime",
        "asian_drama" => "Drama",
        _ => "Series",
    }
}
