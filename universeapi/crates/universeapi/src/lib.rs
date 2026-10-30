//! # UniverseAPI Engine
//!
//! The single backend behind Velmora: catalog, metadata, trending, streaming
//! and downloading, unified from ani-cli, MovieBox-TUI, freeflix-cli and the
//! HDHub custom API.
//!
//! ```no_run
//! # async fn demo() -> universeapi::Result<()> {
//! use universeapi::{Engine, EngineOptions, model::{Target, AudioMode}};
//! let engine = Engine::new(EngineOptions::default())?;
//! let home = engine.home().await;                      // trending rails
//! let hits = engine.search("frieren").await;           // merged across providers
//! let report = engine.sources(&hits[0], Target::Episode { season: 1, episode: 1 }, AudioMode::Sub).await;
//! println!("best: {:?}", report.sources.first().map(|s| &s.url));
//! # Ok(()) }
//! ```

pub mod cache;
pub mod config;
pub mod download;
pub mod engine;
pub mod error;
#[cfg(feature = "impersonate")]
pub mod impersonate;
pub mod metadata;
pub mod model;
pub mod net;
pub mod providers;
pub mod rank;
pub mod release;
pub mod resolvers;
#[cfg(feature = "torrent")]
pub mod torrent;
pub mod util;

pub use config::{Preferences, Registry};
pub use engine::{Engine, EngineOptions, ProviderFailure, SearchBatch, SourcesReport};
pub use error::{Error, Result};
