// Social links on a profile. The database checks the same rules
// (public.valid_links in supabase/social.sql), so keep the two in step.

export interface ProfileLink {
  kind: LinkKind;
  url: string;
}

export type LinkKind = "instagram" | "x" | "youtube" | "tiktok" | "twitch" | "github" | "discord" | "letterboxd" | "anilist" | "myanimelist" | "website";

export const LINK_KINDS: { kind: LinkKind; label: string; host: RegExp; example: string }[] = [
  { kind: "instagram", label: "Instagram", host: /instagram\.com/, example: "https://instagram.com/you" },
  { kind: "x", label: "X", host: /(x|twitter)\.com/, example: "https://x.com/you" },
  { kind: "youtube", label: "YouTube", host: /(youtube\.com|youtu\.be)/, example: "https://youtube.com/@you" },
  { kind: "tiktok", label: "TikTok", host: /tiktok\.com/, example: "https://tiktok.com/@you" },
  { kind: "twitch", label: "Twitch", host: /twitch\.tv/, example: "https://twitch.tv/you" },
  { kind: "github", label: "GitHub", host: /github\.com/, example: "https://github.com/you" },
  { kind: "discord", label: "Discord", host: /(discord\.gg|discord\.com)/, example: "https://discord.gg/invite" },
  { kind: "letterboxd", label: "Letterboxd", host: /letterboxd\.com/, example: "https://letterboxd.com/you" },
  { kind: "anilist", label: "AniList", host: /anilist\.co/, example: "https://anilist.co/user/you" },
  { kind: "myanimelist", label: "MyAnimeList", host: /myanimelist\.net/, example: "https://myanimelist.net/profile/you" },
  { kind: "website", label: "Website", host: /[a-z0-9.-]+\.[a-z]{2,}/, example: "https://your.site" },
];

export const MAX_LINKS = 6;
export const MAX_BIO = 160;

export const linkLabel = (kind: string) => LINK_KINDS.find((k) => k.kind === kind)?.label ?? "Link";

/** Is this URL right for this kind of link? */
export function validLink(l: ProfileLink): boolean {
  const k = LINK_KINDS.find((x) => x.kind === l.kind);
  if (!k || l.url.length > 200) return false;
  return new RegExp(`^https://([a-z0-9-]+\.)*${k.host.source}(/[^\s<>"]*)?$`, "i").test(l.url);
}

/** "instagram.com/you" for showing a link without the https:// noise. */
export const shortUrl = (url: string) => url.replace(/^https:\/\/(www\.)?/, "").replace(/\/$/, "");
