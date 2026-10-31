//! Obfuscated downloads ("vault" files).
//!
//! A downloaded video is stored as `Name 1080p.vlm`: a short header, then the
//! video XORed with a ChaCha20 keystream. The key is per install and lives
//! with the app, so the file plays only through Velmora. ChaCha20 can seek, so
//! any byte range decrypts on its own: downloads resume mid-file and the
//! player's range requests work.
//!
//! Header: `VELMORA1` (8 bytes), nonce (12), extension length (1), extension.

use std::path::{Path, PathBuf};

use chacha20::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};
use chacha20::{ChaCha20, Key, Nonce};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::error::{Error, Result};

/// File extension of vault files.
pub const EXT: &str = "vlm";
const MAGIC: &[u8; 8] = b"VELMORA1";
const FIXED: usize = MAGIC.len() + 12 + 1;

/// The parsed header of a vault file.
#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    pub nonce: [u8; 12],
    /// The original container's extension ("mp4", "mkv").
    pub ext: String,
    /// Header size in bytes; the encrypted video starts here.
    pub len: u64,
}

impl Header {
    fn new(ext: &str) -> Self {
        let mut nonce = [0u8; 12];
        nonce.iter_mut().for_each(|b| *b = fastrand::u8(..));
        let ext: String = ext.chars().filter(|c| c.is_ascii_alphanumeric()).take(16).collect();
        let len = (FIXED + ext.len()) as u64;
        Self { nonce, ext, len }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.len as usize);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&self.nonce);
        out.push(self.ext.len() as u8);
        out.extend_from_slice(self.ext.as_bytes());
        out
    }

    /// Parse a header from the start of a file's bytes; `None` if it isn't one.
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < FIXED || &bytes[..MAGIC.len()] != MAGIC {
            return None;
        }
        let nonce: [u8; 12] = bytes[MAGIC.len()..MAGIC.len() + 12].try_into().ok()?;
        let n = bytes[FIXED - 1] as usize;
        let ext = std::str::from_utf8(bytes.get(FIXED..FIXED + n)?).ok()?.to_string();
        Some(Self { nonce, ext, len: (FIXED + n) as u64 })
    }

    /// MIME type for serving the decrypted video.
    pub fn content_type(&self) -> &'static str {
        match self.ext.as_str() {
            "mp4" | "m4v" => "video/mp4",
            "webm" => "video/webm",
            "avi" => "video/x-msvideo",
            _ => "video/x-matroska",
        }
    }
}

/// Holds the install's key and does the en/decryption.
pub struct Vault {
    key: [u8; 32],
}

impl std::fmt::Debug for Vault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Vault")
    }
}

impl Vault {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    /// A fresh header (random nonce) for a new file.
    pub fn header(&self, ext: &str) -> Header {
        Header::new(ext)
    }

    /// XOR `buf` with the keystream at `offset` bytes into the video.
    /// The same call encrypts and decrypts.
    pub fn apply(&self, nonce: &[u8; 12], offset: u64, buf: &mut [u8]) {
        let mut c = ChaCha20::new(Key::from_slice(&self.key), Nonce::from_slice(nonce));
        c.seek(offset);
        c.apply_keystream(buf);
    }

    /// Read the header of a vault file.
    pub async fn read_header(path: &Path) -> Result<Header> {
        let mut f = tokio::fs::File::open(path).await.map_err(io)?;
        let mut buf = [0u8; FIXED + 255];
        let mut got = 0;
        while got < buf.len() {
            let n = f.read(&mut buf[got..]).await.map_err(io)?;
            if n == 0 {
                break;
            }
            got += n;
        }
        Header::parse(&buf[..got]).ok_or_else(|| Error::parse("vault file", "not a Velmora download"))
    }

    /// Encrypt a finished plain file into a vault file at `dst`, then delete
    /// the plain one. Writes to `dst.part` first so a crash leaves no half file.
    pub async fn seal(&self, src: &Path, dst: &Path) -> Result<PathBuf> {
        let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("mkv").to_ascii_lowercase();
        let header = self.header(&ext);
        let part = PathBuf::from(format!("{}.part", dst.display()));
        let mut input = tokio::fs::File::open(src).await.map_err(io)?;
        let mut out = tokio::fs::File::create(&part).await.map_err(io)?;
        out.write_all(&header.to_bytes()).await.map_err(io)?;
        let mut buf = vec![0u8; 1 << 20];
        let mut offset = 0u64;
        loop {
            let n = input.read(&mut buf).await.map_err(io)?;
            if n == 0 {
                break;
            }
            self.apply(&header.nonce, offset, &mut buf[..n]);
            out.write_all(&buf[..n]).await.map_err(io)?;
            offset += n as u64;
        }
        out.flush().await.map_err(io)?;
        drop(out);
        drop(input);
        tokio::fs::rename(&part, dst).await.map_err(io)?;
        let _ = tokio::fs::remove_file(src).await;
        Ok(dst.to_path_buf())
    }
}

/// Whether a file name is a vault file.
pub fn is_vault(name: &str) -> bool {
    Path::new(name).extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case(EXT))
}

fn io(e: std::io::Error) -> Error {
    Error::Transport(format!("file system: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_round_trip() {
        let h = Header::new("mkv");
        let bytes = h.to_bytes();
        assert_eq!(bytes.len() as u64, h.len);
        assert_eq!(Header::parse(&bytes), Some(h));
        assert_eq!(Header::parse(b"not a vault file at all"), None);
    }

    #[test]
    fn decrypts_any_range() {
        let v = Vault::new([7u8; 32]);
        let nonce = [3u8; 12];
        let plain: Vec<u8> = (0..10_000u32).map(|i| (i % 251) as u8).collect();
        let mut sealed = plain.clone();
        v.apply(&nonce, 0, &mut sealed);
        assert_ne!(sealed, plain);
        // A slice from the middle decrypts on its own, as a range request would.
        let mut mid = sealed[4321..7777].to_vec();
        v.apply(&nonce, 4321, &mut mid);
        assert_eq!(mid, plain[4321..7777]);
    }

    #[tokio::test]
    async fn seal_file() {
        let dir = std::env::temp_dir().join(format!("uapi-vault-{}", fastrand::u64(..)));
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("Movie 1080p.mp4");
        std::fs::write(&src, b"hello video bytes").unwrap();
        let v = Vault::new([1u8; 32]);
        let dst = v.seal(&src, &dir.join("Movie 1080p.vlm")).await.unwrap();
        assert!(!src.exists());
        let h = Vault::read_header(&dst).await.unwrap();
        assert_eq!(h.ext, "mp4");
        let mut body = std::fs::read(&dst).unwrap()[h.len as usize..].to_vec();
        v.apply(&h.nonce, 0, &mut body);
        assert_eq!(body, b"hello video bytes");
        let _ = std::fs::remove_dir_all(dir);
    }
}
