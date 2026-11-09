//! "Where to watch": which official broadcaster carries a match.
//!
//! Two inputs, both about *who has the rights*, never about where a stream
//! can be found:
//!
//! * hand-written rules in the registry (`sports.watch`), by league and country;
//! * TheSportsDB's channel listing for the match, when it has one.
//!
//! A rule may name an iptv-org channel (`iptv_channel`) for a free-to-air or
//! official free channel, and only then does an option carry a Play button.
//! A channel name from the listing is never looked up in the TV library, so no
//! paid channel can end up with a Play button by matching its name.

use serde::{Deserialize, Serialize};

use super::{norm_team, Match, SportsSource, TvListing};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WatchKind {
    /// Free to watch.
    Free,
    /// Needs a subscription or a purchase.
    Subscription,
    #[default]
    Unknown,
}

/// One hand-written rule: this league, in this country, is on this broadcaster.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WatchRule {
    /// Text the league name must contain (case-insensitive).
    pub league: String,
    /// Restrict to one sport (exact, case-insensitive).
    pub sport: Option<String>,
    /// ISO 3166-1 alpha-2 country, or `*` for anywhere.
    pub country: String,
    pub name: String,
    pub kind: WatchKind,
    /// The broadcaster's page for the match or competition.
    pub url: Option<String>,
    /// iptv-org id of a free-to-air or official free channel carrying it.
    pub iptv_channel: Option<String>,
}

/// A row in the "Where to watch" card.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WatchOption {
    pub name: String,
    /// Country the option applies in (ISO code for rules, a name for listings).
    pub country: String,
    pub kind: WatchKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// `registry` or `thesportsdb`.
    pub source: String,
    /// Set only by a hand-written rule: a free channel in the TV library the
    /// app can play. The engine clears it when the channel isn't there.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub play_channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
}

/// Options from the registry's hand-written rules.
pub fn from_rules(rules: &[WatchRule], m: &Match, country: &str) -> Vec<WatchOption> {
    let league = m.league.name.to_lowercase();
    rules
        .iter()
        .filter(|r| !r.league.trim().is_empty() && league.contains(&r.league.trim().to_lowercase()))
        .filter(|r| r.sport.as_deref().is_none_or(|s| s.eq_ignore_ascii_case(&m.sport)))
        .filter(|r| r.country.trim() == "*" || r.country.trim().eq_ignore_ascii_case(country.trim()))
        .map(|r| WatchOption {
            name: r.name.clone(),
            country: r.country.trim().to_string(),
            kind: r.kind,
            url: r.url.clone(),
            source: "registry".into(),
            play_channel: r.iptv_channel.clone().filter(|c| !c.trim().is_empty()),
            logo: None,
        })
        .collect()
}

/// Options from TheSportsDB's channel listing. Names only: these never get a
/// Play button. With a country given, only channels from that country.
pub fn from_listing(listing: &[TvListing], m: &Match, country: &str) -> Vec<WatchOption> {
    let want = country_name(country);
    listing
        .iter()
        .filter(|l| is_this_match(l, m))
        .filter(|l| want.is_none_or(|w| l.country.eq_ignore_ascii_case(w)))
        .map(|l| WatchOption {
            name: l.channel.clone(),
            country: l.country.clone(),
            kind: WatchKind::Unknown,
            url: None,
            source: "thesportsdb".into(),
            play_channel: None,
            logo: l.logo.clone(),
        })
        .collect()
}

/// Same event: by TheSportsDB id when the match came from there, otherwise by
/// both team names appearing in the listing's event title.
fn is_this_match(l: &TvListing, m: &Match) -> bool {
    if m.source == SportsSource::TheSportsDb && m.id.strip_prefix("tsdb:") == Some(l.event_id.as_str()) {
        return true;
    }
    let event = norm_team(&l.event);
    match (&m.home, &m.away) {
        (Some(h), Some(a)) => {
            let (h, a) = (norm_team(&h.name), norm_team(&a.name));
            !h.is_empty() && !a.is_empty() && event.contains(&h) && event.contains(&a)
        }
        _ => false,
    }
}

/// The name TheSportsDB uses for a country, for the ones people ask about most.
/// `None` (an unknown or empty code) means "don't filter by country".
fn country_name(code: &str) -> Option<&'static str> {
    Some(match code.trim().to_ascii_uppercase().as_str() {
        "US" => "United States",
        "GB" | "UK" => "United Kingdom",
        "CA" => "Canada",
        "AU" => "Australia",
        "NZ" => "New Zealand",
        "IE" => "Ireland",
        "IN" => "India",
        "PK" => "Pakistan",
        "BD" => "Bangladesh",
        "LK" => "Sri Lanka",
        "AE" => "United Arab Emirates",
        "ZA" => "South Africa",
        "NG" => "Nigeria",
        "DE" => "Germany",
        "FR" => "France",
        "ES" => "Spain",
        "IT" => "Italy",
        "PT" => "Portugal",
        "NL" => "Netherlands",
        "BR" => "Brazil",
        "AR" => "Argentina",
        "MX" => "Mexico",
        "JP" => "Japan",
        "KR" => "South Korea",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::super::{sample, MatchStatus};
    use super::*;

    fn rule(league: &str, country: &str, name: &str) -> WatchRule {
        WatchRule { league: league.into(), country: country.into(), name: name.into(), ..Default::default() }
    }

    fn listing(event_id: &str, event: &str, channel: &str, country: &str) -> TvListing {
        TvListing { event_id: event_id.into(), event: event.into(), channel: channel.into(), country: country.into(), logo: None }
    }

    #[test]
    fn rules_match_league_and_country() {
        let mut m = sample("tsdb:1", "Seattle", "Portland", 1_800_000_000, MatchStatus::Upcoming);
        m.league.name = "American Major League Soccer".into();
        let rules = vec![
            rule("major league soccer", "US", "Example Sports"),
            rule("Major League Soccer", "GB", "Other Network"),
            rule("MLS Next", "US", "Not this league"),
            rule("soccer", "*", "Anywhere TV"),
            rule("", "US", "Empty league never matches"),
        ];
        let names: Vec<_> = from_rules(&rules, &m, "us").into_iter().map(|o| o.name).collect();
        assert_eq!(names, ["Example Sports", "Anywhere TV"]);
    }

    #[test]
    fn sport_filter_applies() {
        let m = sample("tsdb:1", "A", "B", 1, MatchStatus::Upcoming);
        let mut r = rule("Test League", "US", "Hockey only");
        r.sport = Some("Ice Hockey".into());
        assert!(from_rules(std::slice::from_ref(&r), &m, "US").is_empty());
        r.sport = Some("soccer".into());
        assert_eq!(from_rules(std::slice::from_ref(&r), &m, "US").len(), 1);
    }

    #[test]
    fn only_a_hand_written_rule_can_offer_play() {
        let m = sample("tsdb:2475508", "Chicago Bears", "Tampa Bay Buccaneers", 1_800_000_000, MatchStatus::Upcoming);
        let mut free = rule("Test League", "US", "Free-to-air");
        free.iptv_channel = Some("Example.us".into());
        let mut blank = rule("Test League", "US", "Blank channel");
        blank.iptv_channel = Some("  ".into());
        let opts = from_rules(&[free, blank], &m, "US");
        assert_eq!(opts[0].play_channel.as_deref(), Some("Example.us"));
        assert_eq!(opts[1].play_channel, None, "a blank channel id is no channel");

        let listed = from_listing(
            &[listing("2475508", "Chicago Bears vs Tampa Bay Buccaneers", "Sky Sports NFL", "United Kingdom")],
            &m,
            "GB",
        );
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].play_channel, None, "a listed channel never gets a Play button");
        assert_eq!(listed[0].source, "thesportsdb");
    }

    #[test]
    fn listing_matches_by_id_or_team_names_and_filters_country() {
        let by_id = sample("tsdb:99", "Alpha", "Beta", 1, MatchStatus::Upcoming);
        let l = [listing("99", "whatever", "Chan A", "United Kingdom"), listing("5", "Other Match", "Chan B", "United Kingdom")];
        assert_eq!(from_listing(&l, &by_id, "").len(), 1);

        // A football-data match has no TheSportsDB id, so team names decide.
        let mut by_name = sample("fd:7", "Arsenal FC", "Chelsea FC", 1, MatchStatus::Upcoming);
        by_name.source = SportsSource::FootballData;
        let l = [
            listing("12", "Arsenal vs Chelsea", "Sky Sports", "United Kingdom"),
            listing("12", "Arsenal vs Chelsea", "ESPN", "United States"),
            listing("13", "Arsenal vs Spurs", "Wrong game", "United Kingdom"),
        ];
        assert_eq!(from_listing(&l, &by_name, "").len(), 2, "no country, no filter");
        let uk: Vec<_> = from_listing(&l, &by_name, "gb").into_iter().map(|o| o.name).collect();
        assert_eq!(uk, ["Sky Sports"]);
        assert_eq!(from_listing(&l, &by_name, "ZZ").len(), 2, "an unknown code doesn't filter");
    }
}
