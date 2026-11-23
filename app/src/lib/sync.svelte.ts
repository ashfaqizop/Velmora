// Keeping My List, watch progress and settings the same on every PC you sign in on.
//
// Local first: the app reads and writes its own copy, so it is instant and works
// offline. Changes made here are noted (library.onChange) and pushed in the
// background; what other PCs changed is pulled in. When two PCs edited the same
// thing, the newest edit wins. Removals travel as marks, so they reach the
// other PCs too. The database side is supabase/sync.sql.
//
// Settings are compared as a whole: when what this PC holds differs from what
// was last synced, that is an edit made here.
import { M3eSnackbar } from "@m3e/web/snackbar";
import { account } from "./account.svelte";
import { finder } from "./finder.svelte";
import { motion } from "./motion.svelte";
import { keyOf, library, settings } from "./state.svelte";
import { chunk, cleanSettings, dirtyEntries, pickPrefs, pruneTombstones, settle, type Meta, type MetaMap, type SyncedSettings } from "./syncmerge";
import type { HistoryEntry, Preferences, TitleSummary } from "./types";

const STORE = "velmora.sync";
const ON = "velmora.sync.on";
const PAGE = 500;
/** Progress changes every few seconds while watching; send them in batches this far apart. */
const DELAY = 30_000;
const EVERY = 5 * 60_000;

interface Store {
  /** Whose data this PC's list and history are in step with. */
  owner: string | null;
  list: MetaMap;
  progress: MetaMap;
  /** Settings as last synced (JSON), when they were last edited, and whether they still have to be pushed. */
  settingsBlob: string;
  settingsAt: number;
  settingsDirty: boolean;
  cursors: { list: string; progress: string };
  lastSync: number;
}

const fresh = (owner: string | null = null): Store => ({
  owner,
  list: {},
  progress: {},
  settingsBlob: "",
  settingsAt: 0,
  settingsDirty: false,
  cursors: { list: "", progress: "" },
  lastSync: 0,
});

function read(): Store {
  try {
    const v = JSON.parse(localStorage.getItem(STORE) ?? "null");
    return v && typeof v === "object" ? { ...fresh(), ...v, cursors: { ...fresh().cursors, ...v.cursors } } : fresh();
  } catch {
    return fresh();
  }
}

export type SyncStatus = "off" | "idle" | "syncing" | "error";

class Sync {
  /** Settings: keep my data in step across PCs. */
  enabled = $state(localStorage.getItem(ON) !== "0");
  status = $state<SyncStatus>("idle");
  error = $state<string | null>(null);
  lastSync = $state(0);

  private store = read();
  private busy = false;
  private timer: ReturnType<typeof setTimeout> | undefined;

  constructor() {
    this.lastSync = this.store.lastSync;
    library.onChange = (e) => this.touch(e.table, e.key, e.deleted);
  }

  /** Call once at startup. */
  start() {
    window.addEventListener("online", () => this.schedule(2000));
    window.addEventListener("focus", () => this.lastSync && Date.now() - this.lastSync > 60_000 && this.schedule(1000));
    setInterval(() => this.schedule(0), EVERY);
  }

  setEnabled(on: boolean) {
    this.enabled = on;
    try {
      localStorage.setItem(ON, on ? "1" : "0");
    } catch {
      // Not remembered.
    }
    if (on) this.schedule(500);
    else this.status = "off";
  }

  private save() {
    this.store.lastSync = this.lastSync;
    try {
      localStorage.setItem(STORE, JSON.stringify(this.store));
    } catch {
      // The next run starts from what the server has.
    }
  }

  /** Something changed on this PC. */
  private touch(table: "list" | "progress", key: string, deleted: boolean) {
    // A guest who never signs in doesn't need a record of every change.
    if (!account.signedIn && !this.store.owner) return;
    this.store[table][key] = { at: Date.now(), del: deleted, dirty: true };
    this.save();
    this.schedule(DELAY);
  }

  /** Run soon (at most one pending run; an earlier request wins). */
  schedule(delay: number) {
    if (!this.enabled) return;
    clearTimeout(this.timer);
    this.timer = setTimeout(() => this.run(), delay);
  }

  /** Is there something on this PC the server hasn't got? */
  get pending(): boolean {
    return this.store.settingsDirty || dirtyEntries(this.store.list).length > 0 || dirtyEntries(this.store.progress).length > 0;
  }

  /** Sync now. Quiet when signed out, offline or switched off. */
  async run(manual = false) {
    if (!this.enabled || this.busy || !account.signedIn || !account.profile) return;
    if (account.offline || !navigator.onLine) {
      if (manual) this.error = "You're offline. Changes sync when you're back online.";
      return;
    }
    this.busy = true;
    this.status = "syncing";
    this.error = null;
    try {
      const uid = account.profile.id;
      this.adoptOwner(uid);
      await settings.load();
      this.noticeSettings();
      await this.pullList();
      await this.pullProgress();
      await this.pullSettings(uid);
      this.adoptLocal();
      await this.pushAll();
      pruneTombstones(this.store.list);
      pruneTombstones(this.store.progress);
      this.lastSync = Date.now();
      this.status = "idle";
    } catch (e) {
      this.status = "error";
      this.error = (e as Error).message || "Couldn't sync. It will try again.";
      this.schedule(2 * 60_000);
    } finally {
      this.busy = false;
      this.save();
    }
  }

  /** A different account on this PC must not inherit the last one's list. */
  private adoptOwner(uid: string) {
    if (this.store.owner && this.store.owner !== uid) {
      library.clear();
      this.store = fresh(uid);
      this.lastSync = 0;
    } else if (!this.store.owner) {
      this.store.owner = uid;
    }
  }

  // ── Pulling ──

  private listAt = (key: string): number => this.store.list[key]?.at ?? (library.watchlist.some((w) => keyOf(w) === key) ? 0 : -1);

  private progressAt = (key: string): number => {
    const entry = library.history.find((h) => keyOf(h.title) === key);
    return Math.max(this.store.progress[key]?.at ?? -1, entry?.updatedAt ?? -1);
  };

  private async pages<T extends { synced_at: string }>(
    table: "list_items" | "progress",
    columns: string,
    cursor: string,
    each: (rows: T[]) => void,
  ): Promise<string> {
    let at = cursor;
    for (;;) {
      let q = account.client.from(table).select(columns);
      // >= so rows sharing the last timestamp aren't missed; they apply twice at worst, harmlessly.
      if (at) q = q.gte("synced_at", at);
      const { data, error } = await q.order("synced_at", { ascending: true }).limit(PAGE);
      if (error) throw new Error(error.message);
      const rows = (data ?? []) as unknown as T[];
      if (!rows.length) return at;
      each(rows);
      const last = rows[rows.length - 1].synced_at;
      const same = last === at;
      at = last;
      if (rows.length < PAGE || same) return at;
    }
  }

  private async pullList() {
    type Row = { title_key: string; title: TitleSummary | null; updated_at: string; deleted: boolean; synced_at: string };
    this.store.cursors.list = await this.pages<Row>("list_items", "title_key,title,updated_at,deleted,synced_at", this.store.cursors.list, (rows) => {
      for (const r of rows) {
        const at = Date.parse(r.updated_at);
        if (!(at > this.listAt(r.title_key))) continue;
        const live = !r.deleted && r.title && typeof r.title === "object" && r.title.origin?.id !== undefined;
        this.store.list[r.title_key] = { at, del: !live, dirty: false };
        library.applyList(r.title_key, live ? r.title : null, (k) => this.store.list[k]?.at ?? 0);
      }
    });
  }

  private async pullProgress() {
    type Row = { title_key: string; entry: HistoryEntry | null; updated_at: string; deleted: boolean; synced_at: string };
    this.store.cursors.progress = await this.pages<Row>("progress", "title_key,entry,updated_at,deleted,synced_at", this.store.cursors.progress, (rows) => {
      for (const r of rows) {
        const at = Date.parse(r.updated_at);
        if (!(at > this.progressAt(r.title_key))) continue;
        const e = r.entry;
        const live = !r.deleted && e && typeof e === "object" && e.title?.origin?.id !== undefined && typeof e.position === "number" && typeof e.duration === "number";
        this.store.progress[r.title_key] = { at, del: !live, dirty: false };
        library.applyProgress(r.title_key, live ? { ...(e as HistoryEntry), updatedAt: at } : null);
      }
    });
  }

  /** What this PC holds that isn't in the server yet gets sent, stamped now. */
  private adoptLocal() {
    const now = Date.now();
    for (const w of library.watchlist) {
      const k = keyOf(w);
      if (!this.store.list[k]) this.store.list[k] = { at: now, del: false, dirty: true };
    }
    for (const h of library.history) {
      const k = keyOf(h.title);
      if (!this.store.progress[k]) this.store.progress[k] = { at: h.updatedAt, del: false, dirty: true };
    }
  }

  // ── Settings ──

  private collectSettings(): SyncedSettings {
    return {
      v: 1,
      prefs: pickPrefs(settings.prefs as unknown as Record<string, unknown> | null),
      accent: settings.accent,
      audioMode: settings.audioMode,
      motion: motion.pref,
      sourcesAhead: finder.ahead,
    };
  }

  /** Did the settings change here since they were last synced? */
  private noticeSettings() {
    const blob = JSON.stringify(this.collectSettings());
    if (!this.store.settingsBlob) {
      // Never synced on this PC: the account's settings (if it has any) come first.
      this.store.settingsBlob = blob;
      this.store.settingsAt = 0;
    } else if (blob !== this.store.settingsBlob) {
      this.store.settingsBlob = blob;
      this.store.settingsAt = Date.now();
      this.store.settingsDirty = true;
    }
  }

  private async pullSettings(uid: string) {
    const { data, error } = await account.client
      .from("user_settings")
      .select("data,updated_at")
      .eq("user_id", uid)
      .maybeSingle();
    if (error) throw new Error(error.message);
    if (!data) {
      // Nothing in the account yet: this PC's settings are the first.
      if (this.store.settingsAt === 0) {
        this.store.settingsAt = Date.now();
        this.store.settingsDirty = true;
      }
      return;
    }
    const at = Date.parse(data.updated_at as string);
    if (!(at > this.store.settingsAt)) return;
    const clean = cleanSettings(data.data);
    if (!clean) return;
    const first = this.store.settingsAt === 0;
    try {
      await this.applySettings(clean);
    } catch (e) {
      console.warn("sync: couldn't apply settings", e);
      return;
    }
    this.store.settingsAt = at;
    this.store.settingsBlob = JSON.stringify(this.collectSettings());
    this.store.settingsDirty = false;
    if (first) M3eSnackbar.open("Your settings were restored from your account.");
  }

  private async applySettings(s: SyncedSettings) {
    if (Object.keys(s.prefs).length) await settings.update(s.prefs as Partial<Preferences>);
    if (s.accent) settings.setAccent(s.accent);
    if (s.audioMode) settings.setAudioMode(s.audioMode);
    if (s.motion) motion.set(s.motion);
    if (s.sourcesAhead !== undefined) finder.setAhead(s.sourcesAhead);
  }

  // ── Pushing ──

  private async rpc(name: string, args: Record<string, unknown>) {
    const { error } = await account.client.rpc(name, args);
    if (error) throw new Error(error.message);
  }

  private async pushAll() {
    const lists = dirtyEntries(this.store.list);
    for (const part of chunk(lists, 200)) {
      const rows = part.map(([key, m]) => this.listRow(key, m));
      await this.rpc("sync_push_list", { rows });
      settle(this.store.list, part);
    }
    const progress = dirtyEntries(this.store.progress);
    for (const part of chunk(progress, 100)) {
      const rows = part.map(([key, m]) => this.progressRow(key, m));
      await this.rpc("sync_push_progress", { rows });
      settle(this.store.progress, part);
    }
    if (this.store.settingsDirty) {
      const at = this.store.settingsAt;
      await this.rpc("sync_push_settings", { new_data: this.collectSettings(), edited_at: new Date(at).toISOString() });
      if (this.store.settingsAt === at) this.store.settingsDirty = false;
    }
  }

  private listRow(key: string, m: Meta) {
    const title = m.del ? null : (library.watchlist.find((w) => keyOf(w) === key) ?? null);
    return { title_key: key, title: title ? $state.snapshot(title) : null, updated_at: new Date(m.at).toISOString(), deleted: m.del || !title };
  }

  private progressRow(key: string, m: Meta) {
    const entry = m.del ? null : (library.history.find((h) => keyOf(h.title) === key) ?? null);
    return { title_key: key, entry: entry ? $state.snapshot(entry) : null, updated_at: new Date(m.at).toISOString(), deleted: m.del || !entry };
  }

  /** The account was deleted: forget what was synced (the list and history stay on this PC). */
  forgetAccount() {
    this.store = fresh();
    this.lastSync = 0;
    this.save();
  }
}

export const sync = new Sync();
