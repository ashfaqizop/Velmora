//! Embedded web-player hosts whose config is hidden in packed JavaScript.

use std::sync::LazyLock;

use regex::Regex;

use super::ResolveCtx;
use crate::error::{Error, Result};
use crate::model::{HostedLink, Source, StreamFormat, Track};
use crate::util::{origin_of, unpack_packer};

static M3U8: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"https?://[^"'\s\\]+\.m3u8[^"'\s\\]*"#).expect("regex"));
static TRACK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\{\s*file\s*:\s*"([^"]+\.(?:vtt|srt))"\s*(?:,\s*label\s*:\s*"([^"]*)")?"#).expect("regex")
});

/// `hdstream4u.com/file/{id}` — JW Player config inside P.A.C.K.E.R.
/// Yields a signed multi-audio HLS master plus VTT subtitles.
pub async fn hdstream4u(ctx: &ResolveCtx<'_>, link: &HostedLink) -> Result<Vec<Source>> {
    let html = ctx.http.get_text(&link.url, &[("Referer", &origin_of(&link.url))]).await?;
    let js = unpack_packer(&html).unwrap_or_else(|| html.clone());
    let master = M3U8
        .find(&js)
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| Error::parse("hdstream4u", "no m3u8 in player config"))?;

    let mut s = Source::new(ctx.provider, "HDStream4U", &link.label, master).with_referer(origin_of(&link.url));
    s.format = StreamFormat::Hls;
    s.subtitles = parse_tracks(&js);
    Ok(vec![s])
}

fn parse_tracks(js: &str) -> Vec<Track> {
    let mut out: Vec<Track> = Vec::new();
    for c in TRACK.captures_iter(js) {
        let url = c[1].to_string();
        if out.iter().any(|t| t.url.as_deref() == Some(url.as_str())) {
            continue;
        }
        let label = c.get(2).map(|m| m.as_str().to_string()).filter(|l| !l.is_empty());
        let lang = guess_lang(label.as_deref().unwrap_or(&url));
        out.push(Track { label: label.unwrap_or_else(|| lang.clone().unwrap_or_else(|| "Subtitles".into())), lang, url: Some(url), default: out.is_empty() });
    }
    out
}

/// Guess an ISO 639-1 code from a label or a `_eng.vtt` style filename.
pub(crate) fn guess_lang(s: &str) -> Option<String> {
    let l = s.to_ascii_lowercase();
    const MAP: &[(&[&str], &str)] = &[
        (&["english", "_eng", "_en.", "-en.", "eng."], "en"),
        (&["hindi", "_hin", "_hi."], "hi"),
        (&["french", "fran", "_fre", "_fr."], "fr"),
        (&["spanish", "espa", "_spa", "_es."], "es"),
        (&["arabic", "_ara", "_ar."], "ar"),
        (&["indonesian", "_ind", "_id."], "id"),
        (&["portuguese", "_por", "_pt."], "pt"),
        (&["japanese", "_jpn", "_ja."], "ja"),
        (&["korean", "_kor", "_ko."], "ko"),
        (&["bengali", "bangla", "_ben", "_bn."], "bn"),
    ];
    MAP.iter().find(|(keys, _)| keys.iter().any(|k| l.contains(k))).map(|(_, code)| (*code).to_string())
}

pub fn pixeldrain(ctx: &ResolveCtx<'_>, link: &HostedLink) -> Result<Vec<Source>> {
    let url = super::hub::pixeldrain_direct(&link.url);
    Ok(vec![Source::new(ctx.provider, "PixelDrain", &link.label, url)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jw_tracks() {
        let js = r#"tracks:[{file:"https://cdn.x/vtt/01/t1_eng.vtt",label:"English",kind:"captions"},{file:"https://cdn.x/t1_hin.vtt"}]"#;
        let t = parse_tracks(js);
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].lang.as_deref(), Some("en"));
        assert!(t[0].default);
        assert_eq!(t[1].lang.as_deref(), Some("hi"));
    }
}
