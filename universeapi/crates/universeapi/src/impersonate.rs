//! Chrome-fingerprint fallback (feature `impersonate`).
//!
//! Some sites (Coflix, EZTV, Cloudflare-fronted mirrors) refuse clients whose
//! TLS and HTTP/2 handshake doesn't look like a browser, especially from
//! datacenter IPs. `wreq` reproduces Chrome's handshake on BoringSSL. The
//! engine only uses it after the normal client is refused, so the common
//! path stays on rustls.

use async_trait::async_trait;
use wreq_util::Emulation;

use crate::error::{Error, Result};
use crate::net::BlockedFallback;

pub struct ChromeFallback {
    client: wreq::Client,
}

impl ChromeFallback {
    pub fn new() -> Result<Self> {
        let client = wreq::Client::builder()
            .emulation(Emulation::Chrome147)
            .build()
            .map_err(|e| Error::Config(format!("chrome fallback client: {e}")))?;
        Ok(Self { client })
    }
}

#[async_trait]
impl BlockedFallback for ChromeFallback {
    async fn fetch_text(&self, url: &str, headers: &[(String, String)]) -> Result<String> {
        let mut rb = self.client.get(url);
        for (k, v) in headers {
            // the emulation profile sets its own User-Agent; keep it consistent
            if !k.eq_ignore_ascii_case("user-agent") {
                rb = rb.header(k.as_str(), v.as_str());
            }
        }
        let resp = rb.send().await.map_err(|e| Error::Transport(format!("chrome fallback request to {url} failed: {e}")))?;
        let status = resp.status().as_u16();
        let body = resp.text().await.map_err(|e| Error::parse("chrome fallback body", e.to_string()))?;
        if !(200..300).contains(&status) {
            return Err(Error::Http { status, url: url.to_string() });
        }
        if crate::net::looks_like_challenge(&body) {
            return Err(Error::Blocked { url: url.to_string() });
        }
        Ok(body)
    }
}
