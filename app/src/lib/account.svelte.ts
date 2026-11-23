// Your Velmora account. Signing in happens in the browser (the auth site); the
// Rust side (src-tauri/src/account.rs) catches the result and keeps the refresh
// token in Credential Manager. This module turns that into a Supabase session,
// loads the profile, and keeps both fresh.
//
// Guest stays fully usable: nothing in the app requires an account.
import { createClient, isAuthRetryableFetchError, type Session, type SupabaseClient } from "@supabase/supabase-js";
import * as api from "./api";

// Public by design: the publishable key can only do what the row level
// security rules in supabase/schema.sql allow. Keep in step with
// site/src/auth/config.ts and site/api/_lib/config.js.
export const SUPABASE_URL = "https://htvirqckkhfekgfwxops.supabase.co";
export const SUPABASE_KEY = "sb_publishable_YSf01mWc6H2QFTxcBQcCMA_lElqBvgQ";

export interface Profile {
  id: string;
  username: string;
  display_name: string;
  /** "preset:1" to "preset:12", or a Google or Discord photo URL. */
  avatar: string;
  username_changed_at: string | null;
  created_at: string;
}

export type AccountStatus = "loading" | "guest" | "signing-in" | "signed-in";

export const USERNAME = /^[A-Za-z0-9_]{3,20}$/;

// The last profile we saw, so a launch without internet still knows who you
// are. It is only a picture of the profile; the real session is checked once
// the network is back.
const CACHE = "velmora.account.cache";

function readCache(): { profile: Profile; email: string | null } | null {
  try {
    const v = JSON.parse(localStorage.getItem(CACHE) ?? "null");
    return v?.profile?.id && v.profile.username ? v : null;
  } catch {
    return null;
  }
}

function writeCache(profile: Profile | null, email: string | null) {
  try {
    if (profile) localStorage.setItem(CACHE, JSON.stringify({ profile, email }));
    else localStorage.removeItem(CACHE);
  } catch {
    // Not remembered; the next online launch fills it in.
  }
}

/** Give a network call a deadline, so a bad connection can't hold the app up. */
function within<T>(ms: number, p: Promise<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    const t = setTimeout(() => reject(new Error("network timeout")), ms);
    p.then(
      (v) => (clearTimeout(t), resolve(v)),
      (e) => (clearTimeout(t), reject(e)),
    );
  });
}

/** Turn a database or auth error into a sentence for the person. */
function describe(e: unknown): string {
  const err = e as { code?: string; message?: string } | string | null;
  const text = typeof err === "string" ? err : (err?.message ?? "");
  const code = typeof err === "object" && err ? err.code : undefined;
  if (code === "23505") return "That username is taken.";
  if (text.includes("username_reserved")) return "That name isn't available.";
  if (text.includes("username_cooldown")) return "You changed your username recently. You can change it again 30 days after the last change.";
  if (code === "23514") return "One of those values isn't allowed.";
  if (/network|fetch|failed to/i.test(text)) return "Couldn't reach Velmora's server. Check your connection and try again.";
  return text || "Something went wrong. Try again.";
}

class Account {
  status = $state<AccountStatus>("loading");
  profile = $state<Profile | null>(null);
  email = $state<string | null>(null);
  /** The last problem, for the UI to show. */
  error = $state<string | null>(null);
  /** Signed in from what was saved, but the server hasn't confirmed it yet (no internet). */
  offline = $state(false);

  private sb: SupabaseClient | null = null;
  private lastSaved = "";
  /** A typed code is finishing the sign-in, so the browser route ending is not a cancel. */
  private redeeming = false;

  get signedIn() {
    return this.status === "signed-in" && !!this.profile;
  }

  /** The Supabase client, for features that talk to the database (sync, chat). */
  get client(): SupabaseClient {
    if (!this.sb) {
      this.sb = createClient(SUPABASE_URL, SUPABASE_KEY, {
        // The refresh token lives in Credential Manager, not in the page.
        auth: { persistSession: false, autoRefreshToken: true, detectSessionInUrl: false },
      });
      this.sb.auth.onAuthStateChange((event, session) => {
        // No supabase calls in here (they can deadlock); only our own commands.
        if (session?.refresh_token) this.remember(session.refresh_token);
        if (event === "SIGNED_OUT" && this.status === "signed-in") {
          this.profile = null;
          this.email = null;
          this.offline = false;
          writeCache(null, null);
          this.status = "guest";
          api.accountVaultClear().catch(() => {});
        }
      });
    }
    return this.sb;
  }

  private remember(token: string) {
    if (token === this.lastSaved) return;
    this.lastSaved = token;
    api.accountVaultWrite(token).catch((e) => console.warn("account: couldn't keep the session", e));
  }

  private retry: ReturnType<typeof setTimeout> | undefined;
  private waitingForNetwork = false;

  /**
   * At startup: pick up where the last launch left off. Never blocks the app.
   * With a saved sign-in you're shown as signed in straight away, from the
   * cached profile; the server confirms it in the background. No internet is
   * fine: you stay signed in and it reconnects by itself.
   */
  async init() {
    clearTimeout(this.retry);
    let token: string | null = null;
    try {
      token = await api.accountVaultRead();
    } catch {
      // No vault, so no saved sign-in.
    }
    if (!token) {
      writeCache(null, null);
      this.status = "guest";
      return;
    }
    this.lastSaved = token;

    const cached = readCache();
    if (cached && this.status !== "signed-in") {
      this.profile = cached.profile;
      this.email = cached.email;
      this.status = "signed-in";
      this.offline = true;
    }

    try {
      const { data, error } = await within(10_000, this.client.auth.refreshSession({ refresh_token: token }));
      if (error || !data.session) throw error ?? new Error("no session");
      this.offline = false;
      await this.finish(data.session);
    } catch (e) {
      const err = e as Error;
      // Anything but "can't reach the server" means the sign-in is really gone
      // (signed out elsewhere, account deleted): drop it.
      const unreachable = isAuthRetryableFetchError(e) || err.message === "network timeout" || !navigator.onLine || /fetch|network/i.test(err.message);
      if (unreachable) {
        if (this.status !== "signed-in") this.status = "guest";
        this.offline = this.status === "signed-in";
        this.retry = setTimeout(() => this.init(), 60_000);
        if (!this.waitingForNetwork) {
          this.waitingForNetwork = true;
          window.addEventListener(
            "online",
            () => {
              this.waitingForNetwork = false;
              this.init();
            },
            { once: true },
          );
        }
        return;
      }
      this.offline = false;
      this.profile = null;
      this.email = null;
      writeCache(null, null);
      api.accountVaultClear().catch(() => {});
      this.status = "guest";
    }
  }

  /** Open the browser, wait for it to come back, and sign in with the result. */
  async signIn() {
    if (this.status === "signing-in") return;
    if (!navigator.onLine) {
      this.error = "You're offline. Connect to the internet to sign in.";
      return;
    }
    this.error = null;
    this.redeeming = false;
    this.status = "signing-in";
    try {
      const tokens = await api.accountBegin();
      if (!tokens) {
        if (this.status === "signing-in" && !this.redeeming) this.status = "guest";
        return;
      }
      await this.adopt(tokens);
    } catch (e) {
      this.error = describe(e);
      this.status = "guest";
    }
  }

  /** Finish with the short code the website shows, if the browser can't reach the app. */
  async redeem(code: string) {
    this.error = null;
    this.redeeming = true;
    try {
      await this.adopt(await api.accountRedeem(code));
    } catch (e) {
      this.redeeming = false;
      this.error = describe(e);
    }
  }

  cancel() {
    api.accountCancel().catch(() => {});
    if (this.status === "signing-in") this.status = "guest";
  }

  private async adopt(tokens: api.AuthSession) {
    const { data, error } = await this.client.auth.setSession(tokens);
    if (error || !data.session) throw error ?? new Error("Sign-in couldn't finish. Try again.");
    this.remember(data.session.refresh_token);
    await this.finish(data.session);
  }

  private async finish(session: Session) {
    const { data, error } = await this.client.from("profiles").select("*").eq("id", session.user.id).maybeSingle();
    if (error) {
      this.error = describe(error);
      this.status = "guest";
      return;
    }
    if (!data) {
      // Signed in but the profile was never made: send them back to the website.
      this.error = "Your profile isn't finished. Choose Sign in again and complete the last step on the website.";
      await this.signOut();
      return;
    }
    this.profile = data as Profile;
    this.email = session.user.email ?? null;
    this.offline = false;
    writeCache(this.profile, this.email);
    this.status = "signed-in";
  }

  /** Sign out of this PC only; other devices stay signed in. */
  async signOut() {
    try {
      await this.client.auth.signOut({ scope: "local" });
    } catch {
      // The local session is cleared either way.
    }
    this.lastSaved = "";
    clearTimeout(this.retry);
    api.accountVaultClear().catch(() => {});
    writeCache(null, null);
    this.profile = null;
    this.email = null;
    this.offline = false;
    this.status = "guest";
  }

  /** Is this username free? Used while typing. */
  async usernameAvailable(name: string): Promise<boolean> {
    const { data, error } = await this.client.rpc("username_available", { candidate: name });
    if (error) throw new Error(describe(error));
    return data === true;
  }

  /** Change the display name, avatar or username. Rejects with a readable message. */
  async update(patch: Partial<Pick<Profile, "display_name" | "avatar" | "username">>) {
    if (!this.profile) return;
    const { data, error } = await this.client.from("profiles").update(patch).eq("id", this.profile.id).select("*").single();
    if (error) throw new Error(describe(error));
    this.profile = data as Profile;
    writeCache(this.profile, this.email);
  }

  /** Everything Velmora holds about you, as JSON text. */
  async exportData(): Promise<string> {
    const { data, error } = await this.client.rpc("export_my_data");
    if (error) throw new Error(describe(error));
    return JSON.stringify(data, null, 2);
  }

  /** Delete the account and everything on it. This can't be undone. */
  async deleteAccount() {
    const { error } = await this.client.rpc("delete_my_account");
    if (error) throw new Error(describe(error));
    await this.signOut();
  }
}

export const account = new Account();
