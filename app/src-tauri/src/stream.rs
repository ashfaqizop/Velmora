//! Loopback HTTP server that feeds mpv.
//!
//! mpv gets plain `http://127.0.0.1:{port}/p/...` or `/t/...` URLs; every
//! request goes through the engine's `MediaProxy`, which adds the headers a
//! host needs, rewrites HLS/DASH manifests so segments come back here too,
//! and serves torrents while they download. Each request gets its own
//! connection (`Connection: close`), which mpv and FFmpeg handle fine,
//! including the range requests they send when seeking.

use std::sync::Arc;

use futures::StreamExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use universeapi::proxy::ProxyBody;
use universeapi::Engine;

const MAX_HEAD: usize = 16 * 1024;

/// Bind to a free loopback port and serve forever; returns the port.
pub async fn start(engine: Arc<Engine>) -> std::io::Result<u16> {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
    let port = listener.local_addr()?.port();
    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((sock, _)) => {
                    let engine = engine.clone();
                    tokio::spawn(async move {
                        if let Err(e) = serve(sock, engine).await {
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

async fn serve(mut sock: TcpStream, engine: Arc<Engine>) -> std::io::Result<()> {
    let Some(req) = read_request(&mut sock).await? else {
        return Ok(());
    };
    let head_only = req.method == "HEAD";
    if req.method != "GET" && !head_only {
        return respond_text(&mut sock, 405, "method not allowed").await;
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
