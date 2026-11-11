// The YouTube Music session as the UI sees it: signed in or not, the
// account, and sign-in/sign-out. Rust owns the cookies (src-tauri/src/music.rs);
// they never reach the web side.
import { listen } from "@tauri-apps/api/event";
import * as api from "../api";
import type { MusicAccount, MusicStatus } from "../types";

class MusicSession {
  signedIn = $state(false);
  expired = $state(false);
  signingIn = $state(false);
  account = $state<MusicAccount | null>(null);
  loaded = $state(false);
  private started = false;

  private apply(s: MusicStatus) {
    const was = this.signedIn;
    this.signedIn = s.signedIn;
    this.expired = s.expired;
    this.signingIn = s.signingIn;
    if (s.account) this.account = s.account;
    if (!s.signedIn) this.account = null;
    if (s.signedIn && (!was || !this.account)) this.loadAccount();
  }

  /** Read the status once and follow changes from Rust. */
  async init() {
    if (this.started) return;
    this.started = true;
    await listen<MusicStatus>("music-status", (e) => this.apply(e.payload));
    try {
      this.apply(await api.musicStatus());
    } catch {
      // Stays signed out.
    }
    this.loaded = true;
  }

  private async loadAccount() {
    try {
      this.account = await api.musicAccount();
    } catch {
      // `music-status` reports an expired session.
    }
  }

  /** Opens Google's sign-in window. */
  signIn() {
    return api.musicSignIn();
  }

  async signOut() {
    await api.musicSignOut();
    this.signedIn = false;
    this.account = null;
  }
}

export const musicSession = new MusicSession();
