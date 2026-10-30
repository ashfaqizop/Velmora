//! TMDB — primary movie/TV metadata once a key is present in the registry
//! (`tmdb_read_token` v4 bearer, or `tmdb_api_key` v3). Without a key every
//! call returns `Error::Config` and the engine falls back to Cinemeta.

use std::sync::Arc;

use async_trait::async_trait;
use futures::future::BoxFuture;
use futures::stream::{self, StreamExt};
use futures::FutureExt;
use serde::Deserialize;

use crate::cache::ttl;
use crate::error::{Error, Result};
use crate::model::*;
use crate::providers::{Ctx, Provider};
use crate::util::year_from;

const API: &str = "https://api.themoviedb.org/3";
const IMG: &str = "https://image.tmdb.org/t/p";

pub struct Tmdb {
    ctx: Arc<Ctx>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Paged {
    results: Vec<Item>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Item {
    id: u64,
    media_type: Option<String>,
    title: Option<String>,
    name: Option<String>,
    original_title: Option<String>,
    original_name: Option<String>,
    original_language: Option<String>,
    overview: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    release_date: Option<String>,
    first_air_date: Option<String>,
    vote_average: Option<f32>,
    adult: bool,
    genre_ids: Vec<u32>,
    // detail-only
    genres: Vec<Genre>,
    runtime: Option<u32>,
    episode_run_time: Vec<u32>,
    external_ids: Option<ExtIds>,
    imdb_id: Option<String>,
    seasons: Vec<SeasonRef>,
    videos: Option<Videos>,
    credits: Option<Credits>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Genre {
    name: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ExtIds {
    imdb_id: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct SeasonRef {
    season_number: u32,
    name: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Videos {
    results: Vec<VideoRef>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct VideoRef {
    key: String,
    site: String,
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Credits {
    cast: Vec<Person>,
    crew: Vec<Person>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Person {
    name: String,
    job: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct SeasonDetail {
    episodes: Vec<Ep>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Ep {
    episode_number: u32,
    season_number: u32,
    name: Option<String>,
    overview: Option<String>,
    air_date: Option<String>,
    still_path: Option<String>,
}

/// TMDB genre id 16 = Animation; with Japanese origin we treat it as anime.
const ANIMATION: u32 = 16;

impl Tmdb {
    pub fn new(ctx: Arc<Ctx>) -> Self {
        Self { ctx }
    }

    pub fn enabled(&self) -> bool {
        self.ctx.registry().has_tmdb()
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str, query: &[(&str, &str)]) -> Result<T> {
        let reg = self.ctx.registry();
        let mut q: Vec<(&str, &str)> = query.to_vec();
        let lang = "en-US";
        q.push(("language", lang));
        let bearer;
        let mut headers: Vec<(&str, &str)> = vec![("Accept", "application/json")];
        if let Some(tok) = reg.tmdb_read_token.as_deref().filter(|t| !t.is_empty()) {
            bearer = format!("Bearer {tok}");
            headers.push(("Authorization", bearer.as_str()));
        } else if let Some(key) = reg.tmdb_api_key.as_deref().filter(|k| !k.is_empty()) {
            q.push(("api_key", key));
        } else {
            return Err(Error::Config("TMDB key not configured".into()));
        }
        let region = self.ctx.prefs().region.clone();
        if let Some(r) = region.as_deref() {
            q.push(("region", r));
        }
        self.ctx.http.get_json_query(&format!("{API}{path}"), &q, &headers).await
    }
}

fn img(path: &Option<String>, size: &str) -> Option<String> {
    path.as_ref().filter(|p| !p.is_empty()).map(|p| format!("{IMG}/{size}{p}"))
}

fn to_summary(it: &Item, fallback_type: &str) -> TitleSummary {
    let media = it.media_type.as_deref().unwrap_or(fallback_type);
    let is_tv = media == "tv";
    let anime = it.genre_ids.contains(&ANIMATION) && it.original_language.as_deref() == Some("ja");
    let kind = match (anime, is_tv) {
        (true, _) => MediaKind::Anime,
        (false, true) if matches!(it.original_language.as_deref(), Some("ko" | "zh" | "th")) => MediaKind::AsianDrama,
        (false, true) => MediaKind::Series,
        (false, false) => MediaKind::Movie,
    };
    let name = it.title.clone().or_else(|| it.name.clone()).unwrap_or_default();
    let mut s = TitleSummary::new(ProviderId::Tmdb, format!("{}/{}", if is_tv { "tv" } else { "movie" }, it.id), name, kind);
    s.original_name = it.original_title.clone().or_else(|| it.original_name.clone()).filter(|o| *o != s.name);
    s.year = it.release_date.as_deref().or(it.first_air_date.as_deref()).and_then(year_from);
    s.poster = img(&it.poster_path, "w500");
    s.backdrop = img(&it.backdrop_path, "original");
    s.overview = it.overview.clone().filter(|o| !o.is_empty());
    s.rating = it.vote_average.filter(|v| *v > 0.0);
    s.ids.tmdb = Some(it.id);
    s.ids.imdb = it
        .external_ids
        .as_ref()
        .and_then(|e| e.imdb_id.as_deref())
        .or(it.imdb_id.as_deref())
        .and_then(ExternalIds::normalize_imdb);
    if it.adult {
        s.badges.push("18+".into());
    }
    s
}

#[async_trait]
impl Provider for Tmdb {
    fn id(&self) -> ProviderId {
        ProviderId::Tmdb
    }

    fn kinds(&self) -> &'static [MediaKind] {
        &[MediaKind::Movie, MediaKind::Series, MediaKind::AsianDrama]
    }

    async fn search(&self, query: &str) -> Result<Vec<TitleSummary>> {
        let p: Paged = self.get("/search/multi", &[("query", query), ("include_adult", "false")]).await?;
        Ok(p.results.iter().filter(|i| matches!(i.media_type.as_deref(), Some("movie" | "tv"))).map(|i| to_summary(i, "movie")).collect())
    }

    async fn detail(&self, id: &str) -> Result<TitleDetail> {
        let key = format!("tmdb:detail:{id}");
        if let Some(d) = self.ctx.cache.get::<TitleDetail>(&key).await {
            return Ok(d);
        }
        let (kind, tid) = id.split_once('/').ok_or_else(|| Error::NotFound(format!("bad tmdb id {id}")))?;
        let it: Item = self.get(&format!("/{kind}/{tid}"), &[("append_to_response", "external_ids,videos,credits")]).await?;
        let summary = to_summary(&it, kind);

        let mut seasons = Vec::new();
        if kind == "tv" {
            let refs: Vec<&SeasonRef> = it.seasons.iter().filter(|s| s.season_number > 0).collect();
            // Futures built up front (see rust-lang#64552) and boxed so the stream holds no closure.
            let futs: Vec<BoxFuture<'_, (u32, Option<String>, Result<SeasonDetail>)>> = refs
                .into_iter()
                .map(|r| {
                    async move {
                        let path = format!("/tv/{tid}/season/{}", r.season_number);
                        (r.season_number, r.name.clone(), self.get(&path, &[]).await)
                    }
                    .boxed()
                })
                .collect();
            let fetched: Vec<(u32, Option<String>, Result<SeasonDetail>)> = stream::iter(futs).buffered(4).collect().await;
            for (number, name, res) in fetched {
                let episodes = res
                    .map(|sd| {
                        sd.episodes
                            .into_iter()
                            .map(|e| Episode {
                                season: e.season_number,
                                number: e.episode_number,
                                title: e.name,
                                overview: e.overview.filter(|o| !o.is_empty()),
                                air_date: e.air_date,
                                thumbnail: img(&e.still_path, "w300"),
                                key: None,
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                seasons.push(Season { number, name, episodes });
            }
        }

        let mut d = TitleDetail::from_summary(summary);
        d.genres = it.genres.iter().map(|g| g.name.clone()).collect();
        d.runtime_minutes = it.runtime.or_else(|| it.episode_run_time.first().copied());
        if let Some(c) = &it.credits {
            d.cast = c.cast.iter().take(12).map(|p| p.name.clone()).collect();
            d.directors = c.crew.iter().filter(|p| p.job.as_deref() == Some("Director")).map(|p| p.name.clone()).collect();
        }
        d.trailer = it.videos.as_ref().and_then(|v| {
            v.results
                .iter()
                .find(|x| x.site == "YouTube" && x.kind == "Trailer")
                .map(|x| format!("https://www.youtube.com/embed/{}", x.key))
        });
        d.seasons = seasons;
        self.ctx.cache.put(&key, &d, ttl::DETAIL).await;
        Ok(d)
    }

    async fn sources(&self, _: &TitleDetail, _: Target, _: AudioMode) -> Result<Vec<Source>> {
        Ok(Vec::new())
    }

    async fn rails(&self) -> Result<Vec<Rail>> {
        let (movies, tv) = tokio::join!(
            self.get::<Paged>("/trending/movie/week", &[]),
            self.get::<Paged>("/trending/tv/week", &[])
        );
        let mut rails = Vec::new();
        if let Ok(p) = movies {
            rails.push(Rail {
                id: "trending_movies".into(),
                title: "Trending Movies".into(),
                items: p.results.iter().map(|i| to_summary(i, "movie")).collect(),
            });
        }
        if let Ok(p) = tv {
            rails.push(Rail {
                id: "trending_series".into(),
                title: "Trending Series".into(),
                items: p.results.iter().map(|i| to_summary(i, "tv")).collect(),
            });
        }
        if rails.is_empty() {
            return Err(Error::provider(ProviderId::Tmdb, "trending unavailable"));
        }
        Ok(rails)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_from_origin() {
        let anime: Item = serde_json::from_str(r#"{"id":1,"media_type":"tv","name":"Frieren","genre_ids":[16],"original_language":"ja"}"#).unwrap();
        assert_eq!(to_summary(&anime, "tv").kind, MediaKind::Anime);
        let kdrama: Item = serde_json::from_str(r#"{"id":2,"media_type":"tv","name":"Squid Game","original_language":"ko"}"#).unwrap();
        assert_eq!(to_summary(&kdrama, "tv").kind, MediaKind::AsianDrama);
        let movie: Item = serde_json::from_str(r#"{"id":3,"title":"Dune","release_date":"2021-09-15","poster_path":"/p.jpg"}"#).unwrap();
        let s = to_summary(&movie, "movie");
        assert_eq!(s.origin.id, "movie/3");
        assert_eq!(s.year, Some(2021));
        assert_eq!(s.poster.as_deref(), Some("https://image.tmdb.org/t/p/w500/p.jpg"));
    }
}
