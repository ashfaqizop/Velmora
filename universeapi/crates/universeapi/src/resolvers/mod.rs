//! Hoster resolvers: turn a hosting-page link (HubCloud, HubCDN, hdstream4u...)
//! into final playable / downloadable [`Source`]s. Shared by every provider.
//!
//! Hosts are matched by their *name label* (`hubcloud`, `hubcdn`...) rather
//! than the full domain, so TLD rotations (`hubcloud.ist` → `hubcloud.foo`)
//! keep working without a code change.

pub mod embeds;
mod hub;
mod players;
pub mod zoko;

pub use hub::greenmotors_decode;

use std::collections::BTreeMap;
use std::time::Duration;

use async_trait::async_trait;
use futures::future::BoxFuture;
use futures::FutureExt;

use crate::error::{Error, Result};
use crate::model::{HostedLink, ProviderId, Source};
use crate::net::Http;
use crate::util::host_of;

/// Browser-backed solver for hosts that require executing JavaScript and
/// clicking through a timer (GreenMotors). Velmora implements this with a
/// hidden Tauri WebView; the engine itself never ships a browser.
#[async_trait]
pub trait WebSolver: Send + Sync {
    /// Load `url`, wait `wait`, click `click_selector`, and return the first
    /// navigated/opened URL whose host contains `capture_host`.
    async fn click_through(
        &self,
        url: &str,
        click_selector: &str,
        wait: Duration,
        capture_host: &str,
    ) -> Result<Option<String>>;
}

pub struct ResolveCtx<'a> {
    pub http: &'a Http,
    pub provider: ProviderId,
    pub web: Option<&'a dyn WebSolver>,
    /// Remote `host_aliases` from the registry (rebranded hosters).
    pub aliases: &'a BTreeMap<String, String>,
}

/// Which resolver family handles a host.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostKind {
    HubCdn,
    HubDrive,
    HubCloud,
    HbLinks,
    GreenMotors,
    HbPlay,
    HdStream4u,
    PixelDrain,
    HubStream,
    ZokoAnime,
}

impl HostKind {
    pub fn detect(url: &str) -> Option<Self> {
        Self::detect_with(url, &BTreeMap::new())
    }

    /// Like [`HostKind::detect`], honouring remote host aliases.
    pub fn detect_with(url: &str, aliases: &BTreeMap<String, String>) -> Option<Self> {
        let host = host_of(url);
        let label = host.rsplit('.').nth(1).unwrap_or(&host);
        let name = aliases.get(label).map(String::as_str).unwrap_or(label);
        Some(match name {
            n if n.starts_with("hubcdn") => Self::HubCdn,
            n if n.starts_with("hubdrive") => Self::HubDrive,
            n if n.starts_with("hubcloud") && host.starts_with("gpdl.") => return None,
            n if n.starts_with("hubcloud") => Self::HubCloud,
            n if n.starts_with("hblinks") => Self::HbLinks,
            n if n.starts_with("greenmotors") => Self::GreenMotors,
            "hbplay" => Self::HbPlay,
            n if n.starts_with("hdstream4u") => Self::HdStream4u,
            "pixeldrain" => Self::PixelDrain,
            n if n.starts_with("hubstream") => Self::HubStream,
            n if n.starts_with("zokoanime") => Self::ZokoAnime,
            _ if host.ends_with("hbplay.pages.dev") => Self::HbPlay,
            _ => return None,
        })
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::HubCdn => "HubCDN",
            Self::HubDrive => "HubDrive",
            Self::HubCloud => "HubCloud",
            Self::HbLinks => "HBLinks",
            Self::GreenMotors => "GreenMotors",
            Self::HbPlay => "HBPlay",
            Self::HdStream4u => "HDStream4U",
            Self::PixelDrain => "PixelDrain",
            Self::HubStream => "HubStream",
            Self::ZokoAnime => "ZokoAnime",
        }
    }
}

/// Resolve one hosted link into final sources. Aggregator hosts (HBLinks,
/// GreenMotors) recurse one level into the hosts they list.
pub fn resolve<'a>(ctx: &'a ResolveCtx<'a>, link: &'a HostedLink) -> BoxFuture<'a, Result<Vec<Source>>> {
    resolve_depth(ctx, link, 0)
}

fn resolve_depth<'a>(ctx: &'a ResolveCtx<'a>, link: &'a HostedLink, depth: u8) -> BoxFuture<'a, Result<Vec<Source>>> {
    async move {
        let kind = HostKind::detect_with(&link.url, ctx.aliases).ok_or_else(|| Error::Unsupported(format!("no resolver for {}", link.url)))?;
        let mut out = match kind {
            HostKind::HubCdn => hub::hubcdn(ctx, link).await?,
            HostKind::HubDrive => hub::hubdrive(ctx, link).await?,
            HostKind::HubCloud => hub::hubcloud(ctx, link).await?,
            HostKind::HbPlay => hub::hbplay(ctx, link)?,
            HostKind::HdStream4u => players::hdstream4u(ctx, link).await?,
            HostKind::PixelDrain => players::pixeldrain(ctx, link)?,
            HostKind::HbLinks | HostKind::GreenMotors => {
                if depth > 1 {
                    return Ok(Vec::new());
                }
                let children = if kind == HostKind::HbLinks {
                    hub::hblinks(ctx, link).await?
                } else {
                    hub::greenmotors(ctx, link).await?
                };
                let mut all = Vec::new();
                for child in &children {
                    match resolve_depth(ctx, child, depth + 1).await {
                        Ok(s) => all.extend(s),
                        Err(e) => tracing::debug!(url = %child.url, "child resolve failed: {e}"),
                    }
                }
                all
            }
            HostKind::HubStream => {
                return Err(Error::Unsupported("hubstream.art player is encrypted (not yet supported)".into()))
            }
            HostKind::ZokoAnime => {
                return Err(Error::Unsupported("zokoanime links are resolved by the anime provider".into()))
            }
        };
        for s in &mut out {
            s.tags.merge_missing(&link.tags);
            s.is_pack |= link.is_pack;
            s.retag();
        }
        Ok(out)
    }
    .boxed()
}

/// Expiry of an AWS-SigV4 / R2 presigned URL (`X-Amz-Date` + `X-Amz-Expires`).
pub fn presigned_expiry(url: &str) -> Option<i64> {
    let parsed = url::Url::parse(url).ok()?;
    let mut date = None;
    let mut expires = None;
    for (k, v) in parsed.query_pairs() {
        match k.as_ref() {
            "X-Amz-Date" => date = Some(v.into_owned()),
            "X-Amz-Expires" => expires = v.parse::<i64>().ok(),
            _ => {}
        }
    }
    let d = date?;
    if d.len() != 16 {
        return None;
    }
    let num = |r: std::ops::Range<usize>| d.get(r).and_then(|s| s.parse::<u32>().ok());
    let day = chrono::NaiveDate::from_ymd_opt(num(0..4)? as i32, num(4..6)?, num(6..8)?)?;
    let t = day.and_hms_opt(num(9..11)?, num(11..13)?, num(13..15)?)?;
    Some(t.and_utc().timestamp() + expires?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_detection_survives_tld_rotation() {
        assert_eq!(HostKind::detect("https://hubcloud.ist/drive/abc"), Some(HostKind::HubCloud));
        assert_eq!(HostKind::detect("https://hubcloud.foo/drive/abc"), Some(HostKind::HubCloud));
        assert_eq!(HostKind::detect("https://hubcdn.wiki/file/x"), Some(HostKind::HubCdn));
        assert_eq!(HostKind::detect("https://hubcdn.club/file/x"), Some(HostKind::HubCdn));
        assert_eq!(HostKind::detect("https://hubdrive.bar/file/1"), Some(HostKind::HubDrive));
        assert_eq!(HostKind::detect("https://greenmotors.cc/?id=x"), Some(HostKind::GreenMotors));
        assert_eq!(HostKind::detect("https://hbplay.pages.dev/?u=x"), Some(HostKind::HbPlay));
        assert_eq!(HostKind::detect("https://hdstream4u.com/file/x"), Some(HostKind::HdStream4u));
        assert_eq!(HostKind::detect("https://gpdl.hubcloud.ist/?id=x"), None, "10Gbps link is an ad redirect");
        assert_eq!(HostKind::detect("https://example.com"), None);
    }

    #[test]
    fn remote_aliases_cover_rebrands() {
        let aliases = BTreeMap::from([("hubkloud".to_string(), "hubcloud".to_string())]);
        assert_eq!(HostKind::detect("https://hubkloud.xyz/drive/a"), None);
        assert_eq!(HostKind::detect_with("https://hubkloud.xyz/drive/a", &aliases), Some(HostKind::HubCloud));
    }

    #[test]
    fn r2_expiry() {
        let url = "https://x.r2.cloudflarestorage.com/hub/a.mkv?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Date=20261030T120000Z&X-Amz-Expires=18000&X-Amz-Signature=ab";
        let base = chrono::NaiveDate::from_ymd_opt(2026, 10, 30).unwrap().and_hms_opt(12, 0, 0).unwrap().and_utc().timestamp();
        assert_eq!(presigned_expiry(url), Some(base + 18000));
        assert_eq!(presigned_expiry("https://x/y.mkv"), None);
    }
}
