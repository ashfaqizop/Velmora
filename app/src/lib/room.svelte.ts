// A Theatre screening's live room: who's watching, the chat and reactions.
// Two private Supabase Realtime channels (rules in supabase/theatre.sql):
//   theatre:<id>       presence (who's here) and reactions; we send here
//   theatre-chat:<id>  the chat; we only listen, theatre_say sends after checks
// Nothing is stored: the chat lives in memory while the room is open.
import type { RealtimeChannel } from "@supabase/supabase-js";
import { SvelteSet } from "svelte/reactivity";
import { account } from "./account.svelte";
import { theatre } from "./theatre.svelte";

export interface ChatMessage {
  id: string;
  uid: string;
  username: string;
  display_name: string;
  avatar: string;
  verified: boolean;
  body: string;
  /** Movie time in ms when it was sent; null in the lobby. */
  at: number | null;
  ts: string;
  /** It mentions you. */
  mention: boolean;
}

export interface Watcher {
  uid: string;
  username: string;
  display_name: string;
  avatar: string;
  verified: boolean;
  /** When they came in (ms), for "who got a seat in chat first". */
  at: number;
}

export const REACTIONS = ["😂", "😮", "😭", "🔥", "❤️", "👏", "😱"] as const;
export type Reaction = (typeof REACTIONS)[number];

const MAX_MESSAGES = 200;
const QUIET_RETRY_MS = 2 * 60_000;

function friendly(e: { message?: string; details?: string } | null): string {
  const m = e?.message ?? "";
  if (m.includes("chat_slow")) return `Slow mode: wait ${e?.details ?? "a few"} s.`;
  if (m.includes("chat_link")) return "Links can't be posted in Theatre chat.";
  if (m.includes("chat_blocked")) return "That message has a word that isn't allowed.";
  if (m.includes("chat_repeat")) return "You just said that.";
  if (m.includes("chat_length")) return "Messages can be up to 200 characters.";
  if (m.includes("not_in_screening")) return "The chat for this screening is closed.";
  if (/fetch|network/i.test(m)) return "You seem to be offline.";
  return "Couldn't send. Try again.";
}

class Room {
  id: string | null = null;
  messages = $state<ChatMessage[]>([]);
  watchers = $state<Watcher[]>([]);
  /** Muted for this screening (yours only). */
  muted = new SvelteSet<string>();
  /** The chat is full; you're watching in sync without it for now. */
  quiet = $state(false);
  connected = $state(false);
  failed = $state(false);
  sending = $state(false);
  /** Shown under the box: slow mode, a refused message. */
  notice = $state("");
  /** Reactions arrive here (the page floats them over the video). */
  onReact: ((emoji: Reaction, from: string) => void) | null = null;

  private capacity = 1000;
  private room: RealtimeChannel | null = null;
  private chat: RealtimeChannel | null = null;
  private quietTimer: ReturnType<typeof setTimeout> | undefined;
  private joinedAt = 0;
  private lastReact = 0;

  get me() {
    return account.profile;
  }

  async join(id: string, capacity: number) {
    if (this.id === id && (this.connected || this.quiet)) return;
    await this.leave();
    this.id = id;
    this.capacity = Math.max(1, capacity);
    this.messages = [];
    this.muted.clear();
    this.failed = false;
    this.notice = "";
    this.joinedAt = this.joinedAt || Date.now();
    await this.connect();
  }

  private async connect() {
    const me = this.me;
    const id = this.id;
    if (!me || !id) return;
    const sb = account.client;
    // Private channels check the person's session against the rules.
    await sb.realtime.setAuth((await account.accessToken()) ?? null);

    const room = sb.channel(`theatre:${id}`, {
      config: { private: true, presence: { key: me.id }, broadcast: { self: true } },
    });
    room.on("presence", { event: "sync" }, () => this.syncPresence());
    room.on("broadcast", { event: "react" }, ({ payload }) => {
      const e = payload?.e as Reaction;
      if (REACTIONS.includes(e) && !this.muted.has(String(payload?.u))) this.onReact?.(e, String(payload?.u ?? ""));
    });
    room.subscribe(async (status) => {
      if (status === "SUBSCRIBED") {
        this.connected = true;
        await room.track({
          uid: me.id,
          username: me.username,
          display_name: me.display_name,
          avatar: me.avatar,
          verified: !!me.verified,
          at: this.joinedAt,
        });
      } else if (status === "CHANNEL_ERROR" || status === "TIMED_OUT") {
        this.connected = false;
        this.failed = true;
      }
    });

    const chat = sb.channel(`theatre-chat:${id}`, { config: { private: true } });
    chat.on("broadcast", { event: "chat" }, ({ payload }) => this.receive(payload));
    chat.subscribe();

    this.room = room;
    this.chat = chat;
  }

  private syncPresence() {
    if (!this.room) return;
    const state = this.room.presenceState<Watcher>();
    const seen = new Map<string, Watcher>();
    for (const list of Object.values(state)) {
      for (const w of list) if (w?.uid && !seen.has(w.uid)) seen.set(w.uid, w);
    }
    const friends = theatre.following;
    this.watchers = [...seen.values()].sort((a, b) => Number(friends.has(b.uid)) - Number(friends.has(a.uid)) || a.at - b.at);

    // Over capacity: the latest arrivals watch in sync without chat for now,
    // so the room stays within Velmora's live-connection budget.
    const order = [...seen.values()].sort((a, b) => a.at - b.at);
    const mine = order.findIndex((w) => w.uid === this.me?.id);
    if (mine >= this.capacity) this.goQuiet();
  }

  private async goQuiet() {
    this.quiet = true;
    const id = this.id;
    await this.disconnect();
    clearTimeout(this.quietTimer);
    this.quietTimer = setTimeout(async () => {
      if (this.id !== id) return;
      this.quiet = false;
      await this.connect();
    }, QUIET_RETRY_MS);
  }

  private receive(p: Record<string, unknown>) {
    if (!p?.id || typeof p.body !== "string") return;
    const uid = String(p.uid);
    if (this.muted.has(uid)) return;
    const me = this.me?.username?.toLowerCase();
    const msg: ChatMessage = {
      id: String(p.id),
      uid,
      username: String(p.username ?? ""),
      display_name: String(p.display_name ?? ""),
      avatar: String(p.avatar ?? ""),
      verified: !!p.verified,
      body: p.body,
      at: typeof p.at === "number" ? p.at : null,
      ts: String(p.ts ?? new Date().toISOString()),
      mention: !!me && uid !== this.me?.id && new RegExp(`(^|\\W)@${me}(\\W|$)`, "i").test(p.body),
    };
    if (this.messages.some((m) => m.id === msg.id)) return;
    const next = [...this.messages, msg];
    this.messages = next.length > MAX_MESSAGES ? next.slice(-MAX_MESSAGES) : next;
  }

  async say(body: string, movieMs: number | null): Promise<boolean> {
    const text = body.trim();
    if (!text || !this.id || this.sending) return false;
    this.sending = true;
    this.notice = "";
    try {
      const { error } = await account.client.rpc("theatre_say", { sid: this.id, body: text, movie_ms: movieMs === null ? null : Math.round(movieMs) });
      if (error) {
        this.notice = friendly(error);
        return false;
      }
      return true;
    } finally {
      this.sending = false;
    }
  }

  react(e: Reaction) {
    // A few a second at most; bursts come from many people, not one.
    const now = Date.now();
    if (!this.room || !this.connected || now - this.lastReact < 600) return;
    this.lastReact = now;
    this.room.send({ type: "broadcast", event: "react", payload: { e, u: this.me?.id } });
  }

  mute(uid: string, on: boolean) {
    if (on) {
      this.muted.add(uid);
      this.messages = this.messages.filter((m) => m.uid !== uid);
    } else this.muted.delete(uid);
  }

  private async disconnect() {
    const sb = account.client;
    const room = this.room;
    const chat = this.chat;
    this.room = null;
    this.chat = null;
    this.connected = false;
    if (room) {
      await room.untrack().catch(() => {});
      await sb.removeChannel(room).catch(() => {});
    }
    if (chat) await sb.removeChannel(chat).catch(() => {});
  }

  async leave() {
    clearTimeout(this.quietTimer);
    await this.disconnect();
    this.id = null;
    this.quiet = false;
    this.joinedAt = 0;
    this.watchers = [];
    this.messages = [];
  }
}

export const room = new Room();
