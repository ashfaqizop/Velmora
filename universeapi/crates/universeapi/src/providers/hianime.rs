//! hianime.at — ported from ani-cli. Search, episode list and server list
//! are scraped; the ZokoAnime embed is then decoded by
//! [`crate::resolvers::zoko`]. The MAL id is recovered from the embed URL so
//! hianime titles merge with AniList metadata.

use std::sync::{Arc, LazyLock};

use async_trait::async_trait;
use regex::Regex;
use scraper::{Html, Selector};
use serde::Deserialize;

use super::{Ctx, Provider};
use crate::cache::ttl;
use crate::error::{Error, Result};
use crate::model::*;
use crate::resolvers::zoko;
use crate::util::{b64_decode_str, collapse_ws, text_of, unescape_html, year_from};

pub struct HiAnime {
    ctx: Arc<Ctx>,
}

macro_rules! sel {
    ($name:ident, $css:expr) => {
        static $name: LazyLock<Selector> = LazyLock::new(|| Selector::parse($css).expect("selector"));
    };
}

sel!(ITEM, ".flw-item");
sel!(NAME_A, ".film-name a");
sel!(POSTER, "img.film-poster-img");
sel!(SUB, ".tick-sub");
sel!(DUB, ".tick-dub");
sel!(EPS, ".tick-eps");
sel!(D_NAME, "h2.film-name");
sel!(D_POSTER, ".film-poster img");
sel!(D_DESC, ".film-description .text");
sel!(D_INFO, ".item.item-title");
sel!(D_HEAD, ".item-head");
sel!(D_VAL, ".name");
sel!(D_GENRES, ".item-list a[href*='/genre/']");
sel!(EP_ITEM, "a.ep-item");
sel!(SERVER, ".server-item");

static MAL_IN_EMBED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"/mal/(\d+)/").expect("regex"));

#[derive(Debug, Deserialize)]
struct HtmlEnvelope {
    #[serde(default)]
    html: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Server {
    pub mode: String,
    pub name: String,
    pub embed: String,
}

impl HiAnime {
    pub fn new(ctx: Arc<Ctx>) -> Self {
        Self { ctx }
    }

    fn base(&self) -> Result<String> {
        self.ctx.base(ProviderId::HiAnime)
    }

    async fn servers(&self, episode_key: &str) -> Result<Vec<Server>> {
        let url = format!("{}/api/theme/episode/servers?episodeId={episode_key}", self.base()?);
        let env: HtmlEnvelope = self.ctx.http.get_json(&url, &[("X-Requested-With", "XMLHttpRequest")]).await?;
        Ok(parse_servers(&env.html))
    }
}

#[async_trait]
impl Provider for HiAnime {
    fn id(&self) -> ProviderId {
        ProviderId::HiAnime
    }

    fn kinds(&self) -> &'static [MediaKind] {
        &[MediaKind::Anime]
    }

    async fn search(&self, query: &str) -> Result<Vec<TitleSummary>> {
        let base = self.base()?;
        let url = url::Url::parse_with_params(&format!("{base}/search"), &[("keyword", query)])
            .map_err(|e| Error::Config(e.to_string()))?;
        let html = self.ctx.http.get_text(url.as_str(), &[]).await?;
        Ok(parse_search(&html))
    }

    async fn detail(&self, id: &str) -> Result<TitleDetail> {
        let key = format!("hianime:detail:{id}");
        if let Some(d) = self.ctx.cache.get::<TitleDetail>(&key).await {
            return Ok(d);
        }
        let base = self.base()?;
        let html = self.ctx.http.get_text(&format!("{base}/{id}"), &[]).await?;
        let mut d = parse_detail(&html, id)?;

        let numeric = id.rsplit('-').next().unwrap_or(id);
        let env: HtmlEnvelope = self
            .ctx
            .http
            .get_json(&format!("{base}/api/theme/episode/list/{numeric}"), &[("X-Requested-With", "XMLHttpRequest")])
            .await?;
        let episodes = parse_episodes(&env.html, id);

        // MAL id lives in the ZokoAnime embed URL of any episode.
        if let Some(first) = episodes.first().and_then(|e| e.key.clone()) {
            if let Ok(servers) = self.servers(&first).await {
                d.summary.ids.mal = servers
                    .iter()
                    .find_map(|s| MAL_IN_EMBED.captures(&s.embed).and_then(|c| c[1].parse().ok()));
            }
        }
        if !episodes.is_empty() {
            d.seasons = vec![Season { number: 1, name: None, episodes }];
        }
        self.ctx.cache.put(&key, &d, ttl::DETAIL).await;
        Ok(d)
    }

    async fn sources(&self, detail: &TitleDetail, target: Target, mode: AudioMode) -> Result<Vec<Source>> {
        let number = match target {
            Target::Episode { episode, .. } => episode,
            Target::Movie => 1,
            Target::Season { .. } => return Ok(Vec::new()),
        };
        let ep = detail
            .seasons
            .iter()
            .flat_map(|s| &s.episodes)
            .find(|e| e.number == number)
            .and_then(|e| e.key.clone())
            .ok_or_else(|| Error::NotFound(format!("episode {number} not listed")))?;

        let mode_str = match mode {
            AudioMode::Sub => "sub",
            AudioMode::Dub => "dub",
        };
        let servers = self.servers(&ep).await?;
        let mut tried = false;
        let mut out = Vec::new();
        for s in servers.iter().filter(|s| s.mode == mode_str && s.name.eq_ignore_ascii_case("ZokoAnime")) {
            tried = true;
            let label = format!("Episode {number} · {}", mode_str.to_uppercase());
            match zoko::resolve_embed(&self.ctx.http, ProviderId::HiAnime, &s.embed, &label).await {
                Ok(src) => out.extend(src),
                Err(e) => tracing::debug!("hianime zoko embed failed: {e}"),
            }
        }
        if out.is_empty() {
            return Err(if tried {
                Error::provider(ProviderId::HiAnime, "embed had no playable stream")
            } else {
                Error::NotFound(format!("no {mode_str} server for episode {number}"))
            });
        }
        Ok(out)
    }
}

pub fn parse_search(html: &str) -> Vec<TitleSummary> {
    // The "top 10" sidebar repeats result markup; cut it off first (ani-cli does the same).
    let main = html.split("id=\"main-sidebar\"").next().unwrap_or(html);
    let doc = Html::parse_document(main);
    let mut out = Vec::new();
    for item in doc.select(&ITEM) {
        let Some(a) = item.select(&NAME_A).next() else { continue };
        let Some(href) = a.value().attr("href") else { continue };
        let slug = href.trim_end_matches('/').rsplit('/').next().unwrap_or_default().split('?').next().unwrap_or_default();
        if slug.is_empty() {
            continue;
        }
        let name = a.value().attr("title").map(unescape_html).unwrap_or_else(|| text_of(a));
        let mut s = TitleSummary::new(ProviderId::HiAnime, slug, name, MediaKind::Anime);
        s.original_name = a.value().attr("data-jname").map(unescape_html);
        s.poster = item.select(&POSTER).next().and_then(|i| i.value().attr("src").or(i.value().attr("data-src"))).map(str::to_string);
        if let Some(n) = item.select(&SUB).next().map(text_of).filter(|t| !t.is_empty()) {
            s.badges.push(format!("SUB {n}"));
        }
        if let Some(n) = item.select(&DUB).next().map(text_of).filter(|t| !t.is_empty()) {
            s.badges.push(format!("DUB {n}"));
        }
        if s.badges.is_empty() {
            if let Some(n) = item.select(&EPS).next().map(text_of).filter(|t| !t.is_empty()) {
                s.badges.push(format!("{n} eps"));
            }
        }
        out.push(s);
    }
    out
}

pub fn parse_detail(html: &str, slug: &str) -> Result<TitleDetail> {
    let doc = Html::parse_document(html);
    let h2 = doc.select(&D_NAME).next().ok_or_else(|| Error::parse("hianime detail", "no film-name"))?;
    let mut s = TitleSummary::new(ProviderId::HiAnime, slug, text_of(h2), MediaKind::Anime);
    s.original_name = h2.value().attr("data-jname").map(unescape_html);
    s.poster = doc
        .select(&D_POSTER)
        .next()
        .and_then(|i| i.value().attr("src").or(i.value().attr("data-src")))
        .map(str::to_string);
    s.overview = doc.select(&D_DESC).next().map(text_of).filter(|t| !t.is_empty());

    let mut runtime = None;
    for info in doc.select(&D_INFO) {
        let head = info.select(&D_HEAD).next().map(text_of).unwrap_or_default().to_ascii_lowercase();
        let val = info.select(&D_VAL).next().map(text_of).unwrap_or_default();
        if head.starts_with("aired") || head.starts_with("premiered") {
            s.year = s.year.or_else(|| year_from(&val));
        } else if head.starts_with("mal score") {
            s.rating = val.parse().ok();
        } else if head.starts_with("duration") {
            runtime = val.trim_end_matches('m').trim().parse().ok();
        }
    }
    let mut d = TitleDetail::from_summary(s);
    d.genres = doc.select(&D_GENRES).map(text_of).collect();
    d.runtime_minutes = runtime;
    Ok(d)
}

pub fn parse_episodes(html: &str, slug: &str) -> Vec<Episode> {
    let doc = Html::parse_fragment(html);
    let mut out: Vec<Episode> = Vec::new();
    for a in doc.select(&EP_ITEM) {
        let v = a.value();
        // ids from the v5.0 provider share the same shape; require our slug (ani-cli does too)
        if let Some(href) = v.attr("href") {
            if !href.contains(&format!("/watch/{slug}")) {
                continue;
            }
        }
        let (Some(num), Some(id)) = (v.attr("data-number").and_then(|n| n.parse().ok()), v.attr("data-id")) else {
            continue;
        };
        if out.iter().any(|e| e.number == num) {
            continue;
        }
        out.push(Episode {
            season: 1,
            number: num,
            title: v.attr("title").map(unescape_html).map(|t| collapse_ws(&t)).filter(|t| !t.is_empty()),
            overview: None,
            air_date: None,
            thumbnail: None,
            key: Some(id.to_string()),
        });
    }
    out.sort_by_key(|e| e.number);
    out
}

pub fn parse_servers(html: &str) -> Vec<Server> {
    let doc = Html::parse_fragment(html);
    doc.select(&SERVER)
        .filter_map(|el| {
            let v = el.value();
            let embed = b64_decode_str(v.attr("data-hash")?)?;
            Some(Server {
                mode: v.attr("data-type").unwrap_or("sub").to_ascii_lowercase(),
                name: v.attr("data-server-name").unwrap_or_default().to_string(),
                embed,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_items_skip_sidebar() {
        let html = r#"<div class="flw-item"><div class="film-poster"><div class="tick-item tick-sub">28</div><div class="tick-item tick-dub">28</div>
            <img src="https://cdn.x/p.jpg" class="film-poster-img"></div><div class="film-detail"><h3 class="film-name">
            <a href="https://hianime.at/frieren-beyond-journeys-end-481" title="Frieren: Beyond Journey&#039;s End" data-jname="Sousou no Frieren">x</a></h3></div></div>
            <div id="main-sidebar"><div class="flw-item"><h3 class="film-name"><a href="https://hianime.at/one-piece-100" title="One Piece">x</a></h3></div></div>"#;
        let r = parse_search(html);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].origin.id, "frieren-beyond-journeys-end-481");
        assert_eq!(r[0].name, "Frieren: Beyond Journey's End");
        assert_eq!(r[0].original_name.as_deref(), Some("Sousou no Frieren"));
        assert_eq!(r[0].badges, vec!["SUB 28", "DUB 28"]);
    }

    #[test]
    fn episodes_and_servers() {
        let eps = r#"<a title="The Journey&#039;s End" class="ssl-item ep-item" data-number="1" data-id="9227" href="https://hianime.at/watch/frieren-beyond-journeys-end-481?ep=9227"></a>
                     <a class="ssl-item ep-item" data-number="2" data-id="9228" href="https://hianime.at/watch/frieren-beyond-journeys-end-481?ep=9228"></a>
                     <a class="ssl-item ep-item" data-number="1" data-id="1" href="https://hianime.at/watch/other-5?ep=1"></a>"#;
        let e = parse_episodes(eps, "frieren-beyond-journeys-end-481");
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].title.as_deref(), Some("The Journey's End"));
        assert_eq!(e[1].key.as_deref(), Some("9228"));

        let srv = r#"<div class="item server-item" data-type="sub" data-server-name="ZokoAnime" data-hash="aHR0cHM6Ly96b2tvYW5pbWUudmlkZW8vc3RyZWFtL21hbC81Mjk5MS8xL3N1Yg=="></div>
                     <div class="item server-item" data-type="dub" data-server-name="HD-1" data-hash="aHR0cHM6Ly9tZWdhcGxheS5idXp6L3N0cmVhbS9zLTIvMTA3MjU3L2R1Yj9zPXRjZG4="></div>"#;
        let s = parse_servers(srv);
        assert_eq!(s[0].embed, "https://zokoanime.video/stream/mal/52991/1/sub");
        assert_eq!(MAL_IN_EMBED.captures(&s[0].embed).unwrap()[1].parse::<u64>().unwrap(), 52991);
        assert_eq!(s[1].mode, "dub");
    }
}
