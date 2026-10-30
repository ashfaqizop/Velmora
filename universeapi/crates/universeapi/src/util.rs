//! Small decoding / scraping helpers shared by providers and resolvers.

use std::sync::LazyLock;

use base64::engine::general_purpose::{STANDARD_NO_PAD, URL_SAFE_NO_PAD};
use base64::Engine;
use regex::Regex;
use scraper::ElementRef;

/// Lenient base64: tolerates missing padding, whitespace and URL-safe alphabet.
pub fn b64_decode(input: &str) -> Option<Vec<u8>> {
    let clean: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    let trimmed = clean.trim_end_matches('=');
    STANDARD_NO_PAD.decode(trimmed).or_else(|_| URL_SAFE_NO_PAD.decode(trimmed)).ok()
}

pub fn b64_decode_str(input: &str) -> Option<String> {
    b64_decode(input).and_then(|b| String::from_utf8(b).ok())
}

/// Serde helper: treat an explicit JSON `null` like a missing field.
/// `#[serde(default)]` alone only covers *absent* keys, and upstream APIs
/// (Cinemeta, TMDB, AniList) send `"cast": null` style values.
pub fn null_default<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Default + serde::Deserialize<'de>,
{
    use serde::Deserialize;
    Ok(Option::<T>::deserialize(d)?.unwrap_or_default())
}

/// Repeating-key XOR.
pub fn xor(data: &[u8], key: &[u8]) -> Vec<u8> {
    if key.is_empty() {
        return data.to_vec();
    }
    data.iter().zip(key.iter().cycle()).map(|(d, k)| d ^ k).collect()
}

/// Decode HTML entities (`&#039;`, `&amp;`...) in text captured by regex.
pub fn unescape_html(s: &str) -> String {
    html_escape::decode_html_entities(s).into_owned()
}

/// Visible text of an element with whitespace collapsed.
pub fn text_of(el: ElementRef<'_>) -> String {
    collapse_ws(&el.text().collect::<String>())
}

pub fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Resolve `href` against `base`.
pub fn absolutize(base: &str, href: &str) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        return href.to_string();
    }
    if let Some(rest) = href.strip_prefix("//") {
        return format!("https://{rest}");
    }
    url::Url::parse(base).and_then(|b| b.join(href)).map(|u| u.to_string()).unwrap_or_else(|_| href.to_string())
}

/// Host part of a URL, lowercased, without a leading `www.`.
pub fn host_of(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_ascii_lowercase()))
        .map(|h| h.strip_prefix("www.").map(str::to_string).unwrap_or(h))
        .unwrap_or_default()
}

/// Origin (`scheme://host/`) of a URL, used as a Referer.
pub fn origin_of(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .map(|u| format!("{}://{}/", u.scheme(), u.host_str().unwrap_or_default()))
        .unwrap_or_default()
}

pub fn year_from(text: &str) -> Option<u16> {
    static YEAR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b(19[2-9]\d|20[0-4]\d)\b").expect("regex"));
    YEAR.captures(text).and_then(|c| c[1].parse().ok())
}

static PACKED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)\}\s*\(\s*'(.*)',\s*(\d+),\s*(\d+),\s*'(.*?)'\.split\('\|'\)").expect("packer regex")
});
static WORD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b\w+\b").expect("word regex"));

/// Unpacks Dean Edwards' P.A.C.K.E.R. (`eval(function(p,a,c,k,e,d){...})`),
/// used by most file-host players (hdstream4u, filemoon, streamwish...).
pub fn unpack_packer(source: &str) -> Option<String> {
    let start = source.find("eval(function(p,a,c,k,e,")?;
    let caps = PACKED.captures(&source[start..])?;
    let payload = caps[1].replace("\\'", "'").replace("\\\\", "\\");
    let radix: u32 = caps[2].parse().ok()?;
    let count: usize = caps[3].parse().ok()?;
    let words: Vec<&str> = caps.get(4)?.as_str().split('|').collect();
    if radix == 0 || radix > 95 {
        return None;
    }
    let out = WORD.replace_all(&payload, |m: &regex::Captures<'_>| {
        let token = &m[0];
        match decode_radix(token, radix) {
            Some(i) if i < count && i < words.len() && !words[i].is_empty() => words[i].to_string(),
            _ => token.to_string(),
        }
    });
    Some(out.into_owned())
}

fn decode_radix(token: &str, radix: u32) -> Option<usize> {
    let mut n: usize = 0;
    for c in token.chars() {
        let d = match c {
            '0'..='9' => c as u32 - '0' as u32,
            'a'..='z' => c as u32 - 'a' as u32 + 10,
            'A'..='Z' => c as u32 - 'A' as u32 + 36,
            _ => return None,
        };
        if d >= radix {
            return None;
        }
        n = n.checked_mul(radix as usize)?.checked_add(d as usize)?;
    }
    Some(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_lenient() {
        assert_eq!(
            b64_decode_str("aHR0cHM6Ly96b2tvYW5pbWUudmlkZW8vc3RyZWFtL21hbC81Mjk5MS8xL3N1Yg==").as_deref(),
            Some("https://zokoanime.video/stream/mal/52991/1/sub")
        );
        assert_eq!(b64_decode_str("aGVsbG8").as_deref(), Some("hello"));
    }

    #[test]
    fn xor_roundtrip() {
        let key = b"otaku-embed-v1";
        let enc = xor(br#"{"src":"x.m3u8"}"#, key);
        assert_eq!(xor(&enc, key), br#"{"src":"x.m3u8"}"#);
    }

    #[test]
    fn packer() {
        let src = r"<script>eval(function(p,a,c,k,e,d){while(c--)if(k[c])p=p.replace(new RegExp('\\b'+c.toString(a)+'\\b','g'),k[c]);return p}('0 1=\'2://3.4/5.6\';',7,7,'var|src|https|cdn|example|master|m3u8'.split('|'),0,{}))</script>";
        assert_eq!(unpack_packer(src).as_deref(), Some("var src='https://cdn.example/master.m3u8';"));
    }

    #[test]
    fn nulls_become_defaults() {
        #[derive(serde::Deserialize)]
        struct T {
            #[serde(default, deserialize_with = "null_default")]
            v: Vec<String>,
        }
        assert!(serde_json::from_str::<T>(r#"{"v":null}"#).unwrap().v.is_empty());
        assert!(serde_json::from_str::<T>(r#"{}"#).unwrap().v.is_empty());
        assert_eq!(serde_json::from_str::<T>(r#"{"v":["a"]}"#).unwrap().v, vec!["a"]);
    }

    #[test]
    fn urls() {
        assert_eq!(host_of("https://www.Hubcloud.ist/drive/x"), "hubcloud.ist");
        assert_eq!(origin_of("https://zokoanime.video/stream/mal/1/1/sub"), "https://zokoanime.video/");
        assert_eq!(absolutize("https://a.com/x/y", "/z"), "https://a.com/z");
        assert_eq!(year_from("Avengers: Endgame (2019) BluRay"), Some(2019));
    }
}
