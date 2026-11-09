//! `universeapi` — developer & CI front-end for the UniverseAPI Engine.
//!
//! ```text
//! universeapi home
//! universeapi search "frieren"
//! universeapi sources "frieren" --season 1 --episode 1
//! universeapi sports live | upcoming | results | day 2026-11-09 | watch tsdb:123 --country US
//! universeapi tv channels --country US --category sports --q espn
//! universeapi smoke --report smoke.json      # live provider health (CI)
//! ```

mod smoke;

use std::path::PathBuf;

use anyhow::{bail, Context};
use clap::{Parser, Subcommand};
use universeapi::model::{AudioMode, ProviderId, ProviderRef, Target};
use universeapi::sports::{Match, MatchStatus};
use universeapi::tv::TvQuery;
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
    /// Scores, fixtures and results.
    Sports {
        #[command(subcommand)]
        what: SportsCmd,
    },
    /// Live TV channels from the iptv-org playlist.
    Tv {
        #[command(subcommand)]
        what: TvCmd,
    },
    /// Live smoke test of every provider and resolver.
    Smoke {
        /// Write a JSON report here.
        #[arg(long)]
        report: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum SportsCmd {
    /// Matches on one UTC day (default: today).
    Day { date: Option<String> },
    /// Matches in play right now.
    Live,
    /// Matches still to come.
    Upcoming {
        #[arg(long, default_value_t = 2)]
        days: u32,
    },
    /// Finished matches, newest first.
    Results {
        #[arg(long, default_value_t = 2)]
        days: u32,
    },
    /// Where to watch a match id (`tsdb:123` or `fd:456`).
    Watch {
        id: String,
        /// ISO country code.
        #[arg(long, default_value = "US")]
        country: String,
    },
}

#[derive(Subcommand)]
enum TvCmd {
    /// Search the channel library.
    Channels {
        #[arg(long)]
        country: Option<String>,
        #[arg(long)]
        category: Option<String>,
        /// Text to find in channel and network names.
        #[arg(long)]
        q: Option<String>,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Countries with channels.
    Countries,
    /// Categories with channels.
    Categories,
    /// Probe a channel's streams.
    Check { id: String },
}

fn print_matches(matches: &[Match]) {
    for m in matches {
        let at = m
            .kickoff
            .map(|k| format!("{:02}:{:02}", k.rem_euclid(86_400) / 3600, k.rem_euclid(3600) / 60))
            .unwrap_or_else(|| "--:--".into());
        let status = match m.status {
            MatchStatus::Upcoming => "soon",
            MatchStatus::Live => "LIVE",
            MatchStatus::Finished => "done",
            MatchStatus::Postponed => "post",
            MatchStatus::Cancelled => "canc",
        };
        let score = match (m.home_score, m.away_score) {
            (Some(h), Some(a)) => format!("{h}-{a}"),
            _ => "   ".into(),
        };
        println!("  {at} UTC  {status:<5} {score:<5} {}  [{} / {}]  {}", m.title, m.sport, m.league.name, m.id);
    }
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
        Cmd::Sports { what } => match what {
            SportsCmd::Day { date } => {
                let sports = engine.sports();
                let day = match date {
                    Some(d) => sports.day(&d).await?,
                    None => sports.today().await?,
                };
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&day)?);
                } else {
                    println!("{} · {} matches · from {:?}", day.date, day.matches.len(), day.sources);
                    print_matches(&day.matches);
                    for e in &day.errors {
                        eprintln!("  ✗ {e}");
                    }
                }
            }
            SportsCmd::Live => {
                let m = engine.sports().live().await?;
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&m)?);
                } else {
                    println!("{} live", m.len());
                    print_matches(&m);
                }
            }
            SportsCmd::Upcoming { days } => {
                let m = engine.sports().upcoming(days).await?;
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&m)?);
                } else {
                    println!("{} upcoming", m.len());
                    print_matches(&m);
                }
            }
            SportsCmd::Results { days } => {
                let m = engine.sports().results(days).await?;
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&m)?);
                } else {
                    println!("{} results", m.len());
                    print_matches(&m);
                }
            }
            SportsCmd::Watch { id, country } => {
                let m = engine.sports().lookup(&id).await?;
                let options = engine.sports_watch(&m, &country).await;
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&options)?);
                } else {
                    print_matches(std::slice::from_ref(&m));
                    if options.is_empty() {
                        println!("  no official broadcaster listed for {country}");
                    }
                    for o in &options {
                        let play = if o.play_channel.is_some() { "  [free channel]" } else { "" };
                        println!("  {:<24} {:?}  {}{play}", o.name, o.kind, o.country);
                    }
                }
            }
        },
        Cmd::Tv { what } => match what {
            TvCmd::Channels { country, category, q, limit } => {
                let page = engine.tv().channels(&TvQuery { country, category, text: q, offset: 0, limit }).await?;
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&page)?);
                } else {
                    println!("{} channels, showing {}", page.total, page.items.len());
                    for c in &page.items {
                        println!(
                            "  {:<28} {:<3} {:<14} {} streams  {}",
                            c.name,
                            c.country.as_deref().unwrap_or("-"),
                            c.categories.first().map(String::as_str).unwrap_or("-"),
                            c.streams.len(),
                            c.id
                        );
                    }
                }
            }
            TvCmd::Countries => {
                let facets = engine.tv().countries().await?;
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&facets)?);
                } else {
                    for f in facets.iter().take(40) {
                        println!("  {:>5}  {:<5} {}", f.count, f.id, f.name);
                    }
                }
            }
            TvCmd::Categories => {
                let facets = engine.tv().categories().await?;
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&facets)?);
                } else {
                    for f in facets.iter().take(40) {
                        println!("  {:>5}  {:<12} {}", f.count, f.id, f.name);
                    }
                }
            }
            TvCmd::Check { id } => {
                let health = engine.tv().check(&id).await?;
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&health)?);
                } else {
                    for h in &health {
                        let mark = if h.ok { "ok " } else { "bad" };
                        println!("  {mark} {:>3}  {}  {}", h.status, h.url, h.detail.as_deref().unwrap_or(""));
                    }
                }
            }
        },
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
