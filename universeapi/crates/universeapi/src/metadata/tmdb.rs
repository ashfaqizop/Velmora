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
    #[serde(default, deserialize_with = "crate::util::null_default")]
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
    #[serde(default, deserialize_with = "crate::util::null_default")]
    genre_ids: Vec<u32>,
    // detail-only
    #[serde(default, deserialize_with = "crate::util::null_default")]
    genres: Vec<Genre>,
    runtime: Option<u32>,
    #[serde(default, deserialize_with = "crate::util::null_default")]
    episode_run_time: Vec<u32>,
    external_ids: Option<ExtIds>,
    imdb_id: Option<String>,
    #[serde(default, deserialize_with = "crate::util::null_default")]
    seasons: Vec<SeasonRef>,
    videos: Option<Videos>,
    credits: Option<Credits>,
    // extras for the title page
    tagline: Option<String>,
    status: Option<String>,
    homepage: Option<String>,
    budget: Option<u64>,
    revenue: Option<u64>,
    vote_count: Option<u32>,
    number_of_seasons: Option<u32>,
    number_of_episodes: Option<u32>,
    #[serde(default, deserialize_with = "crate::util::null_default")]
    spoken_languages: Vec<Named>,
    #[serde(default, deserialize_with = "crate::util::null_default")]
    production_countries: Vec<Named>,
    #[serde(default, deserialize_with = "crate::util::null_default")]
    production_companies: Vec<Named>,
    #[serde(default, deserialize_with = "crate::util::null_default")]
    networks: Vec<Named>,
    #[serde(default, deserialize_with = "crate::util::null_default")]
    created_by: Vec<Credit>,
    images: Option<Images>,
    recommendations: Option<Box<Paged>>,
    keywords: Option<Keywords>,
    release_dates: Option<Ratings<CountryReleases>>,
    content_ratings: Option<Ratings<CountryRating>>,
}

/// Anything TMDB lists by name (languages, countries, companies, keywords).
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Named {
    name: String,
    english_name: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Images {
    #[serde(default, deserialize_with = "crate::util::null_default")]
    backdrops: Vec<ImagePath>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ImagePath {
    file_path: String,
}

/// Movies list keywords under `keywords`, series under `results`.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Keywords {
    #[serde(default, deserialize_with = "crate::util::null_default")]
    keywords: Vec<Named>,
    #[serde(default, deserialize_with = "crate::util::null_default")]
    results: Vec<Named>,
}

#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
struct Ratings<T> {
    #[serde(default = "Vec::new", deserialize_with = "crate::util::null_default")]
    results: Vec<T>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct CountryReleases {
    iso_3166_1: String,
    #[serde(default, deserialize_with = "crate::util::null_default")]
    release_dates: Vec<ReleaseCert>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ReleaseCert {
    certification: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct CountryRating {
    iso_3166_1: String,
    rating: String,
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
    #[serde(default, deserialize_with = "crate::util::null_default")]
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
    #[serde(default, deserialize_with = "crate::util::null_default")]
    cast: Vec<Credit>,
    #[serde(default, deserialize_with = "crate::util::null_default")]
    crew: Vec<Credit>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Credit {
    name: String,
    job: Option<String>,
    character: Option<String>,
    profile_path: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct SeasonDetail {
    #[serde(default, deserialize_with = "crate::util::null_default")]
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

/// Crew worth showing on the title page, in this order.
const KEY_JOBS: &[&str] = &["Director", "Screenplay", "Writer", "Novel", "Original Music Composer", "Director of Photography", "Producer"];

fn person(c: &Credit, role: Option<String>) -> Person {
    Person { name: c.name.clone(), role: role.filter(|r| !r.is_empty()), photo: img(&c.profile_path, "w185") }
}

/// US age rating first, then any country that has one.
fn certification(it: &Item, kind: &str) -> Option<String> {
    if kind == "tv" {
        let r = it.content_ratings.as_ref()?;
        let pick = |us: bool| r.results.iter().find(|x| (!us || x.iso_3166_1 == "US") && !x.rating.is_empty()).map(|x| x.rating.clone());
        pick(true).or_else(|| pick(false))
    } else {
        let r = it.release_dates.as_ref()?;
        let pick = |us: bool| {
            r.results
                .iter()
                .filter(|x| !us || x.iso_3166_1 == "US")
                .flat_map(|x| x.release_dates.iter())
                .find(|d| !d.certification.is_empty())
                .map(|d| d.certification.clone())
        };
        pick(true).or_else(|| pick(false))
    }
}

fn names(v: &[Named]) -> Vec<String> {
    v.iter().map(|n| n.name.clone()).filter(|n| !n.is_empty()).collect()
}

fn extras(it: &Item, kind: &str) -> TitleExtra {
    let mut crew: Vec<Person> = it.created_by.iter().map(|c| person(c, Some("Creator".into()))).collect();
    if let Some(c) = &it.credits {
        for job in KEY_JOBS {
            for p in c.crew.iter().filter(|p| p.job.as_deref() == Some(*job)) {
                if !crew.iter().any(|x| x.name == p.name) {
                    crew.push(person(p, p.job.clone()));
                }
            }
        }
    }
    let keywords = it.keywords.as_ref().map(|k| if k.keywords.is_empty() { names(&k.results) } else { names(&k.keywords) }).unwrap_or_default();
    TitleExtra {
        tagline: it.tagline.clone().filter(|t| !t.is_empty()),
        release_date: it.release_date.clone().or_else(|| it.first_air_date.clone()).filter(|d| !d.is_empty()),
        status: it.status.clone().filter(|s| !s.is_empty()),
        certification: certification(it, kind),
        original_language: it.original_language.clone(),
        languages: it
            .spoken_languages
            .iter()
            .map(|l| l.english_name.clone().filter(|n| !n.is_empty()).unwrap_or_else(|| l.name.clone()))
            .filter(|n| !n.is_empty())
            .collect(),
        countries: names(&it.production_countries),
        studios: if it.networks.is_empty() { names(&it.production_companies) } else { names(&it.networks) },
        vote_count: it.vote_count.filter(|v| *v > 0),
        rating_source: it.vote_average.filter(|v| *v > 0.0).map(|_| "TMDB".into()),
        budget: it.budget.filter(|b| *b > 0),
        revenue: it.revenue.filter(|r| *r > 0),
        season_count: it.number_of_seasons,
        episode_count: it.number_of_episodes,
        homepage: it.homepage.clone().filter(|h| !h.is_empty()),
        cast: it
            .credits
            .as_ref()
            .map(|c| c.cast.iter().take(20).map(|p| person(p, p.character.clone())).collect())
            .unwrap_or_default(),
        crew,
        keywords: keywords.into_iter().take(12).collect(),
        recommendations: it
            .recommendations
            .as_ref()
            .map(|r| r.results.iter().filter(|i| !i.adult).take(18).map(|i| to_summary(i, kind)).collect())
            .unwrap_or_default(),
        ..Default::default()
    }
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
        let key = format!("tmdb:detail2:{id}");
        if let Some(d) = self.ctx.cache.get::<TitleDetail>(&key).await {
            return Ok(d);
        }
        let (kind, tid) = id.split_once('/').ok_or_else(|| Error::NotFound(format!("bad tmdb id {id}")))?;
        let append = if kind == "tv" {
            "external_ids,videos,credits,images,recommendations,keywords,content_ratings"
        } else {
            "external_ids,videos,credits,images,recommendations,keywords,release_dates"
        };
        // Backdrops in English or with no language (most have no text on them).
        let it: Item = self
            .get(&format!("/{kind}/{tid}"), &[("append_to_response", append), ("include_image_language", "en,null")])
            .await?;
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
        d.screenshots = it
            .images
            .as_ref()
            .map(|i| i.backdrops.iter().take(16).filter_map(|b| img(&Some(b.file_path.clone()), "w1280")).collect())
            .unwrap_or_default();
        d.extra = extras(&it, kind);
        self.ctx.cache.put(&key, &d, ttl::DETAIL).await;
        Ok(d)
    }

    async fn sources(&self, _: &TitleDetail, _: Target, _: AudioMode) -> Result<Vec<Source>> {
        Ok(Vec::new())
    }

    async fn rails(&self) -> Result<Vec<Rail>> {
        let (movies, tv, kdrama) = tokio::join!(
            self.get::<Paged>("/trending/movie/week", &[]),
            self.get::<Paged>("/trending/tv/week", &[]),
            self.get::<Paged>("/discover/tv", &[("with_original_language", "ko"), ("sort_by", "popularity.desc")])
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
        if let Ok(p) = kdrama {
            rails.push(Rail {
                id: "kdrama".into(),
                title: "Popular K-Dramas".into(),
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
    fn extras_from_detail() {
        let json = r#"{
            "id": 7, "title": "Example", "tagline": "Hold on.", "status": "Released",
            "release_date": "2024-05-01", "vote_average": 7.4, "vote_count": 1200, "budget": 0, "revenue": 5000,
            "spoken_languages": [{"english_name": "English", "name": "English"}],
            "production_countries": [{"name": "United States of America"}],
            "production_companies": [{"name": "Studio A"}],
            "credits": {
                "cast": [{"name": "Ada", "character": "Lead", "profile_path": "/a.jpg"}],
                "crew": [{"name": "Rae", "job": "Producer"}, {"name": "Vale", "job": "Director"}]
            },
            "keywords": {"keywords": [{"name": "heist"}]},
            "release_dates": {"results": [
                {"iso_3166_1": "GB", "release_dates": [{"certification": "15"}]},
                {"iso_3166_1": "US", "release_dates": [{"certification": ""}, {"certification": "R"}]}
            ]},
            "recommendations": {"results": [{"id": 8, "title": "Other", "media_type": "movie"}]}
        }"#;
        let it: Item = serde_json::from_str(json).unwrap();
        let x = extras(&it, "movie");
        assert_eq!(x.tagline.as_deref(), Some("Hold on."));
        assert_eq!(x.certification.as_deref(), Some("R"), "US rating wins");
        assert_eq!(x.budget, None, "zero budget is unknown");
        assert_eq!(x.revenue, Some(5000));
        assert_eq!(x.cast[0].role.as_deref(), Some("Lead"));
        assert_eq!(x.cast[0].photo.as_deref(), Some("https://image.tmdb.org/t/p/w185/a.jpg"));
        assert_eq!(x.crew.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), ["Vale", "Rae"], "director first");
        assert_eq!(x.keywords, ["heist"]);
        assert_eq!(x.recommendations[0].origin.id, "movie/8");
        assert_eq!(x.rating_source.as_deref(), Some("TMDB"));
    }

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
