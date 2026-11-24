// Theatre on Velmora Social: the lineup, your tickets, who you follow is
// going, and booking from the web. Watching happens in the Velmora app; the
// same server rules apply here (supabase/theatre.sql), including Max booking
// six hours early and moving to the front of waitlists.
import { supabase } from "../lib/session";
import { auth, friendly as baseFriendly, maxIds } from "./data.svelte";

export interface Screening {
  id: string;
  day: string;
  k: string;
  name: string;
  year: number | null;
  poster: string | null;
  backdrop: string | null;
  runtime: number;
  kind: "trending" | "pick";
  starts_at: string;
  ends_at: string;
  status: "scheduled" | "cancelled";
  mode: "open" | "fixed";
  capacity: number;
  booked: number;
  waitlisted: number;
  opens_at: string | null;
  max_opens_at: string | null;
  special: boolean;
  note: string;
}

export interface TheatreSettings {
  paused: boolean;
  banner: string;
}

export type TicketStatus = "booked" | "waitlist" | "cancelled" | "released" | "attended" | "noshow";
export type Phase = "upcoming" | "lobby" | "live" | "after" | "over";

export interface Goer {
  id: string;
  username: string;
  display_name: string;
  avatar: string;
}

const LOBBY = 30 * 60_000;
const AFTER = 30 * 60_000;
const COLUMNS = "id,day,k,name,year,poster,backdrop,runtime,kind,starts_at,ends_at,status,mode,capacity,booked,waitlisted,opens_at,max_opens_at,special,note";

function friendly(e: { message?: string } | null): string {
  const m = e?.message ?? "";
  if (m.includes("paused")) return "Theatre is paused for a bit. Booking will be back soon.";
  if (m.includes("not_open_yet")) return "Booking for this screening hasn't opened yet.";
  if (m.includes("not_bookable")) return "This screening isn't taking bookings.";
  if (m.includes("over")) return "This screening has finished.";
  if (m.includes("not_allowed")) return "Your account can't book screenings right now.";
  return baseFriendly(e, "Couldn't book. Try again.");
}

class Theatre {
  lineup = $state<Screening[]>([]);
  tickets = $state<Record<string, TicketStatus>>({});
  friends = $state<Record<string, Goer[]>>({});
  loaded = $state(false);
  failed = $state(false);
  settings = $state<TheatreSettings>({ paused: false, banner: "" });
  busy = $state<string | null>(null);
  message = $state("");
  now = $state(Date.now());
  offset = 0;
  private started = false;

  serverNow() {
    return Date.now() + this.offset;
  }

  start() {
    if (this.started) return;
    this.started = true;
    setInterval(() => (this.now = this.serverNow()), 1000);
    setInterval(() => this.load(), 5 * 60_000);
    this.load();
  }

  phase(s: Screening, at = this.now): Phase {
    const start = Date.parse(s.starts_at);
    const end = Date.parse(s.ends_at);
    if (at < start - LOBBY) return "upcoming";
    if (at < start) return "lobby";
    if (at < end) return "live";
    if (at < end + AFTER) return "after";
    return "over";
  }

  get current() {
    return this.lineup.filter((s) => s.status === "scheduled" && this.phase(s) !== "over");
  }

  isMax() {
    return !!auth.me && maxIds.has(auth.me.id);
  }

  opensAt(s: Screening): number | null {
    const at = this.isMax() ? (s.max_opens_at ?? s.opens_at) : s.opens_at;
    const t = at ? Date.parse(at) : 0;
    return t > this.now ? t : null;
  }

  seatsLeft(s: Screening) {
    return Math.max(0, s.capacity - s.booked);
  }

  /** Screenings you hold a ticket for that were cancelled. */
  get cancelledMine(): Screening[] {
    return this.lineup.filter((s) => s.status === "cancelled" && ["booked", "waitlist"].includes(this.tickets[s.id] ?? "") && this.phase(s) !== "over");
  }

  /** Your next booked screening that is in its lobby or playing (for the banner). */
  get soon(): Screening | null {
    return (
      this.current.find((s) => this.tickets[s.id] === "booked" && ["lobby", "live"].includes(this.phase(s))) ??
      this.current.find((s) => this.tickets[s.id] === "booked" && Date.parse(s.starts_at) - this.now < 2 * 3_600_000) ??
      null
    );
  }

  async load() {
    try {
      const t0 = Date.now();
      const clock = await supabase.rpc("theatre_now");
      if (typeof clock.data === "string") this.offset = Date.parse(clock.data) + (Date.now() - t0) / 2 - Date.now();
      const since = new Date(this.serverNow() - 4 * 3_600_000 - AFTER).toISOString();
      const { data, error } = await supabase.from("theatre_screenings").select(COLUMNS).gte("ends_at", since).order("starts_at").limit(60);
      if (error) throw error;
      const s = await supabase.from("theatre_settings").select("paused,banner").maybeSingle();
      if (s.data) this.settings = s.data as TheatreSettings;
      this.lineup = (data ?? []) as Screening[];
      this.now = this.serverNow();
      this.failed = false;
      await Promise.all([this.loadTickets(), this.loadFriends()]);
    } catch {
      this.failed = !this.loaded;
    } finally {
      this.loaded = true;
    }
  }

  private async loadTickets() {
    const me = auth.me?.id;
    if (!me || !this.lineup.length) {
      this.tickets = {};
      return;
    }
    const { data } = await supabase
      .from("theatre_tickets")
      .select("screening_id,status")
      .eq("user_id", me)
      .in(
        "screening_id",
        this.lineup.map((s) => s.id),
      );
    const out: Record<string, TicketStatus> = {};
    for (const t of (data ?? []) as { screening_id: string; status: TicketStatus }[]) out[t.screening_id] = t.status;
    this.tickets = out;
  }

  private async loadFriends() {
    const me = auth.me?.id;
    if (!me || !this.lineup.length) {
      this.friends = {};
      return;
    }
    const { data: follows } = await supabase.from("follows").select("followee").eq("follower", me).limit(1000);
    const ids = (follows ?? []).map((f: { followee: string }) => f.followee);
    if (!ids.length) {
      this.friends = {};
      return;
    }
    const { data } = await supabase
      .from("theatre_tickets")
      .select("screening_id, p:profiles!theatre_tickets_user_id_fkey(id,username,display_name,avatar)")
      .in(
        "screening_id",
        this.lineup.map((s) => s.id),
      )
      .in("user_id", ids.slice(0, 300))
      .in("status", ["booked", "attended"]);
    const out: Record<string, Goer[]> = {};
    for (const row of (data ?? []) as unknown as { screening_id: string; p: Goer | null }[]) if (row.p) (out[row.screening_id] ??= []).push(row.p);
    this.friends = out;
  }

  async book(s: Screening): Promise<string> {
    this.busy = s.id;
    try {
      const { data, error } = await supabase.rpc("theatre_book", { sid: s.id });
      if (error) throw error;
      const r = data as "booked" | "waitlist" | "already";
      this.tickets = { ...this.tickets, [s.id]: r === "waitlist" ? "waitlist" : "booked" };
      await this.load();
      return r === "waitlist"
        ? `Sold out, so you're on the waitlist for ${s.name}. The Velmora app tells you if a seat frees up.`
        : `Booked ${s.name}. The Velmora app reminds you 30 minutes before.`;
    } catch (e) {
      return friendly(e as { message?: string });
    } finally {
      this.busy = null;
    }
  }

  async cancel(s: Screening): Promise<string> {
    this.busy = s.id;
    try {
      const { error } = await supabase.rpc("theatre_cancel", { sid: s.id });
      if (error) throw error;
      const { [s.id]: _gone, ...rest } = this.tickets;
      this.tickets = rest;
      await this.load();
      return "Ticket cancelled.";
    } catch (e) {
      return friendly(e as { message?: string });
    } finally {
      this.busy = null;
    }
  }
}

export const theatre = new Theatre();

// ── Shared helpers ──

export const clock = (iso: string | number) => new Date(iso).toLocaleTimeString([], { hour: "numeric", minute: "2-digit" });

export function dayLabel(iso: string | number, now: number): string {
  const d = new Date(iso);
  const startOf = (x: Date) => new Date(x.getFullYear(), x.getMonth(), x.getDate()).getTime();
  const diff = Math.round((startOf(d) - startOf(new Date(now))) / 86_400_000);
  if (diff === 0) return "Today";
  if (diff === 1) return "Tomorrow";
  if (diff === -1) return "Yesterday";
  return d.toLocaleDateString([], { weekday: "long", day: "numeric", month: "short" });
}

export function span(ms: number): string {
  const m = Math.max(0, Math.round(ms / 60_000));
  if (m < 1) return "under a minute";
  if (m < 60) return `${m} min`;
  const h = Math.floor(m / 60);
  return m % 60 ? `${h} h ${m % 60} min` : `${h} h`;
}

export const runtime = (min: number) => (min >= 60 ? `${Math.floor(min / 60)} h ${min % 60 ? `${min % 60} min` : ""}`.trim() : `${min} min`);

/** Open a screening in the app (the website's /open/ page tries velmora:// first). */
export const openInApp = (id: string) => `https://getvelmora.dpdns.org/open/?to=theatre/${id}`;
