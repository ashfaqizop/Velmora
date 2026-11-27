// Live announcements from the admin site, read straight from Supabase with
// the public key (row level security only returns live ones). Used by the
// main site and Social. The web can't tell Max from Public, so every web
// announcement shows to everyone.
import { SUPABASE_KEY, SUPABASE_URL } from "../auth/config";

export interface Notice {
  id: number;
  kind: "info" | "warning" | "maintenance";
  title: string;
  body: string;
  link_url: string;
  link_label: string;
  updated_at: string;
}

const DISMISSED = "velmora.notices.dismissed";

/** Dismissals are per version, so an edited announcement shows again. */
const stamp = (n: Notice) => `${n.id}@${n.updated_at}`;

function dismissedSet(): Set<string> {
  try {
    return new Set(JSON.parse(localStorage.getItem(DISMISSED) ?? "[]"));
  } catch {
    return new Set();
  }
}

export function dismiss(n: Notice) {
  try {
    const set = dismissedSet();
    set.add(stamp(n));
    localStorage.setItem(DISMISSED, JSON.stringify([...set].slice(-40)));
  } catch {
    // Private window: it just comes back next visit.
  }
}

/** The one announcement to show: maintenance first, then the newest. */
export async function liveNotice(): Promise<Notice | null> {
  try {
    const r = await fetch(
      `${SUPABASE_URL}/rest/v1/announcements?select=id,kind,title,body,link_url,link_label,updated_at&place=in.(web,both)&order=starts_at.desc&limit=5`,
      { headers: { apikey: SUPABASE_KEY }, signal: AbortSignal.timeout(6000) },
    );
    if (!r.ok) return null;
    const rows = (await r.json()) as Notice[];
    const gone = dismissedSet();
    const open = rows.filter((n) => n.kind === "maintenance" || !gone.has(stamp(n)));
    return open.find((n) => n.kind === "maintenance") ?? open[0] ?? null;
  } catch {
    return null;
  }
}
