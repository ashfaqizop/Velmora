//! Loopback HTTP server that feeds mpv.
//!
//! mpv gets plain `http://127.0.0.1:{port}/p/...` or `/t/...` URLs; every
//! request goes through the engine's `MediaProxy`, which adds the headers a
//! host needs, rewrites HLS/DASH manifests so segments come back here too,
//! and serves torrents while they download. Each request gets its own
//! connection (`Connection: close`), which mpv and FFmpeg handle fine,
//! including the range requests they send when seeking.
//!
//! Downloaded vault files (see `universeapi::vault`) are served from
//! `/v/{token}`, decrypted on the fly with full range support.

use std::collections::HashMap;
use std::io::SeekFrom;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use futures::StreamExt;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use universeapi::proxy::ProxyBody;
use universeapi::vault::Vault;
use universeapi::Engine;

const MAX_HEAD: usize = 16 * 1024;

/// Vault files the player may open, by random token, so the server never
/// takes a file path from a URL.
pub struct Sealed {
    pub vault: Arc<Vault>,
    files: std::sync::Mutex<HashMap<String, PathBuf>>,
}

impl Sealed {
    pub fn new(vault: Arc<Vault>) -> Self {
        Self { vault, files: std::sync::Mutex::new(HashMap::new()) }
    }

    /// Make `path` playable; returns the token for `/v/{token}`.
    pub fn register(&self, path: PathBuf) -> Result<String, String> {
        let mut raw = [0u8; 16];
        getrandom::getrandom(&mut raw).map_err(|e| e.to_string())?;
        let token: String = raw.iter().map(|b| format!("{b:02x}")).collect();
        if let Ok(mut files) = self.files.lock() {
            files.insert(token.clone(), path);
        }
        Ok(token)
    }

    fn get(&self, token: &str) -> Option<PathBuf> {
        self.files.lock().ok()?.get(token).cloned()
    }
}

/// The install's vault key, created on first launch. A file of the wrong size
/// is an error rather than replaced, because a new key would orphan every download.
pub fn load_key(path: &Path) -> Result<[u8; 32], String> {
    match std::fs::read(path) {
        Ok(bytes) => bytes.try_into().map_err(|_| format!("{} is damaged", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let mut key = [0u8; 32];
            getrandom::getrandom(&mut key).map_err(|e| e.to_string())?;
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            }
            std::fs::write(path, key).map_err(|e| e.to_string())?;
            Ok(key)
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Bind to a free loopback port and serve forever; returns the port.
pub async fn start(engine: Arc<Engine>, sealed: Arc<Sealed>) -> std::io::Result<u16> {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
    let port = listener.local_addr()?.port();
    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((sock, _)) => {
                    let engine = engine.clone();
                    let sealed = sealed.clone();
                    tokio::spawn(async move {
                        if let Err(e) = serve(sock, engine, sealed).await {
                            tracing::debug!("stream connection ended: {e}");
                        }
                    });
                }
                Err(e) => tracing::warn!("stream accept failed: {e}"),
            }
        }
    });
    tracing::info!("stream server on 127.0.0.1:{port}");
    Ok(port)
}

struct Request {
    method: String,
    target: String,
    range: Option<String>,
}

async fn read_request(sock: &mut TcpStream) -> std::io::Result<Option<Request>> {
    let mut buf = Vec::with_capacity(1024);
    let mut chunk = [0u8; 1024];
    loop {
        let n = sock.read(&mut chunk).await?;
        if n == 0 {
            return Ok(None);
        }
        buf.extend_from_slice(&chunk[..n]);
        if buf.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
        if buf.len() > MAX_HEAD {
            return Ok(None);
        }
    }
    let head = String::from_utf8_lossy(&buf);
    let mut lines = head.split("\r\n");
    let mut first = lines.next().unwrap_or_default().split(' ');
    let method = first.next().unwrap_or_default().to_string();
    let target = first.next().unwrap_or("/").to_string();
    let range = lines.take_while(|l| !l.is_empty()).find_map(|l| {
        let (k, v) = l.split_once(':')?;
        k.trim().eq_ignore_ascii_case("range").then(|| v.trim().to_string())
    });
    Ok(Some(Request { method, target, range }))
}

async fn serve(mut sock: TcpStream, engine: Arc<Engine>, sealed: Arc<Sealed>) -> std::io::Result<()> {
    let Some(req) = read_request(&mut sock).await? else {
        return Ok(());
    };
    let head_only = req.method == "HEAD";
    if req.method != "GET" && !head_only {
        return respond_text(&mut sock, 405, "method not allowed").await;
    }
    if let Some(token) = req.target.strip_prefix("/v/") {
        return serve_vault(&mut sock, &sealed, token, req.range.as_deref(), head_only).await;
    }

    let resp = match engine.proxy().handle(&req.target, req.range.as_deref()).await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(target = %req.target, "proxy error: {e}");
            return respond_text(&mut sock, 502, &e.to_string()).await;
        }
    };

    let mut head = format!("HTTP/1.1 {} {}\r\n", resp.status, reason(resp.status));
    let mut has_length = false;
    for (k, v) in &resp.headers {
        has_length |= k.eq_ignore_ascii_case("content-length");
        head.push_str(&format!("{k}: {v}\r\n"));
    }
    match resp.body {
        ProxyBody::Bytes(bytes) => {
            if !has_length {
                head.push_str(&format!("Content-Length: {}\r\n", bytes.len()));
            }
            head.push_str("Connection: close\r\n\r\n");
            sock.write_all(head.as_bytes()).await?;
            if !head_only {
                sock.write_all(&bytes).await?;
            }
        }
        ProxyBody::Stream(mut stream) => {
            head.push_str("Connection: close\r\n\r\n");
            sock.write_all(head.as_bytes()).await?;
            if !head_only {
                while let Some(chunk) = stream.next().await {
                    sock.write_all(&chunk?).await?;
                }
            }
        }
    }
    sock.shutdown().await
}

/// Decrypt a vault file to the player, honouring range requests so seeking works.
async fn serve_vault(
    sock: &mut TcpStream,
    sealed: &Sealed,
    token: &str,
    range: Option<&str>,
    head_only: bool,
) -> std::io::Result<()> {
    let Some(path) = sealed.get(token) else {
        return respond_text(sock, 404, "unknown file").await;
    };
    let header = match Vault::read_header(&path).await {
        Ok(h) => h,
        Err(e) => return respond_text(sock, 404, &e.to_string()).await,
    };
    let mut file = tokio::fs::File::open(&path).await?;
    let total = file.metadata().await?.len().saturating_sub(header.len);
    let (start, end) = match range {
        None if total == 0 => (0, 0),
        None => (0, total - 1),
        Some(r) => match parse_range(r, total) {
            Some(span) => span,
            None => {
                let msg = format!(
                    "HTTP/1.1 416 {}\r\nContent-Range: bytes */{total}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    reason(416)
                );
                sock.write_all(msg.as_bytes()).await?;
                return sock.shutdown().await;
            }
        },
    };
    let len = if total == 0 { 0 } else { end - start + 1 };
    let mut head = match range {
        Some(_) => format!("HTTP/1.1 206 {}\r\nContent-Range: bytes {start}-{end}/{total}\r\n", reason(206)),
        None => format!("HTTP/1.1 200 {}\r\n", reason(200)),
    };
    head.push_str(&format!(
        "Content-Type: {}\r\nAccept-Ranges: bytes\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n",
        header.content_type()
    ));
    sock.write_all(head.as_bytes()).await?;
    if head_only || len == 0 {
        return sock.shutdown().await;
    }
    file.seek(SeekFrom::Start(header.len + start)).await?;
    let mut buf = vec![0u8; 256 * 1024];
    let (mut offset, mut left) = (start, len);
    while left > 0 {
        let want = left.min(buf.len() as u64) as usize;
        let n = file.read(&mut buf[..want]).await?;
        if n == 0 {
            break;
        }
        sealed.vault.apply(&header.nonce, offset, &mut buf[..n]);
        sock.write_all(&buf[..n]).await?;
        offset += n as u64;
        left -= n as u64;
    }
    sock.shutdown().await
}

/// `bytes=a-b`, `bytes=a-` or `bytes=-n` (the first range only) → inclusive span.
fn parse_range(spec: &str, total: u64) -> Option<(u64, u64)> {
    let r = spec.trim().strip_prefix("bytes=")?.split(',').next()?.trim();
    let (a, b) = r.split_once('-')?;
    if total == 0 {
        return None;
    }
    let (start, end) = if a.is_empty() {
        let n: u64 = b.parse().ok()?;
        if n == 0 {
            return None;
        }
        (total.saturating_sub(n), total - 1)
    } else {
        let s: u64 = a.parse().ok()?;
        let e = if b.is_empty() { total - 1 } else { b.parse::<u64>().ok()?.min(total - 1) };
        (s, e)
    };
    (start <= end && start < total).then_some((start, end))
}

async fn respond_text(sock: &mut TcpStream, status: u16, text: &str) -> std::io::Result<()> {
    let msg = format!(
        "HTTP/1.1 {status} {}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{text}",
        reason(status),
        text.len()
    );
    sock.write_all(msg.as_bytes()).await?;
    sock.shutdown().await
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        206 => "Partial Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        416 => "Range Not Satisfiable",
        502 => "Bad Gateway",
        _ => "Status",
    }
}
