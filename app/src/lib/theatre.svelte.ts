// Theatre: Velmora's scheduled screenings (spec: docs/theatre.md). This keeps
// the lineup, your tickets and who you follow is going, books and cancels,
// keeps the clock in step with the server, reminds you before a screening,
// and quietly helps check that drafted titles can be played.
import { M3eSnackbar } from "@m3e/web/snackbar";
import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";
import { account } from "./account.svelte";
import * as api from "./api";
import { license } from "./license.svelte";
import { nav } from "./state.svelte";

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
  /** Added by hand from the admin site. */
  special: boolean;
  /** A short public message, such as "Delayed to 9 pm". */
  note: string;
  chat_mode: "normal" | "slow" | "closed";
}

export interface TheatreSettings {
  paused: boolean;
  banner: string;
  reminder: string;
}

export type TicketStatus = "booked" | "waitlist" | "cancelled" | "released" | "attended" | "noshow";

export interface Friend {
  id: string;
  username: string;
  display_name: string;
  avatar: string;
}

/** Where a screening is in its life. */
export type Phase = "upcoming" | "lobby" | "live" | "after" | "over";

const LOBBY_MS = 30 * 60_000;
const AFTER_MS = 30 * 60_000;
const REFRESH_MS = 60_000;
const PROBE_KEY = "velmora.theatre.probe";
const PROBED_KEY = "velmora.theatre.probed";
const REMINDED_KEY = "velmora.theatre.reminded";

const COLUMNS = "id,day,k,name,year,poster,backdrop,runtime,kind,starts_at,ends_at,status,mode,capacity,booked,waitlisted,opens_at,max_opens_at,special,note,chat_mode";

function readSet(key: string): Set<string> {
  try {
    return new Set(JSON.parse(localStorage.getItem(key) ?? "[]"));
  } catch {
    return new Set();
  }
}
function writeSet(key: string, s: Set<string>) {
  try {
    localStorage.setItem(key, JSON.stringify([...s].slice(-200)));
  } catch {
    // Not remembered; at worst a reminder repeats.
  }
}

/** Readable words for the server's refusals. */
function friendly(e: { message?: string } | null | undefined): string {
  const m = e?.message ?? "";
  if (m.includes("paused")) return "Theatre is paused for a bit. Booking will be back soon.";
  if (m.includes("not_open_yet")) return "Booking for this screening hasn't opened yet.";
  if (m.includes("not_bookable")) return "This screening isn't taking bookings.";
  if (m.includes("over")) return "This screening has finished.";
  if (m.includes("sign_in")) return "Sign in to book.";
  if (m.includes("not_allowed")) return "Your account can't book screenings right now.";
  if (/fetch|network/i.test(m)) return "You seem to be offline.";
  return "Something went wrong. Try again.";
}

class Theatre {
  lineup = $state<Screening[]>([]);
  /** Your ticket per screening id. */
  tickets = $state<Record<string, TicketStatus>>({});
  /** People you follow who booked, per screening id. */
  friends = $state<Record<string, Friend[]>>({});
  /** Everyone you follow (for "friends first" in a room). */
  following = $state<Set<string>>(new Set());
  loaded = $state(false);
  failed = $state(false);
  /** Switches from the admin site: pause, banner, reminder wording. */
  settings = $state<TheatreSettings>({ paused: false, banner: "", reminder: "" });
  busy = $state<string | null>(null);
  /** Ticks every 15 s so phases and countdowns move. */
  now = $state(Date.now());
  /** Server time minus this PC's time, in ms. */
  offset = 0;

  /** Settings > Privacy: help check Theatre titles for sources. */
  probing = $state(localStorage.getItem(PROBE_KEY) !== "0");

  private started = false;
  private timers = new Map<string, ReturnType<typeof setTimeout>[]>();

  /** Now, by the server's clock. */
  serverNow() {
    return Date.now() + this.offset;
  }

  start() {
    if (this.started) return;
    this.started = true;
    setInterval(() => (this.now = this.serverNow()), 15_000);
    setInterval(() => this.load(), REFRESH_MS * 5);
    window.addEventListener("online", () => this.load());
    // A moment after start-up, then every few hours, check titles for sources.
    setTimeout(() => this.probe(), 3 * 60_000);
    setInterval(() => this.probe(), 4 * 3_600_000);
    this.load();
  }

  phase(s: Screening, at = this.now): Phase {
    const start = Date.parse(s.starts_at);
    const end = Date.parse(s.ends_at);
    if (at < start - LOBBY_MS) return "upcoming";
    if (at < start) return "lobby";
    if (at < end) return "live";
    if (at < end + AFTER_MS) return "after";
    return "over";
  }

  /** When you can book: Max first, everyone else later. Null when it's open. */
  opensAt(s: Screening): number | null {
    const at = license.max ? (s.max_opens_at ?? s.opens_at) : s.opens_at;
    const t = at ? Date.parse(at) : 0;
    return t > this.now ? t : null;
  }

  seatsLeft(s: Screening) {
    return Math.max(0, s.capacity - s.booked);
  }

  ticket(s: Screening): TicketStatus | null {
    return this.tickets[s.id] ?? null;
  }

  /** Booked or waitlisted for a screening that hasn't finished. */
  holding(s: Screening) {
    const t = this.tickets[s.id];
    return (t === "booked" || t === "waitlist") && s.status === "scheduled" && this.phase(s) !== "over";
  }

  /** A screening you hold a seat for is in its lobby or playing: the sidebar's LIVE pulse. */
  get live(): Screening | null {
    return this.lineup.find((s) => this.tickets[s.id] === "booked" && ["lobby", "live"].includes(this.phase(s))) ?? null;
  }

  /** What's on now or next, for Home and the page's top. */
  get current(): Screening[] {
    return this.lineup.filter((s) => s.status === "scheduled" && this.phase(s) !== "over");
  }

  /** The scheduled screening of this title that hasn't finished, if any. */
  forTitle(k: string | null): Screening | null {
    if (!k) return null;
    return this.current.find((s) => s.k === k) ?? null;
  }

  async load() {
    if (!navigator.onLine) return;
    try {
      const sb = account.client;
      const since = new Date(this.serverNow() - 4 * 3_600_000 - AFTER_MS).toISOString();
      const [lineup, clock, settings] = await Promise.all([
        sb.from("theatre_screenings").select(COLUMNS).gte("ends_at", since).order("starts_at").limit(60),
        this.syncClock(),
        sb.from("theatre_settings").select("paused,banner,reminder").maybeSingle(),
      ]);
      if (lineup.error) throw lineup.error;
      void clock;
      if (settings.data) this.settings = settings.data as TheatreSettings;
      const list = (lineup.data ?? []) as Screening[];
      const before = this.lineup;
      this.lineup = list;
      this.now = this.serverNow();
      this.failed = false;
      await Promise.all([this.loadTickets(list), this.loadFriends(list)]);
      this.announce(before, list);
      this.loaded = true;
      this.schedule();
    } catch {
      this.failed = !this.loaded;
      this.loaded = true;
    }
  }

  /** Ask the server for its time a few times and keep the steadiest answer. */
  private async syncClock() {
    let best: { rtt: number; offset: number } | null = null;
    for (let i = 0; i < 3; i++) {
      const sent = Date.now();
      const { data, error } = await account.client.rpc("theatre_now");
      const got = Date.now();
      if (error || typeof data !== "string") continue;
      const rtt = got - sent;
      const offset = Date.parse(data) + rtt / 2 - got;
      if (!best || rtt < best.rtt) best = { rtt, offset };
    }
    if (best) this.offset = best.offset;
  }

  private async loadTickets(list: Screening[]) {
    const me = account.profile?.id;
    if (!me || !list.length) {
      this.tickets = {};
      return;
    }
    const before = this.tickets;
    const { data } = await account.client
      .from("theatre_tickets")
      .select("screening_id,status")
      .eq("user_id", me)
      .in(
        "screening_id",
        list.map((s) => s.id),
      );
    const next: Record<string, TicketStatus> = {};
    for (const t of (data ?? []) as { screening_id: string; status: TicketStatus }[]) next[t.screening_id] = t.status;
    // Off the waitlist since we last looked: say so.
    for (const [id, status] of Object.entries(next)) {
      if (before[id] === "waitlist" && status === "booked") {
        const s = list.find((x) => x.id === id);
        if (s) this.notify("You got a seat", `A seat freed up for ${s.name}. You're in.`, `seat:${id}`);
      }
    }
    this.tickets = next;
  }

  private async loadFriends(list: Screening[]) {
    const me = account.profile?.id;
    if (!me || !list.length) {
      this.friends = {};
      return;
    }
    const { data: follows } = await account.client.from("follows").select("followee").eq("follower", me).limit(1000);
    const ids = (follows ?? []).map((f: { followee: string }) => f.followee);
    this.following = new Set(ids);
    if (!ids.length) {
      this.friends = {};
      return;
    }
    const { data } = await account.client
      .from("theatre_tickets")
      .select("screening_id, p:profiles!theatre_tickets_user_id_fkey(id,username,display_name,avatar)")
      .in(
        "screening_id",
        list.map((s) => s.id),
      )
      .in("user_id", ids.slice(0, 300))
      .in("status", ["booked", "attended"]);
    const out: Record<string, Friend[]> = {};
    for (const row of (data ?? []) as unknown as { screening_id: string; p: Friend | null }[]) {
      if (row.p) (out[row.screening_id] ??= []).push(row.p);
    }
    this.friends = out;
  }

  async book(s: Screening) {
    if (!account.signedIn) {
      M3eSnackbar.open("Sign in to book.");
      return;
    }
    this.busy = s.id;
    try {
      const { data, error } = await account.client.rpc("theatre_book", { sid: s.id });
      if (error) throw error;
      const result = data as "booked" | "waitlist" | "already";
      if (result === "waitlist") {
        this.tickets = { ...this.tickets, [s.id]: "waitlist" };
        M3eSnackbar.open(`Sold out. You're on the waitlist for ${s.name}; Velmora tells you if a seat frees up.`);
      } else {
        this.tickets = { ...this.tickets, [s.id]: "booked" };
        if (result === "booked") M3eSnackbar.open(`Booked. Velmora reminds you 30 minutes before ${s.name}.`);
      }
      this.askToNotify();
      await this.load();
    } catch (e) {
      M3eSnackbar.open(friendly(e as { message?: string }));
    } finally {
      this.busy = null;
    }
  }

  async cancel(s: Screening) {
    this.busy = s.id;
    try {
      const { error } = await account.client.rpc("theatre_cancel", { sid: s.id });
      if (error) throw error;
      const { [s.id]: _gone, ...rest } = this.tickets;
      this.tickets = rest;
      M3eSnackbar.open("Ticket cancelled.");
      await this.load();
    } catch (e) {
      M3eSnackbar.open(friendly(e as { message?: string }));
    } finally {
      this.busy = null;
    }
  }

  // ── Reminders ──

  /** Tell ticket holders when a screening they hold was cancelled or moved. */
  private announce(before: Screening[], list: Screening[]) {
    const was = new Map(before.map((s) => [s.id, s]));
    for (const s of list) {
      const t = this.tickets[s.id];
      if ((t !== "booked" && t !== "waitlist") || this.phase(s) === "over") continue;
      if (s.status === "cancelled") {
        this.notify("Theatre: screening cancelled", `${s.name} was cancelled.${s.note ? ` ${s.note}` : ""}`, `cancel:${s.id}`);
        continue;
      }
      const old = was.get(s.id);
      if (old && old.starts_at !== s.starts_at) {
        const at = new Date(s.starts_at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
        this.notify("Theatre: new start time", `${s.name} now starts at ${at}.${s.note ? ` ${s.note}` : ""}`, `moved:${s.id}:${s.starts_at}`);
      }
    }
  }

  private async askToNotify() {
    try {
      if (!(await isPermissionGranted())) await requestPermission();
    } catch {
      // Notifications unavailable; in-app snackbars still work.
    }
  }

  private async notify(title: string, body: string, key: string) {
    const done = readSet(REMINDED_KEY);
    if (done.has(key)) return;
    done.add(key);
    writeSet(REMINDED_KEY, done);
    M3eSnackbar.open(body);
    try {
      if (await isPermissionGranted()) sendNotification({ title, body });
    } catch {
      // The snackbar is enough.
    }
  }

  /** Reminders for your booked screenings: when the lobby opens, and 5 minutes before. */
  private schedule() {
    for (const list of this.timers.values()) list.forEach(clearTimeout);
    this.timers.clear();
    for (const s of this.lineup) {
      if (this.tickets[s.id] !== "booked" || s.status !== "scheduled") continue;
      const start = Date.parse(s.starts_at);
      const at = (t: number) => t - this.serverNow();
      const list: ReturnType<typeof setTimeout>[] = [];
      const lobby = at(start - LOBBY_MS);
      const soon = at(start - 5 * 60_000);
      // setTimeout can't wait beyond ~24 days; reloads reschedule anyway.
      if (lobby > -60_000 && lobby < 2 * 86_400_000) {
        list.push(setTimeout(() => this.notify("Theatre: the lobby is open", (this.settings.reminder || "{title} starts in 30 minutes. Chat is open.").replace("{title}", s.name), `lobby:${s.id}:${s.starts_at}`), Math.max(0, lobby)));
      }
      if (soon > -60_000 && soon < 2 * 86_400_000) {
        list.push(setTimeout(() => this.notify("Theatre: starting soon", `${s.name} starts in 5 minutes.`, `soon:${s.id}:${s.starts_at}`), Math.max(0, soon)));
      }
      this.timers.set(s.id, list);
    }
  }

  // ── Checking titles for sources ──

  setProbing(on: boolean) {
    this.probing = on;
    try {
      localStorage.setItem(PROBE_KEY, on ? "1" : "0");
    } catch {
      // Not remembered.
    }
  }

  /**
   * Look for sources for a few drafted titles and report found / not found,
   * anonymously. Only while Velmora is open and nothing is playing; each title
   * once per PC.
   */
  async probe() {
    if (!this.probing || !navigator.onLine || document.hidden || nav.route.name === "player") return;
    try {
      const { titles } = (await api.theatreProbes()) as { titles?: { k: string }[] };
      const done = readSet(PROBED_KEY);
      const todo = (titles ?? []).map((t) => t.k).filter((k) => /^movie\/\d+$/.test(k) && !done.has(k)).slice(0, 3);
      for (const k of todo) {
        // Something started playing meanwhile: stop.
        if (!this.probing || (nav.route.name as string) === "player") break;
        const [title] = await api.tmdbTitles([k]);
        let found = false;
        if (title) {
          const report = await api.sourcesWatch(title, { type: "movie" }, "sub", () => {});
          found = (report.sources?.length ?? 0) > 0;
        }
        await api.theatreProbe(k, found).catch(() => false);
        done.add(k);
        writeSet(PROBED_KEY, done);
      }
    } catch {
      // Next time.
    }
  }
}

export const theatre = new Theatre();
