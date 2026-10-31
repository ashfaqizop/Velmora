// Display helpers for engine sources, shared by the player and download picker.
import { LANGUAGES, providerLabel } from "./options";
import type { Source } from "./types";

export const providerName = (s: Source) => providerLabel(s.provider);

export const isTorrent = (s: Source) => !!s.torrent || s.format === "torrent";

/** Short chips: quality first, then codec, HDR, language, size, seeds. */
export function sourceChips(s: Source): string[] {
  const t = s.tags;
  const size = s.size_bytes ?? t.size_bytes;
  return [
    t.resolution ? (t.resolution >= 2160 ? "4K" : `${t.resolution}p`) : null,
    t.codec === "h265" ? "HEVC" : t.codec?.toUpperCase(),
    t.hdr && t.hdr !== "sdr" ? (t.hdr === "dolby_vision" ? "Dolby Vision" : "HDR") : null,
    t.french ?? null,
    t.multi_audio ? "Multi audio" : null,
    size ? formatBytes(size) : null,
    s.torrent?.seeders != null ? `${s.torrent.seeders} seeds` : null,
  ].filter(Boolean) as string[];
}

export function formatBytes(n: number) {
  if (n >= 1_073_741_824) return `${(n / 1_073_741_824).toFixed(1)} GB`;
  if (n >= 1_048_576) return `${Math.round(n / 1_048_576)} MB`;
  return `${Math.round(n / 1024)} KB`;
}

/** "Provider · Host", with the host left out when it repeats the provider or is a junk server name. */
export function sourceTitle(s: Source): string {
  const host = hostLabel(s);
  return host ? `${providerName(s)} · ${host}` : providerName(s);
}

/**
 * A readable host name, or nothing. Sites label their players with emoji and
 * nicknames ("🍭Playyyy"); those are dropped, real file hosts (HubCloud) stay.
 */
export function hostLabel(s: Source): string | undefined {
  const clean = s.host
    .replace(/[^\p{L}\p{N} ._-]/gu, "")
    .replace(/\s+/g, " ")
    .trim();
  if (clean.length < 3) return undefined;
  if (clean.toLowerCase() === providerName(s).toLowerCase()) return undefined;
  // Stretched letters ("Playyyy") or generic player words aren't hosts.
  if (/(\p{L})\1{2,}/u.test(clean) || /^(player|play|server|stream|watch|video|link|source)\s*\d*$/i.test(clean)) return undefined;
  return clean;
}

/** "Hindi" from "hi"; unknown codes and names pass through. */
export function languageName(code: string): string {
  const c = code.trim().toLowerCase();
  const known = LANGUAGE_NAMES.get(c) ?? LANGUAGE_NAMES.get(c.slice(0, 2));
  if (known && (c.length <= 3 || c.startsWith(`${c.slice(0, 2)}-`))) return known;
  return code.length <= 3 ? code.toUpperCase() : code.charAt(0).toUpperCase() + code.slice(1);
}

const LANGUAGE_NAMES = new Map<string, string>([
  ...LANGUAGES,
  ["eng", "English"],
  ["hin", "Hindi"],
  ["jpn", "Japanese"],
  ["kor", "Korean"],
  ["fre", "French"],
  ["fra", "French"],
  ["spa", "Spanish"],
  ["ger", "German"],
  ["deu", "German"],
  ["tam", "Tamil"],
  ["tel", "Telugu"],
]);

/** Audio languages as names: "Hindi", "Hindi, English", or "Multi audio". */
export function audioLabel(s: Source): string | undefined {
  const raw = s.audio?.map((a) => a.lang ?? a.label) ?? s.tags.audio ?? s.tags.languages ?? [];
  const names = [...new Set(raw.filter(Boolean).map(languageName))];
  if (names.length > 3) return `${names.slice(0, 2).join(", ")} +${names.length - 2}`;
  if (names.length) return names.join(", ");
  return s.tags.multi_audio ? "Multi audio" : undefined;
}

/** Subtitle languages: "English subs", "Subs: English, Hindi", or "Subtitles" when unknown. */
export function subtitleLabel(s: Source): string | undefined {
  const names = [...new Set((s.subtitles ?? []).map((t) => (t.lang ? languageName(t.lang) : t.label)).filter(Boolean))];
  if (names.length === 1) return `${names[0]} subs`;
  if (names.length > 1) return `Subs: ${names.slice(0, 3).join(", ")}${names.length > 3 ? ` +${names.length - 3}` : ""}`;
  return s.tags.has_subs ? "Subtitles" : undefined;
}

const RELEASE_TAGS: [RegExp, string][] = [
  [/\bweb[-. ]?dl\b/i, "WEB-DL"],
  [/\bweb[-. ]?rip\b/i, "WEBRip"],
  [/\bblu[-. ]?ray\b|\bbdrip\b/i, "BluRay"],
  [/\bhdtv\b/i, "HDTV"],
  [/\bds4k\b/i, "DS4K"],
  [/\b(?:ddp?|dd\+|eac3)[. ]?5[. ]1\b/i, "DD+ 5.1"],
  [/\batmos\b/i, "Atmos"],
  [/\b7[. ]1\b/, "7.1"],
  [/\b5[. ]1\b/, "5.1"],
  [/\b(?:esubs?|msubs?)\b/i, "Subs inside"],
  [/\bdual[-. ]?audio\b/i, "Dual audio"],
  [/\bremux\b/i, "Remux"],
];

/** Clean release tags from a scene file name ("WEB-DL", "5.1", "DS4K"). */
export function releaseTags(s: Source): string[] {
  const name = s.filename ?? s.label;
  if (!name) return [];
  const out: string[] = [];
  for (const [re, tag] of RELEASE_TAGS) {
    if (re.test(name) && !out.includes(tag) && !(tag === "5.1" && out.includes("DD+ 5.1"))) out.push(tag);
  }
  return out.slice(0, 4);
}
