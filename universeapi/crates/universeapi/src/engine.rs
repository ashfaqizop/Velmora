//! The UniverseAPI facade: one object the app talks to.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use futures::stream::{self, FuturesUnordered, Stream, StreamExt};
use futures::FutureExt;
use serde::{Deserialize, Serialize};

use crate::cache::Cache;
use crate::config::{Preferences, Registry};
use crate::error::{Error, Result};
use crate::metadata::{anilist::Anilist, cinemeta::Cinemeta, tmdb::Tmdb};
use crate::model::*;
use crate::net::{BlockedFallback, Http};
use crate::providers::{anime_sama::AnimeSama, coflix::Coflix, dramachi::Dramachi, french_stream::FrenchStream, fourkhdhub::FourKHdHub, hdhub::HdHub, hianime::HiAnime, moviebox::MovieBox, same_title, torrents::{Eztv, Nyaa, Yts}, zokoanime::ZokoAnime, Ctx, Provider};
use crate::rank;
use crate::resolvers::WebSolver;

const SEARCH_TIMEOUT: Duration = Duration::from_secs(12);
const RAIL_TIMEOUT: Duration = Duration::from_secs(15);
const SOURCES_TIMEOUT: Duration = Duration::from_secs(40);

#[derive(Default)]
pub struct EngineOptions {
    pub preferences: Preferences,
    pub registry: Registry,
    /// Persistent cache directory (app cache dir). `None` = memory only.
    pub cache_dir: Option<PathBuf>,
    /// Hidden-WebView solver for JS/timer hosts (GreenMotors).
    pub web_solver: Option<Arc<dyn WebSolver>>,
    /// Browser-fingerprint transport used when a site blocks the normal client.
    pub blocked_fallback: Option<Arc<dyn BlockedFallback>>,
    /// Where downloads (and torrent pieces) are written. Defaults to the OS temp dir.
    pub download_dir: Option<PathBuf>,
    /// Where managed tools (yt-dlp for music) live. Defaults to `cache_dir/tools`.
    pub tools_dir: Option<PathBuf>,
}

/// Sources plus per-provider failures (shown in a "Sources" diagnostics sheet).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SourcesReport {
    pub sources: Vec<Source>,
    pub errors: Vec<ProviderFailure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderFailure {
    pub provider: ProviderId,
    pub message: String,
}

/// Progress of a source search, for a page that shows it as it happens.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SourcesEvent {
    /// The providers being asked, best first.
    Plan { providers: Vec<ProviderId> },
    /// One provider is done. No sources and no error: it doesn't have it.
    Batch {
        provider: ProviderId,
        sources: Vec<Source>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
        /// Came from the cache, no network.
        cached: bool,
    },
}

/// One provider's search results, emitted as soon as it answers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchBatch {
    pub provider: ProviderId,
    pub items: Vec<TitleSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub struct Engine {
    ctx: Arc<Ctx>,
    tmdb: Arc<Tmdb>,
    cinemeta: Arc<Cinemeta>,
    anilist: Arc<Anilist>,
    sports: Arc<crate::sports::Sports>,
    tv: Arc<crate::tv::Tv>,
    music: Arc<crate::music::Music>,
    /// Content providers (produce sources).
    content: Vec<Arc<dyn Provider>>,
    /// The folder used when no `download_dir` preference is set.
    default_download_dir: PathBuf,
    downloads: Arc<crate::download::DownloadManager>,
    proxy: Arc<crate::proxy::MediaProxy>,
    #[cfg(feature = "torrent")]
    torrents: crate::download::TorrentSlot,
}

impl Engine {
    pub fn new(opts: EngineOptions) -> Result<Self> {
        let tools_dir = opts
            .tools_dir
            .clone()
            .or_else(|| opts.cache_dir.as_ref().map(|d| d.join("tools")))
            .unwrap_or_else(|| std::env::temp_dir().join("velmora-tools"));
        let music = Arc::new(crate::music::Music::new(tools_dir)?);
        let mut http = Http::new(&opts.registry.user_agent)?;
        if let Some(fb) = opts.blocked_fallback {
            http = http.with_fallback(fb);
        } else {
            #[cfg(feature = "impersonate")]
            {
                http = http.with_fallback(Arc::new(crate::impersonate::ChromeFallback::new()?));
            }
        }
        let cache = match opts.cache_dir {
            Some(dir) => Cache::with_dir(dir),
            None => Cache::memory(),
        };
        let default_download_dir =
            opts.download_dir.unwrap_or_else(|| std::env::temp_dir().join("velmora-downloads"));
        let download_dir = opts.preferences.download_dir.clone().unwrap_or_else(|| default_download_dir.clone());
        let max_downloads = opts.preferences.max_concurrent_downloads;
        let ctx = Arc::new(Ctx::new(http, cache, opts.registry, opts.preferences, opts.web_solver));
        #[cfg(feature = "torrent")]
        let torrents: crate::download::TorrentSlot = Arc::new(tokio::sync::OnceCell::new());
        let proxy = Arc::new(crate::proxy::MediaProxy::new(
            ctx.http.clone(),
            #[cfg(feature = "torrent")]
            torrents.clone(),
        ));
        let downloads = Arc::new(crate::download::DownloadManager::new(
            ctx.clone(),
            download_dir,
            max_downloads,
            #[cfg(feature = "torrent")]
            torrents.clone(),
        ));
        let content: Vec<Arc<dyn Provider>> = vec![
            Arc::new(MovieBox::new(ctx.clone())),
            Arc::new(HdHub::new(ctx.clone())),
            Arc::new(FourKHdHub::new(ctx.clone())),
            Arc::new(Dramachi::new(ctx.clone())),
            Arc::new(AnimeSama::new(ctx.clone())),
            Arc::new(FrenchStream::new(ctx.clone())),
            Arc::new(Coflix::new(ctx.clone())),
            Arc::new(Yts::new(ctx.clone())),
            Arc::new(Eztv::new(ctx.clone())),
            Arc::new(Nyaa::new(ctx.clone())),
            Arc::new(ZokoAnime::new(ctx.clone())),
            Arc::new(HiAnime::new(ctx.clone())),
        ];
        Ok(Self {
            tmdb: Arc::new(Tmdb::new(ctx.clone())),
            cinemeta: Arc::new(Cinemeta::new(ctx.clone())),
            anilist: Arc::new(Anilist::new(ctx.clone())),
            sports: Arc::new(crate::sports::Sports::new(ctx.clone())),
            tv: Arc::new(crate::tv::Tv::new(ctx.clone())),
            music,
            content,
            default_download_dir,
            downloads,
            proxy,
            #[cfg(feature = "torrent")]
            torrents,
            ctx,
        })
    }

    /// Media proxy for the in-app player (header injection, manifest rewriting, torrent streaming).
    pub fn proxy(&self) -> &Arc<crate::proxy::MediaProxy> {
        &self.proxy
    }

    /// Download queue (direct files, HLS/DASH via ffmpeg, torrents).
    pub fn downloads(&self) -> &Arc<crate::download::DownloadManager> {
        &self.downloads
    }

    /// The folder downloads currently go to.
    pub fn download_dir(&self) -> PathBuf {
        self.downloads.dir()
    }

    /// The folder used when the user hasn't picked one.
    pub fn default_download_dir(&self) -> &std::path::Path {
        &self.default_download_dir
    }

    /// The torrent session, started on first use.
    #[cfg(feature = "torrent")]
    pub async fn torrents(&self) -> Result<&crate::torrent::TorrentEngine> {
        self.torrents.get_or_try_init(|| crate::torrent::TorrentEngine::new(self.download_dir().join("torrents"))).await
    }

    /// TMDB, for read-only lookups beyond search and detail (discover, people, collections).
    pub fn tmdb(&self) -> &Arc<Tmdb> {
        &self.tmdb
    }

    /// AniList, for anime lookups beyond search and detail (filters, schedules).
    pub fn anilist(&self) -> &Arc<Anilist> {
        &self.anilist
    }

    /// Live scores, fixtures and results.
    pub fn sports(&self) -> &Arc<crate::sports::Sports> {
        &self.sports
    }

    /// YouTube Music: browse, search, library, lyrics and playback.
    pub fn music(&self) -> &Arc<crate::music::Music> {
        &self.music
    }

    /// Live TV channels from the public iptv-org playlist.
    pub fn tv(&self) -> &Arc<crate::tv::Tv> {
        &self.tv
    }

    /// Where to watch a match in a country. Like [`crate::sports::Sports::watch`],
    /// plus a check that a hand-listed free channel is actually in the TV
    /// library: if it isn't (or can't be loaded), the Play button is dropped.
    pub async fn sports_watch(&self, m: &crate::sports::Match, country: &str) -> Vec<crate::sports::watch::WatchOption> {
        let mut options = self.sports.watch(m, country).await;
        for o in options.iter_mut() {
            let Some(id) = o.play_channel.clone() else { continue };
            let playable = matches!(self.tv.channel(&id).await, Ok(Some(_)));
            if !playable {
                o.play_channel = None;
            }
        }
        options
    }

    pub fn ctx(&self) -> &Arc<Ctx> {
        &self.ctx
    }

    pub fn preferences(&self) -> Arc<Preferences> {
        self.ctx.prefs()
    }

    pub fn set_preferences(&self, prefs: Preferences) {
        self.downloads.set_max_concurrent(prefs.max_concurrent_downloads);
        self.downloads.set_dir(prefs.download_dir.clone().unwrap_or_else(|| self.default_download_dir.clone()));
        self.ctx.set_prefs(prefs);
    }

    pub fn registry(&self) -> Arc<Registry> {
        self.ctx.registry()
    }

    pub fn set_registry(&self, registry: Registry) {
        self.ctx.set_registry(registry);
    }

    /// Fetch the published remote registry ([`crate::config::REGISTRY_URL`]).
    /// Call this from the launch splash; on failure the engine keeps working
    /// with its current registry.
    pub async fn refresh_registry_default(&self) -> Result<()> {
        self.refresh_registry(crate::config::REGISTRY_URL).await
    }

    /// Download `registry.json` (repo `config` branch) and apply it over defaults.
    pub async fn refresh_registry(&self, url: &str) -> Result<()> {
        let text = self.ctx.http.get_text(url, &[]).await?;
        let mut reg = Registry::from_json_over_defaults(&text)?;
        // The public file never holds secrets: keep the keys the app was built
        // with (and a local mirror override) unless the remote sets its own.
        let current = self.registry();
        if reg.tmdb_read_token.is_none() {
            reg.tmdb_read_token.clone_from(&current.tmdb_read_token);
        }
        if reg.tmdb_api_key.is_none() {
            reg.tmdb_api_key.clone_from(&current.tmdb_api_key);
        }
        if reg.hdhub.force_mirror.is_none() {
            reg.hdhub.force_mirror.clone_from(&current.hdhub.force_mirror);
        }
        self.ctx.set_registry(reg);
        Ok(())
    }

    fn movie_tv_meta(&self) -> Arc<dyn Provider> {
        if self.tmdb.enabled() {
            self.tmdb.clone()
        } else {
            self.cinemeta.clone()
        }
    }

    /// Every provider (metadata + content) by id.
    pub fn provider(&self, id: ProviderId) -> Option<Arc<dyn Provider>> {
        match id {
            ProviderId::Tmdb => Some(self.tmdb.clone()),
            ProviderId::Cinemeta => Some(self.cinemeta.clone()),
            ProviderId::Anilist => Some(self.anilist.clone()),
            _ => self.content.iter().find(|p| p.id() == id).cloned(),
        }
    }

    fn enabled(&self, id: ProviderId) -> bool {
        self.ctx.prefs().is_enabled(id) && !self.ctx.registry().disabled.contains(&id)
    }

    fn visible(&self, item: &TitleSummary) -> bool {
        self.ctx.prefs().show_adult || !item.badges.iter().any(|b| b == "18+")
    }

    // ── Home ────────────────────────────────────────────────────────────

    /// Home feed rails, in display order. "Continue Watching" is built by the
    /// app from local history and inserted at the top.
    pub async fn home(&self) -> Vec<Rail> {
        let mut sources: Vec<Arc<dyn Provider>> = vec![self.movie_tv_meta(), self.anilist.clone()];
        sources.extend(self.content.iter().filter(|p| self.enabled(p.id())).cloned());

        let results = futures::future::join_all(sources.iter().map(|p| async move {
            match tokio::time::timeout(RAIL_TIMEOUT, p.rails()).await {
                Ok(Ok(r)) => r,
                Ok(Err(e)) => {
                    tracing::warn!(provider = %p.id(), "rails failed: {e}");
                    Vec::new()
                }
                Err(_) => {
                    tracing::warn!(provider = %p.id(), "rails timed out");
                    Vec::new()
                }
            }
        }))
        .await;

        const ORDER: &[&str] = &[
            "trending_movies",
            "trending_series",
            "trending_anime",
            "airing_today",
            "hdhub_latest",
            "kdrama",
            "hdhub_bollywood",
            "hdhub_south",
        ];
        let mut rails: Vec<Rail> = results.into_iter().flatten().collect();
        for r in &mut rails {
            r.items.retain(|i| self.visible(i));
        }
        rails.retain(|r| !r.items.is_empty());
        rails.sort_by_key(|r| ORDER.iter().position(|id| *id == r.id).unwrap_or(ORDER.len()));
        rails
    }

    // ── Search ──────────────────────────────────────────────────────────

    fn search_targets(&self) -> Vec<Arc<dyn Provider>> {
        let mut v: Vec<Arc<dyn Provider>> = vec![self.movie_tv_meta(), self.anilist.clone()];
        // Torrent and id-addressed providers only add sources; they never list titles.
        v.extend(
            self.content.iter().filter(|p| self.enabled(p.id()) && p.id() != ProviderId::ZokoAnime && !p.id().is_torrent()).cloned(),
        );
        v
    }

    /// Progressive search: yields one batch per provider as it answers.
    pub fn search_stream<'a>(&'a self, query: &'a str) -> impl Stream<Item = SearchBatch> + 'a {
        let futs: FuturesUnordered<_> = self
            .search_targets()
            .into_iter()
            .map(|p| async move {
                let provider = p.id();
                match tokio::time::timeout(SEARCH_TIMEOUT, p.search(query)).await {
                    Ok(Ok(mut items)) => {
                        items.retain(|i| self.visible(i));
                        SearchBatch { provider, items, error: None }
                    }
                    Ok(Err(e)) => SearchBatch { provider, items: Vec::new(), error: Some(e.to_string()) },
                    Err(_) => SearchBatch { provider, items: Vec::new(), error: Some("timed out".into()) },
                }
            })
            .collect();
        futs
    }

    /// Full search, merged across providers (metadata entries first).
    pub async fn search(&self, query: &str) -> Vec<TitleSummary> {
        let mut batches: Vec<SearchBatch> = self.search_stream(query).collect().await;
        // Canonical metadata sources first so their artwork/ids win.
        batches.sort_by_key(|b| match b.provider {
            ProviderId::Tmdb | ProviderId::Cinemeta | ProviderId::Anilist => 0,
            _ => 1,
        });
        let mut merged: Vec<TitleSummary> = Vec::new();
        for b in batches {
            for item in b.items {
                merge_into(&mut merged, item);
            }
        }
        merged
    }

    /// Full search, looked over: the titles that exactly match the query, plus
    /// everything ranked best first. See [`crate::relevance`].
    pub async fn search_smart(&self, query: &str) -> crate::relevance::Narrowed {
        let all = self.search(query).await;
        crate::relevance::narrow(&all, query)
    }

    /// A title's page. A title that came from a streaming site (a Home row
    /// from HDHub, a search hit only one site had) gets the metadata page for
    /// the same title instead: TMDB (or Cinemeta) for films and shows, AniList
    /// for anime. A site's own page is thin: its headline is a release name,
    /// its "genres" are file tags (1080p, HEVC) and its screenshots won't
    /// load outside the site. The site stays listed as a place it's on, so
    /// its sources are still found. With no match, the site's page is used.
    pub async fn detail(&self, r: &ProviderRef) -> Result<TitleDetail> {
        let p = self.provider(r.provider).ok_or_else(|| Error::Unsupported(format!("provider {}", r.provider)))?;
        let d = p.detail(&r.id).await?;
        if is_metadata(r.provider) {
            return Ok(d);
        }
        Ok(self.canonical_detail(r, d).await)
    }

    async fn canonical_detail(&self, r: &ProviderRef, site: TitleDetail) -> TitleDetail {
        let Some(found) = self.metadata_match(r, &site.summary).await else { return site };
        let Some(meta) = self.provider(found.provider) else { return site };
        let mut d = match meta.detail(&found.id).await {
            Ok(d) => d,
            Err(e) => {
                tracing::debug!("metadata page for {} failed: {e}", site.summary.name);
                return site;
            }
        };
        // Where it's available: the site it came from, and any it already knew.
        for place in std::iter::once(&site.summary.origin).chain(site.summary.also_on.iter()) {
            if place.provider != d.summary.origin.provider && !d.summary.also_on.iter().any(|x| x.provider == place.provider) {
                d.summary.also_on.push(place.clone());
            }
        }
        d.summary.ids.absorb(&site.summary.ids);
        // A show the metadata source has no episode list for keeps the site's.
        if d.seasons.is_empty() {
            d.seasons = site.seasons;
        }
        d
    }

    /// The metadata entry for a title a site listed, found by name, year and
    /// kind (or a shared id). Remembered, so it's looked up once per title.
    async fn metadata_match(&self, r: &ProviderRef, s: &TitleSummary) -> Option<ProviderRef> {
        let key = format!("canonical:{:?}:{}", r.provider, r.id);
        if let Some(hit) = self.ctx.cache.get::<Option<ProviderRef>>(&key).await {
            return hit;
        }
        let meta: Arc<dyn Provider> = if s.kind == MediaKind::Anime { self.anilist.clone() } else { self.movie_tv_meta() };
        let mut queries = vec![s.name.clone()];
        if let Some(o) = s.original_name.as_ref().filter(|o| **o != s.name) {
            queries.push(o.clone());
        }
        let mut hit = None;
        for q in queries {
            let Ok(found) = tokio::time::timeout(SEARCH_TIMEOUT, meta.search(&q)).await else { break };
            let Ok(found) = found else { continue };
            // Prefer an exact year when the site gave one; same_title allows ±1.
            let mut matches: Vec<&TitleSummary> = found.iter().filter(|c| same_title(s, c) && kinds_compatible(s.kind, c.kind)).collect();
            matches.sort_by_key(|c| match (s.year, c.year) {
                (Some(a), Some(b)) => a.abs_diff(b),
                _ => 1,
            });
            if let Some(c) = matches.first() {
                hit = Some(c.origin.clone());
                break;
            }
        }
        // A miss is remembered for less long: the metadata source may catch up.
        let ttl = if hit.is_some() { crate::cache::ttl::DETAIL } else { crate::cache::ttl::DETAIL / 4 };
        self.ctx.cache.put(&key, &hit, ttl).await;
        hit
    }

    // ── Sources ─────────────────────────────────────────────────────────

    /// Find, resolve, verify and rank every source for a title.
    pub async fn sources(&self, title: &TitleSummary, target: Target, mode: AudioMode) -> SourcesReport {
        self.sources_each(title, target, mode, |_| {}).await
    }

    /// [`Engine::sources`], reporting each provider as it finishes, so a page
    /// can show where a title is while the slower sites are still working.
    ///
    /// Each provider's verified sources are cached until the first of their
    /// links expires (40 minutes when a link doesn't say, 6 hours for
    /// torrents), and a provider that doesn't have the title is remembered
    /// for 30 minutes. Coming back to a title is then instant. A cached link
    /// that fails to play is dropped with [`Engine::forget_sources`].
    pub async fn sources_each<F: FnMut(SourcesEvent) + Send>(
        &self,
        title: &TitleSummary,
        target: Target,
        mode: AudioMode,
        mut on: F,
    ) -> SourcesReport {
        let title = &self.with_ids(title).await;
        let prefs = self.ctx.prefs();
        let kind = title.kind;
        let mut providers: Vec<Arc<dyn Provider>> = self
            .content
            .iter()
            .filter(|p| self.enabled(p.id()) && p.kinds().contains(&kind))
            .cloned()
            .collect();
        providers.sort_by_key(|p| prefs.provider_rank(kind, p.id()));
        on(SourcesEvent::Plan { providers: providers.iter().map(|p| p.id()).collect() });

        let verify = prefs.verify_sources;
        let futs: Vec<_> = providers
            .iter()
            .map(|p| async move {
                let provider = p.id();
                (provider, self.cached_provider_sources(p.as_ref(), title, target, mode, verify).await)
            })
            .collect();
        let mut pending: FuturesUnordered<_> = futs.into_iter().collect();

        let mut report = SourcesReport::default();
        while let Some((provider, r)) = pending.next().await {
            match r {
                Ok((list, cached)) => {
                    on(SourcesEvent::Batch { provider, sources: list.clone(), error: None, cached });
                    for s in list {
                        if !report.sources.iter().any(|x| x.url == s.url) {
                            report.sources.push(s);
                        }
                    }
                }
                Err(e) => {
                    let message = e.to_string();
                    on(SourcesEvent::Batch { provider, sources: Vec::new(), error: Some(message.clone()), cached: false });
                    report.errors.push(ProviderFailure { provider, message });
                }
            }
        }
        rank::rank(&mut report.sources, &prefs, kind, target);
        report
    }

    /// Forget the cached sources for a title (one of them didn't play).
    pub async fn forget_sources(&self, title: &TitleSummary, target: Target, mode: AudioMode) {
        for p in &self.content {
            self.ctx.cache.put(&sources_key(p.id(), title, target, mode), &Option::<Vec<Source>>::None, Duration::ZERO).await;
        }
    }

    /// One provider's sources for a title, from the cache when they're still good.
    async fn cached_provider_sources(
        &self,
        p: &dyn Provider,
        title: &TitleSummary,
        target: Target,
        mode: AudioMode,
        verify: bool,
    ) -> Result<(Vec<Source>, bool)> {
        let pid = p.id();
        let key = sources_key(pid, title, target, mode);
        if let Some(Some(hit)) = self.ctx.cache.get::<Option<Vec<Source>>>(&key).await {
            return Ok((hit, true));
        }
        let found = match tokio::time::timeout(SOURCES_TIMEOUT, self.provider_sources(p, title, target, mode)).await {
            Ok(r) => r,
            Err(_) => return Err(Error::provider(pid, "timed out")),
        };
        let mut list = match found {
            Ok(list) => list,
            Err(Error::NotFound(_)) => {
                self.ctx.cache.put(&key, &Some(Vec::<Source>::new()), Duration::from_secs(30 * 60)).await;
                return Ok((Vec::new(), false));
            }
            Err(e) => return Err(e),
        };
        if verify && !list.is_empty() {
            self.verify(&mut list).await;
        }
        let ttl = if list.is_empty() { Duration::from_secs(30 * 60) } else { sources_ttl(&list) };
        self.ctx.cache.put(&key, &Some(list.clone()), ttl).await;
        Ok((list, false))
    }

    /// Search hits often lack external ids (a TMDB search result has no IMDb
    /// id until its detail is fetched). Fill them from the title's own detail
    /// page so id-addressed providers (YTS, EZTV, ZokoAnime) can be used.
    async fn with_ids(&self, title: &TitleSummary) -> TitleSummary {
        let mut t = title.clone();
        let missing = match t.kind {
            MediaKind::Anime => t.ids.mal.is_none(),
            _ => t.ids.imdb.is_none(),
        };
        if missing {
            if let Ok(d) = self.detail(&t.origin).await {
                t.ids.absorb(&d.summary.ids);
            }
        }
        t
    }

    async fn provider_sources(
        &self,
        p: &dyn Provider,
        title: &TitleSummary,
        target: Target,
        mode: AudioMode,
    ) -> Result<Vec<Source>> {
        if let Some(s) = p.sources_by_ids(&title.ids, title.kind, target, mode).await? {
            return Ok(s);
        }
        let id = self.locate(p, title).await?;
        let detail = p.detail(&id).await?;
        p.sources(&detail, target, mode).await
    }

    /// Find this provider's id for `title` (known reference, else search).
    async fn locate(&self, p: &dyn Provider, title: &TitleSummary) -> Result<String> {
        let pid = p.id();
        if title.origin.provider == pid {
            return Ok(title.origin.id.clone());
        }
        if let Some(r) = title.also_on.iter().find(|r| r.provider == pid) {
            return Ok(r.id.clone());
        }
        let key = format!("locate:{pid:?}:{}:{:?}", title.name, title.year);
        if let Some(id) = self.ctx.cache.get::<String>(&key).await {
            return Ok(id);
        }
        let mut queries = vec![title.name.clone()];
        if let Some(o) = title.original_name.as_ref().filter(|o| **o != title.name) {
            queries.push(o.clone());
        }
        for q in queries {
            let found = p.search(&q).await?;
            if let Some(hit) = found.iter().find(|c| same_title(title, c) && kinds_compatible(title.kind, c.kind)) {
                let id = hit.origin.id.clone();
                // Where a site keeps a title rarely changes; finding it is most of the wait.
                self.ctx.cache.put(&key, &id, crate::cache::ttl::LOCATE).await;
                return Ok(id);
            }
        }
        Err(Error::NotFound(format!("{} not on {}", title.name, pid)))
    }

    /// Probe each source; drop decoys (HTML ad pages), record sizes.
    pub async fn verify(&self, sources: &mut [Source]) {
        let http = &self.ctx.http;
        // Futures are built up front (no borrowing closure inside the stream) so
        // this stays `Send` for Tauri commands — see rust-lang#64552.
        let futs: Vec<_> = sources
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let url = s.url.clone();
                let headers: Vec<(String, String)> = s.headers.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                let skip = s.is_torrent();
                async move {
                    if skip {
                        return (i, None);
                    }
                    (i, http.probe(&url, &headers).await.ok())
                }
                .boxed()
            })
            .collect();
        let probes: Vec<(usize, Option<crate::net::Probe>)> = stream::iter(futs).buffer_unordered(8).collect().await;
        for (i, probe) in probes {
            let Some(p) = probe else { continue };
            let s = &mut sources[i];
            s.verified = Some(p.is_media);
            if s.size_bytes.is_none() && s.format != StreamFormat::Hls {
                s.size_bytes = p.total_size;
            }
            if s.format == StreamFormat::Unknown {
                if let Some(ct) = &p.content_type {
                    s.format = match ct.as_str() {
                        c if c.contains("mpegurl") => StreamFormat::Hls,
                        c if c.contains("dash") => StreamFormat::Dash,
                        c if c.contains("mp4") => StreamFormat::Mp4,
                        c if c.contains("matroska") || c.contains("mkv") => StreamFormat::Mkv,
                        c if c.contains("zip") => StreamFormat::Archive,
                        _ => s.format,
                    };
                }
            }
            s.retag();
        }
    }
}

/// The cache key for one provider's sources for a title: by its canonical id
/// when it has one, so the same film reached from TMDB or from a site's row
/// shares one entry.
fn sources_key(p: ProviderId, t: &TitleSummary, target: Target, mode: AudioMode) -> String {
    let who = if let Some(i) = &t.ids.imdb {
        format!("imdb:{i}")
    } else if let Some(i) = t.ids.tmdb {
        format!("tmdb:{:?}:{i}", t.kind)
    } else if let Some(i) = t.ids.anilist {
        format!("anilist:{i}")
    } else if let Some(i) = t.ids.mal {
        format!("mal:{i}")
    } else {
        format!("{:?}:{}", t.origin.provider, t.origin.id)
    };
    let at = match target {
        Target::Movie => "movie".to_string(),
        Target::Episode { season, episode } => format!("s{season}e{episode}"),
        Target::Season { season } => format!("s{season}"),
    };
    format!("sources:v1:{p:?}:{who}:{at}:{mode:?}")
}

/// How long a provider's sources stay good: until the first signed link
/// expires (less five minutes), 40 minutes when they don't say, and longer
/// for torrents, whose magnets don't expire.
fn sources_ttl(list: &[Source]) -> Duration {
    if list.iter().all(|s| s.is_torrent()) {
        return Duration::from_secs(6 * 60 * 60);
    }
    let now = chrono::Utc::now().timestamp();
    match list.iter().filter_map(|s| s.expires_at).min() {
        Some(exp) => Duration::from_secs((exp - now - 300).clamp(60, 60 * 60) as u64),
        None => Duration::from_secs(40 * 60),
    }
}

/// TMDB, Cinemeta and AniList describe titles; the rest are sites that host them.
fn is_metadata(p: ProviderId) -> bool {
    matches!(p, ProviderId::Tmdb | ProviderId::Cinemeta | ProviderId::Anilist)
}

fn kinds_compatible(a: MediaKind, b: MediaKind) -> bool {
    a == b || (a.is_episodic() && b.is_episodic())
}

/// Merge `item` into `list`: same title → record the provider, fill gaps.
fn merge_into(list: &mut Vec<TitleSummary>, item: TitleSummary) {
    if let Some(existing) = list.iter_mut().find(|e| kinds_compatible(e.kind, item.kind) && same_title(e, &item)) {
        if !existing.also_on.iter().any(|r| r.provider == item.origin.provider) && existing.origin.provider != item.origin.provider {
            existing.also_on.push(item.origin.clone());
        }
        existing.ids.absorb(&item.ids);
        if existing.poster.is_none() {
            existing.poster = item.poster;
        }
        if existing.year.is_none() {
            existing.year = item.year;
        }
        for b in item.badges {
            if !existing.badges.contains(&b) && existing.badges.len() < 6 {
                existing.badges.push(b);
            }
        }
        return;
    }
    list.push(item);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_collapses_same_title() {
        let mut list = Vec::new();
        let mut a = TitleSummary::new(ProviderId::Cinemeta, "movie/tt1", "Dune", MediaKind::Movie);
        a.year = Some(2021);
        a.ids.imdb = Some("tt1160419".into());
        let mut b = TitleSummary::new(ProviderId::HdHub, "dune-2021", "Dune", MediaKind::Movie);
        b.year = Some(2021);
        b.ids.imdb = Some("tt1160419".into());
        b.badges = vec!["Dual Audio".into()];
        let c = TitleSummary::new(ProviderId::HdHub, "dune-1984", "Dune", MediaKind::Movie);
        merge_into(&mut list, a);
        merge_into(&mut list, b);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].also_on[0].id, "dune-2021");
        assert!(list[0].badges.contains(&"Dual Audio".to_string()));
        let mut c = c;
        c.year = Some(1984);
        merge_into(&mut list, c);
        assert_eq!(list.len(), 2, "different year stays separate");
    }

    /// Tauri commands require `Send` futures; this fails to compile if any
    /// public engine future stops being `Send`.
    #[allow(dead_code)]
    fn public_futures_are_send(e: &Engine, t: &TitleSummary, r: &ProviderRef, s: &mut [Source]) {
        fn assert_send<T: Send>(_: T) {}
        assert_send(e.home());
        assert_send(e.search("x"));
        assert_send(e.search_smart("x"));
        assert_send(e.sports().today());
        assert_send(e.sports().live());
        assert_send(e.sports().upcoming(2));
        assert_send(e.sports().results(2));
        assert_send(e.sports().lookup("tsdb:1"));
        assert_send(e.tv().channels(&crate::tv::TvQuery::default()));
        assert_send(e.tv().check("x"));
        let t = TitleSummary::new(ProviderId::HdHub, "x", "x", MediaKind::Movie);
        assert_send(e.detail(&t.origin));
        assert_send(e.sources_each(&t, Target::Movie, AudioMode::Sub, |_| {}));
        assert_send(e.forget_sources(&t, Target::Movie, AudioMode::Sub));
        let m = e.music();
        assert_send(m.home(None));
        assert_send(m.feed("FEmusic_charts", None, None));
        assert_send(m.search("x", crate::music::SearchFilter::All));
        assert_send(m.album("x"));
        assert_send(m.artist("x"));
        assert_send(m.playlist("x"));
        assert_send(m.playlist_more("x"));
        assert_send(m.queue(Some("x"), None, None));
        assert_send(m.radio("x"));
        assert_send(m.lyrics("x", "t", "a", None));
        assert_send(m.account());
        assert_send(m.liked_songs(10));
        assert_send(m.library(crate::music::LibraryKind::Albums));
        assert_send(m.rate("x", crate::music::Rating::Like));
        assert_send(m.playlist_create("t", &[]));
        assert_send(m.resolve("x", crate::music::StreamMode::Audio, 720));
        assert_send(m.ytdlp().update());
        assert_send(e.search_stream("x").collect::<Vec<_>>());
        assert_send(e.detail(r));
        assert_send(e.sources(&t, Target::Movie, AudioMode::Sub));
        assert_send(e.verify(s));
        assert_send(e.refresh_registry("https://x"));
    }

    #[tokio::test]
    async fn engine_builds_offline() {
        let e = Engine::new(EngineOptions::default()).unwrap();
        assert!(e.provider(ProviderId::HdHub).is_some());
        assert!(e.provider(ProviderId::Cinemeta).is_some());
        assert!(!e.tmdb.enabled(), "no TMDB key by default → Cinemeta");
        assert_eq!(e.movie_tv_meta().id(), ProviderId::Cinemeta);
    }
}
