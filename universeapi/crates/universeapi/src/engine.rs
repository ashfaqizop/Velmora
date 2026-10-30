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
use crate::providers::{hdhub::HdHub, hianime::HiAnime, same_title, zokoanime::ZokoAnime, Ctx, Provider};
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
    /// Content providers (produce sources).
    content: Vec<Arc<dyn Provider>>,
}

impl Engine {
    pub fn new(opts: EngineOptions) -> Result<Self> {
        let mut http = Http::new(&opts.registry.user_agent)?;
        if let Some(fb) = opts.blocked_fallback {
            http = http.with_fallback(fb);
        }
        let cache = match opts.cache_dir {
            Some(dir) => Cache::with_dir(dir),
            None => Cache::memory(),
        };
        let ctx = Arc::new(Ctx::new(http, cache, opts.registry, opts.preferences, opts.web_solver));
        let content: Vec<Arc<dyn Provider>> = vec![
            Arc::new(HdHub::new(ctx.clone())),
            Arc::new(ZokoAnime::new(ctx.clone())),
            Arc::new(HiAnime::new(ctx.clone())),
        ];
        Ok(Self {
            tmdb: Arc::new(Tmdb::new(ctx.clone())),
            cinemeta: Arc::new(Cinemeta::new(ctx.clone())),
            anilist: Arc::new(Anilist::new(ctx.clone())),
            content,
            ctx,
        })
    }

    pub fn ctx(&self) -> &Arc<Ctx> {
        &self.ctx
    }

    pub fn preferences(&self) -> Arc<Preferences> {
        self.ctx.prefs()
    }

    pub fn set_preferences(&self, prefs: Preferences) {
        self.ctx.set_prefs(prefs);
    }

    pub fn registry(&self) -> Arc<Registry> {
        self.ctx.registry()
    }

    pub fn set_registry(&self, registry: Registry) {
        self.ctx.set_registry(registry);
    }

    /// Download `registry.json` (repo `config` branch) and apply it over defaults.
    pub async fn refresh_registry(&self, url: &str) -> Result<()> {
        let text = self.ctx.http.get_text(url, &[]).await?;
        let reg = Registry::from_json_over_defaults(&text)?;
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
        v.extend(self.content.iter().filter(|p| self.enabled(p.id()) && p.id() != ProviderId::ZokoAnime).cloned());
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

    pub async fn detail(&self, r: &ProviderRef) -> Result<TitleDetail> {
        let p = self.provider(r.provider).ok_or_else(|| Error::Unsupported(format!("provider {}", r.provider)))?;
        p.detail(&r.id).await
    }

    // ── Sources ─────────────────────────────────────────────────────────

    /// Find, resolve, verify and rank every source for a title.
    pub async fn sources(&self, title: &TitleSummary, target: Target, mode: AudioMode) -> SourcesReport {
        let prefs = self.ctx.prefs();
        let kind = title.kind;
        let mut providers: Vec<Arc<dyn Provider>> = self
            .content
            .iter()
            .filter(|p| self.enabled(p.id()) && p.kinds().contains(&kind))
            .cloned()
            .collect();
        providers.sort_by_key(|p| prefs.provider_rank(kind, p.id()));

        let results = futures::future::join_all(providers.iter().map(|p| async move {
            let provider = p.id();
            let r = tokio::time::timeout(SOURCES_TIMEOUT, self.provider_sources(p.as_ref(), title, target, mode)).await;
            (provider, r.unwrap_or_else(|_| Err(Error::provider(provider, "timed out"))))
        }))
        .await;

        let mut report = SourcesReport::default();
        for (provider, r) in results {
            match r {
                Ok(list) => {
                    for s in list {
                        if !report.sources.iter().any(|x| x.url == s.url) {
                            report.sources.push(s);
                        }
                    }
                }
                Err(Error::NotFound(_)) => {}
                Err(e) => report.errors.push(ProviderFailure { provider, message: e.to_string() }),
            }
        }

        if prefs.verify_sources {
            self.verify(&mut report.sources).await;
        }
        rank::rank(&mut report.sources, &prefs, kind, target);
        report
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
                self.ctx.cache.put(&key, &id, crate::cache::ttl::DETAIL).await;
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
    fn public_futures_are_send(e: &Engine, t: &TitleSummary, r: &ProviderRef, s: &mut Vec<Source>) {
        fn assert_send<T: Send>(_: T) {}
        assert_send(e.home());
        assert_send(e.search("x"));
        assert_send(e.detail(r));
        assert_send(e.sources(t, Target::Movie, AudioMode::Sub));
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
