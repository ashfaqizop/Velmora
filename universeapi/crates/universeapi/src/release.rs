//! Release-name parsing: turns scene/site labels such as
//! `"Jurassic World (2025) 1080p 10bit HEVC [Hindi (DD5.1) & English] [2.2GB]"`
//! into structured tags used for ranking and UI chips.

use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoCodec {
    H264,
    H265,
    Av1,
    Vp9,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Hdr {
    Sdr,
    Hdr10,
    HdrPlus,
    DolbyVision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Cam,
    Telesync,
    HdRip,
    WebRip,
    WebDl,
    HdTv,
    BluRay,
    Remux,
}

/// French release variants (freeflix sources).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FrenchVariant {
    /// Version française (French dub).
    Vf,
    /// Version originale sous-titrée français.
    Vostfr,
    /// Version française internationale / québécoise.
    Vfq,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReleaseTags {
    /// Vertical resolution: 360, 480, 720, 1080, 2160.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codec: Option<VideoCodec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bit_depth: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hdr: Option<Hdr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceType>,
    /// Audio formats mentioned, e.g. ["DD5.1", "Atmos", "AAC"].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub audio: Vec<String>,
    /// Language codes mentioned, e.g. ["hi", "en"].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub languages: Vec<String>,
    #[serde(default)]
    pub multi_audio: bool,
    #[serde(default)]
    pub has_subs: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub french: Option<FrenchVariant>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    #[serde(default)]
    pub is_pack: bool,
}

macro_rules! re {
    ($name:ident, $pat:expr) => {
        static $name: LazyLock<Regex> = LazyLock::new(|| Regex::new($pat).expect("valid regex"));
    };
}

re!(RES, r"(?i)\b(2160|1440|1080|720|576|480|360|240)[pi]\b");
re!(RES_4K, r"(?i)\b(4k|uhd)\b");
re!(H265, r"(?i)\b(x\.?265|h\.?265|hevc)\b");
re!(H264, r"(?i)\b(x\.?264|h\.?264|avc)\b");
re!(AV1, r"(?i)\bav1\b");
re!(VP9, r"(?i)\bvp9\b");
re!(BITS, r"(?i)\b(8|10|12)[ -]?bits?\b");
re!(DV, r"(?i)\b(dolby[ .]?vision|dovi|dv)\b");
re!(HDR_PLUS, r"(?i)\bhdr10(\+|plus)");
re!(HDR, r"(?i)\bhdr(10)?\b");
re!(SDR, r"(?i)\bsdr\b");
re!(REMUX, r"(?i)\bremux\b");
re!(BLURAY, r"(?i)\b(blu-?ray|bdrip|brrip|bd-?rip)\b");
re!(WEBDL, r"(?i)\bweb[ .-]?dl\b");
re!(WEBRIP, r"(?i)\bweb[ .-]?rip\b|\bweb\b");
re!(HDTV, r"(?i)\bhdtv\b");
re!(HDRIP, r"(?i)\b(hd-?rip|dvdrip|pre-?dvd|ds4k)\b");
re!(CAM, r"(?i)\b(cam|camrip|hdcam|hq-?cam)\b");
re!(TS, r"(?i)\b(telesync|hdts|hd-ts|ts)\b");
re!(AUDIO, r"(?i)\b(ddp?\+?[ .]?[257]\.[01]|dd\+|ddp|eac3|e-ac-3|ac3|truehd|atmos|dts(?:-hd)?(?:[ .]?ma)?|aac(?:[ .]?[257]\.[01])?|opus|flac)\b");
re!(SUBS, r"(?i)\b(e-?subs?|msubs?|subs?|subtitles?|sub)\b");
re!(MULTI, r"(?i)\b(multi|dual[ -]?audio|tri[ -]?audio)\b");
re!(PACK, r"(?i)\b(pack|complete|full[ -]season|s\d{1,2}[ .-]?complete)\b");
re!(VOSTFR, r"(?i)\bvostfr\b");
re!(VFQ, r"(?i)\bvfq\b");
re!(VF, r"(?i)\b(vf|vff|vfi|truefrench|french)\b");
re!(SIZE, r"(?i)(\d+(?:[.,]\d+)?)\s*(tb|gb|mb|kb|gib|mib)\b");

const LANGS: &[(&str, &str)] = &[
    ("hindi", "hi"),
    ("english", "en"),
    ("eng", "en"),
    ("tamil", "ta"),
    ("telugu", "te"),
    ("malayalam", "ml"),
    ("kannada", "kn"),
    ("bengali", "bn"),
    ("bangla", "bn"),
    ("punjabi", "pa"),
    ("gujarati", "gu"),
    ("marathi", "mr"),
    ("urdu", "ur"),
    ("japanese", "ja"),
    ("korean", "ko"),
    ("chinese", "zh"),
    ("mandarin", "zh"),
    ("french", "fr"),
    ("spanish", "es"),
    ("german", "de"),
    ("italian", "it"),
    ("portuguese", "pt"),
    ("russian", "ru"),
    ("arabic", "ar"),
    ("turkish", "tr"),
    ("thai", "th"),
    ("indonesian", "id"),
];

impl ReleaseTags {
    pub fn parse(text: &str) -> Self {
        let mut t = Self::default();
        if text.is_empty() {
            return t;
        }
        // Treat separators in scene names as spaces so \b boundaries work.
        let norm = text.replace(['_', '[', ']', '(', ')', '|', '–', '—'], " ");

        t.resolution = RES
            .captures_iter(&norm)
            .filter_map(|c| c[1].parse::<u16>().ok())
            .max()
            .or_else(|| RES_4K.is_match(&norm).then_some(2160));

        t.codec = if H265.is_match(&norm) {
            Some(VideoCodec::H265)
        } else if AV1.is_match(&norm) {
            Some(VideoCodec::Av1)
        } else if VP9.is_match(&norm) {
            Some(VideoCodec::Vp9)
        } else if H264.is_match(&norm) {
            Some(VideoCodec::H264)
        } else {
            None
        };

        t.bit_depth = BITS.captures(&norm).and_then(|c| c[1].parse().ok());

        t.hdr = if DV.is_match(&norm) {
            Some(Hdr::DolbyVision)
        } else if HDR_PLUS.is_match(&norm) {
            Some(Hdr::HdrPlus)
        } else if HDR.is_match(&norm) {
            Some(Hdr::Hdr10)
        } else if SDR.is_match(&norm) {
            Some(Hdr::Sdr)
        } else {
            None
        };

        t.source = if REMUX.is_match(&norm) {
            Some(SourceType::Remux)
        } else if BLURAY.is_match(&norm) {
            Some(SourceType::BluRay)
        } else if WEBDL.is_match(&norm) {
            Some(SourceType::WebDl)
        } else if HDTV.is_match(&norm) {
            Some(SourceType::HdTv)
        } else if WEBRIP.is_match(&norm) {
            Some(SourceType::WebRip)
        } else if HDRIP.is_match(&norm) {
            Some(SourceType::HdRip)
        } else if CAM.is_match(&norm) {
            Some(SourceType::Cam)
        } else if TS.is_match(&norm) {
            Some(SourceType::Telesync)
        } else {
            None
        };

        for m in AUDIO.find_iter(&norm) {
            let a = canonical_audio(m.as_str());
            if !t.audio.contains(&a) {
                t.audio.push(a);
            }
        }

        let lower = norm.to_ascii_lowercase();
        for word in lower.split(|c: char| !c.is_ascii_alphanumeric()) {
            if let Some((_, code)) = LANGS.iter().find(|(name, _)| *name == word) {
                let code = (*code).to_string();
                if !t.languages.contains(&code) {
                    t.languages.push(code);
                }
            }
        }

        t.french = if VOSTFR.is_match(&norm) {
            Some(FrenchVariant::Vostfr)
        } else if VFQ.is_match(&norm) {
            Some(FrenchVariant::Vfq)
        } else if VF.is_match(&norm) {
            Some(FrenchVariant::Vf)
        } else {
            None
        };
        if matches!(t.french, Some(FrenchVariant::Vf | FrenchVariant::Vfq)) && !t.languages.iter().any(|l| l == "fr") {
            t.languages.push("fr".into());
        }

        t.multi_audio = MULTI.is_match(&norm) || t.languages.len() >= 2;
        t.has_subs = SUBS.is_match(&norm) || t.french == Some(FrenchVariant::Vostfr);
        t.is_pack = PACK.is_match(&norm);
        t.size_bytes = parse_size(&norm);
        t
    }

    /// Fill fields that are unknown here from `other`.
    pub fn merge_missing(&mut self, other: &ReleaseTags) {
        self.resolution = self.resolution.or(other.resolution);
        self.codec = self.codec.or(other.codec);
        self.bit_depth = self.bit_depth.or(other.bit_depth);
        self.hdr = self.hdr.or(other.hdr);
        self.source = self.source.or(other.source);
        self.french = self.french.or(other.french);
        self.size_bytes = self.size_bytes.or(other.size_bytes);
        for a in &other.audio {
            if !self.audio.contains(a) {
                self.audio.push(a.clone());
            }
        }
        for l in &other.languages {
            if !self.languages.contains(l) {
                self.languages.push(l.clone());
            }
        }
        self.multi_audio |= other.multi_audio;
        self.has_subs |= other.has_subs;
        self.is_pack |= other.is_pack;
    }

    /// Short human chips: "1080p", "HEVC", "10-bit", "DD5.1", "Dual Audio", "VOSTFR".
    pub fn badges(&self) -> Vec<String> {
        let mut b = Vec::new();
        if let Some(r) = self.resolution {
            b.push(if r >= 2160 { "4K".to_string() } else { format!("{r}p") });
        }
        if let Some(h) = self.hdr {
            b.push(match h {
                Hdr::DolbyVision => "Dolby Vision",
                Hdr::HdrPlus => "HDR10+",
                Hdr::Hdr10 => "HDR",
                Hdr::Sdr => "SDR",
            }
            .into());
        }
        if let Some(c) = self.codec {
            b.push(match c {
                VideoCodec::H264 => "H.264",
                VideoCodec::H265 => "HEVC",
                VideoCodec::Av1 => "AV1",
                VideoCodec::Vp9 => "VP9",
            }
            .into());
        }
        if self.bit_depth == Some(10) {
            b.push("10-bit".into());
        }
        b.extend(self.audio.iter().take(2).cloned());
        if self.multi_audio {
            b.push(if self.languages.len() > 2 { "Multi Audio".into() } else { "Dual Audio".into() });
        }
        if let Some(f) = self.french {
            b.push(
                match f {
                    FrenchVariant::Vf => "VF",
                    FrenchVariant::Vostfr => "VOSTFR",
                    FrenchVariant::Vfq => "VFQ",
                }
                .into(),
            );
        }
        if self.is_pack {
            b.push("Pack".into());
        }
        b
    }
}

fn canonical_audio(raw: &str) -> String {
    let u = raw.to_ascii_uppercase().replace(' ', "");
    if u.starts_with("ATMOS") {
        "Atmos".into()
    } else if u.starts_with("TRUEHD") {
        "TrueHD".into()
    } else if u.starts_with("DTS") {
        u.replace('.', " ").trim().to_string()
    } else if u.starts_with("DDP") || u.starts_with("DD+") || u.starts_with("EAC3") || u.starts_with("E-AC-3") {
        let ch = u.trim_start_matches(|c: char| !c.is_ascii_digit());
        if ch.is_empty() {
            "DD+".into()
        } else {
            format!("DD+{ch}")
        }
    } else if u.starts_with("DD") || u.starts_with("AC3") {
        let ch = u.trim_start_matches(|c: char| !c.is_ascii_digit());
        if ch.is_empty() {
            "DD".into()
        } else {
            format!("DD{ch}")
        }
    } else if u.starts_with("AAC") {
        let ch = u.trim_start_matches("AAC").trim_start_matches('.');
        if ch.is_empty() {
            "AAC".into()
        } else {
            format!("AAC{ch}")
        }
    } else {
        u
    }
}

/// Parses the largest "1.6GB" / "500 MB" style size mentioned in the text.
pub fn parse_size(text: &str) -> Option<u64> {
    SIZE.captures_iter(text)
        .filter_map(|c| {
            let n: f64 = c[1].replace(',', ".").parse().ok()?;
            let mult: f64 = match c[2].to_ascii_lowercase().as_str() {
                "tb" => 1e12,
                "gb" | "gib" => 1024.0 * 1024.0 * 1024.0,
                "mb" | "mib" => 1024.0 * 1024.0,
                "kb" => 1024.0,
                _ => return None,
            };
            Some((n * mult) as u64)
        })
        .max()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hdhub_movie_label() {
        let t = ReleaseTags::parse("1080p 10bit HEVC [2.2GB]");
        assert_eq!(t.resolution, Some(1080));
        assert_eq!(t.codec, Some(VideoCodec::H265));
        assert_eq!(t.bit_depth, Some(10));
        assert_eq!(t.size_bytes, Some((2.2 * 1024.0 * 1024.0 * 1024.0) as u64));
    }

    #[test]
    fn hdhub_title() {
        let t = ReleaseTags::parse(
            "Jurassic World: Rebirth (2025) DS4K WEB-DL [Hindi (DD5.1) & English] 4K 1080p 720p & 480p Dual Audio [x264/10Bit-HEVC]",
        );
        assert_eq!(t.resolution, Some(1080), "explicit p-resolutions win over the 4K marketing tag");
        assert_eq!(t.source, Some(SourceType::WebDl));
        assert!(t.languages.contains(&"hi".to_string()) && t.languages.contains(&"en".to_string()));
        assert!(t.multi_audio);
        assert!(t.audio.contains(&"DD5.1".to_string()));
    }

    #[test]
    fn four_k_pack() {
        let t = ReleaseTags::parse("4K [2160p SDR MULTi WEB-DL PACK – 31.1GB]");
        assert_eq!(t.resolution, Some(2160));
        assert_eq!(t.hdr, Some(Hdr::Sdr));
        assert!(t.is_pack && t.multi_audio);
        assert!(t.size_bytes.unwrap() > 30 * 1024 * 1024 * 1024);
    }

    #[test]
    fn scene_filename() {
        let t = ReleaseTags::parse("Akka.S01.1080p.NF.WEB-DL.Multi.DDP5.1.Atmos.H.264-HDHub4u.Ms.zip");
        assert_eq!(t.resolution, Some(1080));
        assert_eq!(t.codec, Some(VideoCodec::H264));
        assert!(t.audio.contains(&"DD+5.1".to_string()));
        assert!(t.audio.contains(&"Atmos".to_string()));
    }

    #[test]
    fn french_variants() {
        assert_eq!(ReleaseTags::parse("One Piece 1100 VOSTFR").french, Some(FrenchVariant::Vostfr));
        let vf = ReleaseTags::parse("Dune Part Two TRUEFRENCH 1080p");
        assert_eq!(vf.french, Some(FrenchVariant::Vf));
        assert!(vf.languages.contains(&"fr".to_string()));
    }

    #[test]
    fn badges_render() {
        let b = ReleaseTags::parse("720p x264 ESubs [1.2GB]").badges();
        assert_eq!(b, vec!["720p", "H.264"]);
    }

    #[test]
    fn sizes() {
        assert_eq!(parse_size("[500MB]"), Some(500 * 1024 * 1024));
        assert_eq!(parse_size("Download File [1.25 GB]"), Some((1.25 * 1073741824.0) as u64));
        assert_eq!(parse_size("no size"), None);
    }
}
