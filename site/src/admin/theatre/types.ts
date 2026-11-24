export interface Screening {
  id: string;
  day: string;
  k: string;
  name: string;
  year: number | null;
  poster: string | null;
  runtime: number;
  kind: "trending" | "pick";
  starts_at: string;
  ends_at: string;
  status: "draft" | "scheduled" | "cancelled";
  mode: "open" | "fixed";
  capacity: number;
  booked: number;
  waitlisted: number;
  proven: boolean;
  opens_at: string | null;
  max_opens_at: string | null;
  special: boolean;
  note: string;
  chat_mode: "normal" | "slow" | "closed";
  released: boolean;
  settled: boolean;
  entered?: number;
}

export interface Settings {
  paused: boolean;
  banner: string;
  reminder: string;
}

export interface Who {
  id: string;
  username: string;
  display_name: string;
  avatar: string;
}

/** 21:30 on Fri, in the admin's own time zone. */
export const local = (iso: string) => new Date(iso).toLocaleString(undefined, { weekday: "short", hour: "2-digit", minute: "2-digit" });
export const utc = (iso: string) => `${new Date(iso).toISOString().slice(11, 16)} UTC`;
export const dayLabel = (iso: string) => new Date(iso).toLocaleDateString(undefined, { weekday: "long", month: "short", day: "numeric" });
export const clockOf = (iso: string) => new Date(iso).toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" });

/** Value for <input type="datetime-local"> from an ISO time, in local time. */
export function toInput(iso: string): string {
  const d = new Date(iso);
  const p = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}T${p(d.getHours())}:${p(d.getMinutes())}`;
}

export const fromInput = (v: string) => (v ? new Date(v).toISOString() : "");

export type Phase = "upcoming" | "lobby" | "live" | "after" | "over";
export function phaseOf(s: Pick<Screening, "starts_at" | "ends_at">, now = Date.now()): Phase {
  const a = Date.parse(s.starts_at);
  const b = Date.parse(s.ends_at);
  if (now < a - 30 * 60_000) return "upcoming";
  if (now < a) return "lobby";
  if (now < b) return "live";
  if (now < b + 30 * 60_000) return "after";
  return "over";
}

export const PHASE_LABEL: Record<Phase, string> = { upcoming: "Upcoming", lobby: "Lobby open", live: "Live", after: "After-chat", over: "Over" };

export interface Movie {
  k: string;
  name: string;
  year: number | null;
  poster: string | null;
  overview?: string;
}
