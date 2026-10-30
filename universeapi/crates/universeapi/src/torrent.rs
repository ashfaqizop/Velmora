//! Torrent engine on librqbit (pure Rust, builds for Windows and Android).
//!
//! * Adds a magnet paused, reads its file list, keeps only the wanted video
//!   (the right `SxxEyy` inside a season pack, otherwise the largest video),
//!   then starts downloading.
//! * [`TorrentEngine::stream`] gives an `AsyncRead + AsyncSeek` reader over
//!   the selected file. librqbit prioritises the pieces around the read
//!   position, which is what makes watching while downloading work.
//! * Seeding is off unless the user turns it on: a finished torrent is
//!   paused unless `seed` is true.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use librqbit::{AddTorrent, AddTorrentOptions, ManagedTorrent, Session, SessionOptions};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncSeek};

use crate::error::{Error, Result};
use crate::model::StreamFormat;

const METADATA_TIMEOUT: Duration = Duration::from_secs(120);
const VIDEO_EXT: &[&str] = &["mkv", "mp4", "m4v", "webm", "avi", "mov", "ts", "wmv"];

fn terr(e: impl std::fmt::Display) -> Error {
    Error::Torrent(e.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TorrentFile {
    pub index: usize,
    pub name: String,
    pub len: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TorrentProgress {
    pub downloaded: u64,
    pub total: u64,
    pub uploaded: u64,
    pub finished: bool,
}

/// A torrent added to the session, narrowed to one file.
pub struct TorrentJob {
    pub id: usize,
    pub info_hash: Option<String>,
    pub files: Vec<TorrentFile>,
    pub selected: usize,
    handle: Arc<ManagedTorrent>,
}

impl TorrentJob {
    pub fn selected_file(&self) -> Option<&TorrentFile> {
        self.files.iter().find(|f| f.index == self.selected)
    }
}

pub struct TorrentEngine {
    session: Arc<Session>,
}

impl TorrentEngine {
    /// Start a session that downloads into `dir`.
    pub async fn new(dir: PathBuf) -> Result<Self> {
        tokio::fs::create_dir_all(&dir).await.map_err(terr)?;
        let session = Session::new_with_opts(dir, SessionOptions::default()).await.map_err(terr)?;
        Ok(Self { session })
    }

    /// Add a magnet link or `.torrent` URL and select the file to fetch.
    /// `episode` = (season, episode) picks the matching file in a pack.
    pub async fn add(&self, magnet_or_url: &str, episode: Option<(u32, u32)>) -> Result<TorrentJob> {
        let opts = AddTorrentOptions { paused: true, overwrite: true, ..Default::default() };
        let added = tokio::time::timeout(METADATA_TIMEOUT, self.session.add_torrent(AddTorrent::from_url(magnet_or_url), Some(opts)))
            .await
            .map_err(|_| Error::Torrent("timed out fetching torrent metadata (no peers?)".into()))?
            .map_err(terr)?;
        let handle = added.into_handle().ok_or_else(|| Error::Torrent("torrent was listed but not added".into()))?;

        // librqbit initialises (checks existing pieces) even a paused torrent and
        // refuses file-selection changes until that is done.
        tokio::time::timeout(METADATA_TIMEOUT, handle.wait_until_initialized())
            .await
            .map_err(|_| Error::Torrent("timed out initialising torrent".into()))?
            .map_err(terr)?;
        let files = read_files(&handle)?;
        let selected = pick_file(&files, episode).ok_or_else(|| Error::Torrent("torrent has no files".into()))?;
        let only: HashSet<usize> = [selected].into_iter().collect();
        self.session.update_only_files(&handle, &only).await.map_err(terr)?;
        self.session.unpause(&handle).await.map_err(terr)?;

        Ok(TorrentJob { id: handle.id(), info_hash: info_hash(magnet_or_url), files, selected, handle })
    }

    /// Seekable reader over the selected file; pieces near the read position
    /// are fetched first, so playback can start before the download ends.
    pub async fn stream(&self, job: &TorrentJob) -> Result<impl AsyncRead + AsyncSeek + Unpin + Send + 'static> {
        job.handle.clone().stream(job.selected).await.map_err(terr)
    }

    pub fn progress(&self, job: &TorrentJob) -> TorrentProgress {
        let s = job.handle.stats();
        TorrentProgress { downloaded: s.progress_bytes, total: s.total_bytes, uploaded: s.uploaded_bytes, finished: s.finished }
    }

    /// Wait for the download to finish, then stop uploading unless `seed`.
    pub async fn finish(&self, job: &TorrentJob, seed: bool) -> Result<()> {
        job.handle.wait_until_completed().await.map_err(terr)?;
        if !seed {
            self.session.pause(&job.handle).await.map_err(terr)?;
        }
        Ok(())
    }

    pub async fn pause(&self, job: &TorrentJob) -> Result<()> {
        self.session.pause(&job.handle).await.map_err(terr)
    }

    pub async fn resume(&self, job: &TorrentJob) -> Result<()> {
        self.session.unpause(&job.handle).await.map_err(terr)
    }

    pub async fn remove(&self, job: TorrentJob, delete_files: bool) -> Result<()> {
        self.session.delete(job.id.into(), delete_files).await.map_err(terr)
    }

    pub fn output_folder(&self, job: &TorrentJob) -> PathBuf {
        job.handle.output_folder().to_path_buf()
    }
}

fn read_files(handle: &Arc<ManagedTorrent>) -> Result<Vec<TorrentFile>> {
    handle
        .with_metadata(|m| {
            m.file_infos
                .iter()
                .enumerate()
                .map(|(index, f)| TorrentFile { index, name: f.relative_filename.to_string_lossy().into_owned(), len: f.len })
                .collect::<Vec<_>>()
        })
        .map_err(terr)
}

fn is_video(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    VIDEO_EXT.iter().any(|e| lower.ends_with(&format!(".{e}"))) && !lower.contains("sample")
}

/// The file to download: the wanted episode in a pack, else the largest video, else the largest file.
pub fn pick_file(files: &[TorrentFile], episode: Option<(u32, u32)>) -> Option<usize> {
    let videos: Vec<&TorrentFile> = files.iter().filter(|f| is_video(&f.name)).collect();
    if let Some((season, ep)) = episode {
        let base = |f: &&TorrentFile| f.name.rsplit(['/', '\\']).next().unwrap_or(&f.name).to_string();
        let hit = videos.iter().find(|f| crate::providers::fourkhdhub::season_episode(&base(f)) == Some((season, Some(ep)))).or_else(|| {
            videos.iter().find(|f| crate::providers::torrents::names_episode(&base(f), ep))
        });
        if let Some(f) = hit {
            return Some(f.index);
        }
    }
    videos
        .iter()
        .max_by_key(|f| f.len)
        .map(|f| f.index)
        .or_else(|| files.iter().max_by_key(|f| f.len).map(|f| f.index))
}

/// Lower-case hex info hash from a magnet link.
pub fn info_hash(magnet: &str) -> Option<String> {
    let rest = magnet.split("btih:").nth(1)?;
    let hash: String = rest.chars().take_while(char::is_ascii_alphanumeric).collect();
    (!hash.is_empty()).then(|| hash.to_ascii_lowercase())
}

/// Container format of a torrent file, for the player.
pub fn file_format(name: &str) -> StreamFormat {
    StreamFormat::from_url_or_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(index: usize, name: &str, len: u64) -> TorrentFile {
        TorrentFile { index, name: name.into(), len }
    }

    #[test]
    fn picks_episode_in_pack() {
        let files = vec![
            f(0, "Show.S01/Show.S01E01.1080p.mkv", 2_000),
            f(1, "Show.S01/Show.S01E02.1080p.mkv", 2_100),
            f(2, "Show.S01/Sample/show.sample.mkv", 50),
            f(3, "Show.S01/Show.nfo", 1),
        ];
        assert_eq!(pick_file(&files, Some((1, 2))), Some(1));
        assert_eq!(pick_file(&files, None), Some(1), "largest video");
        let anime = vec![f(0, "[Sub] Frieren - 03 (1080p).mkv", 9), f(1, "[Sub] Frieren - 04 (1080p).mkv", 9)];
        assert_eq!(pick_file(&anime, Some((1, 4))), Some(1));
        assert_eq!(pick_file(&[f(0, "ubuntu.iso", 5)], None), Some(0), "non-video falls back to largest");
    }

    #[test]
    fn magnet_hash() {
        assert_eq!(info_hash("magnet:?xt=urn:btih:ABCDEF12&dn=x").as_deref(), Some("abcdef12"));
        assert_eq!(info_hash("https://x/y.torrent"), None);
    }
}
