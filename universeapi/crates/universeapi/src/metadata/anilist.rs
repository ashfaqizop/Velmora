//! AniList GraphQL — anime metadata, trending and the airing schedule.
//! Supplies the MAL id used by the ZokoAnime fast path.

use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;

use super::strip_tags;
use crate::cache::ttl;
use crate::error::{Error, Result};
use crate::model::*;
use crate::providers::{Ctx, Provider};

pub struct Anilist {
    ctx: Arc<Ctx>,
}

const MEDIA_FIELDS: &str = "id idMal title { romaji english native } coverImage { extraLarge large } bannerImage \
    description(asHtml: false) averageScore seasonYear episodes format status genres isAdult \
    nextAiringEpisode { episode airingAt }";

#[derive(Debug, Deserialize)]
struct Gql<T> {
    data: Option<T>,
    #[serde(default, deserialize_with = "crate::util::null_default")]
    errors: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct PageData {
    #[serde(rename = "Page")]
    page: PageMedia,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct PageMedia {
    #[serde(default, deserialize_with = "crate::util::null_default")]
    media: Vec<Media>,
    #[serde(rename = "airingSchedules")]
    #[serde(default, deserialize_with = "crate::util::null_default")]
    airing: Vec<Airing>,
}

#[derive(Debug, Deserialize)]
struct Airing {
    episode: u32,
    media: Media,
}

#[derive(Debug, Deserialize)]
struct MediaData {
    #[serde(rename = "Media")]
    media: Media,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Media {
    id: u64,
    id_mal: Option<u64>,
    title: Titles,
    cover_image: Cover,
    banner_image: Option<String>,
    description: Option<String>,
    average_score: Option<u32>,
    season_year: Option<u16>,
    episodes: Option<u32>,
    format: Option<String>,
    #[serde(default, deserialize_with = "crate::util::null_default")]
    genres: Vec<String>,
    is_adult: bool,
    next_airing_episode: Option<NextEp>,
    #[serde(default, deserialize_with = "crate::util::null_default")]
    streaming_episodes: Vec<StreamingEp>,
    duration: Option<u32>,
    trailer: Option<TrailerRef>,
    // extras for the title page (detail query only)
    status: Option<String>,
    season: Option<String>,
    source: Option<String>,
    country_of_origin: Option<String>,
    start_date: Option<FuzzyDate>,
    studios: Option<Studios>,
    characters: Option<Characters>,
    staff: Option<Staff>,
    #[serde(default, deserialize_with = "crate::util::null_default")]
    tags: Vec<Tag>,
    recommendations: Option<Recommendations>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct FuzzyDate {
    year: Option<u32>,
    month: Option<u32>,
    day: Option<u32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Studios {
    #[serde(default, deserialize_with = "crate::util::null_default")]
    nodes: Vec<NamedNode>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct NamedNode {
    name: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct PersonNode {
    name: PersonName,
    image: Option<PersonImage>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct PersonName {
    full: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct PersonImage {
    medium: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Characters {
    #[serde(default, deserialize_with = "crate::util::null_default")]
    edges: Vec<CharacterEdge>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct CharacterEdge {
    node: PersonNode,
    #[serde(default, deserialize_with = "crate::util::null_default")]
    voice_actors: Vec<PersonNode>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Staff {
    #[serde(default, deserialize_with = "crate::util::null_default")]
    edges: Vec<StaffEdge>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct StaffEdge {
    role: Option<String>,
    node: PersonNode,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Tag {
    name: String,
    is_media_spoiler: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Recommendations {
    #[serde(default, deserialize_with = "crate::util::null_default")]
    nodes: Vec<RecommendationNode>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct RecommendationNode {
    media_recommendation: Option<Box<Media>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Titles {
    romaji: Option<String>,
    english: Option<String>,
    native: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Cover {
    extra_large: Option<String>,
    large: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct NextEp {
    episode: u32,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct StreamingEp {
    title: Option<String>,
    thumbnail: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct TrailerRef {
    id: Option<String>,
    site: Option<String>,
}

impl Anilist {
    pub fn new(ctx: Arc<Ctx>) -> Self {
        Self { ctx }
    }

    async fn query<T: serde::de::DeserializeOwned>(&self, query: &str, vars: serde_json::Value) -> Result<T> {
        let url = self.ctx.registry().anilist_url.clone();
        let body = json!({ "query": query, "variables": vars });
        let resp: Gql<T> = self.ctx.http.post_json(&url, &body, &[("Accept", "application/json")]).await?;
        match resp.data {
            Some(d) => Ok(d),
            None => Err(Error::provider(ProviderId::Anilist, format!("graphql: {:?}", resp.errors.first()))),
        }
    }

    /// `decl` must declare exactly the variables `args` uses (GraphQL rejects unused ones).
    async fn page(&self, decl: &str, args: &str, vars: serde_json::Value) -> Result<Vec<TitleSummary>> {
        let q = format!("query({decl}){{ Page(perPage: 24) {{ media({args}, type: ANIME, isAdult: false) {{ {MEDIA_FIELDS} }} }} }}");
        let d: PageData = self.query(&q, vars).await?;
        Ok(d.page.media.into_iter().map(to_summary).collect())
    }

    pub async fn trending(&self) -> Result<Vec<TitleSummary>> {
        self.page("$p:Int", "sort: TRENDING_DESC, popularity_greater: $p", json!({ "p": 1000 })).await
    }

    /// Episodes airing in the next/previous 24 h, newest first.
    pub async fn airing_today(&self) -> Result<Vec<TitleSummary>> {
        let now = chrono::Utc::now().timestamp();
        let q = format!(
            "query($from:Int,$to:Int){{ Page(perPage: 40) {{ airingSchedules(airingAt_greater: $from, airingAt_lesser: $to, sort: TIME_DESC) {{ episode media {{ {MEDIA_FIELDS} }} }} }} }}"
        );
        let d: PageData = self.query(&q, json!({ "from": now - 86_400, "to": now + 3_600 })).await?;
        let mut out: Vec<TitleSummary> = Vec::new();
        for a in d.page.airing {
            if a.media.is_adult || out.iter().any(|s| s.ids.anilist == Some(a.media.id)) {
                continue;
            }
            let ep = a.episode;
            let mut s = to_summary(a.media);
            s.badges.insert(0, format!("EP {ep}"));
            out.push(s);
        }
        Ok(out)
    }
}

fn title_case(s: &str) -> String {
    s.split('_')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn anilist_person(n: &PersonNode, role: Option<String>) -> Option<Person> {
    Some(Person { name: n.name.full.clone()?, role, photo: n.image.as_ref().and_then(|i| i.medium.clone()) })
}

/// Title-page extras; takes the recommendation list out of `m`.
fn extras(m: &mut Media) -> TitleExtra {
    let release_date = m.start_date.as_ref().and_then(|d| match (d.year, d.month, d.day) {
        (Some(y), Some(mo), Some(da)) => Some(format!("{y:04}-{mo:02}-{da:02}")),
        (Some(y), Some(mo), None) => Some(format!("{y:04}-{mo:02}")),
        (Some(y), None, _) => Some(format!("{y:04}")),
        _ => None,
    });
    let season_label = match (m.season.as_deref(), m.season_year) {
        (Some(s), Some(y)) => Some(format!("{} {y}", title_case(s))),
        _ => None,
    };
    // Each character with their Japanese voice actor, e.g. "Frieren · Atsumi Tanezaki".
    let cast = m
        .characters
        .as_ref()
        .map(|c| {
            c.edges
                .iter()
                .filter_map(|e| {
                    let character = e.node.name.full.clone()?;
                    match e.voice_actors.first() {
                        Some(va) => anilist_person(va, Some(character)),
                        None => anilist_person(&e.node, None),
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    let crew = m
        .staff
        .as_ref()
        .map(|s| s.edges.iter().filter_map(|e| anilist_person(&e.node, e.role.clone())).collect())
        .unwrap_or_default();
    let recommendations = m
        .recommendations
        .take()
        .map(|r| {
            r.nodes
                .into_iter()
                .filter_map(|n| n.media_recommendation)
                .filter(|x| !x.is_adult)
                .map(|x| to_summary(*x))
                .collect()
        })
        .unwrap_or_default();
    TitleExtra {
        release_date,
        status: m.status.as_deref().map(title_case),
        original_language: match m.country_of_origin.as_deref() {
            Some("JP") => Some("ja".into()),
            Some("CN" | "TW") => Some("zh".into()),
            Some("KR") => Some("ko".into()),
            _ => None,
        },
        countries: m.country_of_origin.clone().into_iter().collect(),
        studios: m.studios.as_ref().map(|s| s.nodes.iter().map(|n| n.name.clone()).collect()).unwrap_or_default(),
        rating_source: m.average_score.map(|_| "AniList".into()),
        episode_count: m.episodes,
        format: m.format.as_deref().map(|f| if f == "TV" || f == "OVA" || f == "ONA" { f.to_string() } else { title_case(f) }),
        source_material: m.source.as_deref().map(title_case),
        season_label,
        cast,
        crew,
        keywords: m.tags.iter().filter(|t| !t.is_media_spoiler).take(12).map(|t| t.name.clone()).collect(),
        recommendations,
        ..Default::default()
    }
}

fn to_summary(m: Media) -> TitleSummary {
    let name = m.title.english.clone().or_else(|| m.title.romaji.clone()).unwrap_or_default();
    let mut s = TitleSummary::new(ProviderId::Anilist, m.id.to_string(), name, MediaKind::Anime);
    s.original_name = m.title.romaji.clone().filter(|r| Some(r) != m.title.english.as_ref()).or_else(|| m.title.native.clone());
    s.year = m.season_year;
    s.poster = m.cover_image.extra_large.clone().or_else(|| m.cover_image.large.clone());
    s.backdrop = m.banner_image.clone();
    s.overview = m.description.as_deref().map(strip_tags);
    s.rating = m.average_score.map(|x| x as f32 / 10.0);
    s.ids.anilist = Some(m.id);
    s.ids.mal = m.id_mal;
    if let Some(f) = m.format.as_deref() {
        if f == "MOVIE" {
            s.badges.push("Movie".into());
        }
    }
    if m.is_adult {
        s.badges.push("18+".into());
    }
    s
}

#[async_trait]
impl Provider for Anilist {
    fn id(&self) -> ProviderId {
        ProviderId::Anilist
    }

    fn kinds(&self) -> &'static [MediaKind] {
        &[MediaKind::Anime]
    }

    async fn search(&self, query: &str) -> Result<Vec<TitleSummary>> {
        self.page("$q:String", "search: $q, sort: SEARCH_MATCH", json!({ "q": query })).await
    }

    async fn detail(&self, id: &str) -> Result<TitleDetail> {
        let key = format!("anilist:detail2:{id}");
        if let Some(d) = self.ctx.cache.get::<TitleDetail>(&key).await {
            return Ok(d);
        }
        let aid: u64 = id.parse().map_err(|_| Error::NotFound(format!("bad anilist id {id}")))?;
        let q = format!(
            "query($id:Int){{ Media(id: $id, type: ANIME) {{ {MEDIA_FIELDS} duration trailer {{ id site }} \
             streamingEpisodes {{ title thumbnail }} season source countryOfOrigin \
             startDate {{ year month day }} studios(isMain: true) {{ nodes {{ name }} }} \
             characters(sort: [ROLE, RELEVANCE], perPage: 16) {{ edges {{ node {{ name {{ full }} image {{ medium }} }} \
             voiceActors(language: JAPANESE) {{ name {{ full }} image {{ medium }} }} }} }} \
             staff(perPage: 8) {{ edges {{ role node {{ name {{ full }} image {{ medium }} }} }} }} \
             tags {{ name isMediaSpoiler }} \
             recommendations(perPage: 12, sort: RATING_DESC) {{ nodes {{ mediaRecommendation {{ {MEDIA_FIELDS} }} }} }} }} }}"
        );
        let d: MediaData = self.query(&q, json!({ "id": aid })).await?;
        let mut m = d.media;

        // Released episode count: known total, else (next airing − 1).
        let released = match (&m.next_airing_episode, m.episodes) {
            (Some(n), _) => n.episode.saturating_sub(1),
            (None, Some(total)) => total,
            (None, None) => m.streaming_episodes.len() as u32,
        };
        let thumbs: Vec<(Option<String>, Option<String>)> =
            m.streaming_episodes.iter().map(|e| (e.title.clone(), e.thumbnail.clone())).collect();
        let episodes = (1..=released.max(u32::from(m.format.as_deref() == Some("MOVIE"))))
            .map(|n| {
                let (title, thumb) = thumbs.get(n as usize - 1).cloned().unwrap_or((None, None));
                Episode { season: 1, number: n, title, overview: None, air_date: None, thumbnail: thumb, key: None }
            })
            .collect::<Vec<_>>();

        let genres = m.genres.clone();
        let runtime = m.duration;
        let trailer = m.trailer.as_ref().and_then(|t| match (t.site.as_deref(), t.id.as_deref()) {
            (Some("youtube"), Some(id)) => Some(format!("https://www.youtube.com/embed/{id}")),
            _ => None,
        });
        let extra = extras(&mut m);
        let mut detail = TitleDetail::from_summary(to_summary(m));
        detail.extra = extra;
        detail.genres = genres;
        detail.runtime_minutes = runtime;
        detail.trailer = trailer;
        if !episodes.is_empty() {
            detail.seasons = vec![Season { number: 1, name: None, episodes }];
        }
        self.ctx.cache.put(&key, &detail, ttl::DETAIL).await;
        Ok(detail)
    }

    async fn sources(&self, _: &TitleDetail, _: Target, _: AudioMode) -> Result<Vec<Source>> {
        Ok(Vec::new())
    }

    async fn rails(&self) -> Result<Vec<Rail>> {
        let (trending, airing) = tokio::join!(self.trending(), self.airing_today());
        let mut rails = Vec::new();
        if let Ok(items) = trending {
            rails.push(Rail { id: "trending_anime".into(), title: "Trending Anime".into(), items });
        }
        if let Ok(items) = airing {
            if !items.is_empty() {
                rails.push(Rail { id: "airing_today".into(), title: "Airing Today".into(), items });
            }
        }
        if rails.is_empty() {
            return Err(Error::provider(ProviderId::Anilist, "anilist unavailable"));
        }
        Ok(rails)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_mapping() {
        let m: Media = serde_json::from_str(
            r#"{"id":154587,"idMal":52991,"title":{"romaji":"Sousou no Frieren","english":"Frieren: Beyond Journey's End"},
                "coverImage":{"extraLarge":"x.jpg"},"description":"A <i>mage</i><br>","averageScore":91,"seasonYear":2023,"format":"TV","isAdult":false}"#,
        )
        .unwrap();
        let s = to_summary(m);
        assert_eq!(s.name, "Frieren: Beyond Journey's End");
        assert_eq!(s.original_name.as_deref(), Some("Sousou no Frieren"));
        assert_eq!(s.ids.mal, Some(52991));
        assert_eq!(s.rating, Some(9.1));
        assert_eq!(s.overview.as_deref(), Some("A mage"));
    }
}
