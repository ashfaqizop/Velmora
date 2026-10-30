//! `universeapi` — developer & CI front-end for the UniverseAPI Engine.
//!
//! ```text
//! universeapi home
//! universeapi search "frieren"
//! universeapi sources "frieren" --season 1 --episode 1
//! universeapi smoke --report smoke.json      # live provider health (CI)
//! ```

mod smoke;

use std::path::PathBuf;

use anyhow::{bail, Context};
use clap::{Parser, Subcommand};
use universeapi::model::{AudioMode, ProviderId, ProviderRef, Target};
use universeapi::{Engine, EngineOptions, Registry};

#[derive(Parser)]
#[command(name = "universeapi", version, about = "UniverseAPI Engine CLI")]
struct Cli {
    /// Remote registry.json to apply over defaults (URL).
    #[arg(long, global = true)]
    registry: Option<String>,
    /// Print raw JSON instead of a summary.
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Home feed rails.
    Home,
    /// Merged search across all providers.
    Search { query: String },
    /// Provider detail by id, e.g. `detail hd_hub akka-season-1-hindi-webrip-full-series`.
    Detail { provider: String, id: String },
    /// Search, take the best match, and list ranked sources.
    Sources {
        query: String,
        #[arg(long)]
        season: Option<u32>,
        #[arg(long)]
        episode: Option<u32>,
        /// Whole-season packs instead of a single episode.
        #[arg(long)]
        pack: bool,
        #[arg(long)]
        dub: bool,
        /// Index into the merged search results.
        #[arg(long, default_value_t = 0)]
        pick: usize,
    },
    /// Live smoke test of every provider and resolver.
    Smoke {
        /// Write a JSON report here.
        #[arg(long)]
        report: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()))
        .with_writer(std::io::stderr)
        .init();
    let cli = Cli::parse();

    let engine = Engine::new(EngineOptions { registry: Registry::default().with_env_overrides(), ..Default::default() })?;
    if let Some(url) = &cli.registry {
        engine.refresh_registry(url).await.context("loading remote registry")?;
        engine.set_registry((*engine.registry()).clone().with_env_overrides());
    }

    match cli.cmd {
        Cmd::Home => {
            let rails = engine.home().await;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&rails)?);
            } else {
                for r in rails {
                    println!("▸ {} ({} items)", r.title, r.items.len());
                    for i in r.items.iter().take(5) {
                        println!("    {} {}", i.name, i.year.map(|y| format!("({y})")).unwrap_or_default());
                    }
                }
            }
        }
        Cmd::Search { query } => {
            let hits = engine.search(&query).await;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&hits)?);
            } else {
                for (i, h) in hits.iter().enumerate() {
                    let on: Vec<String> = std::iter::once(h.origin.provider)
                        .chain(h.also_on.iter().map(|r| r.provider))
                        .map(|p| p.to_string())
                        .collect();
                    println!(
                        "{i:>2}. {} {} [{:?}] — {}  {}",
                        h.name,
                        h.year.map(|y| format!("({y})")).unwrap_or_default(),
                        h.kind,
                        on.join(", "),
                        h.badges.join(" · ")
                    );
                }
            }
        }
        Cmd::Detail { provider, id } => {
            let provider: ProviderId = serde_json::from_value(serde_json::Value::String(provider.clone()))
                .with_context(|| format!("unknown provider {provider}"))?;
            let d = engine.detail(&ProviderRef { provider, id }).await?;
            println!("{}", serde_json::to_string_pretty(&d)?);
        }
        Cmd::Sources { query, season, episode, pack, dub, pick } => {
            let hits = engine.search(&query).await;
            let Some(title) = hits.get(pick) else { bail!("no search result #{pick} for {query:?}") };
            let target = match (season, episode, pack) {
                (Some(s), _, true) => Target::Season { season: s },
                (s, Some(e), _) => Target::Episode { season: s.unwrap_or(1), episode: e },
                _ if title.kind.is_episodic() => Target::Episode { season: season.unwrap_or(1), episode: 1 },
                _ => Target::Movie,
            };
            let mode = if dub { AudioMode::Dub } else { AudioMode::Sub };
            eprintln!("→ {} ({:?}) {:?}", title.name, title.kind, target);
            let report = engine.sources(title, target, mode).await;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                for s in &report.sources {
                    println!(
                        "{:>5}  {:<10} {:<14} {:<6?} {:<40} {}",
                        s.score,
                        s.provider.to_string(),
                        s.host,
                        s.format,
                        s.tags.badges().join(" "),
                        if s.verified == Some(true) { "✓" } else { "?" }
                    );
                }
                for e in &report.errors {
                    eprintln!("  ✗ {}: {}", e.provider, e.message);
                }
            }
        }
        Cmd::Smoke { report } => {
            let result = smoke::run(&engine).await;
            smoke::print(&result);
            if let Some(path) = report {
                std::fs::write(&path, serde_json::to_vec_pretty(&result)?)?;
            }
            if result.critical_failures() > 0 {
                std::process::exit(1);
            }
        }
    }
    Ok(())
}
