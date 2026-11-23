// Profile pictures: twelve fox presets, or the photo from Google or Discord.
// The same rules are enforced by the database (supabase/schema.sql), so this
// list and that check have to agree. Mirrored in app/src/lib/avatars.ts.

/** Gradient behind the fox, from and to. Dark enough for the pale fox to stand out. */
export const PRESETS: [string, string][] = [
  ["#3271fe", "#1b3fa8"],
  ["#7c3aed", "#4c1d95"],
  ["#0ea5a4", "#0b5e73"],
  ["#e5484d", "#8e1f3a"],
  ["#f97316", "#b4410a"],
  ["#d946a8", "#7e1f6b"],
  ["#16a34a", "#0b5c34"],
  ["#475569", "#1e293b"],
  ["#0ea5e9", "#1d4ed8"],
  ["#ca8a04", "#7a4b05"],
  ["#6366f1", "#2e1f7a"],
  ["#14b8a6", "#3271fe"],
];

export const presetId = (n: number) => `preset:${n}`;

/** "preset:3" gives its gradient; anything else is a photo. */
export function presetOf(avatar: string): [string, string] | null {
  const m = /^preset:(\d{1,2})$/.exec(avatar);
  return m ? (PRESETS[Number(m[1]) - 1] ?? null) : null;
}

/** Only photos from these hosts are accepted as an avatar. */
export const isPhotoUrl = (u: unknown): u is string =>
  typeof u === "string" && u.length <= 450 && /^https:\/\/(lh3\.googleusercontent\.com|cdn\.discordapp\.com)\/\S+$/.test(u);
