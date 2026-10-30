//! Direct ZokoAnime access by MyAnimeList id — the fast path for anime.
//! AniList supplies the MAL id, so no hianime scraping is needed at all.

use std::sync::Arc;

use async_trait::async_trait;

use super::{Ctx, Provider};
use crate::error::{Error, Result};
use crate::model::*;
use crate::resolvers::zoko;

pub struct ZokoAnime {
    ctx: Arc<Ctx>,
}

impl ZokoAnime {
    pub fn new(ctx: Arc<Ctx>) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl Provider for ZokoAnime {
    fn id(&self) -> ProviderId {
        ProviderId::ZokoAnime
    }

    fn kinds(&self) -> &'static [MediaKind] {
        &[MediaKind::Anime]
    }

    async fn search(&self, _query: &str) -> Result<Vec<TitleSummary>> {
        Ok(Vec::new())
    }

    async fn detail(&self, _id: &str) -> Result<TitleDetail> {
        Err(Error::Unsupported("ZokoAnime is id-addressed; use sources_by_ids".into()))
    }

    async fn sources(&self, detail: &TitleDetail, target: Target, mode: AudioMode) -> Result<Vec<Source>> {
        self.sources_by_ids(&detail.summary.ids, detail.summary.kind, target, mode)
            .await?
            .ok_or_else(|| Error::NotFound("title has no MAL id".into()))
    }

    async fn sources_by_ids(
        &self,
        ids: &ExternalIds,
        kind: MediaKind,
        target: Target,
        mode: AudioMode,
    ) -> Result<Option<Vec<Source>>> {
        let Some(mal) = ids.mal else { return Ok(None) };
        if kind != MediaKind::Anime {
            return Ok(None);
        }
        let episode = match target {
            Target::Episode { episode, .. } => episode,
            Target::Movie => 1,
            Target::Season { .. } => return Ok(None),
        };
        let base = self.ctx.base(ProviderId::ZokoAnime)?;
        let embed = zoko::embed_url(&base, mal, episode, mode);
        let label = format!("Episode {episode} · {}", if mode == AudioMode::Dub { "DUB" } else { "SUB" });
        zoko::resolve_embed(&self.ctx.http, ProviderId::ZokoAnime, &embed, &label).await.map(Some)
    }
}
