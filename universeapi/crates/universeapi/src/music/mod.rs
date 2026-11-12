// SPDX-License-Identifier: GPL-3.0-only
// Ported in part from YTubic (https://github.com/NUber-dev/YTubic), Copyright George Shyshov.
//! YouTube Music: browse, search, the library, lyrics and playback.
//!
//! Talks to YouTube Music's own API (InnerTube, the `WEB_REMIX` web client)
//! the way music.youtube.com does, signed in with the user's cookies, and
//! plays through a managed yt-dlp (see [`ytdlp`]). The approach and much of
//! the parsing are ported from YTubic (GPL-3.0-only,
//! github.com/NUber-dev/YTubic). The files in this module carry that credit.
//!
//! The app owns sign-in: it reads the cookies from a Google sign-in window,
//! hands them over with [`Music::set_cookies`], and saves what
//! [`Music::set_on_rotate`] reports when Google rotates them.
//!
//! This module has its own HTTP client with no cookie store. A second store
//! that quietly kept Google's rotated cookies apart from the jar is what made
//! YTubic sessions get revoked, so the jar here is the only one.

pub mod auth;
pub mod lyrics;
pub mod parse;
pub mod ytdlp;

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::{Error, Result};

use self::auth::{Cookie, CookieJar, ORIGIN};
pub use self::lyrics::{LyricLine, Lyrics};
use self::parse::{arr, s, text};
pub use self::ytdlp::{AudioQuality, Resolved, StreamMode, YtDlp, YtDlpStatus};

const API: &str = "https://music.youtube.com/youtubei/v1";
const CLIENT_VERSION: &str = "1.20260510.02.00";
const UA: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
/// A runaway guard for paged lists; far above any real library.
const MAX_PAGES: usize = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    Song,
    Video,
    Album,
    Playlist,
    Artist,
    /// A mood or genre tile; open it with `browse(id, params)`.
    Category,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail {
    pub url: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistRef {
    pub id: Option<String>,
    pub name: String,
}

/// A song, video, album, playlist, artist or category, as shown on a card or row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicItem {
    pub kind: ItemKind,
    /// Video ID for songs and videos, browse ID for everything else.
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub thumbnails: Vec<Thumbnail>,
    pub artists: Vec<ArtistRef>,
    pub album: Option<String>,
    pub album_id: Option<String>,
    /// Seconds.
    pub duration: Option<u32>,
    pub explicit: bool,
    pub play_count: Option<String>,
    /// A song's slot in a playlist, needed to remove it.
    pub set_video_id: Option<String>,
    /// A playlist card whose play button starts a video directly.
    pub play_video_id: Option<String>,
    /// Browse params for category tiles.
    pub params: Option<String>,
    /// A category tile's color, `#rrggbb`.
    pub tint: Option<String>,
}

impl MusicItem {
    pub fn playable(&self) -> bool {
        matches!(self.kind, ItemKind::Song | ItemKind::Video)
    }

    pub fn artist_names(&self) -> String {
        self.artists.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", ")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShelfDisplay {
    /// Song rows.
    List,
    /// A carousel of cards.
    Card,
    /// Colored category tiles.
    Grid,
}

/// Where a shelf's "More" goes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShelfMore {
    pub browse_id: String,
    pub params: Option<String>,
    pub page_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shelf {
    pub id: String,
    pub title: String,
    pub items: Vec<MusicItem>,
    pub display: ShelfDisplay,
    pub more: Option<ShelfMore>,
}

/// A page of shelves: Home, Explore, Charts, a mood, a "More" page.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedPage {
    pub title: Option<String>,
    pub shelves: Vec<Shelf>,
    /// Pass back to get the next page.
    pub next: Option<String>,
}

/// What a play, shuffle or radio button starts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchTarget {
    pub video_id: Option<String>,
    pub playlist_id: Option<String>,
    pub params: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchFilter {
    All,
    Songs,
    Videos,
    Albums,
    Artists,
    Playlists,
}

impl SearchFilter {
    fn params(self) -> Option<&'static str> {
        match self {
            Self::All => None,
            Self::Songs => Some("EgWKAQIIAWoQEAkQBRAKEAMQBBAQEBUQEQ=="),
            Self::Videos => Some("EgWKAQIQAWoQEAkQBRAKEAMQBBAQEBUQEQ=="),
            Self::Albums => Some("EgWKAQIYAWoQEAkQBRAKEAMQBBAQEBUQEQ=="),
            Self::Artists => Some("EgWKAQIgAWoQEAkQBRAKEAMQBBAQEBUQEQ=="),
            Self::Playlists => Some("EgWKAQIoAWoQEAkQBRAKEAMQBBAQEBUQEQ=="),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    pub query: String,
    pub top: Option<MusicItem>,
    pub top_action: Option<WatchTarget>,
    pub shelves: Vec<Shelf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    pub id: String,
    pub title: String,
    pub artists: Vec<ArtistRef>,
    pub year: Option<String>,
    pub track_count: Option<u32>,
    /// "45 minutes", as YouTube Music writes it.
    pub length: Option<String>,
    pub description: Option<String>,
    pub thumbnails: Vec<Thumbnail>,
    pub tracks: Vec<MusicItem>,
    /// The album's own playlist, for shuffle and radio.
    pub playlist_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub monthly_audience: Option<String>,
    pub subscribers: Option<String>,
    pub channel_id: Option<String>,
    pub subscribed: Option<bool>,
    pub thumbnails: Vec<Thumbnail>,
    pub shuffle: Option<WatchTarget>,
    pub radio: Option<WatchTarget>,
    pub shelves: Vec<Shelf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    /// Without the `VL` prefix.
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub subtitle: Option<String>,
    pub thumbnails: Vec<Thumbnail>,
    pub track_count: Option<u32>,
    pub tracks: Vec<MusicItem>,
    /// More tracks: pass to [`Music::playlist_more`].
    pub next: Option<String>,
    pub shuffle: Option<WatchTarget>,
    /// The signed-in user can edit it.
    pub editable: bool,
}

/// A page of songs: a queue, or more of a playlist.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackPage {
    pub tracks: Vec<MusicItem>,
    pub next: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub name: String,
    pub handle: Option<String>,
    pub photo: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rating {
    Like,
    Dislike,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LibraryKind {
    Playlists,
    Albums,
    Artists,
}

impl LibraryKind {
    fn browse_id(self) -> &'static str {
        match self {
            Self::Playlists => "FEmusic_liked_playlists",
            Self::Albums => "FEmusic_liked_albums",
            Self::Artists => "FEmusic_library_corpus_artists",
        }
    }
}

/// Called with the whole jar whenever Google rotates a cookie, so the app can save it.
pub type RotateHook = Arc<dyn Fn(&[Cookie]) + Send + Sync>;

pub struct Music {
    http: reqwest::Client,
    jar: Mutex<CookieJar>,
    visitor: Mutex<Option<String>>,
    on_rotate: Mutex<Option<RotateHook>>,
    ytdlp: Arc<YtDlp>,
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

impl Music {
    /// `tools_dir` is where the managed yt-dlp lives.
    pub fn new(tools_dir: PathBuf) -> Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent(UA)
            .gzip(true)
            .brotli(true)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(25))
            .build()
            .map_err(|e| Error::Config(format!("music http client: {e}")))?;
        Ok(Self {
            ytdlp: Arc::new(YtDlp::new(tools_dir, http.clone())),
            http,
            jar: Mutex::new(CookieJar::default()),
            visitor: Mutex::new(None),
            on_rotate: Mutex::new(None),
        })
    }

    // ── Session ─────────────────────────────────────────────────────

    /// Use these cookies from now on (from the sign-in window or saved).
    pub fn set_cookies(&self, cookies: Vec<Cookie>) {
        *self.jar.lock().unwrap() = CookieJar::new(cookies);
    }

    pub fn clear_cookies(&self) {
        *self.jar.lock().unwrap() = CookieJar::default();
    }

    /// The current jar, rotations included.
    pub fn cookies(&self) -> Vec<Cookie> {
        self.jar.lock().unwrap().cookies.clone()
    }

    pub fn signed_in(&self) -> bool {
        self.jar.lock().unwrap().signed_in()
    }

    pub fn set_on_rotate(&self, hook: Option<RotateHook>) {
        *self.on_rotate.lock().unwrap() = hook;
    }

    /// YouTube's visitor token, kept between launches so the first calls
    /// aren't treated as a new visitor.
    pub fn visitor_data(&self) -> Option<String> {
        self.visitor.lock().unwrap().clone()
    }

    pub fn set_visitor_data(&self, v: Option<String>) {
        *self.visitor.lock().unwrap() = v;
    }

    pub fn ytdlp(&self) -> &Arc<YtDlp> {
        &self.ytdlp
    }

    // ── InnerTube ───────────────────────────────────────────────────

    fn context(&self) -> Value {
        let mut client = json!({
            "clientName": "WEB_REMIX",
            "clientVersion": CLIENT_VERSION,
            "hl": "en",
            "gl": "US",
            "platform": "DESKTOP",
            "userAgent": format!("{UA},gzip(gfe)"),
            "originalUrl": "https://music.youtube.com/",
        });
        if let Some(v) = self.visitor_data() {
            client["visitorData"] = json!(v);
        }
        json!({ "client": client, "user": { "lockedSafetyMode": false }, "request": { "useSsl": true } })
    }

    /// POST to an InnerTube endpoint, signed in when there are cookies.
    async fn post(&self, endpoint: &str, mut body: Value) -> Result<Value> {
        let sep = if endpoint.contains('?') { '&' } else { '?' };
        let url = format!("{API}/{endpoint}{sep}prettyPrint=false");
        body["context"] = self.context();

        let (cookie, auth) = {
            let jar = self.jar.lock().unwrap();
            let t = now();
            (jar.header(t), jar.authorization(t))
        };
        let mut req = self
            .http
            .post(&url)
            .header("Content-Type", "application/json")
            .header("X-YouTube-Client-Name", "67")
            .header("X-YouTube-Client-Version", CLIENT_VERSION)
            .header("Origin", ORIGIN)
            .header("X-Origin", ORIGIN)
            .header("Referer", "https://music.youtube.com/")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("X-Goog-AuthUser", "0");
        if let Some(v) = self.visitor_data() {
            req = req.header("X-Goog-Visitor-Id", v);
        }
        if !cookie.is_empty() {
            req = req.header("Cookie", cookie);
        }
        if let Some(a) = auth {
            req = req.header("Authorization", a);
        }
        let resp =
            req.body(body.to_string()).send().await.map_err(|e| Error::Network { url: url.clone(), source: e })?;

        // Google rotates cookies on error answers too, so merge before bailing.
        self.merge_rotations(&resp);

        let status = resp.status();
        if !status.is_success() {
            return Err(Error::Http { status: status.as_u16(), url });
        }
        let json: Value = resp.json().await.map_err(|e| Error::parse("YouTube Music response", e.to_string()))?;
        if let Some(v) = s(&json, "/responseContext/visitorData") {
            *self.visitor.lock().unwrap() = Some(v.to_string());
        }
        Ok(json)
    }

    fn merge_rotations(&self, resp: &reqwest::Response) {
        let lines: Vec<&str> =
            resp.headers().get_all(reqwest::header::SET_COOKIE).iter().filter_map(|v| v.to_str().ok()).collect();
        if lines.is_empty() {
            return;
        }
        let snapshot = {
            let mut jar = self.jar.lock().unwrap();
            // Never start a session from a stray header while signed out.
            if jar.is_empty() || !jar.merge_set_cookies(lines, now()) {
                return;
            }
            jar.cookies.clone()
        };
        let hook = self.on_rotate.lock().unwrap().clone();
        if let Some(hook) = hook {
            hook(&snapshot);
        }
    }

    async fn browse(&self, browse_id: &str, params: Option<&str>) -> Result<Value> {
        let mut body = json!({ "browseId": browse_id });
        if let Some(p) = params {
            body["params"] = json!(p);
        }
        self.post("browse", body).await
    }

    async fn browse_continuation(&self, token: &str) -> Result<Value> {
        self.post("browse", json!({ "continuation": token })).await
    }

    /// Every row after the first page, following tokens until they run out.
    async fn follow(&self, first: Option<String>) -> Vec<Value> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        let mut token = first;
        for _ in 0..MAX_PAGES {
            let Some(t) = token.take() else { break };
            if !seen.insert(t.clone()) {
                break;
            }
            let Ok(json) = self.browse_continuation(&t).await else { break };
            let (items, next) = parse::continuation_page(&json);
            if items.is_empty() {
                break;
            }
            out.extend(items);
            token = next;
        }
        out
    }

    // ── Feeds ───────────────────────────────────────────────────────

    /// Music Home. Pass `next` from the last page to get more shelves.
    pub async fn home(&self, next: Option<&str>) -> Result<FeedPage> {
        self.feed("FEmusic_home", None, next).await
    }

    /// Any page of shelves: `FEmusic_explore`, `FEmusic_charts`,
    /// `FEmusic_new_releases`, `FEmusic_moods_and_genres`, a mood tile
    /// (its ID and params), or a shelf's "More".
    pub async fn feed(&self, browse_id: &str, params: Option<&str>, next: Option<&str>) -> Result<FeedPage> {
        let tag = next_tag(next, browse_id);
        let json = match next {
            Some(t) => self.browse_continuation(t).await?,
            None => self.browse(browse_id, params).await?,
        };
        let (sections, next, title): (Vec<Value>, Option<String>, Option<String>) = if next.is_some() {
            let mut sections: Vec<Value> = Vec::new();
            for a in arr(&json, "/onResponseReceivedActions") {
                sections.extend(arr(a, "/appendContinuationItemsAction/continuationItems").iter().cloned());
            }
            let legacy = json.pointer("/continuationContents/sectionListContinuation");
            if sections.is_empty() {
                if let Some(l) = legacy {
                    sections = arr(l, "/contents").to_vec();
                }
            }
            let next = trailing_token(&sections).or_else(|| legacy.and_then(parse::paging_token));
            (sections, next, None)
        } else {
            let list = json
                .pointer("/contents/singleColumnBrowseResultsRenderer/tabs/0/tabRenderer/content/sectionListRenderer")
                .or_else(|| {
                    json.pointer("/contents/twoColumnBrowseResultsRenderer/secondaryContents/sectionListRenderer")
                });
            let sections = list.map(|l| arr(l, "/contents").to_vec()).unwrap_or_default();
            let next = trailing_token(&sections).or_else(|| list.and_then(parse::paging_token));
            let title = parse::non_empty_text(json.pointer("/header/musicHeaderRenderer/title"));
            (sections, next, title)
        };

        let mut shelves = Vec::new();
        let mut titles: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for (i, node) in parse::shelf_nodes(&sections).iter().enumerate() {
            let mut shelf = parse::shelf(node, String::new(), &[]);
            if shelf.items.is_empty() {
                continue;
            }
            if shelf.title.is_empty() {
                shelf.title = format!("Section {}", i + 1);
            }
            let n = titles.entry(shelf.title.clone()).or_insert(0);
            shelf.id = if *n == 0 { format!("{tag}-{}", shelf.title) } else { format!("{tag}-{}-{n}", shelf.title) };
            *n += 1;
            shelves.push(shelf);
        }
        Ok(FeedPage { title, shelves, next })
    }

    // ── Search ──────────────────────────────────────────────────────

    pub async fn search(&self, query: &str, filter: SearchFilter) -> Result<SearchResults> {
        let query = query.trim();
        if query.is_empty() {
            return Ok(SearchResults::default());
        }
        let mut body = json!({ "query": query });
        if let Some(p) = filter.params() {
            body["params"] = json!(p);
        }
        let json = self.post("search", body).await?;
        let sections = json
            .pointer("/contents/tabbedSearchResultsRenderer/tabs/0/tabRenderer/content/sectionListRenderer/contents")
            .or_else(|| json.pointer("/contents/sectionListRenderer/contents"))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        Ok(if filter == SearchFilter::All {
            search_all(query, &sections)
        } else {
            search_filtered(query, &sections, filter)
        })
    }

    // ── Pages ───────────────────────────────────────────────────────

    pub async fn album(&self, id: &str) -> Result<Album> {
        let json = self.browse(id, None).await?;
        let header = json
            .pointer("/header/musicDetailHeaderRenderer")
            .or_else(|| json.pointer("/header/musicResponsiveHeaderRenderer"))
            .or_else(|| parse::find_key(&json, "musicResponsiveHeaderRenderer"))
            .cloned()
            .unwrap_or(Value::Null);
        let title = text(header.get("title"));
        if title.is_empty() {
            return Err(Error::NotFound(format!("album {id}")));
        }
        let thumbnails = header_thumbnails(&header);
        let mut artists = Vec::new();
        let mut year = None;
        for run in arr(&header, "/subtitle/runs").iter().chain(arr(&header, "/straplineTextOne/runs")) {
            let t = s(run, "/text").unwrap_or("").trim();
            if let Some(aid) = s(run, "/navigationEndpoint/browseEndpoint/browseId").filter(|a| a.starts_with("UC")) {
                artists.push(ArtistRef { id: Some(aid.to_string()), name: t.to_string() });
            } else if t.len() == 4 && t.chars().all(|c| c.is_ascii_digit()) {
                year = Some(t.to_string());
            }
        }
        let second = text(header.get("secondSubtitle"));
        let length = second.split('•').nth(1).map(|x| x.trim().to_string()).filter(|x| !x.is_empty());
        let description =
            parse::non_empty_text(header.pointer("/description/musicDescriptionShelfRenderer/description"))
                .or_else(|| parse::non_empty_text(header.get("description")));
        let playlist_id = parse::find(&json, |n| {
            n.pointer("/watchPlaylistEndpoint/playlistId")
                .and_then(Value::as_str)
                .is_some_and(|p| p.starts_with("OLAK"))
        })
        .and_then(|n| s(n, "/watchPlaylistEndpoint/playlistId"))
        .map(str::to_string);

        let mut seen = HashSet::new();
        let mut tracks = parse::song_rows(&json, &mut seen);
        // Album rows come without covers and often without artists: fill them from the album.
        for t in &mut tracks {
            t.thumbnails = thumbnails.clone();
            if t.artists.is_empty() {
                t.artists = artists.clone();
                t.subtitle = Some(artists.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", "));
            }
            if t.album.is_none() {
                t.album = Some(title.clone());
                t.album_id = Some(id.to_string());
            }
        }
        let counted = tracks.len() as u32;
        Ok(Album {
            id: id.to_string(),
            track_count: parse::first_number(&second).or(Some(counted)),
            title,
            artists,
            year,
            length,
            description,
            thumbnails,
            tracks,
            playlist_id,
        })
    }

    pub async fn artist(&self, id: &str) -> Result<Artist> {
        let json = self.browse(id, None).await?;
        let header = json
            .pointer("/header/musicImmersiveHeaderRenderer")
            .or_else(|| json.pointer("/header/musicDetailHeaderRenderer"))
            .or_else(|| json.pointer("/header/musicVisualHeaderRenderer"))
            .cloned()
            .unwrap_or(Value::Null);
        let name = text(header.get("title"));
        if name.is_empty() {
            return Err(Error::NotFound(format!("artist {id}")));
        }
        let sub = header.pointer("/subscriptionButton/subscribeButtonRenderer").cloned().unwrap_or(Value::Null);
        let mut thumbnails = header_thumbnails(&header);
        if thumbnails.is_empty() {
            thumbnails = parse::thumbnails(header.pointer("/foregroundThumbnail/musicThumbnailRenderer/thumbnail"));
        }
        let sections = arr(
            &json,
            "/contents/singleColumnBrowseResultsRenderer/tabs/0/tabRenderer/content/sectionListRenderer/contents",
        );
        let shelves = parse::shelf_nodes(sections)
            .iter()
            .enumerate()
            .map(|(i, n)| parse::shelf(n, format!("{id}-{i}"), &[]))
            .filter(|s| !s.items.is_empty())
            .collect();
        Ok(Artist {
            id: id.to_string(),
            name,
            description: parse::non_empty_text(header.get("description")),
            monthly_audience: parse::non_empty_text(header.get("monthlyListenerCount")),
            subscribers: parse::non_empty_text(sub.get("subscriberCountText")),
            channel_id: s(&sub, "/channelId").map(str::to_string),
            subscribed: sub.get("subscribed").and_then(Value::as_bool),
            thumbnails,
            shuffle: parse::watch_target(header.pointer("/playButton/buttonRenderer/navigationEndpoint")),
            radio: parse::watch_target(header.pointer("/startRadioButton/buttonRenderer/navigationEndpoint")),
            shelves,
        })
    }

    /// A playlist's first page. `id` may carry the `VL` prefix or not.
    pub async fn playlist(&self, id: &str) -> Result<Playlist> {
        let raw_id = id.strip_prefix("VL").unwrap_or(id);
        let json = self.browse(&format!("VL{raw_id}"), None).await?;
        let header = parse::find(&json, |n| {
            n.get("musicDetailHeaderRenderer").is_some() || n.get("musicResponsiveHeaderRenderer").is_some()
        })
        .and_then(|n| n.get("musicDetailHeaderRenderer").or_else(|| n.get("musicResponsiveHeaderRenderer")))
        .cloned()
        .unwrap_or(Value::Null);
        let mut thumbnails = header_thumbnails(&header);
        if thumbnails.is_empty() {
            thumbnails = parse::deep_thumbnails(header.get("thumbnail"));
        }
        let shelf = parse::find_key(&json, "musicPlaylistShelfRenderer");
        let scope = shelf.unwrap_or(&json);
        let mut seen = HashSet::new();
        let tracks = parse::song_rows(scope, &mut seen);
        let next = parse::find_continuation(scope);
        let editable = parse::find(&json, |n| {
            n.get("musicEditablePlaylistDetailHeaderRenderer").is_some() || n.get("editPlaylistEndpoint").is_some()
        })
        .is_some();
        let shuffle = parse::find(&json, |n| {
            n.get("watchPlaylistEndpoint").is_some_and(|ep| {
                s(ep, "/playlistId") == Some(raw_id) && s(ep, "/params").is_some_and(|p| p.contains("8gECKAE"))
            })
        })
        .and_then(|n| parse::watch_target(Some(n)));
        let title = text(header.get("title"));
        if title.is_empty() && tracks.is_empty() {
            return Err(Error::NotFound(format!("playlist {raw_id}")));
        }
        let second = text(header.get("secondSubtitle"));
        Ok(Playlist {
            id: raw_id.to_string(),
            title,
            description: parse::non_empty_text(
                header.pointer("/description/musicDescriptionShelfRenderer/description"),
            )
            .or_else(|| parse::non_empty_text(header.get("description"))),
            subtitle: parse::non_empty_text(header.get("subtitle")),
            thumbnails,
            track_count: parse::first_number(&second),
            tracks,
            next,
            shuffle,
            editable,
        })
    }

    /// More of a playlist.
    pub async fn playlist_more(&self, token: &str) -> Result<TrackPage> {
        let json = self.browse_continuation(token).await?;
        let (rows, next) = parse::continuation_page(&json);
        let mut seen = HashSet::new();
        let tracks = if rows.is_empty() {
            parse::song_rows(&json, &mut seen)
        } else {
            rows.iter()
                .filter_map(parse::any_item)
                .filter(|t| t.kind == ItemKind::Song && seen.insert(t.id.clone()))
                .collect()
        };
        Ok(TrackPage { tracks, next: next.or_else(|| parse::find_continuation(&json)) })
    }

    /// Every track of a playlist, following pages up to `limit`.
    pub async fn playlist_all(&self, id: &str, limit: usize) -> Result<Playlist> {
        let mut p = self.playlist(id).await?;
        let mut pages = 0;
        while let Some(token) = p.next.take() {
            if p.tracks.len() >= limit || pages >= MAX_PAGES {
                break;
            }
            let more = self.playlist_more(&token).await?;
            if more.tracks.is_empty() {
                break;
            }
            p.tracks.extend(more.tracks);
            p.next = more.next;
            pages += 1;
        }
        p.tracks.truncate(limit);
        Ok(p)
    }

    // ── Queues ──────────────────────────────────────────────────────

    /// Up next for a video, a playlist, or both. `params` is a shuffle
    /// target's params.
    pub async fn queue(
        &self,
        video_id: Option<&str>,
        playlist_id: Option<&str>,
        params: Option<&str>,
    ) -> Result<TrackPage> {
        let mut body = json!({ "isAudioOnly": true });
        if let Some(v) = video_id {
            body["videoId"] = json!(v);
        }
        if let Some(p) = playlist_id {
            body["playlistId"] = json!(p);
        }
        if let Some(p) = params {
            body["params"] = json!(p);
        }
        if video_id.is_none() && playlist_id.is_none() {
            return Err(Error::NotFound("a queue needs a video or a playlist".into()));
        }
        Ok(parse_queue(&self.post("next", body).await?))
    }

    /// A radio of songs like this one.
    pub async fn radio(&self, video_id: &str) -> Result<TrackPage> {
        self.queue(Some(video_id), Some(&format!("RDAMVM{video_id}")), None).await
    }

    pub async fn queue_more(&self, token: &str) -> Result<TrackPage> {
        Ok(parse_queue(&self.post("next", json!({ "continuation": token, "isAudioOnly": true })).await?))
    }

    // ── Lyrics ──────────────────────────────────────────────────────

    /// YouTube Music's lyrics, then LRCLIB's.
    pub async fn lyrics(
        &self,
        video_id: &str,
        title: &str,
        artist: &str,
        duration: Option<u32>,
    ) -> Result<Option<Lyrics>> {
        match lyrics::youtube_music(&self.http, video_id).await {
            Ok(Some(l)) => return Ok(Some(l)),
            Ok(None) => {}
            Err(e) => tracing::debug!("YouTube Music lyrics failed: {e}"),
        }
        lyrics::lrclib(&self.http, title, artist, duration).await
    }

    // ── Account and library (signed in) ─────────────────────────────

    fn need_sign_in(&self) -> Result<()> {
        if self.signed_in() {
            Ok(())
        } else {
            Err(Error::Config("sign in to YouTube Music first".into()))
        }
    }

    pub async fn account(&self) -> Result<Account> {
        self.need_sign_in()?;
        let json = self.post("account/account_menu", json!({})).await?;
        let header = json
            .pointer("/actions/0/openPopupAction/popup/multiPageMenuRenderer/header/activeAccountHeaderRenderer")
            .ok_or_else(|| Error::Config("YouTube Music didn't recognise the session; sign in again".into()))?;
        let photos = arr(header, "/accountPhoto/thumbnails");
        Ok(Account {
            name: text(header.get("accountName")),
            handle: parse::non_empty_text(header.get("channelHandle"))
                .or_else(|| parse::non_empty_text(header.get("email"))),
            photo: photos.last().and_then(|p| s(p, "/url")).map(str::to_string),
        })
    }

    /// Liked songs, every page up to `limit`.
    pub async fn liked_songs(&self, limit: usize) -> Result<Playlist> {
        self.need_sign_in()?;
        self.playlist_all("LM", limit).await
    }

    /// Saved playlists, albums or artists, every page.
    pub async fn library(&self, kind: LibraryKind) -> Result<Vec<MusicItem>> {
        self.need_sign_in()?;
        let json = self.browse(kind.browse_id(), None).await?;
        let list =
            json.pointer("/contents/singleColumnBrowseResultsRenderer/tabs/0/tabRenderer/content/sectionListRenderer");
        let sections = list.map(|l| arr(l, "/contents")).unwrap_or(&[]);
        let nodes = parse::shelf_nodes(sections);
        let mut items = Vec::new();
        let mut seen = HashSet::new();
        for (i, node) in nodes.iter().enumerate() {
            let token = parse::paging_token(node.container()).or_else(|| {
                if nodes.len() == 1 {
                    list.and_then(parse::paging_token)
                } else {
                    None
                }
            });
            let rest = self.follow(token).await;
            let shelf = parse::shelf(node, format!("library-{i}"), &rest);
            for it in shelf.items {
                // Skip the "New playlist" and "Liked music" shortcut tiles.
                if it.kind == ItemKind::Category || it.id == "VLLM" {
                    continue;
                }
                if seen.insert(it.id.clone()) {
                    items.push(it);
                }
            }
        }
        Ok(items)
    }

    /// Playlists the user can add songs to.
    pub async fn own_playlists(&self) -> Result<Vec<MusicItem>> {
        let all = self.library(LibraryKind::Playlists).await?;
        Ok(all.into_iter().filter(|p| p.kind == ItemKind::Playlist && p.id.starts_with("VLPL")).collect())
    }

    pub async fn rate(&self, video_id: &str, rating: Rating) -> Result<()> {
        self.need_sign_in()?;
        let endpoint = match rating {
            Rating::Like => "like/like",
            Rating::Dislike => "like/dislike",
            Rating::None => "like/removelike",
        };
        self.post(endpoint, json!({ "target": { "videoId": video_id } })).await.map(|_| ())
    }

    pub async fn playlist_add(&self, playlist_id: &str, video_ids: &[String]) -> Result<()> {
        self.need_sign_in()?;
        let actions: Vec<Value> = video_ids
            .iter()
            .map(|v| json!({ "action": "ACTION_ADD_VIDEO", "addedVideoId": v, "dedupeOption": "DEDUPE_OPTION_SKIP" }))
            .collect();
        let json = self
            .post(
                "browse/edit_playlist",
                json!({ "playlistId": playlist_id.strip_prefix("VL").unwrap_or(playlist_id), "actions": actions }),
            )
            .await?;
        edit_status(&json)
    }

    pub async fn playlist_remove(&self, playlist_id: &str, video_id: &str, set_video_id: &str) -> Result<()> {
        self.need_sign_in()?;
        let json = self
            .post(
                "browse/edit_playlist",
                json!({
                    "playlistId": playlist_id.strip_prefix("VL").unwrap_or(playlist_id),
                    "actions": [{ "action": "ACTION_REMOVE_VIDEO", "removedVideoId": video_id, "setVideoId": set_video_id }],
                }),
            )
            .await?;
        edit_status(&json)
    }

    /// A new private playlist. Returns its ID.
    pub async fn playlist_create(&self, title: &str, video_ids: &[String]) -> Result<String> {
        self.need_sign_in()?;
        let json = self
            .post("playlist/create", json!({ "title": title, "videoIds": video_ids, "privacyStatus": "PRIVATE" }))
            .await?;
        s(&json, "/playlistId")
            .or_else(|| s(&json, "/response/playlistId"))
            .map(str::to_string)
            .ok_or_else(|| Error::parse("playlist/create", "no playlist ID in the answer"))
    }

    pub async fn subscribe(&self, channel_id: &str, on: bool) -> Result<()> {
        self.need_sign_in()?;
        let endpoint = if on { "subscription/subscribe" } else { "subscription/unsubscribe" };
        self.post(endpoint, json!({ "channelIds": [channel_id] })).await.map(|_| ())
    }

    // ── Playback ────────────────────────────────────────────────────

    /// Stream URLs for a song or video. Never uses the user's cookies.
    pub async fn resolve(&self, video_id: &str, mode: StreamMode, max_height: u32) -> Result<Resolved> {
        self.ytdlp.resolve(video_id, mode, max_height).await
    }

    /// `resolve` at a chosen audio quality.
    pub async fn resolve_with(&self, video_id: &str, mode: StreamMode, max_height: u32, quality: AudioQuality) -> Result<Resolved> {
        self.ytdlp.resolve_with(video_id, mode, max_height, quality).await
    }
}

fn edit_status(json: &Value) -> Result<()> {
    match s(json, "/status") {
        Some(st) if st != "STATUS_SUCCEEDED" => Err(Error::Transport(format!("YouTube Music refused the edit ({st})"))),
        _ => Ok(()),
    }
}

fn trailing_token(sections: &[Value]) -> Option<String> {
    sections
        .iter()
        .find_map(|c| s(c, "/continuationItemRenderer/continuationEndpoint/continuationCommand/token"))
        .map(str::to_string)
}

/// A short, stable tag for shelf IDs on one page.
fn next_tag(next: Option<&str>, browse_id: &str) -> String {
    let base = next.unwrap_or(browse_id);
    let mut h: u32 = 5381;
    for b in base.bytes() {
        h = h.wrapping_mul(33).wrapping_add(u32::from(b));
    }
    format!("{h:x}")
}

fn header_thumbnails(header: &Value) -> Vec<Thumbnail> {
    parse::thumbnails(
        header
            .pointer("/thumbnail/musicThumbnailRenderer/thumbnail")
            .or_else(|| header.pointer("/thumbnail/croppedSquareThumbnailRenderer/thumbnail"))
            .or_else(|| header.pointer("/thumbnail/musicThumbnailRenderer"))
            .or_else(|| header.get("thumbnail")),
    )
}

fn parse_queue(json: &Value) -> TrackPage {
    let panel = json
        .pointer(
            "/contents/singleColumnMusicWatchNextResultsRenderer/tabbedRenderer/watchNextTabbedResultsRenderer/tabs/0/tabRenderer/content/musicQueueRenderer/content/playlistPanelRenderer",
        )
        .or_else(|| json.pointer("/continuationContents/playlistPanelContinuation"));
    let Some(panel) = panel else { return TrackPage::default() };
    let tracks = arr(panel, "/contents")
        .iter()
        .filter_map(|c| {
            c.get("playlistPanelVideoRenderer")
                .or_else(|| c.pointer("/playlistPanelVideoWrapperRenderer/primaryRenderer/playlistPanelVideoRenderer"))
        })
        .filter_map(parse::panel_video)
        .collect();
    let next = arr(panel, "/continuations").iter().find_map(|c| {
        s(c, "/nextContinuationData/continuation")
            .or_else(|| s(c, "/nextRadioContinuationData/continuation"))
            .map(str::to_string)
    });
    TrackPage { tracks, next }
}

/// What kind a search row is, from the first word of its subtitle.
fn search_group(row: &Value) -> Option<ItemKind> {
    let col = row.pointer("/flexColumns/1/musicResponsiveListItemFlexColumnRenderer/text");
    let first = col
        .and_then(|c| s(c, "/runs/0/text"))
        .map(|t| t.trim().to_string())
        .unwrap_or_else(|| text(col).split('•').next().unwrap_or("").trim().to_string());
    match first.as_str() {
        "Song" => Some(ItemKind::Song),
        "Video" | "Episode" => Some(ItemKind::Video),
        "Album" | "Single" | "EP" => Some(ItemKind::Album),
        "Artist" => Some(ItemKind::Artist),
        "Playlist" | "Podcast" => Some(ItemKind::Playlist),
        _ => None,
    }
}

fn group_from_title(title: &str) -> Option<ItemKind> {
    let t = title.to_lowercase();
    if t.contains("song") {
        Some(ItemKind::Song)
    } else if t.contains("video") || t.contains("episode") {
        Some(ItemKind::Video)
    } else if t.contains("album") || t.contains("single") {
        Some(ItemKind::Album)
    } else if t.contains("artist") || t.contains("profile") {
        Some(ItemKind::Artist)
    } else if t.contains("playlist") || t.contains("podcast") {
        Some(ItemKind::Playlist)
    } else {
        None
    }
}

fn search_all(query: &str, sections: &[Value]) -> SearchResults {
    let card = sections.iter().find_map(|s| s.get("musicCardShelfRenderer"));
    let top = card.and_then(top_result);
    let top_action = card.and_then(|c| {
        let b = c.pointer("/buttons/0/buttonRenderer")?;
        parse::watch_target(b.get("command").or_else(|| b.get("navigationEndpoint")))
    });

    const ORDER: [ItemKind; 5] =
        [ItemKind::Song, ItemKind::Artist, ItemKind::Album, ItemKind::Video, ItemKind::Playlist];
    let mut buckets: Vec<(ItemKind, Vec<MusicItem>)> = ORDER.iter().map(|k| (*k, Vec::new())).collect();
    let mut seen = HashSet::new();
    if let Some(t) = &top {
        seen.insert((t.kind, t.id.clone()));
    }
    for section in sections {
        let (rows, shelf_title) = if let Some(m) = section.get("musicShelfRenderer") {
            (arr(m, "/contents"), text(m.get("title")))
        } else {
            (arr(section, "/itemSectionRenderer/contents"), String::new())
        };
        for row in rows {
            let Some(r) = row.get("musicResponsiveListItemRenderer") else { continue };
            let Some(group) = search_group(r).or_else(|| group_from_title(&shelf_title)) else { continue };
            let Some(mut it) = parse::responsive(r) else { continue };
            if group == ItemKind::Video && it.kind == ItemKind::Song {
                it.kind = ItemKind::Video;
            }
            if !seen.insert((it.kind, it.id.clone())) {
                continue;
            }
            if let Some((_, b)) = buckets.iter_mut().find(|(k, _)| *k == group) {
                b.push(it);
            }
        }
    }
    let shelves = buckets
        .into_iter()
        .filter(|(_, items)| !items.is_empty())
        .map(|(kind, items)| Shelf {
            id: format!("all-{kind:?}").to_lowercase(),
            title: match kind {
                ItemKind::Song => "Songs",
                ItemKind::Artist => "Artists",
                ItemKind::Album => "Albums and singles",
                ItemKind::Video => "Videos",
                _ => "Playlists",
            }
            .to_string(),
            display: if kind == ItemKind::Song { ShelfDisplay::List } else { ShelfDisplay::Card },
            items,
            more: None,
        })
        .collect();
    SearchResults { query: query.to_string(), top, top_action, shelves }
}

fn top_result(card: &Value) -> Option<MusicItem> {
    let title = parse::non_empty_text(card.get("title"))?;
    let subtitle = parse::non_empty_text(card.get("subtitle"));
    let thumbs = parse::thumbnails(card.pointer("/thumbnail/musicThumbnailRenderer/thumbnail"));
    let tap = card.get("onTap").or_else(|| card.pointer("/title/runs/0/navigationEndpoint"))?;
    let mut artists = Vec::new();
    for run in arr(card, "/subtitle/runs") {
        if let (Some(id), Some(name)) = (s(run, "/navigationEndpoint/browseEndpoint/browseId"), s(run, "/text")) {
            if id.starts_with("UC") {
                artists.push(ArtistRef { id: Some(id.to_string()), name: name.to_string() });
            }
        }
    }
    let (kind, id) = if let Some(v) = s(tap, "/watchEndpoint/videoId") {
        let first = subtitle.as_deref().unwrap_or("").split('•').next().unwrap_or("").trim().to_lowercase();
        (if first == "video" { ItemKind::Video } else { ItemKind::Song }, v)
    } else {
        let b = tap.get("browseEndpoint")?;
        let kind = parse::page_type_kind(
            s(b, "/browseEndpointContextSupportedConfigs/browseEndpointContextMusicConfig/pageType").unwrap_or(""),
        )?;
        (kind, s(b, "/browseId")?)
    };
    let mut it = parse::item(kind, id, title);
    it.subtitle = subtitle;
    it.thumbnails = thumbs;
    it.artists = artists;
    Some(it)
}

fn search_filtered(query: &str, sections: &[Value], filter: SearchFilter) -> SearchResults {
    let shelves = parse::shelf_nodes(sections)
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let mut shelf = parse::shelf(n, format!("{filter:?}-{i}").to_lowercase(), &[]);
            if filter == SearchFilter::Videos {
                for it in &mut shelf.items {
                    if it.kind == ItemKind::Song {
                        it.kind = ItemKind::Video;
                    }
                }
            }
            shelf
        })
        .filter(|s| !s.items.is_empty())
        .collect();
    SearchResults { query: query.to_string(), top: None, top_action: None, shelves }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_rows() {
        let j = json!({ "contents": { "singleColumnMusicWatchNextResultsRenderer": { "tabbedRenderer": { "watchNextTabbedResultsRenderer": { "tabs": [ { "tabRenderer": { "content": { "musicQueueRenderer": { "content": { "playlistPanelRenderer": {
            "contents": [ { "playlistPanelVideoRenderer": {
                "title": { "runs": [{ "text": "Song" }] },
                "navigationEndpoint": { "watchEndpoint": { "videoId": "4NRXx6U8ABQ" } },
                "lengthText": { "runs": [{ "text": "3:20" }] },
                "shortBylineText": { "runs": [{ "text": "Someone" }] }
            } } ],
            "continuations": [ { "nextRadioContinuationData": { "continuation": "MORE" } } ]
        } } } } } } ] } } } } });
        let q = parse_queue(&j);
        assert_eq!(q.tracks.len(), 1);
        assert_eq!(q.tracks[0].duration, Some(200));
        assert_eq!(q.tracks[0].artists[0].name, "Someone");
        assert_eq!(q.next.as_deref(), Some("MORE"));
    }

    #[test]
    fn search_groups() {
        let row = json!({ "flexColumns": [ {}, { "musicResponsiveListItemFlexColumnRenderer": { "text": { "runs": [{ "text": "Single" }, { "text": " • 2020" }] } } } ] });
        assert_eq!(search_group(&row), Some(ItemKind::Album));
        assert_eq!(group_from_title("Community playlists"), Some(ItemKind::Playlist));
    }

    #[test]
    fn music_client_builds() {
        let m = Music::new(std::env::temp_dir().join("velmora-music-test")).unwrap();
        assert!(!m.signed_in());
        m.set_cookies(vec![Cookie {
            name: "SAPISID".into(),
            value: "x".into(),
            domain: ".youtube.com".into(),
            path: "/".into(),
            expires: None,
            secure: true,
            http_only: false,
        }]);
        assert!(m.signed_in());
        m.clear_cookies();
        assert!(!m.signed_in());
    }
}
