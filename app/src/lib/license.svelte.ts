// Velmora Max: whether this install is ad free. The checks themselves run on
// the Rust side (src-tauri/src/license.rs); this keeps the answer for the UI.
import { M3eSnackbar } from "@m3e/web/snackbar";
import * as api from "./api";
import { account } from "./account.svelte";
import type { LicenseStatus } from "./types";

/** Set when Max is removed from this PC on purpose, so signing in doesn't quietly bring it back. */
const DECLINED = "velmora.max.declined";
const declined = {
  get: () => {
    try {
      return localStorage.getItem(DECLINED) === "1";
    } catch {
      return false;
    }
  },
  set: (on: boolean) => {
    try {
      if (on) localStorage.setItem(DECLINED, "1");
      else localStorage.removeItem(DECLINED);
    } catch {
      // Not remembered.
    }
  },
};

const NONE: LicenseStatus = { tier: "public", state: "none", id: null, name: null, validUntil: null, launches: 0 };

class License {
  status = $state<LicenseStatus>(NONE);
  /** False until the first answer, so nothing ad related shows before we know. */
  loaded = $state(false);
  /** This key is linked to the signed-in account (known after the last check with the server). */
  linked = $state(false);
  /** Why the last account check didn't work, for the License page to show. */
  accountProblem = $state<string | null>(null);
  syncing = $state(false);
  private resolveReady!: () => void;
  /** Settles once the first answer is in, so account checks know whether this PC has Max. */
  private ready = new Promise<void>((r) => (this.resolveReady = r));

  get max() {
    return this.status.tier === "max";
  }

  async load() {
    try {
      this.status = await api.licenseStatus();
    } catch {
      // Keep the default.
    }
    this.loaded = true;
    this.resolveReady();
    // Renewing needs the network, so it runs after the first answer.
    api
      .licenseRefresh()
      .then((s) => (this.status = s))
      .catch(() => {});
  }

  /** Rejects with a message meant for the user. */
  async activate(key: string) {
    this.status = await api.licenseActivate(key);
    declined.set(false);
    // Signed in: keep it with the account too.
    this.syncAccount(true);
  }

  async remove() {
    this.status = await api.licenseRemove();
    this.linked = false;
    declined.set(true);
  }

  /**
   * Keep Max in step with the signed-in account. With Max on this PC the key is
   * linked to the account; without it, the account's key (if it has one) is
   * brought here, which moves it off any other PC. Quiet unless something happened.
   */
  async syncAccount(quiet = true) {
    await this.ready;
    if (this.syncing || !account.signedIn || account.offline) return;
    // Max was taken off this PC on purpose: only the Restore button brings it back.
    if (!this.max && quiet && declined.get()) return;
    if (!quiet) declined.set(false);
    const token = await account.accessToken();
    if (!token) return;
    this.syncing = true;
    this.accountProblem = null;
    try {
      const r = await api.licenseAccount(this.max ? "link" : "restore", token);
      this.status = r.status;
      this.linked = r.result === "linked" || r.result === "already" || (r.result === "restored" && r.status.tier === "max");
      if (r.result === "restored") M3eSnackbar.open("Welcome back. Velmora Max is on for this PC.");
      else if (r.result === "linked" && !quiet) M3eSnackbar.open("Max is linked to your account.");
    } catch (e) {
      this.accountProblem = String(e);
      if (!quiet) M3eSnackbar.open(String(e));
    } finally {
      this.syncing = false;
    }
  }

  /** Forget the link between this account and its key (the key keeps working). */
  async unlinkAccount() {
    const token = await account.accessToken();
    if (!token) return;
    const r = await api.licenseAccount("unlink", token);
    this.status = r.status;
    this.linked = false;
  }
}

export const license = new License();
