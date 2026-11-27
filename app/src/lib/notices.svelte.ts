// Announcements and maintenance messages from the admin site. Read from
// Supabase with the public key; row level security only returns live ones.
// Checked at launch and every 10 minutes. Dismissals are per version, so an
// edited announcement shows again.
import { SUPABASE_KEY, SUPABASE_URL } from "./account.svelte";
import { license } from "./license.svelte";

export interface Notice {
  id: number;
  kind: "info" | "warning" | "maintenance";
  title: string;
  body: string;
  link_url: string;
  link_label: string;
  audience: "everyone" | "public" | "max";
  updated_at: string;
}

const DISMISSED = "velmora.notices.dismissed";
const EVERY = 10 * 60_000;
const stamp = (n: Notice) => `${n.id}@${n.updated_at}`;

class Notices {
  rows = $state<Notice[]>([]);
  private gone = $state(new Set<string>(read()));
  private started = false;

  /** The one to show: maintenance first, then the newest. */
  get current(): Notice | null {
    const tier = license.max ? "max" : "public";
    const open = this.rows.filter((n) => (n.audience === "everyone" || n.audience === tier) && (n.kind === "maintenance" || !this.gone.has(stamp(n))));
    return open.find((n) => n.kind === "maintenance") ?? open[0] ?? null;
  }

  start() {
    if (this.started) return;
    this.started = true;
    void this.refresh();
    setInterval(() => void this.refresh(), EVERY);
  }

  async refresh() {
    try {
      const r = await fetch(
        `${SUPABASE_URL}/rest/v1/announcements?select=id,kind,title,body,link_url,link_label,audience,updated_at&place=in.(app,both)&order=starts_at.desc&limit=5`,
        { headers: { apikey: SUPABASE_KEY }, signal: AbortSignal.timeout(8000) },
      );
      if (r.ok) this.rows = (await r.json()) as Notice[];
    } catch {
      // Offline or Supabase down: keep what we had.
    }
  }

  dismiss(n: Notice) {
    const next = new Set(this.gone);
    next.add(stamp(n));
    this.gone = next;
    try {
      localStorage.setItem(DISMISSED, JSON.stringify([...next].slice(-40)));
    } catch {
      // Not saved; it shows again next launch.
    }
  }
}

function read(): string[] {
  try {
    return JSON.parse(localStorage.getItem(DISMISSED) ?? "[]");
  } catch {
    return [];
  }
}

export const notices = new Notices();
