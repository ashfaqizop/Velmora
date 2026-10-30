//! Metadata sources: trending rails, rich detail (seasons, cast, artwork) and
//! the external ids that let the engine merge titles across providers.
//! They implement [`crate::providers::Provider`] but never return sources.

pub mod anilist;
pub mod cinemeta;
pub mod tmdb;

/// Strip HTML tags (`<br>`, `<i>`) from descriptions.
pub(crate) fn strip_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    crate::util::collapse_ws(&crate::util::unescape_html(&out))
}

#[cfg(test)]
mod tests {
    #[test]
    fn tags_stripped() {
        assert_eq!(super::strip_tags("A <i>mage</i><br><br>story &amp; more"), "A magestory & more");
    }
}
