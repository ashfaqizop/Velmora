//! UniverseAPI for Velmora on Android.
//!
//! Kotlin talks to the engine through UniFFI. Everything crosses the bridge
//! as JSON in the same shapes the desktop app's frontend gets from its Tauri
//! commands, so the engine's model types stay free of FFI derives and the
//! Kotlin side decodes with kotlinx.serialization.
//!
//! The engine runs on its own Tokio runtime. Async methods hand their work to
//! that runtime and UniFFI waits on the task from Kotlin coroutines, so a
//! cancelled coroutine stops waiting while the engine finishes (and caches)
//! what it started, the same as a closed page on desktop.
//!
//! Playback goes through a loopback HTTP server (see `stream`), so ExoPlayer
//! only ever sees `http://127.0.0.1:{port}/...` URLs.

mod logcat;
mod stream;

use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use futures::StreamExt;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use universeapi::model::{AudioMode, ProviderRef, Source, Target, TitleSummary};
use universeapi::torrent::TorrentJob;
use universeapi::vault::Vault;
use universeapi::{Engine, EngineOptions, Preferences, Registry};

uniffi::setup_scaffolding!();

// Built in by CI from the repo secrets, the same as the desktop release.
const TMDB_READ_TOKEN: Option<&str> = option_env!("VELMORA_TMDB_READ_TOKEN");
const TMDB_API_KEY: Option<&str> = option_env!("VELMORA_TMDB_API_KEY");
const SPORTSDB_KEY: Option<&str> = option_env!("VELMORA_SPORTSDB_KEY");
const FOOTBALL_DATA_TOKEN: Option<&str> = option_env!("VELMORA_FOOTBALL_DATA_TOKEN");

const CONFIG_TIMEOUT: Duration = Duration::from_secs(8);
const WORKER_THREADS: usize = 4;

// ── Errors ─────────────────────────────────────────────────────────────

/// What Kotlin sees as `EngineException`.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum EngineError {
    /// The engine tried and failed (network, provider, torrent, file).
    #[error("{reason}")]
    Failed { reason: String },
    /// The JSON handed over didn't match the engine's types.
    #[error("bad input: {reason}")]
    BadInput { reason: String },
}

fn failed(e: impl std::fmt::Display) -> EngineError {
    EngineError::Failed { reason: e.to_string() }
}

fn to_json<T: Serialize>(value: &T) -> Result<String, EngineError> {
    serde_json::to_string(value).map_err(failed)
}

fn from_json<T: DeserializeOwned>(what: &str, json: &str) -> Result<T, EngineError> {
    serde_json::from_str(json).map_err(|e| EngineError::BadInput { reason: format!("{what}: {e}") })
}

fn audio_mode(mode: &str) -> Result<AudioMode, EngineError> {
    match mode.trim().to_ascii_lowercase().as_str() {
        "sub" | "" => Ok(AudioMode::Sub),
        "dub" => Ok(AudioMode::Dub),
        other => Err(EngineError::BadInput { reason: format!("audio mode {other:?} (want sub or dub)") }),
    }
}

fn built_in(key: Option<&str>) -> Option<String> {
    key.map(str::trim).filter(|s| !s.is_empty()).map(str::to_string)
}

// ── Runtime ────────────────────────────────────────────────────────────

fn runtime() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(WORKER_THREADS)
            .thread_name("velmora-engine")
            .enable_all()
            .build()
            .expect("the engine's Tokio runtime could not start")
    })
}

/// Run `fut` on the engine's runtime and wait for it from whichever executor
/// UniFFI polls us on.
async fn on_runtime<T, F>(fut: F) -> Result<T, EngineError>
where
    T: Send + 'static,
    F: Future<Output = Result<T, EngineError>> + Send + 'static,
{
    runtime().spawn(fut).await.map_err(|e| failed(format!("engine task stopped: {e}")))?
}

// ── Bridge types ───────────────────────────────────────────────────────

/// Where the engine keeps its files. All paths are absolute.
#[derive(uniffi::Record)]
pub struct EngineConfig {
    /// Persistent cache (`Context.cacheDir/engine`).
    pub cache_dir: String,
    /// Downloads and torrent pieces.
    pub download_dir: String,
    /// The install's vault key (32 bytes, created on first launch).
    pub vault_key_path: String,
    /// Saved `Preferences` as JSON; `None` for the defaults.
    pub preferences_json: Option<String>,
}

/// Receives progress as JSON while a long call runs (search batches, source events).
#[uniffi::export(callback_interface)]
pub trait EventSink: Send + Sync {
    fn on_event(&self, json: String);
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SubtitleTrack {
    label: String,
    lang: Option<String>,
    url: String,
}

/// Same shape as the desktop's `open_stream` result.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenedStream {
    /// Handle for `close_stream` / `torrent_status`.
    id: String,
    /// Loopback URL to hand to ExoPlayer.
    url: String,
    torrent: bool,
    subtitles: Vec<SubtitleTrack>,
}

struct Session {
    path: String,
    torrent: Option<Arc<TorrentJob>>,
}

#[derive(Serialize)]
struct Check {
    name: &'static str,
    ok: bool,
    detail: String,
    ms: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SelfTest {
    version: &'static str,
    arch: &'static str,
    stream_base: String,
    checks: Vec<Check>,
}

// ── The engine object ──────────────────────────────────────────────────

#[derive(uniffi::Object)]
pub struct VelmoraEngine {
    engine: Arc<Engine>,
    sealed: Arc<stream::Sealed>,
    port: u16,
    cache_dir: PathBuf,
    sessions: Arc<Mutex<HashMap<String, Session>>>,
}

#[uniffi::export]
impl VelmoraEngine {
    /// Start the engine and its stream server. Blocks briefly; call it off
    /// the main thread.
    #[uniffi::constructor]
    pub fn new(config: EngineConfig) -> Result<Arc<Self>, EngineError> {
        logcat::init();

        let preferences = match config.preferences_json.as_deref().map(str::trim) {
            Some(json) if !json.is_empty() => from_json::<Preferences>("preferences", json)?,
            _ => Preferences::default(),
        };
        let mut registry = Registry::default();
        registry.tmdb_read_token = built_in(TMDB_READ_TOKEN);
        registry.tmdb_api_key = built_in(TMDB_API_KEY);
        registry.thesportsdb_key = built_in(SPORTSDB_KEY);
        registry.football_data_token = built_in(FOOTBALL_DATA_TOKEN);

        let cache_dir = PathBuf::from(&config.cache_dir);
        let download_dir = PathBuf::from(&config.download_dir);
        for dir in [&cache_dir, &download_dir] {
            std::fs::create_dir_all(dir).map_err(|e| failed(format!("{}: {e}", dir.display())))?;
        }
        let opts = EngineOptions {
            preferences,
            registry,
            cache_dir: Some(cache_dir.clone()),
            download_dir: Some(download_dir),
            tools_dir: Some(cache_dir.join("tools")),
            ..Default::default()
        };

        let key = stream::load_key(Path::new(&config.vault_key_path)).map_err(failed)?;
        let sealed = Arc::new(stream::Sealed::new(Arc::new(Vault::new(key))));

        let rt = runtime();
        // The download manager spawns tasks, so the engine is built inside the runtime.
        let engine = Arc::new(rt.block_on(async move { Engine::new(opts) }).map_err(failed)?);
        // Downloads are sealed with the same key the stream server opens them with.
        engine.downloads().set_vault(Some(sealed.vault.clone()));
        let port = rt.block_on(stream::start(engine.clone(), sealed.clone())).map_err(failed)?;
        tracing::info!("engine ready, stream server on 127.0.0.1:{port}");

        Ok(Arc::new(Self { engine, sealed, port, cache_dir, sessions: Arc::new(Mutex::new(HashMap::new())) }))
    }

    /// `http://127.0.0.1:{port}`, the base of every playable URL.
    pub fn stream_base(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    /// Whether TMDB keys were built in (otherwise metadata comes from Cinemeta).
    pub fn has_tmdb(&self) -> bool {
        self.engine.registry().has_tmdb()
    }

    pub fn preferences(&self) -> Result<String, EngineError> {
        to_json(&*self.engine.preferences())
    }

    pub fn set_preferences(&self, json: String) -> Result<(), EngineError> {
        let prefs: Preferences = from_json("preferences", &json)?;
        self.engine.set_preferences(prefs);
        Ok(())
    }

    /// Pull the remote registry (mirrors, kill switches, host aliases).
    /// On failure the engine keeps the registry it has.
    pub async fn refresh_config(&self) -> Result<(), EngineError> {
        let engine = self.engine.clone();
        on_runtime(async move {
            match tokio::time::timeout(CONFIG_TIMEOUT, engine.refresh_registry_default()).await {
                Ok(Ok(())) => Ok(()),
                Ok(Err(e)) => Err(failed(e)),
                Err(_) => Err(failed("the config download timed out")),
            }
        })
        .await
    }

    /// Trending rails, as `Rail[]`.
    pub async fn home(&self) -> Result<String, EngineError> {
        let engine = self.engine.clone();
        on_runtime(async move { to_json(&engine.home().await) }).await
    }

    /// Merged results across providers, as `TitleSummary[]`.
    pub async fn search(&self, query: String) -> Result<String, EngineError> {
        let engine = self.engine.clone();
        on_runtime(async move { to_json(&engine.search(&query).await) }).await
    }

    /// Results looked over for relevance, as `Narrowed`.
    pub async fn search_smart(&self, query: String) -> Result<String, EngineError> {
        let engine = self.engine.clone();
        on_runtime(async move { to_json(&engine.search_smart(&query).await) }).await
    }

    /// One `SearchBatch` per provider to `sink` as each answers; returns when all have.
    pub async fn search_stream(&self, query: String, sink: Box<dyn EventSink>) -> Result<(), EngineError> {
        let engine = self.engine.clone();
        on_runtime(async move {
            let batches = engine.search_stream(&query);
            futures::pin_mut!(batches);
            while let Some(batch) = batches.next().await {
                sink.on_event(to_json(&batch)?);
            }
            Ok(())
        })
        .await
    }

    /// Full title page data for one provider's copy, as `TitleDetail`.
    pub async fn detail(&self, origin_json: String) -> Result<String, EngineError> {
        let origin: ProviderRef = from_json("origin", &origin_json)?;
        let engine = self.engine.clone();
        on_runtime(async move { to_json(&engine.detail(&origin).await.map_err(failed)?) }).await
    }

    // ── Browse: catalog lists behind the Browse tabs and title pages ──

    /// Titles from a TMDB list endpoint (`/discover/movie`, `/trending/tv/week`,
    /// `/movie/603/recommendations`...), as `TitleSummary[]`. `query_json` is a
    /// flat object of query parameters; `field` names the array (usually
    /// `results`); `kind` is `movie` or `tv` for lists that don't say.
    pub async fn tmdb_list(&self, path: String, query_json: String, field: String, kind: String) -> Result<String, EngineError> {
        let query: HashMap<String, serde_json::Value> = from_json("query", &query_json)?;
        let pairs: Vec<(String, String)> = query
            .into_iter()
            .map(|(k, v)| (k, v.as_str().map(str::to_owned).unwrap_or_else(|| v.to_string())))
            .collect();
        let engine = self.engine.clone();
        on_runtime(async move {
            let show_adult = engine.preferences().show_adult;
            let mut items = engine.tmdb().list(&path, &pairs, &field, &kind).await.map_err(failed)?;
            items.retain(|i| show_adult || !i.badges.iter().any(|b| b == "18+"));
            to_json(&items)
        })
        .await
    }

    /// A read-only TMDB call (the engine only allows catalog paths), as TMDB's
    /// own JSON. The title page uses it for cast photos, the age rating, the
    /// gallery and "More like this" in one request.
    pub async fn tmdb_raw(&self, path: String, query_json: String) -> Result<String, EngineError> {
        let query: HashMap<String, serde_json::Value> = from_json("query", &query_json)?;
        let pairs: Vec<(String, String)> = query
            .into_iter()
            .map(|(k, v)| (k, v.as_str().map(str::to_owned).unwrap_or_else(|| v.to_string())))
            .collect();
        let engine = self.engine.clone();
        on_runtime(async move { to_json(&engine.tmdb().raw(&path, &pairs).await.map_err(failed)?) }).await
    }

    /// Summaries for TMDB ids like `movie/603` or `tv/1396`, as `TitleSummary[]`.
    pub async fn tmdb_titles(&self, ids_json: String) -> Result<String, EngineError> {
        let ids: Vec<String> = from_json("ids", &ids_json)?;
        let engine = self.engine.clone();
        on_runtime(async move { to_json(&engine.tmdb().titles(&ids).await) }).await
    }

    /// Anime from AniList filters (`search`, `genres`, `sort`, `year`,
    /// `season`, `status`, `format`), as `TitleSummary[]`.
    pub async fn anilist_find(&self, filters_json: String) -> Result<String, EngineError> {
        let filters: serde_json::Value = from_json("filters", &filters_json)?;
        let engine = self.engine.clone();
        on_runtime(async move { to_json(&engine.anilist().find(filters).await.map_err(failed)?) }).await
    }

    // ── Downloads ──

    /// Queue a download of `source_json`; returns its id. `title` names the
    /// file ("Dune (2021)", "Frieren S01E03"); season and episode pick the
    /// file inside a torrent pack. Files are sealed into `.vlm` vault files.
    pub async fn download_start(&self, source_json: String, title: String, season: Option<u32>, episode: Option<u32>) -> Result<u64, EngineError> {
        let source: Source = from_json("source", &source_json)?;
        let engine = self.engine.clone();
        on_runtime(async move {
            let req = universeapi::download::DownloadRequest { source, title, dest_dir: None, episode: season.zip(episode) };
            Ok(engine.downloads().enqueue(req).await)
        })
        .await
    }

    /// Every download this run, as `DownloadStatus[]`.
    pub async fn downloads_list(&self) -> Result<String, EngineError> {
        let engine = self.engine.clone();
        on_runtime(async move { to_json(&engine.downloads().list().await) }).await
    }

    pub async fn download_pause(&self, id: u64) -> Result<(), EngineError> {
        let engine = self.engine.clone();
        on_runtime(async move {
            engine.downloads().pause(id).await;
            Ok(())
        })
        .await
    }

    /// Resume a paused or failed download; HTTP and HLS pick up where they stopped.
    pub async fn download_resume(&self, id: u64) -> Result<(), EngineError> {
        let engine = self.engine.clone();
        on_runtime(async move { engine.downloads().resume(id).await.map_err(failed) }).await
    }

    pub async fn download_cancel(&self, id: u64) -> Result<(), EngineError> {
        let engine = self.engine.clone();
        on_runtime(async move {
            engine.downloads().cancel(id).await;
            Ok(())
        })
        .await
    }

    /// Forget finished and cancelled downloads (failed ones stay to retry).
    pub async fn downloads_clear(&self) -> Result<(), EngineError> {
        let engine = self.engine.clone();
        on_runtime(async move {
            engine.downloads().clear_finished().await;
            Ok(())
        })
        .await
    }

    /// Finished videos in the download folder, newest first, as `LocalFile[]`.
    pub async fn downloads_library(&self) -> Result<String, EngineError> {
        let engine = self.engine.clone();
        on_runtime(async move { to_json(&engine.downloads().library().await) }).await
    }

    pub fn download_dir(&self) -> String {
        self.engine.downloads().dir().display().to_string()
    }

    pub fn set_download_dir(&self, dir: String) -> Result<(), EngineError> {
        let path = PathBuf::from(&dir);
        std::fs::create_dir_all(&path).map_err(|e| failed(format!("{dir}: {e}")))?;
        self.engine.downloads().set_dir(path);
        Ok(())
    }

    /// Make a downloaded file playable, as `OpenedStream`: vault files are
    /// decrypted on the fly from `/v/`, plain ones served from `/f/`, and
    /// subtitles saved beside the video come along.
    pub fn open_local(&self, path: String) -> Result<String, EngineError> {
        let file = PathBuf::from(&path);
        if !file.is_file() {
            return Err(EngineError::BadInput { reason: format!("{path} isn't a file") });
        }
        let name = file.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_string();
        let token = if universeapi::vault::is_vault(&name) {
            format!("/v/{}", self.sealed.register(file.clone()).map_err(failed)?)
        } else {
            format!("/f/{}", self.sealed.register_plain(file.clone()).map_err(failed)?)
        };
        let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or_default().to_string();
        let mut subtitles = Vec::new();
        if let Some(dir) = file.parent() {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for e in entries.flatten() {
                    let n = e.file_name().to_string_lossy().to_string();
                    let Some(rest) = n.strip_prefix(&format!("{stem}.")) else { continue };
                    let ext = rest.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
                    if !matches!(ext.as_str(), "vtt" | "srt" | "ass" | "ssa") {
                        continue;
                    }
                    let lang = rest.rsplit_once('.').map(|(l, _)| l.to_string()).filter(|l| !l.is_empty() && !l.starts_with("sub"));
                    if let Ok(t) = self.sealed.register_plain(e.path()) {
                        subtitles.push(SubtitleTrack { label: lang.clone().unwrap_or_else(|| "Subtitles".into()), lang, url: format!("{}/f/{t}#.{ext}", self.stream_base()) });
                    }
                }
            }
        }
        to_json(&OpenedStream { id: token.clone(), url: format!("{}{token}", self.stream_base()), torrent: false, subtitles })
    }

    /// Delete a downloaded video and the subtitles saved beside it.
    pub fn delete_local(&self, path: String) -> Result<(), EngineError> {
        let file = PathBuf::from(&path);
        let downloads = self.engine.downloads().dir();
        // Only files inside the download folder.
        let inside = file.canonicalize().ok().zip(downloads.canonicalize().ok()).is_some_and(|(f, d)| f.starts_with(d));
        if !inside {
            return Err(EngineError::BadInput { reason: "that file isn't in the download folder".into() });
        }
        let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or_default().to_string();
        if let Some(dir) = file.parent() {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for e in entries.flatten() {
                    let n = e.file_name().to_string_lossy().to_string();
                    let sub = n.strip_prefix(&format!("{stem}.")).is_some_and(|r| {
                        let ext = r.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
                        matches!(ext.as_str(), "vtt" | "srt" | "ass" | "ssa")
                    });
                    if sub {
                        let _ = std::fs::remove_file(e.path());
                    }
                }
            }
        }
        std::fs::remove_file(&file).map_err(|e| failed(format!("couldn't delete {path}: {e}")))
    }

    // ── Live TV ──

    /// One page of channels (`TvQuery`: country, category, text, offset, limit), as `TvPage`.
    pub async fn tv_channels(&self, query_json: String) -> Result<String, EngineError> {
        let query: universeapi::tv::TvQuery = from_json("tv query", &query_json)?;
        let engine = self.engine.clone();
        on_runtime(async move { to_json(&engine.tv().channels(&query).await.map_err(failed)?) }).await
    }

    /// Channel categories with counts, as `Facet[]`.
    pub async fn tv_categories(&self) -> Result<String, EngineError> {
        let engine = self.engine.clone();
        on_runtime(async move { to_json(&engine.tv().categories().await.map_err(failed)?) }).await
    }

    /// Countries with channel counts, as `Facet[]`.
    pub async fn tv_countries(&self) -> Result<String, EngineError> {
        let engine = self.engine.clone();
        on_runtime(async move { to_json(&engine.tv().countries().await.map_err(failed)?) }).await
    }

    // ── Sports ──

    /// Matches being played now, as `Match[]`.
    pub async fn sports_live(&self) -> Result<String, EngineError> {
        let engine = self.engine.clone();
        on_runtime(async move { to_json(&engine.sports().live().await.map_err(failed)?) }).await
    }

    /// Fixtures over the next `days` days, as `Match[]`.
    pub async fn sports_upcoming(&self, days: u32) -> Result<String, EngineError> {
        let engine = self.engine.clone();
        on_runtime(async move { to_json(&engine.sports().upcoming(days.clamp(1, 7)).await.map_err(failed)?) }).await
    }

    /// Results from the last `days` days, as `Match[]`.
    pub async fn sports_results(&self, days: u32) -> Result<String, EngineError> {
        let engine = self.engine.clone();
        on_runtime(async move { to_json(&engine.sports().results(days.clamp(1, 7)).await.map_err(failed)?) }).await
    }

    /// Ranked, probed sources for a movie or one episode, as `SourcesReport`.
    /// `target_json` is `{"type":"movie"}` or `{"type":"episode","season":1,"episode":2}`.
    pub async fn sources(&self, title_json: String, target_json: String, mode: String) -> Result<String, EngineError> {
        let title: TitleSummary = from_json("title", &title_json)?;
        let target: Target = from_json("target", &target_json)?;
        let mode = audio_mode(&mode)?;
        let engine = self.engine.clone();
        on_runtime(async move { to_json(&engine.sources(&title, target, mode).await) }).await
    }

    /// [`Self::sources`], with each `SourcesEvent` sent to `sink` as a provider finishes.
    pub async fn sources_watch(
        &self,
        title_json: String,
        target_json: String,
        mode: String,
        sink: Box<dyn EventSink>,
    ) -> Result<String, EngineError> {
        let title: TitleSummary = from_json("title", &title_json)?;
        let target: Target = from_json("target", &target_json)?;
        let mode = audio_mode(&mode)?;
        let engine = self.engine.clone();
        on_runtime(async move {
            let report = engine
                .sources_each(&title, target, mode, move |event| {
                    if let Ok(json) = serde_json::to_string(&event) {
                        sink.on_event(json);
                    }
                })
                .await;
            to_json(&report)
        })
        .await
    }

    /// A cached source didn't play: look them up afresh next time.
    pub async fn forget_sources(&self, title_json: String, target_json: String, mode: String) -> Result<(), EngineError> {
        let title: TitleSummary = from_json("title", &title_json)?;
        let target: Target = from_json("target", &target_json)?;
        let mode = audio_mode(&mode)?;
        let engine = self.engine.clone();
        on_runtime(async move {
            engine.forget_sources(&title, target, mode).await;
            Ok(())
        })
        .await
    }

    /// Register a source with the stream server and return `OpenedStream`.
    /// Torrents join the session first (metadata can take a while).
    pub async fn open_stream(
        &self,
        source_json: String,
        season: Option<u32>,
        episode: Option<u32>,
    ) -> Result<String, EngineError> {
        let source: Source = from_json("source", &source_json)?;
        let engine = self.engine.clone();
        let sessions = self.sessions.clone();
        let base = self.stream_base();
        on_runtime(async move {
            let (path, torrent) = if source.is_torrent() {
                let magnet = source.torrent.as_ref().map(|t| t.magnet.clone()).unwrap_or_else(|| source.url.clone());
                let te = engine.torrents().await.map_err(failed)?;
                let job = Arc::new(te.add(&magnet, season.zip(episode)).await.map_err(failed)?);
                (engine.proxy().register_torrent(job.clone()), Some(job))
            } else {
                (engine.proxy().register(&source).map_err(failed)?, None)
            };
            let opened = OpenedStream {
                id: path.clone(),
                url: format!("{base}{path}"),
                torrent: torrent.is_some(),
                subtitles: source
                    .subtitles
                    .iter()
                    .filter_map(|t| Some(SubtitleTrack { label: t.label.clone(), lang: t.lang.clone(), url: t.url.clone()? }))
                    .collect(),
            };
            if let Ok(mut map) = sessions.lock() {
                map.insert(path.clone(), Session { path, torrent });
            }
            to_json(&opened)
        })
        .await
    }

    /// Forget a stream; a streaming torrent is removed along with its pieces.
    pub async fn close_stream(&self, id: String) -> Result<(), EngineError> {
        let session = self.sessions.lock().ok().and_then(|mut s| s.remove(&id));
        let Some(session) = session else { return Ok(()) };
        let engine = self.engine.clone();
        on_runtime(async move {
            engine.proxy().unregister(&session.path);
            if let Some(job) = session.torrent {
                let te = engine.torrents().await.map_err(failed)?;
                // The proxy dropped its handle above, so this is normally the last one.
                match Arc::try_unwrap(job) {
                    Ok(job) => te.remove(job, true).await.map_err(failed)?,
                    Err(job) => te.pause(&job).await.map_err(failed)?,
                }
            }
            Ok(())
        })
        .await
    }

    /// Download progress of a streaming torrent as `TorrentProgress`, or `None`.
    pub async fn torrent_status(&self, id: String) -> Result<Option<String>, EngineError> {
        let job = self.sessions.lock().ok().and_then(|s| s.get(&id).and_then(|x| x.torrent.clone()));
        let Some(job) = job else { return Ok(None) };
        let engine = self.engine.clone();
        on_runtime(async move {
            let te = engine.torrents().await.map_err(failed)?;
            Ok(Some(to_json(&te.progress(&job))?))
        })
        .await
    }

    /// Checks that each part of the engine works on this device: the stream
    /// server, the vault (sealed and read back through the server), the
    /// torrent session, and TLS (the remote config). Returns `SelfTest` JSON.
    pub async fn self_test(&self) -> Result<String, EngineError> {
        let engine = self.engine.clone();
        let sealed = self.sealed.clone();
        let port = self.port;
        let scratch = self.cache_dir.join("selftest");
        let stream_base = self.stream_base();
        on_runtime(async move {
            let mut checks = Vec::new();

            let t = Instant::now();
            let r = http_get(port, "/selftest", None).await;
            checks.push(Check {
                name: "Stream server",
                ok: r.as_ref().is_ok_and(|(head, _)| head.starts_with("HTTP/1.1 ")),
                detail: match &r {
                    Ok((head, _)) => format!("answered: {}", head.lines().next().unwrap_or_default()),
                    Err(e) => e.to_string(),
                },
                ms: ms(t),
            });

            let t = Instant::now();
            let vault = vault_round_trip(&sealed, port, &scratch).await;
            checks.push(Check { name: "Vault", ok: vault.is_ok(), detail: vault.unwrap_or_else(|e| e), ms: ms(t) });
            let _ = tokio::fs::remove_dir_all(&scratch).await;

            let t = Instant::now();
            let torrents = engine.torrents().await;
            checks.push(Check {
                name: "Torrent engine",
                ok: torrents.is_ok(),
                detail: match torrents {
                    Ok(_) => "session started".into(),
                    Err(e) => e.to_string(),
                },
                ms: ms(t),
            });

            let t = Instant::now();
            let config = tokio::time::timeout(CONFIG_TIMEOUT, engine.refresh_registry_default()).await;
            checks.push(Check {
                name: "Network and TLS",
                ok: matches!(config, Ok(Ok(()))),
                detail: match config {
                    Ok(Ok(())) => "remote config downloaded".into(),
                    Ok(Err(e)) => e.to_string(),
                    Err(_) => "timed out".into(),
                },
                ms: ms(t),
            });

            checks.push(Check {
                name: "TMDB",
                ok: engine.registry().has_tmdb(),
                detail: if engine.registry().has_tmdb() {
                    "keys built in".into()
                } else {
                    "no keys built in, metadata falls back to Cinemeta".into()
                },
                ms: 0,
            });

            to_json(&SelfTest { version: env!("CARGO_PKG_VERSION"), arch: std::env::consts::ARCH, stream_base, checks })
        })
        .await
    }
}

/// The bridge's version and the CPU it was built for, e.g. `0.6.6 (aarch64)`.
#[uniffi::export]
pub fn engine_version() -> String {
    format!("{} ({})", env!("CARGO_PKG_VERSION"), std::env::consts::ARCH)
}

// ── Self-test helpers ──────────────────────────────────────────────────

fn ms(t: Instant) -> u64 {
    t.elapsed().as_millis() as u64
}

/// Seal a random file, then read a range of it back through `/v/` and compare.
async fn vault_round_trip(sealed: &stream::Sealed, port: u16, dir: &Path) -> Result<String, String> {
    const SIZE: usize = 384 * 1024;
    const FROM: usize = 100_000;
    const TO: usize = 165_535;
    tokio::fs::create_dir_all(dir).await.map_err(|e| e.to_string())?;
    let mut plain = vec![0u8; SIZE];
    getrandom::getrandom(&mut plain).map_err(|e| e.to_string())?;
    let src = dir.join("sample.mp4");
    let dst = dir.join(format!("sample.{}", universeapi::vault::EXT));
    tokio::fs::write(&src, &plain).await.map_err(|e| e.to_string())?;
    let path = sealed.vault.seal(&src, &dst).await.map_err(|e| e.to_string())?;

    let on_disk = tokio::fs::read(&path).await.map_err(|e| e.to_string())?;
    if on_disk.windows(64).any(|w| w == &plain[FROM..FROM + 64]) {
        return Err("the sealed file still holds readable video bytes".into());
    }

    let token = sealed.register(path)?;
    let (head, body) = http_get(port, &format!("/v/{token}"), Some(&format!("bytes={FROM}-{}", TO - 1)))
        .await
        .map_err(|e| e.to_string())?;
    if !head.starts_with("HTTP/1.1 206") {
        return Err(format!("range read answered {}", head.lines().next().unwrap_or_default()));
    }
    if body.as_slice() != &plain[FROM..TO] {
        return Err(format!("read back {} bytes that don't match", body.len()));
    }
    Ok(format!("sealed {} KB, read {} bytes back from the middle", SIZE / 1024, TO - FROM))
}

/// A bare HTTP/1.1 GET against our own server; returns the head and the body.
async fn http_get(port: u16, path: &str, range: Option<&str>) -> std::io::Result<(String, Vec<u8>)> {
    let mut sock = tokio::net::TcpStream::connect(("127.0.0.1", port)).await?;
    let mut req = format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n");
    if let Some(r) = range {
        req.push_str(&format!("Range: {r}\r\n"));
    }
    req.push_str("Connection: close\r\n\r\n");
    sock.write_all(req.as_bytes()).await?;
    let mut buf = Vec::new();
    tokio::time::timeout(Duration::from_secs(10), sock.read_to_end(&mut buf))
        .await
        .map_err(|_| std::io::Error::other("no answer in 10 s"))??;
    let split = buf
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(|| std::io::Error::other("the answer had no header"))?;
    let head = String::from_utf8_lossy(&buf[..split]).into_owned();
    Ok((head, buf[split + 4..].to_vec()))
}
