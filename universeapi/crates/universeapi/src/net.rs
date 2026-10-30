//! Shared HTTP layer: one connection pool, retries with jittered backoff,
//! anti-bot detection, and the range-probe used to verify resolved links.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_RANGE, CONTENT_TYPE};
use reqwest::{Client, RequestBuilder, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::{Error, Result};

/// Called when a site blocks the normal client (Cloudflare challenge, TLS
/// fingerprinting). An implementation can retry with a browser-identical
/// client (`wreq`) or a hidden WebView and return the page body.
#[async_trait]
pub trait BlockedFallback: Send + Sync {
    async fn fetch_text(&self, url: &str, headers: &[(String, String)]) -> Result<String>;
}

#[derive(Clone)]
pub struct Http {
    client: Client,
    user_agent: String,
    max_retries: u32,
    fallback: Option<Arc<dyn BlockedFallback>>,
}

impl std::fmt::Debug for Http {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Http").field("user_agent", &self.user_agent).finish_non_exhaustive()
    }
}

/// Result of probing a resolved URL with a tiny ranged GET.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Probe {
    pub status: u16,
    pub content_type: Option<String>,
    pub total_size: Option<u64>,
    pub final_url: String,
    /// True when the response looks like real media bytes (not an HTML ad page).
    pub is_media: bool,
    pub accepts_ranges: bool,
}

pub type Headers<'a> = &'a [(&'a str, &'a str)];

impl Http {
    pub fn new(user_agent: &str) -> Result<Self> {
        let client = Client::builder()
            .user_agent(user_agent)
            .cookie_store(true)
            .gzip(true)
            .brotli(true)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(25))
            .pool_idle_timeout(Duration::from_secs(90))
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .map_err(|e| Error::Config(format!("http client: {e}")))?;
        Ok(Self { client, user_agent: user_agent.to_string(), max_retries: 2, fallback: None })
    }

    pub fn with_fallback(mut self, fallback: Arc<dyn BlockedFallback>) -> Self {
        self.fallback = Some(fallback);
        self
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    fn apply(rb: RequestBuilder, headers: Headers<'_>) -> RequestBuilder {
        let mut map = HeaderMap::new();
        for (k, v) in headers {
            if let (Ok(name), Ok(val)) = (HeaderName::from_bytes(k.as_bytes()), HeaderValue::from_str(v)) {
                map.insert(name, val);
            }
        }
        rb.header("Accept-Language", "en-US,en;q=0.9").headers(map)
    }

    /// Sends a request built by `build`, retrying transient failures.
    async fn execute(&self, url: &str, build: impl Fn() -> RequestBuilder) -> Result<Response> {
        let mut attempt = 0;
        loop {
            let result = build().send().await;
            let err = match result {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() || status.is_redirection() || status == StatusCode::PARTIAL_CONTENT {
                        return Ok(resp);
                    }
                    if matches!(status.as_u16(), 403 | 503) && is_challenge(resp.headers()) {
                        return Err(Error::Blocked { url: url.to_string() });
                    }
                    Error::Http { status: status.as_u16(), url: url.to_string() }
                }
                Err(e) => Error::Network { url: url.to_string(), source: e },
            };
            if attempt >= self.max_retries || !err.is_transient() {
                return Err(err);
            }
            attempt += 1;
            let backoff = 300u64 * 2u64.pow(attempt) + fastrand::u64(0..250);
            tracing::debug!(%url, attempt, backoff, "retrying after transient error: {err}");
            tokio::time::sleep(Duration::from_millis(backoff)).await;
        }
    }

    pub async fn get(&self, url: &str, headers: Headers<'_>) -> Result<Response> {
        self.execute(url, || Self::apply(self.client.get(url), headers)).await
    }

    pub async fn get_text(&self, url: &str, headers: Headers<'_>) -> Result<String> {
        match self.get(url, headers).await {
            Ok(resp) => {
                let text = read_text(url, resp).await?;
                if looks_like_challenge(&text) {
                    return self.blocked(url, headers).await;
                }
                Ok(text)
            }
            Err(Error::Blocked { .. }) => self.blocked(url, headers).await,
            // Plain 403/429/503 without challenge markers is often a datacenter-IP
            // or fingerprint block too; the browser-like fallback may get through.
            Err(e @ Error::Http { status: 403 | 429 | 503, .. }) if self.fallback.is_some() => {
                self.blocked(url, headers).await.map_err(|_| e)
            }
            Err(e) => Err(e),
        }
    }

    async fn blocked(&self, url: &str, headers: Headers<'_>) -> Result<String> {
        match &self.fallback {
            Some(fb) => {
                tracing::info!(%url, "normal client blocked; using fallback transport");
                let owned: Vec<(String, String)> = headers.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
                fb.fetch_text(url, &owned).await
            }
            None => Err(Error::Blocked { url: url.to_string() }),
        }
    }

    pub async fn get_json<T: DeserializeOwned>(&self, url: &str, headers: Headers<'_>) -> Result<T> {
        let text = self.get_text(url, headers).await?;
        serde_json::from_str(&text).map_err(|e| Error::parse("json", format!("{url}: {e}")))
    }

    pub async fn get_json_query<T: DeserializeOwned, Q: Serialize + ?Sized>(
        &self,
        url: &str,
        query: &Q,
        headers: Headers<'_>,
    ) -> Result<T> {
        let resp = self.execute(url, || Self::apply(self.client.get(url).query(query), headers)).await?;
        let text = read_text(url, resp).await?;
        serde_json::from_str(&text).map_err(|e| Error::parse("json", format!("{url}: {e}")))
    }

    pub async fn post_form_text(&self, url: &str, form: &[(&str, &str)], headers: Headers<'_>) -> Result<String> {
        let resp = self.execute(url, || Self::apply(self.client.post(url).form(form), headers)).await?;
        read_text(url, resp).await
    }

    pub async fn post_json<T: DeserializeOwned, B: Serialize + ?Sized>(
        &self,
        url: &str,
        body: &B,
        headers: Headers<'_>,
    ) -> Result<T> {
        let resp = self.execute(url, || Self::apply(self.client.post(url).json(body), headers)).await?;
        let text = read_text(url, resp).await?;
        serde_json::from_str(&text).map_err(|e| Error::parse("json", format!("{url}: {e}")))
    }

    /// Ranged GET of the first KiB to check a URL really serves media.
    pub async fn probe(&self, url: &str, headers: &[(String, String)]) -> Result<Probe> {
        let mut rb = self.client.get(url).header("Range", "bytes=0-1023").timeout(Duration::from_secs(15));
        for (k, v) in headers {
            rb = rb.header(k.as_str(), v.as_str());
        }
        let mut resp = rb.send().await.map_err(|e| Error::Network { url: url.to_string(), source: e })?;
        let status = resp.status().as_u16();
        let final_url = resp.url().to_string();
        let content_type = resp
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_ascii_lowercase());
        let total_size = resp
            .headers()
            .get(CONTENT_RANGE)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.rsplit('/').next())
            .and_then(|s| s.trim().parse::<u64>().ok())
            .or_else(|| if status == 200 { resp.content_length() } else { None });
        let accepts_ranges = status == 206;
        let first = resp.chunk().await.ok().flatten().unwrap_or_default();
        let head = String::from_utf8_lossy(&first[..first.len().min(512)]).to_ascii_lowercase();

        let ok_status = status == 200 || status == 206;
        let is_html = content_type.as_deref().is_some_and(|c| c.contains("text/html"))
            || head.trim_start().starts_with("<!doctype html")
            || head.trim_start().starts_with("<html");
        let is_playlist = head.trim_start().starts_with("#extm3u") || head.contains("<mpd");
        let is_media = ok_status && (is_playlist || !is_html);
        Ok(Probe { status, content_type, total_size, final_url, is_media, accepts_ranges })
    }
}

async fn read_text(url: &str, resp: Response) -> Result<String> {
    resp.text().await.map_err(|e| Error::Network { url: url.to_string(), source: e })
}

fn is_challenge(headers: &HeaderMap) -> bool {
    headers.get("cf-mitigated").is_some()
        || headers
            .get("server")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|s| s.eq_ignore_ascii_case("cloudflare") && headers.get("cf-chl-bypass").is_some())
}

/// Detects interstitial anti-bot pages served with a 200 status.
pub fn looks_like_challenge(body: &str) -> bool {
    let head = prefix(body, 8192);
    head.contains("<title>Just a moment")
        || head.contains("cf_chl_opt")
        || head.contains("cf-browser-verification")
        || head.contains("<title>Attention Required! | Cloudflare")
}

/// UTF-8-safe prefix of at most `max` bytes.
pub fn prefix(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn challenge_detection() {
        assert!(looks_like_challenge("<html><head><title>Just a moment...</title>"));
        assert!(!looks_like_challenge("<html><head><title>Frieren</title>"));
        assert!(!looks_like_challenge(r#"<script src="/cdn-cgi/challenge-platform/scripts/jsd/main.js"></script>"#));
    }

    #[test]
    fn prefix_is_char_safe() {
        assert_eq!(prefix("héllo", 2), "h");
        assert_eq!(prefix("abc", 10), "abc");
    }
}
