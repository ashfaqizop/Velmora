//! First-launch runtime components (mpv DLLs, ffmpeg).
//!
//! The installer ships without them to stay small. On first launch the
//! splash calls [`install`], which reads `runtime.json` from the pinned
//! `runtime-v1` GitHub release, downloads each zip, checks its SHA-256,
//! and unpacks it into `<app local data>/runtime/v1`. A copy of the manifest
//! written there afterwards marks the install as complete, so later launches
//! need no network for this step.

use std::path::{Path, PathBuf};
use std::time::Duration;

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::ipc::Channel;
use tokio::io::AsyncWriteExt;
use universeapi::Engine;

/// Bump together with a new `runtime-vN` release when a component changes.
const RUNTIME_TAG: &str = "runtime-v1";
const RELEASE_BASE: &str = "https://github.com/ashfaqizop/Velmora/releases/download";
const MARKER: &str = "installed.json";
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(60 * 30);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: u32,
    pub components: Vec<Component>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: String,
    pub label: String,
    pub asset: String,
    pub sha256: String,
    pub size: u64,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    /// Label of the component being fetched ("Video player").
    pub label: String,
    /// Bytes received across all components so far.
    pub received: u64,
    /// Total bytes to download.
    pub total: u64,
}

pub fn dir(local_data: &Path) -> PathBuf {
    local_data.join("runtime").join(RUNTIME_TAG.trim_start_matches("runtime-"))
}

/// True when the marker exists and every file it lists is on disk.
pub fn is_ready(dir: &Path) -> bool {
    let Ok(text) = std::fs::read_to_string(dir.join(MARKER)) else { return false };
    let Ok(manifest) = serde_json::from_str::<Manifest>(&text) else { return false };
    manifest.components.iter().flat_map(|c| &c.files).all(|f| dir.join(f).is_file())
}

/// Point the DLL loader and the download manager at the runtime folder.
pub fn activate(dir: &Path, engine: &Engine) {
    #[cfg(windows)]
    set_dll_directory(dir);
    let ffmpeg = dir.join(if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" });
    if ffmpeg.is_file() {
        engine.downloads().set_ffmpeg(ffmpeg);
    }
}

/// Download and unpack every missing component, reporting progress.
pub async fn install(engine: &Engine, dir: &Path, progress: &Channel<Progress>) -> Result<(), String> {
    let client = engine.ctx().http.client().clone();
    let manifest_url = format!("{RELEASE_BASE}/{RUNTIME_TAG}/runtime.json");
    let manifest: Manifest = client
        .get(&manifest_url)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("couldn't reach GitHub: {e}"))?
        .json()
        .await
        .map_err(|e| format!("bad runtime manifest: {e}"))?;

    tokio::fs::create_dir_all(dir).await.map_err(|e| e.to_string())?;
    let missing: Vec<&Component> =
        manifest.components.iter().filter(|c| !c.files.iter().all(|f| dir.join(f).is_file())).collect();
    let total: u64 = missing.iter().map(|c| c.size).sum();
    let mut received = 0u64;

    for component in missing {
        let url = format!("{RELEASE_BASE}/{RUNTIME_TAG}/{}", component.asset);
        let zip_path = dir.join(format!("{}.part", component.asset));
        let mut file = tokio::fs::File::create(&zip_path).await.map_err(|e| e.to_string())?;
        let resp = client
            .get(&url)
            .timeout(DOWNLOAD_TIMEOUT)
            .send()
            .await
            .and_then(|r| r.error_for_status())
            .map_err(|e| format!("{}: {e}", component.label))?;

        let mut hasher = Sha256::new();
        let mut body = resp.bytes_stream();
        let _ = progress.send(Progress { label: component.label.clone(), received, total });
        let mut last_report = 0u64;
        while let Some(chunk) = body.next().await {
            let chunk = chunk.map_err(|e| format!("{}: {e}", component.label))?;
            hasher.update(&chunk);
            file.write_all(&chunk).await.map_err(|e| e.to_string())?;
            received += chunk.len() as u64;
            // About every 256 KB is plenty for a smooth ring.
            if received - last_report >= 256 * 1024 {
                last_report = received;
                let _ = progress.send(Progress { label: component.label.clone(), received, total });
            }
        }
        file.flush().await.map_err(|e| e.to_string())?;
        drop(file);

        let digest = format!("{:x}", hasher.finalize());
        if !digest.eq_ignore_ascii_case(&component.sha256) {
            let _ = tokio::fs::remove_file(&zip_path).await;
            return Err(format!("{} failed its integrity check", component.label));
        }
        universeapi::download::extract_zip(&zip_path, dir).await.map_err(|e| e.to_string())?;
        let _ = tokio::fs::remove_file(&zip_path).await;
    }

    let marker = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
    tokio::fs::write(dir.join(MARKER), marker).await.map_err(|e| e.to_string())?;
    activate(dir, engine);
    Ok(())
}

/// Lets the libmpv plugin's bare `LoadLibrary("libmpv-wrapper.dll")`, and
/// the wrapper's own load of `libmpv-2.dll`, find the runtime folder.
#[cfg(windows)]
fn set_dll_directory(dir: &Path) {
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "kernel32")]
    extern "system" {
        fn SetDllDirectoryW(path: *const u16) -> i32;
    }
    let wide: Vec<u16> = dir.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    // SAFETY: `wide` is a NUL-terminated UTF-16 path that outlives the call.
    let ok = unsafe { SetDllDirectoryW(wide.as_ptr()) };
    if ok == 0 {
        tracing::warn!("SetDllDirectoryW failed for {}", dir.display());
    }
}
