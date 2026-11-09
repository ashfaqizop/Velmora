//! TheSportsDB v1: many sports behind one free key. The free public test key
//! (`123`) works but is thin, so the owner's key goes in the registry
//! (`UNIVERSEAPI_SPORTSDB_KEY`). Point `sports.thesportsdb_url` at a caching
//! proxy to keep the key out of the app.

use std::sync::Arc;
use std::time::Duration;

use chrono::NaiveDate;
use futures::stream::{self, StreamExt};
use serde::Deserialize;
use serde_json::Value;

use super::{day_ttl, ymd, League, Match, MatchStatus, SportsSource, Team, TvListing};
use crate::cache::ttl;
use crate::error::{Error, Result};
use crate::providers::Ctx;

const TEST_KEY: &str = "123";

pub struct TheSportsDb {
    ctx: Arc<Ctx>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Events {
    #[serde(default, deserialize_with = "crate::util::null_default")]
    events: Vec<Event>,
}

/// An event as TheSportsDB sends it. Ids and counts arrive as strings, numbers
/// or null depending on the sport, so those stay raw JSON until read.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Event {
    #[serde(rename = "idEvent")]
    id: Option<Value>,
    #[serde(rename = "strEvent")]
    event: Option<String>,
    #[serde(rename = "strSport")]
    sport: Option<String>,
    #[serde(rename = "idLeague")]
    league_id: Option<Value>,
    #[serde(rename = "strLeague")]
    league: Option<String>,
    #[serde(rename = "strLeagueBadge")]
    league_badge: Option<String>,
    #[serde(rename = "strHomeTeam")]
    home: Option<String>,
    #[serde(rename = "strAwayTeam")]
    away: Option<String>,
    #[serde(rename = "strHomeTeamBadge")]
    home_badge: Option<String>,
    #[serde(rename = "strAwayTeamBadge")]
    away_badge: Option<String>,
    #[serde(rename = "intHomeScore")]
    home_score: Option<Value>,
    #[serde(rename = "intAwayScore")]
    away_score: Option<Value>,
    #[serde(rename = "intRound")]
    round: Option<Value>,
    #[serde(rename = "strTimestamp")]
    timestamp: Option<String>,
    #[serde(rename = "dateEvent")]
    date: Option<String>,
    #[serde(rename = "strTime")]
    time: Option<String>,
    #[serde(rename = "strStatus")]
    status: Option<String>,
    #[serde(rename = "strPostponed")]
    postponed: Option<String>,
    #[serde(rename = "strVenue")]
    venue: Option<String>,
    #[serde(rename = "strCountry")]
    country: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct TvEvents {
    #[serde(default, deserialize_with = "crate::util::null_default")]
    tvevents: Vec<TvEvent>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct TvEvent {
    #[serde(rename = "idEvent")]
    id: Option<Value>,
    #[serde(rename = "strEvent")]
    event: Option<String>,
    #[serde(rename = "strChannel")]
    channel: Option<String>,
    #[serde(rename = "strCountry")]
    country: Option<String>,
    #[serde(rename = "strLogo")]
    logo: Option<String>,
}

impl TheSportsDb {
    pub fn new(ctx: Arc<Ctx>) -> Self {
        Self { ctx }
    }

    /// `<base>/<key>`; every endpoint hangs off it.
    fn base(&self) -> String {
        let reg = self.ctx.registry();
        let key = reg.thesportsdb_key.as_deref().map(str::trim).filter(|k| !k.is_empty()).unwrap_or(TEST_KEY);
        format!("{}/{key}", reg.sports.thesportsdb_url.trim_end_matches('/'))
    }

    /// All configured sports for one UTC day. A sport that fails is skipped;
    /// the day only fails when every sport did.
    pub async fn day(&self, date: NaiveDate) -> Result<Vec<Match>> {
        let sports = self.ctx.registry().sports.sports.clone();
        let day = ymd(date);
        let keep_for = day_ttl(date);
        let results: Vec<Result<Vec<Match>>> =
            stream::iter(sports).map(|sport| self.day_sport(&day, sport, keep_for)).buffer_unordered(3).collect().await;

        let mut out = Vec::new();
        let mut answered = false;
        let mut first_err = None;
        for r in results {
            match r {
                Ok(v) => {
                    answered = true;
                    out.extend(v);
                }
                Err(e) => {
                    tracing::warn!("TheSportsDB {day}: {e}");
                    if first_err.is_none() {
                        first_err = Some(e);
                    }
                }
            }
        }
        match (answered, first_err) {
            (false, Some(e)) => Err(e),
            _ => Ok(out),
        }
    }

    async fn day_sport(&self, day: &str, sport: String, keep_for: Duration) -> Result<Vec<Match>> {
        let key = format!("sports:tsdb:{day}:{sport}");
        if let Some(hit) = self.ctx.cache.get::<Vec<Match>>(&key).await {
            return Ok(hit);
        }
        let url = format!("{}/eventsday.php", self.base());
        let resp: Events = self.ctx.http.get_json_query(&url, &[("d", day), ("s", sport.as_str())], &[]).await?;
        let now = chrono::Utc::now().timestamp();
        let out: Vec<Match> = resp.events.into_iter().filter_map(|e| to_match(e, now)).collect();
        self.ctx.cache.put(&key, &out, keep_for).await;
        Ok(out)
    }

    /// One event, refreshed.
    pub async fn lookup(&self, id: &str) -> Result<Match> {
        let url = format!("{}/lookupevent.php", self.base());
        let resp: Events = self.ctx.http.get_json_query(&url, &[("id", id)], &[]).await?;
        let now = chrono::Utc::now().timestamp();
        resp.events
            .into_iter()
            .find_map(|e| to_match(e, now))
            .ok_or_else(|| Error::NotFound(format!("TheSportsDB event {id}")))
    }

    /// Channels listed for events on a day. Thin on the free key.
    pub async fn tv(&self, date: NaiveDate) -> Result<Vec<TvListing>> {
        let day = ymd(date);
        let key = format!("sports:tsdb:tv:{day}");
        if let Some(hit) = self.ctx.cache.get::<Vec<TvListing>>(&key).await {
            return Ok(hit);
        }
        let url = format!("{}/eventstv.php", self.base());
        let resp: TvEvents = self.ctx.http.get_json_query(&url, &[("d", day.as_str())], &[]).await?;
        let out = tv_listings(resp);
        self.ctx.cache.put(&key, &out, ttl::SPORTS_PAST).await;
        Ok(out)
    }
}

fn tv_listings(resp: TvEvents) -> Vec<TvListing> {
    resp.tvevents
        .into_iter()
        .filter_map(|t| {
            Some(TvListing {
                event_id: text(&t.id)?,
                event: clean(&t.event).unwrap_or_default(),
                channel: clean(&t.channel)?,
                country: clean(&t.country).unwrap_or_default(),
                logo: clean(&t.logo),
            })
        })
        .collect()
}

// ── Mapping ─────────────────────────────────────────────────────────────

/// A JSON value as text: strings and numbers, nothing else.
fn text(v: &Option<Value>) -> Option<String> {
    match v {
        Some(Value::String(s)) if !s.trim().is_empty() => Some(s.trim().to_string()),
        Some(Value::Number(n)) => Some(n.to_string()),
        _ => None,
    }
}

fn count(v: &Option<Value>) -> Option<u32> {
    text(v).and_then(|s| s.parse().ok())
}

fn clean(s: &Option<String>) -> Option<String> {
    s.as_deref().map(str::trim).filter(|s| !s.is_empty()).map(str::to_string)
}

/// Kickoff as unix seconds (TheSportsDB times are UTC).
fn kickoff(e: &Event) -> Option<i64> {
    let parse = |s: &str, fmt: &str| chrono::NaiveDateTime::parse_from_str(s, fmt).ok().map(|d| d.and_utc().timestamp());
    if let Some(t) = clean(&e.timestamp).and_then(|s| parse(&s, "%Y-%m-%dT%H:%M:%S")) {
        return Some(t);
    }
    let date = clean(&e.date)?;
    let time = clean(&e.time).unwrap_or_else(|| "00:00:00".into());
    parse(&format!("{date} {time}"), "%Y-%m-%d %H:%M:%S")
}

/// Maps TheSportsDB's status codes to ours. It mixes short codes ("FT", "1H")
/// with phrases ("Match Finished"), and a lagging feed can leave a finished
/// game on "NS", so a score long after kickoff counts as finished.
pub(crate) fn map_status(raw: Option<&str>, postponed: bool, kickoff: Option<i64>, scored: bool, now: i64) -> MatchStatus {
    let s = raw.unwrap_or("").trim().to_ascii_uppercase();
    if postponed || matches!(s.as_str(), "PST" | "POSTPONED" | "SUSP" | "SUSPENDED") {
        return MatchStatus::Postponed;
    }
    if matches!(s.as_str(), "CANC" | "CANCELLED" | "CANCELED" | "ABD" | "ABANDONED" | "WO") {
        return MatchStatus::Cancelled;
    }
    if matches!(
        s.as_str(),
        "FT" | "AET" | "PEN" | "AOT" | "AP" | "FIN" | "FINISHED" | "MATCH FINISHED" | "AFTER OVERTIME" | "AFTER EXTRA TIME" | "AFTER PENALTIES"
    ) {
        return MatchStatus::Finished;
    }
    if matches!(
        s.as_str(),
        "1H" | "2H" | "HT" | "ET" | "BT" | "P" | "LIVE" | "INT" | "OT" | "SO" | "Q1" | "Q2" | "Q3" | "Q4" | "P1" | "P2" | "P3" | "IN PROGRESS" | "HALFTIME"
    ) {
        return MatchStatus::Live;
    }
    let long_ago = kickoff.is_some_and(|k| now - k > 4 * 3600);
    if scored && long_ago {
        MatchStatus::Finished
    } else {
        MatchStatus::Upcoming
    }
}

fn to_match(e: Event, now: i64) -> Option<Match> {
    let id = text(&e.id)?;
    let title = clean(&e.event).or_else(|| match (clean(&e.home), clean(&e.away)) {
        (Some(h), Some(a)) => Some(format!("{h} vs {a}")),
        _ => None,
    })?;
    let kick = kickoff(&e);
    let (home_score, away_score) = (count(&e.home_score), count(&e.away_score));
    let postponed = e.postponed.as_deref().is_some_and(|p| p.eq_ignore_ascii_case("yes"));
    let raw_status = clean(&e.status);
    let status = map_status(raw_status.as_deref(), postponed, kick, home_score.is_some() || away_score.is_some(), now);
    let team = |name: &Option<String>, badge: &Option<String>| clean(name).map(|name| Team { name, badge: clean(badge) });
    Some(Match {
        id: format!("{}:{id}", SportsSource::TheSportsDb.prefix()),
        source: SportsSource::TheSportsDb,
        sport: clean(&e.sport).unwrap_or_default(),
        league: League {
            id: text(&e.league_id).unwrap_or_default(),
            name: clean(&e.league).unwrap_or_default(),
            badge: clean(&e.league_badge),
            country: clean(&e.country),
        },
        title,
        home: team(&e.home, &e.home_badge),
        away: team(&e.away, &e.away_badge),
        status,
        kickoff: kick,
        home_score,
        away_score,
        phase: if status == MatchStatus::Live { raw_status } else { None },
        venue: clean(&e.venue),
        round: text(&e.round),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Trimmed from a real eventsday.php response.
    const DAY: &str = r#"{"events":[
        {"idEvent":"2420125","strTimestamp":"2026-11-09T16:00:00","strEvent":"Iberia 1999 vs Gagra","strSport":"Soccer",
         "idLeague":"4638","strLeague":"Georgian Erovnuli Liga","strLeagueBadge":"https://r2.thesportsdb.com/l.png",
         "strHomeTeam":"Iberia 1999","strAwayTeam":"Gagra","intHomeScore":null,"intAwayScore":null,"intRound":"32",
         "dateEvent":"2026-11-09","strTime":"16:00:00","strHomeTeamBadge":"https://r2.thesportsdb.com/h.png",
         "strAwayTeamBadge":"","strVenue":"Mikheil Meskhi Stadium","strStatus":"NS","strPostponed":"no","strCountry":"Georgia"},
        {"idEvent":"2475508","strTimestamp":"2026-11-09T01:20:00","strEvent":"Chicago Bears vs Tampa Bay Buccaneers",
         "strSport":"American Football","idLeague":"4391","strLeague":"NFL","strHomeTeam":"Chicago Bears",
         "strAwayTeam":"Tampa Bay Buccaneers","intHomeScore":"24","intAwayScore":"20","strStatus":"FT","strPostponed":"no"},
        {"idEvent":77,"strEvent":"Brazilian Grand Prix","strSport":"Motorsport","strHomeTeam":null,"strAwayTeam":null,
         "dateEvent":"2026-11-09","strTime":"17:00:00","strStatus":"1H","strPostponed":"no"}
    ]}"#;

    fn parsed() -> Vec<Match> {
        let resp: Events = serde_json::from_str(DAY).unwrap();
        resp.events.into_iter().filter_map(|e| to_match(e, 1_793_000_000)).collect()
    }

    #[test]
    fn parses_a_real_day() {
        let m = parsed();
        assert_eq!(m.len(), 3);

        let a = &m[0];
        assert_eq!(a.id, "tsdb:2420125");
        assert_eq!(a.status, MatchStatus::Upcoming);
        assert_eq!(a.title, "Iberia 1999 vs Gagra");
        assert_eq!(a.home.as_ref().unwrap().name, "Iberia 1999");
        assert_eq!(a.away.as_ref().unwrap().badge, None, "an empty badge is no badge");
        assert_eq!(a.kickoff, Some(1_794_240_000), "2026-11-09 16:00 UTC");
        assert_eq!(a.round.as_deref(), Some("32"));
        assert_eq!(a.league.name, "Georgian Erovnuli Liga");
        assert_eq!(a.venue.as_deref(), Some("Mikheil Meskhi Stadium"));

        let b = &m[1];
        assert_eq!((b.status, b.home_score, b.away_score), (MatchStatus::Finished, Some(24), Some(20)));

        // No teams (racing) still shows as an event, and a numeric id is fine.
        let c = &m[2];
        assert_eq!((c.id.as_str(), c.title.as_str()), ("tsdb:77", "Brazilian Grand Prix"));
        assert!(c.home.is_none() && c.away.is_none());
        assert_eq!(c.status, MatchStatus::Live);
        assert_eq!(c.phase.as_deref(), Some("1H"));
        assert_eq!(c.kickoff, Some(1_794_243_600), "falls back to date + time");
    }

    #[test]
    fn null_events_means_none() {
        let resp: Events = serde_json::from_str(r#"{"events":null}"#).unwrap();
        assert!(resp.events.is_empty());
    }

    #[test]
    fn status_codes_map() {
        let now = 1_000_000;
        let s = |raw: &str| map_status(Some(raw), false, Some(now), false, now);
        assert_eq!(s("NS"), MatchStatus::Upcoming);
        assert_eq!(s("Match Finished"), MatchStatus::Finished);
        assert_eq!(s("AET"), MatchStatus::Finished);
        assert_eq!(s("HT"), MatchStatus::Live);
        assert_eq!(s("Q3"), MatchStatus::Live);
        assert_eq!(s("PST"), MatchStatus::Postponed);
        assert_eq!(s("Cancelled"), MatchStatus::Cancelled);
        assert_eq!(map_status(Some("NS"), true, Some(now), false, now), MatchStatus::Postponed);
        // A feed that never moved off "NS" but has a score long after kickoff.
        assert_eq!(map_status(Some("NS"), false, Some(now - 6 * 3600), true, now), MatchStatus::Finished);
        assert_eq!(map_status(None, false, None, false, now), MatchStatus::Upcoming);
    }

    #[test]
    fn tv_listing_parses() {
        let resp: TvEvents = serde_json::from_str(
            r#"{"tvevents":[{"idEvent":"2475508","strEvent":"Chicago Bears vs Tampa Bay Buccaneers",
            "strChannel":"Sky Sports NFL","strCountry":"United Kingdom","strLogo":"https://r2.thesportsdb.com/c.png"},
            {"idEvent":"1","strChannel":""}]}"#,
        )
        .unwrap();
        let l = tv_listings(resp);
        assert_eq!(l.len(), 1, "a row without a channel is dropped");
        assert_eq!((l[0].channel.as_str(), l[0].country.as_str()), ("Sky Sports NFL", "United Kingdom"));
    }
}
