// SPDX-License-Identifier: GPL-3.0-only
// Ported in part from YTubic (https://github.com/NUber-dev/YTubic), Copyright George Shyshov.
//! Turning InnerTube's renderer trees into [`MusicItem`]s and [`Shelf`]s.
//!
//! Ported from YTubic (GPL-3.0-only, github.com/NUber-dev/YTubic),
//! `src/lib/innertube/shared.ts` and the page modules next to it. The shapes
//! YouTube Music returns vary by page and change over time, so most readers
//! try several paths and some walk the whole tree.

use serde_json::Value;

use super::{ArtistRef, ItemKind, MusicItem, Shelf, ShelfDisplay, ShelfMore, Thumbnail, WatchTarget};

const PAGE_TYPE: &str = "/browseEndpointContextSupportedConfigs/browseEndpointContextMusicConfig/pageType";

/// A non-empty string at a JSON pointer.
pub fn s<'a>(v: &'a Value, ptr: &str) -> Option<&'a str> {
    v.pointer(ptr).and_then(Value::as_str).filter(|s| !s.is_empty())
}

/// The array at a JSON pointer, or an empty slice.
pub fn arr<'a>(v: &'a Value, ptr: &str) -> &'a [Value] {
    v.pointer(ptr).and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[])
}

/// `{ runs: [{text}] }`, `{ simpleText }` or a plain string, as text.
pub fn text(v: Option<&Value>) -> String {
    match v {
        Some(Value::String(s)) => s.clone(),
        Some(v) => {
            if let Some(t) = v.get("simpleText").and_then(Value::as_str) {
                return t.to_string();
            }
            v.get("runs")
                .and_then(Value::as_array)
                .map(|runs| runs.iter().filter_map(|r| r.get("text").and_then(Value::as_str)).collect::<String>())
                .unwrap_or_default()
        }
        None => String::new(),
    }
}

pub fn text_at(v: &Value, ptr: &str) -> String {
    text(v.pointer(ptr))
}

/// Depth-first walk; returns the first node `pred` accepts.
pub fn find<'a>(root: &'a Value, mut pred: impl FnMut(&'a Value) -> bool) -> Option<&'a Value> {
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if pred(n) {
            return Some(n);
        }
        match n {
            Value::Array(a) => stack.extend(a.iter().rev()),
            Value::Object(o) => {
                let vals: Vec<&Value> = o.values().collect();
                stack.extend(vals.into_iter().rev());
            }
            _ => {}
        }
    }
    None
}

/// The value under the first `key` found anywhere in the tree.
pub fn find_key<'a>(root: &'a Value, key: &str) -> Option<&'a Value> {
    find(root, |n| n.get(key).is_some_and(Value::is_object)).and_then(|n| n.get(key))
}

/// Every value stored under `key`, in document order.
pub fn collect_key<'a>(root: &'a Value, key: &str) -> Vec<&'a Value> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        match n {
            Value::Array(a) => stack.extend(a.iter().rev()),
            Value::Object(o) => {
                if let Some(v) = o.get(key) {
                    out.push(v);
                }
                let vals: Vec<&Value> = o.iter().filter(|(k, _)| k.as_str() != key).map(|(_, v)| v).collect();
                stack.extend(vals.into_iter().rev());
            }
            _ => {}
        }
    }
    out
}

/// The first continuation token anywhere in the tree.
pub fn find_continuation(root: &Value) -> Option<String> {
    let mut out = None;
    find(root, |n| {
        out = s(n, "/nextContinuationData/continuation")
            .or_else(|| s(n, "/continuationCommand/token"))
            .map(str::to_string);
        out.is_some()
    });
    out
}

pub fn thumbnails(v: Option<&Value>) -> Vec<Thumbnail> {
    let Some(v) = v else { return Vec::new() };
    let list = v
        .get("thumbnails")
        .or_else(|| v.pointer("/thumbnail/thumbnails"))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    list.iter()
        .filter_map(|t| {
            Some(Thumbnail {
                url: s(t, "/url")?.to_string(),
                width: t.get("width").and_then(Value::as_u64).unwrap_or(0) as u32,
                height: t.get("height").and_then(Value::as_u64).unwrap_or(0) as u32,
            })
        })
        .collect()
}

/// Any `thumbnails` array under a node, for renderer variants not listed.
pub fn deep_thumbnails(v: Option<&Value>) -> Vec<Thumbnail> {
    let Some(v) = v else { return Vec::new() };
    find(v, |n| n.get("thumbnails").and_then(Value::as_array).is_some_and(|a| !a.is_empty()))
        .map(|n| thumbnails(Some(n)))
        .unwrap_or_default()
}

/// Thumbnails YouTube generates for every video, for rows that come without one.
pub fn video_thumbnails(video_id: &str) -> Vec<Thumbnail> {
    vec![
        Thumbnail { url: format!("https://i.ytimg.com/vi/{video_id}/mqdefault.jpg"), width: 320, height: 180 },
        Thumbnail { url: format!("https://i.ytimg.com/vi/{video_id}/hqdefault.jpg"), width: 480, height: 360 },
    ]
}

/// "3:42" or "1:05:03" as seconds.
pub fn duration(t: &str) -> Option<u32> {
    let parts: Vec<u32> = t.trim().split(':').map(|p| p.parse().ok()).collect::<Option<_>>()?;
    match parts.as_slice() {
        [m, s] => Some(m * 60 + s),
        [h, m, s] => Some(h * 3600 + m * 60 + s),
        _ => None,
    }
}

pub fn page_type_kind(page_type: &str) -> Option<ItemKind> {
    if page_type.contains("ARTIST") {
        Some(ItemKind::Artist)
    } else if page_type.contains("ALBUM") {
        Some(ItemKind::Album)
    } else if page_type.contains("PLAYLIST") || page_type.contains("PODCAST_SHOW") {
        Some(ItemKind::Playlist)
    } else {
        None
    }
}

fn explicit(raw: &Value) -> bool {
    ["badges", "subtitleBadges"].iter().any(|k| {
        arr(raw, &format!("/{k}")).iter().any(|b| {
            let Some(r) = b.get("musicInlineBadgeRenderer") else { return false };
            s(r, "/icon/iconType").is_some_and(|t| t.contains("EXPLICIT"))
                || s(r, "/accessibilityData/accessibilityData/label")
                    .or_else(|| s(r, "/accessibilityText"))
                    .is_some_and(|l| l.to_ascii_lowercase().contains("explicit"))
        })
    })
}

/// A run's link to an artist or album page.
enum RunLink {
    Artist(ArtistRef),
    Album { id: String, name: String },
    None,
}

fn run_link(run: &Value) -> RunLink {
    let Some(id) = s(run, "/navigationEndpoint/browseEndpoint/browseId") else { return RunLink::None };
    let page = s(run, &format!("/navigationEndpoint/browseEndpoint{PAGE_TYPE}")).unwrap_or("");
    let name = s(run, "/text").unwrap_or("").to_string();
    if page.contains("ARTIST") {
        RunLink::Artist(ArtistRef { id: Some(id.to_string()), name })
    } else if page.contains("ALBUM") {
        RunLink::Album { id: id.to_string(), name }
    } else {
        RunLink::None
    }
}

pub fn item(kind: ItemKind, id: impl Into<String>, title: String) -> MusicItem {
    MusicItem {
        kind,
        id: id.into(),
        title,
        subtitle: None,
        thumbnails: Vec::new(),
        artists: Vec::new(),
        album: None,
        album_id: None,
        duration: None,
        explicit: false,
        play_count: None,
        set_video_id: None,
        play_video_id: None,
        params: None,
        tint: None,
    }
}

fn non_empty(s: String) -> Option<String> {
    (!s.trim().is_empty()).then_some(s)
}

/// Text that isn't blank.
pub fn non_empty_text(v: Option<&Value>) -> Option<String> {
    non_empty(text(v))
}

/// `playlistPanelVideoRenderer`: a row in a radio or up-next queue.
pub fn panel_video(raw: &Value) -> Option<MusicItem> {
    let video_id = s(raw, "/navigationEndpoint/watchEndpoint/videoId")?;
    let mut it = item(ItemKind::Song, video_id, text(raw.get("title")));
    let runs = raw.pointer("/longBylineText/runs").or_else(|| raw.pointer("/shortBylineText/runs"));
    for run in runs.and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]) {
        match run_link(run) {
            RunLink::Artist(a) => it.artists.push(a),
            RunLink::Album { id, name } => {
                it.album = Some(name);
                it.album_id = Some(id);
            }
            RunLink::None => {}
        }
    }
    if it.artists.is_empty() {
        // Some rows name the artist without a link: "Artist • Album • 2021".
        let byline = text_at(raw, "/shortBylineText");
        if let Some(first) = byline.split('•').next().map(str::trim).filter(|s| !s.is_empty()) {
            it.artists.push(ArtistRef { id: None, name: first.to_string() });
        }
    }
    it.subtitle = non_empty(it.artists.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", "));
    it.duration = duration(&text_at(raw, "/lengthText"));
    it.thumbnails = thumbnails(raw.get("thumbnail"));
    if it.thumbnails.is_empty() {
        it.thumbnails = video_thumbnails(video_id);
    }
    it.explicit = explicit(raw);
    it.set_video_id = s(raw, "/playlistSetVideoId").map(str::to_string);
    Some(it)
}

/// `musicNavigationButtonRenderer`: a colored mood or genre tile.
pub fn nav_button(raw: &Value) -> Option<MusicItem> {
    let title = non_empty(text(raw.get("buttonText")))?;
    let id = s(raw, "/clickCommand/browseEndpoint/browseId")?;
    let mut it = item(ItemKind::Category, id, title);
    it.params = s(raw, "/clickCommand/browseEndpoint/params").map(str::to_string);
    if let Some(c) = raw.pointer("/solid/leftStripeColor").and_then(Value::as_i64) {
        it.tint = Some(format!("#{:06x}", (c as u32) & 0x00ff_ffff));
    }
    Some(it)
}

/// `musicTwoRowItemRenderer`: the card in carousels and grids.
pub fn two_row(raw: &Value) -> Option<MusicItem> {
    let title = text(raw.get("title"));
    let mut thumbs = thumbnails(
        raw.pointer("/thumbnailRenderer/musicThumbnailRenderer/thumbnail")
            .or_else(|| raw.pointer("/thumbnail/musicThumbnailRenderer/thumbnail"))
            .or_else(|| raw.get("thumbnail")),
    );
    if thumbs.is_empty() {
        thumbs = deep_thumbnails(raw.get("thumbnailRenderer"));
    }
    if thumbs.is_empty() {
        thumbs = deep_thumbnails(raw.get("thumbnail"));
    }

    let mut it = if let Some(video_id) = s(raw, "/navigationEndpoint/watchEndpoint/videoId") {
        // A 16:9 cover means a music video; songs come square.
        let widest = thumbs.iter().max_by_key(|t| t.width);
        let ratio = widest.filter(|t| t.height > 0).map(|t| t.width as f32 / t.height as f32).unwrap_or(1.0);
        item(if ratio > 1.4 { ItemKind::Video } else { ItemKind::Song }, video_id, title)
    } else {
        let id = s(raw, "/navigationEndpoint/browseEndpoint/browseId")?;
        let page = s(raw, &format!("/navigationEndpoint/browseEndpoint{PAGE_TYPE}")).unwrap_or("");
        let mut it = item(page_type_kind(page).unwrap_or(ItemKind::Playlist), id, title);
        if it.kind == ItemKind::Playlist {
            it.play_video_id = s(
                raw,
                "/overlay/musicItemThumbnailOverlayRenderer/content/musicPlayButtonRenderer/playNavigationEndpoint/watchEndpoint/videoId",
            )
            .map(str::to_string);
        }
        it
    };

    for run in arr(raw, "/subtitle/runs") {
        if let RunLink::Artist(a) = run_link(run) {
            it.artists.push(a);
        }
    }
    it.subtitle = non_empty(text(raw.get("subtitle")));
    it.thumbnails = thumbs;
    it.explicit = explicit(raw);
    Some(it)
}

fn looks_like_count(t: &str) -> bool {
    let lower = t.to_lowercase();
    if lower.contains("plays") || lower.contains("views") {
        return true;
    }
    // A bare "1,234,567" or "1.2M", as artist top songs show.
    let mut chars = t.chars();
    chars.next().is_some_and(|c| c.is_ascii_digit()) && t.chars().all(|c| c.is_ascii_digit() || " .,KMBkmb".contains(c))
}

/// `musicResponsiveListItemRenderer`: a row in a song list, or a search row
/// that links to an artist, album or playlist.
pub fn responsive(raw: &Value) -> Option<MusicItem> {
    let flex = arr(raw, "/flexColumns");
    let col = |i: usize| flex.get(i).and_then(|c| c.pointer("/musicResponsiveListItemFlexColumnRenderer/text"));
    let title_run = col(0).and_then(|c| c.pointer("/runs/0"));
    let title = text(col(0));

    let mut artists = Vec::new();
    let mut album = None;
    let mut album_id = None;
    let mut play_count = None;
    for i in 1..flex.len() {
        let Some(node) = col(i) else { continue };
        let mut linked = false;
        for run in node.get("runs").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]) {
            match run_link(run) {
                RunLink::Artist(a) => {
                    artists.push(a);
                    linked = true;
                }
                RunLink::Album { id, name } => {
                    album = Some(name);
                    album_id = Some(id);
                    linked = true;
                }
                RunLink::None => {}
            }
        }
        if linked {
            continue;
        }
        let whole = text(Some(node)).trim().to_string();
        if !whole.is_empty() && duration(&whole).is_none() && looks_like_count(&whole) && play_count.is_none() {
            play_count = Some(whole);
        }
    }

    let fixed = raw.pointer("/fixedColumns/0/musicResponsiveListItemFixedColumnRenderer/text");
    let mut dur = duration(&text(fixed));
    if dur.is_none() {
        dur = col(1)
            .and_then(|c| c.get("runs"))
            .and_then(Value::as_array)
            .and_then(|r| r.last())
            .and_then(|r| r.get("text").and_then(Value::as_str))
            .and_then(duration);
    }

    let video_id = title_run
        .and_then(|r| s(r, "/navigationEndpoint/watchEndpoint/videoId"))
        .or_else(|| {
            s(
                raw,
                "/overlay/musicItemThumbnailOverlayRenderer/content/musicPlayButtonRenderer/playNavigationEndpoint/watchEndpoint/videoId",
            )
        })
        .or_else(|| s(raw, "/playlistItemData/videoId"))
        .or_else(|| s(raw, "/navigationEndpoint/watchEndpoint/videoId"));

    let mut thumbs = thumbnails(raw.pointer("/thumbnail/musicThumbnailRenderer/thumbnail"));
    let subtitle = {
        let names = artists.iter().map(|a: &ArtistRef| a.name.as_str()).collect::<Vec<_>>().join(", ");
        non_empty(names).or_else(|| non_empty(text(col(1))))
    };

    if let Some(video_id) = video_id {
        if thumbs.is_empty() {
            thumbs = video_thumbnails(video_id);
        }
        let mut it = item(ItemKind::Song, video_id, title);
        it.subtitle = subtitle;
        it.thumbnails = thumbs;
        it.artists = artists;
        it.album = album;
        it.album_id = album_id;
        it.duration = dur;
        it.explicit = explicit(raw);
        it.play_count = play_count;
        it.set_video_id = s(raw, "/playlistItemData/playlistSetVideoId").map(str::to_string);
        return Some(it);
    }

    let nav = title_run
        .and_then(|r| r.pointer("/navigationEndpoint/browseEndpoint"))
        .or_else(|| raw.pointer("/navigationEndpoint/browseEndpoint"))?;
    let id = s(nav, "/browseId")?;
    let kind = page_type_kind(s(nav, PAGE_TYPE).unwrap_or(""))?;
    let mut it = item(kind, id, title);
    it.subtitle = subtitle;
    it.thumbnails = thumbs;
    it.explicit = explicit(raw);
    Some(it)
}

/// Any item renderer.
pub fn any_item(node: &Value) -> Option<MusicItem> {
    if let Some(r) = node.get("musicTwoRowItemRenderer") {
        two_row(r)
    } else if let Some(r) = node.get("musicResponsiveListItemRenderer") {
        responsive(r)
    } else if let Some(r) = node.get("musicNavigationButtonRenderer") {
        nav_button(r)
    } else if let Some(r) = node.get("playlistPanelVideoRenderer") {
        panel_video(r)
    } else {
        None
    }
}

/// Every `musicResponsiveListItemRenderer` in the tree.
pub fn responsive_rows(root: &Value) -> Vec<&Value> {
    collect_key(root, "musicResponsiveListItemRenderer")
}

/// Song rows anywhere in the tree, without repeats.
pub fn song_rows(root: &Value, seen: &mut std::collections::HashSet<String>) -> Vec<MusicItem> {
    responsive_rows(root)
        .into_iter()
        .filter_map(responsive)
        .filter(|it| it.kind == ItemKind::Song && seen.insert(it.id.clone()))
        .collect()
}

/// A shelf node inside a section list.
pub enum ShelfNode<'a> {
    Carousel(&'a Value),
    List(&'a Value),
    Card(&'a Value),
    Grid(&'a Value),
}

impl<'a> ShelfNode<'a> {
    /// The node that owns this shelf's paging token.
    pub fn container(&self) -> &'a Value {
        match self {
            Self::Carousel(v) | Self::List(v) | Self::Card(v) | Self::Grid(v) => v,
        }
    }
}

/// Shelves in a section list, through the item-section and nested-list wrappers.
pub fn shelf_nodes(sections: &[Value]) -> Vec<ShelfNode<'_>> {
    fn walk<'a>(node: &'a Value, out: &mut Vec<ShelfNode<'a>>) {
        if let Some(v) = node.get("musicCarouselShelfRenderer") {
            out.push(ShelfNode::Carousel(v));
        } else if let Some(v) = node.get("musicShelfRenderer") {
            out.push(ShelfNode::List(v));
        } else if let Some(v) = node.get("musicCardShelfRenderer") {
            out.push(ShelfNode::Card(v));
        } else {
            for c in arr(node, "/itemSectionRenderer/contents") {
                walk(c, out);
            }
            for c in arr(node, "/sectionListRenderer/contents") {
                walk(c, out);
            }
            if let Some(g) = node.get("gridRenderer") {
                out.push(ShelfNode::Grid(g));
            }
        }
    }
    let mut out = Vec::new();
    for s in sections {
        walk(s, &mut out);
    }
    out
}

fn shelf_more(music: &Value) -> Option<ShelfMore> {
    let nav = music
        .pointer("/header/musicCarouselShelfBasicHeaderRenderer/moreContentButton/buttonRenderer/navigationEndpoint")
        .or_else(|| music.get("bottomEndpoint"))
        .or_else(|| music.pointer("/header/musicCarouselShelfBasicHeaderRenderer/title/runs/0/navigationEndpoint"))
        .or_else(|| music.pointer("/title/runs/0/navigationEndpoint"))?;
    let b = nav.get("browseEndpoint")?;
    Some(ShelfMore {
        browse_id: s(b, "/browseId")?.to_string(),
        params: s(b, "/params").map(str::to_string),
        page_type: s(b, PAGE_TYPE).map(str::to_string),
    })
}

/// The featured song on a card shelf ("Quick picks", search top result).
fn card_featured(card: &Value) -> Option<MusicItem> {
    let video_id = s(card, "/onTap/watchEndpoint/videoId")?;
    let title = non_empty(text(card.get("title")))?;
    let mut it = item(ItemKind::Song, video_id, title);
    it.subtitle = non_empty(text(card.get("subtitle")));
    it.thumbnails = thumbnails(card.pointer("/thumbnail/musicThumbnailRenderer/thumbnail"));
    it.explicit = explicit(card);
    Some(it)
}

/// Map one shelf. `extra` holds rows from later pages of the same shelf.
pub fn shelf(node: &ShelfNode<'_>, id: String, extra: &[Value]) -> Shelf {
    let (title, contents, more) = match node {
        ShelfNode::Carousel(m) => {
            (text(m.pointer("/header/musicCarouselShelfBasicHeaderRenderer/title")), arr(m, "/contents"), shelf_more(m))
        }
        ShelfNode::List(m) => (
            non_empty(text(m.get("title")))
                .unwrap_or_else(|| text(m.pointer("/header/musicCarouselShelfBasicHeaderRenderer/title"))),
            arr(m, "/contents"),
            shelf_more(m),
        ),
        ShelfNode::Card(c) => {
            (text(c.pointer("/header/musicCardShelfHeaderBasicRenderer/title")), arr(c, "/contents"), None)
        }
        ShelfNode::Grid(g) => (text(g.pointer("/header/gridHeaderRenderer/title")), arr(g, "/items"), None),
    };

    let (mut two, mut rows, mut tiles) = (false, false, false);
    let mut items = Vec::new();
    for c in contents.iter().chain(extra) {
        if c.get("musicTwoRowItemRenderer").is_some() {
            two = true;
        } else if c.get("musicResponsiveListItemRenderer").is_some() {
            rows = true;
        } else if c.get("musicNavigationButtonRenderer").is_some() {
            tiles = true;
        }
        if let Some(it) = any_item(c) {
            items.push(it);
        }
    }
    if let ShelfNode::Card(card) = node {
        if let Some(f) = card_featured(card) {
            items.insert(0, f);
        }
    }
    // Drop repeats that later pages sometimes bring.
    let mut seen = std::collections::HashSet::new();
    items.retain(|it| seen.insert((it.kind, it.id.clone())));

    let display = if tiles && !two && !rows {
        ShelfDisplay::Grid
    } else if rows && !two {
        ShelfDisplay::List
    } else {
        ShelfDisplay::Card
    };
    Shelf { id, title, items, display, more }
}

/// The paging token owned by one container: the legacy `continuations` list
/// or a trailing `continuationItemRenderer`.
pub fn paging_token(container: &Value) -> Option<String> {
    for c in arr(container, "/continuations") {
        if let Some(t) = s(c, "/nextContinuationData/continuation")
            .or_else(|| s(c, "/continuationEndpoint/continuationCommand/token"))
            .or_else(|| s(c, "/continuationCommand/token"))
        {
            return Some(t.to_string());
        }
    }
    let items = container.get("items").or_else(|| container.get("contents")).and_then(Value::as_array)?;
    s(items.last()?, "/continuationItemRenderer/continuationEndpoint/continuationCommand/token").map(str::to_string)
}

/// Rows of a section that is itself a grid or shelf, unwrapped.
fn flatten_rows(rows: &[Value]) -> Vec<Value> {
    let mut out = Vec::new();
    for row in rows {
        let inner = row
            .get("gridRenderer")
            .or_else(|| row.get("musicShelfRenderer"))
            .or_else(|| row.get("musicPlaylistShelfRenderer"));
        match inner.and_then(|i| i.get("items").or_else(|| i.get("contents"))).and_then(Value::as_array) {
            Some(nested) if !nested.is_empty() => out.extend(nested.iter().cloned()),
            _ => out.push(row.clone()),
        }
    }
    out
}

/// A `browse { continuation }` answer: its rows and the token after it.
pub fn continuation_page(json: &Value) -> (Vec<Value>, Option<String>) {
    let cc = json.get("continuationContents");
    let container = cc.and_then(|cc| {
        [
            "gridContinuation",
            "musicShelfContinuation",
            "musicPlaylistShelfContinuation",
            "sectionListContinuation",
            "playlistVideoListContinuation",
        ]
        .iter()
        .find_map(|k| cc.get(*k))
    });
    if let Some(c) = container {
        let items =
            c.get("items").or_else(|| c.get("contents")).and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
        return (flatten_rows(items), paging_token(c));
    }
    for a in arr(json, "/onResponseReceivedActions") {
        let appended = arr(a, "/appendContinuationItemsAction/continuationItems");
        if appended.is_empty() {
            continue;
        }
        let token = appended
            .last()
            .and_then(|l| s(l, "/continuationItemRenderer/continuationEndpoint/continuationCommand/token"))
            .map(str::to_string);
        return (flatten_rows(appended), token);
    }
    (Vec::new(), None)
}

/// A play or shuffle button's target.
pub fn watch_target(endpoint: Option<&Value>) -> Option<WatchTarget> {
    let ep = endpoint?;
    let video_id = s(ep, "/watchEndpoint/videoId").map(str::to_string);
    let playlist_id =
        s(ep, "/watchEndpoint/playlistId").or_else(|| s(ep, "/watchPlaylistEndpoint/playlistId")).map(str::to_string);
    let params = s(ep, "/watchEndpoint/params").or_else(|| s(ep, "/watchPlaylistEndpoint/params")).map(str::to_string);
    (video_id.is_some() || playlist_id.is_some()).then_some(WatchTarget { video_id, playlist_id, params })
}

/// The first number in "23 songs • 1 hour" or "1,024 tracks".
pub fn first_number(t: &str) -> Option<u32> {
    let digits: String = t
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit() || *c == ',' || *c == '.')
        .collect();
    digits.replace([',', '.'], "").parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn durations() {
        assert_eq!(duration("3:42"), Some(222));
        assert_eq!(duration("1:05:03"), Some(3903));
        assert_eq!(duration("Song"), None);
    }

    #[test]
    fn counts() {
        assert!(looks_like_count("1.2M plays"));
        assert!(looks_like_count("1,234,567"));
        assert!(!looks_like_count("21 Savage"));
        assert_eq!(first_number("1,024 songs • 3 hours"), Some(1024));
    }

    #[test]
    fn maps_song_row() {
        let row = json!({
            "flexColumns": [
                { "musicResponsiveListItemFlexColumnRenderer": { "text": { "runs": [
                    { "text": "Blinding Lights", "navigationEndpoint": { "watchEndpoint": { "videoId": "4NRXx6U8ABQ" } } }
                ] } } },
                { "musicResponsiveListItemFlexColumnRenderer": { "text": { "runs": [
                    { "text": "The Weeknd", "navigationEndpoint": { "browseEndpoint": { "browseId": "UC1", "browseEndpointContextSupportedConfigs": { "browseEndpointContextMusicConfig": { "pageType": "MUSIC_PAGE_TYPE_ARTIST" } } } } },
                    { "text": " • " },
                    { "text": "After Hours", "navigationEndpoint": { "browseEndpoint": { "browseId": "MPRE1", "browseEndpointContextSupportedConfigs": { "browseEndpointContextMusicConfig": { "pageType": "MUSIC_PAGE_TYPE_ALBUM" } } } } }
                ] } } }
            ],
            "fixedColumns": [ { "musicResponsiveListItemFixedColumnRenderer": { "text": { "simpleText": "3:20" } } } ]
        });
        let it = responsive(&row).unwrap();
        assert_eq!(it.kind, ItemKind::Song);
        assert_eq!(it.id, "4NRXx6U8ABQ");
        assert_eq!(it.artists[0].name, "The Weeknd");
        assert_eq!(it.album_id.as_deref(), Some("MPRE1"));
        assert_eq!(it.duration, Some(200));
        assert!(!it.thumbnails.is_empty());
    }

    #[test]
    fn continuation_pages() {
        let page = json!({ "onResponseReceivedActions": [ { "appendContinuationItemsAction": { "continuationItems": [
            { "musicTwoRowItemRenderer": {} },
            { "continuationItemRenderer": { "continuationEndpoint": { "continuationCommand": { "token": "NEXT" } } } }
        ] } } ] });
        let (items, token) = continuation_page(&page);
        assert_eq!(items.len(), 2);
        assert_eq!(token.as_deref(), Some("NEXT"));
    }

    #[test]
    fn tint_from_argb() {
        let b = json!({ "buttonText": { "runs": [{ "text": "Chill" }] }, "clickCommand": { "browseEndpoint": { "browseId": "FEmusic_moods_and_genres_category", "params": "p" } }, "solid": { "leftStripeColor": -16777216_i64 + 0x123456 } });
        let it = nav_button(&b).unwrap();
        assert_eq!(it.tint.as_deref(), Some("#123456"));
        assert_eq!(it.params.as_deref(), Some("p"));
    }
}
