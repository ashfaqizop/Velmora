//! Content providers. Each maps a site/API into the unified model.

pub mod hdhub;
pub mod hianime;
pub mod zokoanime;

use std::sync::{Arc, RwLock};

use async_trait::async_trait;

use crate::cache::Cache;
use crate::config::{Preferences, Registry};
use crate::error::{Error, Result};
use crate::model::{AudioMode, ExternalIds, MediaKind, ProviderId, Rail, Source, TitleDetail, TitleSummary, Target};
use crate::net::Http;
use crate::resolvers::WebSolver;

/// Shared state handed to every provider.
pub struct Ctx {
    pub http: Http,
    pub cache: Cache,
    pub web: Option<Arc<dyn WebSolver>>,
    registry: RwLock<Arc<Registry>>,
    prefs: RwLock<Arc<Preferences>>,
}

impl Ctx {
    pub fn new(http: Http, cache: Cache, registry: Registry, prefs: Preferences, web: Option<Arc<dyn WebSolver>>) -> Self {
        Self { http, cache, web, registry: RwLock::new(Arc::new(registry)), prefs: RwLock::new(Arc::new(prefs)) }
    }

    pub fn registry(&self) -> Arc<Registry> {
        self.registry.read().map(|r| Arc::clone(&r)).unwrap_or_else(|p| Arc::clone(&p.into_inner()))
    }

    pub fn set_registry(&self, reg: Registry) {
        if let Ok(mut r) = self.registry.write() {
            *r = Arc::new(reg);
        }
    }

    pub fn prefs(&self) -> Arc<Preferences> {
        self.prefs.read().map(|r| Arc::clone(&r)).unwrap_or_else(|p| Arc::clone(&p.into_inner()))
    }

    pub fn set_prefs(&self, prefs: Preferences) {
        if let Ok(mut p) = self.prefs.write() {
            *p = Arc::new(prefs);
        }
    }

    /// Configured base URL for a provider (no trailing slash).
    pub fn base(&self, p: ProviderId) -> Result<String> {
        self.registry()
            .mirror(p)
            .map(|m| m.trim_end_matches('/').to_string())
            .ok_or_else(|| Error::Config(format!("no mirror configured for {p}")))
    }
}

#[async_trait]
pub trait Provider: Send + Sync {
    fn id(&self) -> ProviderId;

    /// Content kinds this provider carries (used to route searches/sources).
    fn kinds(&self) -> &'static [MediaKind];

    /// Free-text search. Providers without search return an empty list.
    async fn search(&self, query: &str) -> Result<Vec<TitleSummary>>;

    async fn detail(&self, id: &str) -> Result<TitleDetail>;

    /// Sources for a title this provider returned from [`Provider::detail`].
    async fn sources(&self, detail: &TitleDetail, target: Target, mode: AudioMode) -> Result<Vec<Source>>;

    /// Sources from external ids alone (no search). `Ok(None)` = not supported.
    async fn sources_by_ids(
        &self,
        _ids: &ExternalIds,
        _kind: MediaKind,
        _target: Target,
        _mode: AudioMode,
    ) -> Result<Option<Vec<Source>>> {
        Ok(None)
    }

    /// Home-screen rails this provider contributes.
    async fn rails(&self) -> Result<Vec<Rail>> {
        Ok(Vec::new())
    }
}

/// Normalised title for fuzzy cross-provider matching.
pub fn norm_title(s: &str) -> String {
    let lower = s.to_lowercase();
    let cleaned: String = lower.chars().map(|c| if c.is_alphanumeric() { c } else { ' ' }).collect();
    cleaned
        .split_whitespace()
        .filter(|w| !matches!(*w, "the" | "a" | "an" | "season" | "movie" | "series" | "full"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Does `candidate` plausibly refer to the same title as `wanted`?
pub fn same_title(wanted: &TitleSummary, candidate: &TitleSummary) -> bool {
    if !wanted.ids.is_empty() && !candidate.ids.is_empty() {
        if wanted.ids.shares_any(&candidate.ids) {
            return true;
        }
        // Both carry the same id type but disagree → definitely different.
        if (wanted.ids.imdb.is_some() && candidate.ids.imdb.is_some())
            || (wanted.ids.mal.is_some() && candidate.ids.mal.is_some())
        {
            return false;
        }
    }
    let years_ok = match (wanted.year, candidate.year) {
        (Some(a), Some(b)) => a.abs_diff(b) <= 1,
        _ => true,
    };
    if !years_ok {
        return false;
    }
    let a = norm_title(&wanted.name);
    let b = norm_title(&candidate.name);
    if a.is_empty() || b.is_empty() {
        return false;
    }
    if a == b {
        return true;
    }
    let alt = wanted.original_name.as_deref().map(norm_title);
    if alt.as_deref() == Some(b.as_str()) {
        return true;
    }
    // "frieren beyond journey s end" vs "frieren beyond journeys end"
    a.replace(' ', "") == b.replace(' ', "")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(name: &str, year: Option<u16>) -> TitleSummary {
        let mut s = TitleSummary::new(ProviderId::HdHub, "x", name, MediaKind::Movie);
        s.year = year;
        s
    }

    #[test]
    fn title_matching() {
        assert!(same_title(&t("The Batman", Some(2022)), &t("Batman", Some(2022))));
        assert!(same_title(&t("Frieren: Beyond Journey's End", None), &t("Frieren Beyond Journeys End", None)));
        assert!(!same_title(&t("Batman", Some(1989)), &t("Batman", Some(2022))));
        let mut a = t("Dune", Some(2021));
        a.ids.imdb = Some("tt1160419".into());
        let mut b = t("Dune", Some(2021));
        b.ids.imdb = Some("tt0087182".into());
        assert!(!same_title(&a, &b), "conflicting imdb ids never merge");
    }
}
