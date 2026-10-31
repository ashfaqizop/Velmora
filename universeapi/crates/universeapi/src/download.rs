//! Download manager.
//!
//! * direct files (MP4/MKV/zip): HTTP with `Range` resume from a `.part` file
//! * HLS/DASH: the ffmpeg sidecar remuxes into MKV, keeping every audio and
//!   subtitle track (`-map 0 -c copy`)
//! * torrents: the shared librqbit session (feature `torrent`)
//!
//! Jobs run under a semaphore sized from `max_concurrent_downloads` (change
//! it live with [`DownloadManager::set_max_concurrent`]). Season zips are
//! extracted and deleted, and external subtitles are saved beside the video.
//! The app watches progress through [`DownloadManager::subscribe`].

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::sync::{watch, Mutex, Semaphore};

use crate::error::{Error, Result};
use crate::model::{Source, StreamFormat};
use crate::vault::{Header, Vault};
use crate::providers::Ctx;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRequest {
    pub source: Source,
    /// Human title used for the file name ("Dune (2021)", "Frieren S01E03").
    pub title: String,
    /// Folder for this download; defaults to the manager's download dir.
    #[serde(default)]
    pub dest_dir: Option<PathBuf>,
    /// (season, episode) — picks the right file inside torrent packs.
    #[serde(default)]
    pub episode: Option<(u32, u32)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum DownloadState {
    Queued,
    Running,
    Paused,
    Extracting,
    Completed { path: PathBuf },
    Failed { error: String },
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadStatus {
    pub id: u64,
    pub title: String,
    pub state: DownloadState,
    pub downloaded: u64,
    pub total: Option<u64>,
    pub bytes_per_sec: u64,
}

struct Job {
    req: DownloadRequest,
    tx: watch::Sender<DownloadStatus>,
    stop: AtomicBool,
    pause: AtomicBool,
    /// The final file this job writes, picked on its first run.
    target: std::sync::Mutex<Option<PathBuf>>,
}

impl Job {
    fn update(&self, f: impl FnOnce(&mut DownloadStatus)) {
        self.tx.send_modify(f);
    }

    fn interrupted(&self) -> bool {
        self.stop.load(Ordering::Relaxed) || self.pause.load(Ordering::Relaxed)
    }

    fn target(&self) -> Option<PathBuf> {
        self.target.lock().ok().and_then(|t| t.clone())
    }
}

/// The in-progress files a download leaves next to its target.
fn partials(target: &Path) -> [PathBuf; 2] {
    [PathBuf::from(format!("{}.part", target.display())), target.with_extension("part.mkv")]
}

async fn discard_partials(target: &Path) {
    for p in partials(target) {
        let _ = tokio::fs::remove_file(p).await;
    }
}

fn is_video(name: &str) -> bool {
    Path::new(name)
        .extension()
        .and_then(|x| x.to_str())
        .is_some_and(|x| matches!(x.to_ascii_lowercase().as_str(), "mkv" | "mp4" | "webm" | "avi" | "m4v" | "vlm"))
}

/// Seal every video under `dir` (an extracted season pack) into vault files.
async fn seal_tree(vault: &Vault, dir: &Path) -> Result<()> {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let mut entries = tokio::fs::read_dir(&d).await.map_err(io)?;
        while let Some(e) = entries.next_entry().await.map_err(io)? {
            let path = e.path();
            if e.file_type().await.map_err(io)?.is_dir() {
                stack.push(path);
                continue;
            }
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            if is_video(name) && !crate::vault::is_vault(name) {
                vault.seal(&path, &path.with_extension(crate::vault::EXT)).await?;
            }
        }
    }
    Ok(())
}

/// A finished video found in the download folder.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalFile {
    pub path: PathBuf,
    /// File name without the extension: "Saiyaara (2025) 480p".
    pub title: String,
    pub size: u64,
    /// Unix seconds.
    pub modified: u64,
}

fn is_partial(name: &str) -> bool {
    name.ends_with(".part") || name.ends_with(".part.mkv")
}

/// "Dune (2021) 1080p.mkv" -> "Dune (2021) 1080p (2).mkv"
fn numbered(target: &Path, n: u32) -> PathBuf {
    let stem = target.file_stem().and_then(|s| s.to_str()).unwrap_or("download");
    match target.extension().and_then(|e| e.to_str()) {
        Some(ext) => target.with_file_name(format!("{stem} ({n}).{ext}")),
        None => target.with_file_name(format!("{stem} ({n})")),
    }
}

#[cfg(feature = "torrent")]
pub type TorrentSlot = Arc<tokio::sync::OnceCell<crate::torrent::TorrentEngine>>;

pub struct DownloadManager {
    ctx: Arc<Ctx>,
    dir: std::sync::RwLock<PathBuf>,
    ffmpeg: std::sync::RwLock<PathBuf>,
    slots: Arc<Semaphore>,
    max: AtomicUsize,
    jobs: Mutex<HashMap<u64, Arc<Job>>>,
    next: AtomicU64,
    /// When set, finished downloads are stored as vault files only Velmora can play.
    vault: std::sync::RwLock<Option<Arc<Vault>>>,
    #[cfg(feature = "torrent")]
    torrents: TorrentSlot,
}

const INTERRUPTED: &str = "download interrupted";

/// Safe file name: no path separators or characters Windows rejects.
pub fn sanitize(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| if matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') || c.is_control() { ' ' } else { c })
        .collect();
    let joined = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    let trimmed = joined.trim_end_matches(['.', ' ']).to_string();
    if trimmed.is_empty() {
        "download".into()
    } else {
        trimmed.chars().take(150).collect()
    }
}

fn extension(src: &Source) -> String {
    if let Some(ext) = src.filename.as_deref().and_then(|f| Path::new(f).extension()).and_then(|e| e.to_str()) {
        return ext.to_ascii_lowercase();
    }
    match src.format {
        StreamFormat::Mp4 => "mp4",
        StreamFormat::Webm => "webm",
        StreamFormat::Archive => "zip",
        _ => "mkv",
    }
    .into()
}

/// "Dune (2021) 1080p.mkv"
pub fn file_name(title: &str, src: &Source) -> String {
    let res = src.tags.resolution.map(|r| if r >= 2160 { " 4K".to_string() } else { format!(" {r}p") }).unwrap_or_default();
    format!("{}{res}.{}", sanitize(title), extension(src))
}

impl DownloadManager {
    pub fn new(
        ctx: Arc<Ctx>,
        dir: PathBuf,
        max_concurrent: usize,
        #[cfg(feature = "torrent")] torrents: TorrentSlot,
    ) -> Self {
        let max = max_concurrent.max(1);
        Self {
            ctx,
            dir: std::sync::RwLock::new(dir),
            ffmpeg: std::sync::RwLock::new(PathBuf::from("ffmpeg")),
            slots: Arc::new(Semaphore::new(max)),
            max: AtomicUsize::new(max),
            jobs: Mutex::new(HashMap::new()),
            next: AtomicU64::new(1),
            vault: std::sync::RwLock::new(None),
            #[cfg(feature = "torrent")]
            torrents,
        }
    }

    /// Store new downloads as vault files with this key holder (`None` = plain files).
    pub fn set_vault(&self, vault: Option<Arc<Vault>>) {
        if let Ok(mut v) = self.vault.write() {
            *v = vault;
        }
    }

    fn vault(&self) -> Option<Arc<Vault>> {
        self.vault.read().ok().and_then(|v| v.clone())
    }

    /// The folder new downloads go to.
    pub fn dir(&self) -> PathBuf {
        self.dir.read().map(|d| d.clone()).unwrap_or_default()
    }

    /// Change the folder for downloads started from now on. Running and
    /// paused jobs keep the file they already claimed.
    pub fn set_dir(&self, dir: PathBuf) {
        if let Ok(mut d) = self.dir.write() {
            *d = dir;
        }
    }

    /// Path of the bundled ffmpeg sidecar (default: `ffmpeg` on PATH).
    pub fn set_ffmpeg(&self, path: PathBuf) {
        if let Ok(mut p) = self.ffmpeg.write() {
            *p = path;
        }
    }

    fn ffmpeg(&self) -> PathBuf {
        self.ffmpeg.read().map(|p| p.clone()).unwrap_or_else(|_| PathBuf::from("ffmpeg"))
    }

    /// Change how many downloads run at once; applies to queued jobs immediately.
    pub fn set_max_concurrent(&self, n: usize) {
        let n = n.max(1);
        let old = self.max.swap(n, Ordering::SeqCst);
        if n > old {
            self.slots.add_permits(n - old);
        } else if n < old {
            self.slots.forget_permits(old - n);
        }
    }

    pub async fn enqueue(self: &Arc<Self>, req: DownloadRequest) -> u64 {
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        let status = DownloadStatus { id, title: req.title.clone(), state: DownloadState::Queued, downloaded: 0, total: None, bytes_per_sec: 0 };
        let (tx, _) = watch::channel(status);
        let job = Arc::new(Job { req, tx, stop: AtomicBool::new(false), pause: AtomicBool::new(false), target: std::sync::Mutex::new(None) });
        self.jobs.lock().await.insert(id, job.clone());
        self.spawn(job);
        id
    }

    fn spawn(self: &Arc<Self>, job: Arc<Job>) {
        let me = self.clone();
        tokio::spawn(async move {
            let Ok(_permit) = me.slots.clone().acquire_owned().await else { return };
            if job.stop.load(Ordering::Relaxed) {
                job.update(|s| s.state = DownloadState::Cancelled);
                return;
            }
            job.update(|s| s.state = DownloadState::Running);
            let result = me.run(&job).await;
            let state = match result {
                Ok(path) => DownloadState::Completed { path },
                Err(_) if job.stop.load(Ordering::Relaxed) => {
                    if let Some(t) = job.target() {
                        discard_partials(&t).await;
                    }
                    DownloadState::Cancelled
                }
                Err(_) if job.pause.load(Ordering::Relaxed) => DownloadState::Paused,
                Err(e) => DownloadState::Failed { error: e.to_string() },
            };
            job.update(|s| {
                s.state = state;
                s.bytes_per_sec = 0;
            });
        });
    }

    pub async fn subscribe(&self, id: u64) -> Option<watch::Receiver<DownloadStatus>> {
        self.jobs.lock().await.get(&id).map(|j| j.tx.subscribe())
    }

    pub async fn list(&self) -> Vec<DownloadStatus> {
        let mut v: Vec<DownloadStatus> = self.jobs.lock().await.values().map(|j| j.tx.borrow().clone()).collect();
        v.sort_by_key(|s| s.id);
        v
    }

    pub async fn pause(&self, id: u64) {
        if let Some(j) = self.jobs.lock().await.get(&id) {
            j.pause.store(true, Ordering::Relaxed);
        }
    }

    /// Resume a paused or failed job; HTTP downloads continue from the `.part` file.
    pub async fn resume(self: &Arc<Self>, id: u64) -> Result<()> {
        let job = self.jobs.lock().await.get(&id).cloned().ok_or_else(|| Error::NotFound(format!("download {id}")))?;
        if matches!(job.tx.borrow().state, DownloadState::Running | DownloadState::Queued | DownloadState::Extracting) {
            return Ok(());
        }
        job.pause.store(false, Ordering::Relaxed);
        job.stop.store(false, Ordering::Relaxed);
        job.update(|s| s.state = DownloadState::Queued);
        self.spawn(job);
        Ok(())
    }

    pub async fn cancel(&self, id: u64) {
        if let Some(j) = self.jobs.lock().await.get(&id) {
            j.stop.store(true, Ordering::Relaxed);
            let idle = matches!(j.tx.borrow().state, DownloadState::Paused | DownloadState::Failed { .. });
            if idle {
                j.update(|s| s.state = DownloadState::Cancelled);
                if let Some(t) = j.target() {
                    discard_partials(&t).await;
                }
            }
        }
    }

    /// Forget completed and cancelled jobs. Failed ones stay so they can be retried.
    pub async fn clear_finished(&self) {
        self.jobs.lock().await.retain(|_, j| !matches!(j.tx.borrow().state, DownloadState::Completed { .. } | DownloadState::Cancelled));
    }

    /// Delete partial files no job owns. Jobs don't survive a restart, so at
    /// launch every `.part` in the download folder is left over from a past session.
    pub async fn remove_orphaned_parts(&self) {
        let owned: Vec<PathBuf> =
            self.jobs.lock().await.values().filter_map(|j| j.target()).flat_map(|t| partials(&t)).collect();
        let Ok(mut entries) = tokio::fs::read_dir(self.dir()).await else { return };
        while let Ok(Some(e)) = entries.next_entry().await {
            let path = e.path();
            let partial = path.file_name().and_then(|n| n.to_str()).is_some_and(is_partial);
            if partial && !owned.contains(&path) && e.file_type().await.is_ok_and(|t| t.is_file()) {
                let _ = tokio::fs::remove_file(&path).await;
            }
        }
    }

    /// Finished videos in the download folder named after `title`
    /// ("Dune (2021)" matches "Dune (2021) 1080p.mkv"), for "you already have this".
    pub async fn existing(&self, title: &str) -> Vec<PathBuf> {
        let base = sanitize(title);
        let mut found = Vec::new();
        let Ok(mut entries) = tokio::fs::read_dir(self.dir()).await else { return found };
        while let Ok(Some(e)) = entries.next_entry().await {
            let Some(name) = e.file_name().to_str().map(str::to_string) else { continue };
            // The title must end at a space or dot, so "Dune" doesn't match "Dune Messiah".
            let boundary = name.strip_prefix(&base).is_some_and(|r| r.starts_with(' ') || r.starts_with('.'));
            if boundary && is_video(&name) && !is_partial(&name) {
                found.push(e.path());
            }
        }
        found.sort();
        found
    }

    /// Every finished video in the download folder, newest first, so the
    /// Downloads page can list files from earlier sessions too.
    pub async fn library(&self) -> Vec<LocalFile> {
        let mut files = Vec::new();
        let Ok(mut entries) = tokio::fs::read_dir(self.dir()).await else { return files };
        while let Ok(Some(e)) = entries.next_entry().await {
            let Some(name) = e.file_name().to_str().map(str::to_string) else { continue };
            if is_partial(&name) || !is_video(&name) {
                continue;
            }
            let Ok(meta) = e.metadata().await else { continue };
            if !meta.is_file() {
                continue;
            }
            let modified = meta.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_secs());
            let title = Path::new(&name).file_stem().and_then(|s| s.to_str()).unwrap_or(&name).to_string();
            files.push(LocalFile { path: e.path(), title, size: meta.len(), modified });
        }
        files.sort_by(|a, b| b.modified.cmp(&a.modified));
        files
    }

    /// Pick this job's file: the planned name, numbered if a finished file or
    /// another job already has it. Resumes keep the name picked the first time.
    async fn claim_target(&self, job: &Job, planned: PathBuf) -> PathBuf {
        if let Some(t) = job.target() {
            return t;
        }
        let jobs = self.jobs.lock().await;
        let taken: Vec<PathBuf> = jobs
            .values()
            .filter(|j| !std::ptr::eq(j.as_ref(), job) && !matches!(j.tx.borrow().state, DownloadState::Cancelled))
            .filter_map(|j| j.target())
            .collect();
        let free = |p: &PathBuf| !p.exists() && !taken.contains(p);
        let mut target = planned.clone();
        let mut n = 2;
        while !free(&target) {
            target = numbered(&planned, n);
            n += 1;
        }
        if let Ok(mut t) = job.target.lock() {
            *t = Some(target.clone());
        }
        target
    }

    async fn run(&self, job: &Job) -> Result<PathBuf> {
        let dir = job.req.dest_dir.clone().unwrap_or_else(|| self.dir());
        tokio::fs::create_dir_all(&dir).await.map_err(io)?;
        let src = &job.req.source;
        let planned = dir.join(file_name(&job.req.title, src));
        let vault = self.vault();
        let zip = src.format == StreamFormat::Archive || extension(src) == "zip";
        let path = match src.format {
            #[cfg(feature = "torrent")]
            StreamFormat::Torrent => {
                let path = self.torrent(job, &dir).await?;
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_string();
                return match vault {
                    Some(v) if is_video(&name) && !crate::vault::is_vault(&name) => {
                        let dst = self.claim_target(job, path.with_extension(crate::vault::EXT)).await;
                        v.seal(&path, &dst).await
                    }
                    _ => Ok(path),
                };
            }
            #[cfg(not(feature = "torrent"))]
            StreamFormat::Torrent => return Err(Error::Unsupported("built without the torrent feature".into())),
            StreamFormat::Hls | StreamFormat::Dash => match &vault {
                // ffmpeg writes a plain MKV beside the vault file, which is sealed and removed.
                Some(v) => {
                    let target = self.claim_target(job, planned.with_extension(crate::vault::EXT)).await;
                    let plain = self.remux(job, &target.with_extension("remux.mkv")).await?;
                    v.seal(&plain, &target).await?
                }
                None => {
                    let target = self.claim_target(job, planned.with_extension("mkv")).await;
                    self.remux(job, &target).await?
                }
            },
            _ => match &vault {
                Some(v) if !zip => {
                    let target = self.claim_target(job, planned.with_extension(crate::vault::EXT)).await;
                    self.fetch(job, &target, Some((v.clone(), extension(src)))).await?
                }
                _ => {
                    let target = self.claim_target(job, planned).await;
                    self.fetch(job, &target, None).await?
                }
            },
        };
        self.save_subtitles(src, &path).await;
        if zip {
            job.update(|s| s.state = DownloadState::Extracting);
            let out = path.with_extension("");
            extract_zip(&path, &out).await?;
            let _ = tokio::fs::remove_file(&path).await;
            if let Some(v) = &vault {
                seal_tree(v, &out).await?;
            }
            return Ok(out);
        }
        Ok(path)
    }

    /// HTTP download with resume. With `seal`, bytes are encrypted as they're
    /// written (a vault file); the header in the `.part` lets a resume continue.
    async fn fetch(&self, job: &Job, target: &Path, seal: Option<(Arc<Vault>, String)>) -> Result<PathBuf> {
        let src = &job.req.source;
        let part = PathBuf::from(format!("{}.part", target.display()));
        let on_disk = tokio::fs::metadata(&part).await.map(|m| m.len()).unwrap_or(0);
        // Payload bytes already on disk, after the vault header if there is one.
        let mut start = on_disk;
        let header: Option<Header> = match &seal {
            None => None,
            Some((v, ext)) => match Vault::read_header(&part).await {
                Ok(h) if on_disk >= h.len => {
                    start = on_disk - h.len;
                    Some(h)
                }
                _ => {
                    start = 0;
                    Some(v.header(ext))
                }
            },
        };

        let mut rb = self.ctx.http.client().get(&src.url).timeout(Duration::from_secs(60 * 60 * 6));
        for (k, v) in &src.headers {
            rb = rb.header(k.as_str(), v.as_str());
        }
        if start > 0 {
            rb = rb.header("Range", format!("bytes={start}-"));
        }
        let resp = rb.send().await.map_err(|e| Error::Network { url: src.url.clone(), source: e })?;
        let status = resp.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(Error::Http { status, url: src.url.clone() });
        }
        if start > 0 && status == 200 {
            start = 0; // server ignored Range: start over
        }
        let total = resp.content_length().map(|l| l + start).or(src.size_bytes);
        job.update(|s| {
            s.total = total;
            s.downloaded = start;
        });

        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(start > 0)
            .truncate(start == 0)
            .open(&part)
            .await
            .map_err(io)?;
        if let (0, Some(h)) = (start, &header) {
            file.write_all(&h.to_bytes()).await.map_err(io)?;
        }
        let mut stream = resp.bytes_stream();
        let mut done = start;
        let mut window = (Instant::now(), done);
        while let Some(chunk) = stream.next().await {
            if job.interrupted() {
                file.flush().await.map_err(io)?;
                return Err(Error::Transport(INTERRUPTED.into()));
            }
            let chunk = chunk.map_err(|e| Error::Network { url: src.url.clone(), source: e })?;
            match (&seal, &header) {
                (Some((v, _)), Some(h)) => {
                    let mut buf = chunk.to_vec();
                    v.apply(&h.nonce, done, &mut buf);
                    file.write_all(&buf).await.map_err(io)?;
                }
                _ => file.write_all(&chunk).await.map_err(io)?,
            }
            done += chunk.len() as u64;
            let elapsed = window.0.elapsed();
            if elapsed >= Duration::from_millis(500) {
                let speed = ((done - window.1) as f64 / elapsed.as_secs_f64()) as u64;
                job.update(|s| {
                    s.downloaded = done;
                    s.bytes_per_sec = speed;
                });
                window = (Instant::now(), done);
            }
        }
        file.flush().await.map_err(io)?;
        drop(file);
        tokio::fs::rename(&part, target).await.map_err(io)?;
        job.update(|s| s.downloaded = done);
        Ok(target.to_path_buf())
    }

    /// HLS/DASH → MKV through the ffmpeg sidecar, keeping every track.
    async fn remux(&self, job: &Job, target: &Path) -> Result<PathBuf> {
        let src = &job.req.source;
        let tmp = target.with_extension("part.mkv");
        let headers: String = src.headers.iter().map(|(k, v)| format!("{k}: {v}\r\n")).collect();
        let mut cmd = tokio::process::Command::new(self.ffmpeg());
        cmd.args(["-hide_banner", "-loglevel", "error", "-nostats", "-y"]);
        if !headers.is_empty() {
            cmd.arg("-headers").arg(&headers);
        }
        cmd.arg("-i")
            .arg(&src.url)
            .args(["-map", "0", "-c", "copy", "-progress", "pipe:1"])
            .arg(&tmp)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);
        let mut child = cmd.spawn().map_err(|e| Error::Unsupported(format!("ffmpeg not available: {e}")))?;
        let stdout = child.stdout.take().ok_or_else(|| Error::Transport("ffmpeg stdout".into()))?;
        let mut lines = tokio::io::BufReader::new(stdout).lines();
        let mut window = (Instant::now(), 0u64);

        loop {
            tokio::select! {
                line = lines.next_line() => {
                    match line.map_err(io)? {
                        Some(l) => {
                            if let Some(v) = l.strip_prefix("total_size=").and_then(|v| v.trim().parse::<u64>().ok()) {
                                let elapsed = window.0.elapsed();
                                let speed = if elapsed.as_millis() > 0 { ((v.saturating_sub(window.1)) as f64 / elapsed.as_secs_f64()) as u64 } else { 0 };
                                job.update(|s| { s.downloaded = v; s.bytes_per_sec = speed; });
                                window = (Instant::now(), v);
                            }
                        }
                        None => break,
                    }
                }
                _ = tokio::time::sleep(Duration::from_millis(500)) => {
                    if job.interrupted() {
                        let _ = child.kill().await;
                        let _ = tokio::fs::remove_file(&tmp).await;
                        return Err(Error::Transport(INTERRUPTED.into()));
                    }
                }
            }
        }
        let out = child.wait_with_output().await.map_err(io)?;
        if !out.status.success() {
            let _ = tokio::fs::remove_file(&tmp).await;
            let err = String::from_utf8_lossy(&out.stderr);
            return Err(Error::Transport(format!("ffmpeg failed: {}", crate::net::prefix(err.trim(), 300))));
        }
        tokio::fs::rename(&tmp, target).await.map_err(io)?;
        Ok(target.to_path_buf())
    }

    #[cfg(feature = "torrent")]
    async fn torrent(&self, job: &Job, dir: &Path) -> Result<PathBuf> {
        let src = &job.req.source;
        let magnet = src.torrent.as_ref().map(|t| t.magnet.clone()).unwrap_or_else(|| src.url.clone());
        let te = self.torrents.get_or_try_init(|| crate::torrent::TorrentEngine::new(self.dir().join("torrents"))).await?;
        let tjob = te.add(&magnet, job.req.episode).await?;
        let name = tjob.selected_file().map(|f| f.name.clone()).unwrap_or_default();
        let seed = self.ctx.prefs().seed_after_download;

        let mut window = (Instant::now(), 0u64);
        let finished = {
            let fin = te.finish(&tjob, seed);
            tokio::pin!(fin);
            loop {
                tokio::select! {
                    r = &mut fin => break Some(r),
                    _ = tokio::time::sleep(Duration::from_secs(1)) => {
                        let p = te.progress(&tjob);
                        let elapsed = window.0.elapsed().as_secs_f64().max(0.001);
                        let speed = (p.downloaded.saturating_sub(window.1) as f64 / elapsed) as u64;
                        job.update(|s| { s.downloaded = p.downloaded; s.total = Some(p.total); s.bytes_per_sec = speed; });
                        window = (Instant::now(), p.downloaded);
                        if job.interrupted() {
                            break None;
                        }
                    }
                }
            }
        };
        match finished {
            Some(r) => {
                r?;
                let from = te.output_folder(&tjob).join(&name);
                let to = dir.join(Path::new(&name).file_name().unwrap_or_default());
                if from != to && tokio::fs::rename(&from, &to).await.is_ok() {
                    return Ok(to);
                }
                Ok(from)
            }
            None => {
                // A paused torrent keeps its pieces; a cancelled one is removed with its data.
                let cancelled = job.stop.load(Ordering::Relaxed);
                if cancelled {
                    te.remove(tjob, true).await?;
                } else {
                    te.pause(&tjob).await?;
                }
                Err(Error::Transport(INTERRUPTED.into()))
            }
        }
    }

    /// Save external subtitle tracks next to the video ("Movie.en.vtt").
    async fn save_subtitles(&self, src: &Source, video: &Path) {
        for (i, t) in src.subtitles.iter().enumerate() {
            let Some(url) = t.url.as_deref() else { continue };
            let ext = Path::new(url.split('?').next().unwrap_or(url)).extension().and_then(|e| e.to_str()).unwrap_or("vtt");
            let lang = t.lang.clone().unwrap_or_else(|| format!("sub{i}"));
            let out = video.with_extension(format!("{lang}.{ext}"));
            let headers: Vec<(&str, &str)> = src.headers.iter().filter(|(k, _)| k.eq_ignore_ascii_case("referer")).map(|(k, v)| (k.as_str(), v.as_str())).collect();
            if let Ok(resp) = self.ctx.http.get(url, &headers).await {
                if let Ok(bytes) = resp.bytes().await {
                    let _ = tokio::fs::write(&out, &bytes).await;
                }
            }
        }
    }
}

fn io(e: std::io::Error) -> Error {
    Error::Transport(format!("file system: {e}"))
}

/// Extract a zip into `out`, rejecting entries that escape the folder.
pub async fn extract_zip(zip_path: &Path, out: &Path) -> Result<()> {
    let zip_path = zip_path.to_path_buf();
    let out = out.to_path_buf();
    tokio::task::spawn_blocking(move || -> Result<()> {
        let file = std::fs::File::open(&zip_path).map_err(io)?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| Error::parse("zip archive", e.to_string()))?;
        std::fs::create_dir_all(&out).map_err(io)?;
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| Error::parse("zip entry", e.to_string()))?;
            let Some(rel) = entry.enclosed_name() else { continue };
            let dest = out.join(rel);
            if entry.is_dir() {
                std::fs::create_dir_all(&dest).map_err(io)?;
                continue;
            }
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent).map_err(io)?;
            }
            let mut f = std::fs::File::create(&dest).map_err(io)?;
            std::io::copy(&mut entry, &mut f).map_err(io)?;
        }
        Ok(())
    })
    .await
    .map_err(|e| Error::Transport(format!("extract task: {e}")))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ProviderId;

    #[test]
    fn names() {
        assert_eq!(sanitize("Dune: Part Two / IMAX?"), "Dune Part Two IMAX");
        assert_eq!(sanitize("..."), "download");
        let mut s = Source::new(ProviderId::HdHub, "HubDrive", "1080p x264", "https://r2.dev/abc");
        s.filename = Some("Dune.2021.1080p.mkv".into());
        assert_eq!(file_name("Dune (2021)", &s), "Dune (2021) 1080p.mkv");
        let hls = Source::new(ProviderId::ZokoAnime, "Zoko", "Episode 1", "https://x/master.m3u8");
        assert_eq!(file_name("Frieren S01E01", &hls), "Frieren S01E01.mkv");
    }

    fn test_manager(dir: PathBuf) -> DownloadManager {
        let http = crate::net::Http::new("test").unwrap();
        let ctx = Arc::new(Ctx::new(http, crate::cache::Cache::memory(), Default::default(), Default::default(), None));
        DownloadManager::new(
            ctx,
            dir,
            1,
            #[cfg(feature = "torrent")]
            Arc::new(tokio::sync::OnceCell::new()),
        )
    }

    #[test]
    fn numbering() {
        assert_eq!(numbered(Path::new("/d/Dune (2021) 1080p.mkv"), 2), Path::new("/d/Dune (2021) 1080p (2).mkv"));
        assert_eq!(partials(Path::new("/d/A.mp4"))[0], Path::new("/d/A.mp4.part"));
        assert!(is_partial("A.mp4.part") && is_partial("A.part.mkv") && !is_partial("A.mkv"));
    }

    #[tokio::test]
    async fn existing_and_orphans() {
        let dir = std::env::temp_dir().join(format!("uapi-dl-{}", fastrand::u64(..)));
        std::fs::create_dir_all(&dir).unwrap();
        for f in ["Dune (2021) 1080p.mkv", "Dune (2021) 1080p.en.srt", "Dune Messiah 1080p.mkv", "Dune (2021) 4K.mkv.part", "Old.part.mkv"] {
            std::fs::write(dir.join(f), b"x").unwrap();
        }
        let m = test_manager(dir.clone());
        assert_eq!(m.existing("Dune (2021)").await, vec![dir.join("Dune (2021) 1080p.mkv")]);
        assert_eq!(m.library().await.len(), 2);
        m.remove_orphaned_parts().await;
        assert!(!dir.join("Dune (2021) 4K.mkv.part").exists());
        assert!(!dir.join("Old.part.mkv").exists());
        assert!(dir.join("Dune (2021) 1080p.mkv").exists());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn zip_extract_rejects_traversal() {
        use std::io::Write;
        let dir = std::env::temp_dir().join(format!("uapi-zip-{}", fastrand::u64(..)));
        std::fs::create_dir_all(&dir).unwrap();
        let zpath = dir.join("pack.zip");
        {
            let f = std::fs::File::create(&zpath).unwrap();
            let mut w = zip::ZipWriter::new(f);
            let opts = zip::write::SimpleFileOptions::default();
            w.start_file("Show.S01E01.mkv", opts).unwrap();
            w.write_all(b"ep1").unwrap();
            w.start_file("../evil.txt", opts).unwrap();
            w.write_all(b"x").unwrap();
            w.finish().unwrap();
        }
        let out = dir.join("pack");
        extract_zip(&zpath, &out).await.unwrap();
        assert_eq!(std::fs::read(out.join("Show.S01E01.mkv")).unwrap(), b"ep1");
        assert!(!dir.join("evil.txt").exists());
        let _ = std::fs::remove_dir_all(dir);
    }
}
