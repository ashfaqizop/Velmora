//! MovieBox (aoneroom "wefeed" mobile API) — ported from MovieBox-TUI.
//!
//! Every request is signed like the Android app:
//! * `x-client-token` = `"{ts},{md5(reversed ts)}"`
//! * `x-tr-signature` = `"{ts}|2|{base64(HMAC-MD5(secret, canonical))}"`
//!
//! A visitor JWT (persisted in the engine cache until it expires) authorises
//! calls. Hosts come from the registry and fail over on 403/429/5xx.
//!
//! Streams come from three endpoints merged together: detail "detectors"
//! (direct MP4), `play-info/v2` (DASH behind CloudFront signed cookies) and
//! `resource/v2` (direct files, with `resource-position/v2` recovering files
//! that the listing replaced with a placeholder clip). Language dubs are
//! separate subjects; the ones matching the user's audio preferences are
//! fetched too.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use base64::Engine as _;
use hmac::{Hmac, Mac};
use md5::{Digest, Md5};
use serde_json::{json, Value};

use super::{Ctx, Provider};
use crate::cache::ttl;
use crate::error::{Error, Result};
use crate::model::*;
use crate::release::ReleaseTags;
use crate::util::year_from;

const SECRET: &[u8] = b"\xef\xa8\x91\x97\x4e\xec\xd3\x14\x8d\xf6\x3a\xa6\x11\x60\x2d\xef\xd1\x01\x25\x9b\xa5\x21\x02\x2c\x57\xae\x05\x66\xbd\x8e";
const SIGNATURE_BODY_MAX_BYTES: usize = 102_400;
const RETRY_STATUS: &[u16] = &[403, 406, 407, 429, 500, 502, 503, 504];
/// DASH segments are served only with this Referer.
pub const STREAM_REFERER: &str = "https://sportslive.wine";
const SESSION_KEY: &str = "moviebox:session";
pub const MEMBERS_ONLY: &str = "MovieBox keeps this series for paid members; other providers serve it";
const API: &str = "/wefeed-mobile-bff";

// ── signing ──────────────────────────────────────────────────────────────

fn md5_hex(bytes: &[u8]) -> String {
    format!("{:x}", Md5::digest(bytes))
}

pub fn client_token(ts_ms: u64) -> String {
    let ts = ts_ms.to_string();
    let reversed: String = ts.chars().rev().collect();
    format!("{ts},{}", md5_hex(reversed.as_bytes()))
}

fn sorted_query(url: &url::Url) -> String {
    let mut params: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (k, v) in url.query_pairs() {
        params.entry(k.into_owned()).or_default().push(v.into_owned());
    }
    params
        .into_iter()
        .flat_map(|(k, vs)| vs.into_iter().map(move |v| format!("{k}={v}")))
        .collect::<Vec<_>>()
        .join("&")
}

pub fn canonical_string(method: &str, url: &str, body: Option<&str>, ts_ms: u64) -> String {
    let canonical_url = match url::Url::parse(url) {
        Ok(u) => {
            let q = sorted_query(&u);
            if q.is_empty() {
                u.path().to_string()
            } else {
                format!("{}?{q}", u.path())
            }
        }
        Err(_) => url.to_string(),
    };
    let (hash, len) = match body {
        Some(b) => {
            let bytes = b.as_bytes();
            (md5_hex(&bytes[..bytes.len().min(SIGNATURE_BODY_MAX_BYTES)]), bytes.len().to_string())
        }
        None => (String::new(), String::new()),
    };
    format!(
        "{}\napplication/json\napplication/json\n{len}\n{ts_ms}\n{hash}\n{canonical_url}",
        method.to_uppercase()
    )
}

pub fn tr_signature(method: &str, url: &str, body: Option<&str>, ts_ms: u64) -> String {
    let canonical = canonical_string(method, url, body, ts_ms);
    let mut mac = <Hmac<Md5> as Mac>::new_from_slice(SECRET).expect("hmac accepts any key length");
    mac.update(canonical.as_bytes());
    let sig = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
    format!("{ts_ms}|2|{sig}")
}

/// Randomised Android client identity, fixed for the process lifetime.
struct Identity {
    user_agent: String,
    client_info: String,
    ip: String,
}

fn pick<T: Copy>(items: &[T]) -> T {
    items[fastrand::usize(..items.len())]
}

fn random_hex(len: usize) -> String {
    (0..len).map(|_| char::from_digit(fastrand::u32(..16), 16).unwrap_or('0')).collect()
}

impl Identity {
    fn generate() -> Self {
        let android = pick(&[
            ("10", "QP1A.191005.007.A3"),
            ("11", "RP1A.200720.011"),
            ("12", "S1B.220414.015"),
            ("13", "TQ2A.230405.003"),
        ]);
        let device = pick(&["23078RKD5C", "2201117TY", "2201117TG", "22101316G", "21121210G", "M2012K11AG"]);
        let version_code = pick(&[50020117u32, 50020118, 50020119, 50020120, 50020121]);
        let net = pick(&["NETWORK_WIFI", "NETWORK_MOBILE"]);
        let tz = pick(&["Asia/Kolkata", "Asia/Tokyo", "America/New_York", "Europe/London"]);
        let gaid = format!("{}-{}-{}-{}-{}", random_hex(8), random_hex(4), random_hex(4), random_hex(4), random_hex(12));
        let user_agent = format!(
            "com.community.oneroom/{version_code} (Linux; U; Android {}; en_US; {device}; Build/{}; Cronet/135.0.7012.3)",
            android.0, android.1
        );
        let client_info = json!({
            "package_name": "com.community.oneroom",
            "version_name": "4.0.01.0813.03",
            "version_code": version_code,
            "os": "android",
            "os_version": android.0,
            "install_ch": "ps",
            "device_id": random_hex(32),
            "install_store": "ps",
            "gaid": gaid,
            "brand": "Redmi",
            "model": device,
            "system_language": "en",
            "net": net,
            "region": "US",
            "timezone": tz,
            "sp_code": "40401",
            "X-Play-Mode": "2",
        })
        .to_string();
        let prefix = pick(&["103.241", "49.36", "117.195", "106.198", "122.162", "157.32", "182.70", "27.60"]);
        let ip = format!("{prefix}.{}.{}", fastrand::u8(1..254), fastrand::u8(1..254));
        Self { user_agent, client_info, ip }
    }
}

// ── session ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Session {
    token: String,
    expires_at: Option<i64>,
}

impl Session {
    fn from_token(token: String) -> Self {
        let expires_at = token
            .split('.')
            .nth(1)
            .and_then(crate::util::b64_decode)
            .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
            .and_then(|claims| claims.get("exp").and_then(Value::as_i64));
        Self { token, expires_at }
    }

    fn is_valid(&self) -> bool {
        match self.expires_at {
            Some(exp) => chrono::Utc::now().timestamp() + 60 < exp,
            None => true,
        }
    }
}

// ── provider ─────────────────────────────────────────────────────────────

pub struct MovieBox {
    ctx: Arc<Ctx>,
    identity: OnceLock<Identity>,
    session: std::sync::RwLock<Option<Session>>,
    login: tokio::sync::Mutex<()>,
    active_host: AtomicUsize,
}

impl MovieBox {
    pub fn new(ctx: Arc<Ctx>) -> Self {
        Self {
            ctx,
            identity: OnceLock::new(),
            session: std::sync::RwLock::new(None),
            login: tokio::sync::Mutex::new(()),
            active_host: AtomicUsize::new(0),
        }
    }

    fn identity(&self) -> &Identity {
        self.identity.get_or_init(Identity::generate)
    }

    fn cached_session(&self) -> Option<Session> {
        self.session.read().ok().and_then(|s| s.clone()).filter(Session::is_valid)
    }

    fn store_session(&self, s: Session) {
        if let Ok(mut w) = self.session.write() {
            *w = Some(s);
        }
    }

    async fn ensure_session(&self) -> Result<String> {
        if let Some(s) = self.cached_session() {
            return Ok(s.token);
        }
        let _guard = self.login.lock().await;
        if let Some(s) = self.cached_session() {
            return Ok(s.token);
        }
        if let Some(s) = self.ctx.cache.get::<Session>(SESSION_KEY).await.filter(Session::is_valid) {
            self.store_session(s.clone());
            return Ok(s.token);
        }
        let v = self.request_hosts("POST", &format!("{API}/user-api/visitor-login"), Some("{}".into()), None).await?;
        let token = v
            .get("token")
            .and_then(Value::as_str)
            .filter(|t| !t.trim().is_empty())
            .ok_or_else(|| Error::provider(ProviderId::MovieBox, "visitor login returned no token"))?;
        let s = Session::from_token(token.to_string());
        self.persist(&s).await;
        Ok(s.token)
    }

    async fn persist(&self, s: &Session) {
        self.store_session(s.clone());
        let ttl = s
            .expires_at
            .map(|exp| (exp - chrono::Utc::now().timestamp() - 60).max(60) as u64)
            .unwrap_or(6 * 3600);
        self.ctx.cache.put(SESSION_KEY, s, std::time::Duration::from_secs(ttl)).await;
    }

    async fn invalidate_session(&self) {
        if let Ok(mut w) = self.session.write() {
            *w = None;
        }
        self.ctx.cache.invalidate(SESSION_KEY).await;
    }

    pub async fn get(&self, path: &str) -> Result<Value> {
        self.request("GET", path, None).await
    }

    pub async fn post(&self, path: &str, body: &Value) -> Result<Value> {
        self.request("POST", path, Some(body.to_string())).await
    }

    async fn request(&self, method: &str, path: &str, body: Option<String>) -> Result<Value> {
        let token = self.ensure_session().await?;
        match self.request_hosts(method, path, body.clone(), Some(&token)).await {
            Err(Error::Http { status: 401 | 403, .. }) | Err(Error::Provider { .. }) => {
                self.invalidate_session().await;
                let fresh = self.ensure_session().await?;
                self.request_hosts(method, path, body, Some(&fresh)).await
            }
            other => other,
        }
    }

    async fn request_hosts(&self, method: &str, path: &str, body: Option<String>, token: Option<&str>) -> Result<Value> {
        let reg = self.ctx.registry();
        let hosts = reg.mirrors(ProviderId::MovieBox);
        if hosts.is_empty() {
            return Err(Error::Config("no MovieBox hosts configured".into()));
        }
        let id = self.identity();
        let start = self.active_host.load(Ordering::Relaxed);
        let mut last: Option<Error> = None;

        for i in 0..hosts.len() {
            let idx = (start + i) % hosts.len();
            let url = format!("{}{path}", hosts[idx].trim_end_matches('/'));
            let ts = chrono::Utc::now().timestamp_millis() as u64;
            let mut rb = match method {
                "POST" => self.ctx.http.client().post(&url),
                _ => self.ctx.http.client().get(&url),
            }
            .header("User-Agent", &id.user_agent)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .header("x-client-token", client_token(ts))
            .header("x-tr-signature", tr_signature(method, &url, body.as_deref(), ts))
            .header("x-client-info", &id.client_info)
            .header("x-client-status", "0")
            .header("x-forwarded-for", &id.ip);
            if let Some(t) = token {
                rb = rb.header("Authorization", format!("Bearer {t}"));
            }
            if let Some(b) = &body {
                rb = rb.body(b.clone());
            }

            let resp = match rb.send().await {
                Ok(r) => r,
                Err(e) => {
                    tracing::debug!(host = idx, "moviebox request failed: {e}");
                    self.active_host.store((idx + 1) % hosts.len(), Ordering::Relaxed);
                    last = Some(Error::Network { url, source: e });
                    continue;
                }
            };
            self.absorb_x_user(resp.headers()).await;
            let status = resp.status().as_u16();
            if RETRY_STATUS.contains(&status) {
                self.active_host.store((idx + 1) % hosts.len(), Ordering::Relaxed);
                if status == 429 {
                    tokio::time::sleep(std::time::Duration::from_millis(400)).await;
                }
                last = Some(Error::Http { status, url });
                continue;
            }
            if !(200..300).contains(&status) {
                return Err(Error::Http { status, url });
            }
            self.active_host.store(idx, Ordering::Relaxed);
            let text = resp.text().await.map_err(|e| Error::Network { url: url.clone(), source: e })?;
            let v: Value = serde_json::from_str(&text).map_err(|e| Error::parse("moviebox json", format!("{url}: {e}")))?;
            return Ok(v.get("data").cloned().unwrap_or(v));
        }
        Err(last.unwrap_or_else(|| Error::provider(ProviderId::MovieBox, "all hosts exhausted")))
    }

    /// The API may hand out a refreshed token in an `x-user` header.
    async fn absorb_x_user(&self, headers: &reqwest::header::HeaderMap) {
        let token = headers
            .get("x-user")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| serde_json::from_str::<Value>(s).ok())
            .and_then(|j| j.get("token").and_then(Value::as_str).map(str::to_string))
            .filter(|t| !t.is_empty());
        if let Some(t) = token {
            self.persist(&Session::from_token(t)).await;
        }
    }

    /// Raw subject JSON (+ seasons for series), cached.
    async fn subject(&self, id: &str) -> Result<Value> {
        let key = format!("moviebox:subject:{id}");
        if let Some(v) = self.ctx.cache.get::<Value>(&key).await {
            return Ok(v);
        }
        let mut v = self.get(&format!("{API}/subject-api/get?subjectId={id}")).await?;
        if subject_type(&v) == 2 {
            if let Ok(seasons) = self.get(&format!("{API}/subject-api/season-info?subjectId={id}")).await {
                if let Value::Object(map) = &mut v {
                    map.insert("seasons".into(), seasons);
                }
            }
        }
        self.ctx.cache.put(&key, &v, ttl::DETAIL).await;
        Ok(v)
    }

    /// All sources for one subject id (the main title or one of its dubs).
    async fn subject_sources(&self, id: &str, season: u32, episode: u32, lang: Option<&str>) -> Result<Vec<Source>> {
        let ua = self.identity().user_agent.clone();
        let play_path = if season == 0 {
            format!("{API}/subject-api/play-info/v2?subjectId={id}")
        } else {
            format!("{API}/subject-api/play-info/v2?subjectId={id}&se={season}&ep={episode}")
        };
        let res_path = format!(
            "{API}/subject-api/resource/v2?subjectId={id}&page=1&perPage=20&all=0&startPosition=1&endPosition=1&pagerMode=2&resolution=0&se={season}&epFrom={episode}&epTo={episode}&isVip=true"
        );
        let subject_path = format!("{API}/subject-api/get?subjectId={id}");
        let (subject, play, resources) = tokio::join!(self.get(&subject_path), self.get(&play_path), self.get(&res_path));

        let mut out: Vec<Source> = Vec::new();
        let mut first_resource: Option<String> = None;

        if let Ok(v) = &subject {
            for item in detector_items(v) {
                if let Some(s) = resource_source(&item, season, episode, &ua) {
                    first_resource = first_resource.or_else(|| item.get("resourceId").and_then(id_str));
                    push_unique(&mut out, s);
                }
            }
        }
        let mut placeholders = 0usize;
        match &play {
            Ok(v) => {
                let (dash, ph) = play_info_sources(v, &ua);
                placeholders += ph;
                for s in dash {
                    push_unique(&mut out, s);
                }
            }
            Err(e) => tracing::debug!("moviebox play-info failed: {e}"),
        }
        if let Ok(v) = &resources {
            for item in v.get("list").and_then(Value::as_array).cloned().unwrap_or_default() {
                let (se, ep) = (num(item.get("se")).unwrap_or(0) as u32, num(item.get("ep")).unwrap_or(0) as u32);
                if season != 0 && (se != season || ep != episode) {
                    continue;
                }
                let rid = item.get("resourceId").and_then(id_str);
                first_resource = first_resource.or_else(|| rid.clone());
                if let Some(s) = resource_source(&item, season, episode, &ua) {
                    push_unique(&mut out, s);
                    continue;
                }
                // Placeholder clip → ask resource-position for the genuine file.
                let link = item.get("resourceLink").or_else(|| item.get("url")).and_then(Value::as_str).unwrap_or("");
                if let (Some(rid), true) = (rid, is_placeholder(link)) {
                    placeholders += 1;
                    let q = url::form_urlencoded::Serializer::new(String::new())
                        .append_pair("subjectId", id)
                        .append_pair("resourceId", &rid)
                        .append_pair("failUrl", link)
                        .append_pair("failCode", "404")
                        .append_pair("resourceNum", "0")
                        .append_pair("isVip", "true")
                        .finish();
                    if let Ok(pos) = self.get(&format!("{API}/subject-api/resource-position/v2?{q}")).await {
                        for cand in pos.get("list").and_then(Value::as_array).cloned().unwrap_or_default() {
                            if cand.get("resourceId").and_then(id_str).as_deref() == Some(rid.as_str()) {
                                if let Some(s) = resource_source(&cand, season, episode, &ua) {
                                    push_unique(&mut out, s);
                                }
                            }
                        }
                    }
                }
            }
        }

        if out.is_empty() {
            // Placeholder clips with nothing recoverable means MovieBox keeps the
            // title for paid members, even when the response carries no vipInfo
            // (seen from US datacenter IPs). Report it as unavailable, not broken.
            return Err(if placeholders > 0 {
                Error::NotFound(MEMBERS_ONLY.into())
            } else {
                Error::NotFound("MovieBox has no streams for this title".into())
            });
        }

        // External captions (same for every file of this subject/episode).
        if let Some(rid) = first_resource {
            if let Ok(c) = self.get(&format!("{API}/subject-api/get-ext-captions?subjectId={id}&resourceId={rid}")).await {
                let subs = captions(&c);
                for s in &mut out {
                    if s.subtitles.is_empty() {
                        s.subtitles.clone_from(&subs);
                    }
                }
            }
        }
        if let Some(l) = lang {
            for s in &mut out {
                if !s.tags.languages.iter().any(|x| x == l) {
                    s.tags.languages.insert(0, l.to_string());
                }
            }
        }
        Ok(out)
    }
}

// ── JSON mapping ─────────────────────────────────────────────────────────

/// Add unless the same file (ignoring signed query strings) is already listed.
fn push_unique(out: &mut Vec<Source>, s: Source) {
    let base = s.url.split('?').next().unwrap_or(&s.url);
    if !out.iter().any(|x| x.url.split('?').next() == Some(base)) {
        out.push(s);
    }
}

fn id_str(v: &Value) -> Option<String> {
    v.as_i64().map(|n| n.to_string()).or_else(|| v.as_u64().map(|n| n.to_string())).or_else(|| v.as_str().map(str::to_string))
}

fn num(v: Option<&Value>) -> Option<u64> {
    let v = v?;
    v.as_u64().or_else(|| v.as_i64().map(|n| n.max(0) as u64)).or_else(|| v.as_str().and_then(|s| s.trim().parse().ok()))
}

fn subject_of(v: &Value) -> &Value {
    v.get("subject").unwrap_or(v)
}

fn subject_type(v: &Value) -> i64 {
    let s = subject_of(v);
    s.get("subjectType").or_else(|| s.get("stype")).and_then(Value::as_i64).unwrap_or(1)
}

fn strings(v: Option<&Value>) -> Vec<String> {
    match v {
        Some(Value::Array(a)) => a.iter().filter_map(|x| x.as_str().map(|s| s.trim().to_string())).filter(|s| !s.is_empty()).collect(),
        Some(Value::String(s)) => s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect(),
        _ => Vec::new(),
    }
}

fn kind_for(s: &Value) -> MediaKind {
    let series = subject_type(s) == 2;
    let country = s.get("countryName").and_then(Value::as_str).unwrap_or("").to_ascii_lowercase();
    let genres = strings(s.get("genre").or_else(|| s.get("genres"))).join(",").to_ascii_lowercase();
    if genres.contains("anim") && country.contains("japan") {
        MediaKind::Anime
    } else if series && ["korea", "china", "taiwan", "thailand", "japan"].iter().any(|c| country.contains(c)) {
        MediaKind::AsianDrama
    } else if series {
        MediaKind::Series
    } else {
        MediaKind::Movie
    }
}

/// Strip MovieBox decorations: `"[Hindi] Dune (2021) [CAM]"` → `"Dune (2021)"` → name + year.
pub fn clean_title(raw: &str) -> String {
    let mut t = raw.trim();
    while let Some(rest) = t.strip_prefix('[').and_then(|r| r.split_once(']')).map(|(_, r)| r.trim()) {
        if rest.is_empty() {
            break;
        }
        t = rest;
    }
    if let Some(pos) = t.find('[').filter(|p| *p > 0) {
        t = t[..pos].trim();
    }
    if let Some(pos) = t.find('(').filter(|p| *p > 0) {
        t = t[..pos].trim();
    }
    if let Some(pos) = t.rfind(" - ") {
        let tag = t[pos + 3..].to_ascii_lowercase();
        if ["hindi", "tamil", "telugu", "kannada", "malayalam", "bengali", "punjabi", "dub", "sub"].iter().any(|l| tag.contains(l)) {
            t = t[..pos].trim();
        }
    }
    t.to_string()
}

pub fn to_summary(s: &Value) -> Option<TitleSummary> {
    let id = s.get("subjectId").or_else(|| s.get("id")).and_then(id_str).filter(|x| !x.is_empty())?;
    let raw = s.get("title").or_else(|| s.get("name")).and_then(Value::as_str).unwrap_or_default();
    let mut t = TitleSummary::new(ProviderId::MovieBox, id, clean_title(raw), kind_for(s));
    t.year = s
        .get("releaseDate")
        .or_else(|| s.get("year"))
        .and_then(|y| y.as_str().map(str::to_string).or_else(|| y.as_u64().map(|n| n.to_string())))
        .and_then(|y| year_from(&y))
        .or_else(|| year_from(raw));
    t.poster = s
        .get("cover")
        .and_then(|c| c.get("url"))
        .or_else(|| s.get("coverUrl"))
        .or_else(|| s.get("poster"))
        .and_then(Value::as_str)
        .map(str::to_string);
    t.rating = s
        .get("imdbRatingValue")
        .and_then(|r| r.as_f64().map(|f| f as f32).or_else(|| r.as_str().and_then(|x| x.parse().ok())))
        .filter(|r| *r > 0.0);
    t.overview = s.get("description").or_else(|| s.get("intro")).and_then(Value::as_str).map(str::to_string).filter(|d| !d.is_empty());
    let tags = ReleaseTags::parse(raw);
    if !tags.languages.is_empty() {
        t.badges.extend(tags.languages.iter().take(2).map(|l| l.to_uppercase()));
    }
    Some(t)
}

pub fn parse_search(v: &Value) -> Vec<TitleSummary> {
    let subjects = v
        .get("results")
        .and_then(Value::as_array)
        .and_then(|a| a.first())
        .and_then(|f| f.get("subjects"))
        .or_else(|| v.get("items"))
        .or_else(|| v.get("list"))
        .and_then(Value::as_array);
    subjects.map(|a| a.iter().filter_map(to_summary).collect()).unwrap_or_default()
}

pub fn parse_detail(v: &Value) -> Result<TitleDetail> {
    let s = subject_of(v);
    let summary = to_summary(s).ok_or_else(|| Error::parse("moviebox subject", "no subjectId"))?;
    let mut d = TitleDetail::from_summary(summary);
    d.genres = strings(s.get("genre").or_else(|| s.get("genres")));
    d.cast = strings(s.get("stars"));
    d.directors = strings(s.get("director"));
    d.runtime_minutes = num(s.get("duration")).filter(|n| *n > 0).map(|secs| (secs / 60) as u32);
    let seasons_v = v.get("seasons").or_else(|| s.get("seasons"));
    let arr = seasons_v.and_then(|x| x.get("seasons").or(Some(x))).and_then(Value::as_array);
    for se in arr.into_iter().flatten() {
        let number = num(se.get("se")).unwrap_or(1) as u32;
        let eps: Vec<u32> = match se.get("episodeNumbers").and_then(Value::as_array) {
            Some(a) => a.iter().filter_map(|e| e.as_u64().map(|n| n as u32)).collect(),
            None => (1..=num(se.get("maxEp")).unwrap_or(0) as u32).collect(),
        };
        d.seasons.push(Season {
            number,
            name: None,
            episodes: eps
                .into_iter()
                .map(|n| Episode { season: number, number: n, title: None, overview: None, air_date: None, thumbnail: None, key: None })
                .collect(),
        });
    }
    Ok(d)
}

/// `(subject id, language code)` for each language dub of a subject.
pub fn dubs(v: &Value) -> Vec<(String, String)> {
    subject_of(v)
        .get("dubs")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|d| {
                    let id = d.get("subjectId").or_else(|| d.get("id")).and_then(id_str)?;
                    let name = d.get("lanName").or_else(|| d.get("language")).and_then(Value::as_str).unwrap_or_default();
                    let code = ReleaseTags::parse(name).languages.into_iter().next()?;
                    Some((id, code))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// MovieBox gates some subjects (all series, as of 2026-10) behind its paid
/// membership: `vipInfo.requireMemberType > 0`. Visitor sessions then get a
/// placeholder clip for every episode, so the engine skips them and lets
/// other providers serve the title.
pub fn members_only(v: &Value) -> bool {
    let s = subject_of(v);
    v.get("vipInfo")
        .or_else(|| s.get("vipInfo"))
        .and_then(|i| i.get("requireMemberType"))
        .and_then(Value::as_i64)
        .unwrap_or(0)
        > 0
}

pub fn is_placeholder(url: &str) -> bool {
    let l = url.to_ascii_lowercase();
    l.contains("1c7de0bd3393702d9191801f15f88f8d")
        || l.contains("9a0461bc39da389663bf3dbb17091d3f")
        || l.contains("b164fbfb4347792950bdfbfb563d39d9")
        || l.contains("/notice.mp4")
        || (l.contains("macdn.aoneroom.com") && l.contains("/other/"))
}

fn detector_items(v: &Value) -> Vec<Value> {
    let s = subject_of(v);
    let title = s.get("title").and_then(Value::as_str).unwrap_or_default();
    let mut items = Vec::new();
    for det in v.get("resourceDetectors").or_else(|| s.get("resourceDetectors")).and_then(Value::as_array).into_iter().flatten() {
        if let Some(list) = det.get("resolutionList").and_then(Value::as_array).filter(|l| !l.is_empty()) {
            items.extend(list.iter().cloned());
            continue;
        }
        let Some(dl) = det.get("downloadUrl").and_then(Value::as_str) else { continue };
        let mut item = json!({ "title": title, "resourceLink": dl, "se": 0, "ep": 0 });
        for (from, to) in [("firstSize", "size"), ("codecName", "codecName"), ("resourceId", "resourceId"), ("uploadBy", "uploadBy")] {
            if let Some(x) = det.get(from) {
                item[to] = x.clone();
            }
        }
        if let Some(r) = det.get("resourceLink").and_then(Value::as_str).and_then(|l| ReleaseTags::parse(l).resolution) {
            item["resolution"] = r.into();
        }
        items.push(item);
    }
    items
}

/// A resource/detector item → direct-file source (None for placeholders / wrong episode).
fn resource_source(item: &Value, season: u32, episode: u32, ua: &str) -> Option<Source> {
    let (se, ep) = (num(item.get("se")).unwrap_or(0) as u32, num(item.get("ep")).unwrap_or(0) as u32);
    if se != season || ep != episode {
        return None;
    }
    let link = item.get("resourceLink").or_else(|| item.get("url")).and_then(Value::as_str).filter(|l| l.starts_with("http"))?;
    if is_placeholder(link) {
        return None;
    }
    let filename = item.get("fileName").or_else(|| item.get("title")).and_then(Value::as_str).unwrap_or_default();
    let res = num(item.get("resolution")).filter(|r| *r > 0);
    let codec = item.get("codecName").or_else(|| item.get("codec")).and_then(Value::as_str).unwrap_or_default();
    let label = format!("{} {codec}", res.map(|r| format!("{r}p")).unwrap_or_default()).trim().to_string();
    let host = item.get("uploadBy").or_else(|| item.get("source")).and_then(Value::as_str).filter(|h| !h.is_empty()).unwrap_or("MovieBox");
    let mut s = Source::new(ProviderId::MovieBox, format!("MovieBox · {host}"), if label.is_empty() { "MovieBox".into() } else { label }, link)
        // Direct MP4s reject the DASH referer with HTTP 429 — send only the app UA.
        .with_header("User-Agent", ua);
    if !filename.is_empty() {
        s.filename = Some(filename.to_string());
    }
    s.size_bytes = num(item.get("size")).filter(|n| *n > 0);
    if let Some(lang) = item.get("language").or_else(|| item.get("lanName")).and_then(Value::as_str) {
        s.tags.languages.extend(ReleaseTags::parse(lang).languages);
    }
    s.retag();
    if s.tags.resolution.is_none() {
        s.tags.resolution = res.map(|r| r as u16);
    }
    if s.format == StreamFormat::Unknown {
        s.format = StreamFormat::Mp4;
    }
    Some(s)
}

/// DASH manifest from CloudFront signed cookies (`urlprefix=` or the Policy statement).
pub fn dash_manifest_from_cookie(cookie: &str) -> Option<String> {
    let to_mpd = |u: &str| {
        let base = u.trim_end_matches('*').trim_end_matches('/');
        (base.starts_with("http") && base.contains("/dash/")).then(|| format!("{base}/index.mpd"))
    };
    for part in cookie.split(';').map(str::trim) {
        if let Some(idx) = part.find("urlprefix=") {
            let tok: String = part[idx + 10..].split(':').next().unwrap_or("").trim().replace('-', "+").replace('_', "/");
            if let Some(m) = crate::util::b64_decode_str(&tok).as_deref().and_then(to_mpd) {
                return Some(m);
            }
        }
        if let Some(policy) = part.strip_prefix("CloudFront-Policy=") {
            let norm: String = policy.trim().chars().map(|c| match c { '-' => '+', '_' => '=', '~' => '/', o => o }).collect();
            let resource = crate::util::b64_decode(&norm)
                .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
                .and_then(|j| j.pointer("/Statement/0/Resource").and_then(Value::as_str).map(str::to_string));
            if let Some(m) = resource.as_deref().and_then(to_mpd) {
                return Some(m);
            }
        }
    }
    None
}

/// `play-info/v2` streams → DASH sources. Returns (sources, placeholder count).
fn play_info_sources(v: &Value, ua: &str) -> (Vec<Source>, usize) {
    let mut out = Vec::new();
    let mut placeholders = 0;
    for st in v.get("streams").and_then(Value::as_array).into_iter().flatten() {
        let url = st.get("url").and_then(Value::as_str).unwrap_or_default();
        if is_placeholder(url) {
            placeholders += 1;
        }
        let cookie = st.get("signCookie").and_then(Value::as_str).unwrap_or_default();
        let playable = dash_manifest_from_cookie(cookie).or_else(|| (url.starts_with("http") && !is_placeholder(url)).then(|| url.to_string()));
        let Some(playable) = playable else { continue };
        let resolutions: Vec<u16> = st
            .get("resolutions")
            .or_else(|| v.get("displayResolutions"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .split(',')
            .filter_map(|r| r.trim().parse().ok())
            .collect();
        let top = resolutions.iter().copied().max();
        let codec = st.get("codecName").and_then(Value::as_str).unwrap_or_default();
        let fmt = st.get("format").and_then(Value::as_str).unwrap_or("DASH");
        let label = format!("{} {codec} {fmt}", top.map(|r| format!("{r}p")).unwrap_or_default()).split_whitespace().collect::<Vec<_>>().join(" ");
        let mut s = Source::new(ProviderId::MovieBox, "MovieBox CDN", label, playable).with_referer(STREAM_REFERER).with_header("User-Agent", ua);
        if !cookie.is_empty() {
            let c = cookie.split(';').map(str::trim).filter(|x| !x.is_empty()).collect::<Vec<_>>().join("; ");
            s = s.with_header("Cookie", c);
        }
        if s.format == StreamFormat::Unknown || s.url.ends_with(".mpd") {
            s.format = if s.url.contains(".m3u8") { StreamFormat::Hls } else { StreamFormat::Dash };
        }
        s.tags.resolution = top;
        s.size_bytes = num(st.get("size")).filter(|n| *n > 0);
        out.push(s);
    }
    (out, placeholders)
}

fn captions(v: &Value) -> Vec<Track> {
    let mut out: Vec<Track> = Vec::new();
    for c in v.get("extCaptions").and_then(Value::as_array).into_iter().flatten() {
        let Some(url) = c.get("url").and_then(Value::as_str).filter(|u| !u.is_empty() && !u.contains("aa348f2541d13ffe")) else { continue };
        let size = num(c.get("size")).unwrap_or(0);
        if size > 0 && size <= 50 {
            continue; // empty stub files
        }
        if out.iter().any(|t| t.url.as_deref() == Some(url)) {
            continue;
        }
        let name = c.get("lanName").and_then(Value::as_str).filter(|n| !n.trim().is_empty()).or_else(|| c.get("lan").and_then(Value::as_str)).unwrap_or("Unknown");
        let lang = c.get("lan").and_then(Value::as_str).map(|l| l.to_ascii_lowercase()).filter(|l| l.len() == 2)
            .or_else(|| ReleaseTags::parse(name).languages.into_iter().next());
        out.push(Track { lang, label: name.to_string(), url: Some(url.to_string()), default: false });
    }
    out
}

#[async_trait]
impl Provider for MovieBox {
    fn id(&self) -> ProviderId {
        ProviderId::MovieBox
    }

    fn kinds(&self) -> &'static [MediaKind] {
        &[MediaKind::Movie, MediaKind::Series, MediaKind::AsianDrama, MediaKind::Anime]
    }

    async fn search(&self, query: &str) -> Result<Vec<TitleSummary>> {
        let v = self.post(&format!("{API}/subject-api/search/v2"), &json!({ "keyword": query, "page": 1, "perPage": 20, "subjectType": 0 })).await?;
        Ok(parse_search(&v))
    }

    async fn detail(&self, id: &str) -> Result<TitleDetail> {
        parse_detail(&self.subject(id).await?)
    }

    async fn sources(&self, detail: &TitleDetail, target: Target, _mode: AudioMode) -> Result<Vec<Source>> {
        let (se, ep) = match target {
            Target::Movie => (0, 0),
            Target::Episode { season, episode } => (season, episode),
            Target::Season { .. } => return Ok(Vec::new()),
        };
        let id = detail.summary.origin.id.clone();
        let prefs = self.ctx.prefs();

        // Dubs in the user's preferred audio languages (max 2 extra subjects).
        let mut wanted_dubs: Vec<(String, String)> = Vec::new();
        if let Ok(subject) = self.subject(&id).await {
            if se > 0 && members_only(&subject) {
                return Err(Error::NotFound(MEMBERS_ONLY.into()));
            }
            for (dub_id, code) in dubs(&subject) {
                if dub_id != id && prefs.audio_languages.contains(&code) && wanted_dubs.len() < 2 {
                    wanted_dubs.push((dub_id, code));
                }
            }
        }

        let main = self.subject_sources(&id, se, ep, None);
        let extra = futures::future::join_all(wanted_dubs.iter().map(|(d, c)| self.subject_sources(d, se, ep, Some(c.as_str()))));
        let (main, extra) = tokio::join!(main, extra);

        let mut out = Vec::new();
        let mut first_err = None;
        match main {
            Ok(s) => out.extend(s),
            Err(e) => first_err = Some(e),
        }
        for s in extra.into_iter().flatten() {
            out.extend(s);
        }
        if out.is_empty() {
            return Err(first_err.unwrap_or_else(|| Error::NotFound("no MovieBox streams".into())));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signing_matches_reference_shape() {
        assert_eq!(client_token(1_700_000_000_123), format!("1700000000123,{}", md5_hex(b"3210000000071")));
        let c = canonical_string("get", "https://api6.aoneroom.com/wefeed-mobile-bff/x?b=2&a=1", None, 42);
        assert_eq!(c, "GET\napplication/json\napplication/json\n\n42\n\n/wefeed-mobile-bff/x?a=1&b=2");
        let with_body = canonical_string("POST", "https://h/p", Some("{}"), 7);
        assert!(with_body.ends_with(&format!("\n2\n7\n{}\n/p", md5_hex(b"{}"))));
        let sig = tr_signature("GET", "https://h/p", None, 42);
        assert!(sig.starts_with("42|2|") && sig.len() > 10);
    }

    #[test]
    fn jwt_expiry() {
        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(br#"{"uid":1,"exp":4102444800}"#);
        let s = Session::from_token(format!("h.{payload}.sig"));
        assert_eq!(s.expires_at, Some(4_102_444_800));
        assert!(s.is_valid());
    }

    #[test]
    fn titles() {
        assert_eq!(clean_title("[Hindi] Dune: Part Two (2024) [CAM]"), "Dune: Part Two");
        assert_eq!(clean_title("Squid Game - Hindi"), "Squid Game");
        assert_eq!(clean_title("Spider-Man - No Way Home"), "Spider-Man - No Way Home");
    }

    #[test]
    fn search_and_detail_mapping() {
        let v = json!({"results":[{"subjects":[{"subjectId":"123","title":"Squid Game","subjectType":2,"countryName":"Korea","releaseDate":"2021-09-17","cover":{"url":"p.jpg"},"imdbRatingValue":"8.0"}]}]});
        let r = parse_search(&v);
        assert_eq!(r[0].origin.id, "123");
        assert_eq!(r[0].kind, MediaKind::AsianDrama);
        assert_eq!(r[0].year, Some(2021));
        assert_eq!(r[0].rating, Some(8.0));

        let d = parse_detail(&json!({"subject":{"subjectId":5,"title":"Dark","subjectType":2,"genre":"Drama,Sci-Fi","stars":"A, B","duration":3600},
            "seasons":{"seasons":[{"se":1,"maxEp":3},{"se":2,"episodeNumbers":[1,2]}]}})).unwrap();
        assert_eq!(d.genres, vec!["Drama", "Sci-Fi"]);
        assert_eq!(d.cast, vec!["A", "B"]);
        assert_eq!(d.runtime_minutes, Some(60));
        assert_eq!(d.seasons.len(), 2);
        assert_eq!(d.seasons[0].episodes.len(), 3);
        assert_eq!(d.seasons[1].episodes[1].number, 2);
    }

    #[test]
    fn dash_from_cookie() {
        let prefix = base64::engine::general_purpose::STANDARD.encode("https://cdn.x/dash/abc/*");
        let cookie = format!("CloudFront-Key-Pair-Id=K; urlprefix={prefix}:sig");
        assert_eq!(dash_manifest_from_cookie(&cookie).as_deref(), Some("https://cdn.x/dash/abc/index.mpd"));
        assert_eq!(dash_manifest_from_cookie("CloudFront-Key-Pair-Id=K"), None);
    }

    #[test]
    fn resources_and_placeholders() {
        let ok = json!({"resourceLink":"https://cdn.x/a.mp4","resolution":1080,"codecName":"h264","se":1,"ep":2,"size":"123456","resourceId":"9"});
        let s = resource_source(&ok, 1, 2, "ua").unwrap();
        assert_eq!(s.tags.resolution, Some(1080));
        assert_eq!(s.size_bytes, Some(123456));
        assert_eq!(s.format, StreamFormat::Mp4);
        assert!(resource_source(&ok, 1, 3, "ua").is_none(), "wrong episode");
        let ph = json!({"resourceLink":"https://macdn.aoneroom.com/other/notice.mp4","se":0,"ep":0});
        assert!(resource_source(&ph, 0, 0, "ua").is_none());
        assert_eq!(dubs(&json!({"subject":{"dubs":[{"subjectId":7,"lanName":"Hindi"},{"subjectId":8,"lanName":"Klingon"}]}})), vec![("7".to_string(), "hi".to_string())]);
    }

    #[test]
    fn membership_gate() {
        assert!(members_only(&json!({"subject":{"subjectId":"1"},"vipInfo":{"requireMemberType":1,"freeEpisodeCount":2}})));
        assert!(!members_only(&json!({"subject":{"subjectId":"1"},"vipInfo":{"requireMemberType":0}})));
        assert!(!members_only(&json!({"subject":{"subjectId":"1"}})));
    }

    #[test]
    fn caption_filtering() {
        let c = json!({"extCaptions":[{"url":"https://s/en.srt","lanName":"English","lan":"en","size":2000},
            {"url":"https://s/stub.srt","lanName":"X","size":10},{"url":"https://s/en.srt","lanName":"dup"}]});
        let t = captions(&c);
        assert_eq!(t.len(), 1);
        assert_eq!(t[0].lang.as_deref(), Some("en"));
    }
}
