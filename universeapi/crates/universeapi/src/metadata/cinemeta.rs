//! Cinemeta (Stremio's public IMDb-backed catalog). Needs no API key — the
//! interim metadata source until a TMDB key is placed in the registry.

use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;

use crate::cache::ttl;
use crate::error::{Error, Result};
use crate::model::*;
use crate::providers::{Ctx, Provider};
use crate::util::year_from;

pub struct Cinemeta {
    ctx: Arc<Ctx>,
}

#[derive(Debug, Deserialize)]
struct Catalog {
    #[serde(default)]
    metas: Vec<Meta>,
}

#[derive(Debug, Deserialize)]
struct MetaEnvelope {
    meta: Option<Meta>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Meta {
    id: String,
    imdb_id: Option<String>,
    moviedb_id: Option<serde_json::Value>,
    #[serde(rename = "type")]
    kind: String,
    name: String,
    poster: Option<String>,
    background: Option<String>,
    description: Option<String>,
    #[serde(rename = "releaseInfo")]
    release_info: Option<String>,
    #[serde(rename = "imdbRating")]
    imdb_rating: Option<String>,
    genres: Vec<String>,
    cast: Vec<String>,
    director: Vec<String>,
    runtime: Option<String>,
    trailers: Vec<Trailer>,
    videos: Vec<Video>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Trailer {
    source: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Video {
    season: Option<u32>,
    episode: Option<u32>,
    name: Option<String>,
    title: Option<String>,
    released: Option<String>,
    thumbnail: Option<String>,
    overview: Option<String>,
    description: Option<String>,
}

impl Cinemeta {
    pub fn new(ctx: Arc<Ctx>) -> Self {
        Self { ctx }
    }

    fn base(&self) -> String {
        self.ctx.registry().cinemeta_url.trim_end_matches('/').to_string()
    }

    async fn catalog(&self, kind: &str, extra: Option<&str>) -> Result<Vec<TitleSummary>> {
        let url = match extra {
            Some(e) => format!("{}/catalog/{kind}/top/{e}.json", self.base()),
            None => format!("{}/catalog/{kind}/top.json", self.base()),
        };
        let c: Catalog = self.ctx.http.get_json(&url, &[]).await?;
        Ok(c.metas.into_iter().filter(|m| !m.name.is_empty()).map(to_summary).collect())
    }
}

fn kind_of(m: &Meta) -> MediaKind {
    if m.kind == "series" {
        MediaKind::Series
    } else {
        MediaKind::Movie
    }
}

fn to_summary(m: Meta) -> TitleSummary {
    let kind = kind_of(&m);
    let imdb = m.imdb_id.clone().or_else(|| m.id.starts_with("tt").then(|| m.id.clone()));
    let id = format!("{}/{}", if kind == MediaKind::Series { "series" } else { "movie" }, imdb.as_deref().unwrap_or(&m.id));
    let mut s = TitleSummary::new(ProviderId::Cinemeta, id, m.name, kind);
    s.year = m.release_info.as_deref().and_then(year_from);
    s.poster = m.poster;
    s.backdrop = m.background;
    s.overview = m.description;
    s.rating = m.imdb_rating.and_then(|r| r.parse().ok());
    s.ids.imdb = imdb.as_deref().and_then(ExternalIds::normalize_imdb);
    s.ids.tmdb = match m.moviedb_id {
        Some(serde_json::Value::Number(n)) => n.as_u64(),
        Some(serde_json::Value::String(x)) => x.parse().ok(),
        _ => None,
    };
    s
}

#[async_trait]
impl Provider for Cinemeta {
    fn id(&self) -> ProviderId {
        ProviderId::Cinemeta
    }

    fn kinds(&self) -> &'static [MediaKind] {
        &[MediaKind::Movie, MediaKind::Series]
    }

    async fn search(&self, query: &str) -> Result<Vec<TitleSummary>> {
        let extra = format!("search={}", urlencode(query));
        let (movies, series) = tokio::join!(self.catalog("movie", Some(&extra)), self.catalog("series", Some(&extra)));
        let mut out = movies.unwrap_or_default();
        out.extend(series.unwrap_or_default());
        Ok(out)
    }

    async fn detail(&self, id: &str) -> Result<TitleDetail> {
        let key = format!("cinemeta:detail:{id}");
        if let Some(d) = self.ctx.cache.get::<TitleDetail>(&key).await {
            return Ok(d);
        }
        let (kind, imdb) = id.split_once('/').ok_or_else(|| Error::NotFound(format!("bad cinemeta id {id}")))?;
        let url = format!("{}/meta/{kind}/{imdb}.json", self.base());
        let env: MetaEnvelope = self.ctx.http.get_json(&url, &[]).await?;
        let m = env.meta.ok_or_else(|| Error::NotFound(format!("cinemeta {id}")))?;

        let genres = m.genres.clone();
        let cast = m.cast.clone();
        let directors = m.director.clone();
        let runtime = m.runtime.as_deref().and_then(|r| r.split_whitespace().next()).and_then(|n| n.parse().ok());
        let trailer = m.trailers.first().filter(|t| !t.source.is_empty()).map(|t| format!("https://www.youtube.com/embed/{}", t.source));
        let mut seasons: Vec<Season> = Vec::new();
        for v in &m.videos {
            let (Some(sn), Some(ep)) = (v.season, v.episode) else { continue };
            if sn == 0 {
                continue; // specials
            }
            let episode = Episode {
                season: sn,
                number: ep,
                title: v.name.clone().or_else(|| v.title.clone()),
                overview: v.overview.clone().or_else(|| v.description.clone()),
                air_date: v.released.as_deref().map(|r| r.chars().take(10).collect()),
                thumbnail: v.thumbnail.clone(),
                key: None,
            };
            match seasons.iter_mut().find(|s| s.number == sn) {
                Some(s) => s.episodes.push(episode),
                None => seasons.push(Season { number: sn, name: None, episodes: vec![episode] }),
            }
        }
        seasons.sort_by_key(|s| s.number);
        for s in &mut seasons {
            s.episodes.sort_by_key(|e| e.number);
        }

        let mut d = TitleDetail::from_summary(to_summary(m));
        d.genres = genres;
        d.cast = cast;
        d.directors = directors;
        d.runtime_minutes = runtime;
        d.trailer = trailer;
        d.seasons = seasons;
        self.ctx.cache.put(&key, &d, ttl::DETAIL).await;
        Ok(d)
    }

    async fn sources(&self, _: &TitleDetail, _: Target, _: AudioMode) -> Result<Vec<Source>> {
        Ok(Vec::new())
    }

    async fn rails(&self) -> Result<Vec<Rail>> {
        let (movies, series) = tokio::join!(self.catalog("movie", None), self.catalog("series", None));
        let mut rails = Vec::new();
        if let Ok(items) = movies {
            rails.push(Rail { id: "trending_movies".into(), title: "Trending Movies".into(), items });
        }
        if let Ok(items) = series {
            rails.push(Rail { id: "trending_series".into(), title: "Trending Series".into(), items });
        }
        if rails.is_empty() {
            return Err(Error::provider(ProviderId::Cinemeta, "catalogs unavailable"));
        }
        Ok(rails)
    }
}

pub(crate) fn urlencode(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect::<String>().replace('+', "%20")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_mapping() {
        let m: Meta = serde_json::from_str(
            r#"{"id":"tt0903747","imdb_id":"tt0903747","moviedb_id":1396,"type":"series","name":"Breaking Bad","releaseInfo":"2008-2013","imdbRating":"9.5","poster":"p.jpg"}"#,
        )
        .unwrap();
        let s = to_summary(m);
        assert_eq!(s.origin.id, "series/tt0903747");
        assert_eq!(s.kind, MediaKind::Series);
        assert_eq!(s.year, Some(2008));
        assert_eq!(s.ids.tmdb, Some(1396));
        assert_eq!(s.rating, Some(9.5));
    }

    #[test]
    fn encodes_queries() {
        assert_eq!(urlencode("dune part two"), "dune%20part%20two");
    }
}
