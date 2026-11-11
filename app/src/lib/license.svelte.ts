// Velmora Max: whether this install is ad free. The checks themselves run on
// the Rust side (src-tauri/src/license.rs); this keeps the answer for the UI.
import * as api from "./api";
import type { LicenseStatus } from "./types";

const NONE: LicenseStatus = { tier: "public", state: "none", id: null, name: null, validUntil: null, launches: 0 };

class License {
  status = $state<LicenseStatus>(NONE);
  /** False until the first answer, so nothing ad related shows before we know. */
  loaded = $state(false);

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
    // Renewing needs the network, so it runs after the first answer.
    api
      .licenseRefresh()
      .then((s) => (this.status = s))
      .catch(() => {});
  }

  /** Rejects with a message meant for the user. */
  async activate(key: string) {
    this.status = await api.licenseActivate(key);
  }

  async remove() {
    this.status = await api.licenseRemove();
  }
}

export const license = new License();
