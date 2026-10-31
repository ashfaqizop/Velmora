// Ani's chats: the list, the open conversation, and sending a message.
// Each chat is saved under its own localStorage key; an index keeps the list.
import { runTurn, type Block, type Step } from "./agent";
import type { Content } from "./gemini";
import { RefBook, type RefData } from "./refs";

export type Turn =
  | { id: string; role: "user"; text: string; at: number }
  | { id: string; role: "ani"; blocks: Block[]; steps: Step[]; error?: string; stopped?: boolean; at: number };

export interface Chat {
  id: string;
  title: string;
  created: number;
  updated: number;
  turns: Turn[];
  /** Answer-stage history sent back to Gemini, thought signatures included. */
  history: Content[];
  refs: RefData;
}

export interface ChatIndex {
  id: string;
  title: string;
  updated: number;
}

const INDEX_KEY = "velmora.ani.chats";
const chatKey = (id: string) => `velmora.ani.chat.${id}`;
const MAX_CHATS = 40;
/** Exchanges the research stage sees as plain text. */
const RECENT_TURNS = 6;

function read<T>(key: string): T | null {
  try {
    const raw = localStorage.getItem(key);
    return raw ? (JSON.parse(raw) as T) : null;
  } catch {
    return null;
  }
}

/** Plain text of an Ani turn, for the research stage and chat titles. */
export function turnText(t: Turn): string {
  if (t.role === "user") return t.text;
  return t.blocks
    .map((b) => (b.kind === "text" ? b.text : b.kind === "ui" ? `[showed ${b.name.replace("render_", "")}]` : ""))
    .join("\n")
    .trim();
}

class Chats {
  list = $state<ChatIndex[]>(read<ChatIndex[]>(INDEX_KEY) ?? []);
  current = $state<Chat | null>(null);
  busy = $state(false);
  private controller: AbortController | null = null;
  private refs: RefBook | null = null;

  /** Refs of the open chat, for the UI blocks. */
  get book(): RefBook | null {
    return this.refs;
  }

  newChat() {
    if (this.busy) this.stop();
    this.current = null;
    this.refs = null;
  }

  open(id: string) {
    if (this.busy) this.stop();
    const chat = read<Chat>(chatKey(id));
    if (!chat) {
      this.list = this.list.filter((c) => c.id !== id);
      this.saveIndex();
      return;
    }
    this.current = chat;
    this.refs = new RefBook(chat.refs);
  }

  remove(id: string) {
    if (this.current?.id === id) this.newChat();
    localStorage.removeItem(chatKey(id));
    this.list = this.list.filter((c) => c.id !== id);
    this.saveIndex();
  }

  clearAll() {
    this.newChat();
    for (const c of this.list) localStorage.removeItem(chatKey(c.id));
    this.list = [];
    this.saveIndex();
  }

  stop() {
    this.controller?.abort();
  }

  async send(text: string) {
    const message = text.trim();
    if (!message || this.busy) return;
    if (!this.current) {
      const now = Date.now();
      this.current = {
        id: crypto.randomUUID(),
        title: message.length > 48 ? `${message.slice(0, 47).trimEnd()}…` : message,
        created: now,
        updated: now,
        turns: [],
        history: [],
        refs: { next: { t: 0, l: 0, p: 0 }, items: {} },
      };
      this.refs = new RefBook();
    }
    const chat = this.current;
    const refs = this.refs ?? (this.refs = new RefBook(chat.refs));
    const recent = chat.turns.slice(-RECENT_TURNS).map((t) => ({ role: t.role, text: turnText(t) }));
    chat.turns.push({ id: crypto.randomUUID(), role: "user", text: message, at: Date.now() });
    chat.turns.push({ id: crypto.randomUUID(), role: "ani", blocks: [], steps: [], at: Date.now() });
    // Work on the proxied copy so the page updates as the reply streams.
    const reply = chat.turns.at(-1) as Extract<Turn, { role: "ani" }>;

    this.busy = true;
    const controller = new AbortController();
    this.controller = controller;
    try {
      const result = await runTurn(
        { message, history: $state.snapshot(chat.history) as Content[], recent, refs },
        {
          onSteps: (steps) => (reply.steps = steps),
          onBlocks: (blocks) => (reply.blocks = blocks),
        },
        controller.signal,
      );
      reply.blocks = result.blocks;
      reply.steps = result.steps;
      chat.history.push(...result.history);
      if (controller.signal.aborted) reply.stopped = true;
    } catch (e) {
      reply.error = e instanceof Error ? e.message : String(e);
      // Keep the question in the history so a retry or follow-up has it.
      chat.history.push({ role: "user", parts: [{ text: message }] }, { role: "model", parts: [{ text: "(no answer)" }] });
    } finally {
      this.busy = false;
      this.controller = null;
      chat.refs = refs.toJSON();
      chat.updated = Date.now();
      this.save(chat);
    }
  }

  /** Ask the last question again after an error or a stop. */
  retry() {
    const chat = this.current;
    if (!chat || this.busy) return;
    const lastUser = [...chat.turns].reverse().find((t) => t.role === "user");
    if (!lastUser || lastUser.role !== "user") return;
    const i = chat.turns.lastIndexOf(lastUser);
    chat.turns.splice(i);
    // Drop that question's history too: from its user text to the end.
    for (let h = chat.history.length - 1; h >= 0; h--) {
      const c = chat.history[h];
      if (c.role === "user" && c.parts.some((p) => p.text === lastUser.text)) {
        chat.history.splice(h);
        break;
      }
    }
    this.send(lastUser.text);
  }

  private save(chat: Chat) {
    const snapshot = $state.snapshot(chat) as Chat;
    try {
      localStorage.setItem(chatKey(chat.id), JSON.stringify(snapshot));
    } catch {
      // Storage full: drop the oldest chats and try once more.
      for (const old of this.list.slice(-5)) if (old.id !== chat.id) localStorage.removeItem(chatKey(old.id));
      this.list = this.list.filter((c) => c.id === chat.id || localStorage.getItem(chatKey(c.id)));
      try {
        localStorage.setItem(chatKey(chat.id), JSON.stringify(snapshot));
      } catch (e) {
        console.warn("chat save failed", e);
      }
    }
    const entry: ChatIndex = { id: chat.id, title: chat.title, updated: chat.updated };
    const rest = this.list.filter((c) => c.id !== chat.id);
    for (const old of rest.slice(MAX_CHATS - 1)) localStorage.removeItem(chatKey(old.id));
    this.list = [entry, ...rest.slice(0, MAX_CHATS - 1)];
    this.saveIndex();
  }

  private saveIndex() {
    localStorage.setItem(INDEX_KEY, JSON.stringify(this.list));
  }
}

export const chats = new Chats();
