// SPDX-License-Identifier: GPL-3.0-only
// Ported in part from YTubic (https://github.com/NUber-dev/YTubic), Copyright George Shyshov.
//! The YouTube Music session: the user's cookies, the `SAPISIDHASH` header
//! made from them, and the cookie rotation Google expects a client to follow.
//!
//! Ported from YTubic (GPL-3.0-only, github.com/NUber-dev/YTubic),
//! `src/lib/innertube/shared.ts` and `merge_response_cookies` in its lib.rs.
//!
//! Google rotates its session-security cookies (SIDCC, __Secure-*PSIDCC,
//! LOGIN_INFO) in ordinary responses and expects the client to send the new
//! values from then on. A client that keeps replaying the old snapshot looks
//! like a stolen session and gets signed out within hours, so every
//! `Set-Cookie` from music.youtube.com is merged back here.

use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

pub const ORIGIN: &str = "https://music.youtube.com";

/// One cookie, as read from the sign-in window or a `Set-Cookie` header.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cookie {
    pub name: String,
    pub value: String,
    /// `.youtube.com` and the like. Empty means the host that set it.
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub path: String,
    /// Unix seconds; `None` for a session cookie.
    #[serde(default)]
    pub expires: Option<i64>,
    #[serde(default)]
    pub secure: bool,
    #[serde(default)]
    pub http_only: bool,
}

/// The cookies sent to music.youtube.com.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CookieJar {
    pub cookies: Vec<Cookie>,
}

/// Cookies that apply to music.youtube.com: youtube.com and its subdomains.
fn applies_to_youtube(domain: &str) -> bool {
    let d = domain.trim_start_matches('.').to_ascii_lowercase();
    d.is_empty() || d == "youtube.com" || d.ends_with(".youtube.com")
}

impl CookieJar {
    pub fn new(cookies: Vec<Cookie>) -> Self {
        let mut jar = Self { cookies: Vec::new() };
        for c in cookies {
            if applies_to_youtube(&c.domain) && !c.name.is_empty() {
                jar.upsert(c);
            }
        }
        jar
    }

    pub fn is_empty(&self) -> bool {
        self.cookies.is_empty()
    }

    fn get(&self, name: &str) -> Option<&str> {
        self.cookies.iter().find(|c| c.name == name).map(|c| c.value.as_str())
    }

    /// The jar holds a signed-in Google session.
    pub fn signed_in(&self) -> bool {
        self.sapisid().is_some()
    }

    fn sapisid(&self) -> Option<&str> {
        self.get("__Secure-3PAPISID").or_else(|| self.get("SAPISID")).filter(|v| !v.is_empty())
    }

    /// The `Cookie` header value, dropping anything already expired.
    pub fn header(&self, now: i64) -> String {
        self.cookies
            .iter()
            .filter(|c| c.expires.is_none_or(|e| e > now))
            .map(|c| format!("{}={}", c.name, c.value))
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// `SAPISIDHASH <ts>_<sha1("<ts> <SAPISID> <origin>")>`, what InnerTube
    /// wants for signed-in calls.
    pub fn authorization(&self, now: i64) -> Option<String> {
        let sapisid = self.sapisid()?;
        Some(format!("SAPISIDHASH {now}_{}", sapisidhash(now, sapisid, ORIGIN)))
    }

    fn upsert(&mut self, cookie: Cookie) -> bool {
        match self.cookies.iter_mut().find(|c| c.name == cookie.name) {
            Some(existing) if *existing == cookie => false,
            Some(existing) => {
                *existing = cookie;
                true
            }
            None => {
                self.cookies.push(cookie);
                true
            }
        }
    }

    /// Merge `Set-Cookie` header lines from a youtube.com response. Returns
    /// whether anything changed.
    pub fn merge_set_cookies<'a>(&mut self, lines: impl IntoIterator<Item = &'a str>, now: i64) -> bool {
        let mut changed = false;
        for line in lines {
            let Some(c) = parse_set_cookie(line, now) else { continue };
            if !applies_to_youtube(&c.domain) {
                continue;
            }
            // An expiry in the past is how a server deletes a cookie.
            if c.expires.is_some_and(|e| e <= now) {
                let before = self.cookies.len();
                self.cookies.retain(|x| x.name != c.name);
                changed |= self.cookies.len() != before;
                continue;
            }
            // Keep the domain and path the jar already had when the header omits them.
            let c = match self.cookies.iter().find(|x| x.name == c.name) {
                Some(old) if c.domain.is_empty() => Cookie { domain: old.domain.clone(), path: old.path.clone(), ..c },
                _ => c,
            };
            changed |= self.upsert(c);
        }
        changed
    }
}

pub fn sapisidhash(ts: i64, sapisid: &str, origin: &str) -> String {
    let digest = Sha1::digest(format!("{ts} {sapisid} {origin}").as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// Parse one `Set-Cookie` header line.
pub fn parse_set_cookie(line: &str, now: i64) -> Option<Cookie> {
    let mut parts = line.split(';');
    let (name, value) = parts.next()?.split_once('=')?;
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let mut c = Cookie {
        name: name.to_string(),
        value: value.trim().to_string(),
        domain: String::new(),
        path: String::new(),
        expires: None,
        secure: false,
        http_only: false,
    };
    for attr in parts {
        let (k, v) = attr.split_once('=').map(|(k, v)| (k.trim(), v.trim())).unwrap_or((attr.trim(), ""));
        match k.to_ascii_lowercase().as_str() {
            "domain" => c.domain = v.to_string(),
            "path" => c.path = v.to_string(),
            // Max-Age wins over Expires when both are present.
            "max-age" => {
                if let Ok(secs) = v.parse::<i64>() {
                    c.expires = Some(now + secs);
                }
            }
            "expires" if c.expires.is_none() => {
                c.expires = chrono::DateTime::parse_from_rfc2822(&v.replace('-', " ")).ok().map(|d| d.timestamp());
            }
            "secure" => c.secure = true,
            "httponly" => c.http_only = true,
            _ => {}
        }
    }
    Some(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cookie(name: &str, value: &str) -> Cookie {
        Cookie {
            name: name.into(),
            value: value.into(),
            domain: ".youtube.com".into(),
            path: "/".into(),
            expires: None,
            secure: true,
            http_only: false,
        }
    }

    #[test]
    fn sapisidhash_matches_known_value() {
        // sha1("1700000000 abc https://music.youtube.com")
        let h = sapisidhash(1_700_000_000, "abc", ORIGIN);
        assert_eq!(h.len(), 40);
        assert_eq!(h, sapisidhash(1_700_000_000, "abc", ORIGIN));
        assert_ne!(h, sapisidhash(1_700_000_001, "abc", ORIGIN));
    }

    #[test]
    fn jar_header_and_auth() {
        let jar = CookieJar::new(vec![
            cookie("SID", "s"),
            cookie("__Secure-3PAPISID", "p3"),
            Cookie { domain: ".google.com".into(), ..cookie("NID", "x") },
        ]);
        assert_eq!(jar.header(0), "SID=s; __Secure-3PAPISID=p3");
        assert!(jar.signed_in());
        assert!(jar.authorization(10).unwrap().starts_with("SAPISIDHASH 10_"));
        assert!(!CookieJar::default().signed_in());
    }

    #[test]
    fn merges_rotations_and_deletions() {
        let mut jar = CookieJar::new(vec![cookie("SIDCC", "old"), cookie("LOGIN_INFO", "l")]);
        let changed = jar.merge_set_cookies(
            [
                "SIDCC=new; expires=Fri, 01-Jan-2100 00:00:00 GMT; path=/; domain=.youtube.com; priority=high",
                "LOGIN_INFO=; Max-Age=0; path=/; domain=.youtube.com",
                "NID=1; domain=.google.com",
            ],
            1_000,
        );
        assert!(changed);
        assert_eq!(jar.get("SIDCC"), Some("new"));
        assert_eq!(jar.get("LOGIN_INFO"), None);
        assert_eq!(jar.get("NID"), None);
        assert!(!jar.merge_set_cookies(
            ["SIDCC=new; expires=Fri, 01-Jan-2100 00:00:00 GMT; path=/; domain=.youtube.com"],
            1_000
        ));
    }

    #[test]
    fn parses_max_age() {
        let c = parse_set_cookie("A=b; Max-Age=60; Secure; HttpOnly", 100).unwrap();
        assert_eq!(c.expires, Some(160));
        assert!(c.secure && c.http_only);
    }
}
