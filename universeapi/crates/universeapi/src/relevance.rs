//! How well a title matches what was typed.
//!
//! Search asks every source and gets a long tail back. When one of the results
//! *is* the title the user typed, that result should lead, and the tail can wait
//! behind a button. [`narrow`] does the sorting and picks out the exact matches;
//! the app shows only those first and keeps the full, ranked list for "Can't
//! find it? Extensive search".

use serde::{Deserialize, Serialize};

use crate::model::TitleSummary;

/// How a title matches the query, best first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Fit {
    /// The whole title is what was typed (case, accents, punctuation and a
    /// leading "The" aside; a typed year has to agree).
    Exact,
    /// The title starts with the words typed.
    Prefix,
    /// Every word typed is a whole word of the title, or it's the right title
    /// in a different year.
    Word,
    /// What was typed appears somewhere inside the title.
    Contains,
    Other,
}

/// What a search returns once it has been looked over.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Narrowed {
    /// Titles that exactly match the query. Empty when nothing does.
    pub exact: Vec<TitleSummary>,
    /// Everything, best match first; the order within a tier is kept.
    pub ranked: Vec<TitleSummary>,
}

/// The query broken into what we compare: lowercase words and an optional year.
struct Wanted {
    words: Vec<String>,
    text: String,
    year: Option<u16>,
}

/// Common Latin accents folded away, so "Pokemon" finds "Pokémon".
fn fold(c: char) -> char {
    match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' => 'a',
        'ç' | 'ć' | 'č' => 'c',
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ě' => 'e',
        'ì' | 'í' | 'î' | 'ï' | 'ī' => 'i',
        'ñ' | 'ń' => 'n',
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' => 'o',
        'ù' | 'ú' | 'û' | 'ü' | 'ū' => 'u',
        'ý' | 'ÿ' => 'y',
        'š' | 'ś' => 's',
        'ž' | 'ź' | 'ż' => 'z',
        _ => c,
    }
}

fn words_of(s: &str) -> Vec<String> {
    let folded: String = s.to_lowercase().chars().map(fold).collect();
    folded.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).map(str::to_string).collect()
}

fn is_year(w: &str) -> bool {
    w.len() == 4 && matches!(w.parse::<u16>(), Ok(1900..=2099))
}

fn parse_query(q: &str) -> Wanted {
    let mut words = words_of(q);
    // "dune 2021": the year narrows the match, but a lone "1917" is a title.
    let year = words.last().filter(|w| words.len() > 1 && is_year(w)).and_then(|w| w.parse::<u16>().ok());
    if year.is_some() {
        words.pop();
    }
    let text = words.concat();
    Wanted { words, text, year }
}

fn fit_with(w: &Wanted, t: &TitleSummary) -> Fit {
    if w.text.is_empty() {
        return Fit::Other;
    }
    let mut best = Fit::Other;
    for name in std::iter::once(t.name.as_str()).chain(t.original_name.as_deref()) {
        let words = words_of(name);
        let full = words.concat();
        // "The Batman" is found by typing "batman".
        let bare = match words.first().map(String::as_str) {
            Some("the" | "a" | "an") if words.len() > 1 => words[1..].concat(),
            _ => full.clone(),
        };
        let fit = if full == w.text || bare == w.text {
            match (w.year, t.year) {
                (Some(want), Some(have)) if want != have => Fit::Word,
                _ => Fit::Exact,
            }
        } else if words.len() >= w.words.len() && words[..w.words.len()] == w.words[..] {
            Fit::Prefix
        } else if w.words.iter().all(|q| words.contains(q)) {
            Fit::Word
        } else if full.contains(&w.text) {
            Fit::Contains
        } else {
            Fit::Other
        };
        best = best.min(fit);
    }
    best
}

/// How well one title matches a query.
pub fn fit(query: &str, t: &TitleSummary) -> Fit {
    fit_with(&parse_query(query), t)
}

/// Best matches first. Titles that fit equally well keep the order they came in.
pub fn rank(mut items: Vec<TitleSummary>, query: &str) -> Vec<TitleSummary> {
    let w = parse_query(query);
    items.sort_by_cached_key(|t| fit_with(&w, t));
    items
}

/// Ranks the results and pulls out the exact matches.
pub fn narrow(items: &[TitleSummary], query: &str) -> Narrowed {
    let w = parse_query(query);
    let ranked = rank(items.to_vec(), query);
    let exact = ranked.iter().filter(|t| fit_with(&w, t) == Fit::Exact).cloned().collect();
    Narrowed { exact, ranked }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{MediaKind, ProviderId};

    fn title(name: &str, year: Option<u16>) -> TitleSummary {
        let mut t = TitleSummary::new(ProviderId::HdHub, name.to_lowercase().replace(' ', "-"), name, MediaKind::Movie);
        t.year = year;
        t
    }

    fn names(v: &[TitleSummary]) -> Vec<&str> {
        v.iter().map(|t| t.name.as_str()).collect()
    }

    #[test]
    fn an_exact_title_stands_alone() {
        let items = vec![
            title("SHIROHIYO", None),
            title("Shugo Chara", None),
            title("Saiyaara", Some(2025)),
            title("Aashiqui 2", Some(2013)),
        ];
        let n = narrow(&items, "Saiyaara");
        assert_eq!(names(&n.exact), ["Saiyaara"]);
        assert_eq!(n.ranked.len(), 4, "the full list is still there for the extensive search");
        assert_eq!(n.ranked[0].name, "Saiyaara", "and the exact match leads it");
    }

    #[test]
    fn nothing_exact_means_no_narrowing() {
        let items = vec![title("Dune: Part Two", Some(2024)), title("Dune", Some(2021))];
        let n = narrow(&items, "dune part");
        assert!(n.exact.is_empty());
        assert_eq!(n.ranked.len(), 2);
        assert_eq!(n.ranked[0].name, "Dune: Part Two", "a title starting with the words ranks first");
    }

    #[test]
    fn case_punctuation_and_accents_dont_matter() {
        let items = vec![title("Spider-Man: No Way Home", Some(2021)), title("Amélie", Some(2001))];
        assert_eq!(narrow(&items, "spiderman no way home").exact.len(), 1);
        assert_eq!(narrow(&items, "SPIDER MAN NO WAY HOME").exact.len(), 1);
        assert_eq!(narrow(&items, "Amélie").exact.len(), 1);
        assert_eq!(narrow(&items, "amelie").exact.len(), 1, "accents are folded");
        assert_eq!(narrow(&[title("Pokémon", None)], "Pokemon").exact.len(), 1);
    }

    #[test]
    fn a_leading_article_is_optional() {
        let items = vec![title("The Batman", Some(2022)), title("Batman Begins", Some(2005))];
        assert_eq!(names(&narrow(&items, "batman").exact), ["The Batman"]);
        assert_eq!(names(&narrow(&items, "the batman").exact), ["The Batman"]);
    }

    #[test]
    fn a_typed_year_has_to_agree() {
        let items = vec![title("Dune", Some(1984)), title("Dune", Some(2021))];
        assert_eq!(narrow(&items, "dune").exact.len(), 2, "no year typed: both are the title");
        let n = narrow(&items, "Dune 2021");
        assert_eq!(n.exact.len(), 1);
        assert_eq!(n.exact[0].year, Some(2021));
        assert_eq!(n.ranked[1].year, Some(1984), "the other year still ranks as a close match");
        assert_eq!(narrow(&[title("1917", Some(2019))], "1917").exact.len(), 1, "a lone year is a title");
    }

    #[test]
    fn the_original_name_counts() {
        let mut t = title("Attack on Titan", Some(2013));
        t.original_name = Some("Shingeki no Kyojin".into());
        let items = vec![t, title("Titan A.E.", Some(2000))];
        assert_eq!(names(&narrow(&items, "shingeki no kyojin").exact), ["Attack on Titan"]);
    }

    #[test]
    fn ranking_goes_exact_prefix_word_contains_other() {
        let items = vec![
            title("Unrelated", None),
            title("Superman Returns", None),
            title("The Last Superman", None),
            title("Superman", Some(1978)),
            title("Supermanhattan", None),
        ];
        let ranked = rank(items, "superman");
        // Exact, then starts with the words, then has them as words, then only contains the letters.
        assert_eq!(names(&ranked), ["Superman", "Superman Returns", "The Last Superman", "Supermanhattan", "Unrelated"]);
    }

    #[test]
    fn ties_keep_their_order() {
        let items = vec![title("Zeta Quest", None), title("Alpha Quest", None), title("Quest", None)];
        let r = rank(items, "quest");
        assert_eq!(names(&r), ["Quest", "Zeta Quest", "Alpha Quest"]);
    }

    #[test]
    fn an_empty_query_changes_nothing() {
        let items = vec![title("B", None), title("A", None)];
        let n = narrow(&items, "  ");
        assert!(n.exact.is_empty());
        assert_eq!(names(&n.ranked), ["B", "A"]);
    }

    #[test]
    fn non_latin_titles_match_too() {
        let items = vec![title("進撃の巨人", Some(2013)), title("他のタイトル", None)];
        assert_eq!(narrow(&items, "進撃の巨人").exact.len(), 1);
    }
}
