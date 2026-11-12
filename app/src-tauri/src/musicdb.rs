// SPDX-License-Identifier: GPL-3.0-or-later
//! Music's local database (SQLite, in the app data folder).
//!
//! - `cache`: the last copy of Home, the library, albums, artists and
//!   playlists, so a page shows straight away and refreshes behind it.
//! - `searches`: recent Music searches for the search box.
//! - `plays`: what was played, when and for how long. Later phases build
//!   "Recently played" and the personal Home rows from it.
//! - `state`: the queue, so Velmora reopens on the same song.
//!
//! Everything here can be lost without harm: a broken file is deleted and
//! made again, and if SQLite can't open at all Music runs from memory.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use serde_json::Value;
use tauri::State;

/// Cached pages older than this are dropped at startup.
const CACHE_DAYS: i64 = 30;
/// Cache size cap; the oldest pages go first.
const CACHE_BYTES: i64 = 64 * 1024 * 1024;
const SEARCHES_KEPT: i64 = 50;
const PLAYS_KEPT: i64 = 20_000;
/// A queue longer than this is cut when saved (the rest is radio anyway).
const QUEUE_KEPT: usize = 500;
const SCHEMA: i32 = 1;

pub struct MusicDb {
    conn: Mutex<Connection>,
    path: Option<PathBuf>,
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    // journal_mode answers with a row, so it can't go in the batch below.
    conn.pragma_update_and_check(None, "journal_mode", "WAL", |_| Ok(()))?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS cache (
             key   TEXT PRIMARY KEY,
             value TEXT NOT NULL,
             at    INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS searches (
             query TEXT PRIMARY KEY COLLATE NOCASE,
             at    INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS plays (
             id       INTEGER PRIMARY KEY,
             video_id TEXT NOT NULL,
             at       INTEGER NOT NULL,
             seconds  INTEGER NOT NULL,
             item     TEXT NOT NULL
         );
         CREATE INDEX IF NOT EXISTS plays_video ON plays(video_id);
         CREATE INDEX IF NOT EXISTS plays_at ON plays(at);
         CREATE TABLE IF NOT EXISTS state (
             key   TEXT PRIMARY KEY,
             value TEXT NOT NULL
         );",
    )?;
    conn.pragma_update(None, "user_version", SCHEMA)
}

/// Drop old cache pages and trim the tables to their caps.
fn prune(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM cache WHERE at < ?1", params![now() - CACHE_DAYS * 86_400])?;
    let total: i64 = conn.query_row("SELECT COALESCE(SUM(LENGTH(value)), 0) FROM cache", [], |r| r.get(0))?;
    if total > CACHE_BYTES {
        // Delete oldest first until under the cap.
        let mut over = total - CACHE_BYTES;
        let mut stmt = conn.prepare("SELECT key, LENGTH(value) FROM cache ORDER BY at ASC")?;
        let rows: Vec<(String, i64)> = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?.filter_map(Result::ok).collect();
        for (key, len) in rows {
            if over <= 0 {
                break;
            }
            conn.execute("DELETE FROM cache WHERE key = ?1", params![key])?;
            over -= len;
        }
    }
    conn.execute(
        "DELETE FROM searches WHERE query NOT IN (SELECT query FROM searches ORDER BY at DESC LIMIT ?1)",
        params![SEARCHES_KEPT],
    )?;
    conn.execute("DELETE FROM plays WHERE id NOT IN (SELECT id FROM plays ORDER BY id DESC LIMIT ?1)", params![PLAYS_KEPT])?;
    Ok(())
}

fn open_file(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    migrate(&conn)?;
    let _ = prune(&conn);
    Ok(conn)
}

impl MusicDb {
    pub fn open(dir: &Path) -> Self {
        let _ = std::fs::create_dir_all(dir);
        let path = dir.join("music.db");
        let conn = open_file(&path).or_else(|e| {
            // A damaged file: start over. It only holds caches and history.
            tracing::warn!("music database unusable ({e}); starting a new one");
            for ext in ["", "-wal", "-shm"] {
                let _ = std::fs::remove_file(format!("{}{ext}", path.display()));
            }
            open_file(&path)
        });
        match conn {
            Ok(conn) => Self { conn: Mutex::new(conn), path: Some(path) },
            Err(e) => {
                tracing::warn!("music database can't open ({e}); keeping it in memory");
                let conn = Connection::open_in_memory().expect("in-memory SQLite");
                let _ = migrate(&conn);
                Self { conn: Mutex::new(conn), path: None }
            }
        }
    }

    fn with<T>(&self, f: impl FnOnce(&Connection) -> rusqlite::Result<T>) -> Result<T, String> {
        let conn = self.conn.lock().map_err(|_| "music database is busy".to_string())?;
        f(&conn).map_err(|e| e.to_string())
    }
}

// ── Page cache ──────────────────────────────────────────────────────

/// Keys are short names the pages choose ("home", "album:MPREb…").
fn valid_key(key: &str) -> bool {
    !key.is_empty() && key.len() <= 200
}

#[tauri::command]
pub async fn music_cache_get(db: State<'_, MusicDb>, key: String) -> Result<Option<Value>, String> {
    if !valid_key(&key) {
        return Ok(None);
    }
    let text: Option<String> =
        db.with(|c| c.query_row("SELECT value FROM cache WHERE key = ?1", params![key], |r| r.get(0)).optional())?;
    Ok(text.and_then(|t| serde_json::from_str(&t).ok()))
}

#[tauri::command]
pub async fn music_cache_put(db: State<'_, MusicDb>, key: String, value: Value) -> Result<(), String> {
    if !valid_key(&key) || value.is_null() {
        return Ok(());
    }
    let text = serde_json::to_string(&value).map_err(|e| e.to_string())?;
    db.with(|c| {
        c.execute(
            "INSERT INTO cache (key, value, at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, at = excluded.at",
            params![key, text, now()],
        )
        .map(|_| ())
    })
}

// ── Search history ──────────────────────────────────────────────────

#[tauri::command]
pub async fn music_searches(db: State<'_, MusicDb>) -> Result<Vec<String>, String> {
    db.with(|c| {
        let mut stmt = c.prepare("SELECT query FROM searches ORDER BY at DESC LIMIT 12")?;
        let rows: Vec<String> = stmt.query_map([], |r| r.get(0))?.filter_map(Result::ok).collect();
        Ok(rows)
    })
}

#[tauri::command]
pub async fn music_search_add(db: State<'_, MusicDb>, query: String) -> Result<(), String> {
    let q = query.trim();
    if q.chars().count() < 2 || q.len() > 200 {
        return Ok(());
    }
    db.with(|c| {
        c.execute(
            "INSERT INTO searches (query, at) VALUES (?1, ?2)
             ON CONFLICT(query) DO UPDATE SET query = excluded.query, at = excluded.at",
            params![q, now()],
        )?;
        c.execute(
            "DELETE FROM searches WHERE query NOT IN (SELECT query FROM searches ORDER BY at DESC LIMIT ?1)",
            params![SEARCHES_KEPT],
        )
        .map(|_| ())
    })
}

/// Remove one search, or all of them with `None`.
#[tauri::command]
pub async fn music_search_remove(db: State<'_, MusicDb>, query: Option<String>) -> Result<(), String> {
    db.with(|c| match &query {
        Some(q) => c.execute("DELETE FROM searches WHERE query = ?1", params![q]).map(|_| ()),
        None => c.execute("DELETE FROM searches", []).map(|_| ()),
    })
}

// ── Play history ────────────────────────────────────────────────────

/// Log a song that was listened to. `item` is the MusicItem as the page has it.
#[tauri::command]
pub async fn music_play_log(db: State<'_, MusicDb>, item: Value, seconds: u32) -> Result<(), String> {
    let Some(id) = item.get("id").and_then(Value::as_str).map(str::to_string) else { return Ok(()) };
    if seconds == 0 || id.is_empty() {
        return Ok(());
    }
    let text = serde_json::to_string(&item).map_err(|e| e.to_string())?;
    db.with(|c| {
        c.execute(
            "INSERT INTO plays (video_id, at, seconds, item) VALUES (?1, ?2, ?3, ?4)",
            params![id, now(), seconds, text],
        )
        .map(|_| ())
    })
}

// ── Queue ───────────────────────────────────────────────────────────

#[tauri::command]
pub async fn music_queue_save(db: State<'_, MusicDb>, mut value: Value) -> Result<(), String> {
    // Keep the saved queue bounded around the current song.
    if let Some(obj) = value.as_object_mut() {
        let index = obj.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
        if let Some(Value::Array(q)) = obj.get_mut("queue") {
            if q.len() > QUEUE_KEPT {
                let start = index.saturating_sub(QUEUE_KEPT / 5).min(q.len() - QUEUE_KEPT);
                let kept: Vec<Value> = q.drain(start..start + QUEUE_KEPT).collect();
                *q = kept;
                obj.insert("index".into(), Value::from(index - start));
            }
        }
    }
    let text = serde_json::to_string(&value).map_err(|e| e.to_string())?;
    db.with(|c| {
        c.execute(
            "INSERT INTO state (key, value) VALUES ('queue', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![text],
        )
        .map(|_| ())
    })
}

#[tauri::command]
pub async fn music_queue_load(db: State<'_, MusicDb>) -> Result<Option<Value>, String> {
    let text: Option<String> =
        db.with(|c| c.query_row("SELECT value FROM state WHERE key = 'queue'", [], |r| r.get(0)).optional())?;
    Ok(text.and_then(|t| serde_json::from_str(&t).ok()))
}

// ── Storage (Settings) ──────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Storage {
    /// Size of the database file(s) on disk.
    file_bytes: u64,
    cache_pages: i64,
    cache_bytes: i64,
    searches: i64,
    plays: i64,
}

#[tauri::command]
pub async fn music_storage(db: State<'_, MusicDb>) -> Result<Storage, String> {
    let (cache_pages, cache_bytes, searches, plays) = db.with(|c| {
        let (pages, bytes): (i64, i64) =
            c.query_row("SELECT COUNT(*), COALESCE(SUM(LENGTH(value)), 0) FROM cache", [], |r| Ok((r.get(0)?, r.get(1)?)))?;
        let searches: i64 = c.query_row("SELECT COUNT(*) FROM searches", [], |r| r.get(0))?;
        let plays: i64 = c.query_row("SELECT COUNT(*) FROM plays", [], |r| r.get(0))?;
        Ok((pages, bytes, searches, plays))
    })?;
    let file_bytes = db
        .path
        .as_ref()
        .map(|p| {
            ["", "-wal"]
                .iter()
                .filter_map(|ext| std::fs::metadata(format!("{}{ext}", p.display())).ok())
                .map(|m| m.len())
                .sum()
        })
        .unwrap_or(0);
    Ok(Storage { file_bytes, cache_pages, cache_bytes, searches, plays })
}

/// Clear "cache" (saved pages) or "history" (searches and plays).
#[tauri::command]
pub async fn music_storage_clear(db: State<'_, MusicDb>, what: String) -> Result<(), String> {
    db.with(|c| {
        match what.as_str() {
            "cache" => {
                c.execute("DELETE FROM cache", [])?;
            }
            "history" => {
                c.execute("DELETE FROM searches", [])?;
                c.execute("DELETE FROM plays", [])?;
            }
            _ => return Ok(()),
        }
        // Give the space back to the disk. The checkpoint answers with a row.
        let _ = c.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()));
        c.execute_batch("VACUUM;")
    })
}
