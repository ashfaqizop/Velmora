// Following a Theatre screening's chat from Velmora Social: messages and
// reactions arrive live, and the watcher count, but the web only listens.
// Chatting and reacting happen in the Velmora app. Same private Realtime
// channels and rules as the app (supabase/theatre.sql): open screenings, or
// a ticket for ticketed ones. The web never announces itself as watching.
import type { RealtimeChannel } from "@supabase/supabase-js";
import { supabase } from "../lib/session";
import { auth } from "./data.svelte";

export interface LiveMessage {
  id: string;
  uid: string;
  username: string;
  display_name: string;
  avatar: string;
  verified: boolean;
  body: string;
  at: number | null;
  mention: boolean;
}

export interface LiveWatcher {
  uid: string;
  username: string;
  display_name: string;
  avatar: string;
}

const REACTIONS = new Set(["😂", "😮", "😭", "🔥", "❤️", "👏", "😱"]);
const MAX = 200;

class LiveRoom {
  id = $state<string | null>(null);
  messages = $state<LiveMessage[]>([]);
  watchers = $state<LiveWatcher[]>([]);
  status = $state<"idle" | "connecting" | "live" | "denied">("idle");
  onReact: ((e: string) => void) | null = null;

  private room: RealtimeChannel | null = null;
  private chat: RealtimeChannel | null = null;

  async open(id: string) {
    if (this.id === id) return;
    await this.close();
    this.id = id;
    this.messages = [];
    this.watchers = [];
    this.status = "connecting";
    const { data } = await supabase.auth.getSession();
    await supabase.realtime.setAuth(data.session?.access_token ?? null);

    const room = supabase.channel(`theatre:${id}`, { config: { private: true } });
    room.on("presence", { event: "sync" }, () => {
      const seen = new Map<string, LiveWatcher>();
      for (const list of Object.values(room.presenceState<LiveWatcher>())) for (const w of list) if (w?.uid && !seen.has(w.uid)) seen.set(w.uid, w);
      this.watchers = [...seen.values()];
    });
    room.on("broadcast", { event: "react" }, ({ payload }) => {
      if (REACTIONS.has(payload?.e)) this.onReact?.(payload.e);
    });
    room.subscribe((s) => {
      if (s === "SUBSCRIBED") this.status = "live";
      else if (s === "CHANNEL_ERROR") this.status = "denied";
    });

    const chat = supabase.channel(`theatre-chat:${id}`, { config: { private: true } });
    chat.on("broadcast", { event: "chat" }, ({ payload }) => this.receive(payload));
    chat.subscribe();

    this.room = room;
    this.chat = chat;
  }

  private receive(p: Record<string, unknown>) {
    if (!p?.id || typeof p.body !== "string") return;
    const me = auth.me?.username?.toLowerCase();
    const m: LiveMessage = {
      id: String(p.id),
      uid: String(p.uid),
      username: String(p.username ?? ""),
      display_name: String(p.display_name ?? ""),
      avatar: String(p.avatar ?? ""),
      verified: !!p.verified,
      body: p.body,
      at: typeof p.at === "number" ? p.at : null,
      mention: !!me && new RegExp(`(^|\\W)@${me}(\\W|$)`, "i").test(p.body),
    };
    if (this.messages.some((x) => x.id === m.id)) return;
    const next = [...this.messages, m];
    this.messages = next.length > MAX ? next.slice(-MAX) : next;
  }

  async close() {
    const room = this.room;
    const chat = this.chat;
    this.room = null;
    this.chat = null;
    this.id = null;
    this.status = "idle";
    if (room) await supabase.removeChannel(room).catch(() => {});
    if (chat) await supabase.removeChannel(chat).catch(() => {});
  }
}

export const liveRoom = new LiveRoom();
