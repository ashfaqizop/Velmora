//! Source ranking. Higher score = auto-selected first. All weights derive
//! from [`Preferences`], so quality / codec / language / provider order are
//! fully user-configurable.

use crate::config::Preferences;
use crate::model::{MediaKind, Source, StreamFormat, Target};
use crate::release::VideoCodec;

pub fn score(s: &Source, prefs: &Preferences, kind: MediaKind, target: Target) -> i32 {
    let mut score = 0i32;

    // Verified links first; unverified sit in the middle; failed ones are dropped earlier.
    score += match s.verified {
        Some(true) => 400,
        Some(false) => -2_000,
        None => 0,
    };

    // Provider priority (user-orderable).
    let rank = prefs.provider_rank(kind, s.provider).min(20) as i32;
    score -= rank * 45;

    // Resolution: closest to preferred wins; above the auto-cap is manual-only.
    match s.tags.resolution {
        Some(r) if r > prefs.max_auto_resolution => score -= 600,
        Some(r) => score -= (i32::from(r) - i32::from(prefs.preferred_resolution)).abs() / 6,
        None => score -= 70,
    }

    // Codec preference.
    match (s.tags.codec, prefs.preferred_codec) {
        (Some(c), p) if c == p => score += 60,
        (Some(VideoCodec::H265), VideoCodec::H264) => score -= 20,
        _ => {}
    }

    // Audio language preference (first choice weighs most).
    for (i, lang) in prefs.audio_languages.iter().enumerate() {
        if s.tags.languages.iter().any(|l| l == lang) || s.audio.iter().any(|t| t.lang.as_deref() == Some(lang.as_str())) {
            score += 90 - (i as i32 * 25).min(80);
            break;
        }
    }

    // Subtitles in a preferred language.
    if prefs
        .subtitle_languages
        .iter()
        .any(|lang| s.subtitles.iter().any(|t| t.lang.as_deref() == Some(lang.as_str())))
    {
        score += 35;
    }

    // Format: adaptive streams start fastest; archives can't play.
    score += match s.format {
        StreamFormat::Hls | StreamFormat::Dash => 40,
        StreamFormat::Mp4 => 35,
        StreamFormat::Mkv | StreamFormat::Webm => 15,
        StreamFormat::Unknown => 0,
        StreamFormat::Torrent => -150,
        // A zip is exactly what a whole-season download wants; elsewhere it can't play.
        StreamFormat::Archive if matches!(target, Target::Season { .. }) => 0,
        StreamFormat::Archive => -400,
    };

    // Torrents: health matters.
    if let Some(t) = &s.torrent {
        let seeds = t.seeders.unwrap_or(0);
        score += match seeds {
            0 => -500,
            1..=4 => -100,
            5..=49 => 0,
            _ => 40,
        };
    }

    // Season packs only make sense when the whole season was requested.
    match (s.is_pack, target) {
        (true, Target::Season { .. }) => score += 200,
        (true, _) => score -= 450,
        _ => {}
    }

    // About to expire → prefer something that will last the whole film.
    if let Some(exp) = s.expires_at {
        let left = exp - chrono::Utc::now().timestamp();
        if left < 15 * 60 {
            score -= 300;
        }
    }

    // Cam/telesync are a last resort.
    if matches!(s.tags.source, Some(crate::release::SourceType::Cam | crate::release::SourceType::Telesync)) {
        score -= 350;
    }

    score
}

/// Score and sort in place (best first). Failed verifications are removed.
pub fn rank(sources: &mut Vec<Source>, prefs: &Preferences, kind: MediaKind, target: Target) {
    sources.retain(|s| s.verified != Some(false));
    for s in sources.iter_mut() {
        s.score = score(s, prefs, kind, target);
    }
    sources.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| b.tags.size_bytes.cmp(&a.tags.size_bytes)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ProviderId;

    fn src(provider: ProviderId, label: &str, url: &str) -> Source {
        let mut s = Source::new(provider, "x", label, url);
        s.verified = Some(true);
        s
    }

    #[test]
    fn prefers_1080p_h264_over_4k_and_480p() {
        let p = Preferences::default();
        let mut v = vec![
            src(ProviderId::HdHub, "4K [2160p SDR WEB-DL]", "https://a/1.mkv"),
            src(ProviderId::HdHub, "480p x264", "https://a/2.mkv"),
            src(ProviderId::HdHub, "1080p x264", "https://a/3.mkv"),
            src(ProviderId::HdHub, "1080p 10bit HEVC", "https://a/4.mkv"),
        ];
        rank(&mut v, &p, MediaKind::Movie, Target::Movie);
        assert_eq!(v[0].label, "1080p x264");
        assert_eq!(v[1].label, "1080p 10bit HEVC");
        assert_eq!(v.last().unwrap().label, "4K [2160p SDR WEB-DL]", "4K is manual-only by default");
    }

    #[test]
    fn four_k_auto_when_user_allows() {
        let p = Preferences { preferred_resolution: 2160, max_auto_resolution: 2160, ..Default::default() };
        let mut v = vec![src(ProviderId::HdHub, "1080p x264", "https://a/1.mkv"), src(ProviderId::HdHub, "2160p x264", "https://a/2.mkv")];
        rank(&mut v, &p, MediaKind::Movie, Target::Movie);
        assert_eq!(v[0].tags.resolution, Some(2160));
    }

    #[test]
    fn provider_order_and_failed_dropped() {
        let p = Preferences::default();
        let mut bad = src(ProviderId::MovieBox, "1080p x264", "https://a/x.mp4");
        bad.verified = Some(false);
        let mut v = vec![
            src(ProviderId::HdHub, "1080p x264", "https://a/1.mp4"),
            src(ProviderId::MovieBox, "1080p x264", "https://a/2.mp4"),
            bad,
        ];
        rank(&mut v, &p, MediaKind::Movie, Target::Movie);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].provider, ProviderId::MovieBox, "MovieBox outranks HDHub for movies");
    }

    #[test]
    fn packs_only_for_seasons() {
        let p = Preferences::default();
        let mut pack = src(ProviderId::HdHub, "1080p WEB-DL PACK", "https://a/s.zip");
        pack.is_pack = true;
        let ep = src(ProviderId::HdHub, "720p x264", "https://a/e.mkv");
        let mut v = vec![pack.clone(), ep.clone()];
        rank(&mut v, &p, MediaKind::Series, Target::Episode { season: 1, episode: 1 });
        assert!(!v[0].is_pack);
        let mut v = vec![ep, pack];
        rank(&mut v, &p, MediaKind::Series, Target::Season { season: 1 });
        assert!(v[0].is_pack);
    }
}
