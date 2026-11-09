//! Live scores, fixtures and results, plus "where to watch".
//!
//! Two free sources, merged: TheSportsDB (many sports) and football-data.org
//! (football, needs a token). Both free tiers run a few minutes behind, so a
//! [`SportsDay`] always says `delayed: true` and the UI should say so too.
//!
//! "Where to watch" only ever lists official broadcasters. A Play button
//! appears only for a free-to-air channel that someone listed by hand in the
//! registry; nothing here matches a match to a stream on its own.

pub mod footballdata;
pub mod thesportsdb;
pub mod watch;

use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use chrono::{Days, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::cache::ttl;
use crate::error::{Error, Result};
use crate::providers::Ctx;

use self::footballdata::FootballData;
use self::thesportsdb::TheSportsDb;
use self::watch::WatchOption;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SportsSource {
    TheSportsDb,
    FootballData,
}

impl SportsSource {
    /// Prefix of a [`Match::id`] from this source.
    pub fn prefix(self) -> &'static str {
        match self {
            Self::TheSportsDb => "tsdb",
            Self::FootballData => "fd",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchStatus {
    Upcoming,
    Live,
    Finished,
    Postponed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct League {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<String>,
    /// Country (or area) name as the source gives it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Team {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<String>,
}

/// One fixture, in the shape the app shows: sources differ, this doesn't.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Match {
    /// `tsdb:<id>` or `fd:<id>`; pass it to [`Sports::lookup`].
    pub id: String,
    pub source: SportsSource,
    pub sport: String,
    pub league: League,
    /// "Home vs Away", or the event name for sports without teams (racing, golf).
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home: Option<Team>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub away: Option<Team>,
    pub status: MatchStatus,
    /// Kickoff, unix seconds (UTC).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kickoff: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home_score: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub away_score: Option<u32>,
    /// Phase while live ("1H", "HT", "Q3") when the source says.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub venue: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub round: Option<String>,
}

/// One UTC day of matches, merged across sources.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SportsDay {
    pub date: String,
    pub matches: Vec<Match>,
    /// Sources that answered.
    pub sources: Vec<SportsSource>,
    /// Sources that failed, in plain words.
    pub errors: Vec<String>,
    /// Free tiers run a few minutes behind real time.
    pub delayed: bool,
}

/// A channel TheSportsDB lists as showing an event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TvListing {
    pub event_id: String,
    pub event: String,
    pub channel: String,
    /// Country name as TheSportsDB writes it ("United Kingdom").
    pub country: String,
    pub logo: Option<String>,
}

pub struct Sports {
    ctx: Arc<Ctx>,
    tsdb: TheSportsDb,
    fd: FootballData,
}

impl Sports {
    pub fn new(ctx: Arc<Ctx>) -> Self {
        Self { tsdb: TheSportsDb::new(ctx.clone()), fd: FootballData::new(ctx.clone()), ctx }
    }

    /// Sources that can answer with the current registry and keys.
    pub fn sources(&self) -> Vec<SportsSource> {
        let mut v = Vec::new();
        if self.fd.available() {
            v.push(SportsSource::FootballData);
        }
        v.push(SportsSource::TheSportsDb);
        v
    }

    /// Every match on one UTC day (`YYYY-MM-DD`), live ones first, then by kickoff.
    pub async fn day(&self, date: &str) -> Result<SportsDay> {
        let date = parse_date(date)?;
        let fd = async {
            if self.fd.available() {
                Some(self.fd.day(date).await)
            } else {
                None
            }
        };
        let (fd, tsdb) = futures::future::join(fd, self.tsdb.day(date)).await;

        let mut matches = Vec::new();
        let mut sources = Vec::new();
        let mut errors = Vec::new();
        match fd {
            Some(Ok(v)) => {
                sources.push(SportsSource::FootballData);
                matches.extend(v);
            }
            Some(Err(e)) => errors.push(format!("football-data.org: {e}")),
            None => {}
        }
        match tsdb {
            Ok(v) => {
                sources.push(SportsSource::TheSportsDb);
                matches.extend(v);
            }
            Err(e) => errors.push(format!("TheSportsDB: {e}")),
        }
        if sources.is_empty() {
            return Err(Error::Config(format!("no sports source answered: {}", errors.join("; "))));
        }
        let mut matches = dedupe(matches);
        order_day(&mut matches);
        Ok(SportsDay { date: ymd(date), matches, sources, errors, delayed: true })
    }

    /// [`Sports::day`] for today (UTC).
    pub async fn today(&self) -> Result<SportsDay> {
        self.day(&ymd(today())).await
    }

    /// Matches in play right now. Looks at yesterday too, for late-night games.
    pub async fn live(&self) -> Result<Vec<Match>> {
        let t = today();
        let mut out = self.collect(&[plus_days(t, -1), t], |m| m.status == MatchStatus::Live).await?;
        order_day(&mut out);
        Ok(out)
    }

    /// Matches still to come over the next `days` days (1 to 7), soonest first.
    pub async fn upcoming(&self, days: u32) -> Result<Vec<Match>> {
        let t = today();
        let dates: Vec<NaiveDate> = (0..days.clamp(1, 7)).map(|i| plus_days(t, i64::from(i))).collect();
        // A match that just kicked off can still read "not started" for a while.
        let cutoff = chrono::Utc::now().timestamp() - 2 * 3600;
        let mut out = self
            .collect(&dates, |m| m.status == MatchStatus::Upcoming && m.kickoff.is_none_or(|k| k >= cutoff))
            .await?;
        out.sort_by_key(|m| m.kickoff.unwrap_or(i64::MAX));
        Ok(out)
    }

    /// Finished matches from the last `days` days (1 to 7), newest first.
    pub async fn results(&self, days: u32) -> Result<Vec<Match>> {
        let t = today();
        let dates: Vec<NaiveDate> = (0..days.clamp(1, 7)).map(|i| plus_days(t, -i64::from(i))).collect();
        let mut out = self.collect(&dates, |m| m.status == MatchStatus::Finished).await?;
        out.sort_by_key(|m| std::cmp::Reverse(m.kickoff.unwrap_or(0)));
        Ok(out)
    }

    /// One match by its `tsdb:` or `fd:` id, refreshed from its source.
    pub async fn lookup(&self, id: &str) -> Result<Match> {
        match id.split_once(':') {
            Some(("tsdb", raw)) => self.tsdb.lookup(raw).await,
            Some(("fd", raw)) => self.fd.lookup(raw).await,
            _ => Err(Error::NotFound(format!("sports match id {id:?}"))),
        }
    }

    /// Official broadcasters for a match in a country (ISO 3166-1 alpha-2).
    /// Rules come from the registry; TheSportsDB's channel listing is added
    /// when it has one. See [`watch`] for why nothing here is ever matched to
    /// a stream automatically.
    pub async fn watch(&self, m: &Match, country: &str) -> Vec<WatchOption> {
        let reg = self.ctx.registry();
        let mut out = watch::from_rules(&reg.sports.watch, m, country);
        let listing = match m.kickoff.and_then(utc_date) {
            Some(date) => self.tsdb.tv(date).await.unwrap_or_default(),
            None => Vec::new(),
        };
        out.extend(watch::from_listing(&listing, m, country));
        let mut seen = HashSet::new();
        out.retain(|o| seen.insert(format!("{}|{}", o.name.to_lowercase(), o.country.to_lowercase())));
        out
    }

    /// Runs [`Sports::day`] for each date and keeps what `keep` accepts. Only
    /// fails when no day could be loaded at all.
    async fn collect(&self, dates: &[NaiveDate], keep: impl Fn(&Match) -> bool) -> Result<Vec<Match>> {
        let mut out = Vec::new();
        let mut loaded = false;
        let mut last_err = None;
        for d in dates {
            match self.day(&ymd(*d)).await {
                Ok(day) => {
                    loaded = true;
                    for m in day.matches {
                        if keep(&m) {
                            out.push(m);
                        }
                    }
                }
                Err(e) => last_err = Some(e),
            }
        }
        match (loaded, last_err) {
            (false, Some(e)) => Err(e),
            _ => Ok(out),
        }
    }
}

// ── Shared helpers ──────────────────────────────────────────────────────

pub(crate) fn parse_date(s: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").map_err(|e| Error::parse("date", format!("{s:?}: {e}")))
}

pub(crate) fn today() -> NaiveDate {
    chrono::Utc::now().date_naive()
}

pub(crate) fn ymd(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

pub(crate) fn plus_days(d: NaiveDate, n: i64) -> NaiveDate {
    let by = Days::new(n.unsigned_abs());
    let moved = if n >= 0 { d.checked_add_days(by) } else { d.checked_sub_days(by) };
    moved.unwrap_or(d)
}

/// The UTC date of a unix timestamp.
pub(crate) fn utc_date(ts: i64) -> Option<NaiveDate> {
    chrono::DateTime::from_timestamp(ts, 0).map(|d| d.date_naive())
}

/// How long to keep a day: today refreshes often, the past hardly at all.
pub(crate) fn day_ttl(date: NaiveDate) -> Duration {
    let t = today();
    if date == t {
        ttl::SPORTS_LIVE
    } else if date > t {
        ttl::SPORTS_SOON
    } else {
        ttl::SPORTS_PAST
    }
}

/// A team name reduced to letters and digits, without "FC", "AC" and the like,
/// so "Manchester United FC" and "Manchester United" compare equal.
pub(crate) fn norm_team(name: &str) -> String {
    const NOISE: [&str; 8] = ["fc", "cf", "afc", "sc", "ac", "ssc", "club", "the"];
    name.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty() && !NOISE.contains(t))
        .collect::<Vec<_>>()
        .join("")
}

/// Drops the same fixture reported by two sources; the first one wins, and
/// football-data.org is queried first.
fn dedupe(mut all: Vec<Match>) -> Vec<Match> {
    let mut seen = HashSet::new();
    all.retain(|m| match (&m.home, &m.away) {
        (Some(h), Some(a)) => {
            let day = m.kickoff.map(|k| k.div_euclid(86_400)).unwrap_or(0);
            seen.insert(format!("{day}|{}|{}", norm_team(&h.name), norm_team(&a.name)))
        }
        _ => true,
    });
    all
}

/// Live matches first, then by kickoff.
fn order_day(v: &mut [Match]) {
    v.sort_by_key(|m| (m.status != MatchStatus::Live, m.kickoff.unwrap_or(i64::MAX)));
}

/// Spends a rate budget: `max` calls per `window`.
pub(crate) struct Budget {
    max: usize,
    window: Duration,
    hits: Mutex<VecDeque<Instant>>,
}

impl Budget {
    pub fn new(max: usize, window: Duration) -> Self {
        Self { max, window, hits: Mutex::new(VecDeque::new()) }
    }

    /// Takes one call. `false` means the budget is spent for now.
    pub fn take(&self) -> bool {
        let Ok(mut hits) = self.hits.lock() else { return true };
        let now = Instant::now();
        while hits.front().is_some_and(|t| now.duration_since(*t) > self.window) {
            hits.pop_front();
        }
        if hits.len() >= self.max {
            return false;
        }
        hits.push_back(now);
        true
    }
}

#[cfg(test)]
pub(crate) fn sample(id: &str, home: &str, away: &str, kickoff: i64, status: MatchStatus) -> Match {
    Match {
        id: id.into(),
        source: SportsSource::TheSportsDb,
        sport: "Soccer".into(),
        league: League { id: "1".into(), name: "Test League".into(), badge: None, country: None },
        title: format!("{home} vs {away}"),
        home: Some(Team { name: home.into(), badge: None }),
        away: Some(Team { name: away.into(), badge: None }),
        status,
        kickoff: Some(kickoff),
        home_score: None,
        away_score: None,
        phase: None,
        venue: None,
        round: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn team_names_compare_across_sources() {
        assert_eq!(norm_team("Manchester United FC"), norm_team("Manchester United"));
        assert_eq!(norm_team("FC Bayern München"), norm_team("Bayern München"));
        assert_ne!(norm_team("Real Madrid"), norm_team("Real Betis"));
    }

    #[test]
    fn same_fixture_from_two_sources_is_kept_once() {
        let a = sample("fd:1", "Arsenal FC", "Chelsea FC", 1_800_000_000, MatchStatus::Upcoming);
        let b = sample("tsdb:9", "Arsenal", "Chelsea", 1_800_000_300, MatchStatus::Upcoming);
        let other_day = sample("tsdb:10", "Arsenal", "Chelsea", 1_800_000_000 + 86_400 * 3, MatchStatus::Upcoming);
        let out = dedupe(vec![a, b, other_day]);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].id, "fd:1", "the first source wins");
    }

    #[test]
    fn live_matches_sort_first_then_kickoff() {
        let mut v = vec![
            sample("c", "A", "B", 300, MatchStatus::Upcoming),
            sample("b", "C", "D", 200, MatchStatus::Upcoming),
            sample("a", "E", "F", 900, MatchStatus::Live),
        ];
        order_day(&mut v);
        let ids: Vec<_> = v.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(ids, ["a", "b", "c"]);
    }

    #[test]
    fn budget_runs_out_then_refuses() {
        let b = Budget::new(2, Duration::from_secs(3600));
        assert!(b.take());
        assert!(b.take());
        assert!(!b.take());
    }

    #[test]
    fn date_helpers() {
        let d = parse_date("2026-11-09").unwrap();
        assert_eq!(ymd(plus_days(d, 1)), "2026-11-10");
        assert_eq!(ymd(plus_days(d, -9)), "2026-10-31");
        assert!(parse_date("09/11/2026").is_err());
        assert_eq!(utc_date(0).map(ymd).as_deref(), Some("1970-01-01"));
    }

    #[test]
    fn today_refreshes_faster_than_the_past() {
        let t = today();
        assert!(day_ttl(t) < day_ttl(plus_days(t, 1)));
        assert!(day_ttl(plus_days(t, 1)) < day_ttl(plus_days(t, -1)));
    }
}
