//! football-data.org v4. The free tier covers a dozen competitions with
//! delayed scores and allows 10 calls a minute, so calls are budgeted and every
//! day is cached. It needs a token (`UNIVERSEAPI_FOOTBALL_DATA_TOKEN`), unless
//! `sports.football_data_url` points at a proxy that holds it.

use std::sync::Arc;
use std::time::Duration;

use chrono::NaiveDate;
use serde::de::DeserializeOwned;
use serde::Deserialize;

use super::{day_ttl, plus_days, utc_date, ymd, Budget, League, Match, MatchStatus, SportsSource, Team};
use crate::config::FOOTBALL_DATA_URL;
use crate::error::{Error, Result};
use crate::providers::Ctx;

pub struct FootballData {
    ctx: Arc<Ctx>,
    budget: Budget,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Matches {
    #[serde(default, deserialize_with = "crate::util::null_default")]
    matches: Vec<Fd>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Fd {
    id: u64,
    utc_date: Option<String>,
    status: Option<String>,
    matchday: Option<u32>,
    stage: Option<String>,
    venue: Option<String>,
    competition: Comp,
    area: Area,
    home_team: Side,
    away_team: Side,
    score: Score,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Comp {
    id: Option<u64>,
    name: Option<String>,
    emblem: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Area {
    name: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Side {
    name: Option<String>,
    crest: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Score {
    full_time: Pair,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Pair {
    home: Option<u32>,
    away: Option<u32>,
}

impl FootballData {
    pub fn new(ctx: Arc<Ctx>) -> Self {
        // The free tier allows 10 a minute; leave some headroom.
        Self { ctx, budget: Budget::new(8, Duration::from_secs(60)) }
    }

    /// True with a token, or when the URL points at a proxy that holds it.
    pub fn available(&self) -> bool {
        let reg = self.ctx.registry();
        reg.football_data_token.as_deref().is_some_and(|t| !t.trim().is_empty())
            || reg.sports.football_data_url.trim_end_matches('/') != FOOTBALL_DATA_URL
    }

    async fn get<T: DeserializeOwned>(&self, path_and_query: &str) -> Result<T> {
        if !self.budget.take() {
            return Err(Error::Unsupported("football-data.org allows 10 calls a minute; try again shortly".into()));
        }
        let reg = self.ctx.registry();
        let url = format!("{}{path_and_query}", reg.sports.football_data_url.trim_end_matches('/'));
        let token = reg.football_data_token.clone().unwrap_or_default();
        let mut headers: Vec<(&str, &str)> = Vec::new();
        if !token.trim().is_empty() {
            headers.push(("X-Auth-Token", token.trim()));
        }
        self.ctx.http.get_json(&url, &headers).await
    }

    /// Matches on one UTC day. The API takes a date range, so this asks for
    /// that day and the next and keeps only the day wanted.
    pub async fn day(&self, date: NaiveDate) -> Result<Vec<Match>> {
        let day = ymd(date);
        let key = format!("sports:fd:{day}");
        if let Some(hit) = self.ctx.cache.get::<Vec<Match>>(&key).await {
            return Ok(hit);
        }
        let to = ymd(plus_days(date, 1));
        let resp: Matches = self.get(&format!("/matches?dateFrom={day}&dateTo={to}")).await?;
        let out: Vec<Match> = resp
            .matches
            .into_iter()
            .filter_map(to_match)
            .filter(|m| m.kickoff.and_then(utc_date) == Some(date))
            .collect();
        self.ctx.cache.put(&key, &out, day_ttl(date)).await;
        Ok(out)
    }

    /// One match, refreshed.
    pub async fn lookup(&self, id: &str) -> Result<Match> {
        if id.is_empty() || !id.chars().all(|c| c.is_ascii_digit()) {
            return Err(Error::NotFound(format!("football-data.org match {id:?}")));
        }
        let fd: Fd = self.get(&format!("/matches/{id}")).await?;
        to_match(fd).ok_or_else(|| Error::NotFound(format!("football-data.org match {id}")))
    }
}

fn map_status(raw: Option<&str>) -> MatchStatus {
    match raw.unwrap_or("").to_ascii_uppercase().as_str() {
        "IN_PLAY" | "PAUSED" | "LIVE" | "EXTRA_TIME" | "PENALTY_SHOOTOUT" => MatchStatus::Live,
        "FINISHED" | "AWARDED" => MatchStatus::Finished,
        "POSTPONED" | "SUSPENDED" => MatchStatus::Postponed,
        "CANCELLED" => MatchStatus::Cancelled,
        _ => MatchStatus::Upcoming,
    }
}

fn phase(raw: Option<&str>) -> Option<String> {
    let label = match raw.unwrap_or("").to_ascii_uppercase().as_str() {
        "IN_PLAY" => "Live",
        "PAUSED" => "HT",
        "EXTRA_TIME" => "ET",
        "PENALTY_SHOOTOUT" => "Pens",
        _ => return None,
    };
    Some(label.to_string())
}

fn clean(s: Option<String>) -> Option<String> {
    s.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn to_match(m: Fd) -> Option<Match> {
    if m.id == 0 {
        return None;
    }
    // A knockout fixture whose teams aren't decided yet has no names; skip it.
    let home = clean(m.home_team.name)?;
    let away = clean(m.away_team.name)?;
    let kickoff = m.utc_date.as_deref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()).map(|d| d.timestamp());
    let status = map_status(m.status.as_deref());
    let round = match (m.matchday, clean(m.stage)) {
        (Some(n), _) => Some(format!("Matchday {n}")),
        (None, Some(stage)) => Some(stage.replace('_', " ").to_lowercase()),
        _ => None,
    };
    Some(Match {
        id: format!("{}:{}", SportsSource::FootballData.prefix(), m.id),
        source: SportsSource::FootballData,
        sport: "Soccer".into(),
        league: League {
            id: m.competition.id.map(|i| i.to_string()).unwrap_or_default(),
            name: clean(m.competition.name).unwrap_or_default(),
            badge: clean(m.competition.emblem),
            country: clean(m.area.name),
        },
        title: format!("{home} vs {away}"),
        home: Some(Team { name: home, badge: clean(m.home_team.crest) }),
        away: Some(Team { name: away, badge: clean(m.away_team.crest) }),
        status,
        kickoff,
        home_score: m.score.full_time.home,
        away_score: m.score.full_time.away,
        phase: phase(m.status.as_deref()),
        venue: clean(m.venue),
        round,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: &str = r#"{"filters":{},"resultSet":{"count":3},"matches":[
        {"id":551234,"utcDate":"2026-11-09T20:00:00Z","status":"TIMED","matchday":12,"stage":"REGULAR_SEASON",
         "area":{"name":"England"},"competition":{"id":2021,"name":"Premier League","emblem":"https://crests.football-data.org/PL.png"},
         "homeTeam":{"id":57,"name":"Arsenal FC","crest":"https://crests.football-data.org/57.png"},
         "awayTeam":{"id":61,"name":"Chelsea FC","crest":null},
         "score":{"winner":null,"duration":"REGULAR","fullTime":{"home":null,"away":null},"halfTime":{"home":null,"away":null}}},
        {"id":551235,"utcDate":"2026-11-09T17:30:00Z","status":"PAUSED","stage":"GROUP_STAGE",
         "competition":{"id":2001,"name":"UEFA Champions League"},"homeTeam":{"name":"Inter"},"awayTeam":{"name":"Benfica"},
         "score":{"fullTime":{"home":1,"away":0}}},
        {"id":551236,"utcDate":"2026-11-10T19:00:00Z","status":"SCHEDULED","homeTeam":{"name":null},"awayTeam":{"name":null}}
    ]}"#;

    #[test]
    fn parses_a_day() {
        let resp: Matches = serde_json::from_str(DAY).unwrap();
        let m: Vec<Match> = resp.matches.into_iter().filter_map(to_match).collect();
        assert_eq!(m.len(), 2, "the fixture without teams is skipped");

        let a = &m[0];
        assert_eq!(a.id, "fd:551234");
        assert_eq!(a.status, MatchStatus::Upcoming);
        assert_eq!(a.title, "Arsenal FC vs Chelsea FC");
        assert_eq!(a.league.name, "Premier League");
        assert_eq!(a.league.country.as_deref(), Some("England"));
        assert_eq!(a.round.as_deref(), Some("Matchday 12"));
        assert_eq!(a.kickoff, Some(1_794_254_400), "2026-11-09 20:00 UTC");
        assert_eq!(a.away.as_ref().unwrap().badge, None);
        assert_eq!((a.home_score, a.away_score), (None, None));

        let b = &m[1];
        assert_eq!(b.status, MatchStatus::Live);
        assert_eq!(b.phase.as_deref(), Some("HT"));
        assert_eq!((b.home_score, b.away_score), (Some(1), Some(0)));
        assert_eq!(b.round.as_deref(), Some("group stage"));
    }

    #[test]
    fn statuses_map() {
        assert_eq!(map_status(Some("IN_PLAY")), MatchStatus::Live);
        assert_eq!(map_status(Some("FINISHED")), MatchStatus::Finished);
        assert_eq!(map_status(Some("TIMED")), MatchStatus::Upcoming);
        assert_eq!(map_status(Some("SCHEDULED")), MatchStatus::Upcoming);
        assert_eq!(map_status(Some("POSTPONED")), MatchStatus::Postponed);
        assert_eq!(map_status(Some("CANCELLED")), MatchStatus::Cancelled);
        assert_eq!(map_status(None), MatchStatus::Upcoming);
    }
}
