// Display helpers for engine sources, shared by the player and download picker.
import { providerLabel } from "./options";
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
