//! Rich Presence through the Discord app's local pipe.
//!
//! This is Discord's official, supported way: the Discord app on this PC
//! listens on `\\.\pipe\discord-ipc-N`, we say hello with our Application ID
//! and then send activities. No login, no token. If the Discord app isn't
//! running there's no pipe, and `connect` simply fails until it is.

use std::io;

use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const OP_HANDSHAKE: u32 = 0;
const OP_FRAME: u32 = 1;
const OP_CLOSE: u32 = 2;

#[cfg(windows)]
type Pipe = tokio::net::windows::named_pipe::NamedPipeClient;
#[cfg(unix)]
type Pipe = tokio::net::UnixStream;

pub struct Ipc {
    pipe: Pipe,
    nonce: u64,
}

impl Ipc {
    /// Find the Discord app's pipe (it may be any of 0 to 9) and say hello.
    pub async fn connect(client_id: &str) -> io::Result<Self> {
        let mut last = io::Error::new(io::ErrorKind::NotFound, "Discord isn't running");
        for n in 0..10 {
            match open(n).await {
                Ok(pipe) => {
                    let mut ipc = Ipc { pipe, nonce: 0 };
                    match ipc.handshake(client_id).await {
                        Ok(()) => return Ok(ipc),
                        Err(e) => last = e,
                    }
                }
                Err(e) => last = e,
            }
        }
        Err(last)
    }

    async fn handshake(&mut self, client_id: &str) -> io::Result<()> {
        self.write(OP_HANDSHAKE, &json!({ "v": 1, "client_id": client_id })).await?;
        // Discord answers with a READY dispatch, or closes on a bad ID.
        let (op, body) = self.read().await?;
        if op == OP_CLOSE {
            let why = body.get("message").and_then(Value::as_str).unwrap_or("closed").to_string();
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, why));
        }
        Ok(())
    }

    /// Set the activity, or clear it with `None`.
    pub async fn set_activity(&mut self, activity: Option<&Value>) -> io::Result<()> {
        self.nonce += 1;
        let frame = json!({
            "cmd": "SET_ACTIVITY",
            "args": { "pid": std::process::id(), "activity": activity },
            "nonce": self.nonce.to_string(),
        });
        self.write(OP_FRAME, &frame).await?;
        // Read the reply so the pipe never backs up. A rejected activity comes
        // back as an ERROR event; log it but keep the connection.
        let (op, body) = self.read().await?;
        if op == OP_CLOSE {
            return Err(io::Error::new(io::ErrorKind::ConnectionAborted, "Discord closed the pipe"));
        }
        if body.get("evt").and_then(Value::as_str) == Some("ERROR") {
            let msg = body.pointer("/data/message").and_then(Value::as_str).unwrap_or("rejected");
            tracing::warn!("Discord rejected the activity: {msg}");
        }
        Ok(())
    }

    async fn write(&mut self, op: u32, body: &Value) -> io::Result<()> {
        let data = serde_json::to_vec(body).map_err(io::Error::other)?;
        let mut buf = Vec::with_capacity(8 + data.len());
        buf.extend_from_slice(&op.to_le_bytes());
        buf.extend_from_slice(&(data.len() as u32).to_le_bytes());
        buf.extend_from_slice(&data);
        self.pipe.write_all(&buf).await?;
        self.pipe.flush().await
    }

    async fn read(&mut self) -> io::Result<(u32, Value)> {
        let mut head = [0u8; 8];
        self.pipe.read_exact(&mut head).await?;
        let op = u32::from_le_bytes([head[0], head[1], head[2], head[3]]);
        let len = u32::from_le_bytes([head[4], head[5], head[6], head[7]]) as usize;
        // A sane reply is a few KB; refuse anything absurd.
        if len > 1 << 20 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Discord sent an oversized frame"));
        }
        let mut data = vec![0u8; len];
        self.pipe.read_exact(&mut data).await?;
        let body = serde_json::from_slice(&data).unwrap_or(Value::Null);
        Ok((op, body))
    }
}

#[cfg(windows)]
async fn open(n: u8) -> io::Result<Pipe> {
    use tokio::net::windows::named_pipe::ClientOptions;
    ClientOptions::new().open(format!(r"\\.\pipe\discord-ipc-{n}"))
}

#[cfg(unix)]
async fn open(n: u8) -> io::Result<Pipe> {
    // Discord puts the socket in the first of these that's set.
    let dir = ["XDG_RUNTIME_DIR", "TMPDIR", "TMP", "TEMP"]
        .iter()
        .find_map(|k| std::env::var(k).ok())
        .unwrap_or_else(|| "/tmp".into());
    tokio::net::UnixStream::connect(format!("{dir}/discord-ipc-{n}")).await
}
